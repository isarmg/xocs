import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
import { summary } from './browser-fixtures.mjs';
const reexport = path => `export * from ${JSON.stringify(new URL(path, import.meta.url).href)};`;
const { Categories, SearchPage, JourneyPage, Wall } = await originalModule(new URL('../src/PublicExtras.tsx', import.meta.url), {
  './api': reexport('../src/api.ts'), './public-contracts': reexport('../src/public-contracts.ts'),
  './comment-submission': reexport('../src/comment-submission.ts'),
  './PublicChrome': 'export const PublicChrome=({children})=>children;',
  './PublicComments': 'export const PublicComments=()=>null;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './PhotoGallery': 'export const PhotoGrid=()=>null;', './LovePage': 'export const LovePage=()=>null;',
}, ['Categories', 'SearchPage', 'JourneyPage', 'Wall']);
const text = host => textContent(expand(host.tree));
const button = (host, label) => walk(expand(host.tree), node=>node.type==='button' && textContent(node).trim()===label)[0];
const settle = async host => { await new Promise(resolve=>setImmediate(resolve)); host.render(); };
const page = (title,total=1,number=1) => ({items:title?[{...summary,article_title:title}]:[],total,page:number,size:12});
function setup(context, component, search='') {
  const old = { fetch:globalThis.fetch, window:globalThis.window };
  const requests=[];
  globalThis.window={location:{search,origin:'https://example.test'},history:{replaceState(){}}};
  globalThis.fetch=(url,options={})=>new Promise((resolve,reject)=>requests.push({url:String(url),options,reject,resolve(value){resolve(new Response(JSON.stringify(value),{headers:{'Content-Type':'application/json'}}));}}));
  const host=new HookHost(component);context.after(()=>{host.unmount();Object.assign(globalThis,old);});host.render();return {host,requests};
}
function submit(host, term) {
  walk(host.tree,node=>node.type==='input')[0].props.onChange({target:{value:term}});host.render();
  walk(host.tree,node=>node.type==='form')[0].props.onSubmit({preventDefault(){}});host.render();
}
test('failed search can resubmit the same query and recover without stale errors',async context=>{
  const {host,requests}=setup(context,SearchPage,'?q=first');
  requests.at(-1).reject(new Error('offline'));await settle(host);
  assert.match(text(host),/Search failed/);
  submit(host,'first');assert.equal(requests.length,2);
  requests.at(-1).resolve(page('Recovered'));await settle(host);
  assert.match(text(host),/Recovered/);assert.doesNotMatch(text(host),/Search failed/);
});
test('changing a search hides old results and ignores obsolete responses',async context=>{
  const {host,requests}=setup(context,SearchPage,'?q=first');requests.at(-1).resolve(page('OLD_RESULT'));await settle(host);
  submit(host,'second');assert.doesNotMatch(text(host),/OLD_RESULT/);const obsolete=requests.at(-1);
  submit(host,'third');requests.at(-1).resolve(page('CURRENT_RESULT'));await settle(host);
  obsolete.resolve(page('OBSOLETE_RESULT'));await settle(host);
  assert.match(text(host),/CURRENT_RESULT/);assert.doesNotMatch(text(host),/OLD_RESULT|OBSOLETE_RESULT/);
});
test('category changes hide old results and old tags and expose retry after interruption',async context=>{
  const {host,requests}=setup(context,Categories,'?sort_id=1');
  requests.find(r=>r.url==='/api/v1/categories').resolve([{id:1,sort_name:'A',sort_description:null,priority:0,article_count:1},{id:2,sort_name:'B',sort_description:null,priority:0,article_count:1}]);
  requests.find(r=>r.url.includes('/labels')).resolve([{id:1,sort_id:1,label_name:'OLD_TAG',label_description:null,article_count:1}]);
  requests.at(-1).resolve(page('OLD_RESULT'));await settle(host);
  const chooseB=walk(expand(host.tree),node=>node.type==='button'&&textContent(node).startsWith('B '))[0];chooseB.props.onClick();host.render();
  assert.doesNotMatch(text(host),/OLD_RESULT|OLD_TAG/);
  requests.at(-1).reject(new Error('offline'));await settle(host);
  assert.ok(button(host,'Try again'));button(host,'Try again').props.onClick();host.render();requests.at(-1).resolve(page('NEW_RESULT'));await settle(host);
  assert.match(text(host),/NEW_RESULT/);assert.doesNotMatch(text(host),/Unable to load articles/);
});

const journeyPage = (title,total=41,number=1) => ({...page(title,total,number),size:20});
test('timeline hides the preceding page and retries a failed next page without stale errors',async context=>{
  const {host,requests}=setup(context,JourneyPage);
  requests[0].resolve(journeyPage('PAGE_ONE'));await settle(host);
  button(host,'Next page').props.onClick();host.render();
  assert.doesNotMatch(text(host),/PAGE_ONE/);assert.match(text(host),/Loading articles/);
  requests[1].reject(new Error('offline'));await settle(host);
  assert.doesNotMatch(text(host),/PAGE_ONE/);assert.match(text(host),/Unable to load travels/);
  button(host,'Try again').props.onClick();host.render();
  assert.match(requests[2].url,/page=2&size=20/);
  requests[2].resolve(journeyPage('PAGE_TWO',41,2));await settle(host);
  assert.match(text(host),/PAGE_TWO/);assert.match(text(host),/Page 2/);
  assert.doesNotMatch(text(host),/Unable to load travels|PAGE_ONE/);
});
test('timeline repeated same-render Next callbacks advance only one page',async context=>{
  const {host,requests}=setup(context,JourneyPage);
  requests[0].resolve(journeyPage('PAGE_ONE',61));await settle(host);
  const next=button(host,'Next page');next.props.onClick();next.props.onClick();host.render();
  assert.equal(requests.length,2);assert.match(requests[1].url,/page=2&size=20/);
  assert.equal(button(host,'Next page'),undefined);
  requests[1].resolve(journeyPage('PAGE_TWO',61,2));await settle(host);
  assert.match(text(host),/PAGE_TWO/);assert.match(text(host),/Page 2/);
});

const notePage = (content,number=1) => ({items:[{id:1,user_id:1,username:'Owner',content,image_path:null,like_count:0,is_public:1,create_time:'2026-10-11 00:00:00'}],total:21,page:number,size:10});
for(const path of ['/weiYan','/jotting'])test(`${path} hides stale journal entries and retries the requested page`,async context=>{
  const {host,requests}=setup(context,Wall);window.location.pathname=path;host.render();
  requests[0].resolve(notePage('OLD_NOTE'));await settle(host);
  const next=button(host,'Next page');next.props.onClick();next.props.onClick();host.render();
  assert.equal(requests.length,2);assert.match(requests[1].url,/page=2$/);
  assert.doesNotMatch(text(host),/OLD_NOTE/);assert.equal(button(host,'Next page'),undefined);
  requests[1].reject(new Error('offline'));await settle(host);
  assert.doesNotMatch(text(host),/OLD_NOTE/);assert.match(text(host),/Unable to load content/);
  button(host,'Try again').props.onClick();host.render();
  assert.match(requests[2].url,/page=2$/);
  requests[2].resolve(notePage('NEW_NOTE',2));await settle(host);
  assert.match(text(host),/NEW_NOTE/);assert.match(text(host),/Page 2/);
  assert.doesNotMatch(text(host),/OLD_NOTE|Unable to load content/);
});
