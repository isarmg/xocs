import assert from 'node:assert/strict';
import {chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import test from 'node:test';

const workflow = readFileSync(new URL('../.github/workflows/ci.yml', import.meta.url), 'utf8');
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
    'cargo fmt --all -- --check', 'cargo clippy --locked --all-targets -- -D warnings',
    'cargo test --locked', 'cargo build --locked', 'bash scripts/smoke.sh',
    'verify-manifest --product-root', 'verify-source --product-root',
    'fmt --manifest-path web/leptos/Cargo.toml -- --check',
    'clippy --manifest-path web/leptos/Cargo.toml --target wasm32-unknown-unknown --locked -- -D warnings',
    'python3 scripts/build-leptos.py test', 'python3 -I -B scripts/test-leptos-build.py',
  ]) assert.ok(server.includes(command), `missing server check: ${command}`);
  const expected = [
    'language', 'admin-layout', 'account-settings', 'admin-cleanup',
    'management', 'contract-fixes', 'leptos-runtime', 'public-footer', 'public-layout', 'anonymous-comments',
  ];
  const actual = [...browser.matchAll(/node tests\/([a-z-]+)\.mjs/g)].map(match => match[1]);
  assert.deepEqual(actual.sort(), expected.sort());
  assert.equal(browser.match(/npm run test:unit/g)?.length, 1);
  assert.match(browser, /TEST_GROUP: \$\{\{ matrix.group \}\}/);
  for (const block of [server, browser]) {
    assert.match(block, /npm ci\n          npm run build/);
    assert.match(block, /playwright install --with-deps chromium firefox/);
    assert.match(block, /rustup toolchain install 1\.99\.0 .*--target wasm32-unknown-unknown/);
    assert.match(block, /cargo \+1\.99\.0 install wasm-bindgen-cli --version 0\.2\.129 --locked/);
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

test('both matrix branches execute every original UI script exactly once', () => {
  const run = browser.match(/        run: \|\n(          set -euo pipefail\n(?:          .*\n)+)/)?.[1];
  assert.ok(run, 'missing browser matrix shell');
  const directory = mkdtempSync(join(tmpdir(), 'xocs-ci-dispatch-'));
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
      'run test:unit', 'tests/language.mjs', 'tests/admin-layout.mjs',
      'tests/account-settings.mjs', 'tests/admin-cleanup.mjs', 'tests/management.mjs',
      'tests/contract-fixes.mjs', 'tests/leptos-runtime.mjs',
      'tests/public-footer.mjs', 'tests/public-layout.mjs', 'tests/anonymous-comments.mjs',
    ].sort());
    const unknown = spawnSync('bash', ['-c', run], {
      encoding: 'utf8',
      env: {...process.env, PATH: `${directory}:${process.env.PATH}`, TEST_GROUP: 'unknown'},
    });
    assert.equal(unknown.status, 1);
    assert.equal(unknown.stdout, '');
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});
