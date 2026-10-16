import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

// Exercise the original TSX and installed shared UI without a browser/DOM.
const components = await originalModule(new URL('./fixtures/react-baseline/src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__ADMIN_PAGINATION_APP;',
  './api': 'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout': 'export const adminGroups=[];',
});
const byName = (tree, name) => walk(tree, node => typeof node.type === 'function' && node.type.name === name);
const button = (tree, label) => walk(expand(tree), node => node.type === 'button' && textContent(node) === label)[0];
const text = host => textContent(expand(host.tree));
const page = (items, total, number = 1) => ({ items, total, page: number, size: 20 });
const settle = async host => { await new Promise(resolve => setImmediate(resolve)); host.render(); };
const cases = [
  { name: 'CommentsPage', path: 'comments', field: 'comment_content', action: 'Delete', empty: 'No comments yet', localEmpty: 'No comments on this page', item: { id: 42, source: 0, comment_type: 'message', user_id: null, username: null, comment_content: 'OLD_ROW', create_time: null } },
  { name: 'TreeHolePage', path: 'tree-hole', field: 'message', action: 'Delete', empty: 'No messages yet', localEmpty: 'No messages on this page', item: { id: 42, user_id: null, username: null, avatar: null, message: 'OLD_ROW', image_path: null, create_time: null } },
  { name: 'NotesPage', path: 'notes', field: 'content', action: 'Delete', empty: 'No notes yet', localEmpty: 'No notes on this page', item: { id: 42, user_id: 1, username: 'owner', content: 'OLD_ROW', image_path: null, is_public: 1, create_time: null } },
  { name: 'ResourcesPage', path: 'resources', field: 'original_name', action: null, empty: 'No files yet', localEmpty: 'No files on this page', item: { id: 42, original_name: 'OLD_ROW', path: '/media/old.png', resource_type: 'image', size: 12, status: 1, create_time: null } },
];
const row = (fixture, label, id = 43) => ({ ...fixture.item, id, [fixture.field]: label });
function setup(context, fixture) {
  const previous = { window: globalThis.window, app: globalThis.__ADMIN_PAGINATION_APP };
  const requests = [], notifications = [];
  const client = () => ({ request(url, validate, options = {}) {
    let resolve, reject;
    const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
    requests.push({ url, options, validate, reject, resolve(value) {
      if (validate(value)) resolve(value);
      else reject(new Error('Response does not match its runtime contract'));
    } });
    return promise;
  } });
  globalThis.window = { confirm: () => true };
  globalThis.__ADMIN_PAGINATION_APP = { client: client(), notify: value => notifications.push(value) };
  const host = new HookHost(components[fixture.name]);
  context.after(() => {
    host.unmount();
    globalThis.window = previous.window;
    globalThis.__ADMIN_PAGINATION_APP = previous.app;
  });
  host.render();
  return { host, requests, notifications, replaceClient() { globalThis.__ADMIN_PAGINATION_APP.client = client(); } };
}
function assertNoRows(tree) {
  assert.equal(byName(tree, 'Table').length, 0);
  assert.equal(byName(tree, 'EmptyState').length, 0);
  for (const label of ['Delete', 'Disable', 'Enable']) assert.equal(button(tree, label), undefined);
  assert.equal(walk(expand(tree), node => node.type === 'a' && node.props.href === '/media/old.png').length, 0);
}
function assertLoading(tree) {
  assertNoRows(tree);
  assert.equal(byName(tree, 'LoadingState').length, 1);
  assert.equal(byName(tree, 'ErrorState').length, 0);
}
function assertFailed(tree) {
  assertNoRows(tree);
  assert.equal(byName(tree, 'LoadingState').length, 0);
  assert.equal(byName(tree, 'ErrorState').length, 1);
}
function retry(host) { byName(host.tree, 'ErrorState')[0].props.onRetry(); host.render(assertLoading); }
function next(host) { button(host.tree, 'Next page').props.onClick(); host.render(); }

for (const fixture of cases) {
  test(`${fixture.name}: page changes immediately hide old rows/actions; failure retries the selected page`, async context => {
    const { host, requests } = setup(context, fixture);
    assertLoading(host.tree);
    requests[0].resolve(page([fixture.item], 41)); await settle(host);
    assert.match(text(host), /OLD_ROW/);
    const nextPage = button(host.tree, 'Next page');
    nextPage.props.onClick(); nextPage.props.onClick();
    host.render(assertLoading); // Check the render before effect cleanup as well.
    assert.equal(requests.length, 2);
    assert.match(requests.at(-1).url, /page=2&size=20$/);
    assert.doesNotMatch(text(host), /OLD_ROW/);
    requests.at(-1).reject(new Error('ordinary network failure')); await settle(host);
    assertFailed(host.tree);
    assert.doesNotMatch(text(host), /OLD_ROW/);
    retry(host);
    assert.match(requests.at(-1).url, /page=2&size=20$/);
    requests.at(-1).resolve(page([row(fixture, 'CURRENT_ROW')], 41, 2)); await settle(host);
    assert.match(text(host), /CURRENT_ROW/);
    assert.match(text(host), /Page 2/);
    assert.equal(byName(host.tree, 'ErrorState').length, 0);
    assert.doesNotMatch(text(host), /OLD_ROW/);
    if (fixture.action) assert.ok(button(host.tree, fixture.action));
  });

  test(`${fixture.name}: initial failure retries without false emptiness; a confirmed empty collection renders empty`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].reject(new Error('offline')); await settle(host);
    assertFailed(host.tree);
    retry(host);
    requests.at(-1).resolve(page([], 0)); await settle(host);
    assert.ok(text(host).includes(fixture.empty));
    assert.equal(byName(host.tree, 'LoadingState').length, 0);
    assert.equal(button(host.tree, 'Previous page'), undefined);
    assert.equal(button(host.tree, 'Next page'), undefined);
  });

  for (const outcome of ['success', 'failure']) {
    test(`${fixture.name}: a changed client owns its rows even after obsolete request ${outcome}`, async context => {
      const { host, requests, replaceClient } = setup(context, fixture);
      requests[0].resolve(page([fixture.item], 41)); await settle(host);
      next(host);
      const obsolete = requests.at(-1);
      replaceClient(); host.render(assertLoading);
      assert.equal(obsolete.options.signal.aborted, true);
      const current = requests.at(-1);
      current.resolve(page([row(fixture, 'CURRENT_CLIENT_ROW')], 41, 2)); await settle(host);
      if (outcome === 'success') obsolete.resolve(page([row(fixture, 'OBSOLETE_ROW')], 41, 2));
      else obsolete.reject(new Error('obsolete failure'));
      await settle(host);
      assert.match(text(host), /CURRENT_CLIENT_ROW/);
      assert.doesNotMatch(text(host), /OLD_ROW|OBSOLETE_ROW/);
      assert.equal(byName(host.tree, 'ErrorState').length, 0);
    });
  }

  test(`${fixture.name}: shrinking totals skip vanished pages and failed fallback retries page one`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].resolve(page([fixture.item], 41)); await settle(host);
    next(host); requests.at(-1).resolve(page([row(fixture, 'PAGE_TWO')], 41, 2)); await settle(host);
    next(host); assert.match(requests.at(-1).url, /page=3/);
    requests.at(-1).resolve(page([], 0, 3)); await settle(host);
    assertLoading(host.tree);
    assert.match(requests.at(-1).url, /page=1/);
    requests.at(-1).reject(new Error('fallback unavailable')); await settle(host);
    assertFailed(host.tree);
    retry(host); assert.match(requests.at(-1).url, /page=1/);
    requests.at(-1).resolve(page([], 0)); await settle(host);
    assert.ok(text(host).includes(fixture.empty));
  });

  test(`${fixture.name}: a nonempty collection does not claim to be empty when only this page is empty`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].resolve(page([], 1)); await settle(host);
    assert.ok(text(host).includes(fixture.localEmpty));
    assert.ok(!text(host).includes(fixture.empty));
  });

  test(`${fixture.name}: invalid pagination metadata fails instead of rendering or clamping`, async context => {
    const { host, requests } = setup(context, fixture);
    const valid = page([fixture.item], 41);
    const request = requests[0];
    assert.equal(request.validate(valid), true);
    for (const invalid of [
      { ...valid, total: -1 }, { ...valid, total: NaN }, { ...valid, total: Infinity }, { ...valid, total: 0.5 },
      { ...valid, page: 2 }, { ...valid, page: undefined }, { ...valid, page: 0 },
      { ...valid, size: 0 }, { ...valid, size: 10 }, { ...valid, size: undefined },
      { ...valid, items: Array(21).fill(fixture.item) },
    ]) assert.equal(request.validate(invalid), false);
    request.resolve({ ...valid, size: 0 }); await settle(host);
    assertFailed(host.tree);
    assert.equal(requests.length, 1);
  });
}

for (const fixture of cases.filter(fixture => fixture.action === 'Delete')) {
  test(`${fixture.name}: deleting the only last-page row reloads the remaining page without false empty state`, async context => {
    const { host, requests, notifications } = setup(context, fixture);
    requests[0].resolve(page([fixture.item], 21)); await settle(host);
    next(host); requests.at(-1).resolve(page([row(fixture, 'LAST_ROW', 21)], 21, 2)); await settle(host);
    button(host.tree, 'Delete').props.onClick();
    const deletion = requests.at(-1);
    assert.equal(deletion.options.method, 'DELETE');
    assert.equal(deletion.url, `/api/v1/content/${fixture.path}/21`);
    deletion.resolve({ deleted: true }); await settle(host);
    assertLoading(host.tree);
    assert.doesNotMatch(text(host), /LAST_ROW/);
    assert.match(requests.at(-1).url, /page=2/);
    requests.at(-1).resolve(page([], 20, 2)); await settle(host);
    assertLoading(host.tree);
    assert.match(requests.at(-1).url, /page=1/);
    requests.at(-1).resolve(page([row(fixture, 'REMAINING_ROW')], 20)); await settle(host);
    assert.match(text(host), /REMAINING_ROW/);
    assert.doesNotMatch(text(host), /LAST_ROW/);
    assert.ok(!text(host).includes(fixture.empty));
    assert.equal(notifications.length, 1);
  });

  test(`${fixture.name}: deletion refresh failure cannot resurrect a deleted row; retry can confirm the last row is gone`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].resolve(page([fixture.item], 1)); await settle(host);
    button(host.tree, 'Delete').props.onClick();
    requests.at(-1).resolve({ deleted: true }); await settle(host);
    assertLoading(host.tree);
    requests.at(-1).reject(new Error('refresh failed')); await settle(host);
    assertFailed(host.tree);
    assert.doesNotMatch(text(host), /OLD_ROW/);
    retry(host);
    assert.equal(requests.filter(request => request.options.method === 'DELETE').length, 1);
    requests.at(-1).resolve(page([], 0)); await settle(host);
    assert.ok(text(host).includes(fixture.empty));
  });
}

for (const fixture of cases.filter(fixture => ['NotesPage', 'ResourcesPage'].includes(fixture.name))) {
  test(`${fixture.name}: page-load retries preserve unsaved input and never submit it`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].resolve(page([fixture.item], 41)); await settle(host);
    const file = new File(['image'], 'draft.png', { type: 'image/png' });
    const input = walk(expand(host.tree), node => fixture.name === 'NotesPage' ? node.type === 'textarea' : node.type === 'input' && node.props.type === 'file')[0];
    input.props.onChange({ target: { value: 'UNSAVED_DRAFT', files: [file] } }); host.render();
    next(host); requests.at(-1).reject(new Error('page unavailable')); await settle(host);
    retry(host);
    assert.equal(requests.filter(request => request.options.method).length, 0);
    if (fixture.name === 'NotesPage') assert.equal(walk(host.tree, node => node.type === 'textarea')[0].props.value, 'UNSAVED_DRAFT');
    else assert.equal(button(host.tree, 'Upload').props.disabled, false);
    requests.at(-1).resolve(page([row(fixture, 'CURRENT_ROW')], 41, 2)); await settle(host);
    walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
    const submitted = requests.at(-1);
    assert.equal(submitted.options.method, 'POST');
    if (fixture.name === 'NotesPage') assert.equal(JSON.parse(submitted.options.body).content, 'UNSAVED_DRAFT');
    else assert.equal(submitted.options.body, file);
  });

  test(`${fixture.name}: a submission failure after changing pages stays visible and preserves the retry payload`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].resolve(page([fixture.item], 41)); await settle(host);
    const file = new File(['image'], 'draft.png', { type: 'image/png' });
    const input = walk(expand(host.tree), node => fixture.name === 'NotesPage' ? node.type === 'textarea' : node.type === 'input' && node.props.type === 'file')[0];
    input.props.onChange({ target: { value: 'UNSAVED_DRAFT', files: [file] } }); host.render();
    walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
    const submission = requests.at(-1);
    next(host);
    const list = requests.at(-1);
    submission.reject(new Error('submission failed after navigation')); await settle(host);
    assert.equal(byName(host.tree, 'ErrorState').length, 1);
    list.resolve(page([row(fixture, 'CURRENT_ROW')], 41, 2)); await settle(host);
    assert.equal(byName(host.tree, 'ErrorState').length, 1, 'list success cannot erase a form failure');
    walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
    assert.equal(requests.at(-1).options.method, 'POST');
    assert.equal(requests.at(-1).options.body, submission.options.body);
  });

  test(`${fixture.name}: read retry neither resubmits nor hides a simultaneous form failure`, async context => {
    const { host, requests } = setup(context, fixture);
    requests[0].resolve(page([fixture.item], 41)); await settle(host);
    const input = walk(expand(host.tree), node => fixture.name === 'NotesPage' ? node.type === 'textarea' : node.type === 'input' && node.props.type === 'file')[0];
    input.props.onChange({ target: { value: 'UNSAVED_DRAFT', files: [new File(['image'], 'draft.png', { type: 'image/png' })] } }); host.render();
    walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
    const submission = requests.at(-1);
    next(host);
    submission.reject(new Error('submission unavailable'));
    requests.at(-1).reject(new Error('page unavailable')); await settle(host);
    assert.equal(byName(host.tree, 'ErrorState').length, 2);
    const loadError = byName(host.tree, 'ErrorState').find(node => node.props.onRetry);
    assert.ok(loadError);
    loadError.props.onRetry(); host.render();
    assert.equal(byName(host.tree, 'LoadingState').length, 1);
    assert.equal(byName(host.tree, 'ErrorState').length, 1, 'the form error stays visible during read retry');
    const formRetry = byName(host.tree, 'ErrorState')[0].props.onRetry;
    if (fixture.name === 'ResourcesPage') {
      assert.equal(typeof formRetry, 'function');
      formRetry(); host.render();
      assert.equal(requests.at(-1).options.method, undefined, 'Refreshing after a file failure only reads the selected list page');
      assert.match(requests.at(-1).url, /page=2&size=20$/);
      assert.equal(byName(host.tree, 'ErrorState').length, 1, 'Refreshing cannot hide the upload failure');
    } else assert.equal(formRetry, undefined);
    assert.equal(requests.filter(request => request.options.method === 'POST').length, 1);
    requests.at(-1).resolve(page([row(fixture, 'CURRENT_ROW')], 41, 2)); await settle(host);
    assert.equal(byName(host.tree, 'ErrorState').length, 1);
    walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} }); host.render();
    assert.equal(byName(host.tree, 'ErrorState').length, 0);
    assert.equal(requests.at(-1).options.body, submission.options.body);
  });
}

test('NotesPage: publishing during page-two loading selects page one and ignores the obsolete page response', async context => {
  const fixture = cases.find(fixture => fixture.name === 'NotesPage');
  const { host, requests } = setup(context, fixture);
  requests[0].resolve(page([fixture.item], 41)); await settle(host);
  next(host); const obsolete = requests.at(-1);
  walk(host.tree, node => node.type === 'textarea')[0].props.onChange({ target: { value: 'NEW_POST' } }); host.render();
  walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
  requests.at(-1).resolve(row(fixture, 'NEW_POST')); await settle(host);
  assertLoading(host.tree);
  assert.match(requests.at(-1).url, /page=1/);
  assert.equal(obsolete.options.signal.aborted, true);
  requests.at(-1).resolve(page([row(fixture, 'NEW_POST')], 42)); await settle(host);
  obsolete.resolve(page([], 0, 2)); await settle(host);
  assert.match(text(host), /NEW_POST/);
  assert.match(text(host), /Page 1/);
  assert.equal(walk(host.tree, node => node.type === 'textarea')[0].props.value, '');
});
