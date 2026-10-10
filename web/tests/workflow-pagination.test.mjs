import uploadNameFixtures from './fixtures/upload-names.json' with { type: 'json' };
import { uploadFilename } from '../src/upload-filename.ts';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
import { request } from '../src/api.ts';
import { article, summary } from './browser-fixtures.mjs';

const reexport = path => `export * from ${JSON.stringify(new URL(path, import.meta.url).href)};`;
const boundaries = {
  './api': reexport('../src/api.ts'),
  './public-contracts': reexport('../src/public-contracts.ts'),
  './comment-submission': reexport('../src/comment-submission.ts'),
  './PublicChrome': 'export const PublicChrome=({children})=>children;',
  './PublicComments': 'export const PublicComments=()=>null;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './PhotoGallery': 'export const PhotoGrid=({items})=>items.map(item=>item.title);',
  './LovePage': 'export const LovePage=()=>null;',
};
const { CommentReplies } = await originalModule(new URL('../src/PublicComments.tsx', import.meta.url), boundaries, ['CommentReplies']);
const { PublicExtras } = await originalModule(new URL('../src/PublicExtras.tsx', import.meta.url), boundaries);
const { LovePage } = await originalModule(new URL('../src/LovePage.tsx', import.meta.url), boundaries);
const { LabelsPage, SitePage, ResourcesPage } = await originalModule(new URL('../src/AdminExtras.tsx', import.meta.url), { ...boundaries, '@xcss/web/admin-shell':'export const useAdminApplication=()=>globalThis.__WORKFLOW_APP;', './AdminLayout':'export const adminGroups=[];' });
const { HomeSectionsPage } = await originalModule(new URL('../src/HomeSections.tsx', import.meta.url), { ...boundaries, '@xcss/web/admin-shell':'export const useAdminApplication=()=>globalThis.__WORKFLOW_APP;' });
const beforeImportWindow = globalThis.window;
globalThis.window = { location: { pathname: '/admin' } };
const { ArticleManager, ArticleEditor, ArticleNewsEditor, CategoryManager } = await originalModule(new URL('../src/main.tsx', import.meta.url), {
  ...boundaries,
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__WORKFLOW_APP;export const createXcssAdminApplication=()=>()=>null;export const InstanceHeaderActions=()=>null;export const AccountPage=()=>null;',
  'react-dom/client': 'export const createRoot=()=>({render(){}});',
  '@xcss/web/web-fonts': 'export const startAfterFonts=()=>{};',
  './AdminLayout': 'export const AdminWorkspace=({children})=>children;',
  './AdminExtras': ['CommentsPage','Dashboard','FamilyPage','LabelsPage','LinksPage','NotesPage','ResourcesPage','SitePage','TreeHolePage','UsersPage'].map(name=>`export const ${name}=()=>null;`).join(''),
  './PublicExtras': 'export const PublicExtras=()=>null;',
  './HomeSections': 'export const HomeSectionsPage=()=>null;',
  './ArticleNews': 'export const ArticleNews=()=>null;',
  './language': 'export const languageHref=x=>x;export const localizeValidation=()=>{};export const clearValidation=()=>{};export const preserveLinkLanguage=()=>{};',
  'markdown-it': 'export default class MarkdownIt {render(value){return value;}}',
  'dompurify': 'export default {sanitize:value=>value};',
  ...Object.fromEntries(['@xcss/web/design-tokens/tokens.css','@xcss/web/design-tokens/tokens.dark.css','@xcss/web/admin-ui/styles.css','@xcss/web/design-tokens/reset.css','@xcss/web/design-tokens/accessibility.css','@xcss/web/web-fonts/fonts.css','./style.css','./public-original.css','./public-layout.css','./admin.css'].map(path=>[path,''])),
}, ['ArticleManager', 'ArticleEditor', 'ArticleNewsEditor', 'CategoryManager']);
globalThis.window = beforeImportWindow;

const page = (items, total, number = 1, size = 5) => ({ items, total, page: number, size });
const comment = id => ({ id, user_id:null, username:null, avatar:null, comment_content:`reply-${id}`, create_time:null, parent_username:null, reply_count:0 });
const photo = id => ({ id, title:`photo-${id}`, classify:'A', cover:null, url:null, introduction:null, link_type:'lovePhoto', create_time:null });
const photos = (start, end) => Array.from({length:end-start+1}, (_,index)=>photo(start+index));
const text = host => textContent(expand(host.tree));
const button = (host, label) => walk(expand(host.tree), node=>node.type==='button' && textContent(node).replace(/\s+/g,' ').includes(label))[0];
const settle = async host => { await new Promise(resolve=>setImmediate(resolve)); host.render(); };
function setup(context, component, props = {}, path = '/admin') {
  const previous = { fetch:globalThis.fetch, window:globalThis.window, app:globalThis.__WORKFLOW_APP };
  const requests = [], notifications = [];
  globalThis.window = { location:{pathname:path, search:'', origin:'https://example.test', hash:''}, history:{replaceState(){}}, confirm:()=>true, setInterval:()=>1, clearInterval(){} };
  globalThis.fetch = (url, options={}) => new Promise((resolve,reject)=>requests.push({url:String(url), options, reject, resolve(value,status=200){resolve(new Response(JSON.stringify(value),{status,headers:{'Content-Type':'application/json'}}));}}));
  globalThis.__WORKFLOW_APP = {client:{request}, notify:value=>notifications.push(value)};
  const element = component ? {type:component, props} : PublicExtras();
  const host = new HookHost(element.type, element.props);
  context.after(()=>{host.unmount();globalThis.fetch=previous.fetch;globalThis.window=previous.window;globalThis.__WORKFLOW_APP=previous.app;});
  host.render();
  return {host, requests, notifications};
}

test('reply append pagination admits one click and retries the failed page without losing replies', async context=>{
  const {host,requests}=setup(context,CommentReplies,{root:{id:1,reply_count:11},refresh:0,kind:'article',onReply(){}});
  requests.at(-1).resolve(page([1,2,3,4,5].map(comment),11));await settle(host);
  const next=button(host,'more replies');next.props.onClick();next.props.onClick();host.render();
  assert.equal(requests.length,2);assert.match(requests.at(-1).url,/page=2/);assert.equal(button(host,'more replies').props.disabled,true);
  requests.at(-1).reject(new Error('ordinary connection failure'));await settle(host);
  button(host,'more replies').props.onClick();host.render();assert.match(requests.at(-1).url,/page=2/);
  requests.at(-1).resolve(page([6,7,8,9,10].map(comment),11,2));await settle(host);
  button(host,'more replies').props.onClick();host.render();requests.at(-1).resolve(page([comment(11)],11,3));await settle(host);
  for(let id=1;id<=11;id++)assert.match(text(host),new RegExp(`reply-${id}\\b`));
  assert.equal(button(host,'more replies'),undefined);
});

test('reply refresh discards an obsolete append response and retries an initially failed first page', async context=>{
  const {host,requests}=setup(context,CommentReplies,{root:{id:1,reply_count:11},refresh:0,kind:'article',onReply(){}});
  requests.at(-1).reject(new Error('offline'));await settle(host);button(host,'Try again').props.onClick();host.render();requests.at(-1).resolve(page([1,2,3,4,5].map(comment),11));await settle(host);
  button(host,'more replies').props.onClick();host.render();const obsolete=requests.at(-1);
  host.props={...host.props,root:{id:2,reply_count:1}};host.render();assert.equal(obsolete.options.signal.aborted,true);
  requests.at(-1).resolve(page([comment(20)],1));await settle(host);obsolete.resolve(page([6,7,8,9,10].map(comment),11,2));await settle(host);
  assert.match(text(host),/reply-20/);assert.doesNotMatch(text(host),/reply-[1-9]\b/);
});

async function travel(context) {
  const fixture=setup(context,null,{},'/travel');
  fixture.requests.find(item=>item.url.includes('/classes')).resolve([{classify:'A',count:25},{classify:'B',count:1}]);await settle(fixture.host);
  fixture.requests.at(-1).resolve(page(photos(1,12),25,1,12));await settle(fixture.host);return fixture;
}
test('travel photo append blocks rapid clicks and retries the same interrupted page',async context=>{
  const {host,requests}=await travel(context);const next=button(host,'Next page');const before=requests.length;
  next.props.onClick();next.props.onClick();host.render();assert.equal(requests.length,before+1);assert.match(requests.at(-1).url,/page=2/);
  requests.at(-1).reject(new Error('temporary failure'));await settle(host);button(host,'Next page').props.onClick();host.render();assert.match(requests.at(-1).url,/page=2/);
  requests.at(-1).resolve(page(photos(13,24),25,2,12));await settle(host);button(host,'Next page').props.onClick();host.render();requests.at(-1).resolve(page([photo(25)],25,3,12));await settle(host);
  for(let id=1;id<=25;id++)assert.match(text(host),new RegExp(`photo-${id}\\b`));assert.equal(button(host,'Next page'),undefined);
});
test('travel category changes discard old append responses and selecting the active category preserves photos',async context=>{
  const {host,requests}=await travel(context);button(host,'A 25').props.onClick();host.render();assert.match(text(host),/photo-1\b/);
  button(host,'Next page').props.onClick();host.render();const obsolete=requests.at(-1);button(host,'B 1').props.onClick();host.render();assert.equal(obsolete.options.signal.aborted,true);
  requests.at(-1).resolve(page([photo(30)],1,1,12));await settle(host);obsolete.resolve(page(photos(13,24),25,2,12));await settle(host);
  assert.match(text(host),/photo-30/);assert.doesNotMatch(text(host),/photo-1\b|photo-13\b/);
});
async function love(context) {
  const fixture=setup(context,LovePage);fixture.requests.find(item=>item.url.endsWith('/family')).resolve([]);
  fixture.requests.find(item=>item.url.includes('/classes')).resolve([{classify:'A',count:25}]);fixture.requests.at(-1).resolve(page(photos(1,12),25,1,12));await settle(fixture.host);return fixture;
}
test('love photos preserve the active filter and retry a failed append before advancing',async context=>{
  const {host,requests}=await love(context);button(host,'All').props.onClick();host.render();assert.match(text(host),/photo-1\b/);
  const before=requests.length,next=button(host,'Load more photos');next.props.onClick();next.props.onClick();host.render();assert.equal(requests.length,before+1);
  requests.at(-1).reject(new Error('temporary failure'));await settle(host);button(host,'Load more photos').props.onClick();host.render();assert.match(requests.at(-1).url,/page=2/);
  requests.at(-1).resolve(page(photos(13,24),25,2,12));await settle(host);button(host,'Load more photos').props.onClick();host.render();requests.at(-1).resolve(page([photo(25)],25,3,12));await settle(host);
  for(let id=1;id<=25;id++)assert.match(text(host),new RegExp(`photo-${id}\\b`));assert.equal(button(host,'Load more photos'),undefined);
});
test('love photo filters ignore an obsolete append response',async context=>{
  const {host,requests}=await love(context);button(host,'Load more photos').props.onClick();host.render();const obsolete=requests.at(-1);
  button(host,'A 25').props.onClick();host.render();requests.at(-1).resolve(page([photo(30)],1,1,12));await settle(host);obsolete.resolve(page(photos(13,24),25,2,12));await settle(host);
  assert.match(text(host),/photo-30/);assert.doesNotMatch(text(host),/photo-13\b/);
});

test('article deletion on the sole last-page row converges to a real remaining page',async context=>{
  const {host,requests}=setup(context,ArticleManager);requests.at(-1).resolve(page([summary],16,1,15));await settle(host);
  button(host,'Next page').props.onClick();host.render();requests.at(-1).resolve(page([{...summary,id:16,article_title:'Last row'}],16,2,15));await settle(host);
  button(host,'Delete').props.onClick();requests.at(-1).resolve({deleted:true});await settle(host);requests.at(-1).resolve(page([],15,2,15));await settle(host);
  assert.match(requests.at(-1).url,/page=1/);assert.doesNotMatch(text(host),/No articles yet/);
  requests.at(-1).resolve(page([{...summary,article_title:'Remaining article'}],15,1,15));await settle(host);assert.match(text(host),/Remaining article/);assert.equal(button(host,'Previous page'),undefined);
});
test('deleting the only article shows the real empty collection after refreshing',async context=>{
  const {host,requests}=setup(context,ArticleManager);requests.at(-1).resolve(page([summary],1,1,15));await settle(host);button(host,'Delete').props.onClick();requests.at(-1).resolve({deleted:true});await settle(host);requests.at(-1).resolve(page([],0,1,15));await settle(host);assert.match(text(host),/No articles yet/);assert.equal(button(host,'Previous page'),undefined);
});

async function editor(context) {
  const fixture=setup(context,ArticleEditor);fixture.requests[0].resolve([{id:1,sort_name:'A',sort_description:null,priority:0,article_count:0}]);fixture.requests[1].resolve([{id:1,sort_id:1,label_name:'Travel',label_description:null}]);await settle(fixture.host);
  const host=fixture.host;walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='TextField')[0].props.onChange({target:{value:'Owned draft'}});
  walk(host.tree,node=>node.type==='textarea')[0].props.onChange({target:{value:'Original text'}});
  const selects=walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='Select');selects[0].props.onChange({target:{value:'1'}});selects[1].props.onChange({target:{value:'1'}});host.render();return fixture;
}
const upload=(host,index=0)=>{
  const target={files:[{name:'owned.png',size:5,type:'image/png'}],value:'C:\\fakepath\\owned.png'};
  walk(host.tree,node=>node.type==='input'&&node.props.type==='file')[index].props.onChange({target});
  assert.equal(target.value,'','The same file must remain selectable after a failed upload.');
};
const save=host=>walk(host.tree,node=>node.type==='form')[0].props.onSubmit({preventDefault(){}});
test('article save waits for the selected upload and includes its confirmed image path',async context=>{
  const {host,requests}=await editor(context);upload(host);const before=requests.length;save(host);host.render();assert.equal(requests.length,before);assert.equal(button(host,'Uploading image').props.disabled,true);
  requests.at(-1).resolve({path:'/media/owned.png'});await settle(host);assert.equal(button(host,'Save article').props.disabled,false);save(host);assert.equal(JSON.parse(requests.at(-1).options.body).article_cover,'/media/owned.png');requests.at(-1).resolve(article);await settle(host);assert.equal(window.location.hash,'articles');
});
test('failed image uploads preserve the draft and allow a deliberate retry',async context=>{
  const {host,requests}=await editor(context);upload(host);requests.at(-1).reject(new Error('ordinary connection failure'));await settle(host);assert.match(text(host),/Image upload failed/);assert.equal(button(host,'Save article').props.disabled,false);
  assert.equal(walk(host.tree,node=>node.type==='textarea')[0].props.value,'Original text');upload(host);requests.at(-1).resolve({path:'/media/retry.png'});await settle(host);save(host);assert.equal(JSON.parse(requests.at(-1).options.body).article_cover,'/media/retry.png');
});
test('canceling or leaving the editor ignores late upload success',async context=>{
  const {host,requests,notifications}=await editor(context);upload(host);const obsolete=requests.at(-1);button(host,'Cancel').props.onClick();host.render();assert.equal(obsolete.options.signal.aborted,true);host.unmount();obsolete.resolve({path:'/media/late.png'});await new Promise(resolve=>setImmediate(resolve));assert.deepEqual(notifications,[]);
});
test('content upload preserves edits made while waiting; stale save success cannot navigate a new editor',async context=>{
  const {host,requests}=await editor(context);upload(host,1);host.render();walk(host.tree,node=>node.type==='textarea')[0].props.onChange({target:{value:'Continued writing'}});host.render();requests.at(-1).resolve({path:'/media/content.png'});await settle(host);save(host);const saving=requests.at(-1);assert.match(JSON.parse(saving.options.body).article_content,/Continued writing[\s\S]*\/media\/content.png/);
  host.unmount();window.location.hash='new';saving.resolve(article);await new Promise(resolve=>setImmediate(resolve));assert.equal(window.location.hash,'new');
});


test('canceled selections send nothing and canceled uploads cannot publish a late failure',async context=>{
  const {host,requests,notifications}=await editor(context);const before=requests.length;
  walk(host.tree,node=>node.type==='input'&&node.props.type==='file')[0].props.onChange({target:{files:[],value:''}});
  assert.equal(requests.length,before);upload(host);const obsolete=requests.at(-1);button(host,'Cancel').props.onClick();host.render();obsolete.reject(new Error('late rejected upload'));await settle(host);
  assert.doesNotMatch(text(host),/Image upload failed/);assert.deepEqual(notifications,[]);
});
test('a blocked in-use label move explains the conflict without exposing server diagnostics',async context=>{
  const {host,requests}=setup(context,LabelsPage);requests[0].resolve([{id:1,sort_id:1,label_name:'Used',label_description:null}]);requests[1].resolve([{id:1,sort_name:'A'},{id:2,sort_name:'B'}]);await settle(host);
  button(host,'Edit').props.onClick();host.render();walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='Select')[0].props.onChange({target:{value:'2'}});host.render();
  walk(host.tree,node=>node.type==='form')[0].props.onSubmit({preventDefault(){}});requests.at(-1).resolve({code:'conflict',message:'PRIVATE SERVER DETAIL',retryable:false},409);await settle(host);
  assert.match(text(host),/This tag is still used by articles/);assert.doesNotMatch(text(host),/PRIVATE SERVER DETAIL/);
});

test('travel retries the failed category request without advancing successful photos',async context=>{
  const {host,requests}=setup(context,null,{},'/travel');const categories=requests.find(item=>item.url.includes('/classes'));
  requests.find(item=>item.url.includes('/page')).resolve(page(photos(1,12),25,1,12));await settle(host);
  categories.reject(new Error('categories temporarily unavailable'));await settle(host);assert.match(text(host),/Unable to load photo categories/);
  const before=requests.length,retry=button(host,'Retry categories');retry.props.onClick();retry.props.onClick();host.render();assert.equal(requests.length,before+1);assert.match(requests.at(-1).url,/\/classes\?/);assert.equal(button(host,'Next page').props.disabled,false);assert.match(text(host),/photo-1\b/);
  requests.at(-1).reject(new Error('still offline'));await settle(host);button(host,'Retry categories').props.onClick();host.render();assert.match(requests.at(-1).url,/\/classes\?/);
  requests.at(-1).resolve([{classify:'A',count:25}]);await settle(host);assert.match(text(host),/A\s+25/);assert.match(requests.at(-1).url,/page=1.*classify=A/);assert.doesNotMatch(text(host),/Unable to load photo categories/);
});
test('travel photo completion cannot erase a separate category failure',async context=>{
  const {host,requests}=setup(context,null,{},'/travel');requests.find(item=>item.url.includes('/classes')).reject(new Error('categories offline'));await settle(host);
  requests.find(item=>item.url.includes('/page')).resolve(page(photos(1,12),25,1,12));await settle(host);assert.match(text(host),/Unable to load photo categories/);assert.match(text(host),/photo-1\b/);
});

test('existing article editing waits for load and retries failures before enabling uploads or saves',async context=>{
  const {host,requests}=setup(context,ArticleEditor,{id:1});const initial=requests.find(item=>item.url.endsWith('/articles/1'));
  assert.equal(walk(host.tree,node=>node.type==='form').length,0);assert.equal(walk(host.tree,node=>node.type==='input'&&node.props.type==='file').length,0);assert.ok(button(host,'Cancel'));
  initial.reject(new Error('temporary load failure'));await settle(host);assert.match(text(host),/Unable to load article/);assert.equal(walk(host.tree,node=>node.type==='form').length,0);
  button(host,'Try again').props.onClick();host.render();assert.match(requests.at(-1).url,/\/articles\/1$/);assert.equal(walk(host.tree,node=>node.type==='input'&&node.props.type==='file').length,0);
  requests.at(-1).resolve({...article,article_cover:'/media/old.png'});await settle(host);assert.equal(walk(host.tree,node=>node.type==='input'&&node.props.type==='file').length,2);
  upload(host);requests.at(-1).resolve({path:'/media/new.png'});await settle(host);save(host);assert.equal(JSON.parse(requests.at(-1).options.body).article_cover,'/media/new.png');assert.equal(requests.at(-1).options.method,'PUT');
});
test('article identity changes discard stale initial loads and pending uploads',async context=>{
  const {host,requests}=setup(context,ArticleEditor,{id:1});const obsolete=requests.find(item=>item.url.endsWith('/articles/1'));
  host.props={id:2};host.render();assert.equal(obsolete.options.signal.aborted,true);const current=requests.at(-1);obsolete.resolve({...article,article_title:'Obsolete title'});await settle(host);assert.equal(walk(host.tree,node=>node.type==='form').length,0);
  current.resolve({...article,id:2,article_title:'Current title'});await settle(host);assert.equal(walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='TextField')[0].props.value,'Current title');
  upload(host);const oldUpload=requests.at(-1);host.props={id:3};host.render();assert.equal(oldUpload.options.signal.aborted,true);const next=requests.at(-1);oldUpload.resolve({path:'/media/obsolete.png'});await settle(host);assert.equal(walk(host.tree,node=>node.type==='form').length,0);
  next.resolve({...article,id:3,article_cover:'/media/third.png'});await settle(host);save(host);assert.match(requests.at(-1).url,/\/articles\/3$/);assert.equal(JSON.parse(requests.at(-1).options.body).article_cover,'/media/third.png');
});


// Native disabled fieldsets lock all descendant form controls, including row actions.
function controls(host) {
  const result=[];
  function visit(node,disabled=false){
    if(Array.isArray(node)){node.forEach(item=>visit(item,disabled));return;}
    if(!node||typeof node!=='object')return;
    const locked=disabled||(node.type==='fieldset'&&node.props.disabled===true);
    if(['input','textarea','select','button'].includes(node.type))result.push({node,disabled:locked||node.props.disabled===true});
    visit(node.props?.children,locked);
  }
  visit(expand(host.tree));return result;
}
const fields=(host,name)=>walk(host.tree,node=>typeof node.type==='function'&&node.type.name===name);
const categoryFixture={id:1,sort_name:'Category A',sort_description:null,priority:0,article_count:0};
const labelFixture={id:1,sort_id:1,label_name:'Tag A',label_description:null};

test('article updates admit one pending publish, lock its draft, and preserve it for retry',async context=>{
  const {host,requests,notifications}=setup(context,ArticleNewsEditor,{id:1});requests[0].resolve([]);await settle(host);
  walk(host.tree,node=>node.type==='textarea')[0].props.onChange({target:{value:'First update'}});
  fields(host,'TextField')[0].props.onChange({target:{value:'2026-10-10T10:30'}});host.render();
  const before=requests.length;save(host);save(host);host.render();save(host);
  assert.equal(requests.length,before+1);assert.ok(controls(host).every(item=>item.disabled));
  assert.deepEqual(JSON.parse(requests.at(-1).options.body),{content:'First update',create_time:new Date('2026-10-10T10:30').toISOString()});
  requests.at(-1).reject(new Error('ordinary save failure'));await settle(host);
  assert.ok(controls(host).every(item=>!item.disabled));assert.equal(walk(host.tree,node=>node.type==='textarea')[0].props.value,'First update');
  assert.equal(fields(host,'TextField')[0].props.value,'2026-10-10T10:30');
  save(host);requests.at(-1).resolve({id:1,content:'First update',create_time:'2026-10-10 10:30:00'});await settle(host);
  assert.equal(walk(host.tree,node=>node.type==='textarea')[0].props.value,'');assert.equal(fields(host,'TextField')[0].props.value,'');
  assert.deepEqual(notifications,['Article update published']);assert.match(text(host),/First update/);
});

for(const [Component,kind] of [[CategoryManager,'category'],[LabelsPage,'label']]){
  test(`${kind} create and edit lock the form and row actions, prevent duplicate posts, and retry`,async context=>{
    const {host,requests,notifications}=setup(context,Component);
    if(kind==='category')requests[0].resolve([categoryFixture]);else{requests[0].resolve([labelFixture]);requests[1].resolve([categoryFixture]);}await settle(host);
    fields(host,'TextField')[0].props.onChange({target:{value:'New item'}});
    if(kind==='label')fields(host,'Select')[0].props.onChange({target:{value:'1'}});host.render();
    const before=requests.length;save(host);save(host);host.render();save(host);
    assert.equal(requests.length,before+1);assert.equal(requests.at(-1).options.method,'POST');assert.ok(controls(host).every(item=>item.disabled));
    requests.at(-1).reject(new Error('ordinary save failure'));await settle(host);
    assert.ok(controls(host).every(item=>!item.disabled));assert.equal(fields(host,'TextField')[0].props.value,'New item');
    save(host);requests.at(-1).resolve(kind==='category'?{...categoryFixture,id:2,sort_name:'New item'}:{...labelFixture,id:2,label_name:'New item'});await settle(host);
    assert.equal(fields(host,'TextField')[0].props.value,'');assert.equal(notifications.length,1);
    button(host,'Edit').props.onClick();host.render();save(host);host.render();
    assert.equal(requests.at(-1).options.method,'PUT');assert.ok(controls(host).every(item=>item.disabled));
    requests.at(-1).reject(new Error('ordinary edit failure'));await settle(host);
    assert.equal(fields(host,'TextField')[0].props.value,kind==='category'?'Category A':'Tag A');assert.ok(button(host,'Cancel editing'));
  });
}

test('article saving locks editable content and failure restores the submitted draft for correction',async context=>{
  const {host,requests}=await editor(context);save(host);host.render();assert.ok(controls(host).every(item=>item.disabled));
  requests.at(-1).reject(new Error('ordinary save failure'));await settle(host);
  assert.ok(controls(host).every(item=>!item.disabled));assert.equal(walk(host.tree,node=>node.type==='textarea')[0].props.value,'Original text');
  walk(host.tree,node=>node.type==='textarea')[0].props.onChange({target:{value:'Corrected final text'}});host.render();save(host);
  assert.equal(JSON.parse(requests.at(-1).options.body).article_content,'Corrected final text');requests.at(-1).resolve(article);await settle(host);assert.equal(window.location.hash,'articles');
});

test('home-section saves lock add, edit, reorder, and delete until success or failure',async context=>{
  const rows=[{id:1,title:'Latest',kind:'latest',sort_id:null,priority:0,enabled:true},{id:2,title:'Category A',kind:'category',sort_id:1,priority:10,enabled:true}];
  const {host,requests}=setup(context,HomeSectionsPage);requests[0].resolve(rows);requests[1].resolve([categoryFixture]);await settle(host);
  button(host,'Save sections').props.onClick();host.render();assert.ok(controls(host).every(item=>item.disabled));
  requests.at(-1).reject(new Error('ordinary save failure'));await settle(host);assert.equal(button(host,'Add category section').props.disabled,false);assert.equal(fields(host,'TextField').length,2);
  button(host,'Add category section').props.onClick();host.render();assert.equal(fields(host,'TextField').length,3);button(host,'Save sections').props.onClick();host.render();
  const payload=JSON.parse(requests.at(-1).options.body);assert.equal(payload.length,3);requests.at(-1).resolve(payload.map((row,index)=>({...row,id:index+1})));await settle(host);
  assert.equal(fields(host,'TextField').length,3);assert.equal(button(host,'Save sections').props.disabled,false);
});

test('site settings retain the existing pending lock and allow correction after failed save',async context=>{
  const {host,requests}=setup(context,SitePage);requests[0].resolve({web_name:'Site A'});await settle(host);save(host);host.render();assert.ok(controls(host).every(item=>item.disabled));
  requests.at(-1).reject(new Error('ordinary save failure'));await settle(host);assert.ok(controls(host).every(item=>!item.disabled));assert.equal(fields(host,'TextField')[0].props.value,'Site A');
  fields(host,'TextField')[0].props.onChange({target:{value:'Site B'}});host.render();save(host);requests.at(-1).resolve({web_name:'Site B'});await settle(host);assert.equal(fields(host,'TextField')[0].props.value,'Site B');
});

test('article page changes hide old destructive actions and a failed page can retry',async context=>{
  const {host,requests}=setup(context,ArticleManager);requests.at(-1).resolve(page([{...summary,article_title:'OLD_ARTICLE'}],16,1,15));await settle(host);
  button(host,'Next page').props.onClick();host.render();
  assert.doesNotMatch(text(host),/OLD_ARTICLE/);assert.equal(button(host,'Delete'),undefined);
  requests.at(-1).reject(new Error('offline'));await settle(host);
  assert.equal(button(host,'Delete'),undefined);
  walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='ErrorState')[0].props.onRetry();host.render();
  assert.match(requests.at(-1).url,/page=2/);requests.at(-1).resolve(page([{...summary,id:16,article_title:'PAGE_TWO'}],16,2,15));await settle(host);
  assert.match(text(host),/PAGE_TWO/);assert.doesNotMatch(text(host),/OLD_ARTICLE|Unable to load articles/);
});

for (const [zone, local, expected] of [
  ['Asia/Shanghai', '2026-10-10T20:00', '2026-10-10T12:00:00.000Z'],
  ['America/New_York', '2026-07-10T20:00', '2026-07-11T00:00:00.000Z'],
  ['America/New_York', '2026-01-10T20:00', '2026-01-11T01:00:00.000Z'],
  ['UTC', '', null],
]) {
  test(`article update local date becomes an absolute API timestamp (${zone}, ${local || 'default'})`, async context => {
    const previousTimezone = process.env.TZ;
    process.env.TZ = zone;
    context.after(() => { if (previousTimezone === undefined) delete process.env.TZ; else process.env.TZ = previousTimezone; });
    const { host, requests } = setup(context, ArticleNewsEditor, { id: 1 });
    requests[0].resolve([]); await settle(host);
    walk(host.tree, node => node.type === 'textarea')[0].props.onChange({ target: { value: 'Timed update' } });
    fields(host, 'TextField')[0].props.onChange({ target: { value: local } }); host.render();
    save(host);
    assert.equal(JSON.parse(requests.at(-1).options.body).create_time, expected);
    requests.at(-1).resolve({ id: 1, content: 'Timed update', create_time: expected }); await settle(host);
  });
}

const uploadNames = uploadNameFixtures.map(fixture => fixture.name);
for (const name of uploadNames) {
  test(`resource upload preserves UTF-8 filename: ${name}`, async context => {
    const {host,requests}=setup(context,ResourcesPage);
    requests[0].resolve(page([],0,1,20));await settle(host);
    const file=new File(['image'],name,{type:'image/png'});
    walk(host.tree,node=>node.type==='input'&&node.props.type==='file')[0].props.onChange({target:{files:[file]}});host.render();
    walk(host.tree,node=>node.type==='form')[0].props.onSubmit({preventDefault(){}});
    const pending=requests.at(-1);
    assert.equal(new Headers(pending.options.headers).get('X-File-Name'),encodeURIComponent(name));
    assert.equal(pending.options.body,file);
    pending.resolve({id:1,path:'/media/fixture.png'});await settle(host);
    requests.at(-1).resolve(page([{id:1,original_name:name,path:'/media/fixture.png',resource_type:'image',size:5,status:1,create_time:null}],1,1,20));await settle(host);
    assert.ok(text(host).includes(name));
  });
  for (const target of [0,1]) test(`article ${target===0?'cover':'content'} upload preserves UTF-8 filename: ${name}`, async context => {
    const {host,requests}=await editor(context);
    const file=new File(['image'],name,{type:'image/png'});
    walk(host.tree,node=>node.type==='input'&&node.props.type==='file')[target].props.onChange({target:{files:[file],value:name}});
    assert.equal(new Headers(requests.at(-1).options.headers).get('X-File-Name'),encodeURIComponent(name));
    assert.equal(requests.at(-1).options.body,file);
    requests.at(-1).resolve({path:'/media/fixture.png'});await settle(host);
    save(host);const saved=JSON.parse(requests.at(-1).options.body);
    if(target===0)assert.equal(saved.article_cover,'/media/fixture.png');
    else assert.ok(saved.article_content.includes(`![${name}](/media/fixture.png)`));
  });
}

test('upload filename metadata keeps the existing 200-byte limit without splitting UTF-8',()=>{
  for(const [name,expected] of [
    ['a'.repeat(201),'a'.repeat(200)],
    ['a'.repeat(197)+'🌅.png','a'.repeat(197)],
    ['a'.repeat(196)+'🌅.png','a'.repeat(196)+'🌅'],
    ['旅'.repeat(67),'旅'.repeat(66)],
  ]) assert.equal(decodeURIComponent(uploadFilename(name)),expected);
  for(const fixture of uploadNameFixtures)assert.equal(uploadFilename(fixture.name),fixture.header);
});

test('pending cover upload locks its URL while leaving body edits and cancellation available',async context=>{
  const {host,requests}=await editor(context);upload(host);host.render();
  const field=walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='FormField'&&node.props.label==='Cover image URL')[0];
  const input=walk(expand(field),node=>node.type==='input')[0];
  if(!input.props.disabled){input.props.onChange({target:{value:'/media/manual.png'}});host.render();}
  requests.at(-1).resolve({path:'/media/uploaded.png'});await settle(host);
  const updated=walk(host.tree,node=>typeof node.type==='function'&&node.type.name==='FormField'&&node.props.label==='Cover image URL')[0];
  if(!input.props.disabled)assert.equal(walk(expand(updated),node=>node.type==='input')[0].props.value,'/media/manual.png','old upload cannot overwrite a newer manual cover');
  assert.equal(input.props.disabled,true);
  assert.equal(walk(expand(updated),node=>node.type==='input')[0].props.disabled,false);
  assert.equal(button(host,'Cancel').props.disabled,false);
});
