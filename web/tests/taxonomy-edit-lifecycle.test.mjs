import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import {transformWithOxc} from 'vite';
import {HookHost,expand,walk,textContent} from './fixtures/hook-host.mjs';
import {originalModule} from './fixtures/original-module.mjs';
const {LabelsPage}=await originalModule(new URL('../src/AdminExtras.tsx',import.meta.url),{
  '@xcss/web/admin-shell':'export const useAdminApplication=()=>globalThis.__TAXONOMY_APP;',
  './api':'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview':'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout':'export const adminGroups=[];',
});
const main=await readFile(new URL('../src/main.tsx',import.meta.url),'utf8');
const prelude=`import {useState,useRef,useEffect,jsx,Fragment} from '${new URL('./fixtures/hook-host.mjs',import.meta.url).href}';
const h=(type,props,...children)=>jsx(type,{...props,children});
const useAdminApplication=()=>globalThis.__TAXONOMY_APP;
const t=(_zh,en)=>en;const publicErrorMessage=(_reason,fallback)=>fallback;
const PageHeader='header',Button='button',TextField='input',FormField='field',ErrorState='error',LoadingState='loading',Table='table',EmptyState='empty';
const isCategories=isCategory=isDelete=()=>true;var isCategory,isDelete;`;
const {code}=await transformWithOxc(prelude+main.slice(main.indexOf('function CategoryManager()'),main.indexOf('function HomeArticleCard('))+'\nexport {CategoryManager};','categories.tsx',{jsx:{runtime:'classic',pragma:'h',pragmaFrag:'Fragment'}});
const {CategoryManager}=await import(`data:text/javascript,${encodeURIComponent(code)}`);
const categories=[{id:1,sort_name:'One',sort_description:null,priority:0,article_count:0},{id:2,sort_name:'Two',sort_description:null,priority:0,article_count:0}];
const labels=[{id:1,sort_id:1,label_name:'One',label_description:null},{id:2,sort_id:1,label_name:'Two',label_description:null}];
const nodes=(h,type)=>walk(expand(h.tree),n=>n.type===type);
const buttons=(h,name)=>nodes(h,'button').filter(n=>textContent(n)===name);
const settle=async h=>{await new Promise(r=>setImmediate(r));h.render();};
const submit=h=>nodes(h,'form')[0].props.onSubmit({preventDefault(){}});
const mutations=rs=>rs.filter(r=>r.options.method);
async function setup(t,Component){
  const requests=[],notifications=[];
  globalThis.window={confirm:()=>true};
  const client=()=>({request(url,validate,options={}){let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});requests.push({url,options,resolve,reject});return promise;}});
  globalThis.__TAXONOMY_APP={client:client(),notify:s=>notifications.push(s)};
  const host=new HookHost(Component);host.render();t.after(()=>host.unmount());
  requests.forEach(r=>r.resolve(r.url.endsWith('labels')?labels:categories));await settle(host);
  return {host,requests,notifications,replaceClient(){globalThis.__TAXONOMY_APP.client=client();host.render();}};
}
for(const [name,Component,createLabel] of [['category',CategoryManager,'Create category'],['tag',LabelsPage,'Add tag']]){
  test(`${name}: deleting the edited row exits its stale edit state`,async t=>{
    const {host,requests}=await setup(t,Component);buttons(host,'Edit')[0].props.onClick();host.render();
    buttons(host,'Delete')[0].props.onClick();mutations(requests)[0].resolve({deleted:true});await settle(host);
    assert.equal(buttons(host,'Save changes').length,0,'must not keep PUT targeting a deleted row');
    assert.equal(buttons(host,createLabel).length,1);
    assert.equal(nodes(host,'input')[0].props.value,'');
  });
  test(`${name}: a pending deletion suppresses duplicate deletes and same-row saves; failure preserves draft`,async t=>{
    const {host,requests}=await setup(t,Component);buttons(host,'Edit')[0].props.onClick();host.render();
    const remove=buttons(host,'Delete')[0].props.onClick;remove();remove();submit(host);host.render();
    assert.equal(mutations(requests).length,1);
    assert.equal(buttons(host,'Save changes')[0].props.disabled,true);
    assert.ok(nodes(host,'fieldset').every(n=>!n.props.disabled),'unrelated editing stays available');
    mutations(requests)[0].reject(Error('offline'));await settle(host);
    assert.equal(nodes(host,'input')[0].props.value,'One');
    assert.ok(nodes(host,'fieldset').every(n=>!n.props.disabled));
    buttons(host,'Delete')[0].props.onClick();assert.equal(mutations(requests).length,2);
  });
  test(`${name}: deleting another row preserves the current edit`,async t=>{
    const {host,requests}=await setup(t,Component);buttons(host,'Edit')[0].props.onClick();host.render();
    buttons(host,'Delete')[1].props.onClick();mutations(requests)[0].resolve({deleted:true});await settle(host);
    assert.equal(nodes(host,'input')[0].props.value,'One');assert.equal(buttons(host,'Save changes').length,1);
  });
  test(`${name}: switching to another edit during deletion keeps the newer draft`,async t=>{
    const {host,requests}=await setup(t,Component);buttons(host,'Edit')[0].props.onClick();host.render();
    buttons(host,'Delete')[0].props.onClick();host.render();
    buttons(host,'Edit')[1].props.onClick();host.render();
    nodes(host,'input')[0].props.onChange({target:{value:'Newer draft'}});host.render();
    assert.equal(buttons(host,'Save changes')[0].props.disabled,false);
    mutations(requests)[0].resolve({deleted:true});await settle(host);
    assert.equal(nodes(host,'input')[0].props.value,'Newer draft');
    submit(host);assert.match(mutations(requests)[1].url,/\/2$/);
  });
  test(`${name}: successful delete removes its stale edit button before refresh finishes`,async t=>{
    const {host,requests}=await setup(t,Component);buttons(host,'Delete')[0].props.onClick();
    mutations(requests)[0].resolve({deleted:true});await settle(host);
    assert.equal(buttons(host,'Edit').length,1);
    buttons(host,'Edit')[0].props.onClick();host.render();assert.equal(nodes(host,'input')[0].props.value,'Two');
  });
  test(`${name}: pending save blocks deletion until it settles`,async t=>{
    const {host,requests}=await setup(t,Component);buttons(host,'Edit')[0].props.onClick();host.render();
    const remove=buttons(host,'Delete')[0].props.onClick;submit(host);remove();host.render();
    assert.equal(mutations(requests).length,1);assert.equal(mutations(requests)[0].options.method,'PUT');
    assert.ok(nodes(host,'fieldset').some(n=>n.props.disabled));
    mutations(requests)[0].reject(Error('offline'));await settle(host);
    assert.equal(nodes(host,'input')[0].props.value,'One');
    buttons(host,'Delete')[0].props.onClick();assert.equal(mutations(requests)[1].options.method,'DELETE');
  });
  for(const method of ['save','delete'])test(`${name}: client change cancels pending ${method} callbacks`,async t=>{
    const {host,requests,notifications,replaceClient}=await setup(t,Component);buttons(host,'Edit')[0].props.onClick();host.render();
    if(method==='save')submit(host);else buttons(host,'Delete')[0].props.onClick();
    const pending=mutations(requests)[0];replaceClient();
    assert.equal(pending.options.signal?.aborted,true);
    pending.resolve(method==='delete'?{deleted:true}:{});await settle(host);
    assert.deepEqual(notifications,[]);
    assert.ok(nodes(host,'fieldset').every(n=>!n.props.disabled));
  });
}
