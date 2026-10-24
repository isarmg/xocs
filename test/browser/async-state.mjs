import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi, pageOf, summary } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = server.resolvedUrls.local[0];
const settle = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const failed = route => route.fulfill({ status: 503, json: { code: 'unavailable', retryable: true } });
const cases = [
  ['comments', 'comment_content', { id: 42, source: 0, comment_type: 'message', user_id: null, username: null, create_time: null }],
  ['resources', 'original_name', { id: 42, path: '/media/old.png', resource_type: 'image', size: 12, status: 1, create_time: null }],
  ['tree-hole', 'message', { id: 42, user_id: null, username: null, avatar: null, image_path: null, create_time: null }],
  ['notes', 'content', { id: 42, user_id: 1, username: 'Owner', image_path: null, is_public: 1, create_time: null }],
  ['links', 'title', { id: 42, classify: null, cover: null, url: null, introduction: null, link_type: 'favorites', status: 1, create_time: null }],
];

try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 } });
      const errors = [], wasm = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('request', request => { if (new URL(request.url()).pathname.endsWith('.wasm')) wasm.push(request.url()); });
      page.on('dialog', dialog => dialog.accept());
      await page.addInitScript(() => {
        const nativeFetch = window.fetch.bind(window);
        window.asyncNotices = [];
        window.addEventListener('blog-notify', event => window.asyncNotices.push(event.detail));
        // Delay a response only after native fetch has succeeded. Rust still
        // owns its AbortSignal, decoding and completion callbacks.
        window.fetch = async (input, options = {}) => {
          const response = await nativeFetch(input, options);
          const path = new URL(input instanceof Request ? input.url : input, location.href).pathname;
          if (window.asyncHold?.path === path && window.asyncHold.method === (options.method ?? 'GET')) {
            window.asyncHold = null;
            await new Promise(resolve => { window.asyncHeld = { release: resolve, signal: options.signal }; });
          }
          return response;
        };
      });
      await mockApi(page, { authenticated: true });

      for (const [kind, field, record] of cases) {
        const reads = [], pending = [];
        const pattern = `**/api/v1/content/${kind}?*`;
        await page.route(pattern, route => {
          const request = route.request(), url = new URL(request.url());
          assert.equal(request.method(), 'GET'); reads.push(url.pathname + url.search);
          const number = Number(url.searchParams.get('page'));
          if (number === 2) { pending.push(route); return; }
          return route.fulfill({ json: { items: [{ ...record, [field]: `Old ${kind} row` }], total: 21, page: number, size: 20 } });
        });
        await page.goto(`${base}admin?lang=en#${kind}`);
        await expect(page.locator('html')).toHaveAttribute('data-blog-runtime', 'blog-interface');
        await expect(page.getByRole('table')).toContainText(`Old ${kind} row`);
        if (kind === 'notes') await page.getByLabel('Note content', { exact: true }).fill('Unsaved note');
        await page.getByRole('button', { name: 'Next page', exact: true }).click();
        await expect.poll(() => pending.length).toBe(1);
        await expect(page.getByRole('table')).toHaveCount(0);
        await expect(page.getByRole('button', { name: /^(Delete|Approve|Disable|Edit)$/, exact: true })).toHaveCount(0);
        await failed(pending.shift());
        await expect(page.getByRole('alert')).toBeVisible();
        await expect(page.locator('body')).not.toContainText(`Old ${kind} row`);
        await page.getByRole('button', { name: 'Try again', exact: true }).click();
        await expect.poll(() => pending.length).toBe(1);
        assert.deepEqual(reads, [`/api/v1/content/${kind}?page=1&size=20`, `/api/v1/content/${kind}?page=2&size=20`, `/api/v1/content/${kind}?page=2&size=20`]);
        await pending.shift().fulfill({ json: { items: [{ ...record, [field]: `Current ${kind} row` }], total: 21, page: 2, size: 20 } });
        await expect(page.getByRole('table')).toContainText(`Current ${kind} row`);
        await expect(page.getByRole('navigation', { name: 'Pagination', exact: true })).toContainText('Page 2');
        if (kind === 'notes') await expect(page.getByLabel('Note content', { exact: true })).toHaveValue('Unsaved note');
        await page.unroute(pattern);
      }

      const articles = Array.from({ length: 16 }, (_, index) => ({ ...summary, id: index + 1, article_title: `Deletion row ${index + 1}` }));
      const articleReads = [], deletes = [];
      let pendingDelete, immediateDelete = false;
      await page.route('**/api/v1/content/articles?*', route => {
        const url = new URL(route.request().url()); articleReads.push(Number(url.searchParams.get('page')));
        return route.fulfill({ json: pageOf(articles, url) });
      });
      await page.route(/\/api\/v1\/content\/articles\/\d+$/, route => {
        assert.equal(route.request().method(), 'DELETE'); deletes.push(new URL(route.request().url()).pathname);
        if (immediateDelete) return route.fulfill({ json: { deleted: true } });
        pendingDelete = route;
      });
      await page.goto(`${base}admin?lang=en#articles`);
      await page.getByRole('button', { name: 'Next page', exact: true }).click();
      let deletion = page.getByRole('row').filter({ hasText: 'Deletion row 16' }).getByRole('button', { name: 'Delete', exact: true });
      await deletion.click(); await expect.poll(() => Boolean(pendingDelete)).toBe(true);
      await expect(deletion).toBeDisabled();
      await deletion.evaluate(button => { for (let index = 0; index < 2; index += 1) button.dispatchEvent(new MouseEvent('click', { bubbles: true })); });
      await settle(page); assert.equal(deletes.length, 1, 'pending DELETE ignores duplicate events');
      await failed(pendingDelete); pendingDelete = null;
      await expect(deletion).toBeEnabled();
      await deletion.click(); await expect.poll(() => Boolean(pendingDelete)).toBe(true);
      articles.pop(); await pendingDelete.fulfill({ json: { deleted: true } }); pendingDelete = null;
      await expect(page.getByRole('table', { name: 'Article list', exact: true }).locator('tbody tr')).toHaveCount(15);
      assert.deepEqual(articleReads, [1, 2, 2, 1], 'deleting a sole last-page row reloads the clamped first page');
      assert.equal(deletes.length, 2);

      immediateDelete = true;
      await page.evaluate(() => { window.asyncHold = { path: '/api/v1/content/articles/1', method: 'DELETE' }; window.asyncHeld = null; });
      await page.getByRole('row').filter({ hasText: /^Deletion row 1#/ }).getByRole('button', { name: 'Delete', exact: true }).click();
      await expect.poll(() => page.evaluate(() => Boolean(window.asyncHeld))).toBe(true);
      await page.evaluate(() => { window.asyncNotices = []; location.hash = 'notes'; });
      await expect(page.getByRole('heading', { name: 'Notes', exact: true })).toBeVisible();
      await page.getByLabel('Note content', { exact: true }).fill('New page draft');
      assert.equal(await page.evaluate(() => window.asyncHeld.signal.aborted), true);
      await page.evaluate(() => window.asyncHeld.release()); await settle(page);
      await expect(page.getByLabel('Note content', { exact: true })).toHaveValue('New page draft');
      assert.deepEqual(await page.evaluate(() => window.asyncNotices), [], 'late deletion cannot notify or reload the departed page');

      await page.route('**/api/v1/content/upload', route => route.fulfill({ json: { id: 1, path: `/media/${'a'.repeat(64)}.png`, mime_type: 'image/png', size: 1 } }));
      await page.goto(`${base}admin?lang=en#new`);
      await page.getByLabel('Title', { exact: true }).fill('Departed article');
      await page.evaluate(() => { window.asyncHold = { path: '/api/v1/content/upload', method: 'POST' }; window.asyncHeld = null; });
      const chooser = page.waitForEvent('filechooser');
      await page.getByRole('button', { name: 'Choose file: Upload cover image', exact: true }).click();
      await (await chooser).setFiles({ name: 'late.png', mimeType: 'image/png', buffer: Buffer.from([1]) });
      await expect.poll(() => page.evaluate(() => Boolean(window.asyncHeld))).toBe(true);
      await expect(page.getByRole('button', { name: 'Uploading media…', exact: true })).toBeDisabled();
      await page.getByRole('button', { name: 'Cancel', exact: true }).click();
      await page.getByRole('button', { name: 'Add article', exact: true }).click();
      await page.getByLabel('Title', { exact: true }).fill('Fresh article draft');
      assert.equal(await page.evaluate(() => window.asyncHeld.signal.aborted), true);
      await page.evaluate(() => { window.asyncNotices = []; window.asyncHeld.release(); }); await settle(page);
      await expect(page.getByLabel('Title', { exact: true })).toHaveValue('Fresh article draft');
      await expect(page.getByLabel('Cover image URL', { exact: true })).toHaveValue('');
      assert.deepEqual(await page.evaluate(() => window.asyncNotices), [], 'a cancelled upload cannot mutate or notify the new editor');

      const comments = Array.from({ length: 11 }, (_, index) => ({ id: index + 1, user_id: null, username: null, avatar: null, comment_content: `Recovery comment ${index + 1}`, create_time: null, parent_username: null, reply_count: 0 }));
      const postBodies = [], commentReads = [];
      let failRead = true;
      await page.route('**/api/v1/message-comments?*', route => {
        const url = new URL(route.request().url()); commentReads.push(url.pathname + url.search);
        if (url.searchParams.get('page') === '2' && failRead) { failRead = false; return failed(route); }
        return route.fulfill({ json: pageOf(comments, url) });
      });
      await page.route('**/api/v1/message-comments', route => {
        postBodies.push(route.request().postDataJSON());
        if (postBodies.length === 1) return route.abort('failed');
        const saved = { ...comments[0], id: 99, comment_content: postBodies.at(-1).content };
        comments.unshift(saved); return route.fulfill({ json: saved });
      });
      await page.goto(`${base}message?lang=en`);
      await expect(page.locator('.public-comment')).toHaveCount(10);
      const input = page.getByRole('textbox', { name: 'Write your thoughts', exact: true });
      await input.fill('Unknown write survives read recovery');
      await page.getByRole('button', { name: 'Post anonymous comment', exact: true }).click();
      await expect(page.getByRole('alert')).toContainText('not yet confirmed');
      await page.getByRole('button', { name: 'Next page', exact: true }).click();
      await page.getByRole('button', { name: 'Try again', exact: true }).click();
      await expect(page.locator('.public-comment')).toHaveCount(1);
      assert.equal(commentReads[1], commentReads[2], 'read recovery retries the selected page');
      assert.equal(postBodies.length, 1, 'GET recovery cannot repeat an unknown POST');
      await expect(page.getByRole('alert')).toContainText('not yet confirmed');
      await expect(input).toHaveValue('Unknown write survives read recovery'); await expect(input).toBeDisabled();
      await page.getByRole('button', { name: 'Confirm original submission', exact: true }).click();
      await expect(page.getByRole('status').filter({ hasText: 'Your comment has been posted.' })).toBeVisible();
      assert.equal(postBodies.length, 2); assert.deepEqual(postBodies[1], postBodies[0]);
      assert.ok(wasm.length >= 1); assert.deepEqual(errors, []);
      console.log(`${engine.name()}: current WASM admin page retry, DELETE serialization/clamping/cancellation, upload cancellation and independent unknown-write recovery passed`);
      await page.close();
    } finally { await browser.close(); }
  }
} finally { await server.close(); }
