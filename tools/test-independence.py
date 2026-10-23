#!/usr/bin/env python3
"""Exercise actual Cargo metadata against source paths that escape a checkout."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('blog_independence', Path(__file__).with_name('check-independence.py'))
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


class SourceBoundary(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='blog-source-boundary-')
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.root = self.base / 'checkout'
        (self.root / 'src').mkdir(parents=True)
        (self.root / 'src/lib.rs').write_text('pub fn value() -> u8 { 1 }\n')
        (self.base / 'outside.rs').write_text('fn main() {}\n')
        source = Path(__file__).resolve().parent / 'assets/src/platform_contract.rs'
        shutil.copyfile(source, self.root / 'platform_contract.rs')
        (self.root / 'build.rs').write_text(
            '#[path = "platform_contract.rs"]\nmod platform_contract;\n'
            'fn main() { platform_contract::check(); }\n')
        policy = self.root / guard.source_manager.POLICY
        policy.parent.mkdir(parents=True)
        policy.write_text(json.dumps({'platform_guard': 'platform_contract.rs',
                                      'owned_sources': [{'directory': '.'}]}))
        self.previous_root = guard.ROOT
        guard.ROOT = self.root
        self.addCleanup(setattr, guard, 'ROOT', self.previous_root)

    def manifest(self, *, package='', tail=''):
        (self.root / 'Cargo.toml').write_text(
            '[package]\nname="blog-boundary-fixture"\nversion="0.1.0"\nedition="2024"\n'
            + package + '\n[workspace]\n' + tail)

    def graph(self):
        subprocess.run(['cargo', '+1.99.0', 'generate-lockfile', '--offline', '--manifest-path',
                        str(self.root / 'Cargo.toml')], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        return guard.check_graph('Cargo.toml', 'x86_64-unknown-linux-gnu')

    def test_current_source_policy_is_required(self):
        self.manifest()
        (self.root / guard.source_manager.POLICY).unlink()
        with self.assertRaisesRegex(RuntimeError, 'current source policy is required'):
            guard.check_declarations()

    def test_local_sources_are_accepted(self):
        self.manifest()
        guard.check_declarations()
        self.assertEqual(self.graph()['packages'], 1)

    def test_nested_purpose_package_with_default_internal_version(self):
        directory = self.root / 'src/foundation/network/message'
        (directory / 'src').mkdir(parents=True)
        (directory / 'Cargo.toml').write_text('[package]\nname="blog-message"\npublish=false\n')
        (directory / 'src/lib.rs').write_text('pub fn value() {}\n')
        self.manifest(tail='[dependencies]\nmessage={package="blog-message",path="src/foundation/network/message"}\n')
        guard.check_declarations()
        self.assertEqual(self.graph()['packages'], 2)

    def test_source_target_directory_is_checked_but_package_output_is_ignored(self):
        self.manifest()
        real_source = self.root / 'src/target'
        real_source.mkdir()
        (real_source / 'Cargo.lock').write_text('version=4\n')
        with self.assertRaisesRegex(RuntimeError, 'Only the root Cargo.lock'):
            guard.check_declarations()
        (real_source / 'Cargo.lock').unlink()
        generated = self.root / 'target/debug/build/legacy'
        generated.mkdir(parents=True)
        (generated / 'Cargo.lock').write_text('version=4\n')
        (generated / 'Cargo.toml').write_text('[workspace]\n')
        guard.check_declarations()

    def test_external_library_source_is_rejected(self):
        self.manifest(tail='[lib]\npath="../outside.rs"\n')
        with self.assertRaisesRegex(RuntimeError, 'target.*escapes'):
            self.graph()

    def test_external_build_script_is_rejected(self):
        self.manifest(package='build="../outside.rs"\n')
        with self.assertRaisesRegex(RuntimeError, 'target.*escapes'):
            self.graph()

    def test_external_path_dependency_is_rejected_before_cargo(self):
        self.manifest(tail='[dependencies]\nescaped={path="../outside"}\n')
        with self.assertRaisesRegex(RuntimeError, 'escapes'):
            guard.check_declarations()

    def test_shared_git_dependency_is_rejected_before_fetch(self):
        self.manifest(tail='[dependencies]\nrenamed={git="https://github.com/isarmg/xcss.git"}\n')
        with self.assertRaisesRegex(RuntimeError, 'External Git or registry'):
            guard.check_declarations()

    def test_registry_version_declaration_is_rejected_before_metadata(self):
        self.manifest(tail='[dependencies]\nserde="1"\n')
        with self.assertRaisesRegex(RuntimeError, 'Registry dependency must use a local path'):
            guard.check_declarations()

    def test_any_git_dependency_is_rejected_before_fetch(self):
        self.manifest(tail='[dependencies]\nforeign={git="https://example.invalid/library"}\n')
        with self.assertRaisesRegex(RuntimeError, 'External Git or registry'):
            guard.check_declarations()

    def test_nested_workspace_is_rejected(self):
        self.manifest()
        nested = self.root / 'nested'
        nested.mkdir()
        (nested / 'Cargo.toml').write_text('[workspace]\n')
        with self.assertRaisesRegex(RuntimeError, 'Nested Cargo workspace'):
            guard.check_declarations()

    def test_nested_lockfile_is_rejected(self):
        self.manifest()
        nested = self.root / 'nested'
        nested.mkdir()
        (nested / 'Cargo.lock').write_text('version=4\n')
        with self.assertRaisesRegex(RuntimeError, 'Only the root Cargo.lock'):
            guard.check_declarations()

    def test_registry_metadata_is_rejected_and_cargo_is_always_offline(self):
        self.manifest()
        metadata = {'workspace_root': str(self.root), 'packages': [{
            'name': 'external', 'source': 'registry+https://github.com/rust-lang/crates.io-index',
            'manifest_path': str(self.root / 'Cargo.toml'), 'targets': []}]}
        with patch.object(guard.subprocess, 'run', return_value=subprocess.CompletedProcess(
                [], 0, stdout=json.dumps(metadata))) as run:
            with self.assertRaisesRegex(RuntimeError, 'external source is forbidden'):
                guard.check_graph('Cargo.toml', 'x86_64-unknown-linux-gnu')
        arguments = run.call_args.args[0]
        self.assertIn('--offline', arguments)
        self.assertIn('--locked', arguments)

    def test_external_sources_in_root_lockfile_are_rejected(self):
        self.manifest()
        (self.root / 'Cargo.lock').write_text('version=4\n[[package]]\nname="external"\nversion="1.0.0"\nsource="registry+https://example.invalid/index"\n')
        with self.assertRaisesRegex(RuntimeError, 'Root lockfile contains an external source'):
            guard.check_declarations()

    def test_unsupported_metadata_target_is_rejected_before_cargo(self):
        with patch.object(guard.subprocess, 'run') as run:
            with self.assertRaisesRegex(RuntimeError, 'Unsupported source graph target'):
                guard.check_graph('Cargo.toml', 'x86_64-unknown-linux-gnux32')
            run.assert_not_called()


class PlatformContract(unittest.TestCase):
    """Exercise direct Cargo entry points without building the product graph."""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='blog-platform-contract-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = Path(__file__).resolve().parent / 'assets/src/platform_contract.rs'
        shutil.copyfile(self.source, self.root / 'platform_contract.rs')
        (self.root / 'src').mkdir()
        (self.root / 'src/lib.rs').write_text('#![no_std]\npub fn supported() {}\n')

    def manifest(self, package, *, proc_macro=False, wrapper=False):
        build = 'build.rs' if wrapper else 'platform_contract.rs'
        (self.root / 'Cargo.toml').write_text(
            f'[package]\nname={json.dumps(package)}\nversion="0.0.0"\nedition="2024"\n'
            f'build={json.dumps(build)}\n[workspace]\n'
            + ('[lib]\nproc-macro=true\n' if proc_macro else ''))
        if wrapper:
            (self.root / 'build.rs').write_text(
                '#[path = "platform_contract.rs"]\nmod platform_contract;\n'
                'fn main() { platform_contract::check(); }\n')
        if proc_macro:
            (self.root / 'src/lib.rs').write_text(
                'extern crate proc_macro;\n'
                '#[proc_macro]\npub fn identity(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream { tokens }\n')

    def cargo(self, *arguments):
        return subprocess.run(
            ['cargo', '+1.99.0', *arguments, '--offline', '--manifest-path', str(self.root / 'Cargo.toml')],
            cwd=self.root, env={**os.environ, 'CARGO_TARGET_DIR': str(self.root / 'target')},
            text=True, capture_output=True)

    def test_direct_native_build_and_tests_accept_the_shared_guard(self):
        self.manifest('blog-application', wrapper=True)
        for arguments in [('check',), ('test', '--lib')]:
            result = self.cargo(*arguments)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_browser_build_and_native_tests_are_both_supported(self):
        self.manifest('blog-browser')
        for arguments in [('check', '--target', 'wasm32-unknown-unknown'), ('test', '--lib')]:
            result = self.cargo(*arguments)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_native_product_cannot_be_built_as_browser_wasm(self):
        self.manifest('blog-application')
        result = self.cargo('check', '--target', 'wasm32-unknown-unknown')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('unsupported build target "wasm32-unknown-unknown"', result.stderr)

    def test_direct_cargo_rejects_other_architectures_and_gnu_x32(self):
        self.manifest('blog-browser')
        for target in ('aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnux32'):
            result = self.cargo('check', '--target', target)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(f'unsupported build target "{target}"', result.stderr)

    def test_wasm_consumers_still_compile_procedural_macros_for_the_host(self):
        self.manifest('blog-component-macros', proc_macro=True)
        result = self.cargo('build', '--target', 'wasm32-unknown-unknown')
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_cross_package_shared_guard_is_tracked_by_cargo(self):
        shared = self.root / 'platform_contract.rs'
        self.root /= 'consumer'
        (self.root / 'src').mkdir(parents=True)
        (self.root / 'src/lib.rs').write_text('#![no_std]\npub fn supported() {}\n')
        self.manifest('blog-application', wrapper=True)
        (self.root / 'build.rs').write_text(
            '#[path = "../platform_contract.rs"]\nmod platform_contract;\n'
            'fn main() { platform_contract::check(); }\n')
        result = self.cargo('check')
        self.assertEqual(result.returncode, 0, result.stderr)
        output = next((self.root / 'target/debug/build').glob('blog-application-*/output')).read_text()
        watched = next(line.split('=', 1)[1] for line in output.splitlines()
                       if line.startswith('cargo:rerun-if-changed='))
        self.assertEqual(Path(watched).resolve(), shared.resolve())
        shared.write_text(shared.read_text() + '\ncompile_error!("shared guard change was rebuilt");\n')
        result = self.cargo('check')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('shared guard change was rebuilt', result.stderr)

    def test_standalone_guard_covers_unsupported_build_hosts_without_product_dependencies(self):
        binary = self.root / 'platform-contract-tests'
        subprocess.run(['rustc', '+1.99.0', '--edition=2024', '--test', str(self.source), '-o', str(binary)],
                       cwd=self.root, check=True, capture_output=True, text=True)
        result = subprocess.run([str(binary)], check=True, capture_output=True, text=True)
        self.assertIn('4 passed', result.stdout)


if __name__ == '__main__':
    unittest.main()
