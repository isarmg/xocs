import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

const { LabelsPage } = await originalModule(new URL('../src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__LABEL_CATEGORIES_APP;',
  './api': 'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout': 'export const adminGroups=[];',
});
const category = { id: 2, sort_name: 'Category two', sort_description: null, priority: 0, article_count: 0 };
const label = { id: 1, sort_id: 2, label_name: 'Existing tag', label_description: 'Original description' };
const byName = (host, name) => walk(host.tree, node => typeof node.type === 'function' && node.type.name === name);
const button = (host, name) => walk(expand(host.tree), node => node.type === 'button' && textContent(node) === name)[0];
const text = host => textContent(expand(host.tree));
const settle = async host => { await new Promise(resolve => setImmediate(resolve)); host.render(); };
const submit = host => walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
function setup(context) {
  const previous = globalThis.__LABEL_CATEGORIES_APP;
  const requests = [];
  const client = () => ({ request(url, validate, options = {}) {
    let resolve, reject;
    const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
    requests.push({ url, options, reject, resolve(value) {
      if (validate(value)) resolve(value);
      else reject(new Error('Invalid response fixture'));
    } });
    return promise;
  } });
  globalThis.__LABEL_CATEGORIES_APP = { client: client(), notify() {} };
  const host = new HookHost(LabelsPage);
  context.after(() => { host.unmount(); globalThis.__LABEL_CATEGORIES_APP = previous; });
  host.render();
  return { host, requests, replaceClient() { globalThis.__LABEL_CATEGORIES_APP.client = client(); } };
}
function categoryError(host) { return byName(host, 'ErrorState').find(node => textContent(node).includes('Unable to load categories')); }
function retryCategories(host) { categoryError(host).props.onRetry(); host.render(); }

test('LabelsPage: pending categories are visible and block both native submission and direct submit handlers', context => {
  const { host, requests } = setup(context);
  assert.match(text(host), /Loading categories/);
  assert.equal(button(host, 'Add tag').props.disabled, true);
  assert.equal(byName(host, 'Select')[0].props.disabled, true);
  submit(host);
  assert.equal(requests.length, 2);
});

test('LabelsPage: category errors survive later tag-list success; category-only retry keeps the create draft', async context => {
  const { host, requests } = setup(context);
  requests[1].reject(new Error('categories unavailable')); await settle(host);
  requests[0].resolve([]); await settle(host);
  assert.ok(categoryError(host));
  assert.equal(button(host, 'Add tag').props.disabled, true);
  byName(host, 'TextField')[0].props.onChange({ target: { value: 'Draft tag' } });
  byName(host, 'TextField')[1].props.onChange({ target: { value: 'Draft description' } }); host.render();
  submit(host); assert.equal(requests.length, 2);
  retryCategories(host);
  assert.equal(categoryError(host), undefined);
  assert.match(text(host), /Loading categories/);
  assert.equal(requests.length, 3);
  assert.equal(requests.at(-1).url, '/api/v1/categories');
  assert.equal(requests.at(-1).options.method, undefined);
  assert.equal(byName(host, 'TextField')[0].props.value, 'Draft tag');
  requests.at(-1).resolve([category]); await settle(host);
  assert.equal(button(host, 'Add tag').props.disabled, true, 'a real category selection is still required');
  byName(host, 'Select')[0].props.onChange({ target: { value: '2' } }); host.render();
  assert.equal(button(host, 'Add tag').props.disabled, false);
  submit(host);
  assert.equal(requests.at(-1).options.method, 'POST');
  assert.deepEqual(JSON.parse(requests.at(-1).options.body), { sort_id: 2, name: 'Draft tag', description: 'Draft description' });
});

test('LabelsPage: category retry preserves the selected tag and all edited fields', async context => {
  const { host, requests } = setup(context);
  requests[0].resolve([label]); requests[1].reject(new Error('offline')); await settle(host);
  button(host, 'Edit').props.onClick(); host.render();
  byName(host, 'TextField')[0].props.onChange({ target: { value: 'Edited tag' } });
  byName(host, 'TextField')[1].props.onChange({ target: { value: 'Edited description' } }); host.render();
  assert.equal(button(host, 'Save changes').props.disabled, true);
  retryCategories(host);
  requests.at(-1).resolve([category]); await settle(host);
  assert.equal(byName(host, 'Select')[0].props.value, 2);
  assert.equal(byName(host, 'TextField')[0].props.value, 'Edited tag');
  assert.equal(byName(host, 'TextField')[1].props.value, 'Edited description');
  assert.equal(button(host, 'Save changes').props.disabled, false);
  submit(host);
  assert.equal(requests.at(-1).url, '/api/v1/content/labels/1');
  assert.equal(requests.at(-1).options.method, 'PUT');
  assert.deepEqual(JSON.parse(requests.at(-1).options.body), { sort_id: 2, name: 'Edited tag', description: 'Edited description' });
});

test('LabelsPage: tag-list errors and retries are independent of categories', async context => {
  const { host, requests } = setup(context);
  requests[0].reject(new Error('tags unavailable')); requests[1].reject(new Error('categories unavailable')); await settle(host);
  assert.equal(byName(host, 'ErrorState').length, 2);
  retryCategories(host); requests.at(-1).resolve([category]); await settle(host);
  assert.match(text(host), /Unable to load tags/);
  assert.equal(categoryError(host), undefined);
  byName(host, 'ErrorState')[0].props.onRetry(); host.render();
  assert.equal(requests.at(-1).url, '/api/v1/content/labels');
  assert.equal(requests.filter(request => request.url === '/api/v1/categories').length, 2);
  assert.equal(byName(host, 'Select')[0].props.disabled, false);
  requests.at(-1).resolve([]); await settle(host);
  assert.equal(byName(host, 'ErrorState').length, 0);
});

test('LabelsPage: a confirmed empty category list explains the missing prerequisite and prevents invalid submission', async context => {
  const { host, requests } = setup(context);
  requests[0].resolve([]); requests[1].resolve([]); await settle(host);
  assert.match(text(host), /Create a category before adding tags/);
  assert.equal(button(host, 'Add tag').props.disabled, true);
  submit(host); assert.equal(requests.length, 2);
});

test('LabelsPage: a changed client invalidates loaded categories before effects and ignores obsolete category results', async context => {
  const { host, requests, replaceClient } = setup(context);
  requests[0].resolve([label]); requests[1].resolve([category]); await settle(host);
  button(host, 'Edit').props.onClick(); host.render();
  assert.equal(button(host, 'Save changes').props.disabled, false);
  replaceClient(); host.render(() => {
    assert.equal(button(host, 'Save changes').props.disabled, true);
    assert.equal(byName(host, 'Select')[0].props.disabled, true);
  });
  const obsolete = requests.at(-1);
  replaceClient(); host.render();
  assert.equal(obsolete.options.signal.aborted, true);
  requests.at(-1).resolve([category]); await settle(host);
  obsolete.resolve([]); await settle(host);
  assert.equal(button(host, 'Save changes').props.disabled, false);
  assert.match(text(host), /Category two/);
});
