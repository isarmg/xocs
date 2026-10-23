#!/usr/bin/env python3
"""Local candidate CI keeps source identity, real installation and fail-closed gates."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
CI = (ROOT / '.github/workflows/ci.yml').read_text()
SOURCE = CI.split('\n  installed_candidate:\n', 1)[0]
INSTALLED = CI.split('\n  installed_candidate:\n', 1)[1]


class CandidateWorkflow(unittest.TestCase):
    def test_lane_is_independent_and_preserves_checks(self):
        self.assertNotIn('    needs:', INSTALLED)
        for command in ('cargo fmt -p blog -p blog-application -p blog-setup -p blog-assets -p blog-browser -p blog-browser-domain-tests -p blog-interface -p blog-dom -p blog-component-macros -- --check',
                        'cargo clippy --locked --offline --all-targets -- -D warnings',
                        'cargo test --locked --offline', 'cargo build --locked --offline',
                        'npx playwright install --with-deps chromium firefox',
                        'bash test/operations/smoke.sh', 'npm run test:unit',
                        'python3 -I -B test/operations/test-installed-lifecycle-adapter.py'):
            self.assertIn(command, SOURCE)
        self.assertIn('needs: [server-and-smoke, browser, installed_candidate]', SOURCE)
        self.assertIn('"$INSTALLED_RESULT" == success', SOURCE)
        self.assertNotIn('continue-on-error:', CI)

    def test_one_real_candidate_build_feeds_local_installed_acceptance(self):
        self.assertEqual(INSTALLED.count('-p blog-assets -- server --product-root . --config'), 1)
        self.assertIn('--mode candidate --source-revision "$GITHUB_SHA"', INSTALLED)
        self.assertIn('BLOG_BUILD_METADATA_DIR:', INSTALLED)
        self.assertIn('BLOG_FRONTEND_METADATA_FILE:', INSTALLED)
        self.assertIn('src/operations/package-installed-candidate.sh', INSTALLED)
        self.assertIn('test/operations/check-installed-candidate.sh "${candidates[0]}"', INSTALLED)
        self.assertIn('sudo --preserve-env=BLOG_INSTALL_DIAGNOSTICS', INSTALLED)
        self.assertNotIn('github.com/isarmg/xssi', INSTALLED)
        self.assertNotIn('git tag', INSTALLED)
        for forbidden in ('smoke-fixture', '--mode development', 'continue-on-error:'):
            self.assertNotIn(forbidden, INSTALLED)

    def test_candidate_stager_binds_exact_server_and_current_setup(self):
        stager = (ROOT / 'src/operations/stage-package.py').read_text()
        self.assertIn("'source-stamp'", stager)
        self.assertIn("'source_tree_sha256'", stager)
        self.assertIn("'blog-setup'", stager)
        self.assertIn("'--locked', '--offline', '--release'", stager)
        self.assertIn('stage_verified_server(', stager)
        self.assertIn('source changed while building Blog setup'.lower(), stager.lower())
        self.assertNotIn('xssi', stager)
        caller = (ROOT / 'src/operations/package-installed-candidate.sh').read_text()
        self.assertIn('[[ $# == 3 ]]', caller)
        self.assertNotIn('git fetch', caller)
        self.assertNotIn('config/xssi.lock', caller)

    def test_formal_release_still_requires_current_annotated_tag(self):
        text = (ROOT / 'src/operations/package-linux-x86_64.sh').read_text()
        self.assertIn('cat-file -t "refs/tags/v$version")" = tag', text)
        self.assertIn('rev-parse "refs/tags/v$version^{commit}")" = "$source_revision"', text)
        self.assertIn('A source-bound annotated release tag is required.', text)
        self.assertIn('--kind release', text)
        self.assertNotIn('github.com/isarmg/xssi', text)
        self.assertNotIn('XSSI_', text)

    def test_actual_installed_acceptance_covers_safety_and_business(self):
        text = (ROOT / 'test/operations/check-installed-candidate.py').read_text()
        for assertion in ('Package corruption was accepted', 'Initialization overwrote an existing database',
                          'CSRF token omission was accepted', 'Foreign Origin was accepted',
                          'Offline check bypassed a running writer lock',
                          'Offline credential reset bypassed a running writer lock',
                          'Password reset retained a previous session',
                          'Previous password survived the reset',
                          'Production administrator cookie lacks Secure/HttpOnly',
                          'Runtime environment redirected direct offline initialization',
                          'Runtime environment redirected direct offline check',
                          'Runtime environment redirected direct offline password reset',
                          'Offline operations changed the other instance password'):
            self.assertIn(assertion, text)
        self.assertIn('adapter.exercise(', text)
        self.assertIn('adapter.persisted(', text)
        self.assertIn('ssl.PROTOCOL_TLS_SERVER', text)
        self.assertIn("'isarmg-blog'", text)


if __name__ == '__main__':
    unittest.main()
