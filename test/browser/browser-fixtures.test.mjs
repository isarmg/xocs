import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mockApi, summary } from './browser-fixtures.mjs';

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

// The real router exposes category reads publicly; its managed endpoint is POST-only.
test('browser fixture rejects the nonexistent managed category GET rather than masking a 405',async()=>{
 let handler;await mockApi({async route(_pattern,callback){handler=callback;}},{authenticated:true});let response;
 await handler({request:()=>({url:()=> 'https://fixture.test/api/v1/content/categories',method:()=> 'GET'}),async fulfill(value){response=value;}});
 assert.equal(response.status,405);assert.equal(response.json.code,'method_not_allowed');
 const read=await fixtureApi();assert.ok((await read('/api/v1/categories')).length>0);
});
