#!/usr/bin/env python3
"""Adversarial provenance and empty-home resolution checks for source management."""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('blog_source_manager', Path(__file__).with_name('manage.py'))
manage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manage)
PLATFORM_SOURCE = (Path(__file__).resolve().parents[1] / 'assets/src/platform_contract.rs').read_text()


class Sources(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='blog-source-manager-test-')
        self.root = Path(self.temporary.name)
        self.write('Cargo.toml', '[workspace]\nresolver = "3"\nmembers = ["src/server"]\nexclude = ["src/foundation"]\n')
        self.write('src/server/Cargo.toml', '''[package]
name = "blog"
version = "1.0.1"
edition = "2024"
[dependencies]
third = { path = "../foundation/third-1.0.0" }
''')
        self.write('src/server/src/lib.rs', 'pub fn result() -> u32 { third::value() }\n')
        self.write('src/server/platform_contract.rs', PLATFORM_SOURCE)
        self.write('src/server/build.rs', '#[path = "platform_contract.rs"]\nmod platform_contract;\nfn main() { platform_contract::check(); }\n')
        self.write('.cargo/config.toml', '''[net]
offline = true
[source.crates-io]
replace-with = "unavailable-registry"
[source.unavailable-registry]
directory = "compliance/empty-registry"
''')
        self.write('compliance/empty-registry/README.txt', 'No registry packages.\n')
        policy = dict(schema=1, foundation='src/foundation', owned_sources=[dict(directory='src/server')],
                      platform_guard='src/server/platform_contract.rs',
                      profiles=[dict(id='native', target='x86_64-unknown-linux-gnu', role='runtime', manifests=['src/server/Cargo.toml']),
                                dict(id='web-release', target='wasm32-unknown-unknown', role='runtime', manifests=['src/server/Cargo.toml'])])
        self.write(manage.POLICY, json.dumps(policy))
        self.original = {'Cargo.toml': b'[package]\nname = "third"\nversion = "1.0.0"\nedition = "2024"\n',
                         'src/lib.rs': b'pub fn value() -> u32 { 1 }\n', 'LICENSE': b'Original permissive license text.\n'}
        archive = self.root / 'compliance/provenance/archives/third-1.0.0.crate'
        archive.parent.mkdir(parents=True)
        with tarfile.open(archive, 'w:gz') as stream:
            for name, value in self.original.items():
                entry = tarfile.TarInfo('third-1.0.0/' + name)
                entry.size = len(value)
                stream.addfile(entry, io.BytesIO(value))
                self.write('src/foundation/third-1.0.0/' + name, value)
        self.write('compliance/licenses/upstream/third-1.0.0/LICENSE', self.original['LICENSE'])
        record = dict(id='third@1.0.0', name='third', version='1.0.0', directory='src/foundation/third-1.0.0',
                      license='MIT', repository='https://example.invalid/third',
                      upstream=dict(name='third', version='1.0.0', archive_url='https://static.crates.io/crates/third/third-1.0.0.crate',
                                    archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest(),
                                    archive='compliance/provenance/archives/third-1.0.0.crate'),
                      original_files_sha256={key: hashlib.sha256(value).hexdigest() for key, value in self.original.items()},
                      license_materials=['compliance/licenses/upstream/third-1.0.0/LICENSE'], changes=['Local source import.'])
        self.write(manage.ORIGINS, json.dumps(dict(schema=1, packages=[record])))
        subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=self.root, check=True, capture_output=True,
                       env={**os.environ, 'CARGO_NET_OFFLINE': 'true'})
        manage.refresh(self.root)

    def tearDown(self):
        self.temporary.cleanup()

    def write(self, path, value):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(value.encode() if isinstance(value, str) else value)

    def test_source_and_target_graphs_above_250_are_accepted(self):
        policy = manage.read_json(self.root / manage.POLICY)
        members = ['src/server']
        # These are real package roots and a real Cargo graph, including the
        # pre-existing server and original dependency: 249 + 2 = 251.
        for index in range(249):
            directory = f'tools/module-{index}'
            members.append(directory)
            policy['owned_sources'].append(dict(directory=directory))
            self.write(directory + '/Cargo.toml',
                       f'[package]\nname = "module-{index}"\nversion = "0.0.0"\nedition = "2024"\nbuild = "../../src/server/platform_contract.rs"\n')
            self.write(directory + '/src/lib.rs', 'pub fn maintained() {}\n')
        self.write('Cargo.toml', '[workspace]\nresolver = "3"\nmembers = ' + json.dumps(members) +
                   '\nexclude = ["src/foundation"]\n')
        subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=self.root,
                       check=True, capture_output=True,
                       env={**os.environ, 'CARGO_NET_OFFLINE': 'true'})
        for profile in policy['profiles']:
            profile['manifests'] = [directory + '/Cargo.toml' for directory in members]
        self.write(manage.POLICY, json.dumps(policy))
        current, _ = manage.snapshot(self.root)
        self.assertEqual(len(current['packages']), 251)
        for resolution in manage.resolutions(self.root, policy):
            self.assertEqual(len(resolution['packages']), 251)
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        self.assertEqual(len(manage.read_json(self.root / manage.CATALOG)['packages']), 251)

    def test_empty_home_native_and_wasm_metadata_and_current_sources(self):
        manage.verify(self.root, metadata=True)
        catalog = manage.read_json(self.root / manage.CATALOG)
        self.assertEqual([row['target'] for row in catalog['resolutions']],
                         ['x86_64-unknown-linux-gnu', 'wasm32-unknown-unknown'])
        self.assertIn('src/foundation/third-1.0.0', catalog['resolutions'][0]['packages'])

    def enable_platform_guard(self):
        policy = manage.read_json(self.root / manage.POLICY)
        manifest = self.root / 'src/server/Cargo.toml'
        manifest.write_text(manifest.read_text().replace('[package]', '[package]\nbuild = "platform_contract.rs"'))
        return policy

    def test_current_policy_requires_platform_guard(self):
        policy = manage.read_json(self.root / manage.POLICY)
        for value in (None, False, ''):
            policy['platform_guard'] = value
            self.write(manage.POLICY, json.dumps(policy))
            with self.assertRaisesRegex(ValueError, 'must declare the shared platform guard'):
                manage.refresh(self.root)
        del policy['platform_guard']
        self.write(manage.POLICY, json.dumps(policy))
        with self.assertRaisesRegex(ValueError, 'must declare the shared platform guard'):
            manage.refresh(self.root)

    def test_platform_guard_is_owned_hashed_and_cannot_be_disabled(self):
        policy = self.enable_platform_guard()
        manage.refresh(self.root)
        receipt = manage.read_json(self.root / manage.CATALOG)
        self.assertIn(policy['platform_guard'], receipt['platform_guard_sha256'])
        self.assertIn('platform_contract.rs', receipt['packages']['src/server']['files_sha256'])
        self.write(policy['platform_guard'], PLATFORM_SOURCE + '\n// Source drift.\n')
        with self.assertRaisesRegex(ValueError, 'receipt drift'):
            manage.verify(self.root)
        manifest = self.root / 'src/server/Cargo.toml'
        manifest.write_text(manifest.read_text().replace('build = "platform_contract.rs"', 'build = false'))
        with self.assertRaisesRegex(ValueError, 'disables its platform guard'):
            manage.refresh(self.root)

    def test_existing_build_script_must_call_owned_shared_guard_first(self):
        self.enable_platform_guard()
        manifest = self.root / 'src/server/Cargo.toml'
        manifest.write_text(manifest.read_text().replace('build = "platform_contract.rs"', 'build = "build.rs"'))
        self.write('src/server/build.rs', '#[path = "platform_contract.rs"]\nmod platform_contract;\nfn main() { platform_contract::check(); }\n')
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        self.write('src/server/build.rs', '#[path = "platform_contract.rs"]\nmod platform_contract;\nfn main() {}\n')
        with self.assertRaisesRegex(ValueError, 'invoke the shared platform guard first'):
            manage.refresh(self.root)

    def test_platform_guard_cannot_live_outside_owned_sources(self):
        policy = self.enable_platform_guard()
        policy['platform_guard'] = 'tools/unowned.rs'
        self.write(policy['platform_guard'], 'fn main() {}\n')
        self.write(manage.POLICY, json.dumps(policy))
        with self.assertRaisesRegex(ValueError, 'needs a current project source owner'):
            manage.refresh(self.root)

    def test_metadata_rejects_unsupported_target_before_invoking_cargo(self):
        with patch.object(manage.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'unsupported source graph target'):
                manage.cargo_metadata(self.root, 'aarch64-unknown-linux-gnu', self.root / 'cargo-home')
            run.assert_not_called()

    def test_test_profile_includes_development_only_dependency(self):
        manifest = self.root / 'src/server/Cargo.toml'
        manifest.write_text(manifest.read_text().replace('[dependencies]', '[dev-dependencies]'))
        policy = manage.read_json(self.root / manage.POLICY)
        policy['profiles'].append(dict(id='tests', target='x86_64-unknown-linux-gnu',
                                      role='compile', include_dev=True,
                                      manifests=['src/server/Cargo.toml']))
        self.write(manage.POLICY, json.dumps(policy))
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        profiles = {row['profile']: row['packages']
                    for row in manage.read_json(self.root / manage.CATALOG)['resolutions']}
        self.assertNotIn('src/foundation/third-1.0.0', profiles['native'])
        self.assertIn('src/foundation/third-1.0.0', profiles['tests'])

    def test_test_profile_requires_boolean_development_selection(self):
        policy = manage.read_json(self.root / manage.POLICY)
        policy['profiles'][0]['include_dev'] = 'false'
        self.write(manage.POLICY, json.dumps(policy))
        with self.assertRaisesRegex(ValueError, 'include_dev must be boolean'):
            manage.refresh(self.root)

    def test_virtual_root_supports_same_name_multiple_local_versions(self):
        original = {**self.original, 'Cargo.toml': self.original['Cargo.toml'].replace(b'1.0.0', b'2.0.0')}
        archive = self.root / 'compliance/provenance/archives/third-2.0.0.crate'
        with tarfile.open(archive, 'w:gz') as stream:
            for name, value in original.items():
                entry = tarfile.TarInfo('third-2.0.0/' + name)
                entry.size = len(value)
                stream.addfile(entry, io.BytesIO(value))
                self.write('src/foundation/third-2.0.0/' + name, value)
        self.write('compliance/licenses/upstream/third-2.0.0/LICENSE', original['LICENSE'])
        origins = manage.read_json(self.root / manage.ORIGINS)
        second = json.loads(json.dumps(origins['packages'][0]))
        second.update(id='third@2.0.0', version='2.0.0', directory='src/foundation/third-2.0.0',
                      original_files_sha256={name: hashlib.sha256(value).hexdigest() for name, value in original.items()},
                      license_materials=['compliance/licenses/upstream/third-2.0.0/LICENSE'])
        second['upstream'].update(version='2.0.0', archive_url='https://static.crates.io/crates/third/third-2.0.0.crate',
                                  archive='compliance/provenance/archives/third-2.0.0.crate',
                                  archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest())
        origins['packages'].append(second)
        self.write(manage.ORIGINS, json.dumps(origins))
        manifest = self.root / 'src/server/Cargo.toml'
        manifest.write_text(manifest.read_text() + 'third_v2 = { package = "third", path = "../foundation/third-2.0.0" }\n')
        subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=self.root, check=True, capture_output=True,
                       env={**os.environ, 'CARGO_NET_OFFLINE': 'true'})
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        catalog = manage.read_json(self.root / manage.CATALOG)
        self.assertEqual(len(catalog['origins']), 2)
        self.assertEqual(len(catalog['packages']), 3)

    def test_verify_never_updates_hashes_refresh_is_explicit(self):
        before = (self.root / manage.CATALOG).read_bytes()
        self.write('src/foundation/third-1.0.0/src/lib.rs', 'pub fn value() -> u32 { 2 }\n')
        with self.assertRaises(ValueError):
            manage.verify(self.root)
        self.assertEqual((self.root / manage.CATALOG).read_bytes(), before)
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)

    def test_local_modification_history_accepts_appends_but_refuses_rewrite(self):
        entry = dict(directory='src/foundation/third-1.0.0', file='src/lib.rs', reason='First reviewed local fix.')
        self.write(manage.LOCAL_MODIFICATIONS, json.dumps(dict(schema=1, changes=[entry])))
        manage.refresh(self.root)
        manage.verify(self.root)
        following = dict(directory='src/foundation/third-1.0.0', file='src/lib.rs', reason='Second reviewed local fix.')
        self.write(manage.LOCAL_MODIFICATIONS, json.dumps(dict(schema=1, changes=[entry, following])))
        manage.refresh(self.root)
        manage.verify(self.root)
        self.write(manage.LOCAL_MODIFICATIONS, json.dumps(dict(schema=1, changes=[following])))
        with self.assertRaisesRegex(ValueError, 'local modification history changed'):
            manage.refresh(self.root)

    def test_refresh_refuses_changed_or_missing_original_origin(self):
        value = manage.read_json(self.root / manage.ORIGINS)
        value['packages'][0]['changes'].append('Rewrite immutable history.')
        self.write(manage.ORIGINS, json.dumps(value))
        with self.assertRaises(ValueError):
            manage.refresh(self.root)

    def test_changed_archive_or_legal_bytes_are_rejected(self):
        self.write('compliance/licenses/upstream/third-1.0.0/LICENSE', 'changed license')
        with self.assertRaises(ValueError):
            manage.refresh(self.root)
        self.write('compliance/licenses/upstream/third-1.0.0/LICENSE', self.original['LICENSE'])
        self.write('compliance/provenance/archives/third-1.0.0.crate', b'not the original archive')
        with self.assertRaises(ValueError):
            manage.verify(self.root)

    def test_conditional_optional_external_dependency_is_rejected(self):
        path = self.root / 'src/foundation/third-1.0.0/Cargo.toml'
        path.write_text(path.read_text() + '\n[target.\'cfg(windows)\'.dependencies]\nexternal = { version = "1", optional = true }\n')
        with self.assertRaisesRegex(ValueError, 'non-path dependency'):
            manage.refresh(self.root)

    def test_nested_workspace_lock_and_external_paths_are_rejected(self):
        self.write('src/foundation/third-1.0.0/Cargo.lock', 'version = 4\n')
        with self.assertRaisesRegex(ValueError, 'nested Cargo.lock'):
            manage.verify(self.root)
        (self.root / 'src/foundation/third-1.0.0/Cargo.lock').unlink()
        package = self.root / 'src/foundation/third-1.0.0/Cargo.toml'
        package.write_text(package.read_text() + '\n[workspace]\n')
        with self.assertRaisesRegex(ValueError, 'nested Cargo workspace'):
            manage.refresh(self.root)
        package.write_bytes(self.original['Cargo.toml'])
        server = self.root / 'src/server/Cargo.toml'
        server.write_text(server.read_text().replace('../foundation/third-1.0.0', '/tmp/outside-blog-source'))
        with self.assertRaisesRegex(ValueError, 'external source path'):
            manage.refresh(self.root)

    def test_removed_package_identity_and_dangling_features_are_rejected(self):
        policy = manage.read_json(self.root / manage.POLICY)
        documents = manage.static_structure(self.root, policy)
        previous, _ = manage.snapshot(self.root)
        following = dict(previous, packages={key: value for key, value in previous['packages'].items() if key == 'src/server'})
        with self.assertRaisesRegex(ValueError, 'disappeared'):
            manage.preserve(previous, following)
        self.write('src/foundation/third-1.0.0/Cargo.toml', self.original['Cargo.toml'] + b'\n[features]\nfuture = ["dep:removed"]\n')
        with self.assertRaisesRegex(ValueError, 'dangling feature edge'):
            manage.static_structure(self.root, policy)

    def test_archive_receipts_are_rechecked_and_links_forbidden(self):
        value = manage.read_json(self.root / manage.ORIGINS)
        value['packages'][0]['original_files_sha256']['src/lib.rs'] = '0' * 64
        self.write(manage.ORIGINS, json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'file receipts differ'):
            manage.verify(self.root)
        path = self.root / 'hostile.crate'
        with tarfile.open(path, 'w:gz') as archive:
            entry = tarfile.TarInfo('third-1.0.0/src/link')
            entry.type = tarfile.SYMTYPE
            entry.linkname = '/etc/passwd'
            archive.addfile(entry)
        with self.assertRaisesRegex(ValueError, 'links/special'):
            manage.archive_files(path, 'third', '1.0.0')


class GeneratedOutputs(unittest.TestCase):
    """Pure file fixtures: no Cargo command or actual project source writes."""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='blog-legacy-target-test-')
        self.root = Path(self.temporary.name)
        self.write('Cargo.toml', '[workspace]\nresolver = "3"\nmembers = ["tools/web-build", "web/app/domain-tests"]\nexclude = ["src/foundation"]\n')
        self.write('Cargo.lock', 'version = 4\n')
        self.write('.cargo/config.toml', '[net]\noffline = true\n[source.crates-io]\nreplace-with = "unavailable-registry"\n[source.unavailable-registry]\ndirectory = "compliance/empty-registry"\n')
        self.write('compliance/empty-registry/README.txt', 'No registry packages.\n')
        self.directories = ['tools/web-build', 'web/app/domain-tests']
        self.write('.gitignore', ''.join('/' + item + '/target/\n' for item in self.directories))
        for index, directory in enumerate(self.directories):
            self.write(directory + '/Cargo.toml', '[package]\nname = "owned-' + str(index) + '"\nversion = "1.0.0"\n')
            self.write(directory + '/src/lib.rs', 'pub fn retained() {}\n')
            relative_guard = os.path.relpath(self.root / 'tools/web-build/platform_contract.rs', self.root / directory)
            self.write(directory + '/build.rs', f'#[path = "{relative_guard}"]\nmod platform_contract;\nfn main() {{ platform_contract::check(); }}\n')
        self.write('tools/web-build/platform_contract.rs', PLATFORM_SOURCE)
        self.write('src/foundation/third-1.0.0/Cargo.toml', '[package]\nname = "third"\nversion = "1.0.0"\n')
        self.write('src/foundation/third-1.0.0/src/target/real.rs', 'pub const REAL_SOURCE: bool = true;\n')
        self.policy = dict(foundation='src/foundation',
                           platform_guard='tools/web-build/platform_contract.rs',
                           owned_sources=[dict(directory=item) for item in self.directories],
                           profiles=[dict(id='native', target='x86_64-unknown-linux-gnu', role='runtime', manifests=['tools/web-build/Cargo.toml']),
                                     dict(id='web-release', target='wasm32-unknown-unknown', role='runtime', manifests=['web/app/domain-tests/Cargo.toml'])])
        self.origins = {'third@1.0.0': dict(id='third@1.0.0', directory='src/foundation/third-1.0.0', name='third', version='1.0.0')}

    def tearDown(self):
        self.temporary.cleanup()

    def write(self, path, value):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(value)

    def inventory(self):
        documents = manage.static_structure(self.root, self.policy)
        return manage.source_packages(self.root, self.policy, documents, self.origins)

    def test_legacy_ignored_package_targets_do_not_enter_source_receipts(self):
        before = self.inventory()
        for directory in [*self.directories, 'src/foundation/third-1.0.0']:
            self.write(directory + '/target/.rustc_info.json', '{"legacy":true}')
            self.write(directory + '/target/debug/deps/liblegacy.rlib', 'old binary output')
            self.write(directory + '/target/debug/.fingerprint/legacy/output-lib', 'old diagnostics')
            self.write(directory + '/target/debug/Cargo.lock', 'generated nested lock')
            self.write(directory + '/target/debug/build/output/Cargo.toml', '[workspace]\n')
        self.assertEqual(self.inventory(), before)
        self.write('tools/web-build/target/debug/deps/liblegacy.rlib', 'rebuilt binary output')
        self.assertEqual(self.inventory(), before)
        for directory in [*self.directories, 'src/foundation/third-1.0.0']:
            shutil.rmtree(self.root / directory / 'target')
        self.assertEqual(self.inventory(), before)

    def test_real_src_target_remains_hashed_and_checked(self):
        before = self.inventory()
        directory = 'src/foundation/third-1.0.0'
        self.assertIn('src/target/real.rs', before[directory]['files_sha256'])
        self.write(directory + '/src/target/real.rs', 'pub const REAL_SOURCE: bool = false;\n')
        self.assertNotEqual(self.inventory(), before)
        self.write(directory + '/src/target/Cargo.lock', 'version = 4\n')
        with self.assertRaisesRegex(ValueError, 'nested Cargo.lock'):
            self.inventory()


class TestClosures(unittest.TestCase):
    def test_development_dependencies_belong_only_to_selected_roots(self):
        root = Path('/workspace/closure-fixture')
        manifests = {name: f'src/{name}/Cargo.toml'
                     for name in ('entry', 'library', 'test-helper', 'library-test', 'derive', 'build')}
        packages = [dict(id=name, name=name, version='0.0.0',
                         manifest_path=str(root / manifest),
                         targets=[dict(kind=['proc-macro' if name == 'derive' else 'lib'])])
                    for name, manifest in manifests.items()]
        links = {'entry': [('library', None), ('test-helper', 'dev'), ('derive', None)],
                 'library': [('library-test', 'dev'), ('build', 'build')],
                 'test-helper': [('library', None)]}
        nodes = [dict(id=name, features=[],
                      deps=[dict(pkg=target, dep_kinds=[dict(kind=kind)])
                            for target, kind in links.get(name, [])])
                 for name in manifests]
        metadata = dict(packages=packages, resolve=dict(nodes=nodes),
                        workspace_members=['entry', 'library'], workspace_root=str(root))

        production = manage.closure(metadata, [manifests['entry']], 'runtime')
        tests = manage.closure(metadata, [manifests['entry']], 'runtime', include_dev=True)
        self.assertNotIn('src/test-helper', production)
        self.assertIn('src/test-helper', tests)
        self.assertNotIn('src/library-test', tests)
        self.assertEqual(tests['src/derive']['roles'], ['compile'])
        self.assertEqual(tests['src/build']['roles'], ['compile'])
        self.assertEqual(tests['src/test-helper']['roles'], ['runtime'])

        both = manage.closure(metadata, [manifests['entry'], manifests['library']],
                              'compile', include_dev=True)
        self.assertIn('src/library-test', both)


class Migrations(unittest.TestCase):
    setUp = Sources.setUp
    tearDown = Sources.tearDown
    write = Sources.write

    def enable_migrations(self):
        catalog = manage.read_json(self.root / manage.CATALOG)
        baseline = [manage.identity(item) for _, item in sorted(catalog['packages'].items())]
        ledger = dict(schema=1, baseline=baseline, events=[])
        policy = manage.read_json(self.root / manage.POLICY)
        for item in policy['owned_sources']:
            item.update(layer='product', purpose='博客产品实现与业务入口')
        policy['maintained_sources'] = [dict(directory='src/foundation/third-1.0.0', origins=['third@1.0.0'], layer='data', purpose='产品需要的整数数据操作')]
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        return ledger, policy

    def relock(self):
        subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=self.root, check=True, capture_output=True,
                       env={**os.environ, 'CARGO_NET_OFFLINE': 'true'})

    def rename_dependency(self):
        ledger, policy = self.enable_migrations()
        old = next(item for item in ledger['baseline'] if item['origins'])
        current = dict(directory='src/foundation/data/integer-value', name='blog-integer-value', version='0.0.0', origins=['third@1.0.0'])
        destination = self.root / current['directory']
        destination.parent.mkdir(parents=True)
        shutil.move(self.root / old['directory'], destination)
        self.write(current['directory'] + '/Cargo.toml', '[package]\nname = "blog-integer-value"\npublish = false\nedition = "2024"\n')
        self.write('src/server/Cargo.toml', '[package]\nname = "blog"\nversion = "1.0.1"\nedition = "2024"\n[dependencies]\nthird = { package = "blog-integer-value", path = "../foundation/data/integer-value" }\n')
        policy['maintained_sources'][0]['directory'] = current['directory']
        ledger['events'].append(dict(id='purpose-layout', reason='按产品数据职责维护；保留原始来源，内部版本省略', **{'from': [old], 'to': [current]}, retired_origins=[]))
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        self.relock()
        return ledger, policy, current

    def test_rename_nested_directory_internal_name_and_omitted_version(self):
        old_origins = (self.root / manage.ORIGINS).read_bytes()
        ledger, policy, current = self.rename_dependency()
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        self.assertEqual((self.root / manage.ORIGINS).read_bytes(), old_origins)
        catalog = manage.read_json(self.root / manage.CATALOG)
        self.assertEqual(manage.identity(catalog['packages'][current['directory']]), current)
        self.assertEqual(catalog['packages'][current['directory']]['dependencies'], [])
        architecture = (self.root / manage.ARCHITECTURE_DOC).read_text()
        self.assertIn('产品需要的整数数据操作', architecture)
        self.assertIn('src/server', architecture)
        self.assertIn('third@1.0.0', architecture)
        self.assertIn('0.0.0', architecture)

    def test_owned_package_directory_and_name_migrate_too(self):
        ledger, policy, current = self.rename_dependency()
        old = next(item for item in ledger['baseline'] if not item['origins'])
        new = dict(directory='src/service', name='blog-service', version='0.0.0', origins=[])
        shutil.move(self.root / old['directory'], self.root / new['directory'])
        path = self.root / new['directory'] / 'Cargo.toml'
        path.write_text(path.read_text().replace('name = "blog"', 'name = "blog-service"').replace('version = "1.0.1"\n', ''))
        self.write('Cargo.toml', '[workspace]\nresolver = "3"\nmembers = ["src/service"]\nexclude = ["src/foundation"]\n')
        policy['owned_sources'][0]['directory'] = new['directory']
        policy['platform_guard'] = new['directory'] + '/platform_contract.rs'
        for profile in policy['profiles']:
            profile['manifests'] = ['src/service/Cargo.toml']
        ledger['events'][0]['from'].append(old)
        ledger['events'][0]['to'].append(new)
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        self.relock()
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)

    def test_ledger_baseline_must_bind_existing_catalog(self):
        ledger, _ = self.enable_migrations()
        ledger['baseline'][0]['version'] = '999.0.0'
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        previous = manage.read_json(self.root / manage.CATALOG)
        following = dict(previous, source_migrations=ledger)
        with self.assertRaisesRegex(ValueError, 'baseline does not match'):
            manage.preserve(previous, following)

    def test_migration_history_is_append_only_and_cannot_disappear(self):
        ledger, _, _ = self.rename_dependency()
        manage.refresh(self.root)
        ledger['events'][0]['reason'] = 'rewrite'
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        with self.assertRaisesRegex(ValueError, 'migration history changed'):
            manage.refresh(self.root)
        (self.root / manage.MIGRATIONS).unlink()
        with self.assertRaises(ValueError):
            manage.refresh(self.root)

    def test_terminal_identity_and_source_mapping_cannot_be_forged(self):
        ledger, policy, current = self.rename_dependency()
        path = self.root / current['directory'] / 'Cargo.toml'
        path.write_text(path.read_text().replace('blog-integer-value', 'unrecorded-rename'))
        with self.assertRaisesRegex(ValueError, 'terminal identities'):
            manage.refresh(self.root)

    def test_unowned_foundation_code_and_unregistered_manifest_are_rejected(self):
        self.rename_dependency()
        self.write('src/foundation/hidden/secret.rs', 'pub fn hidden() {}')
        with self.assertRaisesRegex(ValueError, 'unowned foundation source'):
            manage.refresh(self.root)
        (self.root / 'src/foundation/hidden/secret.rs').unlink()
        self.write('src/foundation/hidden/Cargo.toml', '[package]\nname = "hidden"\n')
        with self.assertRaisesRegex(ValueError, 'unowned foundation source'):
            manage.refresh(self.root)

    def test_explicit_retirement_keeps_original_archive_and_licenses(self):
        ledger, policy = self.enable_migrations()
        old = next(item for item in ledger['baseline'] if item['origins'])
        ledger['events'].append(dict(id='remove-unused', reason='产品不再需要该实现', **{'from': [old], 'to': []}, retired_origins=[]))
        shutil.rmtree(self.root / old['directory'])
        policy['maintained_sources'] = []
        self.write('src/server/Cargo.toml', '[package]\nname = "blog"\nversion = "1.0.1"\nedition = "2024"\n')
        self.write('src/server/src/lib.rs', 'pub fn result() -> u32 { 1 }\n')
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        self.relock()
        with self.assertRaisesRegex(ValueError, 'silently loses lineage'):
            manage.refresh(self.root)
        ledger['events'][0]['retired_origins'] = [dict(id='third@1.0.0', reason='受支持入口不再需要该能力')]
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        self.assertTrue((self.root / 'compliance/provenance/archives/third-1.0.0.crate').is_file())
        catalog = manage.read_json(self.root / manage.CATALOG)
        self.assertEqual(catalog['retired_origins']['third@1.0.0']['reason'], '受支持入口不再需要该能力')
        self.assertNotIn('third@1.0.0', (self.root / manage.DOC).read_text())

    def prepare_retired_original_restore(self):
        ledger, policy = self.enable_migrations()
        dependency = next(item for item in ledger['baseline'] if item['origins'])
        owner = next(item for item in ledger['baseline'] if not item['origins'])
        ledger['events'].append(dict(id='retire-original', reason='Remove the unused package',
                                    **{'from': [dependency], 'to': []},
                                    retired_origins=[dict(id='third@1.0.0', reason='No active implementation')]))
        shutil.rmtree(self.root / dependency['directory'])
        policy['maintained_sources'] = []
        self.write('src/server/Cargo.toml', '[package]\nname = "blog"\nversion = "1.0.1"\nedition = "2024"\n')
        self.write('src/server/src/lib.rs', 'pub fn result() -> u32 { 1 }\n')
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        self.relock()
        manage.refresh(self.root)
        restored_owner = dict(owner, origins=['third@1.0.0'])
        event = dict(id='restore-copied-helper', reason='The retained owned implementation copies the original helper',
                     **{'from': [owner], 'to': [restored_owner]},
                     restored_origins=['third@1.0.0'], retired_origins=[])
        ledger['events'].append(event)
        policy['owned_sources'][0]['origins'] = ['third@1.0.0']
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        return ledger, policy, event

    def test_explicit_restore_retired_original_into_owned_code(self):
        immutable = (self.root / manage.ORIGINS).read_bytes()
        ledger, _, _ = self.prepare_retired_original_restore()
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        catalog = manage.read_json(self.root / manage.CATALOG)
        self.assertEqual(catalog['packages']['src/server']['origins'], ['third@1.0.0'])
        self.assertNotIn('third@1.0.0', catalog['retired_origins'])
        self.assertEqual(catalog['source_migrations']['events'], ledger['events'])
        self.assertEqual((self.root / manage.ORIGINS).read_bytes(), immutable)

    def test_restore_rejects_unknown_original(self):
        ledger, _, event = self.prepare_retired_original_restore()
        event['restored_origins'] = ['unknown@9.0.0']
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        with self.assertRaisesRegex(ValueError, 'restored origins.*known original'):
            manage.snapshot(self.root)

    def test_restore_rejects_original_that_was_never_retired(self):
        ledger, _ = self.enable_migrations()
        owner = next(item for item in ledger['baseline'] if not item['origins'])
        ledger['events'].append(dict(id='invalid-restore', reason='Already active original',
                                    **{'from': [owner], 'to': [dict(owner, origins=['third@1.0.0'])]},
                                    restored_origins=['third@1.0.0'], retired_origins=[]))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        with self.assertRaisesRegex(ValueError, 'only explicitly retired'):
            manage.snapshot(self.root)

    def test_restore_rejects_unconserved_requested_lineage(self):
        ledger, _, event = self.prepare_retired_original_restore()
        event['to'][0]['origins'] = []
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        with self.assertRaisesRegex(ValueError, 'exactly match introduced lineage'):
            manage.snapshot(self.root)

    def test_restore_requires_unchanged_original_archive(self):
        self.prepare_retired_original_restore()
        origins, _ = manage.original_sources(self.root)
        (self.root / 'compliance/provenance/archives/third-1.0.0.crate').unlink()
        with self.assertRaisesRegex(ValueError, 'restored original archive is missing or changed'):
            manage.source_migrations(self.root, origins)
        self.write('compliance/provenance/archives/third-1.0.0.crate', b'changed original archive bytes')
        with self.assertRaisesRegex(ValueError, 'restored original archive is missing or changed'):
            manage.source_migrations(self.root, origins)

    def test_merge_split_lineage_and_unconsumed_overwrite(self):
        first = dict(directory='src/foundation/a-old', name='a', version='1.0.0', origins=['a@1.0.0'])
        second = dict(directory='src/foundation/b-old', name='b', version='1.0.0', origins=['b@1.0.0'])
        merged = dict(directory='src/foundation/data/combined', name='combined', version='0.0.0', origins=['a@1.0.0', 'b@1.0.0'])
        left = dict(directory='src/foundation/data/left', name='left', version='0.0.0', origins=['a@1.0.0'])
        right = dict(directory='src/foundation/data/right', name='right', version='0.0.0', origins=['b@1.0.0'])
        ledger = dict(schema=1, baseline=[first, second], events=[dict(id='merge', reason='共同的数据职责', **{'from': [first, second], 'to': [merged]}, retired_origins=[]), dict(id='split', reason='分离不同职责', **{'from': [merged], 'to': [left, right]}, retired_origins=[])])
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        _, state = manage.source_migrations(self.root, {'a@1.0.0': {}, 'b@1.0.0': {}})
        self.assertEqual(set(state['packages']), {left['directory'], right['directory']})
        ledger['events'][0]['from'] = [first]
        ledger['events'][0]['to'] = [dict(second, origins=['a@1.0.0'])]
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        with self.assertRaisesRegex(ValueError, 'overwrites an unconsumed'):
            manage.source_migrations(self.root, {'a@1.0.0': {}, 'b@1.0.0': {}})

    def test_historical_patch_paths_survive_owned_tool_merge(self):
        self.write('tools/legacy-binding/Cargo.toml', '[package]\nname="legacy-binding"\nversion="0.2.129"\nedition="2024"\nbuild="../../src/server/platform_contract.rs"\n')
        self.write('tools/legacy-binding/src/lib.rs', 'pub fn binding() {}\n')
        self.write('Cargo.toml', '[workspace]\nresolver="3"\nmembers=["src/server","tools/legacy-binding"]\nexclude=["src/foundation"]\n')
        policy = manage.read_json(self.root / manage.POLICY)
        policy['owned_sources'].append(dict(directory='tools/legacy-binding'))
        self.write(manage.POLICY, json.dumps(policy))
        patch = dict(directory='tools/legacy-binding', file='src/lib.rs', files=['src/lib.rs'], reason='Original binding Web mode fix before integration')
        self.write(manage.LOCAL_MODIFICATIONS, json.dumps(dict(schema=1, changes=[patch])))
        self.write('compliance/provenance/binding-original.json', '{"source":"preserved upstream receipt"}')
        self.relock()
        manage.refresh(self.root)
        history_before = (self.root / manage.LOCAL_MODIFICATIONS).read_bytes()
        provenance_before = (self.root / 'compliance/provenance/binding-original.json').read_bytes()
        ledger, policy = self.enable_migrations()
        destination = self.root / 'tools/assets'
        destination.parent.mkdir(exist_ok=True)
        shutil.move(self.root / 'src/server', destination)
        self.write('tools/assets/src/embedded_binding.rs', (self.root / 'tools/legacy-binding/src/lib.rs').read_text())
        shutil.rmtree(self.root / 'tools/legacy-binding')
        self.write('tools/assets/Cargo.toml', '[package]\nname="blog-assets"\npublish=false\nedition="2024"\n[dependencies]\nthird={path="../../src/foundation/third-1.0.0"}\n')
        self.write('Cargo.toml', '[workspace]\nresolver="3"\nmembers=["tools/assets"]\nexclude=["src/foundation"]\n')
        policy['owned_sources'] = [dict(directory='tools/assets', layer='build', purpose='资产构建并整合浏览器绑定生成')]
        policy['platform_guard'] = 'tools/assets/platform_contract.rs'
        for profile in policy['profiles']:
            profile['manifests'] = ['tools/assets/Cargo.toml']
        product = dict(directory='tools/assets', name='blog-assets', version='0.0.0', origins=[])
        retained = next(item for item in ledger['baseline'] if item['origins'])
        ledger['events'] = [dict(id='integrate-binding', reason='将独立绑定工具实现整合到资产构建器，原始来源及补丁历史保留', **{'from': ledger['baseline'], 'to': [product, retained]}, retired_origins=[])]
        self.write(manage.POLICY, json.dumps(policy))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        self.relock()
        manage.refresh(self.root)
        manage.verify(self.root, metadata=True)
        self.assertFalse((self.root / patch['directory']).exists())
        self.assertEqual((self.root / manage.LOCAL_MODIFICATIONS).read_bytes(), history_before)
        self.assertEqual((self.root / 'compliance/provenance/binding-original.json').read_bytes(), provenance_before)
        catalog = manage.read_json(self.root / manage.CATALOG)
        self.assertEqual(len([item for item in catalog['packages'].values() if not item['origins']]), 1)
        # A plausible-looking but never registered historical path is rejected.
        record = dict(schema=1, changes=[patch, dict(directory='tools/invented-history', file='src/lib.rs', reason='invalid history')])
        self.write(manage.LOCAL_MODIFICATIONS, json.dumps(record))
        with self.assertRaisesRegex(ValueError, 'no current or migration identity'):
            manage.refresh(self.root)

    def test_historical_patch_files_array_cannot_escape_its_directory(self):
        ledger, _, _ = self.rename_dependency()
        old = next(item for item in ledger['baseline'] if item['origins'])
        self.write(manage.LOCAL_MODIFICATIONS, json.dumps(dict(schema=1, changes=[dict(directory=old['directory'], files=['../escape.rs'], reason='invalid file path')])))
        with self.assertRaisesRegex(ValueError, 'unsafe repository path'):
            manage.refresh(self.root)

    def test_migrated_sources_still_cannot_rewrite_original_identity(self):
        self.rename_dependency()
        manage.refresh(self.root)
        origins = manage.read_json(self.root / manage.ORIGINS)
        origins['packages'][0]['changes'].append('Rewrite immutable history after rename')
        self.write(manage.ORIGINS, json.dumps(origins))
        with self.assertRaisesRegex(ValueError, 'immutable original identity'):
            manage.refresh(self.root)

    def test_migration_cannot_shrink_complete_source_receipts_to_an_include_list(self):
        _, policy, _ = self.rename_dependency()
        policy['maintained_sources'][0]['include'] = ['Cargo.toml']
        self.write(manage.POLICY, json.dumps(policy))
        with self.assertRaisesRegex(ValueError, 'complete source directories'):
            manage.refresh(self.root)

    def test_new_origin_needs_a_separate_explicit_import_event(self):
        first = dict(directory='src/foundation/a', name='a', version='1.0.0', origins=['a@1.0.0'])
        second = dict(directory='src/foundation/b', name='b', version='0.0.0', origins=['b@2.0.0'])
        ledger = dict(schema=1, baseline=[first], events=[])
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        origins = {'a@1.0.0': {}, 'b@2.0.0': {}}
        with self.assertRaisesRegex(ValueError, 'must enter migration history explicitly'):
            manage.source_migrations(self.root, origins)
        ledger['events'].append(dict(id='import-new', reason='新增产品需要的数据实现', **{'from': [], 'to': [second]}, retired_origins=[]))
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        _, state = manage.source_migrations(self.root, origins)
        self.assertEqual(set(state['packages']), {first['directory'], second['directory']})

    def test_existing_origin_cannot_be_borrowed_without_consuming_source(self):
        first = dict(directory='src/foundation/a', name='a', version='1.0.0', origins=['a@1.0.0'])
        fake = dict(directory='src/foundation/fake', name='fake', version='0.0.0', origins=['a@1.0.0'])
        ledger = dict(schema=1, baseline=[first], events=[dict(id='borrow', reason='invalid', **{'from': [], 'to': [fake]}, retired_origins=[])])
        self.write(manage.MIGRATIONS, json.dumps(ledger))
        with self.assertRaisesRegex(ValueError, 'explicit import event'):
            manage.source_migrations(self.root, {'a@1.0.0': {}})


if __name__ == '__main__':
    unittest.main()
