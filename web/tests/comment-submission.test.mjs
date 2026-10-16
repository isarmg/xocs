import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { normalizeCommentContent } from './fixtures/react-baseline/src/comment-submission.ts';
import { HookHost, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';

const reexport = path => `export * from ${JSON.stringify(new URL(path, import.meta.url).href)};`;
const { CommentThread, PublicComments } = await originalModule(new URL('./fixtures/react-baseline/src/PublicComments.tsx', import.meta.url), {
  './api': reexport('./fixtures/react-baseline/src/api.ts'),
  './public-contracts': reexport('./fixtures/react-baseline/src/public-contracts.ts'),
  './comment-submission': reexport('./fixtures/react-baseline/src/comment-submission.ts'),
  './MediaPreview': 'export const safeImageUrl=value=>value;',
});
const json = (body, status = 200) => new Response(JSON.stringify(body), { status });
const comment = body => ({ id: 1, user_id: null, username: null, avatar: null, comment_content: body.content, create_time: null, parent_username: null, reply_count: 0 });
const field = host => walk(host.tree, node => node.type === 'textarea')[0];
const submit = host => walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
const text = host => textContent(host.tree);
const settle = async host => { await new Promise(resolve => setImmediate(resolve)); return host.render(); };
function setup(context, post, read = () => json({ items: [], page: 1, size: 10, total: 0 })) {
  const originalFetch = globalThis.fetch, storageDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'sessionStorage'), originalWindow = globalThis.window;
  const stored = new Map(), calls = [];
  const storage = { getItem: key => stored.get(key) ?? null, setItem: (key, value) => stored.set(key, value), removeItem: key => stored.delete(key) };
  Object.defineProperty(globalThis, 'sessionStorage', { configurable: true, writable: true, value: storage });
  globalThis.window = { confirm: () => true };
  globalThis.fetch = async (url, options) => {
    if (options.method === 'POST') {
      const body = JSON.parse(options.body); calls.push({ url, body });
      assert.ok([...stored.values()].some(value => JSON.parse(value).request_id === body.request_id), 'identity must be durable before the request');
      return post(body, calls.length);
    }
    return read(url);
  };
  const hosts = [];
  const mount = (props = { kind: 'message' }) => {
    const host = new HookHost(CommentThread, props); hosts.push(host); host.render(); return host;
  };
  context.after(() => {
    hosts.forEach(host => host.unmount()); globalThis.fetch = originalFetch; globalThis.window = originalWindow;
    if (storageDescriptor) Object.defineProperty(globalThis, 'sessionStorage', storageDescriptor); else delete globalThis.sessionStorage;
  });
  return { stored, calls, storage, mount };
}
function draft(host, value = 'One comment') { field(host).props.onChange({ target: { value } }); host.render(); }

test('lost successful response preserves the exact submission through retry and reload', async context => {
  const { stored, calls, mount } = setup(context, (body, count) => count === 1 ? Promise.reject(new TypeError('connection closed after commit')) : json(comment(body)));
  const host = mount(); await settle(host); draft(host); submit(host); await settle(host);
  assert.equal(calls.length, 1); assert.equal(stored.size, 1);
  assert.match(calls[0].body.request_id, /^[0-9a-f-]{36}$/);
  assert.match(text(host), /not yet confirmed/); assert.doesNotMatch(text(host), /Unable to publish/);
  assert.equal(field(host).props.disabled, true);
  host.unmount();
  const recovered = mount(); await settle(recovered);
  assert.equal(calls.length, 1, 'reload must not automatically resend');
  assert.equal(field(recovered).props.value, 'One comment');
  assert.match(text(recovered), /awaiting confirmation/);
  submit(recovered); await settle(recovered);
  assert.deepEqual(calls[1], calls[0]);
  assert.equal(stored.size, 0); assert.equal(field(recovered).props.value, '');
  assert.match(text(recovered), /has been posted/);
});

test('synchronous repeated submits send only one request while the first is pending', async context => {
  let finish;
  const { calls, mount } = setup(context, body => new Promise(resolve => { finish = () => resolve(json(comment(body))); }));
  const host = mount(); await settle(host); draft(host);
  submit(host); submit(host); host.render();
  assert.equal(calls.length, 1); assert.equal(field(host).props.disabled, true);
  finish(); await settle(host); assert.match(text(host), /has been posted/);
});

for (const malformed of [null, { id: 2, comment_content: 'wrong shape' }, comment({ content: 'different request' })]) {
  test(`malformed write success remains unknown and retries with the same identity: ${JSON.stringify(malformed)}`, async context => {
    const { calls, mount } = setup(context, (body, count) => json(count === 1 ? malformed : comment(body)));
    const host = mount(); await settle(host); draft(host); submit(host); await settle(host);
    assert.match(text(host), /not yet confirmed/); assert.equal(field(host).props.disabled, true);
    submit(host); await settle(host); assert.deepEqual(calls[1].body, calls[0].body);
  });
}

test('a known first-request rejection retains the editable draft without claiming an unknown success', async context => {
  const { stored, calls, mount } = setup(context, (body, count) => count === 1
    ? json({ code: 'too_many_requests', message: 'localizable code', retryable: true }, 429) : json(comment(body)));
  const host = mount(); await settle(host); draft(host); submit(host); await settle(host);
  assert.equal(stored.size, 0); assert.equal(field(host).props.disabled, false);
  assert.equal(field(host).props.value, 'One comment'); assert.match(text(host), /Too many requests/);
  submit(host); await settle(host); assert.notEqual(calls[1].body.request_id, calls[0].body.request_id);
});

test('an error after an unknown result cannot unlock or silently replace the original request', async context => {
  const { calls, stored, mount } = setup(context, (_body, count) => count === 1
    ? Promise.reject(new TypeError('lost response')) : json({ code: 'forbidden', message: 'closed', retryable: false }, 403));
  const host = mount(); await settle(host); draft(host); submit(host); await settle(host); submit(host); await settle(host);
  assert.deepEqual(calls[1], calls[0]); assert.equal(stored.size, 1); assert.equal(field(host).props.disabled, true);
  assert.match(text(host), /not yet confirmed/);
});

test('storage failure prevents an untracked write and clearing pending state requires explicit confirmation', async context => {
  const { calls, storage, mount } = setup(context, body => json(comment(body)));
  const host = mount(); await settle(host); draft(host);
  storage.setItem = () => { throw new Error('storage unavailable'); };
  submit(host); await settle(host); assert.equal(calls.length, 0);
  assert.match(text(host), /could not be read or saved/); assert.equal(field(host).props.disabled, true);
  let confirmed = 0; globalThis.window.confirm = () => { confirmed++; return false; };
  walk(host.tree, node => node.type === 'button' && textContent(node) === 'Clear pending submission')[0].props.onClick(); host.render();
  assert.equal(confirmed, 1); assert.equal(field(host).props.disabled, true);
});

test('pending submissions are scoped to the exact comment thread', async context => {
  const { calls, mount } = setup(context, () => Promise.reject(new TypeError('unknown')));
  const message = mount(); await settle(message); draft(message); submit(message); await settle(message);
  const article = mount({ articleId: 22 }); await settle(article);
  assert.equal(field(article).props.value, ''); assert.equal(field(article).props.disabled, false);
  assert.notEqual(PublicComments({ articleId: 22 }).key, PublicComments({ articleId: 23 }).key);
  assert.equal(calls.length, 1);
});

for (const invalid of [{ items: 'not-an-array', total: 1, page: 1, size: 10 }, { items: [{ ...comment({ content: 'example' }), comment_content: null }], total: 1, page: 1, size: 10 }]) {
  test(`original comment-list effect rejects malformed success before render: ${JSON.stringify(invalid)}`, async context => {
    const { mount } = setup(context, body => json(comment(body)), () => json(invalid));
    const host = mount(); await settle(host);
    assert.match(text(host), /server response was invalid/);
    assert.equal(walk(host.tree, node => node.props.className === 'comment-floor').length, 0);
    assert.doesNotMatch(text(host), /No comments yet/);
  });
}

test('original reply effect rejects malformed success before updating reply rows', async context => {
  const root = { ...comment({ content: 'Root' }), reply_count: 1 };
  const { mount } = setup(context, body => json(comment(body)), url => json(url.includes('/replies') ? { items: [null], total: 1, page: 1, size: 5 } : { items: [root], total: 1, page: 1, size: 10 }));
  const host = mount(); await settle(host);
  const element = walk(host.tree, node => typeof node.type === 'function' && node.type.name === 'CommentReplies')[0];
  const replies = new HookHost(element.type, element.props); context.after(() => replies.unmount());
  replies.render(); await settle(replies);
  assert.match(text(replies), /server response was invalid/);
  assert.equal(walk(replies.tree, node => typeof node.type === 'function' && node.type.name === 'CommentEntry').length, 0);
});

test('U+0085 edges are normalized like the server and a committed comment is confirmed', async context => {
  const { calls, stored, mount } = setup(context, () => json(comment({ content: 'hello' })));
  const host = mount(); await settle(host); draft(host, '\u0085hello\u0085'); submit(host); await settle(host);
  assert.equal(calls[0].body.content, 'hello');
  assert.match(text(host), /has been posted/);
  assert.equal(stored.size, 0);
  assert.equal(field(host).props.value, '');
});

test('U+FEFF edges are preserved like the server rather than stripped by JavaScript trim', async context => {
  const content = '\uFEFFhello\uFEFF';
  const { calls, stored, mount } = setup(context, () => json(comment({ content })));
  const host = mount(); await settle(host); draft(host, content); submit(host); await settle(host);
  assert.equal(calls[0].body.content, content);
  assert.match(text(host), /has been posted/);
  assert.equal(stored.size, 0);
});

test('confirmation compares server-normalized content without changing a persisted raw request', async context => {
  const { calls, stored, mount } = setup(context, () => json(comment({ content: 'hello' })));
  const submission = { request_id: '12345678-1234-4123-8123-123456789abc', content: '\u0085hello\u0085', parent_comment_id: null };
  stored.set('xocs.comment-submission:/api/v1/message-comments:0', JSON.stringify(submission));
  const host = mount(); await settle(host); submit(host); await settle(host);
  assert.deepEqual(calls[0].body, submission, 'retry must preserve the original identity and payload');
  assert.match(text(host), /has been posted/);
  assert.equal(stored.size, 0);
});


test('comment normalization matches the shared Unicode whitespace boundary fixtures', async () => {
  const fixtures = JSON.parse(await readFile(new URL('../../tests/fixtures/comment-whitespace.json', import.meta.url), 'utf8'));
  for (const fixture of fixtures) assert.equal(normalizeCommentContent(fixture.input), fixture.expected, fixture.name);
});

test('Unicode whitespace-only drafts cannot produce a request', async context => {
  const { calls, mount } = setup(context, body => json(comment(body)));
  const host = mount(); await settle(host); draft(host, '\u0085\u2007\u202F');
  assert.equal(walk(host.tree, node => node.type === 'button' && node.props.type === 'submit')[0].props.disabled, true);
  submit(host); await settle(host);
  assert.equal(calls.length, 0);
});
