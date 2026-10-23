import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';

const web = new URL('../', import.meta.url);
const root = new URL('../../', web);
const tests = new URL('test/browser/', root);
const read = path => readFileSync(new URL(path, web), 'utf8');
const readRoot = path => readFileSync(new URL(path, root), 'utf8');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const manifest = JSON.parse(read('package.json'));
const lock = JSON.parse(read('package-lock.json'));
assert.deepEqual(manifest.dependencies ?? {}, {});
assert.deepEqual(lock.packages[''].devDependencies, manifest.devDependencies);
assert.match(readRoot('.node-version'), /^26\.7\.0\n?$/);
for (const name of ['@xcss/web', 'react', 'react-dom', '@vitejs/plugin-react', 'vite', 'typescript']) {
  assert.equal(manifest.dependencies?.[name] ?? manifest.devDependencies?.[name], undefined, name);
  assert.equal(lock.packages[`node_modules/${name}`], undefined, name);
}

// Original snapshots are evidence. Maintained implementation files deliberately
// have their own current build inventory instead of pretending to be unchanged.
function archiveFiles(compressed) {
  const bytes = gunzipSync(compressed), files = new Map();
  for (let offset = 0; offset + 512 <= bytes.length;) {
    const header = bytes.subarray(offset, offset + 512);
    if (header.every(byte => byte === 0)) break;
    const field = (start, end) => header.subarray(start, end).toString().replace(/\0.*$/s, '');
    const prefix = field(345, 500), name = (prefix ? `${prefix}/` : '') + field(0, 100);
    const size = Number.parseInt(field(124, 136).trim(), 8);
    assert.ok(Number.isSafeInteger(size) && size >= 0 && offset + 512 + size <= bytes.length, name);
    assert.ok(!name.startsWith('/') && !name.split('/').includes('..') && !files.has(name), name);
    assert.ok(header[156] === 0 || header[156] === 48, `${name}: original snapshot contains regular files`);
    files.set(name, bytes.subarray(offset + 512, offset + 512 + size));
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  return files;
}
for (const directory of ['src/web/styles', 'src/interface', 'src/interface/dom', 'src/interface/macros', 'src/tools/assets']) {
  const origin = JSON.parse(readRoot(`${directory}/ORIGIN.json`));
  assert.equal(origin.format, 1);
  assert.equal(origin.ownership, 'repository-maintained');
  assert.ok(origin.changes.length > 0, `${directory}: local modifications recorded`);
  const upstream = origin.upstream;
  const originalBytes = readFileSync(new URL(upstream.receipt, root));
  const compressed = readFileSync(new URL(upstream.archive, root));
  assert.equal(digest(originalBytes), upstream.receipt_sha256, `${directory}: original receipt digest`);
  assert.equal(digest(compressed), upstream.archive_sha256, `${directory}: original archive digest`);
  const original = JSON.parse(originalBytes), files = archiveFiles(compressed);
  assert.equal(original.package, upstream.package);
  assert.equal(original.version, upstream.version);
  assert.deepEqual(files.get('SOURCE.json'), originalBytes, `${directory}: original receipt is archived verbatim`);
  assert.deepEqual([...files.keys()].sort(), [...Object.keys(original.files), 'SOURCE.json'].sort());
  for (const [name, expected] of Object.entries(original.files)) {
    assert.equal(digest(files.get(name)), expected, `${directory}: original ${name} digest`);
  }
  assert.equal(digest(readFileSync(new URL(`${directory}/LICENSE`, root))), original.files.LICENSE,
    `${directory}: preserve original license and copyright`);
}
// Check the supported dependency declaration independently of TOML field order.
// Cargo remains the TOML parser; this Node-only guard accepts the single-line
// inline table used by the application and rejects ambiguous duplicate fields.
const appDependencies = readRoot('src/browser/Cargo.toml').match(/^\[dependencies\][ \t]*(?:#.*)?\r?\n([\s\S]*?)(?=^\[|$(?![\s\S]))/m)?.[1];
const localUi = appDependencies?.match(/^blog-interface[ \t]*=[ \t]*\{([^{}\r\n]*)\}[ \t]*(?:#.*)?$/m)?.[1];
assert.ok(localUi, 'Declare blog-interface as an explicit local dependency');
for (const field of ['path', 'default-features', 'features']) {
  const matches = [...localUi.matchAll(new RegExp(`(?:^|,)\\s*${field}\\s*=`, 'g'))];
  assert.equal(matches.length, 1, `blog-interface must declare one ${field} field`);
}
assert.match(localUi, /(?:^|,)\s*path\s*=\s*"\.\.\/interface"\s*(?=,|$)/, 'Local blog-interface dependency path');
assert.match(localUi, /(?:^|,)\s*default-features\s*=\s*false\s*(?=,|$)/, 'Disable default blog-interface features');
assert.match(localUi, /(?:^|,)\s*features\s*=\s*\[\s*"csr"\s*,?\s*\]\s*(?=,|$)/, 'Select only the supported blog-interface CSR feature');
assert.doesNotMatch(localUi, /(?:^|,)\s*(?:git|registry|registry-index|workspace)\s*=/, 'blog-interface must resolve directly to the maintained local source');
assert.match(readRoot('src/interface/Cargo.toml'), /name\s*=\s*"blog-interface"/);
for (const [directory, name] of [['dom', 'blog-dom'], ['macros', 'blog-component-macros']]) {
  assert.match(readRoot(`src/interface/${directory}/Cargo.toml`), new RegExp(`name\\s*=\\s*"${name}"`));
  assert.deepEqual(readFileSync(new URL(`licenses/LICENSE-${name}-MIT.txt`, web)),
    readFileSync(new URL(`src/interface/${directory}/LICENSE`, root)),
    `${name}: standalone assets preserve the original license and copyright`);
}
assert.match(read('rust-web.toml'), /"@blog\/styles\/"\s*=\s*"styles\/"/);
assert.match(read('styles/admin-ui/styles.css'), /@import ["']\.\/content-blocks\.css["']/);
const entryPath = read('rust-web.toml').match(/^entry = "([^"]+)"/m)?.[1];
assert.ok(entryPath);
assert.ok(read('index.html').includes(`src="/${entryPath}"`));
assert.doesNotMatch(read('index.html'), /node_modules|vendor\/xcss/, 'font preloads use local maintained assets');
assert.match(read(entryPath), /blog_browser_bg\.wasm\?url/);
assert.match(read(entryPath), /startApplication/);
assert.match(read(entryPath), /["']\.\.\/runtime\/startup\.js["']/);
assert.match(read(entryPath), /["']\.\.\/runtime\/startup\.css["']/);
assert.match(read('runtime/startup.js'), /export function startApplication/);
assert.equal(existsSync(new URL('xcss-web', web)), false);
assert.equal(existsSync(new URL('vendor/xcss', web)), false);
for (const file of readdirSync(tests).filter(file => file.endsWith('.mjs'))) {
  assert.doesNotMatch(readFileSync(new URL(file, tests), 'utf8'), /^(?:import|export)\s.*?["']@xcss\/web|import\s*\(\s*["']@xcss\/web/m,
    `${file}: test local maintained assets`);
}
console.log('Local Rust/WASM implementation, original archives and preserved licenses passed');
