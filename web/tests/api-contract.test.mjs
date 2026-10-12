import assert from 'node:assert/strict';
import { test } from 'node:test';
import { ApiContractError, ApiError, publicErrorMessage, request } from '../src/api.ts';
import * as contracts from '../src/public-contracts.ts';
import { article, site, summary } from './browser-fixtures.mjs';

const comment = { id: 1, user_id: null, username: null, avatar: null, comment_content: 'A comment', create_time: null, parent_username: null, reply_count: 0 };
const page = items => ({ items, total: items.length, page: 1, size: 10 });
const response = (body, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } });
function mockFetch(context, fetch) {
  const previous = globalThis.fetch;
  globalThis.fetch = fetch;
  context.after(() => { globalThis.fetch = previous; });
}

const fixtures = [
  ['article summary page', contracts.isArticlePage, page([summary]), value => { value.items[0].view_count = '1'; }],
  ['article details without summary-only fields', contracts.isArticle, article, value => { value.article_content = []; }],
  ['article access', contracts.isArticleAccess, { password_required: 1, tips: null }, value => { value.password_required = 2; }],
  ['categories', contracts.isCategories, [{ id: 1, sort_name: 'Category', sort_description: null, priority: -1, article_count: 0 }], value => { value[0].sort_name = null; }],
  ['public labels', contracts.isPublicLabels, [{ id: 1, sort_id: 1, label_name: 'Tag', label_description: null, article_count: 0 }], value => { value[0].article_count = -1; }],
  ['editor labels', contracts.isEditorLabels, [{ id: 1, sort_id: 1, label_name: 'Tag', label_description: null }], value => { value[0].sort_id = '1'; }],
  ['site', contracts.isSiteInfo, site, value => { value.notices = ['wrong shape']; }],
  ['site statistics', contracts.isSiteStats, { article_count: 2, view_count: 5 }, value => { value.view_count = 0.5; }],
  ['home sections', contracts.isHomeSections, [{ id: 1, title: 'Latest', kind: 'latest', sort_id: null, priority: 0, enabled: true }], value => { value[0].enabled = 1; }],
  ['comment page', contracts.isPublicCommentPage, page([comment]), value => { value.items[0].reply_count = -1; }],
  ['comment post acknowledgement', contracts.isPublicComment, comment, value => { delete value.comment_content; }],
  ['public letter', contracts.isPublicLetter, {recipient:'Reader',sender:'Writer',content:'Letter'}, value=>{value.content=false;}],
  ['article news', contracts.isNews, [{ id: 1, content: 'Update', create_time: null }], value => { value[0].content = {}; }],
  ['note page', contracts.isNotePage, page([{ id: 1, user_id: 1, username: null, content: 'Moment', image_path: null, create_time: null, like_count: 0, is_public: 1 }]), value => { value.items[0].image_path = false; }],
  ['wall posts', contracts.isWallPosts, [{ id: 1, user_id: null, username: null, avatar: null, message: 'Hello', image_path: null, create_time: null }], value => { value[0].message = {}; }],
  ['link page', contracts.isLinkPage, page([{ id: 1, title: 'Photo', classify: null, cover: null, url: null, introduction: null, link_type: 'lovePhoto', create_time: null }]), value => { value.items[0].cover = []; }],
  ['album classes', contracts.isLinkClasses, [{ classify: 'Photo', count: 0 }], value => { value[0].count = '0'; }],
  ['family', contracts.isFamilies, [{ id: 1, bg_cover: null, man_cover: null, woman_cover: null, man_name: null, woman_name: null, timing: null, countdown_title: null, countdown_time: null, family_info: null }], value => { value[0].timing = 123; }],
];
for (const [name, validate, valid, corrupt] of fixtures) {
  test(`actual transport accepts current ${name} and rejects malformed consumed fields`, async context => {
    let body = valid;
    mockFetch(context, async () => response(body));
    assert.deepEqual(await request('/api/v1/fixture', validate), valid);
    body = structuredClone(valid); corrupt(body);
    await assert.rejects(request('/api/v1/fixture', validate), error => error instanceof ApiContractError && error.reason === 'invalid_response' && error.status === 200);
    for (body of [null, 'invalid', 0]) {
      if (name === 'public letter' && body === null) assert.equal(await request('/api/v1/fixture', validate), null, 'An unpublished letter has a valid null read model');
      else await assert.rejects(request('/api/v1/fixture', validate), ApiContractError);
    }
  });
}

test('malformed page container, items, missing fields and unsafe pagination are rejected before consumers run', async context => {
  let body;
  mockFetch(context, async () => response(body));
  for (body of [
    { items: 'not-an-array', total: 1 },
    { ...page([comment]), items: [null] },
    { ...page([comment]), items: [{ ...comment, avatar: 7 }] },
    { ...page([comment]), total: -1 },
    { ...page([comment]), total: 1.1 },
    { ...page([comment]), total: Number.MAX_SAFE_INTEGER + 1 },
    { ...page([comment]), page: 0 },
    { ...page([comment]), page: 1.1 },
    { ...page([comment]), page: 100_001 },
    { ...page([comment]), size: 0 },
    { ...page([comment]), size: 51 },
    { ...page([comment]), size: '10' },
    { ...page([comment]), page: undefined },
    { ...page([comment]), items: [comment, comment], size: 1 },
  ]) {
    let consumed = false;
    await assert.rejects(request('/api/v1/message-comments?page=1&size=10', contracts.isPublicCommentPage).then(value => {
      consumed = true;
      value.items.map(item => item.comment_content);
    }), ApiContractError);
    assert.equal(consumed, false);
  }
  body = page([]);
  assert.deepEqual(await request('/api/v1/message-comments', contracts.isPublicCommentPage), body);
});

test('a malformed successful write is distinguishable from a server rejection and is never retried', async context => {
  let calls = 0, status = 201, body = '{invalid json';
  mockFetch(context, async () => { calls++; return new Response(body, { status }); });
  await assert.rejects(request('/api/v1/comments', contracts.isPublicComment, { method: 'POST', body: '{}' }), error => {
    assert.ok(error instanceof ApiContractError);
    assert.equal(error instanceof ApiError, false);
    assert.equal(error.reason, 'invalid_json');
    assert.match(publicErrorMessage(error), /could not be confirmed/);
    return true;
  });
  assert.equal(calls, 1);
  body = '{}';
  await assert.rejects(request('/api/v1/comments', contracts.isPublicComment, { method: 'POST', body: '{}' }), ApiContractError);
  assert.equal(calls, 2);
  status = 400;
  await assert.rejects(request('/api/v1/comments', contracts.isPublicComment), error => error instanceof ApiError && !(error instanceof ApiContractError) && error.status === 400);
  body = 'not json';
  await assert.rejects(request('/api/v1/comments', contracts.isPublicComment), ApiError);
});

test('request requires an explicit validator and preserves transport safety options', async context => {
  let calls = 0;
  mockFetch(context, async (_url, options) => {
    calls++;
    assert.equal(options.credentials, 'same-origin');
    assert.equal(options.redirect, 'error');
    assert.equal(options.headers.get('Content-Type'), 'application/json');
    assert.equal(options.headers.get('X-CSRF-Token'), 'test-csrf');
    return response(comment);
  });
  await assert.rejects(request('/api/v1/comments'), TypeError);
  assert.equal(calls, 0);
  assert.deepEqual(await request('/api/v1/comments', contracts.isPublicComment, { method: 'POST', body: '{}' }, 'test-csrf'), comment);
  assert.equal(calls, 1);
});

// Zero is the current stored sentinel for an article without a category/tag.
test('article read models accept the current uncategorized sentinel', () => {
  assert.equal(contracts.isArticleSummary({ ...summary, sort_id: 0, label_id: 0, sort_name: null, label_name: null }), true);
  assert.equal(contracts.isArticle({ ...article, sort_id: 0, label_id: 0, sort_name: null, label_name: null }), true);
});
