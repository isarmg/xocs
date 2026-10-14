import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { deflateSync } from 'node:zlib';
import { chromium, firefox, expect } from '@playwright/test';

// This script intentionally has no mock, preview server or production fallback.
// scripts/smoke.sh owns this exact listener and its disposable database/media.
const base = process.env.XOCS_SMOKE_URL;
if (typeof base !== 'string' || !/^http:\/\/127\.0\.0\.1:[1-9][0-9]{0,4}$/.test(base) || Number(new URL(base).port) > 65535) {
  throw new Error('management-native requires an explicit loopback XOCS_SMOKE_URL and the smoke fixture');
}

function png(marker) {
  function chunk(type, data) {
    const name = Buffer.from(type), bytes = Buffer.concat([name, data]);
    let crc = 0xffffffff;
    for (const byte of bytes) {
      crc ^= byte;
      for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
    }
    const size = Buffer.alloc(4), checksum = Buffer.alloc(4);
    size.writeUInt32BE(data.length); checksum.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
    return Buffer.concat([size, bytes, checksum]);
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(1, 0); header.writeUInt32BE(1, 4); header[8] = 8; header[9] = 6;
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', header),
    chunk('tEXt', Buffer.from(`Comment\0${marker}`)),
    chunk('IDAT', deflateSync(Buffer.from([0, 100, 150, 200, 255]))), chunk('IEND', Buffer.alloc(0)),
  ]);
}

for (const engine of [chromium, firefox]) {
  const browser = await engine.launch({ headless: true });
  const context = await browser.newContext({ locale: 'en-US', reducedMotion: 'reduce', viewport: { width: 1280, height: 900 } });
  const publicContext = await browser.newContext({ locale: 'en-US', reducedMotion: 'reduce' });
  const page = await context.newPage(), publicPage = await publicContext.newPage();
  const errors = [], receipts = [], cleanupErrors = [];
  page.on('pageerror', error => errors.push(error.message));
  publicPage.on('pageerror', error => errors.push(error.message));
  page.setDefaultTimeout(30_000); publicPage.setDefaultTimeout(30_000);
  const token = `N-${engine.name()}-${Date.now().toString(36)}`;
  const file = { name: `${token}.png`, mimeType: 'image/png', buffer: png(randomUUID()) };
  const own = { note: null, media: null, category: null, label: null, article: null, news: null };
  let csrf, originalSite, originalLetter, phase = 'fixture guard';

  async function read(path) {
    const response = await context.request.get(`${base}${path}`, { maxRedirects: 0 });
    assert.equal(response.status(), 200, `GET ${path}`);
    return response.json();
  }
  async function cleanup(method, path, body) {
    const response = await context.request.fetch(`${base}${path}`, {
      method, data: body, maxRedirects: 0,
      headers: { Origin: base, 'Sec-Fetch-Site': 'same-origin', 'X-CSRF-Token': csrf },
    });
    if (![200, 404].includes(response.status())) throw new Error(`cleanup ${method} ${path}: ${response.status()}`);
  }
  async function admin(hash, heading) {
    await page.goto(`${base}/admin?lang=en#${hash}`);
    assert.equal(new URL(page.url()).origin, base);
    await page.getByRole('heading', { name: heading, exact: true }).waitFor();
  }
  async function click(method, path, locator, expected = 200) {
    const waiting = page.waitForResponse(response => new URL(response.url()).pathname === path && response.request().method() === method);
    await locator.click();
    const response = await waiting;
    assert.equal(response.status(), expected, `${phase}: ${method} ${path}`);
    const value = await response.json();
    receipts.push({ method, path, status: response.status() });
    return value;
  }
  async function chooseFile(locator, payload) {
    const waiting = page.waitForEvent('filechooser');
    await locator.click();
    await (await waiting).setFiles(payload);
  }
  async function chooseOption(label, value) {
    const trigger = page.getByRole('combobox', { name: label, exact: true });
    await trigger.click();
    await page.getByRole('option', { name: value, exact: true }).click();
    await expect(trigger).toContainText(value);
  }
  const noteRow = value => page.getByRole('table', { name: 'Note list', exact: true }).getByRole('row').filter({ hasText: value });
  async function publicNotes(value, visible) {
    const waiting = publicPage.waitForResponse(response => new URL(response.url()).pathname === '/api/v1/notes/page');
    await publicPage.goto(`${base}/jotting?lang=en`);
    assert.equal((await waiting).status(), 200);
    await publicPage.getByText('Smoke author note', { exact: true }).waitFor();
    const entry = publicPage.locator('.wall-card').filter({ hasText: value });
    if (visible) await expect(entry).toBeVisible(); else await expect(entry).toHaveCount(0);
  }

  let failure;
  try {
    assert.equal((await read('/api/v1/health')).application, 'xocs');
    // Refuse an unrelated listener even on a loopback port. Never edit the
    // existing fixture articles 1–3, news 1 or note 2.
    assert.equal((await read('/api/v1/articles/1')).article_title, 'Smoke');
    assert.ok((await read('/api/v1/notes/page?size=50')).items.some(item => item.id === 2 && item.content === 'Smoke author note'));
    assert.equal((await read('/api/v1/letter')).recipient, 'Reader');
    await page.goto(`${base}/admin?lang=en`);
    await page.locator('input[name="username"]').fill('admin');
    await page.locator('input[name="password"]').fill('TemporaryPassphrase123');
    const signedIn = page.waitForResponse(response => new URL(response.url()).pathname === '/api/v1/auth/login' && response.request().method() === 'POST');
    await page.locator('form button[type="submit"]').click();
    assert.equal((await signedIn).status(), 200);
    // Login navigates immediately, so Chromium can discard that page response
    // body. Read the real authenticated session with this context's cookies.
    csrf = (await read('/api/v1/auth/session')).csrf_token;
    assert.equal(typeof csrf, 'string');
    originalSite = await read('/api/v1/content/site'); originalLetter = await read('/api/v1/content/letter');
    assert.ok(!Object.hasOwn(originalSite, 'footer'));

    phase = 'Files native picker upload';
    await admin('resources', 'Files');
    await chooseFile(page.getByRole('button', { name: /^Choose file:/ }), file);
    await expect(page.locator('.xocs-file-picker-name')).toHaveText(file.name);
    own.media = await click('POST', '/api/v1/content/upload', page.getByRole('button', { name: 'Upload', exact: true }));
    assert.equal(own.media.path, `/media/${createHash('sha256').update(file.buffer).digest('hex')}.png`);
    await expect(page.getByRole('table', { name: 'File list' }).getByRole('row').filter({ hasText: file.name })).toBeVisible();

    phase = 'Notes create and edit';
    await admin('notes', 'Notes');
    const draft = `${token} private draft`, edited = `${token} edited note`;
    await page.getByRole('textbox', { name: 'Note content', exact: true }).fill(draft);
    const uploaded = page.waitForResponse(response => new URL(response.url()).pathname === '/api/v1/content/upload' && response.request().method() === 'POST');
    await chooseFile(page.getByRole('button', { name: 'Choose file: Note image', exact: true }), file);
    const attachment = await uploaded; assert.equal(attachment.status(), 200);
    assert.equal((await attachment.json()).path, own.media.path);
    await expect.poll(() => page.getByRole('img', { name: 'View note image', exact: true }).first().evaluate(image => image.complete && image.naturalWidth === 1)).toBe(true);
    await page.getByRole('checkbox', { name: 'Publicly visible', exact: true }).uncheck();
    own.note = await click('POST', '/api/v1/content/notes', page.getByRole('button', { name: 'Publish', exact: true }));
    assert.equal(own.note.content, draft); assert.equal(own.note.is_public, 0); assert.equal(own.note.image_path, own.media.path);
    await expect(noteRow(draft)).toBeVisible(); await publicNotes(draft, false);
    await noteRow(draft).getByRole('button', { name: 'Edit', exact: true }).click();
    await page.getByRole('textbox', { name: 'Note content', exact: true }).fill(edited);
    const notePath = `/api/v1/content/notes/${own.note.id}`;
    own.note = await click('PUT', notePath, page.getByRole('button', { name: 'Save changes', exact: true }));
    assert.equal(own.note.content, edited); assert.equal(own.note.is_public, 0);
    await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toHaveCount(0);
    await expect(noteRow(edited)).toBeVisible();
    own.note = await click('PUT', notePath, noteRow(edited).getByRole('button', { name: 'Publish', exact: true }));
    assert.equal(own.note.is_public, 1); await publicNotes(edited, true);
    own.note = await click('PUT', notePath, noteRow(edited).getByRole('button', { name: 'Hide', exact: true }));
    assert.equal(own.note.is_public, 0); await publicNotes(edited, false);
    own.note = await click('PUT', notePath, noteRow(edited).getByRole('button', { name: 'Publish', exact: true }));
    assert.equal(own.note.is_public, 1); await publicNotes(edited, true);

    phase = 'Files referenced 409 and real removal';
    await admin('resources', 'Files');
    const fileRow = () => page.getByRole('table', { name: 'File list', exact: true }).getByRole('row').filter({ hasText: file.name });
    page.once('dialog', dialog => dialog.accept());
    await click('DELETE', `/api/v1/content/resources/${own.media.id}`, fileRow().getByRole('button', { name: 'Delete', exact: true }), 409);
    await expect(fileRow()).toBeVisible();
    assert.equal((await context.request.get(`${base}${own.media.path}`)).status(), 200);
    await admin('notes', 'Notes');
    await noteRow(edited).getByRole('button', { name: 'Edit', exact: true }).click();
    await page.getByRole('button', { name: 'Remove image', exact: true }).click();
    own.note = await click('PUT', notePath, page.getByRole('button', { name: 'Save changes', exact: true }));
    assert.equal(own.note.image_path, null);
    await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toHaveCount(0);
    await expect(noteRow(edited).getByRole('img')).toHaveCount(0);
    await admin('resources', 'Files'); page.once('dialog', dialog => dialog.accept());
    await click('DELETE', `/api/v1/content/resources/${own.media.id}`, fileRow().getByRole('button', { name: 'Delete', exact: true }));
    await expect(fileRow()).toHaveCount(0);
    assert.equal((await context.request.get(`${base}${own.media.path}`)).status(), 404);

    phase = 'Site owner display name and retained settings';
    await admin('site', 'Site settings');
    const author = `${token} Author`;
    await page.getByLabel('Site owner display name', { exact: true }).fill(author);
    const savedSite = await click('PUT', '/api/v1/content/site', page.getByRole('button', { name: 'Save settings', exact: true }));
    assert.deepEqual(savedSite, { ...originalSite, author_name: author });
    await admin('notes', 'Notes'); await expect(noteRow(edited).getByRole('cell').first()).toHaveText(author);

    phase = 'Letter private draft and publication';
    await admin('letter', 'Letter');
    const recipient = `${token} Reader`, letterText = `${token} first line\nsecond line <plain text>`;
    await page.getByLabel('Recipient', { exact: true }).fill(recipient);
    await page.getByLabel('Sender', { exact: true }).fill(author);
    await page.getByRole('textbox', { name: 'Letter content', exact: true }).fill(letterText);
    await page.getByRole('checkbox', { name: 'Show publicly', exact: true }).uncheck();
    const privateLetter = await click('PUT', '/api/v1/content/letter', page.getByRole('button', { name: 'Save letter', exact: true }));
    assert.equal(privateLetter.is_public, false);
    await publicPage.goto(`${base}/letter?lang=en`);
    await publicPage.getByRole('heading', { name: 'No letter is published yet', exact: true }).waitFor();
    await expect(publicPage.getByText(letterText, { exact: true })).toHaveCount(0);
    await page.getByRole('checkbox', { name: 'Show publicly', exact: true }).check();
    const publishedLetter = await click('PUT', '/api/v1/content/letter', page.getByRole('button', { name: 'Save letter', exact: true }));
    assert.equal(publishedLetter.is_public, true);
    await publicPage.goto(`${base}/letter?lang=en`);
    await publicPage.getByRole('button', { name: 'Open letter', exact: true }).click();
    await expect(publicPage.getByRole('heading', { name: `To ${recipient}`, exact: true })).toBeVisible();
    await expect(publicPage.locator('.letter-content')).toHaveText(letterText);
    await expect(publicPage.locator('.letter-paper strong')).toHaveText(author);

    phase = 'Own category, tag and article creation';
    await admin('categories', 'Categories');
    await page.getByLabel('Category name', { exact: true }).fill(token);
    own.category = await click('POST', '/api/v1/content/categories', page.getByRole('button', { name: 'Create category', exact: true }));
    await admin('labels', 'Tags'); await chooseOption('Category', token);
    await page.getByLabel('Tag name', { exact: true }).fill(token);
    own.label = await click('POST', '/api/v1/content/labels', page.getByRole('button', { name: 'Add tag', exact: true }));
    await admin('new', 'Add article');
    await page.getByLabel('Title', { exact: true }).fill(`${token} article`);
    await page.getByLabel('Markdown content', { exact: true }).fill(`${token} independent article body`);
    await chooseOption('Category', token); await chooseOption('Tags', token);
    own.article = await click('POST', '/api/v1/content/articles', page.getByRole('button', { name: 'Save article', exact: true }));
    assert.equal(own.article.username, author);
    await page.getByRole('heading', { name: 'Articles', exact: true }).waitFor();

    phase = 'Own article news create and UTC edit';
    await admin(`edit/${own.article.id}`, 'Edit article');
    await page.getByRole('textbox', { name: 'Update content', exact: true }).fill(`${token} first update`);
    await page.getByLabel('Update time (UTC)', { exact: true }).fill('2026-10-12T12:34:56');
    const newsPath = `/api/v1/content/articles/${own.article.id}/news`;
    own.news = await click('POST', newsPath, page.getByRole('button', { name: 'Publish update', exact: true }));
    assert.equal(own.news.create_time, '2026-10-12 12:34:56');
    const newsRow = value => page.getByRole('table', { name: 'Article updates', exact: true }).getByRole('row').filter({ hasText: value });
    await expect(newsRow(own.news.content)).toContainText('2026-10-12 12:34:56 UTC');
    await newsRow(own.news.content).getByRole('button', { name: 'Edit', exact: true }).click();
    await page.getByRole('textbox', { name: 'Update content', exact: true }).fill(`${token} edited update`);
    await page.getByLabel('Update time (UTC)', { exact: true }).fill('2026-10-13T01:02:03');
    own.news = await click('PUT', `${newsPath}/${own.news.id}`, page.getByRole('button', { name: 'Save changes', exact: true }));
    assert.equal(own.news.create_time, '2026-10-13 01:02:03');
    await expect(newsRow(own.news.content)).toContainText('2026-10-13 01:02:03 UTC');
    await publicPage.goto(`${base}/article/${own.article.id}?lang=en`);
    await expect(publicPage.getByText(own.news.content, { exact: true })).toBeVisible();
    await expect(publicPage.locator('.article-news')).toContainText('2026-10-13 01:02:03 UTC');

    phase = 'Own article password hint editor';
    await page.getByRole('checkbox', { name: /^Public access/ }).uncheck();
    await page.getByLabel('Article access password', { exact: true }).fill('NativeFixturePassphrase123');
    const hint = `${token} password hint`;
    await page.getByRole('textbox', { name: /^Password hint\b/ }).fill(hint);
    own.article = await click('PUT', `/api/v1/content/articles/${own.article.id}`, page.getByRole('button', { name: 'Save article', exact: true }));
    assert.equal(own.article.tips, hint); assert.equal(own.article.password_required, 1);
    await page.getByRole('heading', { name: 'Articles', exact: true }).waitFor();
    await publicPage.goto(`${base}/article/${own.article.id}?lang=en`);
    await expect(publicPage.getByText(hint, { exact: true })).toBeVisible();
    assert.deepEqual(errors, [], 'Uncaught page errors');
  } catch (error) {
    failure = new Error(`${engine.name()} management-native (${phase}): ${error.message}`, { cause: error });
  } finally {
    if (csrf) {
      const jobs = [];
      if (own.news && own.article) jobs.push(['DELETE', `/api/v1/content/articles/${own.article.id}/news/${own.news.id}`, {}]);
      if (own.article) jobs.push(['DELETE', `/api/v1/content/articles/${own.article.id}`, {}]);
      if (own.note) jobs.push(['DELETE', `/api/v1/content/notes/${own.note.id}`, {}]);
      if (own.media) jobs.push(['DELETE', `/api/v1/content/resources/${own.media.id}`, { path: own.media.path }]);
      if (own.label) jobs.push(['DELETE', `/api/v1/content/labels/${own.label.id}`, {}]);
      if (own.category) jobs.push(['DELETE', `/api/v1/content/categories/${own.category.id}`, {}]);
      if (originalSite) jobs.push(['PUT', '/api/v1/content/site', originalSite]);
      if (originalLetter) jobs.push(['PUT', '/api/v1/content/letter', originalLetter]);
      for (const [method, path, body] of jobs) {
        try { await cleanup(method, path, body); } catch (error) { cleanupErrors.push(error.message); }
      }
      if (!cleanupErrors.length && originalSite && originalLetter) {
        try {
          assert.deepEqual(await read('/api/v1/content/site'), originalSite);
          assert.deepEqual(await read('/api/v1/content/letter'), originalLetter);
          assert.equal((await read('/api/v1/articles/1')).article_title, 'Smoke');
          assert.ok((await read('/api/v1/notes/page?size=50')).items.some(item => item.id === 2 && item.content === 'Smoke author note'));
        } catch (error) { cleanupErrors.push(error.message); }
      }
    }
    await publicContext.close(); await context.close(); await browser.close();
  }
  if (failure || cleanupErrors.length || errors.length) {
    throw new AggregateError([...(failure ? [failure] : []), ...cleanupErrors.map(message => new Error(message)), ...errors.map(message => new Error(message))], `${engine.name()} native management failed`);
  }
  console.log(JSON.stringify({ engine: engine.name(), status: 'passed', real_http_ui_receipts: receipts, own_ids: Object.fromEntries(Object.entries(own).map(([kind, value]) => [kind, value?.id ?? null])), cleanup: 'own records removed; original site and Reader letter restored; fixture articles and note retained', pageerrors: errors }));
}
