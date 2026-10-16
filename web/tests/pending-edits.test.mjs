import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
const pages = await originalModule(new URL('./fixtures/react-baseline/src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__PENDING_APP;',
  './api': 'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout': 'export const adminGroups=[];',
});
const saved={id:1,mime_type:'image/png',size:1,user_id:null,username:null,content:'Saved',image_path:null,is_public:1,create_time:null,path:'/media/7692c3ad3540bb803c020b3aee66cd8887123234ea0c6e7143c0add73ff431ed.png',title:'Saved',classify:null,cover:null,url:null,introduction:null,link_type:'favorites',status:1};
const home={id:1,bg_cover:null,man_cover:null,woman_cover:null,man_name:'Owner',woman_name:'Partner',timing:'2026-10-01',countdown_title:null,countdown_time:null,family_info:'Our home',status:1};
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
    if(name==='FamilyPage'){requests[0].resolve([]);await settle(host);}
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
    requests.filter(r=>r.options.method==='POST').at(-1).resolve(name==='FamilyPage'?home:saved);
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
    if(name==='FamilyPage'){requests[0].resolve([]);await settle(host);}
    const controls=userControls(host);
    const editable=controls.find(n=>name==='ResourcesPage'?n.props.type==='file':name==='NotesPage'?n.type==='textarea':n.type==='input'&&n.props.required);
    const firstFile={name:'first.png',size:1,type:'image/png'};
    editable.props.onChange({target:{value:'First draft',files:[firstFile]}});host.render();
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});host.render();
    const pending=requests.find(r=>r.options.method==='POST');
    const current=userControls(host).find(n=>name==='ResourcesPage'?n.props.type==='file':name==='NotesPage'?n.type==='textarea':n.type==='input'&&n.props.required);
    // Only dispatch changes that a user can make through enabled controls.
    if(!current.disabled)current.props.onChange({target:{value:'New draft',files:[{name:'second.png',size:1,type:'image/png'}]}});
    host.render();pending.resolve(name==='FamilyPage'?home:saved);await settle(host);
    if(!current.disabled){
      if(name==='ResourcesPage')assert.equal(nodes(host,'button').find(n=>textContent(n)==='Upload').props.disabled,false,'new selection remains uploadable');
      else assert.equal(userControls(host).find(n=>n.type===current.type&&n.props.required).props.value,'New draft','new draft survives old success');
    }
    assert.equal(current.disabled,true,'pending operation prevents conflicting user edits');
  });
}

for(const name of ['LinksPage']){
  test(`${name}: pending save prevents switching rows or cancelling into a new edit`,async t=>{
    const {host,requests}=setup(t,name);
    const rows=[{...saved,id:1,title:'First'},{...saved,id:2,title:'Second'}];
    requests[0].resolve({items:rows,page:1,size:20,total:2});await settle(host);
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
  assert.equal(retry.options.body,file);retry.resolve({id:1,mime_type:'image/png',size:42,path:'/media/55c64d0fcd6f9d5f7c828093857e3fdfda68478bb4e9bd24d481ef391c7804e8.png'});await settle(host);
  assert.equal(picker.value,'');
  assert.equal(userControls(host).find(n=>n.type==='button'&&textContent(n)==='Upload').disabled,true);
});


test('FamilyPage loads one home directly, preserves failed edits and stays in edit mode after saving',async t=>{
  const {host,requests}=setup(t,'FamilyPage');
  assert.equal(nodes(host,'form').length,0,'creation stays unavailable until the current home is known');
  requests[0].resolve([home]);await settle(host);
  assert.equal(nodes(host,'input').find(n=>n.props.required).props.value,'Owner');
  assert.equal(nodes(host,'table').length,0,'there is no multi-home list');
  assert.equal(userControls(host).filter(n=>n.type==='button'&&['Edit','Cancel editing'].includes(textContent(n))).length,0);
  nodes(host,'input').find(n=>n.props.required).props.onChange({target:{value:'Updated owner'}});host.render();
  const submit=nodes(host,'form')[0].props.onSubmit;submit({preventDefault(){}});submit({preventDefault(){}});host.render();
  const pending=requests.find(r=>r.options.method==='PUT');
  assert.equal(requests.filter(r=>r.options.method==='PUT').length,1);
  assert.equal(pending.url,'/api/v1/content/family/1');
  assert.ok(userControls(host).find(n=>n.type==='button'&&textContent(n)==='Delete home details').disabled);
  pending.reject(new Error('offline'));await settle(host);
  assert.equal(nodes(host,'input').find(n=>n.props.required).props.value,'Updated owner');
  nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
  const retry=requests.filter(r=>r.options.method==='PUT').at(-1);
  assert.equal(retry.options.body,pending.options.body);
  retry.resolve({...home,man_name:'Updated owner'});await settle(host);
  assert.equal(nodes(host,'h2').map(textContent).join(''),'Edit home details');
  nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
  assert.equal(requests.filter(r=>r.options.method==='POST').length,0,'saving an existing home never creates a second one');
});

test('FamilyPage keeps creation unavailable when current home loading fails',async t=>{
  const {host,requests}=setup(t,'FamilyPage');
  requests[0].reject(new Error('offline'));await settle(host);
  assert.equal(nodes(host,'form').length,0);
  assert.equal(requests.filter(r=>r.options.method==='POST').length,0);
});

for (const existing of [false, true]) {
  test(`FamilyPage ${existing ? 'edit' : 'create'}: a failed save has no reload action that can erase its draft`, async t => {
    const {host,requests}=setup(t,'FamilyPage');
    requests[0].resolve(existing?[home]:[]);await settle(host);
    nodes(host,'input').find(n=>n.props.required).props.onChange({target:{value:'Unsaved owner'}});host.render();
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});host.render();
    const pending=requests.at(-1);pending.reject(new Error('offline'));await settle(host);
    assert.equal(nodes(host,'input').find(n=>n.props.required).props.value,'Unsaved owner');
    assert.equal(nodes(host,'button').filter(n=>textContent(n)==='Try again').length,0,'mutation failures must not offer a destructive reload');
    nodes(host,'form')[0].props.onSubmit({preventDefault(){}});
    assert.equal(requests.at(-1).options.body,pending.options.body);
    assert.equal(requests.at(-1).options.method,existing?'PUT':'POST');
    requests.at(-1).resolve({...home,man_name:'Unsaved owner'});await settle(host);
    assert.equal(nodes(host,'input').find(n=>n.props.required).props.value,'Unsaved owner');
  });
}

test('FamilyPage initial load retries, while failed deletion retains the editable home and successful deletion reloads', async t => {
  const {host,requests}=setup(t,'FamilyPage');
  requests[0].reject(new Error('offline'));await settle(host);
  nodes(host,'button').find(n=>textContent(n)==='Try again').props.onClick();host.render();
  assert.equal(requests.at(-1).url,'/api/v1/content/family');
  requests.at(-1).resolve([home]);await settle(host);
  const oldWindow=globalThis.window;globalThis.window={confirm:()=>true};
  t.after(()=>{if(oldWindow===undefined)delete globalThis.window;else globalThis.window=oldWindow;});
  nodes(host,'input').find(n=>n.props.required).props.onChange({target:{value:'Unsaved owner'}});host.render();
  nodes(host,'button').find(n=>textContent(n)==='Delete home details').props.onClick();host.render();
  assert.equal(requests.at(-1).options.method,'DELETE');
  requests.at(-1).reject(new Error('offline'));await settle(host);
  assert.equal(nodes(host,'input').find(n=>n.props.required).props.value,'Unsaved owner');
  assert.equal(nodes(host,'button').filter(n=>textContent(n)==='Try again').length,0);
  nodes(host,'button').find(n=>textContent(n)==='Delete home details').props.onClick();host.render();
  requests.at(-1).resolve({deleted:true});await settle(host);
  assert.equal(requests.at(-1).url,'/api/v1/content/family');
  assert.equal(requests.at(-1).options.method,undefined);
  assert.equal(nodes(host,'form').length,0);
  requests.at(-1).resolve([]);await settle(host);
  assert.equal(nodes(host,'h2').map(textContent).join(''),'Create my home');
  assert.equal(nodes(host,'input').find(n=>n.props.required).props.value,'');
});
