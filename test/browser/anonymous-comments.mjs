import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { AxeBuilder } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi, pageOf } from './browser-fixtures.mjs';

async function checkHeaderScrolling(page, width, english) {
  const articleUrl = page.url();
  const header = page.locator('.public-header');
  const homeLink = header.locator('nav > a[href="/"]');
  await expect(page.locator('.comment-form textarea')).toBeVisible();
  await expect(header).toBeVisible();
  await page.evaluate(() => scrollTo(0, 180));
  await expect(header).toBeHidden();
  await page.evaluate(() => scrollBy(0, -60));
  await expect(header).toBeVisible();
  await page.evaluate(async () => { scrollBy(0, 1); await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))); });
  await expect(header).toBeVisible();
  await page.evaluate(() => scrollBy(0, 100));
  await expect(header).toBeHidden();
  await page.evaluate(() => scrollTo(0, 0));
  await expect(header).toBeVisible();
  if (width === 390) {
    await page.getByRole('button', { name: english ? 'Open navigation' : '打开导航', exact: true }).click();
    await page.evaluate(() => scrollTo(0, 180));
    await expect(header).toBeVisible();
    await expect(page.locator('.public-header > nav')).toBeVisible();
    await page.getByRole('button', { name: english ? 'Close navigation' : '关闭导航', exact: true }).click();
    await page.evaluate(() => scrollBy(0, 100));
    await expect(header).toBeHidden();
    await page.evaluate(() => scrollTo(0, 0));
    await expect(header).toBeVisible();
  }
  if (width === 390) {
    await page.getByRole('button', { name: english ? 'Open navigation' : '打开导航', exact: true }).click();
  }
  await homeLink.hover();
  assert.equal(await homeLink.getAttribute('aria-expanded'), null);
  await expect(header.locator('nav > a[href="/sort"]')).toBeVisible();
  await homeLink.click();
  await expect.poll(()=>{const url=new URL(page.url());return url.pathname==='/'&&url.searchParams.get('lang')===(english?'en':'zh-CN');}).toBe(true);
  await expect(page.getByRole('heading', { name: english ? 'Believe in the power of stories' : '相信记录的力量', exact: true })).toBeVisible();
  await page.goto(articleUrl);
  await expect(page.locator('.comment-form textarea')).toBeVisible();
  if (width === 390) await page.getByRole('button', { name: english ? 'Open navigation' : '打开导航', exact: true }).click();
  await homeLink.focus();
  await page.keyboard.press('Tab');
  await expect(header.locator('nav > a[href="/love"]')).toBeFocused();
  if (width === 390) await page.getByRole('button', { name: english ? 'Close navigation' : '关闭导航', exact: true }).click();
  await page.locator('.public-header .brand').focus();
  await page.keyboard.press('Tab');
  await expect.poll(() => page.evaluate(() => document.activeElement?.matches('.public-header :focus-visible'))).toBe(true);
  await page.evaluate(() => scrollTo(0, 180));
  await expect(header).toBeVisible();
  await page.evaluate(() => document.activeElement.blur());
  await page.evaluate(() => scrollBy(0, 100));
  await expect(header).toBeHidden();
  await page.evaluate(() => scrollTo(0, 0));
  await expect(header).toBeVisible();
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const width of [390, 1920]) for (const language of ['zh-CN', 'en']) {
        const context = await browser.newContext({ locale: language, viewport: { width, height: 1080 }, reducedMotion: 'reduce' });
        const page = await context.newPage();
        const errors = [], retiredRequests = [], stores = new Map(); let nextId = 1, failNext = false;
        page.on('pageerror', error => errors.push(error.message));
        page.on('request', request => { if (/\/api\/v1\/(?:members|im)\//.test(request.url())) retiredRequests.push(request.url()); });
        await mockApi(page, { authenticated: true });
        await page.route(/\/api\/v1\/(?:comments|message-comments|love-comments)(?:\/[^?]+)?(?:\?.*)?$/, route => {
          const request = route.request(), url = new URL(request.url()), path = url.pathname.replace(/\/\d+\/replies$/, '');
          const items = stores.get(path) || []; stores.set(path, items);
          if (request.method() === 'POST') {
            assert.equal(request.headers()['x-csrf-token'], undefined);
            const payload = request.postDataJSON();
            assert.deepEqual(Object.keys(payload).sort(), (path === '/api/v1/comments' ? ['request_id', 'article_id', 'content', 'parent_comment_id'] : ['request_id', 'content', 'parent_comment_id']).sort());
            if (failNext) { failNext = false; return route.fulfill({ status: 429, json: { code: 'too_many_requests', message: 'Too many requests', retryable: true } }); }
            const parent = items.find(item => item.id === payload.parent_comment_id);
            const comment = { id: nextId++, user_id: null, username: null, avatar: null, comment_content: payload.content, create_time: '2026-10-08', parent_comment_id: payload.parent_comment_id, parent_username: null, floor_comment_id: parent?.floor_comment_id || parent?.id || null, reply_count: 0 };
            if (parent) items.find(item => item.id === comment.floor_comment_id).reply_count++;
            items.unshift(comment); return route.fulfill({ json: comment });
          }
          const root = /\/(\d+)\/replies$/.exec(url.pathname);
          const visible = items.filter(item => root ? item.floor_comment_id === Number(root[1]) : !item.parent_comment_id);
          return route.fulfill({ json: pageOf(visible, url) });
        });
        const english = language === 'en';
        const button = english ? 'Post anonymous comment' : '匿名发表评论';
        for (const path of ['/article/1', '/love', '/message']) {
          await page.goto(`${base}${path}?lang=${language}`);
          if (path === '/article/1') await checkHeaderScrolling(page, width, english);
          if (path === '/love') await page.locator('.love-tabs').getByRole('button', { name: english ? /^Wishes/ : /^祝福板/ }).click();
          const form = page.locator('.comment-form'), comments = page.locator('.article-comments');
          await expect(page.locator('a[href="/im"],a[href="/user"],.public-login-link,.family-apply-dialog')).toHaveCount(0);
          const input = form.getByRole('textbox', { name: english ? 'Write your thoughts' : '写下你的想法', exact: true });
          await input.fill(`Anonymous comment on ${path}`);
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.public-comment').first()).toContainText(`Anonymous comment on ${path}`);
          await expect(comments.locator('.public-comment-head strong').first()).toHaveText(english ? 'Anonymous guest' : '匿名访客');
          await comments.locator('.public-comment').first().getByRole('button', { name: english ? 'Reply' : '回复', exact: true }).click();
          await input.fill('Anonymous reply');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.comment-replies')).toContainText('Anonymous reply');
          failNext = true; await input.fill('Keep this draft on failure');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.getByRole('alert')).toBeVisible();
          await expect(input).toHaveValue('Keep this draft on failure');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.getByRole('alert')).toHaveCount(0);
          await expect(comments.locator('.public-comment').first()).toContainText('Keep this draft on failure');
          assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
          const header = await page.locator('.public-header').evaluate(element => {
            const surface = element.querySelector(':scope > [data-blog-server-corner-decoration]') || element;
            const style = getComputedStyle(surface);
            return { background: style.backgroundColor, blur: style.backdropFilter };
          });
          const alpha = Number(header.background.split(',').at(-1).replace(')', ''));
          assert.ok(alpha > .4 && alpha < .8, JSON.stringify(header)); assert.match(header.blur, /blur\(16px\)/);
        }
        await page.evaluate(async () => {
          scrollTo(0, 0);
          await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        });
        await expect(page.locator('.public-header')).toBeVisible();
        await page.getByRole('button', { name: english ? 'Switch to dark mode' : '切换到深色模式', exact: true }).click();
        if (width === 1920) {
          const { violations } = await new AxeBuilder({ page }).include('.message-comment-wrap').withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
          assert.deepEqual(violations.map(item => ({ id: item.id, nodes: item.nodes.map(node => node.target) })), []);
        }
        assert.deepEqual(retiredRequests, []); assert.deepEqual(errors, []);
        await context.close();
        console.log(`${engine.name()} ${width}px ${language}: header scrolling, anonymous comments, replies, retries and frosted navigation passed`);
      }
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
