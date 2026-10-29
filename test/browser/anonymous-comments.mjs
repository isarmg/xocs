import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { AxeBuilder } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi, pageOf } from './browser-fixtures.mjs';

async function checkHeaderScrolling(page, width, english) {
  const articleUrl = page.url();
  const header = page.locator('.public-header');
  const homeLink = header.locator(`nav > a[href="/?lang=${english ? 'en' : 'zh-CN'}"]`);
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
  await expect(header.locator(`nav > a[href="/sort?lang=${english ? 'en' : 'zh-CN'}"]`)).toBeVisible();
  await homeLink.click();
  await expect.poll(()=>{const url=new URL(page.url());return url.pathname==='/'&&url.searchParams.get('lang')===(english?'en':'zh-CN');}).toBe(true);
  await expect(page.getByRole('heading', { name: english ? 'Believe in the power of stories' : '相信记录的力量', exact: true })).toBeVisible();
  await page.goto(articleUrl);
  await expect(page.locator('.comment-form textarea')).toBeVisible();
  if (width === 390) await page.getByRole('button', { name: english ? 'Open navigation' : '打开导航', exact: true }).click();
  await homeLink.focus();
  await page.keyboard.press('Tab');
  await expect(header.locator(`nav > a[href="/love?lang=${english ? 'en' : 'zh-CN'}"]`)).toBeFocused();
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
        await page.route('**/api/v1/tree-hole/guest', route => {
          const payload = route.request().postDataJSON();
          return route.fulfill({ json: { id: 10000, user_id: null, username: null, avatar: null,
            message: payload.message, image_path: null, create_time: null } });
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
          const literal = `a < b; <3; <img src=x onerror="window.commentXss=1"> ${path}`;
          await input.fill(literal);
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.public-comment').first()).toContainText(literal);
          await expect(comments.locator('.public-comment img[src="x"]')).toHaveCount(0);
          assert.equal(await page.evaluate(() => window.commentXss), undefined);
          await input.fill(`Anonymous comment on ${path}`);
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.public-comment').first()).toContainText(`Anonymous comment on ${path}`);
          await expect(comments.locator('.public-comment-head strong').first()).toHaveText(english ? 'Anonymous guest' : '匿名访客');
          await comments.locator('.public-comment').first().getByRole('button', { name: english ? 'Reply' : '回复', exact: true }).click();
          await input.fill('Anonymous reply');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.comment-floor').filter({ hasText: `Anonymous comment on ${path}` }).locator('.comment-replies')).toContainText('Anonymous reply');
          failNext = true; await input.fill('Keep this draft on failure');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.getByRole('alert')).toBeVisible();
          await expect(input).toHaveValue('Keep this draft on failure');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.getByRole('alert')).toHaveCount(0);
          await expect(comments.locator('.public-comment').first()).toContainText('Keep this draft on failure');
          if (path === '/message') {
            const literal = 'a < b; <3; <img src=x onerror="window.commentXss=1">';
            await page.getByRole('textbox', { name: english ? 'Send a message' : '发送弹幕', exact: true }).fill(literal);
            await page.getByRole('button', { name: english ? 'Send' : '发射', exact: true }).click();
            await expect(page.locator('.message-barrage-item').first()).toHaveText(literal);
            await expect(page.locator('.message-barrage-item img[src="x"]')).toHaveCount(0);
            assert.equal(await page.evaluate(() => window.commentXss), undefined);
          }
          assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
          const header = await page.locator('.public-header').evaluate(element => {
            const surface = element.querySelector(':scope > [data-blog-server-corner-decoration]') || element;
            const style = getComputedStyle(surface);
            return { background: style.backgroundColor, blur: style.backdropFilter };
          });
          const alpha = Number(header.background.split(',').at(-1).replace(')', ''));
          assert.ok(alpha > .4 && alpha < .8, JSON.stringify(header)); assert.match(header.blur, /blur\(16px\)/);
        }
        if (width === 1920 && english) {
          await page.goto(`${base}/article/1?lang=en`);
          const comments = page.locator('.article-comments');
          const form = comments.locator('.comment-form');
          const input = form.getByRole('textbox', { name: 'Write your thoughts', exact: true });
          await input.fill('Posted even when pending cleanup fails');
          await page.evaluate(() => {
            window.restoreCommentStorage = Storage.prototype.removeItem;
            Storage.prototype.removeItem = function(key) {
              if (key.startsWith('blog.comment-submission:')) throw new DOMException('Storage access revoked', 'SecurityError');
              return window.restoreCommentStorage.call(this, key);
            };
          });
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.public-comment').first()).toContainText('Posted even when pending cleanup fails');
          await expect(comments.getByRole('alert')).toContainText('The pending submission could not be read or saved.');
          await expect(form.getByRole('button', { name: button, exact: true })).toBeDisabled();
          await expect.poll(() => page.evaluate(() => sessionStorage.getItem('blog.comment-submission:/api/v1/comments:1') !== null)).toBe(true);
          await page.evaluate(() => {
            Storage.prototype.removeItem = window.restoreCommentStorage;
            delete window.restoreCommentStorage;
          });
          page.once('dialog', dialog => dialog.accept());
          await form.getByRole('button', { name: 'Clear pending submission', exact: true }).click();
          await expect(comments.getByRole('alert')).toHaveCount(0);
          await expect(input).toBeEnabled();
          await expect.poll(() => page.evaluate(() => sessionStorage.getItem('blog.comment-submission:/api/v1/comments:1'))).toBe(null);
          await input.fill('New submission after storage recovery');
          await form.getByRole('button', { name: button, exact: true }).click();
          await expect(comments.locator('.public-comment').first()).toContainText('New submission after storage recovery');
          await page.goto(`${base}/message?lang=en`);
          await expect(page.locator('.message-comment-wrap')).toBeVisible();
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
