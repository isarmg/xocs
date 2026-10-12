import assert from 'node:assert/strict';
import { test } from 'node:test';
import { originalModule } from './fixtures/original-module.mjs';
import { HookHost, expand, walk, textContent } from './fixtures/hook-host.mjs';
import { adminSite } from './browser-fixtures.mjs';

const pages = await originalModule(new URL('../src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__MANAGEMENT_APP;',
  './AdminLayout': 'export const adminGroups=[];',
  './api': `export * from ${JSON.stringify(new URL('../src/api.ts', import.meta.url).href)};`,
  './MediaPreview': 'export const ImageLightbox=()=>null;export const safeImageUrl=value=>value;',
});
const note = { id: 1, user_id: 1, username: 'Writer', content: 'Original note', image_path: null, is_public: 1, create_time: '2026-10-12 04:05:06' };
const row = { id: 21, original_name: 'owned.png', path: `/media/${'a'.repeat(64)}.png`, resource_type: 'image', size: 5, status: 1, create_time: null };
const page = (items, total = items.length, number = 1) => ({ items, total, page: number, size: 20 });
const settle = async host => { await new Promise(resolve => setImmediate(resolve)); host.render(); };
const button = (host, label) => walk(expand(host.tree), node => node.type === 'button' && textContent(node).trim() === label)[0];
const fields = (host, name) => walk(host.tree, node => node.type?.name === name);
const submit = host => walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
function setup(context, component) {
  const previous = { app: globalThis.__MANAGEMENT_APP, window: globalThis.window };
  const requests = [], notifications = [];
  globalThis.window = { confirm: () => true };
  globalThis.__MANAGEMENT_APP = { notify: message => notifications.push(message), client: { request(url, validate, options = {}) {
    return new Promise((resolve, reject) => requests.push({ url, options, reject,
      resolve(value) { if (validate(value)) resolve(value); else reject(new Error('Invalid response contract')); } }));
  } } };
  const host = new HookHost(component); host.render();
  context.after(() => { host.unmount(); globalThis.__MANAGEMENT_APP = previous.app; globalThis.window = previous.window; });
  return { host, requests, notifications };
}

test('notes edit includes an acknowledged uploaded image and explicit visibility, and preserves the retry draft', async context => {
  const { host, requests } = setup(context, pages.NotesPage);
  requests[0].resolve(page([note])); await settle(host);
  button(host, 'Edit').props.onClick(); host.render();
  walk(host.tree, node => node.type === 'textarea')[0].props.onChange({ target: { value: 'Edited note' } }); host.render();
  const input = walk(expand(host.tree), node => node.type === 'input' && node.props.type === 'file')[0];
  const target = { files: [new File(['image'], 'owned.png', { type: 'image/png' })], value: 'owned.png' };
  input.props.onChange({ target }); host.render(); assert.equal(target.value, '');
  assert.equal(requests.at(-1).url, '/api/v1/content/upload');
  submit(host); assert.equal(requests.filter(request => request.options.method === 'PUT').length, 0);
  const image = `/media/${'a'.repeat(64)}.png`;
  requests.at(-1).resolve({ id: 2, path: image, mime_type: 'image/png', size: 5 }); await settle(host);
  walk(host.tree, node => node.type === 'input' && node.props.type === 'checkbox')[0].props.onChange({ target: { checked: false } }); host.render();
  submit(host); host.render(); const first = requests.at(-1);
  assert.equal(first.url, '/api/v1/content/notes/1'); assert.equal(first.options.method, 'PUT');
  assert.deepEqual(JSON.parse(first.options.body), { content: 'Edited note', image_path: image, is_public: false });
  first.reject(new Error('unavailable')); await settle(host);
  assert.equal(walk(host.tree, node => node.type === 'textarea')[0].props.value, 'Edited note');
  submit(host); assert.equal(requests.at(-1).options.body, first.options.body);
  requests.at(-1).resolve({ ...note, content: 'Edited note', image_path: image, is_public: 0 }); await settle(host);
  requests.at(-1).resolve(page([{ ...note, content: 'Edited note', image_path: image, is_public: 0 }])); await settle(host);
  const row = walk(expand(host.tree), node => node.type === 'tr' && textContent(node).includes('Edited note'))[0];
  walk(row, node => node.type === 'button' && textContent(node).trim() === 'Publish')[0].props.onClick(); host.render();
  assert.equal(requests.at(-1).options.method, 'PUT');
  assert.equal(JSON.parse(requests.at(-1).options.body).is_public, true);
  requests.at(-1).resolve({ ...note, content: 'Edited note', image_path: image, is_public: 0 }); await settle(host);
  assert.match(textContent(expand(host.tree)), /Unable to change note visibility/);
  assert.match(textContent(expand(host.tree)), /Private/);
});

test('file deletion rejects false acknowledgements and reloads the same page only after confirmed deletion', async context => {
  const { host, requests } = setup(context, pages.ResourcesPage);
  requests[0].resolve(page([row], 41)); await settle(host);
  button(host, 'Next page').props.onClick(); host.render();
  requests.at(-1).resolve(page([row], 41, 2)); await settle(host);
  button(host, 'Delete').props.onClick(); host.render();
  assert.equal(requests.at(-1).url, '/api/v1/content/resources/21');
  assert.equal(requests.at(-1).options.method, 'DELETE');
  assert.deepEqual(JSON.parse(requests.at(-1).options.body), { path: row.path });
  requests.at(-1).resolve({ deleted: false }); await settle(host);
  assert.match(textContent(expand(host.tree)), /owned.png/);
  assert.match(textContent(expand(host.tree)), /Unable to delete file/);
  const reads = requests.filter(request => !request.options.method).length;
  button(host, 'Delete').props.onClick(); host.render();
  requests.at(-1).reject({ status: 409 }); await settle(host);
  assert.match(textContent(expand(host.tree)), /referenced/);
  assert.equal(requests.filter(request => !request.options.method).length, reads);
  button(host, 'Delete').props.onClick(); host.render();
  requests.at(-1).resolve({ deleted: true }); await settle(host);
  assert.match(requests.at(-1).url, /page=2&size=20$/);
  assert.doesNotMatch(textContent(expand(host.tree)), /owned.png/);
});

test('site author display name uses only the site contract and never reintroduces footer or administrator credentials', async context => {
  const { host, requests } = setup(context, pages.SitePage);
  requests[0].resolve(adminSite); await settle(host);
  const field = fields(host, 'FormField').find(field => field.props.label === 'Site owner display name');
  field.props.children.props.onChange({ target: { value: 'Named writer' } }); host.render(); submit(host);
  const request = requests.at(-1), input = JSON.parse(request.options.body);
  assert.equal(request.url, '/api/v1/content/site'); assert.equal(request.options.method, 'PUT');
  assert.equal(input.author_name, 'Named writer');
  for (const removed of ['footer', 'username', 'current_password', 'new_password']) assert.equal(Object.hasOwn(input, removed), false);
});

test('letter settings keep a failed draft, validate the complete acknowledgement, and reset locally', async context => {
  const { host, requests, notifications } = setup(context, pages.LetterPage);
  const original = { recipient: 'Reader', sender: 'Writer', content: 'Original letter', is_public: false };
  requests[0].resolve(original); await settle(host);
  walk(host.tree, node => node.type === 'textarea')[0].props.onChange({ target: { value: '<script>literal text</script>\nNew letter' } }); host.render();
  walk(host.tree, node => node.type === 'input' && node.props.type === 'checkbox')[0].props.onChange({ target: { checked: true } }); host.render();
  submit(host); const request = requests.at(-1);
  assert.equal(request.url, '/api/v1/content/letter'); assert.equal(request.options.method, 'PUT');
  const input = JSON.parse(request.options.body); assert.equal(input.is_public, true);
  request.resolve({ ...input, is_public: 'true' }); await settle(host);
  assert.match(walk(host.tree, node => node.type === 'textarea')[0].props.value, /literal text/);
  assert.deepEqual(notifications, []);
  button(host, 'Reset changes').props.onClick(); host.render();
  assert.equal(walk(host.tree, node => node.type === 'textarea')[0].props.value, original.content);
  assert.equal(requests.length, 2, 'reset must not fetch over the local draft');
});
