import assert from 'node:assert/strict';
import {chromium,firefox,expect} from '@playwright/test';
import {preview} from 'vite';
import {mockApi,article} from './browser-fixtures.mjs';
const server=await preview({preview:{host:'127.0.0.1',port:0,strictPort:true}});
const base=`http://127.0.0.1:${server.httpServer.address().port}`;
// A real, short PCM WAV lets Firefox finish media metadata loading. An empty
// audio/mpeg body leaves its media request pending even after the UI is ready.
const audio=Buffer.alloc(44+8820);audio.write('RIFF');audio.writeUInt32LE(audio.length-8,4);audio.write('WAVEfmt ',8);audio.writeUInt32LE(16,16);audio.writeUInt16LE(1,20);audio.writeUInt16LE(1,22);audio.writeUInt32LE(44100,24);audio.writeUInt32LE(88200,28);audio.writeUInt16LE(2,32);audio.writeUInt16LE(16,34);audio.write('data',36);audio.writeUInt32LE(audio.length-44,40);
try{for(const engine of [chromium,firefox]){
 const browser=await engine.launch();
 try{
  const page=await browser.newPage({locale:'en',viewport:{width:1280,height:900}}),errors=[],calls=[],saved=[];
  page.on('pageerror',error=>errors.push(error.message));await mockApi(page,{authenticated:true});
  await page.route('**/api/v1/links/page?*',route=>{const params=new URL(route.request().url()).searchParams,kind=params.get('kind'),number=Number(params.get('page')),size=Number(params.get('size'));calls.push({kind,page:number,size});const ids=Array.from({length:201},(_,i)=>201-i).slice((number-1)*size,number*size);return route.fulfill({json:{items:ids.map(id=>({id,title:`${kind}-${id}`,classify:null,cover:null,url:kind==='funny'?`/media/music-${id}.mp3`:'https://example.test/'+id,introduction:null,link_type:kind,create_time:null})),total:201,page:number,size}});});
  await page.route('**/media/music-*',route=>route.fulfill({status:200,body:audio,contentType:'audio/wav'}));
  for(const [tab,kind] of [['favorites','favorites'],['friends','friendUrl'],['music','funny']]){
   await page.goto(`${base}/favorite?lang=en&tab=${tab}`);await expect(page.getByText(`${kind}-201`,{exact:true}).first()).toBeVisible();
   const pager=page.getByRole('navigation',{name:'Toolbox pages',exact:true});
   for(let n=2;n<=9;n++){await pager.getByRole('button',{name:'Next page',exact:true}).click();await expect(page.getByText(`${kind}-${201-(n-1)*24}`,{exact:true}).first()).toBeVisible();}
   await expect(page.getByText(`${kind}-1`,{exact:true}).first()).toBeVisible();await expect(pager.getByRole('button',{name:'Next page',exact:true})).toBeDisabled();await pager.getByRole('button',{name:'Previous page',exact:true}).click();await expect(page.getByText(`${kind}-33`,{exact:true}).first()).toBeVisible();
  }
  assert.equal(calls.filter(row=>row.page===9).length,3);assert.ok(calls.every(row=>row.size===24));
  await page.goto(base+'/jotting?lang=en');await expect(page.getByRole('heading',{name:'xocs',exact:true})).toBeVisible();await expect(page.getByRole('navigation',{name:'Main navigation',exact:true}).getByRole('link',{name:/About/})).toHaveCount(0);await expect(page.getByRole('navigation',{name:'Main navigation',exact:true}).getByRole('link',{name:/Journal/})).toBeVisible();assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth), 'Expanded current navigation fits desktop width');
  await page.route('**/api/v1/articles/1',route=>route.fulfill({json:{...article,update_time:'2026-10-12 04:05:06'}}));await page.goto(base+'/article/1?lang=en');await expect(page.locator('.article-end-info')).toContainText('2026-10-12 04:05:06 UTC');
  await page.route('**/api/v1/content/labels',route=>route.fulfill({json:[{id:1,sort_id:1,label_name:'Tag',label_description:null}]}));
  await page.route('**/api/v1/content/articles',route=>{if(route.request().method()!=='POST')return route.fallback();const input=route.request().postDataJSON();saved.push(input);return route.fulfill({json:{...article,id:42,...input,view_status:0,recommend_status:0,comment_status:1,password_required:1}});});
  await page.goto(base+'/admin?lang=en#new');await page.getByLabel('Title',{exact:true}).fill('Protected article');
  await page.getByRole('combobox',{name:'Category',exact:true}).click();await page.getByRole('option',{name:'中文分类',exact:true}).click();
  await page.getByRole('combobox',{name:'Tags',exact:true}).click();await page.getByRole('option',{name:'Tag',exact:true}).click();
  await expect(page.locator('.editor-form select').nth(0)).toHaveValue('1');await expect(page.locator('.editor-form select').nth(1)).toHaveValue('1');
  await page.getByLabel('Markdown content',{exact:true}).fill('Content');await page.getByRole('checkbox',{name:/Public access/}).uncheck();
  const password=page.getByLabel('Article access password',{exact:true});assert.equal(await password.getAttribute('minlength'),null);await password.fill('short');await page.getByRole('button',{name:'Save article',exact:true}).click();await expect(page.getByRole('alert').first()).toContainText('UTF-8 bytes');assert.equal(saved.length,0);
  await password.fill('密码密码');await page.getByRole('button',{name:'Save article',exact:true}).click();await expect.poll(()=>saved.length).toBe(1);assert.equal(saved[0].password,'密码密码');await expect(page.getByRole('heading',{name:'Articles',exact:true})).toBeVisible();
  await page.setViewportSize({width:390,height:844});await page.goto(base+'/favorite?lang=en');await expect(page.getByText('favorites-201',{exact:true})).toBeVisible();assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));await page.getByRole('button',{name:'Open navigation',exact:true}).click();await expect(page.getByRole('navigation',{name:'Main navigation',exact:true}).getByRole('link',{name:/About/})).toHaveCount(0);await expect(page.getByRole('navigation',{name:'Main navigation',exact:true}).getByRole('link',{name:/Journal/})).toBeVisible();
  assert.deepEqual(errors,[]);console.log(`${engine.name()}: Toolbox 3x201 records, native UTF-8 password form, UTC article update and current desktop/mobile menus passed`);
 }finally{await browser.close();}
}}finally{await new Promise(resolve=>server.httpServer.close(resolve));}
