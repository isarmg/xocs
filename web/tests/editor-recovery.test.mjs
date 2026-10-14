import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import {transformWithOxc} from 'vite';
import {HookHost,walk,textContent} from './fixtures/hook-host.mjs';
import {originalModule} from './fixtures/original-module.mjs';
const {FilePicker}=await originalModule(new URL('../src/FilePicker.tsx',import.meta.url));
globalThis.__EDITOR_RECOVERY_FILE_PICKER=FilePicker;
// Execute the unchanged editor declarations from current source, with only external boundaries stubbed.
const main=await readFile(new URL('../src/main.tsx',import.meta.url),'utf8');
const home=await readFile(new URL('../src/HomeSections.tsx',import.meta.url),'utf8');
const prelude=`import {articlePasswordError} from '${new URL('../src/article-password.ts',import.meta.url).href}';
import {isUploadedImage,isNewsPage} from '${new URL('../src/admin-contracts.ts',import.meta.url).href}';
import {formatUtcTime,parseUtcDate,utcDateTimeInput} from '${new URL('../src/utc-time.ts',import.meta.url).href}';
import {useState,useRef,useEffect,useMemo,jsx,jsxs,Fragment} from '${new URL('./fixtures/hook-host.mjs',import.meta.url).href}';
const h=(type,props,...children)=>jsx(type,{...props,children});
const useAdminApplication=()=>globalThis.__EDIT_APP;
const request=(...args)=>globalThis.__EDIT_APP.client.request(...args);
const t=(_zh,en,args=[])=>en.replace(/\\{(\\d+)\\}/g,(_,i)=>args[i]);
const Button='button',ErrorState='error',FormField='field',LoadingState='loading',PageHeader='header',Select='select',Table='table',TextField='input',EmptyState='empty';
const FilePicker=globalThis.__EDITOR_RECOVERY_FILE_PICKER;
const isCategories=isEditorLabels=isArticle=isNewsEntry=isDelete=isHomeSections=()=>true;

var isEditorLabels,isArticle,isNewsEntry,isDelete,isHomeSections;
const publicErrorMessage=(_reason,fallback)=>fallback;class DisplayError extends Error{}
const markdown={render:v=>v},DOMPurify={sanitize:v=>v};const formatArticleNewsTime=v=>v;const uploadFilename=v=>v;`;
const source=prelude+main.slice(main.indexOf('const emptyArticle:'),main.indexOf('function CategoryManager()'))+home.slice(home.indexOf('export function HomeSectionsPage'))+'\nexport {ArticleEditor,ArticleNewsEditor};';
const {code}=await transformWithOxc(source,'editors.tsx',{jsx:{runtime:'classic',pragma:'h',pragmaFrag:'Fragment'}});
const pages=await import(`data:text/javascript,${encodeURIComponent(code)}`);
delete globalThis.__EDITOR_RECOVERY_FILE_PICKER;
const nodes=(h,type)=>walk(h.tree,n=>n.type===type);
const settle=async h=>{await new Promise(r=>setImmediate(r));h.render();};
const submit=h=>nodes(h,'form')[0].props.onSubmit({preventDefault(){}});
const retry=(h,message)=>nodes(h,'error').find(n=>textContent(n).includes(message))?.props.onRetry;
const row={id:1,title:'Original',kind:'latest',sort_id:null,priority:0,enabled:true};
function setup(t,name,props={}){const requests=[];globalThis.window={location:{hash:''},confirm:()=>true};globalThis.__EDIT_APP={notify(){},client:{request(url,validate,options={}){let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b});requests.push({url,options,resolve,reject});return promise;}}};const host=new HookHost(pages[name],props);host.render();t.after(()=>host.unmount());return {host,requests};}
const mutations=rs=>rs.filter(r=>r.options.method);
const newsPage=items=>({items,total:items.length,page:1,size:20});
const newsDraft=(host,value)=>{nodes(host,'textarea')[0].props.onChange({target:{value}});host.render();};
test('news blocks publish until initial read succeeds and offers read-only retry',async t=>{const {host,requests}=setup(t,'ArticleNewsEditor',{id:1});newsDraft(host,'Valid draft');submit(host);assert.equal(mutations(requests).length,0);requests[0].reject(Error());await settle(host);assert.equal(typeof retry(host,'Unable to load article updates'),'function');retry(host,'Unable to load article updates')();host.render();requests.at(-1).resolve(newsPage([]));await settle(host);assert.equal(nodes(host,'textarea')[0].props.value,'Valid draft');submit(host);assert.equal(mutations(requests).length,1);});
test('news ignores aborted old-article read after switching and keeps new published row',async t=>{const {host,requests}=setup(t,'ArticleNewsEditor',{id:1});host.props={id:2};host.render();requests[1].resolve(newsPage([]));await settle(host);newsDraft(host,'Persisted new row');submit(host);const saved={id:9,content:'Persisted new row',create_time:null};mutations(requests)[0].resolve(saved);await settle(host);requests.at(-1).resolve(newsPage([saved]));await settle(host);requests[0].resolve(newsPage([{id:1,content:'Stale old row',create_time:null}]));await settle(host);assert.match(textContent(host.tree),/Persisted new row/);assert.doesNotMatch(textContent(host.tree),/Stale old row/);});
for(const failed of ['categories','labels'])test(`article retries only ${failed}, preserving draft and options`,async t=>{const {host,requests}=setup(t,'ArticleEditor');const target=requests.find(r=>r.url.endsWith(failed));requests.find(r=>r!==target).resolve(failed==='labels'?[{id:1,sort_name:'Category'}]:[{id:2,sort_id:1,label_name:'Tag'}]);target.reject(Error());await settle(host);nodes(host,'input')[0].props.onChange({target:{value:'My draft'}});nodes(host,'select')[0].props.onChange({target:{value:'1'}});host.render();nodes(host,'select')[1].props.onChange({target:{value:'2'}});host.render();const callback=retry(host,failed==='labels'?'Unable to load tags':'Unable to load categories');assert.equal(typeof callback,'function');callback();host.render();assert.equal(requests.length,3);assert.equal(requests[2].url,target.url);requests[2].resolve([]);await settle(host);assert.equal(nodes(host,'input')[0].props.value,'My draft');assert.equal(nodes(host,'select')[0].props.value,1);assert.equal(nodes(host,'select')[1].props.value,2);});
test('article create retries use one key then successful later save targets created article',async t=>{const {host,requests}=setup(t,'ArticleEditor');requests.forEach(r=>r.resolve([]));await settle(host);submit(host);submit(host);assert.equal(mutations(requests).length,1);const first=mutations(requests)[0];assert.match(first.options.headers?.['Idempotency-Key']??'',/^[0-9a-f-]{36}$/);first.reject(Error());await settle(host);submit(host);const second=mutations(requests)[1];assert.equal(second.options.headers['Idempotency-Key'],first.options.headers['Idempotency-Key']);second.resolve({id:77});await settle(host);submit(host);const third=mutations(requests)[2];assert.equal(third.options.method,'PUT');assert.equal(third.url,'/api/v1/content/articles/77');});
test('home category retry cannot refetch or overwrite dirty/saved sections',async t=>{const {host,requests}=setup(t,'HomeSectionsPage');requests[0].resolve([row]);requests[1].reject(Error());await settle(host);nodes(host,'input')[0].props.onChange({target:{value:'My draft'}});host.render();retry(host,'Unable to load categories')();host.render();assert.equal(requests.filter(r=>r.url.endsWith('home-sections')).length,1);assert.equal(nodes(host,'input')[0].props.value,'My draft');requests.at(-1).resolve([]);await settle(host);assert.equal(nodes(host,'input')[0].props.value,'My draft');});
test('home save retry repeats PUT, never GET, and duplicate clicks are locked',async t=>{const {host,requests}=setup(t,'HomeSectionsPage');requests[0].resolve([row]);requests[1].resolve([]);await settle(host);nodes(host,'input')[0].props.onChange({target:{value:'Changed'}});host.render();const save=nodes(host,'button').find(n=>textContent(n)==='Save sections').props.onClick;save();save();assert.equal(mutations(requests).length,1);mutations(requests)[0].reject(Error());await settle(host);retry(host,'Unable to save')();host.render();assert.equal(mutations(requests).length,2);assert.equal(requests.filter(r=>!r.options.method).length,2);assert.equal(JSON.parse(mutations(requests)[1].options.body)[0].title,'Changed');});
test('news pending duplicate, failed publish retry, and successful second update work normally',async t=>{const {host,requests}=setup(t,'ArticleNewsEditor',{id:1});requests[0].resolve(newsPage([]));await settle(host);newsDraft(host,'Draft');submit(host);submit(host);assert.equal(mutations(requests).length,1);mutations(requests)[0].reject(Error());await settle(host);assert.equal(nodes(host,'textarea')[0].props.value,'Draft');submit(host);assert.equal(mutations(requests)[1].options.body,mutations(requests)[0].options.body);const first={id:1,content:'Draft',create_time:null};mutations(requests)[1].resolve(first);await settle(host);requests.at(-1).resolve(newsPage([first]));await settle(host);newsDraft(host,'Next update');submit(host);const second={id:2,content:'Next update',create_time:null};mutations(requests)[2].resolve(second);await settle(host);requests.at(-1).resolve(newsPage([second,first]));await settle(host);assert.match(textContent(host.tree),/Draft/);assert.match(textContent(host.tree),/Next update/);});
test('news switching away while publish is pending ignores late mutation success',async t=>{const {host,requests}=setup(t,'ArticleNewsEditor',{id:1});requests[0].resolve(newsPage([]));await settle(host);newsDraft(host,'Old draft');submit(host);const old=mutations(requests)[0];host.props={id:2};host.render();requests.at(-1).resolve(newsPage([]));await settle(host);old.resolve({id:9,content:'Other article update',create_time:null});await settle(host);assert.doesNotMatch(textContent(host.tree),/Other article update/);assert.equal(nodes(host,'button').find(n=>textContent(n)==='Publish update').props.disabled,false);});
test('home successful category read does not hide failed section read; section retry is isolated',async t=>{const {host,requests}=setup(t,'HomeSectionsPage');requests[0].reject(Error());await settle(host);requests[1].resolve([]);await settle(host);assert.equal(typeof retry(host,'Unable to load home sections'),'function');retry(host,'Unable to load home sections')();host.render();assert.equal(requests.filter(r=>r.url.endsWith('categories')).length,1);requests.at(-1).resolve([row]);await settle(host);assert.equal(nodes(host,'input')[0].props.value,'Original');assert.equal(nodes(host,'error').length,0);});
test('home category failure and late retry cannot undo saved result',async t=>{const {host,requests}=setup(t,'HomeSectionsPage');requests[0].resolve([row]);requests[1].reject(Error());await settle(host);retry(host,'Unable to load categories')();host.render();const categoryRetry=requests.at(-1);nodes(host,'input')[0].props.onChange({target:{value:'Saved title'}});host.render();nodes(host,'button').find(n=>textContent(n)==='Save sections').props.onClick();mutations(requests)[0].resolve([{...row,title:'Saved title'}]);await settle(host);categoryRetry.resolve([]);await settle(host);assert.equal(nodes(host,'input')[0].props.value,'Saved title');assert.equal(requests.filter(r=>r.url.endsWith('home-sections')&&!r.options.method).length,1);});

test('news publish recovery keeps one key, changed drafts and deliberate later repeats get fresh keys',async t=>{
  const {host,requests}=setup(t,'ArticleNewsEditor',{id:1});requests[0].resolve(newsPage([]));await settle(host);
  newsDraft(host,'One update');submit(host);const first=mutations(requests)[0];
  const key=first.options.headers?.['Idempotency-Key'];assert.match(key??'',/^[0-9a-f-]{36}$/);
  // The server may have committed even though its response was lost.
  first.reject(new TypeError('Response lost'));await settle(host);submit(host);
  const retry=mutations(requests)[1];assert.equal(retry.options.headers['Idempotency-Key'],key);assert.equal(retry.options.body,first.options.body);
  retry.reject(new TypeError('Response lost'));await settle(host);newsDraft(host,'Changed draft');submit(host);
  const changed=mutations(requests)[2];assert.notEqual(changed.options.headers['Idempotency-Key'],key);
  const saved={id:3,content:'Changed draft',create_time:null};changed.resolve(saved);await settle(host);requests.at(-1).resolve(newsPage([saved]));await settle(host);
  newsDraft(host,'Changed draft');submit(host);const later=mutations(requests)[3];
  assert.equal(later.options.body,changed.options.body);assert.notEqual(later.options.headers['Idempotency-Key'],changed.options.headers['Idempotency-Key']);
});
