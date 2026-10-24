import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { adminSite, article, letter, mockApi, pageOf } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = server.resolvedUrls.local[0];
const settle = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));

async function settingsValidation(page) {
  const siteWrites = [], letterWrites = [];
  await page.route('**/api/v1/content/site', route => {
    if (route.request().method() === 'PUT') {
      siteWrites.push(route.request().postDataJSON());
      return route.fulfill({ json: siteWrites.at(-1) });
    }
    return route.fulfill({ json: adminSite });
  });
  await page.route('**/api/v1/content/letter', route => {
    if (route.request().method() === 'PUT') {
      letterWrites.push(route.request().postDataJSON());
      return route.fulfill({ json: letterWrites.at(-1) });
    }
    return route.fulfill({ json: { ...letter, is_public: true } });
  });
  await page.goto(`${base}admin?lang=en#site`);
  const author = page.getByLabel('Site owner display name', { exact: true });
  for (const value of ['x'.repeat(101), '   ']) {
    await author.fill(value);
    await page.getByRole('button', { name: 'Save settings', exact: true }).click();
    await expect(page.getByRole('alert')).toContainText(value.trim().length
      ? 'Use no more than 100 characters' : 'Site owner display name must contain 1–100 characters');
    await expect(author).toBeFocused();
    await expect(author).toHaveValue(value);
    assert.equal(siteWrites.length, 0);
  }
  await author.fill('😀'.repeat(100));
  await expect(page.getByRole('alert')).toHaveCount(0);
  await page.getByRole('button', { name: 'Save settings', exact: true }).click();
  await expect(page.getByText('Site settings saved', { exact: true })).toBeVisible();
  assert.equal(siteWrites.length, 1);
  assert.equal(siteWrites[0].author_name, '😀'.repeat(100));

  await page.goto(`${base}admin?lang=en#letter`);
  for (const [label, value, maximum] of [
    ['Recipient', 'x'.repeat(101), 100], ['Sender', 'x'.repeat(101), 100],
    ['Letter content', 'x'.repeat(16_385), 16_384], ['Letter content', '   ', 16_384],
  ]) {
    const input = page.getByLabel(label, { exact: true });
    const previous = await input.inputValue();
    await input.fill(value);
    await page.getByRole('button', { name: 'Save letter', exact: true }).click();
    await expect(page.getByRole('alert')).toContainText(value.trim().length
      ? `Use no more than ${maximum} characters` : `${label} must contain 1–${maximum} characters`);
    await expect(input).toBeFocused();
    assert.equal(letterWrites.length, 0);
    await input.fill(previous);
  }
  await page.getByLabel('Recipient', { exact: true }).fill('😀'.repeat(100));
  await page.getByRole('button', { name: 'Save letter', exact: true }).click();
  await expect(page.getByText('Letter saved', { exact: true })).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);
  assert.equal(letterWrites[0].recipient, '😀'.repeat(100));
}

async function priorityInput(page) {
  const writes = [];
  await page.route('**/api/v1/content/categories', route => {
    const input = route.request().postDataJSON();
    writes.push(input);
    return route.fulfill({ json: { id: writes.length + 1, sort_name: input.name,
      sort_description: input.description, priority: input.priority, article_count: 0 } });
  });
  await page.goto(`${base}admin?lang=en#categories`);
  const name = page.getByLabel('Category name', { exact: true });
  const priority = page.getByLabel('Priority', { exact: true });
  await name.fill('Signed priority');
  await priority.press('ControlOrMeta+A');
  await priority.pressSequentially('-20');
  await expect(priority).toHaveValue('-20');
  await page.getByRole('button', { name: 'Create category', exact: true }).click();
  await expect.poll(() => writes.length).toBe(1);
  assert.equal(writes[0].priority, -20);
  await expect(priority).toHaveValue('0');
  await name.fill('Preserved priority draft');
  for (const value of ['', '-', '1e3', '1.5', '9007199254740992', '999999999999999999999999']) {
    await priority.fill(value);
    await expect(priority).toHaveValue(value);
    await page.getByRole('button', { name: 'Create category', exact: true }).click();
    await expect(page.getByRole('alert')).toContainText('Priority must be a whole number');
    await expect(priority).toBeFocused();
    assert.equal(writes.length, 1);
    await expect(name).toHaveValue('Preserved priority draft');
  }
  await priority.fill('');
  await priority.pressSequentially('-9007199254740991');
  await page.getByRole('button', { name: 'Create category', exact: true }).click();
  await expect.poll(() => writes.length).toBe(2);
  assert.equal(writes[1].priority, -9_007_199_254_740_991);
  await expect(page.getByRole('alert')).toHaveCount(0);
}

async function linkMutations(page) {
  let links = [1, 2, 3].map(id => ({ id, title: `Resource ${id}`, classify: null, cover: null,
    url: null, introduction: null, link_type: id === 3 ? 'friendUrl' : 'favorites', status: id === 3 ? 0 : 1, create_time: null }));
  const writes = [];
  let pending;
  await page.route('**/api/v1/content/links?*', route => route.fulfill({ json: pageOf(links, new URL(route.request().url())) }));
  await page.route(/\/api\/v1\/content\/links\/\d+(?:\/status)?$/, route => {
    const request = route.request(), path = new URL(request.url()).pathname;
    writes.push({ path, method: request.method(), body: request.postData() ? request.postDataJSON() : null });
    if (request.method() === 'DELETE' || path.endsWith('/status')) { pending = route; return; }
    const id = Number(path.split('/').at(-1)), input = request.postDataJSON();
    const value = { ...links.find(link => link.id === id), ...input, status: input.status ? 1 : 0 };
    links = links.map(link => link.id === id ? value : link);
    return route.fulfill({ json: value });
  });
  await page.goto(`${base}admin?lang=en#links`);
  const table = page.getByRole('table', { name: 'Content resource list', exact: true });
  const row = id => table.getByRole('row').filter({ hasText: `Resource ${id}` });
  const title = page.getByLabel('Title', { exact: true });
  await row(1).getByRole('button', { name: 'Edit', exact: true }).click();
  await row(1).getByRole('button', { name: 'Delete', exact: true }).click();
  await expect.poll(() => Boolean(pending)).toBe(true);
  await expect(title).toBeDisabled();
  await expect(row(1).getByRole('button', { name: 'Edit', exact: true })).toBeDisabled();
  await page.locator('form.editor-form').evaluate(form => form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })));
  await settle(page); assert.equal(writes.length, 1);
  await row(2).getByRole('button', { name: 'Edit', exact: true }).click();
  await title.fill('Newer resource draft');
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled();
  links = links.filter(link => link.id !== 1);
  await pending.fulfill({ json: { deleted: true } }); pending = null;
  await expect(row(1)).toHaveCount(0);
  await expect(title).toHaveValue('Newer resource draft');
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await expect(page.getByText('Resource saved', { exact: true })).toBeVisible();
  assert.equal(writes[1].path, '/api/v1/content/links/2');
  assert.equal(writes[1].body.title, 'Newer resource draft');

  const second = table.getByRole('row').filter({ hasText: 'Newer resource draft' });
  await second.getByRole('button', { name: 'Edit', exact: true }).click();
  await second.getByRole('button', { name: 'Delete', exact: true }).click();
  await expect.poll(() => Boolean(pending)).toBe(true);
  links = links.filter(link => link.id !== 2);
  await pending.fulfill({ json: { deleted: true } }); pending = null;
  await expect(page.getByRole('heading', { name: 'Add resource', exact: true })).toBeVisible();
  await expect(title).toHaveValue('');

  await row(3).getByRole('button', { name: 'Edit', exact: true }).click();
  await row(3).getByRole('button', { name: 'Approve', exact: true }).click();
  await expect.poll(() => Boolean(pending)).toBe(true);
  await expect(title).toBeDisabled();
  await page.locator('form.editor-form').evaluate(form => form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })));
  await settle(page); assert.equal(writes.length, 4);
  links[0].status = 1;
  await pending.fulfill({ json: links[0] }); pending = null;
  await expect(page.getByRole('checkbox', { name: 'Show publicly', exact: true })).toBeChecked();
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await expect.poll(() => writes.length).toBe(5);
  assert.equal(writes[4].body.status, true, 'saving the edited resource cannot undo its completed approval');
}

async function taxonomyUnicode(page) {
  for (const tags of [false, true]) {
    const endpoint = `/api/v1/content/${tags ? 'labels' : 'categories'}`;
    const writes = [], maximum = tags ? 32 : 64;
    await page.route(url => url.pathname === endpoint, route => {
      if (route.request().method() === 'GET') return route.fulfill({ json: [] });
      const input = route.request().postDataJSON(); writes.push(input);
      return route.fulfill({ json: tags
        ? { id: writes.length + 10, sort_id: input.sort_id, label_name: input.name, label_description: input.description }
        : { id: writes.length + 10, sort_name: input.name, sort_description: input.description, priority: input.priority, article_count: 0 } });
    });
    await page.goto(`${base}admin?lang=en#${tags ? 'labels' : 'categories'}`);
    const name = page.getByLabel(tags ? 'Tag name' : 'Category name', { exact: true });
    if (tags) {
      await page.getByRole('combobox', { name: 'Category', exact: true }).click();
      await page.getByRole('option', { name: '中文分类', exact: true }).click();
    }
    const invalid = '😀'.repeat(maximum / 2) + 'x'.repeat(maximum / 2 + 1);
    await name.fill(invalid);
    await expect(name).toHaveValue(invalid);
    await expect(name).toHaveJSProperty('validationMessage', `Use no more than ${maximum} characters.`);
    await name.blur();
    await expect(name).toHaveJSProperty('validationMessage', `Use no more than ${maximum} characters.`);
    await page.getByRole('button', { name: tags ? 'Add tag' : 'Create category', exact: true }).click();
    await expect(name).toBeFocused();
    await expect(page.getByRole('alert')).toContainText(`Use no more than ${maximum} characters`);
    assert.equal(writes.length, 0);
    const valid = '😀'.repeat(maximum);
    await name.fill(valid);
    await expect(name).toHaveValue(valid);
    await expect(name).toHaveJSProperty('validationMessage', '');
    await expect(page.getByRole('alert')).toHaveCount(0);
    await page.getByRole('button', { name: tags ? 'Add tag' : 'Create category', exact: true }).click();
    await expect.poll(() => writes.length).toBe(1);
    assert.equal(writes[0].name, valid);
    await expect(name).toHaveValue('');
    assert.equal(await name.evaluate(node => node.validity.customError), false, 'reset clears scalar validity');
  }
}

async function noteRecovery(page) {
  const notes = [], requests = [], receipts = new Map();
  await page.route(/\/api\/v1\/content\/notes(?:\/\d+)?(?:\?.*)?$/, route => {
    const request = route.request(), url = new URL(request.url());
    if (request.method() === 'GET') return route.fulfill({ json: pageOf(notes, url) });
    const input = request.postDataJSON(), key = request.headers()['idempotency-key'];
    requests.push({ method: request.method(), path: url.pathname, input, key });
    if (request.method() === 'PUT') {
      assert.equal(url.pathname, '/api/v1/content/notes/7');
      notes[0] = { ...notes[0], ...input, content: input.content.trim(), is_public: input.is_public ? 1 : 0 };
      return route.fulfill({ json: notes[0] });
    }
    assert.ok(key, 'Every note creation has a receipt key');
    if (!receipts.has(key)) {
      notes.push({ id: 7, user_id: 1, username: 'Owner', image_path: null, ...input,
        content: input.content.trim(), is_public: input.is_public ? 1 : 0, create_time: '2026-10-08' });
      receipts.set(key, input);
      return route.abort('failed'); // The write committed; its response was lost.
    }
    assert.deepEqual(input, receipts.get(key));
    notes[0].content = 'Original note edited in another tab';
    return route.fulfill({ json: notes[0] });
  });
  await page.goto(`${base}admin?lang=en#notes`);
  const input = page.getByRole('textbox', { name: 'Note content', exact: true });
  await input.fill(' ');
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Note content must contain');
  assert.equal(requests.length, 0);
  await input.fill('Original note');
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  const confirm = page.getByRole('button', { name: 'Confirm original note', exact: true });
  await expect(confirm).toBeEnabled();
  await input.fill(''); // Native required validation must not block recovery.
  await confirm.click();
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeEnabled();
  await expect(input).toHaveValue('');
  assert.equal(notes.length, 1);
  assert.deepEqual(requests[1], requests[0]);
  await input.fill('Newer note draft');
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await expect(input).toHaveValue('');
  await expect(page.getByRole('alert')).toHaveCount(0);
  assert.equal(requests[2].method, 'PUT');
  assert.equal(notes[0].content, 'Newer note draft');
}

async function updateRecovery(page) {
  const updates = [], requests = [], receipts = new Map();
  await page.route(/\/api\/v1\/content\/articles\/1\/news(?:\/\d+)?(?:\?.*)?$/, route => {
    const request = route.request(), url = new URL(request.url());
    if (request.method() === 'GET') return route.fulfill({ json: pageOf(updates, url) });
    const input = request.postDataJSON(), key = request.headers()['idempotency-key'];
    requests.push({ method: request.method(), path: url.pathname, input, key });
    if (request.method() === 'PUT') {
      assert.equal(url.pathname, '/api/v1/content/articles/1/news/9');
      updates[0] = { ...updates[0], content: input.content.trim() };
      return route.fulfill({ json: updates[0] });
    }
    assert.ok(key);
    if (!receipts.has(key)) {
      receipts.set(key, input);
      updates.push({ id: 9, content: input.content.trim(), create_time: '2026-10-08' });
      return route.abort('failed');
    }
    assert.deepEqual(input, receipts.get(key));
    updates[0].content = 'Original update edited in another tab';
    return route.fulfill({ json: updates[0] });
  });
  await page.goto(`${base}admin?lang=en#edit/1`);
  const input = page.getByRole('textbox', { name: 'Update content', exact: true });
  await input.fill('One update');
  await page.getByRole('button', { name: 'Publish update', exact: true }).click();
  const confirm = page.getByRole('button', { name: 'Confirm original update', exact: true });
  await expect(confirm).toBeEnabled();
  await input.fill('One update ');
  await confirm.click();
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeEnabled();
  await expect(input).toHaveValue('One update ');
  assert.equal(updates.length, 1);
  assert.deepEqual(requests[1], requests[0]);
  await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await expect(input).toHaveValue('');
  assert.equal(requests[2].method, 'PUT');
  assert.equal(updates[0].content, 'One update');
}

async function articleRecovery(page) {
  const requests = [], receipts = new Map();
  const response = input => {
    const { password, ...draft } = input;
    return { ...article, ...draft, id: 77, view_status: input.view_status ? 1 : 0,
      recommend_status: input.recommend_status ? 1 : 0, comment_status: input.comment_status ? 1 : 0,
      password_required: password ? 1 : 0 };
  };
  await page.route('**/api/v1/content/labels', route => route.fulfill({ json: [
    { id: 1, sort_id: 1, label_name: 'Tag', label_description: null },
  ] }));
  await page.route(/\/api\/v1\/content\/articles(?:\/77)?$/, route => {
    const request = route.request();
    if (!['POST', 'PUT'].includes(request.method())) return route.fallback();
    const input = request.postDataJSON(), key = request.headers()['idempotency-key'];
    requests.push({ method: request.method(), path: new URL(request.url()).pathname, input, key });
    if (request.method() === 'PUT') return route.fulfill({ json: response(input) });
    assert.ok(key);
    if (!receipts.has(key)) { receipts.set(key, input); return route.abort('failed'); }
    assert.deepEqual(input, receipts.get(key));
    return route.fulfill({ json: { ...response(input), article_title: 'Original article edited elsewhere' } });
  });
  await page.goto(`${base}admin?lang=en#new`);
  const title = page.getByRole('textbox', { name: 'Title', exact: true });
  const content = page.getByRole('textbox', { name: 'Markdown content', exact: true });
  await title.fill('x'.repeat(121));
  await content.fill('Original body');
  await page.getByRole('combobox', { name: 'Category', exact: true }).click();
  await page.getByRole('option', { name: '中文分类', exact: true }).click();
  await page.getByRole('combobox', { name: 'Tags', exact: true }).click();
  await page.getByRole('option', { name: 'Tag', exact: true }).click();
  await page.getByRole('button', { name: 'Save article', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Use no more than 120 characters');
  assert.equal(requests.length, 0);
  await title.fill('😀'.repeat(100));
  await expect(title).toHaveValue('😀'.repeat(100));
  await page.getByRole('button', { name: 'Save article', exact: true }).click();
  const confirm = page.getByRole('button', { name: 'Confirm original article', exact: true });
  await expect(confirm).toBeEnabled();
  await title.fill('Newer article title');
  await content.fill('');
  await confirm.click();
  await expect(page.getByRole('button', { name: 'Save article', exact: true })).toBeEnabled();
  await expect(title).toHaveValue('Newer article title');
  await expect(content).toHaveValue('');
  assert.equal(receipts.size, 1);
  assert.deepEqual(requests[1], requests[0]);
  assert.equal([...requests[0].input.article_title].length, 100);
  await content.fill('Newer body');
  await page.getByRole('button', { name: 'Save article', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Articles', exact: true })).toBeVisible();
  assert.equal(requests[2].method, 'PUT');
  assert.equal(requests[2].path, '/api/v1/content/articles/77');
  assert.equal(requests[2].input.article_title, 'Newer article title');
}

async function commentInteractions(page) {
  const record = (id, body, replies = 0) => ({ id, user_id: null, username: null, avatar: null,
    comment_content: body, create_time: '2026-10-08', parent_username: null, reply_count: replies });
  const comments = [record(10, 'Root comment', 12)];
  const replies = Array.from({ length: 12 }, (_, index) => record(11 + index, `Reply ${index + 1}`));
  const posts = [];
  let replyReads = 0;
  let pending;
  await page.route(/\/api\/v1\/comments(?:\/10\/replies)?(?:\?.*)?$/, route => {
    const request = route.request(), url = new URL(request.url());
    if (request.method() === 'GET') {
      if (url.pathname.endsWith('/replies')) replyReads++;
      if (url.pathname.endsWith('/replies') && url.searchParams.get('page') === '2') { pending = route; return; }
      return route.fulfill({ json: pageOf(url.pathname.endsWith('/replies') ? replies : comments, url) });
    }
    const input = request.postDataJSON(); posts.push(input);
    const saved = record(99, input.content); comments.unshift(saved);
    return route.fulfill({ json: saved });
  });
  await page.goto(`${base}article/1?lang=en`);
  const more = page.getByRole('button', { name: 'Show 7 more replies', exact: true });
  await more.focus(); await page.keyboard.press('Enter');
  await expect.poll(() => Boolean(pending)).toBe(true);
  await expect(more).toBeDisabled(); await expect(more).toBeFocused();
  await page.keyboard.press('Enter'); await settle(page);
  assert.equal(replyReads, 2, 'an aria-disabled expansion rejects repeated keyboard activation');
  await pending.fulfill({ json: pageOf(replies, new URL(pending.request().url())) }); pending = null;
  const remaining = page.getByRole('button', { name: 'Show 2 more replies', exact: true });
  await expect(remaining).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button', { name: 'All replies shown', exact: true })).toBeFocused();
  const input = page.getByRole('textbox', { name: 'Write your thoughts', exact: true });
  await input.fill('x'.repeat(1025));
  await page.getByRole('button', { name: 'Post anonymous comment', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Comment content must contain 1–1024 characters');
  assert.equal(posts.length, 0);
  await input.fill('😀'.repeat(600));
  await expect(input).toHaveValue('😀'.repeat(600));
  await page.getByRole('button', { name: 'Post anonymous comment', exact: true }).click();
  await expect(input).toHaveValue('');
  assert.equal(posts[0].content, '😀'.repeat(600));
}

try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 } });
      const errors = []; page.on('pageerror', error => errors.push(error.message));
      page.on('dialog', dialog => dialog.accept());
      await mockApi(page, { authenticated: true });
      await settingsValidation(page);
      await priorityInput(page);
      await taxonomyUnicode(page);
      await linkMutations(page);
      await noteRecovery(page);
      await updateRecovery(page);
      await articleRecovery(page);
      await commentInteractions(page);
      assert.deepEqual(errors, []);
      console.log(`${engine.name()}: focused validation, signed priority, resource mutation coordination, immutable create recovery, Unicode limits and reply keyboard focus passed`);
    } finally { await browser.close(); }
  }
} finally { await server.close(); }
