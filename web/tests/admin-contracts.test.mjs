import assert from 'node:assert/strict';
import {test} from 'node:test';
import * as contracts from '../src/admin-contracts.ts';
import {originalModule} from './fixtures/original-module.mjs';
import {HookHost,expand,walk,textContent} from './fixtures/hook-host.mjs';
const pages=await originalModule(new URL('../src/AdminExtras.tsx',import.meta.url),{
 '@xcss/web/admin-shell':'export const useAdminApplication=()=>globalThis.__CONTRACT_APP;',
 './api':'export const publicErrorMessage=(_reason,fallback)=>fallback;',
 './AdminLayout':'export const adminGroups=[];',
 './MediaPreview':'export const ImageLightbox=()=>null;export const safeImageUrl=value=>value;'
});
const valid={
 User:{id:1,username:'User',email:null,phone_number:null,user_status:1,user_type:0,create_time:null},
 Comment:{id:1,source:0,comment_type:'message',user_id:null,username:null,comment_content:'Content',create_time:null},
 Resource:{id:1,original_name:null,path:null,resource_type:null,size:0,status:null,create_time:null},
 Label:{id:1,sort_id:1,label_name:'Tag',label_description:null},
 Link:{id:1,title:'Link',classify:null,cover:null,url:null,introduction:null,link_type:'favorites',status:1,create_time:null},
 TreeHole:{id:1,user_id:null,username:null,avatar:null,message:'Hello',image_path:null,create_time:null},
 Note:{id:1,user_id:null,username:null,content:'Note',image_path:null,is_public:1,create_time:null},
 Family:{id:1,bg_cover:null,man_cover:null,woman_cover:null,man_name:null,woman_name:null,timing:null,countdown_title:null,countdown_time:null,family_info:null,status:1},
 Statistics:{articles:0,comments:0,members:0,categories:0},
 UploadedImage:{id:1,path:`/media/${'a'.repeat(64)}.png`,mime_type:'image/png',size:1}
};
for(const [name,value] of Object.entries(valid))test(`${name} validates all current fields rather than a marker field`,()=>{
 const check=contracts[`is${name}`];assert.equal(check(value),true);
 for(const key of Object.keys(value)){const missing={...value};delete missing[key];assert.equal(check(missing),false,`${key} missing`);assert.equal(check({...value,[key]:{invalid:true}}),false,`${key} wrong type`);}
});
const settle=async host=>{await new Promise(resolve=>setImmediate(resolve));host.render()};
for(const [component,name,badField] of [['UsersPage','User','username'],['CommentsPage','Comment','comment_content'],['ResourcesPage','Resource','original_name'],['TreeHolePage','TreeHole','message'],['NotesPage','Note','content']])test(`${component} original request rejects a malformed nested row through its actual validator`,async context=>{
 globalThis.__CONTRACT_APP={notify(){},client:{request:async(_url,validate)=>{const value={items:[{...valid[name],[badField]:{hidden:'invalid'}}],page:1,size:20,total:1};if(!validate(value))throw Error('invalid response');return value}}};
 const host=new HookHost(pages[component]);context.after(()=>host.unmount());host.render();await settle(host);
 assert.match(textContent(expand(host.tree)),/Unable to load/);assert.equal(walk(expand(host.tree),node=>node.type==='button'&&['Delete','Disable','Enable'].includes(textContent(node))).length,0);
});
test('user status mutation rejects a boolean acknowledgement of the opposite requested state',async context=>{
 const requests=[];globalThis.__CONTRACT_APP={notify(){},client:{request:async(url,validate,options)=>{requests.push({url,options});const value=options?.method==='PUT'?{active:true}:{items:[valid.User],total:1,page:1,size:20};if(!validate(value))throw Error('wrong acknowledgement');return value}}};
 const host=new HookHost(pages.UsersPage);context.after(()=>host.unmount());host.render();await settle(host);const action=walk(expand(host.tree),node=>node.type==='button'&&textContent(node)==='Disable')[0];assert.ok(action);await action.props.onClick();await settle(host);assert.equal(JSON.parse(requests[1].options.body).active,false);assert.match(textContent(expand(host.tree)),/Unable to update user status/);assert.equal(requests.length,2);
});
