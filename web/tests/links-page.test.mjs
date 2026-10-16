import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

const { LinksPage } = await originalModule(new URL('./fixtures/react-baseline/src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__LINKS_PAGE_APP;',
  './api': 'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout': 'export const adminGroups=[];',
});

const byName = (tree, name) => walk(tree, node => typeof node.type === 'function' && node.type.name === name);
const button = (tree, label) => walk(expand(tree), node => node.type === 'button' && textContent(node) === label)[0];
const text = host => textContent(expand(host.tree));
const link = (id, title, link_type = 'favorites', status = 1) => ({
  id, title, link_type, classify: null, cover: null, url: null, introduction: null, status, create_time: null,
});
const page = (items, total, number = 1) => ({ items, total, page: number, size: 20 });
const settle = async host => {
  for (let index = 0; index < 6; index++) await Promise.resolve();
  return host.render();
};
function assertLoading(tree) {
  assert.equal(byName(tree, 'LoadingState').length, 1);
  assert.equal(byName(tree, 'EmptyState').length, 0);
  for (const label of ['Edit', 'Delete', 'Approve']) assert.equal(button(tree, label), undefined);
}
function setup(context) {
  const requests = [], notifications = [];
  const client = { request(url, validate, options = {}) {
    let resolve, reject;
    const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
    requests.push({ url, options, validate, reject, resolve(value) {
      if (validate(value)) resolve(value);
      else reject(new Error('Response does not match its runtime contract'));
    } });
    return promise;
  } };
  globalThis.__LINKS_PAGE_APP = { client, notify: message => notifications.push(message) };
  globalThis.window = { confirm: () => true };
  const host = new HookHost(LinksPage);
  context.after(() => {
    host.unmount();
    delete globalThis.__LINKS_PAGE_APP;
    delete globalThis.window;
  });
  host.render();
  return { host, requests, notifications };
}

test('page and type changes hide old rows and actions before effects run; obsolete responses are ignored', async context => {
  const { host, requests } = setup(context);
  assertLoading(host.tree);
  requests.at(-1).resolve(page([link(1, 'OLD_FAVORITE')], 41));
  await settle(host);
  assert.match(text(host), /OLD_FAVORITE/);
  button(host.tree, 'Next page').props.onClick();
  host.render(tree => {
    assertLoading(tree);
    assert.doesNotMatch(textContent(tree), /OLD_FAVORITE/);
  });
  const pageTwo = requests.at(-1);
  assert.match(pageTwo.url, /page=2/);
  byName(host.tree, 'Select')[0].props.onChange({ target: { value: 'lovePhoto' } });
  host.render(tree => {
    assertLoading(tree);
    assert.doesNotMatch(textContent(tree), /OLD_FAVORITE/);
  });
  const photos = requests.at(-1);
  assert.match(photos.url, /page=1.*kind=lovePhoto/);
  assert.equal(pageTwo.options.signal.aborted, true);
  photos.resolve(page([link(2, 'CURRENT_PHOTO', 'lovePhoto')], 1));
  await settle(host);
  pageTwo.resolve(page([link(21, 'LATE_FAVORITE')], 41, 2));
  await settle(host);
  assert.match(text(host), /CURRENT_PHOTO/);
  assert.doesNotMatch(text(host), /OLD_FAVORITE|LATE_FAVORITE/);
});

test('a failed filter load exposes retry without stale rows, stale actions, or a false empty result', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'OLD_FRIEND', 'friendUrl', 0)], 1));
  await settle(host);
  assert.ok(button(host.tree, 'Approve'));
  byName(host.tree, 'Select')[1].props.onChange({ target: { value: '1' } });
  host.render(assertLoading);
  assert.match(requests.at(-1).url, /page=1.*status=1/);
  requests.at(-1).reject(new Error('ordinary network failure'));
  await settle(host);
  assert.equal(byName(host.tree, 'ErrorState').length, 1);
  assert.equal(byName(host.tree, 'EmptyState').length, 0);
  assert.equal(byName(host.tree, 'LoadingState').length, 0);
  assert.doesNotMatch(text(host), /OLD_FRIEND/);
  for (const label of ['Edit', 'Delete', 'Approve']) assert.equal(button(host.tree, label), undefined);
  byName(host.tree, 'ErrorState')[0].props.onRetry();
  host.render(tree => {
    assertLoading(tree);
    assert.equal(byName(tree, 'ErrorState').length, 0);
  });
  requests.at(-1).resolve(page([link(2, 'PUBLISHED_FRIEND', 'friendUrl')], 1));
  await settle(host);
  assert.match(text(host), /PUBLISHED_FRIEND/);
  assert.equal(byName(host.tree, 'ErrorState').length, 0);
});

test('deleting the sole last-page row reloads the clamped page without declaring the collection empty', async context => {
  const { host, requests, notifications } = setup(context);
  requests.at(-1).resolve(page([link(1, 'PAGE_ONE')], 21));
  await settle(host);
  button(host.tree, 'Next page').props.onClick();
  host.render();
  requests.at(-1).resolve(page([link(21, 'LAST_ROW')], 21, 2));
  await settle(host);
  button(host.tree, 'Delete').props.onClick();
  const deletion = requests.at(-1);
  assert.equal(deletion.options.method, 'DELETE');
  assert.match(deletion.url, /\/links\/21$/);
  deletion.resolve({ deleted: true });
  await settle(host);
  assertLoading(host.tree);
  assert.doesNotMatch(text(host), /LAST_ROW/);
  const reload = requests.at(-1);
  assert.match(reload.url, /page=2/);
  reload.resolve(page([], 20, 2));
  await settle(host);
  assertLoading(host.tree);
  assert.doesNotMatch(text(host), /No content resources yet/);
  assert.match(requests.at(-1).url, /page=1/);
  requests.at(-1).resolve(page([link(1, 'REMAINING_ROW')], 20));
  await settle(host);
  assert.match(text(host), /REMAINING_ROW/);
  assert.doesNotMatch(text(host), /No content resources yet|LAST_ROW/);
  assert.deepEqual(notifications, ['Resource deleted']);
});

test('deleting the final collection row produces the genuine empty state only after the reload', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'FINAL_ROW')], 1));
  await settle(host);
  button(host.tree, 'Delete').props.onClick();
  requests.at(-1).resolve({ deleted: true });
  await settle(host);
  assertLoading(host.tree);
  requests.at(-1).resolve(page([], 0));
  await settle(host);
  assert.match(text(host), /No content resources yet/);
  assert.equal(byName(host.tree, 'EmptyState').length, 1);
  assert.equal(byName(host.tree, 'LoadingState').length, 0);
  assert.equal(button(host.tree, 'Previous page'), undefined);
});

test('refresh failure after deletion does not resurrect the deleted row and can recover', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'DELETED_ROW')], 1));
  await settle(host);
  button(host.tree, 'Delete').props.onClick();
  requests.at(-1).resolve({ deleted: true });
  await settle(host);
  requests.at(-1).reject(new Error('reload unavailable'));
  await settle(host);
  assert.doesNotMatch(text(host), /DELETED_ROW/);
  assert.equal(byName(host.tree, 'ErrorState').length, 1);
  assert.equal(byName(host.tree, 'EmptyState').length, 0);
  assert.equal(button(host.tree, 'Delete'), undefined);
  byName(host.tree, 'ErrorState')[0].props.onRetry();
  host.render(assertLoading);
  requests.at(-1).resolve(page([], 0));
  await settle(host);
  assert.equal(byName(host.tree, 'EmptyState').length, 1);
});

test('page totals can skip multiple vanished pages and retain a retry when the fallback request fails', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'ROW_ONE')], 41));
  await settle(host);
  button(host.tree, 'Next page').props.onClick();
  host.render();
  requests.at(-1).resolve(page([link(21, 'ROW_TWO')], 41, 2));
  await settle(host);
  button(host.tree, 'Next page').props.onClick();
  host.render();
  assert.match(requests.at(-1).url, /page=3/);
  requests.at(-1).resolve(page([], 0, 3));
  await settle(host);
  assertLoading(host.tree);
  assert.match(requests.at(-1).url, /page=1/);
  requests.at(-1).reject(new Error('first page unavailable'));
  await settle(host);
  assert.equal(byName(host.tree, 'EmptyState').length, 0);
  assert.equal(byName(host.tree, 'ErrorState').length, 1);
  byName(host.tree, 'ErrorState')[0].props.onRetry();
  host.render();
  requests.at(-1).resolve(page([], 0));
  await settle(host);
  assert.match(text(host), /No content resources yet/);
});

test('the shared pager permits returning from page 2 even when the collection now fits one page', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'ROW')], 21));
  await settle(host);
  const pager = byName(host.tree, 'AdminPager')[0];
  let selected;
  const tree = pager.type({ data: page([], 20, 2), page: 2, onPage: value => { selected = value; } });
  assert.equal(button(tree, 'Previous page').props.disabled, false);
  assert.equal(button(tree, 'Next page').props.disabled, true);
  button(tree, 'Previous page').props.onClick();
  assert.equal(selected, 1);
  assert.equal(pager.type({ data: page([], 0), page: 1, onPage() {} }), null);
});

test('an empty page of a nonempty collection is described as page-local emptiness', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([], 1));
  await settle(host);
  assert.match(text(host), /No resources on this page/);
  assert.doesNotMatch(text(host), /No content resources yet/);
});

test('malformed rows and pagination metadata fail validation instead of rendering or clamping', async context => {
  const { host, requests } = setup(context);
  const valid = page([link(1, 'ROW')], 1);
  const invalid = [
    { ...valid, total: -1 }, { ...valid, total: NaN }, { ...valid, total: Infinity },
    { ...valid, total: 1.5 }, { ...valid, page: 2 }, { ...valid, page: undefined },
    { ...valid, size: 0 }, { ...valid, size: 10 }, { ...valid, size: undefined },
    { ...valid, items: [null] }, { ...valid, items: [{ id: 1, link_type: 'favorites' }] },
    { ...valid, items: [{ ...link(1, 'ROW'), title: {} }] },
    { ...valid, items: [{ ...link(1, 'ROW'), status: 'published' }] },
    { ...valid, items: [{ ...link(1, 'ROW'), id: NaN }] },
  ];
  const request = requests.at(-1);
  assert.equal(request.validate(valid), true);
  for (const value of invalid) assert.equal(request.validate(value), false);
  request.resolve({ ...valid, size: 0 });
  await settle(host);
  assert.equal(byName(host.tree, 'ErrorState').length, 1);
  assert.equal(byName(host.tree, 'EmptyState').length, 0);
  assert.equal(requests.length, 1);
});

test('returning to a previously loaded filter while a different request is pending still reloads without old actions', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'PREVIOUS_ALL')], 1));
  await settle(host);
  byName(host.tree, 'Select')[0].props.onChange({ target: { value: 'lovePhoto' } });
  host.render();
  const photos = requests.at(-1);
  byName(host.tree, 'Select')[0].props.onChange({ target: { value: '' } });
  host.render(tree => {
    assertLoading(tree);
    assert.doesNotMatch(textContent(tree), /PREVIOUS_ALL/);
  });
  assert.equal(photos.options.signal.aborted, true);
  assert.match(requests.at(-1).url, /page=1&size=20$/);
  requests.at(-1).resolve(page([link(2, 'RELOADED_ALL')], 1));
  await settle(host);
  photos.reject(new Error('late obsolete failure'));
  await settle(host);
  assert.match(text(host), /RELOADED_ALL/);
  assert.equal(byName(host.tree, 'ErrorState').length, 0);
});


test('a stale out-of-range response cannot clamp a newer filter before effect cleanup', async context => {
  const { host, requests } = setup(context);
  requests.at(-1).resolve(page([link(1, 'PAGE_ONE')], 61)); await settle(host);
  button(host.tree, 'Next page').props.onClick(); host.render();
  requests.at(-1).resolve(page([link(2, 'PAGE_TWO')], 61, 2)); await settle(host);
  button(host.tree, 'Next page').props.onClick(); host.render();
  const stale = requests.at(-1);
  assert.match(stale.url, /page=3/);
  // Schedule the next filter without running its effects/cleanup yet.
  byName(host.tree, 'Select')[0].props.onChange({ target: { value: 'lovePhoto' } });
  stale.resolve(page([], 21, 3));
  await settle(host);
  assert.match(requests.at(-1).url, /page=1/);
  assert.match(requests.at(-1).url, /kind=lovePhoto/);
  assertLoading(host.tree);
});
