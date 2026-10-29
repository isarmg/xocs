import assert from 'node:assert/strict';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';

const web = new URL('../../src/web/', import.meta.url);
function fixture(run) {
  const root = mkdtempSync(join(tmpdir(), 'blog-toolchain-')), directory = join(root, 'src/web');
  try {
    const sourceRoot = new URL('../../', web);
    const copy = relative => {
      const destination = join(root, relative);
      mkdirSync(dirname(destination), { recursive: true });
      // Fixtures own regular copies, including when the test checkout itself
      // is reached through symlinks. Mutation tests never write through links.
      cpSync(new URL(relative, sourceRoot), destination, { recursive: true, dereference: true });
    };
    for (const file of ['.node-version', 'src/web/package.json', 'src/web/package-lock.json', 'src/web/index.html',
      'src/web/rust-web.toml', 'src/web/scripts/check-toolchain.mjs', 'src/browser/Cargo.toml',
      'src/web/runtime/startup.js', 'src/web/runtime/startup.css', 'src/web/styles/admin-ui/styles.css',
      'src/web/licenses/LICENSE-blog-dom-MIT.txt', 'src/web/licenses/LICENSE-blog-component-macros-MIT.txt']) copy(file);
    for (const component of ['src/web/styles', 'src/interface', 'src/interface/dom', 'src/interface/macros', 'src/tools/assets']) {
      copy(`${component}/ORIGIN.json`); copy(`${component}/LICENSE`);
      if (component.startsWith('src/interface')) copy(`${component}/Cargo.toml`);
      const origin = JSON.parse(readFileSync(join(root, component, 'ORIGIN.json'), 'utf8'));
      copy(origin.upstream.receipt); copy(origin.upstream.archive);
    }
    const entry = readFileSync(join(directory, 'rust-web.toml'), 'utf8').match(/^entry = "([^"]+)"/m)?.[1];
    assert.ok(entry && !entry.startsWith('/') && !entry.split('/').includes('..'));
    copy(`src/web/${entry}`);
    mkdirSync(join(root, 'test/browser'), { recursive: true });
    const check = () => spawnSync(process.execPath, [join(directory, 'scripts/check-toolchain.mjs')], { encoding: 'utf8' });
    const json = (file, change) => {
      const path = join(directory, file), value = JSON.parse(readFileSync(path, 'utf8'));
      change(value); writeFileSync(path, JSON.stringify(value));
    };
    run({ directory, check, json });
  } finally { rmSync(root, { recursive: true, force: true }); }
}
test('local implementation and original archives pass without node_modules', () => fixture(({ check }) => {
  const result = check(); assert.equal(result.status, 0, result.stderr);
}));
test('retired npm package is rejected even with a consistent lockfile', () => fixture(({ check, json }) => {
  json('package.json', data => { data.devDependencies['@xcss/web'] = '1.0.0'; });
  json('package-lock.json', data => { data.packages[''].devDependencies['@xcss/web'] = '1.0.0'; });
  const result = check(); assert.notEqual(result.status, 0); assert.match(result.stderr, /@xcss\/web/);
}));
test('old npm asset paths fail the production contract', () => fixture(({ directory, check }) => {
  const path = join(directory, 'index.html');
  writeFileSync(path, readFileSync(path, 'utf8').replace('</head>', '<link rel="preload" href="/node_modules/@xcss/web/dist/font.woff2"></head>'));
  const result = check(); assert.notEqual(result.status, 0); assert.match(result.stderr, /font preloads/);
}));
test('browser tests cannot silently import a second npm asset version', () => fixture(({ directory, check }) => {
  writeFileSync(join(directory, '../../test/browser/stale.mjs'), "import { geometry } from '@xcss/web/admin-ui/server-corners';");
  const result = check(); assert.notEqual(result.status, 0); assert.match(result.stderr, /test local maintained assets/);
}));
test('maintained source can evolve while historical evidence stays unchanged', () => fixture(({ directory, check }) => {
  writeFileSync(join(directory, 'styles/local-change.css'), '/* maintained locally */');
  const result = check(); assert.equal(result.status, 0, result.stderr);
}));
test('historical source receipt tampering is rejected', () => fixture(({ directory, check, json }) => {
  json('styles/ORIGIN.json', origin => { origin.upstream.receipt_sha256 = '0'.repeat(64); });
  const result = check(); assert.notEqual(result.status, 0); assert.match(result.stderr, /original receipt digest/);
}));

function changeAppDeclaration(directory, declaration) {
  const app = join(directory, '../browser');
  const manifest = readFileSync(join(app, 'Cargo.toml'), 'utf8');
  writeFileSync(join(app, 'Cargo.toml'), manifest.replace(/^blog-interface\s*=.*$/m, declaration));
}
test('local UI declaration accepts reordered fields', () => fixture(({ directory, check }) => {
  for (const declaration of [
    'blog-interface = { features = ["csr"], path = "../interface", default-features = false }',
    'blog-interface = { path = "../interface", default-features = false, features = ["csr",] }',
  ]) {
    changeAppDeclaration(directory, declaration);
    const result = check(); assert.equal(result.status, 0, result.stderr);
  }
}));
for (const [name, declaration, expected] of [
  ['external UI path', 'blog-interface = { features = ["csr"], path = "../other-ui", default-features = false }', /Local blog-interface dependency path/],
  ['server UI feature', 'blog-interface = { path = "../interface", default-features = false, features = ["csr", "ssr"] }', /supported blog-interface CSR feature/],
  ['default UI features', 'blog-interface = { path = "../interface", default-features = true, features = ["csr"] }', /Disable default blog-interface features/],
  ['registry UI declaration', 'blog-interface = { version = "1", default-features = false, features = ["csr"] }', /one path field/],
  ['duplicate UI path', 'blog-interface = { path = "../interface", default-features = false, features = ["csr"], path = "../other-ui" }', /one path field/],
]) {
  test(`${name} fails the local CSR contract`, () => fixture(({ directory, check }) => {
    changeAppDeclaration(directory, declaration);
    const result = check(); assert.notEqual(result.status, 0); assert.match(result.stderr, expected);
  }));
}

test('standalone license copies preserve original copyright bytes', () => fixture(({ directory, check }) => {
  writeFileSync(join(directory, 'licenses/LICENSE-blog-dom-MIT.txt'), 'Changed copyright');
  const result = check(); assert.notEqual(result.status, 0); assert.match(result.stderr, /standalone assets preserve the original license/);
}));
