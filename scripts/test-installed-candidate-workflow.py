#!/usr/bin/env python3
"""Source gates for real candidate packaging, separate from installed tests."""
import importlib.util
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
CI = (ROOT / '.github/workflows/ci.yml').read_text()
SOURCE = CI.split('\n  installed_candidate:\n', 1)[0]
INSTALLED = CI.split('\n  installed_candidate:\n', 1)[1]


class CandidateWorkflow(unittest.TestCase):
    def test_added_lane_is_parallel_and_preserves_existing_checks(self):
        self.assertNotIn('    needs:', INSTALLED)
        for command in ('cargo fmt --all -- --check',
                        'cargo clippy --locked --all-targets -- -D warnings',
                        'cargo test --locked', 'cargo build --locked',
                        'npx playwright install --with-deps chromium firefox',
                        'bash scripts/smoke.sh', 'npm run test:unit',
                        'python3 -I -B scripts/test-installed-lifecycle-adapter.py'):
            self.assertIn(command, SOURCE)
        self.assertIn('needs: [server-and-smoke, browser, installed_candidate]', SOURCE)
        self.assertIn('"$INSTALLED_RESULT" == success', SOURCE)

    def test_one_real_release_build_feeds_pinned_producer_and_shared_lifecycle(self):
        self.assertEqual(INSTALLED.count('tools/web-build/Cargo.toml -- server --product-root . --config'), 1)
        self.assertIn('--mode release --source-revision "$GITHUB_SHA"', INSTALLED)
        self.assertIn('XSSI_BUILD_METADATA_DIR: ${{ runner.temp }}/install-metadata', INSTALLED)
        self.assertIn('XSSI_FRONTEND_METADATA_FILE: ${{ runner.temp }}/install-metadata/frontend.json', INSTALLED)
        self.assertIn('scripts/package-installed-candidate.sh', INSTALLED)
        self.assertIn('"$CARGO_TARGET_DIR/$SERVER_TARGET/release/xocs"', INSTALLED)
        self.assertIn('git -C "$installer" fetch --quiet --depth=1 origin "$revision"', INSTALLED)
        self.assertIn('python3 -I -B "$installer/tests/installed_lifecycle.py"', INSTALLED)
        self.assertIn('--adapter "$GITHUB_WORKSPACE/scripts/installed-lifecycle-adapter.py" "${candidates[0]}" "$GITHUB_SHA"', INSTALLED)
        self.assertIn('sudo env GITHUB_ACTIONS="$GITHUB_ACTIONS" RUNNER_ENVIRONMENT="$RUNNER_ENVIRONMENT"', INSTALLED)
        self.assertNotIn('package-server-release.py', INSTALLED)
        self.assertNotIn('git tag', INSTALLED)

    def test_precise_branch_scoped_caches_do_not_replace_real_builds_or_audits(self):
        for lane, cache in ((INSTALLED, 'release_cache'),):
            self.assertIn('rust-1.99.0', lane)
            self.assertIn("'Cargo.lock'", lane)
            self.assertIn("'tools/web-build/Cargo.lock'", lane)
            self.assertIn("'web/rust-web.toml'", lane)
            self.assertIn("'config/xssi.lock'", lane)
            self.assertIn('fingerprint', lane)
            self.assertIn('cargo clean --package xocs', lane)
            self.assertNotIn('restore-keys:', lane)
            self.assertIn("github.event_name == 'push' && github.ref == 'refs/heads/main'", lane)
            self.assertIn('steps.' + cache + '.outputs.cache-primary-key', lane)
        for version in ('cargo-audit --version 0.22.2', 'cargo-cyclonedx --version 0.5.9', 'node-version: 26.7.0'):
            self.assertIn(version, INSTALLED)
        self.assertIn("cargo audit --version | grep -Fx 'cargo-audit-audit 0.22.2'", INSTALLED)
        self.assertIn("cargo cyclonedx --version | grep -Fx 'cargo-cyclonedx-cyclonedx 0.5.9'", INSTALLED)
        self.assertIn('sha256sum --check SHA256SUMS', INSTALLED)
        self.assertIn('if: ${{ always() }}', INSTALLED)
        self.assertIn('path: ${{ runner.temp }}/install-diagnostics', INSTALLED)

    def test_formal_release_still_requires_current_annotated_tag(self):
        text = (ROOT / 'scripts/package-linux-x86_64.sh').read_text()
        self.assertIn('cat-file -t "refs/tags/v$version")" = tag', text)
        self.assertIn('rev-parse "refs/tags/v$version^{commit}")" = "$source_revision"', text)
        self.assertIn('A source-bound annotated release tag is required.', text)
        self.assertIn('"$source_root/scripts/package-release.py" "$server_binary"', text)
        self.assertNotIn("<<'RECEIPT'", text)

    def test_actions_are_exact_and_no_fixtures_replace_installation(self):
        import re
        pins = {'checkout': '3d3c42e5aac5ba805825da76410c181273ba90b1',
                'setup-node': '820762786026740c76f36085b0efc47a31fe5020',
                'cache/restore': '0400d5f644dc74513175e3cd8d07132dd4860809',
                'cache/save': '0400d5f644dc74513175e3cd8d07132dd4860809',
                'upload-artifact': '043fb46d1a93c77aae656e7c1c64a875d1fc6a0a'}
        actions = re.findall(r'uses: actions/([^@\s]+)@([^\s]+)', INSTALLED)
        self.assertTrue(actions)
        for name, revision in actions:
            self.assertEqual(revision, pins[name])
        for forbidden in ('restore-keys:', 'continue-on-error:', 'smoke-fixture',
                          '--mode development', 'cargo test', 'persist-credentials: true'):
            self.assertNotIn(forbidden, INSTALLED)
        self.assertIn('cargo clean --package xocs --release --target "$SERVER_TARGET"', INSTALLED)

    def test_caller_rejects_wrong_or_dirty_installer_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'source'
            (source / 'scripts').mkdir(parents=True)
            (source / 'config').mkdir()
            caller = source / 'scripts/package-installed-candidate.sh'
            shutil.copyfile(ROOT / 'scripts/package-installed-candidate.sh', caller)
            installer = root / 'installer'
            (installer / 'scripts').mkdir(parents=True)
            marker = root / 'producer-was-executed'
            producer = installer / 'scripts/package-installed-candidate.py'
            producer.write_text('from pathlib import Path\nPath(' + repr(str(marker)) + ').touch()\n')
            subprocess.run(['git', 'init', '--quiet', str(installer)], check=True)
            subprocess.run(['git', '-C', str(installer), 'add', '.'], check=True)
            subprocess.run(['git', '-C', str(installer), '-c', 'user.name=Fixture', '-c',
                            'user.email=fixture@example.invalid', 'commit', '--quiet', '-m', 'Owned source gate fixture'], check=True)
            revision = subprocess.check_output(['git', '-C', str(installer), 'rev-parse', 'HEAD'], text=True).strip()
            lock = source / 'config/xssi.lock'
            args = ['bash', str(caller), str(root / 'binary'), str(root / 'metadata'), str(installer), str(root / 'output')]
            lock.write_text('git_rev=' + 'a' * 40 + '\n')
            self.assertNotEqual(subprocess.run(args, capture_output=True).returncode, 0)
            self.assertFalse(marker.exists())
            lock.write_text('git_rev=' + revision + '\n')
            producer.write_text(producer.read_text() + '# unreviewed modification\n')
            self.assertNotEqual(subprocess.run(args, capture_output=True).returncode, 0)
            self.assertFalse(marker.exists())


if __name__ == '__main__':
    unittest.main()
