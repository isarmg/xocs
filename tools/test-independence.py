#!/usr/bin/env python3
"""Exercise actual Cargo metadata against source paths that escape a checkout."""
import importlib.util
import json
from pathlib import Path
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
        return guard.check_graph('Cargo.toml', 'x86_64-unknown-linux-gnu', True)

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
                guard.check_graph('Cargo.toml', 'x86_64-unknown-linux-gnu', False)
        arguments = run.call_args.args[0]
        self.assertIn('--offline', arguments)
        self.assertIn('--locked', arguments)

    def test_external_sources_in_root_lockfile_are_rejected(self):
        self.manifest()
        (self.root / 'Cargo.lock').write_text('version=4\n[[package]]\nname="external"\nversion="1.0.0"\nsource="registry+https://example.invalid/index"\n')
        with self.assertRaisesRegex(RuntimeError, 'Root lockfile contains an external source'):
            guard.check_declarations()


if __name__ == '__main__':
    unittest.main()
