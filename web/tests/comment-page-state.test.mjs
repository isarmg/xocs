import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
const reexport = path => `export * from ${JSON.stringify(new URL(path, import.meta.url).href)};`;
const { CommentThread } = await originalModule(new URL('./fixtures/react-baseline/src/PublicComments.tsx', import.meta.url), {
  './api': reexport('./fixtures/react-baseline/src/api.ts'), './public-contracts': reexport('./fixtures/react-baseline/src/public-contracts.ts'),
  './comment-submission': reexport('./fixtures/react-baseline/src/comment-submission.ts'), './MediaPreview': 'export const safeImageUrl=value=>value;',
});
const json = body => new Response(JSON.stringify(body));
const row = id => ({ id, user_id: null, username: null, avatar: null, comment_content: `Comment ${id}`, create_time: null, parent_username: null, reply_count: 0 });
const page = n => ({items: Array.from({length: n === 1 ? 10 : 1}, (_, i) => row((n-1)*10+i+1)), page:n, size:10, total:11});
const settle = async host => { await new Promise(resolve=>setImmediate(resolve)); host.render(); };
const button = (host,label) => walk(host.tree,n=>n.type==='button' && textContent(n)===label)[0];
const snapshot = host => ({heading:textContent(walk(host.tree,n=>n.type==='h3')[0]), pager:textContent(walk(host.tree,n=>n.type==='nav')[0]), rowIds:walk(host.tree,n=>typeof n.type==='function' && n.type.name==='CommentEntry').map(n=>n.props.item.id), alerts:walk(host.tree,n=>n.props.role==='alert').map(textContent)});
function setup(t, props) {
 const originalFetch=globalThis.fetch, storage=Object.getOwnPropertyDescriptor(globalThis,'sessionStorage');
 Object.defineProperty(globalThis,'sessionStorage',{configurable:true,value:{getItem:()=>null,setItem(){},removeItem(){}}});
 const requests=[];
 globalThis.fetch=(url, options)=>new Promise((resolve,reject)=>requests.push({url,options,resolve,reject}));
 const host=new HookHost(CommentThread,props); host.render();
 t.after(()=>{host.unmount();globalThis.fetch=originalFetch;if(storage)Object.defineProperty(globalThis,'sessionStorage',storage);else delete globalThis.sessionStorage;});
 return {host,requests};
}
for(const kind of ['article','message','love']) test(`${kind}: page 2 slow/fails then page 1 succeeds`,async t=>{
 const {host,requests}=setup(t,{kind,articleId:22});
 requests[0].resolve(json(page(1))); await settle(host);
 const initial=snapshot(host); assert.deepEqual(initial.rowIds,[1,2,3,4,5,6,7,8,9,10]);
 button(host,'Next page').props.onClick(); host.render();
 assert.match(requests[1].url,/page=2/);
 const slow=snapshot(host); assert.match(slow.pager,/Page 2/); assert.deepEqual(slow.rowIds,[]); assert.match(textContent(host.tree),/Loading comments/);
 requests[1].reject(new TypeError('ordinary network failure')); await settle(host);
 const failed=snapshot(host); assert.match(failed.pager,/Page 2/); assert.deepEqual(failed.rowIds,[]);assert.match(failed.alerts.join(' '),/Unable to load comments/); assert.doesNotMatch(textContent(host.tree),/No comments yet|Loading comments/);
 button(host,'Previous page').props.onClick();host.render();requests[2].resolve(json(page(1)));await settle(host);
 const recovered=snapshot(host);assert.match(recovered.pager,/Page 1/);assert.deepEqual(recovered.rowIds,initial.rowIds);assert.deepEqual(recovered.alerts,[]);

});
test('GET recovery must preserve independent unknown POST result',async t=>{
 const {host,requests}=setup(t,{kind:'message'});requests[0].resolve(json(page(1)));await settle(host);
 walk(host.tree,n=>n.type==='textarea')[0].props.onChange({target:{value:'New comment'}});host.render();
 walk(host.tree,n=>n.type==='form')[0].props.onSubmit({preventDefault(){}});host.render();
 requests[1].reject(new TypeError('lost POST response'));await settle(host);
 assert.match(snapshot(host).alerts.join(' '),/not yet confirmed/);
 button(host,'Next page').props.onClick();host.render();requests[2].resolve(json(page(2)));await settle(host);
 assert.match(snapshot(host).alerts.join(' '),/not yet confirmed/);
 assert.equal(walk(host.tree,n=>n.type==='textarea')[0].props.disabled,true);

});

test('initial slow read has a loading state and initial failure retries the same page',async t=>{
 const {host,requests}=setup(t,{kind:'message'});
 assert.match(textContent(host.tree),/Loading comments/);assert.deepEqual(snapshot(host).rowIds,[]);assert.doesNotMatch(textContent(host.tree),/No comments yet/);
 requests[0].reject(new TypeError('offline'));await settle(host);
 assert.match(snapshot(host).alerts.join(' '),/Unable to load comments/);assert.doesNotMatch(textContent(host.tree),/No comments yet|Loading comments/);
 button(host,'Try again').props.onClick();host.render();assert.equal(requests[1].url,requests[0].url);
 requests[1].resolve(json(page(1)));await settle(host);assert.equal(snapshot(host).rowIds.length,10);assert.deepEqual(snapshot(host).alerts,[]);
});
test('failed page has a direct retry which loads that page, without stale rows',async t=>{
 const {host,requests}=setup(t,{kind:'message'});requests[0].resolve(json(page(1)));await settle(host);
 button(host,'Next page').props.onClick();host.render();requests[1].reject(new TypeError('offline'));await settle(host);
 button(host,'Try again').props.onClick();host.render();assert.equal(requests[2].url,requests[1].url);assert.deepEqual(snapshot(host).rowIds,[]);
 requests[2].resolve(json(page(2)));await settle(host);assert.deepEqual(snapshot(host).rowIds,[11]);assert.deepEqual(snapshot(host).alerts,[]);assert.match(snapshot(host).pager,/Page 2/);
});
for(const late of ['success','failure']) test(`superseded page 2 ${late} cannot change recovered page 1`,async t=>{
 const {host,requests}=setup(t,{kind:'message'});requests[0].resolve(json(page(1)));await settle(host);
 button(host,'Next page').props.onClick();host.render();button(host,'Previous page').props.onClick();host.render();
 assert.equal(requests[1].options.signal.aborted,true);
 requests[2].resolve(json(page(1)));await settle(host);const expected=snapshot(host);
 if(late==='success')requests[1].resolve(json(page(2)));else requests[1].reject(new TypeError('late failure'));
 await settle(host);assert.deepEqual(snapshot(host),expected);assert.doesNotMatch(textContent(host.tree),/Loading comments/);
});
test('known POST rejection remains visible through successful list navigation',async t=>{
 const {host,requests}=setup(t,{kind:'message'});requests[0].resolve(json(page(1)));await settle(host);
 walk(host.tree,n=>n.type==='textarea')[0].props.onChange({target:{value:'New comment'}});host.render();
 walk(host.tree,n=>n.type==='form')[0].props.onSubmit({preventDefault(){}});host.render();
 requests[1].resolve(new Response(JSON.stringify({code:'too_many_requests',message:'retry later',retryable:true}),{status:429}));await settle(host);
 assert.match(snapshot(host).alerts.join(' '),/Too many requests/);
 button(host,'Next page').props.onClick();host.render();requests[2].resolve(json(page(2)));await settle(host);
 assert.match(snapshot(host).alerts.join(' '),/Too many requests/);assert.equal(walk(host.tree,n=>n.type==='textarea')[0].props.disabled,false);
});

for (const total of [0, 10]) test(`deleted last comment returns to last available page (total ${total})`, async t => {
 const {host,requests}=setup(t,{kind:'message'});requests[0].resolve(json(page(1)));await settle(host);
 button(host,'Next page').props.onClick();host.render();
 requests[1].resolve(json({items:[],page:2,size:10,total}));await settle(host);
 assert.equal(requests.length,3);assert.match(requests[2].url,/page=1/);assert.deepEqual(snapshot(host).rowIds,[]);
 requests[2].resolve(json({items:total ? page(1).items : [],page:1,size:10,total}));await settle(host);
 assert.equal(snapshot(host).rowIds.length,total);assert.doesNotMatch(textContent(host.tree),/Page 2/);
});
