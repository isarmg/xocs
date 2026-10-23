import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import { chromium, expect } from './dependencies.mjs';
import { pageOf, site } from './browser-fixtures.mjs';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dist = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../src/web/dist');
const base = process.env.BLOG_SMOKE_URL || 'http://blog.test';
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
try {
  const page = await browser.newPage({ locale: 'zh-CN' });
  const errors = [], wasmUrls = new Set();
  page.on('response', response => { if (new URL(response.url()).pathname.endsWith('.wasm')) wasmUrls.add(response.url()); });
  page.on('pageerror', error => errors.push(error.message));
  if (!process.env.BLOG_SMOKE_URL) await page.route('http://blog.test/**', async route => {
    const url = new URL(route.request().url());
    if (url.pathname.startsWith('/api/')) {
      const payload = url.pathname === '/api/v1/site'
        ? { ...site, web_title: '测试站点' }
        : url.pathname === '/api/v1/letter' ? { recipient:'Reader', sender:'Writer', content:'Smoke letter' }
        : url.pathname === '/api/v1/home-sections' ? [{id:1,title:'最新',kind:'latest',sort_id:null,priority:0,enabled:true}]
        : url.pathname === '/api/v1/categories' ? [{id:1,sort_name:'测试分类',sort_description:null,priority:0,article_count:0}]
        : url.pathname === '/api/v1/articles' ? pageOf([], url)
        : url.pathname === '/api/v1/links/page' ? {items:[],total:0,page:Number(url.searchParams.get('page'))||1,size:Number(url.searchParams.get('size'))||20}
        : url.pathname === '/api/v1/links/classes' ? []
        : url.pathname === '/api/v1/notes/page' ? {items:[],total:0,page:Number(url.searchParams.get('page'))||1,size:10}
        : ['/api/v1/family', '/api/v1/tree-hole'].includes(url.pathname) ? []
        : { error: 'unauthorized' };
      await route.fulfill({ status: 'error' in payload ? 401 : 200, contentType: 'application/json', body: JSON.stringify(payload) });
      return;
    }
    const asset = url.pathname.startsWith('/assets/') || /\.(png|svg)$/.test(url.pathname);
    const target = asset ? path.join(dist, url.pathname) : path.join(dist, 'index.html');
    const contentType = target.endsWith('.wasm') ? 'application/wasm' : target.endsWith('.js') ? 'application/javascript' : target.endsWith('.css') ? 'text/css' : target.endsWith('.woff2') ? 'font/woff2' : target.endsWith('.png') ? 'image/png' : target.endsWith('.svg') ? 'image/svg+xml' : 'text/html';
    try {
      await route.fulfill({ status: 200, contentType, body: await readFile(target) });
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
      await route.fulfill({ status: 404, body: '' });
    }
  });
  const homeHeading = process.env.BLOG_SMOKE_URL ? '相信记录的力量' : '测试站点';
  for (const [route, heading] of [['/', homeHeading], ['/search', '搜索文章'], ['/sort', '文章分类'], ['/jotting', 'blog'], ['/menory', 'blog'], ['/favorite', '百宝箱'], ['/favorite?tab=friends', '百宝箱'], ['/favorite?tab=music', '百宝箱'], ['/travel', '时光相册'], ['/love', '这是我们一起走过的'], ['/message', '弹幕']]) {
    await page.goto(`${base}${route}`);
    await expect(page.locator("html")).toHaveAttribute("data-blog-runtime", "blog-interface");
    await page.getByRole('heading', { name: heading }).first().waitFor();
    console.log(`${route}: rendered`);
  }
  if (process.env.BLOG_SMOKE_URL) {
    assert.ok(wasmUrls.size, 'Axum served an independent WASM artifact');
    const wasmUrl = [...wasmUrls][0];
    const identity = await page.request.get(wasmUrl, {headers: {'Accept-Encoding': 'identity'}});
    const gzip = await page.request.get(wasmUrl, {headers: {'Accept-Encoding': 'gzip'}});
    assert.equal(identity.status(), 200);assert.equal(gzip.status(), 200);
    assert.equal(identity.headers()['content-encoding'], undefined);
    assert.equal(gzip.headers()['content-encoding'], 'gzip');
    assert.equal(gzip.headers()['content-type'], 'application/wasm');
    assert.match(gzip.headers().vary, /(?:^|,)\s*Accept-Encoding\s*(?:,|$)/i);
    const raw = await identity.body();
    assert.ok(raw.equals(await gzip.body()), 'Negotiated gzip decodes to the exact WASM');
    assert.ok(Number(gzip.headers()['content-length']) < raw.length, 'The negotiated representation saves transfer bytes');
    assert.notEqual(gzip.headers().etag, identity.headers().etag);
    const metadata = await (await page.request.get(`${base}/assets/blog-assets.json`)).json();
    assert.equal(createHash('sha256').update(raw).digest('hex'), metadata.files['blog_browser_bg.wasm']);
    const head = await page.request.head(wasmUrl, {headers: {'Accept-Encoding': 'gzip'}});
    assert.equal(head.status(), 200);assert.equal((await head.body()).length, 0);
    assert.equal(head.headers()['content-length'], gzip.headers()['content-length']);
    assert.equal(head.headers().etag, gzip.headers().etag);
    const unchanged = await page.request.get(wasmUrl, {headers: {'Accept-Encoding': 'gzip', 'If-None-Match': gzip.headers().etag}});
    assert.equal(unchanged.status(), 304);assert.equal((await unchanged.body()).length, 0);
    const refused = await page.request.get(wasmUrl, {headers: {'Accept-Encoding': 'gzip;q=0, identity;q=1'}});
    assert.equal(refused.status(), 200);assert.equal(refused.headers()['content-encoding'], undefined);
    assert.ok(raw.equals(await refused.body()));
    console.log('Axum: WASM MIME, gzip bytes, source inventory, representation ETags, HEAD, 304 and gzip refusal passed');
  }
  if (!process.env.BLOG_SMOKE_URL) {
    await page.goto(`${base}/`);
    await page.locator('.aside-categories a[href="/sort?sort_id=1"]').click();
    await page.getByRole('heading', {name: '测试分类'}).first().waitFor();
  }
  await page.goto(`${base}/love`);
  await expect(page.locator('.love-tabs button')).toHaveCount(3);
  await expect(page.locator('.love-story-picker, .family-grid, .family-apply-dialog')).toHaveCount(0);
  await page.locator('.love-tabs').getByRole('button', {name: /点点滴滴/}).click();
  await page.goto(`${base}/letter`);
  await page.getByRole('button', { name: '打开信封' }).click();
  await page.getByRole('heading', { name: '致 Reader' }).waitFor();
  if (process.env.BLOG_SMOKE_URL) {
    await page.goto(`${base}/love`);
    await page.getByRole('button', { name: /祝福板/ }).click();
    await page.getByText('Smoke love wish').waitFor();
    await page.goto(`${base}/jotting`);
    await page.getByText('Smoke author note').waitFor();
    await page.goto(`${base}/`);
    await page.getByRole('heading', { name: '专栏速览' }).waitFor();
    await page.locator('.aside-recommended a').waitFor();
    await page.locator('.home-section[aria-label="专栏速览"] .section-heading a').click();
    if (!page.url().includes('sort_id=1')) throw new Error('Category section did not open its listing');
    await page.getByRole('heading', { name: 'Smoke' }).first().waitFor();
    await page.goto(`${base}/article/1`);
    await page.getByRole('heading', { name: 'Smoke' }).waitFor();
    await page.getByText('Smoke article news').waitFor();
    await page.getByRole('link', { name: 'Chapter One' }).first().click();
    if (!page.url().endsWith('#section-1')) throw new Error('Article table of contents did not navigate');
    await page.getByRole('progressbar', { name: '阅读进度' }).waitFor();
    await page.getByRole('textbox', { name: '写下你的想法', exact: true }).fill('匿名评论端到端测试');
    await page.getByRole('button', { name: '匿名发表评论', exact: true }).click();
    const anonymousComment = page.locator('.public-comment').filter({hasText:'匿名评论端到端测试'});
    await anonymousComment.getByText('匿名访客', {exact:true}).waitFor();
    await anonymousComment.getByRole('button', {name:'回复',exact:true}).click();
    await page.getByRole('textbox', { name: '写下你的想法', exact: true }).fill('匿名回复端到端测试');
    await page.getByRole('button', { name: '匿名发表评论', exact: true }).click();
    await page.locator('.comment-replies').getByText('匿名回复端到端测试', {exact:true}).waitFor();
    console.log('/article/1: rendered');
    await page.goto(`${base}/search?q=Chapter`);
    await page.getByRole('heading', { name: '搜索文章' }).waitFor();
    await page.getByText('正文匹配').waitFor();
    await page.locator('.search-snippet mark').first().waitFor();
    await page.goto(`${base}/message`);
    await page.getByRole('textbox', { name: '发送弹幕' }).fill('访客弹幕测试');
    await page.getByRole('button', { name: '发射' }).click();
    await page.getByText('访客弹幕测试').waitFor();
    const mobile = await browser.newPage({ locale: 'zh-CN', viewport: { width: 390, height: 844 } });
    mobile.on('pageerror', error => errors.push(error.message));
    await mobile.goto(`${base}/message`);
    await mobile.getByRole('textbox', { name: '发送弹幕' }).fill('手机端留言');
    await mobile.getByRole('button', { name: '发射' }).click();
    await mobile.getByText('手机端留言').waitFor();
    if (await mobile.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 2)) throw new Error('Mobile wall overflows viewport');
    await mobile.close();
    await page.goto(`${base}/article/3`);
    await page.getByRole('heading', { name: '这篇文章需要访问密码' }).waitFor();
    await page.locator('input[type="password"]').fill('ProtectedPassphrase123');
    await page.locator('form button[type="submit"]').click();
    await page.getByRole('heading', { name: 'Protected Preview' }).waitFor();
    console.log('/article/3: unlocked');
  }
  await page.goto(`${base}/admin`);
  await page.locator('body').getByText('blog', { exact: true }).first().waitFor();
  if (process.env.BLOG_SMOKE_URL) {
    await page.locator('input[name="username"]').fill('admin');
    await page.locator('input[name="password"]').fill('TemporaryPassphrase123');
    await page.locator('form button[type="submit"]').click();
    await page.getByRole('heading', { name: '文章管理' }).waitFor();
    for (const [group, label] of [['总览','总览'], ['设置','首页栏目'], ['内容','分类管理'], ['内容','标签管理'], ['社区','评论管理'], ['内容','随笔管理'], ['社区','留言板管理'], ['资源','文件管理'], ['资源','内容资源'], ['资源','恋爱笔记'], ['设置','网站设置']]) {
      await page.locator('.blog-header-navigation').getByRole('button', { name: group, exact: true }).click();
      if (group !== '总览') await page.locator('.blog-secondary-navigation').getByRole('button', { name: label, exact: true }).click();
      await page.getByRole('heading', { name: label }).waitFor();
      if (label === '评论管理') {
        const wish = page.getByRole('row').filter({hasText:'Smoke love wish'});
        await expect(wish.getByRole('cell').nth(1)).toHaveText('匿名访客');
        await expect(wish.getByRole('link', {name:'祝福板',exact:true})).toHaveAttribute('href','/love');
        const comment = page.getByRole('row').filter({hasText:'匿名评论端到端测试'});
        await expect(comment.getByRole('cell').nth(1)).toHaveText('匿名访客');
        const removed = page.waitForResponse(response => /\/api\/v1\/content\/comments\/\d+$/.test(response.url()) && response.request().method() === 'DELETE');
        page.once('dialog', dialog => dialog.accept());
        await comment.getByRole('button', {name:'删除',exact:true}).click();
        if (!(await removed).ok()) throw new Error('Anonymous comment moderation failed');
        await expect(comment).toHaveCount(0);
        await expect(page.getByRole('row').filter({hasText:'匿名回复端到端测试'})).toHaveCount(0);
      }
      if (label === '留言板管理') {
        const message = page.getByRole('row').filter({hasText:'Smoke anonymous message'});
        await expect(message.getByRole('cell').nth(1)).toHaveText('匿名访客');
      }
      if (label === '首页栏目') {
        await page.locator('table tbody tr').first().waitFor();
        if (await page.locator('table tbody tr').count() !== 3) throw new Error('Home section editor did not load all sections');
        await page.getByRole('button', { name: '上移栏目 3' }).click();
        const saved = page.waitForResponse(response => response.url().endsWith('/api/v1/content/home-sections') && response.request().method() === 'PUT');
        await page.getByRole('button', { name: '保存栏目' }).click();
        if (!(await saved).ok()) throw new Error('Home section reorder failed');
        const order = await page.evaluate(async () => (await (await fetch('/api/v1/home-sections')).json()).map(section => section.title));
        if (order[1] !== '专栏速览') throw new Error('Home section order was not persisted');
      }
    }
    if (process.env.BLOG_SMOKE_SCREENSHOT) await page.screenshot({ path: process.env.BLOG_SMOKE_SCREENSHOT, fullPage: true });
    console.log('/admin: signed in and navigated');
  }
  console.log('/admin: rendered');
  if (errors.length) throw new Error(`Browser errors: ${errors.join('; ')}`);
} finally {
  await browser.close();
}
