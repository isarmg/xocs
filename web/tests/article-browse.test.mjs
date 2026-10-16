import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
import { summary } from './browser-fixtures.mjs';
const reexport = path => `export * from ${JSON.stringify(new URL(path, import.meta.url).href)};`;
const { Categories, SearchPage, JourneyPage, Jotting, MessagePage } = await originalModule(new URL('./fixtures/react-baseline/src/PublicExtras.tsx', import.meta.url), {
  './api': reexport('./fixtures/react-baseline/src/api.ts'), './public-contracts': reexport('./fixtures/react-baseline/src/public-contracts.ts'),
  './comment-submission': reexport('./fixtures/react-baseline/src/comment-submission.ts'),
  './SlidingTitle': 'export const SlidingTitle=({text})=>text;',
  './ReferencePetals': 'export const ReferencePetals=()=>null;',
  './PublicChrome': 'export const PublicChrome=({children})=>children;',
  './PublicComments': 'export const PublicComments=()=>null;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './PhotoGallery': 'export const PhotoGrid=()=>null;', './LovePage': 'export const LovePage=()=>null;',
}, ['Categories', 'SearchPage', 'JourneyPage', 'Jotting', 'MessagePage']);
const text = host => textContent(expand(host.tree));
const button = (host, label) => walk(expand(host.tree), node=>node.type==='button' && textContent(node).trim()===label)[0];
const settle = async host => { await new Promise(resolve=>setImmediate(resolve)); host.render(); };
const page = (title,total=1,number=1) => ({items:title?[{...summary,article_title:title}]:[],total,page:number,size:12});
function setup(context, component, search='') {
  const old = { fetch:globalThis.fetch, window:globalThis.window };
  const requests=[];
  const listeners=new Map(), entries=[new URL(`${component===SearchPage?'/search':'/sort'}${search}`,'https://example.test')];
  let index=0;
  const history={
    get length(){return entries.length;},
    pushState(_state,_title,url){entries.splice(++index);entries.push(new URL(url,entries[index-1]));window.location=entries[index];},
    replaceState(_state,_title,url){entries[index]=new URL(url,entries[index]);window.location=entries[index];},
    back(){if(index>0){window.location=entries[--index];for(const listener of listeners.get('popstate')||[])listener();}},
    forward(){if(index<entries.length-1){window.location=entries[++index];for(const listener of listeners.get('popstate')||[])listener();}},
  };
  globalThis.window={location:entries[index],history,addEventListener(name,listener){if(!listeners.has(name))listeners.set(name,new Set());listeners.get(name).add(listener);},removeEventListener(name,listener){listeners.get(name)?.delete(listener);}};
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
for(const path of ['/jotting'])test(`${path} hides stale journal entries and retries the requested page`,async context=>{
  const {host,requests}=setup(context,Jotting);window.location.pathname=path;host.render();
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

const wallPost = (id,message) => ({id,user_id:null,username:null,avatar:null,message,image_path:null,create_time:'2026-10-11 03:30:00'});
const messageInput = host => walk(expand(host.tree),node=>node.type==='input')[0];
function editMessage(host,value){messageInput(host).props.onChange({target:{value}});host.render();}
function sendMessage(host){walk(expand(host.tree),node=>node.type==='form')[0].props.onSubmit({preventDefault(){}});host.render();}
test('message success preserves a newer draft including whitespace-only edits',async context=>{
  const {host,requests}=setup(context,MessagePage);requests[0].resolve([]);await settle(host);
  editMessage(host,'FIRST MESSAGE');sendMessage(host);
  assert.equal(JSON.parse(requests[1].options.body).message,'FIRST MESSAGE');
  assert.notEqual(messageInput(host).props.disabled,true);
  editMessage(host,'SECOND DRAFT');requests[1].resolve(wallPost(2,'FIRST MESSAGE'));await settle(host);
  assert.equal(messageInput(host).props.value,'SECOND DRAFT');
  sendMessage(host);editMessage(host,' SECOND DRAFT ');
  requests[2].resolve(wallPost(3,'SECOND DRAFT'));await settle(host);
  assert.equal(messageInput(host).props.value,' SECOND DRAFT ');
});
test('initial message load preserves a post already acknowledged by its publish response',async context=>{
  const {host,requests}=setup(context,MessagePage);
  editMessage(host,'NEW MESSAGE');sendMessage(host);
  requests[1].resolve(wallPost(2,'NEW MESSAGE'));await settle(host);
  requests[0].resolve([wallPost(1,'OLDER MESSAGE')]);await settle(host);
  assert.match(text(host),/NEW MESSAGE/);assert.match(text(host),/OLDER MESSAGE/);
});
test('initial message load and publish response render the saved ID only once',async context=>{
  const {host,requests}=setup(context,MessagePage);
  editMessage(host,'NEW MESSAGE');sendMessage(host);
  requests[0].resolve([wallPost(2,'NEW MESSAGE'),wallPost(1,'OLDER MESSAGE')]);await settle(host);
  requests[1].resolve(wallPost(2,'NEW MESSAGE'));await settle(host);
  const cards=walk(expand(host.tree),node=>node.props?.className==='message-barrage-item');
  assert.equal(cards.filter(node=>textContent(node).includes('NEW MESSAGE')).length,1);
  assert.equal(cards.length,2);
});
test('ordinary message success clears its unchanged draft and failure preserves it',async context=>{
  const {host,requests}=setup(context,MessagePage);requests[0].resolve([]);await settle(host);
  editMessage(host,' SUCCESS ');sendMessage(host);
  assert.equal(JSON.parse(requests[1].options.body).message,'SUCCESS');
  requests[1].resolve(wallPost(1,'SUCCESS'));await settle(host);
  assert.equal(messageInput(host).props.value,'');assert.match(text(host),/SUCCESS/);
  editMessage(host,' KEEP ON FAILURE ');sendMessage(host);
  requests[2].reject(new Error('offline'));await settle(host);
  assert.equal(messageInput(host).props.value,' KEEP ON FAILURE ');
  assert.match(text(host),/Unable to send/);assert.equal(button(host,'Send').props.disabled,false);
});

for(const component of [Categories,SearchPage,JourneyPage,Jotting])test(`${component.name} restores its requested page from the URL`,context=>{
  const {requests}=setup(context,component,'?q=first&sort_id=1&label_id=2&page=2&lang=zh-CN');
  assert.match(requests.at(-1).url,/page=2/);
});
test('search keeps history, page and explicit language through repeated searches',async context=>{
  const {host,requests}=setup(context,SearchPage,'?q=first&lang=zh-CN');
  requests.at(-1).resolve(page('FIRST',25));await settle(host);
  button(host,'Next page').props.onClick();host.render();
  assert.equal(new URLSearchParams(window.location.search).get('page'),'2');
  requests.at(-1).resolve(page('SECOND_PAGE',25,2));await settle(host);
  submit(host,'second');
  assert.equal(new URLSearchParams(window.location.search).get('lang'),'zh-CN');
  assert.equal(window.history.length,3);
  window.history.back();host.render();
  assert.match(requests.at(-1).url,/search=first&page=2/);
  assert.equal(walk(host.tree,node=>node.type==='input')[0].props.value,'first');
  window.history.forward();host.render();
  assert.match(requests.at(-1).url,/search=second&page=1/);
});
test('category and tag choices keep prior filters and explicit language in history',async context=>{
  const {host,requests}=setup(context,Categories,'?sort_id=1&lang=zh-CN');
  requests.find(r=>r.url==='/api/v1/categories').resolve([{id:1,sort_name:'A',sort_description:null,priority:0,article_count:25},{id:2,sort_name:'B',sort_description:null,priority:0,article_count:1}]);
  requests.find(r=>r.url.includes('/labels')).resolve([{id:2,sort_id:1,label_name:'TAG',label_description:null,article_count:25}]);
  requests.at(-1).resolve(page('FIRST',25));await settle(host);
  walk(expand(host.tree),node=>node.type==='button'&&textContent(node).startsWith('TAG '))[0].props.onClick();host.render();
  assert.equal(new URLSearchParams(window.location.search).get('lang'),'zh-CN');
  requests.at(-1).resolve(page('TAG_PAGE',25));await settle(host);
  button(host,'Next page').props.onClick();host.render();
  requests.at(-1).resolve(page('TAG_PAGE_TWO',25,2));await settle(host);
  walk(expand(host.tree),node=>node.type==='button'&&textContent(node).startsWith('B '))[0].props.onClick();host.render();
  assert.equal(window.history.length,4);
  assert.doesNotMatch(text(host),/TAG_PAGE_TWO/);
  assert.match(requests.at(-1).url,/page=1&size=12&sort_id=2$/);
  const obsolete=requests.at(-1);
  window.history.back();host.render();
  assert.match(requests.at(-1).url,/page=2&size=12&sort_id=1&label_id=2/);
  requests.at(-1).resolve(page('RESTORED_PAGE_TWO',25,2));await settle(host);
  obsolete.resolve(page('OBSOLETE_B'));await settle(host);
  assert.match(text(host),/RESTORED_PAGE_TWO/);assert.doesNotMatch(text(host),/OBSOLETE_B/);
});

test('editing a search draft and retrying the same query do not add history entries',async context=>{
  const {host,requests}=setup(context,SearchPage,'?q=first&lang=en');
  walk(host.tree,node=>node.type==='input')[0].props.onChange({target:{value:'typing'}});host.render();
  assert.equal(window.history.length,1);assert.equal(requests.length,1);
  submit(host,'first');assert.equal(window.history.length,1);assert.equal(requests.length,2);
});
for(const component of [JourneyPage,Jotting])test(`${component.name} keeps pagination in history and ignores an abandoned response`,async context=>{
  const {host,requests}=setup(context,component,'?lang=zh-CN');
  const result=component===JourneyPage?journeyPage:notePage;
  requests[0].resolve(component===JourneyPage?result('PAGE_ONE',41):result('PAGE_ONE'));await settle(host);
  const next=button(host,'Next page');next.props.onClick();next.props.onClick();host.render();
  assert.equal(new URLSearchParams(window.location.search).get('page'),'2');
  assert.equal(new URLSearchParams(window.location.search).get('lang'),'zh-CN');
  assert.equal(window.history.length,2);
  const abandoned=requests.at(-1);
  window.history.back();host.render();
  assert.match(requests.at(-1).url,/page=1/);
  requests.at(-1).resolve(component===JourneyPage?result('RESTORED_ONE',41):result('RESTORED_ONE'));await settle(host);
  abandoned.resolve(component===JourneyPage?result('ABANDONED_TWO',41,2):result('ABANDONED_TWO',2));await settle(host);
  assert.match(text(host),/RESTORED_ONE/);assert.doesNotMatch(text(host),/ABANDONED_TWO/);
});

for (const [component,size] of [[JourneyPage,20],[Jotting,10],[Categories,12],[SearchPage,12]]) test(`${component.name} keeps previous-page recovery when the restored page is now empty`,async context=>{
  const {host,requests}=setup(context,component,'?q=needle&page=2&lang=en');
  requests.at(-1).resolve({items:[],total:0,page:2,size});await settle(host);
  assert.equal(button(host,'Previous page')?.props.disabled,false,'An empty page must retain its existing previous-page navigation.');
  assert.equal(button(host,'Next page')?.props.disabled,true);
  button(host,'Previous page').props.onClick();host.render();
  assert.match(requests.at(-1).url,/page=1/);
  assert.equal(new URLSearchParams(window.location.search).get('lang'),'en');
});

test('timeline shrunk to one page still permits reaching the remaining article',async context=>{
  const {host,requests}=setup(context,JourneyPage,'?page=2');
  requests[0].resolve({items:[],total:1,page:2,size:20});await settle(host);
  assert.ok(button(host,'Previous page'));
  button(host,'Previous page').props.onClick();host.render();
  requests.at(-1).resolve({items:[{...summary,article_title:'Remaining story'}],total:1,page:1,size:20});await settle(host);
  assert.match(text(host),/Remaining story/);
  assert.equal(button(host,'Previous page'),undefined);
});

for(const [input,expected] of [['0',1],['-1',1],['2.5',1],['abc',1],['Infinity',1],['100001',100000],['9007199254740992',1]]) test(`page input ${input} normalizes to ${expected}`,context=>{
  const {requests}=setup(context,SearchPage,`?q=needle&page=${input}`);
  assert.equal(new URL(requests[0].url,'https://example.test').searchParams.get('page'),String(expected));
});

test('recommendation selection resets category, tag and page while keeping language',async context=>{
  const {host,requests}=setup(context,Categories,'?sort_id=2&label_id=3&page=2&lang=en');
  button(host,'Recommended').props.onClick();host.render();
  const query=new URLSearchParams(window.location.search);
  assert.equal(query.get('recommended'),'1');
  for(const name of ['sort_id','label_id','page'])assert.equal(query.get(name),null);
  assert.equal(query.get('lang'),'en');
  assert.match(requests.at(-1).url,/page=1&size=12&recommended=true$/);
});

for(const [component,size] of [[JourneyPage,20],[Jotting,10],[Categories,12]]) {
  test(`${component.name} keeps an empty first page free of unnecessary paging controls`,async context=>{
    const {host,requests}=setup(context,component);
    const count=requests.length;
    requests.at(-1).resolve({items:[],total:0,page:1,size});await settle(host);
    assert.equal(button(host,'Previous page'),undefined);
    assert.equal(button(host,'Next page'),undefined);
    assert.equal(requests.length,count);
  });
  for(const total of [0,1]) test(`${component.name} preserves a high restored page after total shrinks to ${total}`,async context=>{
    const {host,requests}=setup(context,component,'?page=100000&lang=en');
    const count=requests.length;
    requests.at(-1).resolve({items:[],total,page:100000,size});await settle(host);
    assert.equal(requests.length,count,'Settling an empty page must not trigger an automatic replacement request.');
    assert.equal(new URLSearchParams(window.location.search).get('page'),'100000');
    assert.equal(button(host,'Previous page')?.props.disabled,false);
    assert.equal(button(host,'Next page')?.props.disabled,true);
    button(host,'Previous page').props.onClick();host.render();
    assert.match(requests.at(-1).url,/page=99999/);
    assert.equal(new URLSearchParams(window.location.search).get('page'),'99999');
    assert.equal(new URLSearchParams(window.location.search).get('lang'),'en');
  });
}
