import assert from 'node:assert/strict';
import {chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import test from 'node:test';

const workflow = readFileSync(new URL('../../.github/workflows/ci.yml', import.meta.url), 'utf8');
// Read only the fixed job/step blocks under test. YAML and Actions validation
// remain separate; this test executes the exact aggregate shell from the file.
function job(id) {
  const match = workflow.match(new RegExp(`^  ${id}:\\n([\\s\\S]*?)(?=^  [a-z][a-z_-]*:|$(?![\\s\\S]))`, 'm'));
  assert.ok(match, `missing CI job: ${id}`);
  return match[1];
}
const server = job('server-and-smoke');
const browser = job('browser');
const aggregate = job('build-and-smoke');

test('server and browser groups are independent and preserve every gate', () => {
  assert.doesNotMatch(server, /^    needs:/m);
  assert.doesNotMatch(browser, /^    needs:/m);
  assert.doesNotMatch(workflow, /continue-on-error:/);
  assert.match(browser, /fail-fast: false/);
  assert.match(browser, /group: \[admin, public\]/);
  for (const command of [
    'cargo fmt -p blog -p blog-application -p blog-setup -p blog-assets -p blog-browser -p blog-browser-domain-tests -p blog-interface -p blog-dom -p blog-component-macros -- --check',
    'cargo clippy --locked --offline --all-targets -- -D warnings',
    'cargo clippy --locked --offline -p blog-assets -p blog-browser-domain-tests --all-targets -- -D warnings',
    'cargo clippy --locked --offline -p blog-browser --target wasm32-unknown-unknown -- -D warnings',
    'cargo test --locked --offline', 'cargo build --locked --offline', 'bash test/operations/smoke.sh',
    'python3 src/tools/check-independence.py',
    'python3 src/tools/rust-sources/manage.py verify --metadata',
    'python3 -I -B test/tools/rust-sources/test_audit.py',
    'cargo run --quiet --locked --offline -p blog-assets -- test --product-root . --offline',
    'cargo test --locked --offline -p blog-assets -p blog-browser-domain-tests',
    'bash test/run-maintained.sh',
  ]) assert.ok(server.includes(command), `missing server check: ${command}`);
  const expected = [
    'language', 'admin-layout', 'configured-media', 'article-video', 'account-settings', 'admin-cleanup',
    'management', 'contract-fixes', 'admin-recovery', 'blog-interface-runtime', 'async-state', 'transport-lifecycle', 'startup-failure', 'rust-native-domain', 'public-footer', 'public-layout', 'style-contract', 'public-recovery', 'anonymous-comments',
  ];
  const actual = [...browser.matchAll(/node \.\.\/\.\.\/test\/browser\/([a-z-]+)\.mjs/g)].map(match => match[1]);
  assert.deepEqual(actual.sort(), expected.sort());
  assert.equal(browser.match(/npm run test:unit/g)?.length, 1);
  assert.match(browser, /TEST_GROUP: \$\{\{ matrix.group \}\}/);
  for (const block of [server, browser]) {
    assert.match(block, /Prepare browser test dependencies\n        working-directory: src\/web\n        run: npm ci/);
    assert.match(block, /Build web from repository Rust sources\n        run: cargo run --quiet --locked --offline -p blog-assets -- build --product-root \. --offline/);
    assert.match(block, /playwright install --with-deps chromium firefox/);
    assert.match(block, /rustup toolchain install 1\.99\.0 .*--target wasm32-unknown-unknown/);
    assert.doesNotMatch(block, /cargo.*install wasm-bindgen-cli/);
    assert.doesNotMatch(block, /--workspace.*--all-features/);
  }
});

test('the old required check fails closed for all server and matrix outcomes', () => {
  assert.match(aggregate, /needs: \[server-and-smoke, browser, installed_candidate\]/);
  assert.match(aggregate, /if: \$\{\{ always\(\) \}\}/);
  assert.match(aggregate, /SERVER_RESULT: \$\{\{ needs.server-and-smoke.result \}\}/);
  assert.match(aggregate, /BROWSER_RESULT: \$\{\{ needs.browser.result \}\}/);
  assert.match(aggregate, /INSTALLED_RESULT: \$\{\{ needs.installed_candidate.result \}\}/);
  const run = aggregate.match(/        run: \|\n((?:          .*\n)+)/)?.[1];
  assert.ok(run, 'missing aggregate shell');
  for (const serverResult of ['success', 'failure', 'cancelled', 'skipped']) {
    for (const browserResult of ['success', 'failure', 'cancelled', 'skipped']) {
      for (const installedResult of ['success', 'failure', 'cancelled', 'skipped']) {
        const result = spawnSync('bash', ['-c', run], {
          encoding: 'utf8',
          env: {...process.env, SERVER_RESULT: serverResult, BROWSER_RESULT: browserResult, INSTALLED_RESULT: installedResult},
        });
        assert.ifError(result.error);
        assert.equal(result.status === 0, serverResult === 'success' && browserResult === 'success' && installedResult === 'success',
          `${serverResult}/${browserResult}/${installedResult}`);
      }
    }
  }
});

test('both matrix branches execute every UI script exactly once', () => {
  const run = browser.match(/        run: \|\n(          set -euo pipefail\n(?:          .*\n)+)/)?.[1];
  assert.ok(run, 'missing browser matrix shell');
  const directory = mkdtempSync(join(tmpdir(), 'blog-ci-dispatch-'));
  try {
    for (const tool of ['node', 'npm']) {
      const path = join(directory, tool);
      writeFileSync(path, '#!/bin/sh\nprintf "%s\\n" "$*"\n');
      chmodSync(path, 0o755);
    }
    const actual = [];
    for (const group of ['admin', 'public']) {
      const result = spawnSync('bash', ['-c', run], {
        encoding: 'utf8',
        env: {...process.env, PATH: `${directory}:${process.env.PATH}`, TEST_GROUP: group},
      });
      assert.ifError(result.error);
      assert.equal(result.status, 0, result.stderr);
      actual.push(...result.stdout.trim().split('\n'));
    }
    assert.deepEqual(actual.sort(), [
      'run test:unit', '../../test/browser/language.mjs', '../../test/browser/admin-layout.mjs',
      '../../test/browser/configured-media.mjs',
      '../../test/browser/article-video.mjs',
      '../../test/browser/account-settings.mjs', '../../test/browser/admin-cleanup.mjs', '../../test/browser/management.mjs',
      '../../test/browser/contract-fixes.mjs', '../../test/browser/admin-recovery.mjs', '../../test/browser/blog-interface-runtime.mjs', '../../test/browser/async-state.mjs', '../../test/browser/transport-lifecycle.mjs', '../../test/browser/startup-failure.mjs', '../../test/browser/rust-native-domain.mjs',
      '../../test/browser/public-footer.mjs', '../../test/browser/public-layout.mjs', '../../test/browser/style-contract.mjs', '../../test/browser/public-recovery.mjs', '../../test/browser/anonymous-comments.mjs',
    ].sort());
    const unknown = spawnSync('bash', ['-c', run], {
      encoding: 'utf8',
      env: {...process.env, PATH: `${directory}:${process.env.PATH}`, TEST_GROUP: 'unknown'},
    });
    assert.equal(unknown.status, 1);
    assert.equal(unknown.stdout, '');
    // Every new Rust/WASM regression gate must propagate failures.
    for (const script of ['rust-native-domain', 'startup-failure', 'async-state', 'configured-media', 'article-video', 'admin-recovery', 'public-recovery', 'style-contract']) {
      writeFileSync(join(directory, 'node'), `#!/bin/sh\nif [ "$*" = "../../test/browser/${script}.mjs" ]; then exit 9; fi\nprintf "%s\\n" "$*"\n`);
      const failed = spawnSync('bash', ['-c', run], {
        encoding: 'utf8',
        env: {...process.env, PATH: `${directory}:${process.env.PATH}`, TEST_GROUP: script === 'public-recovery' ? 'public' : 'admin'},
      });
      assert.ifError(failed.error);
      assert.equal(failed.status, 9, script);
    }
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

// A release publishes the build itself, so it must retain the same UI failure
// gates as branch CI, including the startup failure/timeout tests.
test('release executes every browser CI gate exactly once', () => {
  const release = readFileSync(new URL('../../.github/workflows/release.yml', import.meta.url), 'utf8');
  const scripts = text => [...text.matchAll(/node \.\.\/\.\.\/test\/browser\/([a-z-]+)\.mjs/g)]
    .map(match => match[1]).sort();
  assert.deepEqual(scripts(release), scripts(browser));
  assert.equal(release.match(/npm run test:unit/g)?.length, 1);
});
