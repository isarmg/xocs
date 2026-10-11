import assert from 'node:assert/strict';
import { test } from 'node:test';
import { HookHost, expand, textContent, walk } from './fixtures/hook-host.mjs';
import { originalModule } from './fixtures/original-module.mjs';
import { summary } from './browser-fixtures.mjs';

const reexport = path => `export * from ${JSON.stringify(new URL(path, import.meta.url).href)};`;
const boundaries = {
  './api': reexport('../src/api.ts'),
  './public-contracts': reexport('../src/public-contracts.ts'),
  './SlidingTitle': 'export const SlidingTitle=({text})=>text;',
  './ReferencePetals': 'export const ReferencePetals=()=>null;',
  './PublicChrome': 'export const PublicChrome=({children})=>children;',
  './PublicComments': 'export const PublicComments=()=>null;',
  './MediaPreview': 'export const ImageLightbox=()=>null; export const safeImageUrl=value=>value;',
  './PhotoGallery': 'export const PhotoGrid=({items})=>items.map(item=>item.title);',
  './LovePage': 'export const LovePage=()=>null;',
};
const { PublicExtras } = await originalModule(new URL('../src/PublicExtras.tsx', import.meta.url), boundaries);
const { ArticleNews } = await originalModule(new URL('../src/ArticleNews.tsx', import.meta.url), boundaries);
const page = items => ({ items, total: items.length, page: 1, size: 10 });
const settle = async host => {
  await new Promise(resolve => setImmediate(resolve));
  return host.render();
};
const text = host => textContent(expand(host.tree)).replace(/\s+/g, ' ');
function setup(context, component, props, path, fetch) {
  const originalFetch = globalThis.fetch, originalWindow = globalThis.window;
  globalThis.window = { location: { pathname: path, search: '?q=story', origin: 'https://example.test' }, history: { replaceState() {} } };
  globalThis.fetch = fetch;
  const element = component ? { type: component, props } : PublicExtras();
  const host = new HookHost(element.type, element.props);
  context.after(() => { host.unmount(); globalThis.fetch = originalFetch; globalThis.window = originalWindow; });
  host.render();
  return host;
}
const json = body => new Response(JSON.stringify(body), { status: 200 });

for (const [path, valid, expectedText, invalid, failureText] of [
  ['/menory', page([{ ...summary, article_title: 'A valid journey' }]), 'A valid journey', { items: 'not-an-array', total: 1 }, 'Unable to load travels'],
  ['/search', page([{ ...summary, article_title: 'A valid story' }]), 'A valid story', page([{ ...summary, article_title: null }]), 'Search failed'],
  ['/jotting', page([{ id: 1, user_id: 1, username: 'Writer', content: 'A valid moment', image_path: null, like_count: 0, is_public: 1, create_time: null }]), 'A valid moment', { ...page([]), total: '7' }, 'server response was invalid'],
  ['/message', [{ id: 1, user_id: null, username: null, avatar: null, message: 'A valid message', image_path: null, create_time: null }], 'A valid message', [{ id: 1, message: { hidden: 'not text' } }], 'Unable to load messages'],
]) {
  test(`${path} original component accepts a normal response through the real transport`, async context => {
    const host = setup(context, null, {}, path, async () => json(valid));
    await settle(host);
    assert.match(text(host), new RegExp(expectedText));
    assert.equal(walk(host.tree, node => node.props?.role === 'alert').length, 0);
  });
  test(`${path} original component rejects malformed successful data before rendering consumers`, async context => {
    const host = setup(context, null, {}, path, async () => json(invalid));
    await settle(host);
    assert.match(text(host), new RegExp(failureText));
    assert.doesNotMatch(text(host), new RegExp(expectedText));
  });
}

test('ArticleNews validates each returned update in its original effect callsite', async context => {
  let value = [{ id: 1, content: 'A valid update', create_time: null }];
  const host = setup(context, ArticleNews, { articleId: 1 }, '/', async () => json(value));
  await settle(host);
  assert.match(text(host), /A valid update/);
  value = [{ id: 2, content: { raw: 'not text' }, create_time: null }];
  host.props = { articleId: 2 };
  host.render();
  await settle(host);
  assert.match(text(host), /server response was invalid/);
  assert.doesNotMatch(text(host), /not text/);
});

test('message publishing keeps the draft and does not insert a malformed success into the wall', async context => {
  let writes = 0;
  const host = setup(context, null, {}, '/message', async (_url, options) => {
    if (options.method === 'POST') { writes++; return json({ id: 2 }); }
    return json([]);
  });
  await settle(host);
  walk(host.tree, node => node.type === 'input')[0].props.onChange({ target: { value: 'Keep this draft' } });
  host.render();
  walk(host.tree, node => node.type === 'form')[0].props.onSubmit({ preventDefault() {} });
  await settle(host);
  assert.equal(writes, 1);
  assert.equal(walk(host.tree, node => node.type === 'input')[0].props.value, 'Keep this draft');
  assert.match(text(host), /operation result could not be confirmed/);
  assert.equal(walk(host.tree, node => node.props?.className === 'message-barrage-item').length, 0);
});
