import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import {transformWithOxc} from 'vite';
import {HookHost,walk,textContent} from './fixtures/hook-host.mjs';
const main=await readFile(new URL('./fixtures/react-baseline/src/main.tsx',import.meta.url),'utf8');
const prelude=`import {useState,useRef,useEffect,jsx,Fragment} from '${new URL('./fixtures/hook-host.mjs',import.meta.url).href}';
const h=(type,props,...children)=>jsx(type,{...props,children});
const useAdminApplication=()=>globalThis.__ARTICLE_DELETE_APP;
const t=(_zh,en)=>en;const formatUtcTime=()=>'';
const InstanceHeaderActions='actions',PageHeader='header',Button='button',ErrorState='error',LoadingState='loading',Table='table',EmptyState='empty';
const isArticlePage=isDelete=()=>true;var isDelete;`;
const {code}=await transformWithOxc(prelude+main.slice(main.indexOf('function ArticleManager()'),main.indexOf('const emptyArticle:'))+'\nexport {ArticleManager};','manager.tsx',{jsx:{runtime:'classic',pragma:'h',pragmaFrag:'Fragment'}});
const {ArticleManager}=await import(`data:text/javascript,${encodeURIComponent(code)}`);
const settle=async h=>{await new Promise(r=>setImmediate(r));h.render();};
function setup(t){
  const requests=[],notifications=[];
  globalThis.window={confirm:()=>true};
  globalThis.__ARTICLE_DELETE_APP={notify(message){notifications.push(message);},client:{request(url,validate,options={}){let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});requests.push({url,options,resolve,reject});return promise;}}};
  const host=new HookHost(ArticleManager);host.render();t.after(()=>host.unmount());
  return {host,requests,notifications};
}
const deleteButton=h=>walk(h.tree,n=>n.type==='button'&&textContent(n)==='Delete')[0];
const rows={items:[{id:1,article_title:'Synthetic article',view_status:1,view_count:0,create_time:null}],total:1,page:1,size:15};
test('article deletion ignores repeated clicks and permits retry after failure',async t=>{
  const {host,requests,notifications}=setup(t);requests[0].resolve(rows);await settle(host);
  const remove=deleteButton(host).props.onClick;remove();remove();host.render();
  assert.equal(requests.filter(r=>r.options.method==='DELETE').length,1);
  assert.equal(deleteButton(host).props.disabled,true);
  requests[1].reject(Error('offline'));await settle(host);
  assert.equal(deleteButton(host).props.disabled,false);
  deleteButton(host).props.onClick();requests[2].resolve({deleted:true});await settle(host);
  assert.deepEqual(notifications,['Article deleted']);
});
test('leaving the article list cancels stale delete callbacks',async t=>{
  const {host,requests,notifications}=setup(t);requests[0].resolve(rows);await settle(host);
  deleteButton(host).props.onClick();const pending=requests[1];host.unmount();
  assert.equal(pending.options.signal?.aborted,true);
  pending.resolve({deleted:true});await new Promise(r=>setImmediate(r));
  assert.deepEqual(notifications,[]);
  assert.equal(requests.length,2);
});
