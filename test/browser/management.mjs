import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { article, mockApi, pageOf, site, session } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
const image = { name: 'note.png', mimeType: 'image/png', buffer: Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=', 'base64') };
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 } }), errors = [], mutations = [];
      page.on('pageerror', error => errors.push(error.message)); page.on('dialog', dialog => dialog.accept());
      await mockApi(page, { authenticated: true });
      let author = 'Original author', letter = { recipient: '', sender: '', content: '', is_public: false }, media = [];
      let notes = [{ id: 1, user_id: 1, username: author, content: 'Existing note', image_path: null, is_public: 1, create_time: '2026-10-12 04:05:06' }];
      let news = Array.from({ length: 101 }, (_, index) => ({ id: 101-index, content: `Update ${101-index}`, create_time: '2026-10-12 04:05:06' }));
      let adminArticle = { ...article, username: author, view_status: 0, password_required: 1, tips: 'Old hint' }, failNewsPage = 0, failLetterRead = false;
      await page.route('**/media/**', route => route.fulfill({ contentType: 'image/png', body: image.buffer }));
      await page.route('**/api/v1/**', async route => {
        const request = route.request(), url = new URL(request.url()), pathname = url.pathname, method = request.method();
        if (method !== 'GET') { assert.equal(request.headers()['x-csrf-token'], session.csrf_token); mutations.push({ path: pathname, method, body: method === 'POST' && pathname.endsWith('/upload') ? null : request.postDataJSON() }); }
        // Axum Json(null) sends application/json; Playwright omits it for json:null.
        const json = value => route.fulfill({ json: value, contentType: 'application/json' });
        if (pathname === '/api/v1/content/site') {
          if (method === 'PUT') { const input = request.postDataJSON(); assert.ok(!Object.hasOwn(input, 'footer')); author = input.author_name; }
          return json({ ...site, author_name: author });
        }
        if (pathname === '/api/v1/content/labels') return json([{ id: 1, sort_id: 1, label_name: 'Tag', label_description: null }]);
        if (pathname === '/api/v1/content/upload') {
          assert.equal(request.headers()['content-type'], image.mimeType); assert.deepEqual(request.postDataBuffer(), image.buffer);
          const resource = { id: 1, original_name: image.name, path: `/media/${'a'.repeat(64)}.png`, resource_type: 'image', size: image.buffer.length, status: 1, create_time: null };
          media = [resource]; return json({ id: resource.id, path: resource.path, mime_type: image.mimeType, size: resource.size });
        }
        if (pathname === '/api/v1/content/notes') return json(pageOf(notes.map(note => ({ ...note, username: author })), url));
        if (pathname === '/api/v1/content/notes/1') {
          assert.equal(method, 'PUT'); const input = request.postDataJSON(); assert.equal(typeof input.is_public, 'boolean');
          notes[0] = { ...notes[0], ...input, username: author, is_public: Number(input.is_public) }; return json(notes[0]);
        }
        if (pathname === '/api/v1/notes/page') return json({ ...pageOf(notes.filter(note => note.is_public).map(note => ({ ...note, username: author, like_count: 0 })), url) });
        if (pathname === '/api/v1/content/resources') return json(pageOf(media, url));
        if (pathname === '/api/v1/content/resources/1') {
          assert.equal(method, 'DELETE'); assert.deepEqual(request.postDataJSON(), { path: media[0].path });
          if (notes.some(note => note.image_path === media[0].path)) return route.fulfill({ status: 409, json: { code: 'content.in_use', message: 'SECRET internal data', retryable: false, request_id: 'private-id' } });
          media = []; return json({ deleted: true });
        }
        if (pathname === '/api/v1/content/articles/1') {
          if (method === 'PUT') { const input = request.postDataJSON(); adminArticle = { ...adminArticle, ...input, view_status: Number(input.view_status), recommend_status: Number(input.recommend_status), comment_status: Number(input.comment_status) }; }
          return json(adminArticle);
        }
        if (pathname === '/api/v1/content/articles/1/news') {
          const number = Number(url.searchParams.get('page') || 1), size = Number(url.searchParams.get('size') || 20);
          if (failNewsPage === number) { failNewsPage = 0; return route.fulfill({ status: 500, json: { code: 'unavailable', message: 'SECRET', retryable: true, request_id: 'private-id' } }); }
          return json({ items: news.slice((number-1)*size, number*size), total: news.length, page: number, size });
        }
        if (pathname === '/api/v1/content/articles/1/news/1') {
          assert.equal(method, 'PUT'); const input = request.postDataJSON(); assert.equal(input.create_time, '2026-10-12T06:07:08.000Z');
          const saved = { id: 1, content: input.content, create_time: input.create_time.slice(0,19).replace('T',' ') };
          news = [saved, ...news.filter(item => item.id !== 1)]; return json(saved);
        }
        if (pathname === '/api/v1/content/letter') { if (method === 'PUT') letter = request.postDataJSON(); return json(letter); }
        if (pathname === '/api/v1/letter') {
          if (failLetterRead) { failLetterRead = false; return route.fulfill({ status: 503, json: { code: 'unavailable', message: 'SECRET', retryable: true, request_id: 'private-id' } }); }
          return json(letter.is_public ? { recipient: letter.recipient, sender: letter.sender, content: letter.content } : null);
        }
        return route.fallback();
      });

      await page.goto(`${base}/admin?lang=en#site`);
      await page.getByLabel('Site owner display name', { exact: true }).fill('Named author');
      await page.getByRole('button', { name: 'Save settings', exact: true }).click(); await expect(page.getByText('Site settings saved', { exact: true })).toBeVisible(); assert.equal(author, 'Named author');
      assert.equal(mutations.some(item => item.path.startsWith('/api/v1/platform/administrators')), false);
      await page.goto(`${base}/admin?lang=en#notes`);
      let noteRow = page.getByRole('row').filter({ hasText: 'Existing note' }); await noteRow.getByRole('button', { name: 'Edit', exact: true }).click();
      await page.getByLabel('Note content', { exact: true }).fill('Edited note');
      const dialog = page.waitForEvent('filechooser'); await page.getByRole('button', { name: 'Choose file: Note image', exact: true }).click(); await (await dialog).setFiles(image);
      await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeEnabled();
      await page.getByRole('checkbox', { name: 'Publicly visible', exact: true }).uncheck();
      await page.getByRole('button', { name: 'Save changes', exact: true }).click();
      noteRow = page.getByRole('row').filter({ hasText: 'Edited note' }); await expect(noteRow).toContainText('Private');
      await page.goto(`${base}/jotting?lang=en`); await expect(page.getByText('Edited note', { exact: true })).toHaveCount(0);
      await page.goto(`${base}/admin?lang=en#notes`); noteRow = page.getByRole('row').filter({ hasText: 'Edited note' }); await noteRow.getByRole('button', { name: 'Publish', exact: true }).click(); await expect(noteRow).toContainText('Public');
      await page.goto(`${base}/jotting?lang=en`); await expect(page.getByText('Edited note', { exact: true })).toBeVisible();
      await page.goto(`${base}/admin?lang=en#resources`); const fileRow = page.getByRole('row').filter({ hasText: image.name });
      await fileRow.getByRole('button', { name: 'Delete', exact: true }).click(); await expect(page.getByRole('alert').first()).toContainText('referenced'); await expect(fileRow).toBeVisible();
      await expect(page.locator('body')).not.toContainText('SECRET');
      await page.goto(`${base}/admin?lang=en#notes`); noteRow = page.getByRole('row').filter({ hasText: 'Edited note' }); await noteRow.getByRole('button', { name: 'Edit', exact: true }).click();
      await page.getByRole('button', { name: 'Remove image', exact: true }).click(); await page.getByRole('button', { name: 'Save changes', exact: true }).click(); await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toHaveCount(0); assert.equal(notes[0].image_path, null);
      await page.goto(`${base}/admin?lang=en#resources`); await fileRow.getByRole('button', { name: 'Delete', exact: true }).click(); await expect(fileRow).toHaveCount(0); assert.deepEqual(media, []);

      await page.goto(`${base}/admin?lang=en#edit/1`); await page.getByRole('textbox', { name: /^Password hint\b/ }).fill('A useful hint');
      await page.getByRole('button', { name: 'Save article', exact: true }).click(); await expect(page.getByRole('region', { name: 'Articles', exact: true })).toBeVisible(); assert.equal(adminArticle.tips, 'A useful hint');
      await page.goto(`${base}/admin?lang=en#edit/1`); const pager = page.getByRole('navigation', { name: 'Update pages', exact: true });
      for (let number=2;number<=6;number++) { await pager.getByRole('button', { name: 'Next page', exact: true }).click(); await expect(pager).toContainText(`Page ${number}`); await expect(page.getByRole('table', { name: 'Article updates', exact: true }).getByText(`Update ${101-(number-1)*20}`, { exact: true })).toBeVisible(); }
      await page.getByRole('row').filter({ hasText: 'Update 1' }).getByRole('button', { name: 'Edit', exact: true }).click();
      const content = page.getByLabel('Update content', { exact: true }); await content.fill('Edited oldest update');
      await page.getByLabel('Update time (UTC)', { exact: true }).fill('2026-10-12T06:07:08');
      failNewsPage = 5; await pager.getByRole('button', { name: 'Previous page', exact: true }).click(); await expect(page.getByRole('alert').first()).toContainText('Unable to load article updates');
      await expect(content).toHaveValue('Edited oldest update'); await page.getByRole('button', { name: 'Try again', exact: true }).click(); await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeEnabled();
      await page.getByRole('button', { name: 'Save changes', exact: true }).click(); await expect(page.getByRole('table', { name: 'Article updates', exact: true })).toContainText('Edited oldest update');

      await page.goto(`${base}/admin?lang=en#letter`); await page.getByLabel('Recipient', { exact: true }).fill('Reading friend'); await page.getByLabel('Sender', { exact: true }).fill('Named author');
      const letterText = '<img src=x onerror="alert(1)">\nLiteral letter content'; await page.getByLabel('Letter content', { exact: true }).fill(letterText); await page.getByRole('checkbox', { name: 'Show publicly', exact: true }).check();
      await page.getByRole('button', { name: 'Save letter', exact: true }).click(); await expect(page.getByText('Letter saved', { exact: true })).toBeVisible(); assert.equal(letter.is_public, true);
      failLetterRead = true; await page.goto(`${base}/letter?lang=en`); await expect(page.getByRole('alert')).toContainText('The service is temporarily unavailable');
      await expect(page.getByRole('button', { name: 'Open letter', exact: true })).toHaveCount(0); await expect(page.locator('body')).not.toContainText('SECRET');
      await page.getByRole('button', { name: 'Try again', exact: true }).click(); await page.getByRole('button', { name: 'Open letter', exact: true }).click();
      await expect(page.locator('.letter-content')).toHaveText(letterText); await expect(page.locator('.letter-paper img, .letter-paper script')).toHaveCount(0); await expect(page.getByRole('button', { name: 'Close letter', exact: true })).toBeFocused();
      await page.getByRole('button', { name: 'Close letter', exact: true }).click(); await expect(page.getByRole('button', { name: 'Open letter', exact: true })).toBeFocused();
      await page.goto(`${base}/admin?lang=en#letter`); await page.getByRole('checkbox', { name: 'Show publicly', exact: true }).uncheck(); await page.getByRole('button', { name: 'Save letter', exact: true }).click(); await expect(page.getByText('Letter saved', { exact: true })).toBeVisible(); assert.equal(letter.is_public, false);
      await page.goto(`${base}/letter?lang=en`); await expect(page.getByRole('heading', { name: 'No letter is published yet', exact: true })).toBeVisible(); await expect(page.getByRole('button', { name: 'Open letter', exact: true })).toHaveCount(0);
      assert.deepEqual(errors, []); console.log(`${engine.name()}: site author, notes edit/image/visibility, bound file delete, article tips, 101 updates/edit/UTC/draft retry and managed escaped letter passed`);
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
