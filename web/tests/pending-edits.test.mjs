import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
const pages = await originalModule(new URL('../src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__PENDING_APP;',
  './api': 'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout': 'export const adminGroups=[];',
});
const saved={id:1,path:'/media/one.png',title:'Saved',classify:null,cover:null,url:null,introduction:null,link_type:'favorites',status:1};
const nodes=(host,type)=>walk(expand(host.tree),node=>node.type===type);
const settle=async host=>{for(let i=0;i<6;i++)await Promise.resolve();host.render();};
function setup(t,name){
  const requests=[];
  globalThis.__PENDING_APP={notify(){},client:{request(url,validate,options={}){let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});requests.push({url,options,resolve(value){if(validate(value))resolve(value);else reject(new Error('Invalid response fixture'));},reject});return promise;}}};
  const host=new HookHost(pages[name]);host.render();
  t.after(()=>{host.unmount();delete globalThis.__PENDING_APP;});
  return {host,requests};
}
for(const name of ['ResourcesPage','LinksPage','NotesPage','FamilyPage']){
  test(`${name}: a pending submission locks editable controls and ignores same-tick duplicate submits; failures allow retry`,async t=>{
    const {host,requests}=setup(t,name);
    if(name==='ResourcesPage'){
      nodes(host,'input').find(n=>n.props.type==='file').props.onChange({target:{files:[{name:'one.png',size:1,type:'image/png'}]}});host.render();
    }
    for(const input of nodes(host,'input').filter(n=>n.props.required)) input.props.onChange({target:{value:'Valid draft'}});
    for(const input of nodes(host,'textarea').filter(n=>n.props.required)) input.props.onChange({target:{value:'Valid draft'}});
    host.render();
    const submit=nodes(host,'form')[0].props.onSubmit;
    submit({preventDefault(){}});submit({preventDefault(){}});
    assert.equal(requests.filter(r=>r.options.method==='POST').length,1,'one mutation for repeated submit before rerender');
    host.render();
    const fieldsets=nodes(host,'fieldset');
    assert.ok(fieldsets.some(n=>n.props.disabled===true),'pending editor is a disabled fieldset');
    requests.find(r=>r.options.method==='POST').reject(new Error('network unavailable'));
    await settle(host);
    assert.ok(nodes(host,'fieldset').every(n=>!n.props.disabled),'failed request unlocks editor');
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
    assert.equal(requests.filter(r=>r.options.method==='POST').length,2,'retry sends once');
    const mutations=requests.filter(r=>r.options.method==='POST');
    assert.equal(mutations[0].options.body,mutations[1].options.body,'failure preserves the original draft or selected file');
    requests.filter(r=>r.options.method==='POST').at(-1).resolve(saved);
    await settle(host);
    assert.ok(nodes(host,'fieldset').every(n=>!n.props.disabled),'success unlocks editor');
  });
}

function userControls(host){
  const result=[];
  function visit(node,disabled=false){
    if(Array.isArray(node)){node.forEach(child=>visit(child,disabled));return;}
    if(!node||typeof node!=='object')return;
    disabled ||= node.type==='fieldset'&&node.props.disabled;
    if(['input','textarea','select','button'].includes(node.type)) result.push({...node,disabled:!!(disabled||node.props.disabled)});
    visit(node.props.children,disabled);
  }
  visit(expand(host.tree));return result;
}
for(const name of ['ResourcesPage','LinksPage','NotesPage','FamilyPage']){
  test(`${name}: a slow success cannot erase a newer reachable draft or file selection`,async t=>{
    const {host,requests}=setup(t,name);
    const controls=userControls(host);
    const editable=controls.find(n=>name==='ResourcesPage'?n.props.type==='file':name==='NotesPage'?n.type==='textarea':n.type==='input'&&n.props.required);
    const firstFile={name:'first.png',size:1,type:'image/png'};
    editable.props.onChange({target:{value:'First draft',files:[firstFile]}});host.render();
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});host.render();
    const pending=requests.find(r=>r.options.method==='POST');
    const current=userControls(host).find(n=>name==='ResourcesPage'?n.props.type==='file':name==='NotesPage'?n.type==='textarea':n.type==='input'&&n.props.required);
    // Only dispatch changes that a user can make through enabled controls.
    if(!current.disabled)current.props.onChange({target:{value:'New draft',files:[{name:'second.png',size:1,type:'image/png'}]}});
    host.render();pending.resolve(saved);await settle(host);
    if(!current.disabled){
      if(name==='ResourcesPage')assert.equal(nodes(host,'button').find(n=>textContent(n)==='Upload').props.disabled,false,'new selection remains uploadable');
      else assert.equal(userControls(host).find(n=>n.type===current.type&&n.props.required).props.value,'New draft','new draft survives old success');
    }
    assert.equal(current.disabled,true,'pending operation prevents conflicting user edits');
  });
}

for(const name of ['LinksPage','FamilyPage']){
  test(`${name}: pending save prevents switching rows or cancelling into a new edit`,async t=>{
    const {host,requests}=setup(t,name);
    const rows=name==='LinksPage'?[{...saved,id:1,title:'First'},{...saved,id:2,title:'Second'}]:[{id:1,man_name:'First',woman_name:'Partner',status:1},{id:2,man_name:'Second',woman_name:'Partner',status:1}];
    requests[0].resolve(name==='LinksPage'?{items:rows,page:1,size:20,total:2}:rows);await settle(host);
    userControls(host).find(n=>n.type==='button'&&textContent(n)==='Edit').props.onClick();host.render();
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});host.render();
    const pending=requests.find(r=>r.options.method==='PUT');
    assert.ok(pending.url.endsWith('/1'));
    const switches=userControls(host).filter(n=>n.type==='button'&&['Edit','Cancel editing'].includes(textContent(n)));
    assert.equal(switches.length,3);assert.ok(switches.every(n=>n.disabled),'both row edit actions and cancellation are disabled');
    pending.reject(new Error('save unavailable'));await settle(host);
    assert.ok(userControls(host).filter(n=>n.type==='button'&&['Edit','Cancel editing'].includes(textContent(n))).every(n=>!n.disabled));
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
    const retry=requests.filter(r=>r.options.method==='PUT').at(-1);
    assert.equal(retry.url,pending.url);assert.equal(retry.options.body,pending.options.body,'failed save preserves the complete edited record');
    retry.resolve(rows[0]);await settle(host);
  });
}

test('file upload retains its selection on failure and clears the native picker only on success',async t=>{
  const {host,requests}=setup(t,'ResourcesPage');
  const input=nodes(host,'input').find(n=>n.props.type==='file');
  const picker={value:'C:\\fakepath\\photo.png'};
  if(input.props.ref)input.props.ref.current=picker;
  const file={name:'photo.png',size:42,type:'image/png'};
  input.props.onChange({target:{files:[file]}});host.render();
  nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
  const upload=requests.find(r=>r.options.method==='POST');upload.reject(new Error('offline'));await settle(host);
  assert.equal(picker.value,'C:\\fakepath\\photo.png');
  nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
  const retry=requests.filter(r=>r.options.method==='POST').at(-1);
  assert.equal(retry.options.body,file);retry.resolve({id:1,path:'/media/photo.png'});await settle(host);
  assert.equal(picker.value,'');
  assert.equal(userControls(host).find(n=>n.type==='button'&&textContent(n)==='Upload').disabled,true);
});
