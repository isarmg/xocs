import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { transformWithOxc } from 'vite';
import { walk } from './fixtures/hook-host.mjs';

// Render the original HomeAside declaration, stubbing only unrelated boundaries.
const main = await readFile(new URL('../src/main.tsx', import.meta.url), 'utf8');
const source = `import { jsx } from '${new URL('./fixtures/hook-host.mjs', import.meta.url).href}';
const h=(type,props,...children)=>jsx(type,{...props,children});
const t=(_zh,en)=>en;const safeMediaUrl=value=>value;\n`
  + main.slice(main.indexOf('function HomeAside('), main.indexOf('function Site()')) + '\nexport { HomeAside };';
const { code } = await transformWithOxc(source, 'home-aside.tsx', { jsx: { runtime: 'classic', pragma: 'h' } });
const { HomeAside } = await import(`data:text/javascript,${encodeURIComponent(code)}`);
for (const lang of ['en', 'zh-CN', null]) test(`home search carries explicit language ${lang} in the native GET form`, context => {
  const original = globalThis.window;
  globalThis.window = { location: new URL(`https://example.test/${lang ? `?lang=${lang}` : ''}`) };
  context.after(() => { globalThis.window = original; });
  const tree = HomeAside({ site: null, categories: [], recommendedArticles: [], labels: [], messages: [], articleTotal: 0, viewTotal: 0 });
  const form = walk(tree, node => node.type === 'form')[0];
  assert.equal(form.props.method, 'get');
  const inputs = walk(form, node => node.type === 'input');
  const submitted = new URLSearchParams(inputs.filter(node => node.props.name).map(node => [node.props.name, node.props.name === 'q' ? 'a story' : node.props.value]));
  assert.equal(submitted.get('lang'), lang);
  assert.equal(submitted.get('q'), 'a story');
  assert.equal(inputs.filter(node => node.props.name === 'lang').every(node => node.props.type === 'hidden'), true);
});
