import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {chromium,firefox,expect} from './dependencies.mjs';
import {preview} from '../tools/assets/test-preview.mjs';
import {mockApi,article,summary,pageOf} from './browser-fixtures.mjs';

// These assertions exercise Rust/WASM state, including contracts formerly covered
// only by the React HookHost reference tests. Every browser must execute WASM.
const server=await preview({preview:{host:'127.0.0.1',port:0,strictPort:true}});
const base=`http://127.0.0.1:${server.httpServer.address().port}`;

const categories=['Alpha','Beta','Gamma'].map((sort_name,i)=>({id:i+1,sort_name,sort_description:null,priority:i,article_count:0}));
const comment=id=>({id,user_id:null,username:null,avatar:null,comment_content:`Comment ${id}`,create_time:null,parent_username:null,reply_count:0});
const runtime=async page=>expect(page.locator('html')).toHaveAttribute('data-blog-runtime','blog-interface');
const settle=async page=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
try{
const metadataResponse=await fetch(base+'/assets/blog-assets.json');
assert.equal(metadataResponse.status,200);
const metadataBytes=new Uint8Array(await metadataResponse.arrayBuffer());
assert.ok(metadataBytes.byteLength<=512*1024,'Public source receipt retains the 512 KiB asset budget');
const metadata=JSON.parse(new TextDecoder().decode(metadataBytes));
assert.equal(metadata.format,2);assert.equal(metadata.package,'blog_browser');assert.equal(metadata.profile,'web-release');
assert.equal(metadata.browser_bridge,'blog-browser-bridge/1');assert.equal(metadata.asset_builder,'blog-assets');
const sha256=/^[0-9a-f]{64}$/;
const sourceFiles=(files,required)=>{
 assert.ok(files&&typeof files==='object'&&!Array.isArray(files));
 assert.ok(Object.keys(files).length>0);
 for(const [path,hash] of Object.entries(files)){
  assert.ok(path&&!path.startsWith('/')&&!path.split('/').includes('..'),'Relative source inventory path');
  assert.match(hash,sha256,path);
 }
 assert.ok(files[required],`Source receipt includes ${required}`);
};
sourceFiles(metadata.browser_inputs,'src/lib.rs');sourceFiles(metadata.shared_inputs,'ORIGIN.json');
assert.match(metadata.source_snapshot_sha256,sha256);
assert.ok(metadata.source_packages&&typeof metadata.source_packages==='object'&&!Array.isArray(metadata.source_packages));
assert.ok(Object.keys(metadata.source_packages).length>0);
for(const [name,summary] of Object.entries(metadata.source_packages)){
 assert.match(summary.sha256,sha256,name);assert.ok(Number.isSafeInteger(summary.file_count)&&summary.file_count>0,name);
}
const shared=metadata.source_packages['src/interface'];assert.ok(shared,'Local UI source package summary');
assert.equal(shared.file_count,Object.keys(metadata.shared_inputs).length);
assert.equal(shared.sha256,createHash('sha256').update(JSON.stringify(metadata.shared_inputs)).digest('hex'),'Local UI summary binds its exact public file inventory');
for(const summary of [metadata.foundation_inventory,metadata.rust_runtime?.source_inputs_summary]){
 assert.ok(summary);assert.match(summary.sha256,sha256);assert.ok(Number.isSafeInteger(summary.file_count)&&summary.file_count>0);
}
assert.match(metadata.files['blog_browser_bg.wasm'],sha256);
for(const engine of [chromium,firefox]){
 const browser=await engine.launch();
 try{
  const page=await browser.newPage({locale:'en',viewport:{width:1280,height:900}}),errors=[],wasm=[];
  page.on('pageerror',error=>errors.push(error.message));page.on('response',response=>{if(new URL(response.url()).pathname.endsWith('.wasm'))wasm.push(response.body().then(body=>({ok:response.ok(),hash:createHash('sha256').update(body).digest('hex')})).catch(error=>({ok:false,error:error.message})));});
  await mockApi(page,{authenticated:true});
  await page.emulateMedia({colorScheme:'light'});
  await page.route('**/api/v1/categories',route=>route.fulfill({json:categories}));
  await page.route('**/api/v1/content/labels',route=>route.fulfill({json:[{id:1,sort_id:1,label_name:'One',label_description:null}]}));
  await page.goto(base+'/admin?lang=en#new');await runtime(page);
  await expect(page.getByRole('button',{name:'Switch to dark mode',exact:true})).toBeVisible();
  await page.emulateMedia({colorScheme:'dark'});await expect(page.locator('html')).toHaveAttribute('data-theme','dark');await expect(page.getByRole('button',{name:'Switch to light mode',exact:true})).toBeVisible();
  await page.emulateMedia({colorScheme:'light'});await expect(page.locator('html')).toHaveAttribute('data-theme','light');await page.getByRole('button',{name:'Switch to dark mode',exact:true}).click();
  await page.emulateMedia({colorScheme:'dark'});await page.emulateMedia({colorScheme:'light'});await expect(page.locator('html')).toHaveAttribute('data-theme','dark');await expect(page.getByRole('button',{name:'Switch to light mode',exact:true})).toBeVisible();await page.getByRole('button',{name:'Switch to light mode',exact:true}).click();
  const category=page.getByRole('combobox',{name:'Category',exact:true});
  await category.focus();await category.press('ArrowDown');await expect(category).toHaveAttribute('aria-expanded','true');
  await category.press('End');await category.press('Enter');await expect(page.locator('.editor-form select').nth(0)).toHaveValue('3');
  await category.press('Home');await category.press('Escape');await expect(page.locator('.editor-form select').nth(0)).toHaveValue('3');
  await category.press('a');await category.press('Enter');await expect(page.locator('.editor-form select').nth(0)).toHaveValue('1');
  await category.press('ArrowDown');await category.press('End');await category.press('Tab');await expect(page.locator('.editor-form select').nth(0)).toHaveValue('3');
  await category.click();await page.locator('.editor-form fieldset').evaluate(node=>node.disabled=true);await expect(category).toBeDisabled();await expect(category).toHaveAttribute('aria-expanded','false');
  await page.locator('.editor-form fieldset').evaluate(node=>node.disabled=false);await category.press('Home');await category.press('ArrowDown');await category.press('Enter');
  await page.locator('.editor-form').first().evaluate(node=>node.reset());await settle(page);await expect(page.locator('.editor-form select').nth(0)).toHaveValue('1');
  const tag=page.getByRole('combobox',{name:'Tags',exact:true});await tag.click();await page.getByRole('option',{name:'One',exact:true}).click();
  await page.getByLabel('Title',{exact:true}).fill('Preserved article');await page.getByLabel('Markdown content',{exact:true}).fill('Preserved body');
  const creations=[];let pending;
  await page.route('**/api/v1/content/articles',route=>{if(route.request().method()!=='POST')return route.fallback();creations.push({body:route.request().postDataJSON(),key:route.request().headers()['idempotency-key']});pending=route;});
  await page.getByRole('button',{name:'Save article',exact:true}).click();await expect.poll(()=>creations.length).toBe(1);
  await page.locator('.editor-form').first().evaluate(node=>{node.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));node.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));});await settle(page);assert.equal(creations.length,1);
  await pending.fulfill({status:503,json:{code:'unavailable',retryable:true}});await expect(page.getByRole('alert').first()).toBeVisible();await expect(page.getByLabel('Title',{exact:true})).toHaveValue('Preserved article');
  await page.getByRole('button',{name:'Confirm original article',exact:true}).click();await expect.poll(()=>creations.length).toBe(2);assert.deepEqual(creations[1],creations[0]);
  await pending.fulfill({json:{...article,id:77,...creations[1].body,view_status:1,recommend_status:0,comment_status:1,password_required:0}});await expect(page.getByRole('heading',{name:'Articles',exact:true})).toBeVisible();

  const section={id:1,title:'Original',kind:'latest',sort_id:null,priority:0,enabled:true};let reads=0,categoryReads=0,homeWrites=[],homeRead;
  await page.route('**/api/v1/categories',route=>{categoryReads++;return categoryReads===1?route.fulfill({status:503,json:{code:'unavailable'}}):route.fulfill({json:categories});});
  await page.route('**/api/v1/content/home-sections',route=>{if(route.request().method()==='GET'){reads++;homeRead=route;return;}homeWrites.push(route.request().postDataJSON());return homeWrites.length===1?route.fulfill({status:503,json:{code:'unavailable'}}):route.fulfill({json:homeWrites.at(-1).map((row,i)=>({...row,id:i+1}))});});
  await page.goto(base+'/admin?lang=en#home');await runtime(page);await expect(page.getByRole('button',{name:'Save sections',exact:true})).toBeDisabled();await expect(page.getByRole('button',{name:'Add category section',exact:true})).toBeDisabled();await expect.poll(()=>Boolean(homeRead)).toBe(true);await homeRead.fulfill({json:[section]});await page.getByLabel('Section 1 title',{exact:true}).fill('Dirty section');
  await page.getByRole('alert').getByRole('button',{name:'Try again',exact:true}).click();await expect.poll(()=>categoryReads).toBe(2);await expect(page.getByLabel('Section 1 title',{exact:true})).toHaveValue('Dirty section');assert.equal(reads,1);
  await page.getByRole('button',{name:'Save sections',exact:true}).click();await expect(page.getByRole('alert')).toContainText('Unable to save');await page.getByRole('alert').getByRole('button',{name:'Try again',exact:true}).click();await expect(page.getByText('Home sections saved',{exact:true})).toBeVisible();assert.equal(reads,1);assert.equal(homeWrites.length,2);assert.deepEqual(homeWrites[1],homeWrites[0]);

  page.on('dialog',dialog=>dialog.accept());
  for(const labels of [false,true]){
   const endpoint=`/api/v1/content/${labels?'labels':'categories'}`,readEndpoint=labels?endpoint:'/api/v1/categories';
   let records=[1,2].map(id=>labels?{id,sort_id:1,label_name:`Taxon ${id}`,label_description:null}:{id,sort_name:`Taxon ${id}`,sort_description:null,priority:id,article_count:0});
   const writes=[];let pendingDelete,pendingRefresh,refresh=false;
   const matches=url=>url.pathname===readEndpoint||url.pathname===endpoint||url.pathname.startsWith(endpoint+'/');
   await page.route(matches,route=>{
    const request=route.request(),method=request.method();
    if(method==='GET'){assert.equal(new URL(request.url()).pathname,readEndpoint);if(refresh){pendingRefresh=route;return;}return route.fulfill({json:records});}
    writes.push({method,path:new URL(request.url()).pathname,body:method==='DELETE'?null:request.postDataJSON()});
    if(method==='DELETE'){pendingDelete=route;return;}
    const body=request.postDataJSON();assert.deepEqual(Object.keys(body).sort(),labels?['description','name','sort_id']:['description','name','priority']);
    const value=labels?{id:2,sort_id:body.sort_id,label_name:body.name,label_description:body.description}:{id:2,sort_name:body.name,sort_description:body.description,priority:body.priority,article_count:0};records=[value];return route.fulfill({json:value});
   });
   await page.goto(base+`/admin?lang=en#${labels?'labels':'categories'}`);await runtime(page);
   const table=page.getByRole('table',{name:labels?'Tag list':'Category list',exact:true}),first=table.getByRole('row').filter({hasText:'Taxon 1'}),second=table.getByRole('row').filter({hasText:'Taxon 2'}),name=page.getByLabel(labels?'Tag name':'Category name',{exact:true}),save=page.getByRole('button',{name:'Save changes',exact:true});
   await first.getByRole('button',{name:'Edit',exact:true}).click();await name.fill('First draft');await first.getByRole('button',{name:'Delete',exact:true}).click();await expect.poll(()=>writes.length).toBe(1);await expect(save).toBeDisabled();
   await page.locator('form.inline-form').evaluate(form=>{form.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));form.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));});await settle(page);assert.equal(writes.length,1);
   await second.getByRole('button',{name:'Edit',exact:true}).click();await name.fill('Newer draft');await expect(save).toBeEnabled();
   await pendingDelete.fulfill({status:503,json:{code:'unavailable'}});await expect(page.getByRole('alert').first()).toBeVisible();await expect(name).toHaveValue('Newer draft');
   pendingDelete=null;await first.getByRole('button',{name:'Delete',exact:true}).click();await expect.poll(()=>Boolean(pendingDelete)).toBe(true);records=records.filter(row=>row.id!==1);refresh=true;await pendingDelete.fulfill({json:{deleted:true}});await expect.poll(()=>Boolean(pendingRefresh)).toBe(true);
   await expect(first).toHaveCount(0);await expect(second).toBeVisible();await expect(name).toHaveValue('Newer draft');await pendingRefresh.fulfill({status:503,json:{code:'unavailable'}});await expect(page.getByRole('alert').first()).toBeVisible();await expect(second).toBeVisible();await expect(first).toHaveCount(0);
   pendingRefresh=null;await page.getByRole('alert').getByRole('button',{name:'Try again',exact:true}).click();await expect.poll(()=>Boolean(pendingRefresh)).toBe(true);refresh=false;await pendingRefresh.fulfill({json:records});await expect(second).toBeVisible();await save.click();await expect.poll(()=>writes.length).toBe(3);assert.equal(writes[2].method,'PUT');assert.equal(writes[2].path,`${endpoint}/2`);assert.equal(writes[2].body.name,'Newer draft');await expect(page.getByText(labels?'Tag saved':'Category saved',{exact:true})).toBeVisible();await page.unroute(matches);
  }
  let familyReads=0,familyRetry;
  await page.route('**/api/v1/content/family',route=>{familyReads++;if(familyReads===1)return route.fulfill({status:503,json:{code:'unavailable'}});familyRetry=route;});
  await page.goto(base+'/admin?lang=en#family');await runtime(page);await expect(page.getByRole('alert')).toBeVisible();await expect(page.getByRole('button',{name:'Save',exact:true})).toBeDisabled();
  await page.getByRole('alert').getByRole('button',{name:'Try again',exact:true}).click();await expect.poll(()=>Boolean(familyRetry)).toBe(true);await expect(page.getByRole('button',{name:'Save',exact:true})).toBeDisabled();await familyRetry.fulfill({json:[]});await expect(page.getByRole('button',{name:'Save',exact:true})).toBeEnabled();

  const moderated=[{id:301,user_id:null,username:null,comment_type:'love',source:0,comment_content:'Anonymous moderation',create_time:null},{id:302,user_id:42,username:null,comment_type:'article',source:13,comment_content:'Numeric moderation',create_time:null}];
  await page.route('**/api/v1/content/comments?*',route=>route.fulfill({json:pageOf(moderated,new URL(route.request().url()))}));
  await page.goto(base+'/admin?lang=en#comments');await runtime(page);
  const anonymous=page.getByRole('row').filter({hasText:'Anonymous moderation'}),numeric=page.getByRole('row').filter({hasText:'Numeric moderation'});
  await expect(anonymous.getByRole('cell').nth(1)).toHaveText('Anonymous guest');await expect(anonymous.getByRole('cell').nth(3)).toHaveText('—');await expect(anonymous.getByRole('link',{name:'Wishes',exact:true})).toHaveAttribute('href','/love');
  await expect(numeric.getByRole('cell').nth(1)).toHaveText('42');await expect(numeric.getByRole('link',{name:'Article #13',exact:true})).toHaveAttribute('href','/article/13');await expect(numeric.getByRole('link')).toHaveAttribute('target','_blank');

  await page.route('**/api/v1/content/tree-hole?*',route=>route.fulfill({json:pageOf([{id:401,user_id:42,username:null,avatar:null,message:'',image_path:'/__test-media/portrait.png',create_time:null}],new URL(route.request().url()))}));
  await page.goto(base+'/admin?lang=en#tree-hole');await runtime(page);
  const imageMessage=page.getByRole('table',{name:'Message list',exact:true}).getByRole('row').filter({hasText:'User 42'});
  await expect(imageMessage.getByRole('cell').nth(2)).toHaveText('Image message');await expect(imageMessage.getByRole('cell').nth(4)).toHaveText('—');await expect(imageMessage.getByRole('button',{name:'View message image',exact:true})).toBeVisible();

  let slowPage,postBodies=[],postCount=0;const comments=Array.from({length:11},(_,i)=>comment(i+1));
  await page.route('**/api/v1/message-comments?*',route=>{const url=new URL(route.request().url());if(url.searchParams.get('page')==='2'){slowPage=route;return;}return route.fulfill({json:pageOf(comments,url)});});
  await page.route('**/api/v1/message-comments',route=>{postBodies.push(route.request().postDataJSON());postCount++;if(postCount===1)return route.abort('failed');const saved={...comment(99),comment_content:postBodies.at(-1).content};comments.unshift(saved);return route.fulfill({json:saved});});
  await page.goto(base+'/message?lang=en');await runtime(page);await expect(page.locator('.public-comment')).toHaveCount(10);
  let pager=page.getByRole('navigation',{name:'Comment pages',exact:true});await pager.getByRole('button',{name:'Next page',exact:true}).click();await expect.poll(()=>Boolean(slowPage)).toBe(true);await expect(page.locator('.public-comment')).toHaveCount(0);await expect(pager).toContainText('Page 2');
  await pager.getByRole('button',{name:'Previous page',exact:true}).click();await expect(page.locator('.public-comment')).toHaveCount(10);await slowPage.fulfill({json:{items:[comment(11)],total:11,page:2,size:10}}).catch(()=>{});await settle(page);await expect(pager).toContainText('Page 1');await expect(page.locator('.public-comment')).toHaveCount(10);
  await page.getByRole('textbox',{name:'Write your thoughts',exact:true}).fill('\u0085Recover this comment\u0085');await page.getByRole('button',{name:'Post anonymous comment',exact:true}).click();await expect(page.getByRole('alert')).toContainText('not yet confirmed');
  await page.reload();await runtime(page);await expect(page.getByRole('textbox',{name:'Write your thoughts',exact:true})).toHaveValue('Recover this comment');await expect(page.getByRole('textbox',{name:'Write your thoughts',exact:true})).toBeDisabled();await page.getByRole('button',{name:'Confirm original submission',exact:true}).click();await expect(page.getByRole('status').filter({hasText:'Your comment has been posted.'})).toBeVisible();assert.deepEqual(postBodies[1],postBodies[0]);assert.match(postBodies[0].request_id,/^[0-9a-f-]{36}$/);assert.equal(postBodies[0].content,'Recover this comment');
  assert.ok(wasm.length>0,'The browser fetched an independent WASM artifact');const artifacts=await Promise.all(wasm);assert.deepEqual(artifacts.filter(artifact=>!artifact.ok),[]);for(const artifact of artifacts)assert.equal(artifact.hash,metadata.files['blog_browser_bg.wasm'],'The executed WASM matches the build source inventory');assert.deepEqual(errors,[]);
  await page.close();
  const blocked=await browser.newContext({locale:'en'});await blocked.addInitScript(()=>Object.defineProperty(window,'sessionStorage',{get(){throw Error('blocked');}}));const blockedPage=await blocked.newPage();await mockApi(blockedPage,{authenticated:false});await blockedPage.goto(base+'/message?lang=en');await runtime(blockedPage);await expect(blockedPage.getByRole('alert')).toContainText('pending submission could not be read or saved');await expect(blockedPage.getByRole('textbox',{name:'Write your thoughts',exact:true})).toBeDisabled();await blocked.close();
  console.log(`${engine.name()}: real blog-browser WASM, system/manual theme synchronization, selector keyboard/reset/disabled, creation receipt/double-submit, home isolated retry, taxonomy concurrent deletion/draft retention/read recovery, family load gate, moderation fallback/source links, comment stale-page cancellation and durable submission recovery passed`);
 }finally{await browser.close();}
}}finally{await new Promise(resolve=>server.httpServer.close(resolve));}
