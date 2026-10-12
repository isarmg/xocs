import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mockApi, summary } from './browser-fixtures.mjs';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

// Run the same route handler used by Playwright, without launching a browser.
async function fixtureApi() {
  let handler;
  await mockApi({ async route(_pattern, callback) { handler = callback; } }, { authenticated: true });
  return async path => {
    let response;
    await handler({
      request: () => ({ url: () => new URL(path, 'https://fixture.test').href, method: () => 'GET' }),
      async fulfill(value) { response = value; },
    });
    return response.json;
  };
}

test('browser fixtures honor requested page and size across all paginated admin and public reads', async () => {
  const read = await fixtureApi();
  for (const path of [
    '/api/v1/content/comments', '/api/v1/content/resources',
    '/api/v1/content/tree-hole', '/api/v1/content/notes', '/api/v1/content/links',
    '/api/v1/content/articles', '/api/v1/articles', '/api/v1/notes/page', '/api/v1/links/page',
    '/api/v1/comments', '/api/v1/message-comments', '/api/v1/love-comments',
    '/api/v1/comments/1/replies', '/api/v1/message-comments/1/replies', '/api/v1/love-comments/1/replies',
  ]) {
    const value = await read(`${path}?page=2&size=7`);
    assert.equal(value.page, 2, path);
    assert.equal(value.size, 7, path);
    assert.deepEqual(value.items, [], `${path}: a later page cannot repeat the first-page rows`);
    assert.equal(value.total, path.endsWith('/articles') ? 1 : 0, path);
  }
});

test('browser fixtures paginate article rows without changing their total or authored content', async () => {
  const read = await fixtureApi();
  for (const path of ['/api/v1/articles', '/api/v1/content/articles']) {
    assert.deepEqual(await read(`${path}?page=1&size=1`), { items: [summary], total: 1, page: 1, size: 1 });
    assert.deepEqual(await read(`${path}?page=2&size=1`), { items: [], total: 1, page: 2, size: 1 });
  }
});

test('omitted fixture pagination uses each production endpoint default', async () => {
  const read = await fixtureApi();
  for (const [path, size] of [
    ['/api/v1/content/comments', 20], ['/api/v1/content/notes', 20], ['/api/v1/content/links', 20],
    ['/api/v1/content/articles', 12], ['/api/v1/articles', 12], ['/api/v1/links/page', 12],
    ['/api/v1/notes/page', 10], ['/api/v1/comments', 10], ['/api/v1/message-comments', 10], ['/api/v1/love-comments', 10],
    ['/api/v1/comments/1/replies', 5], ['/api/v1/message-comments/1/replies', 5], ['/api/v1/love-comments/1/replies', 5],
  ]) {
    const value = await read(path);
    assert.equal(value.page, 1, path);
    assert.equal(value.size, size, path);
  }
});

const components = await originalModule(new URL('../src/AdminExtras.tsx', import.meta.url), {
  '@xcss/web/admin-shell': 'export const useAdminApplication=()=>globalThis.__BROWSER_FIXTURE_APP;',
  './api': 'export const publicErrorMessage=(_reason,fallback)=>fallback;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './AdminLayout': 'export const adminGroups=[];',
});
for (const [name, empty] of [
  ['CommentsPage', 'No comments yet'], ['ResourcesPage', 'No files yet'],
  ['TreeHolePage', 'No messages yet'], ['NotesPage', 'No notes yet'], ['LinksPage', 'No content resources yet'],
]) {
  test(`${name}: real browser fixture is accepted by the unchanged production validator and reaches its success empty state`, async context => {
    const read = await fixtureApi(), observations = [];
    const previous = globalThis.__BROWSER_FIXTURE_APP;
    globalThis.__BROWSER_FIXTURE_APP = { notify() {}, client: { async request(path, validate) {
      const value = await read(path), accepted = validate(value);
      observations.push({ path, value, accepted });
      if (!accepted) throw new Error('Fixture violates the production response contract');
      return value;
    } } };
    const host = new HookHost(components[name]);
    context.after(() => { host.unmount(); globalThis.__BROWSER_FIXTURE_APP = previous; });
    host.render();
    await new Promise(resolve => setImmediate(resolve)); host.render();
    assert.equal(observations.length, 1);
    assert.equal(observations[0].accepted, true, JSON.stringify(observations));
    assert.equal(observations[0].value.size, 20);
    const tree = expand(host.tree);
    assert.equal(walk(tree, node => node.props.role === 'alert').length, 0);
    assert.equal(walk(tree, node => node.props.className === 'xcss-loading').length, 0);
    assert.equal(textContent(walk(tree, node => node.props.className === 'xcss-empty')[0]), empty);
  });
}
