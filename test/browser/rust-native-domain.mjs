// New Rust-domain coverage. Production assets and WASM are real; API payloads
// and the OS clipboard are explicit disposable adapters, never live writes.
import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { article, mockApi, session, waitForApplication } from './browser-fixtures.mjs';

const requested = (process.env.BLOG_DOMAIN_ENGINES || 'chromium,firefox').split(',');
assert.ok(requested.length && new Set(requested).size === requested.length && requested.every(name => ['chromium','firefox'].includes(name)));
const external = process.env.BLOG_WEB_URL;
if (external) {
  const url = new URL(external);
  assert.equal(url.protocol, 'http:');
  assert.ok(['127.0.0.1','[::1]'].includes(url.hostname), 'Test assets require an explicit loopback listener');
}
const server = external ? null : await preview({preview:{host:'127.0.0.1',port:0,strictPort:true}});
const base = external || `http://127.0.0.1:${server.httpServer.address().port}`;
const pixel = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=', 'base64');
const code = `const literal = '<img src=x onerror=alert(1)>';`;
const markdown = [
  '## Chapter **One**', 'A line\nA second line',
  '### A smaller chapter', '#### Details',
  `\`\`\`js\n${code}\n\`\`\``,
  '![A " photo](/media/domain-test.png)',
  '| First | Second |\n| --- | --- |\n| one | ~~old~~ |',
  'Visit https://example.org/path, www.example.com or test@example.org.',
  'Bare links: example.org:8080/docs, www.example.com/docs/part:one and example.org/search?url=https://example.net.',
  '[Local link](/sort?q=rust#top)',
  '<script>window.__domainXss=1</script>',
  '<img src=x onerror="window.__domainXss=2">',
  '<svg><a xlink:href="javascript:window.__domainXss=3">bad</a></svg>',
  '<math><mtext><table><mglyph><style><!--</style><img title="--><img src=1 onerror=alert(1)>">',
  '[blocked-1](javascript:alert%281%29)',
  '[blocked-2](jav&#x61;script:alert%281%29)',
  '[blocked-3](java&#x09;script:alert%281%29)',
  '[blocked-4](vbscript:msgbox%281%29)',
  '[blocked-5](data:text/html;base64,PHNjcmlwdD4=)',
  '[blocked-6](file:///etc/passwd)',
  '![blocked-image](data:image/svg+xml;base64,PHN2Zy8+)',
].join('\n\n');
const summary = { base, api:'disposable route fixtures', clipboard:'writeText adapter', engines:[], all_passed:false };
try {
  for (const engine of [chromium,firefox].filter(engine => requested.includes(engine.name()))) {
    const browser = await engine.launch({headless:true});
    const result = {engine:engine.name(),version:browser.version(),groups:[],all_passed:false};
    summary.engines.push(result);
    const context = await browser.newContext({locale:'en-US',viewport:{width:1280,height:900},reducedMotion:'reduce'});
    const page = await context.newPage(), errors = [], posts = [];
    page.setDefaultTimeout(30_000);
    page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(() => {
      window.__domainXss = 0;
      window.__domainClipboard = [];
      Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text => window.__domainClipboard.push(text)}});
      window.__domainSelectTrace = [];
      for (const type of ['pointerdown','pointerup','click']) document.addEventListener(type,event => {
        const target = event.target instanceof Element ? event.target : null;
        const option = target?.closest('[role=option]'), panel = option?.closest('[role=listbox]');
        window.__domainSelectTrace.push({type,option:option?.getAttribute('data-option-index')??null,panelTop:panel?.getBoundingClientRect().top??null,scrollY,tag:target?.tagName});
        if(window.__domainSelectTrace.length>50)window.__domainSelectTrace.shift();
      },true);
    });
    await mockApi(page,{authenticated:true});
    let categories = [
      {id:1,sort_name:'Smoke',sort_description:null,priority:0,article_count:0},
      {id:2,sort_name:'New category',sort_description:null,priority:1,article_count:0},
    ];
    let labels = [];
    await page.route('**/media/domain-test.png',route=>route.fulfill({contentType:'image/png',body:pixel}));
    await page.route('**/api/v1/**', async route => {
      const req=route.request(),url=new URL(req.url()),path=url.pathname,method=req.method();
      if(path==='/api/v1/categories')return route.fulfill({json:categories});
      if(path==='/api/v1/articles/1')return route.fulfill({json:{...article,article_title:'Rust domain article',article_content:markdown}});
      if(path==='/api/v1/content/labels'){
        if(method==='POST'){
          assert.equal(req.headers()['x-csrf-token'],session.csrf_token);
          const body=req.postDataJSON();posts.push(body);
          assert.deepEqual(body,{name:'Rust tag',description:'Typed category selection',sort_id:2});
          const value={id:1,sort_id:body.sort_id,label_name:body.name,label_description:body.description};
          labels=[value];return route.fulfill({json:value});
        }
        return route.fulfill({json:labels});
      }
      return route.fallback();
    });
    try {
      await page.goto(`${base}/article/1?lang=en`);
      await waitForApplication(page);
      const body=page.locator('.article-markdown');
      await expect(body.locator('h2')).toHaveText('Chapter One');
      await expect(body.locator('h2,h3,h4')).toHaveCount(3);
      assert.deepEqual(await body.locator('h2,h3,h4').evaluateAll(nodes=>nodes.map(n=>n.id)),['section-1','section-2','section-3']);
      const outline=page.locator('.reading-toc nav a');
      assert.deepEqual(await outline.allTextContents(),['Chapter One','A smaller chapter','Details']);
      await expect(outline.first()).toHaveAttribute('href','#section-1');
      await outline.first().click();assert.ok(page.url().endsWith('#section-1'));
      await expect(body.locator('table')).toHaveCount(1);
      await expect(body.locator('del')).toHaveText('old');
      assert.ok(await body.locator('p').filter({hasText:'A line'}).locator('br').count());
      result.groups.push('headings, outline IDs, navigation, soft breaks, table and strikethrough');

      for(const [name,href]of[['https://example.org/path','https://example.org/path'],['www.example.com','http://www.example.com'],['test@example.org','mailto:test@example.org'],['Local link','/sort?q=rust#top']]){
        await expect(body.getByRole('link',{name,exact:true})).toHaveAttribute('href',href);
      }
      for(const destination of ['example.org:8080/docs','www.example.com/docs/part:one','example.org/search?url=https://example.net']){
        const link=body.getByRole('link',{name:destination,exact:true});
        await expect(link).toHaveAttribute('href',`http://${destination}`);
        assert.equal(await link.evaluate(node=>new URL(node.href).origin),new URL(`http://${destination}`).origin);
      }
      await expect(body.locator('pre code')).toHaveText(`${code}\n`);
      await body.getByRole('button',{name:'Copy code',exact:true}).click();
      await expect(page.getByRole('status').filter({hasText:'Code copied'})).toBeVisible();
      assert.deepEqual(await page.evaluate(()=>window.__domainClipboard),[`${code}\n`]);
      result.groups.push('bare URLs, email, relative links, escaped code and clipboard adapter');

      const image=body.getByRole('button',{name:'View image: A " photo',exact:true});
      await expect(image).toHaveAttribute('tabindex','0');
      await expect.poll(()=>image.evaluate(node=>node.complete&&node.naturalWidth===1)).toBe(true);
      await image.focus();await page.keyboard.press('Enter');
      const dialog=page.getByRole('dialog',{name:'View image',exact:true});
      await expect(dialog).toBeVisible();await expect(dialog.locator('img')).toHaveAttribute('src',`${base}/media/domain-test.png`);
      await dialog.getByRole('button',{name:'Close image',exact:true}).click();await expect(dialog).toHaveCount(0);
      await expect(image).toBeFocused();
      result.groups.push('image alt escaping, keyboard activation, lightbox and focus restoration');

      await expect(body.locator('script,svg,math,style,iframe,object,embed')).toHaveCount(0);
      await expect(body.locator('img')).toHaveCount(1);
      assert.equal(await page.evaluate(()=>window.__domainXss),0);
      const dangerous=await body.evaluate(node=>[...node.querySelectorAll('*')].flatMap(element=>[...element.attributes].filter(attr=>/^on/i.test(attr.name)||['href','src'].includes(attr.name)&&!['http:','https:','mailto:','ftp:','ftps:','tel:'].includes(new URL(attr.value,location.href).protocol)).map(attr=>`${element.tagName}:${attr.name}=${attr.value}`)));
      assert.deepEqual(dangerous,[]);
      for(let i=1;i<=6;i++)await expect(body.getByRole('link',{name:`blocked-${i}`,exact:true})).toHaveCount(0);
      result.groups.push('raw HTML/SVG/MathML and entity/control-character/dangerous-protocol XSS guards');

      await page.goto(`${base}/admin?lang=en#labels`);await waitForApplication(page);
      await page.getByLabel('Tag name',{exact:true}).fill('Rust tag');
      await page.getByLabel('Tag description',{exact:true}).fill('Typed category selection');
      const select=page.getByRole('combobox',{name:'Category',exact:true});
      await select.click();
      const option=page.getByRole('option',{name:'New category',exact:true});
      await option.waitFor();
      const point=await option.boundingBox();assert.ok(point);
      // One fixed coordinate and one down/up gesture, with no click retry or
      // coordinate reread after hover. Let the original reactive race settle
      // while the pointer is down, so a popup move would fail this assertion.
      await page.mouse.move(point.x+point.width/2,point.y+point.height/2);
      await page.mouse.down();
      await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
      await page.mouse.up();
      await expect(select).toContainText('New category');
      await expect(select.locator('..').locator('select')).toHaveValue('2');
      const trace=await page.evaluate(()=>window.__domainSelectTrace);
      const down=trace.filter(item=>item.type==='pointerdown').at(-1),up=trace.filter(item=>item.type==='pointerup').at(-1);
      assert.equal(down.option,'2');assert.equal(up.option,'2');
      assert.ok(Math.abs(down.panelTop-up.panelTop)<1,JSON.stringify({down,up}));
      result.pointer_selection={down,up};
      await page.getByRole('button',{name:'Add tag',exact:true}).click();
      await expect(page.getByRole('table',{name:'Tag list',exact:true})).toContainText('Rust tag');
      assert.equal(posts.length,1);
      result.groups.push('original Tags→Category pointer path, stable down/up geometry and exact POST schema');

      categories=Array.from({length:28},(_,index)=>({id:index+1,sort_name:`Category ${String(index+1).padStart(2,'0')}`,sort_description:null,priority:index,article_count:0}));
      await page.reload();await waitForApplication(page);
      const keyboard=page.getByRole('combobox',{name:'Category',exact:true});
      await keyboard.focus();
      const beforeKeyboardScroll=await page.evaluate(()=>scrollY);
      await page.keyboard.press('End');
      await expect(page.getByRole('listbox',{name:'Category',exact:true})).toBeVisible();
      const bounds=await page.getByRole('listbox',{name:'Category',exact:true}).evaluate(panel=>({scroll:panel.scrollTop,panel:panel.getBoundingClientRect().toJSON(),last:panel.querySelector('[data-option-index="28"]').getBoundingClientRect().toJSON(),windowScroll:scrollY}));
      assert.ok(bounds.scroll>0);assert.ok(bounds.last.top>=bounds.panel.top-1&&bounds.last.bottom<=bounds.panel.bottom+1,JSON.stringify(bounds));
      assert.equal(bounds.windowScroll,beforeKeyboardScroll,'Keyboard reveal must scroll only the option list');
      await page.keyboard.press('Enter');await expect(keyboard).toContainText('Category 28');
      await keyboard.click();
      await keyboard.evaluate(node=>{node.closest('fieldset').disabled=true;});
      await expect(keyboard).toHaveAttribute('aria-expanded','false');
      await keyboard.evaluate(node=>{node.closest('fieldset').disabled=false;});
      await keyboard.focus();await page.keyboard.press('Home');await page.keyboard.press('Enter');
      await expect(keyboard.locator('..').locator('select')).toHaveValue('');
      assert.equal(await keyboard.locator('..').locator('select').evaluate(node=>node.checkValidity()),false);
      result.groups.push('long-list keyboard scrolling, ancestor disabled cleanup and native required validation');
      assert.deepEqual(errors,[]);
      result.all_passed=true;
      console.log(`${engine.name()}: ${result.groups.length} Rust-domain groups passed`);
    } catch(error) {
      result.error={name:error.name,message:error.message};
      result.pointer_trace=await page.evaluate(()=>window.__domainSelectTrace).catch(()=>[]);
      throw error;
    } finally { await browser.close(); }
  }
  summary.all_passed=true;
} finally {
  if(server)await server.close();
  if(process.env.BLOG_DOMAIN_REPORT)await writeFile(process.env.BLOG_DOMAIN_REPORT,JSON.stringify(summary,null,2)+'\n');
}
