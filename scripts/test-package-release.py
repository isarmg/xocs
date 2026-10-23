#!/usr/bin/env python3
"""The formal and candidate path share exact binary staging, not two packagers."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('release', ROOT / 'scripts/package-release.py')
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class Staging(unittest.TestCase):
    def test_receipt_and_staged_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary, destination = root / 'server', root / 'staged'
            binary.write_bytes(b'owned binary fixture')
            receipt = {'format': 'blog-assets-receipt-v1',
                       'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                       'identity': {'source_revision': 'a' * 40, 'product': 'blog',
                                    'target': 'x86_64-unknown-linux-gnu'}}
            path = Path(str(binary) + '.blog-assets.json')
            path.write_text(json.dumps(receipt))
            release.stage_verified_server(binary, destination, 'a' * 40)
            self.assertEqual(destination.read_bytes(), binary.read_bytes())
            self.assertEqual(destination.stat().st_mode & 0o777, 0o755)
            with self.assertRaises(FileExistsError):
                release.stage_verified_server(binary, destination, 'a' * 40)
            destination.unlink()
            for field, value in [('product', 'xsos'), ('source_revision', 'b' * 40), ('target', 'wrong')]:
                altered = {**receipt, 'identity': {**receipt['identity'], field: value}}
                path.write_text(json.dumps(altered))
                with self.assertRaises(ValueError):
                    release.stage_verified_server(binary, destination, 'a' * 40)
                self.assertFalse(destination.exists())
            path.write_text(json.dumps(receipt))
            binary.write_bytes(b'replaced after receipt')
            with self.assertRaises(ValueError):
                release.stage_verified_server(binary, destination, 'a' * 40)
            self.assertFalse(destination.exists())

    def test_original_license_is_copied_unchanged(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            release.stage_original_notices(destination, source=ROOT)
            self.assertEqual((destination / 'LICENSE').read_bytes(), (ROOT / 'LICENSE').read_bytes())
            self.assertEqual(list(destination.iterdir()), [destination / 'LICENSE'])

    def test_standalone_manual_links_resolve_inside_the_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            release.stage_documentation(destination, 'a' * 40, source=ROOT)
            self.assertEqual(len(list(destination.rglob('*.md'))), 6)
            for manual in destination.rglob('*.md'):
                for target in re.findall(r'\[[^\]\n]+\]\(([^\s)]+)\)', manual.read_text()):
                    reference = urlsplit(target)
                    if reference.scheme or reference.netloc or not reference.path:
                        continue
                    resolved = (manual.parent / reference.path).resolve()
                    self.assertTrue(resolved.is_relative_to(destination), target)
                    self.assertTrue(resolved.is_file(), f'{manual.name}: {target}')
            first_run = (destination / 'docs/getting-started.md').read_text()
            self.assertIn('https://github.com/isarmg/xocs/blob/' + 'a' * 40
                          + '/docs/development.md', first_run)


class CompiledBuildIdentity(unittest.TestCase):
    def identity(self):
        return dict(format='blog-build-v1', product='blog', version='1.0.1',
                    source_revision='a' * 40, target='x86_64-unknown-linux-gnu',
                    web_assets_sha256='b' * 64, source_dirty=True,
                    source_tree_sha256='c' * 64, build_mode='candidate',
                    schema_sha256='d' * 64, core_source_sha256='e' * 64)

    def test_sidecar_identity_cannot_relabel_an_existing_executable(self):
        identity = self.identity()
        encoded = release.canonical_build_identity(identity)
        digest = hashlib.sha256(encoded).hexdigest()
        binary = b'compiler artifact' + encoded + digest.encode()
        release.require_compiled_identity(binary, identity, encoded, digest)
        for field, value in (('source_tree_sha256', 'f' * 64),
                             ('schema_sha256', '0' * 64), ('core_source_sha256', '1' * 64)):
            altered = {**identity, field: value}
            sidecar = release.canonical_build_identity(altered)
            with self.assertRaises(ValueError):
                release.require_compiled_identity(binary, altered, sidecar, hashlib.sha256(sidecar).hexdigest())

    def test_both_canonical_identity_bytes_and_digest_must_be_retained(self):
        identity = self.identity()
        encoded = release.canonical_build_identity(identity)
        digest = hashlib.sha256(encoded).hexdigest()
        for binary in (encoded, digest.encode(), b'unrelated executable'):
            with self.assertRaises(ValueError):
                release.require_compiled_identity(binary, identity, encoded, digest)
        with self.assertRaises(ValueError):
            release.require_compiled_identity(encoded + digest.encode(), identity,
                                              json.dumps(identity, indent=2).encode(), digest)


class PackageIntegrity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import sys
        sys.path.insert(0, str(ROOT / 'tools/packaging'))
        import package
        cls.package = package

    def fixture(self, root, *, dirty=True):
        inventory = {'format': 'blog-browser-assets-v1', 'files': []}
        inventory_bytes = json.dumps(inventory, separators=(',', ':')).encode()
        build_identity = CompiledBuildIdentity().identity()
        build_identity.update(source_dirty=dirty, web_assets_sha256=hashlib.sha256(inventory_bytes).hexdigest())
        encoded = release.canonical_build_identity(build_identity)
        identity_sha256 = hashlib.sha256(encoded).hexdigest()
        binary = b'fixture executable' + encoded + identity_sha256.encode()
        state = dict(contract_version=1, application='blog', application_version='1.0.1',
                     source_revision='a' * 40, schema={'revision': 1, 'sha256': 'd' * 64})
        state_bytes = json.dumps(state).encode()
        identity = dict(product='blog', target='x86_64-unknown-linux-gnu', version='1.0.1',
                        source_revision='a' * 40, state_contract_sha256=hashlib.sha256(state_bytes).hexdigest())
        receipt = dict(format='blog-assets-receipt-v1', binary_sha256=hashlib.sha256(binary).hexdigest(),
                       identity_sha256=identity_sha256, inventory_sha256=build_identity['web_assets_sha256'],
                       identity=build_identity, inventory=inventory)
        contents = {
            'bin/blog': binary, 'libexec/blog-setup': binary,
            'libexec/package.py': b'fixture', 'libexec/install.py': b'fixture',
            'libexec/maintenance.py': b'fixture', 'install.sh': b'fixture', 'maintenance.sh': b'fixture',
            'share/config/config.json.example': b'{}',
            'share/build/blog-identity.json': json.dumps(identity).encode(),
            'share/build/blog-build-identity.json': encoded,
            'share/build/server.blog-assets.json': json.dumps(receipt).encode(),
            'share/build/blog-schema.json': json.dumps(dict(application='blog', application_version='blog-db-v1',
                                                          schema_revision=1, schema_sha256='d' * 64)).encode(),
            'share/build/blog-state-contract.json': state_bytes,
            'share/build/blog-browser-assets.json': inventory_bytes,
            'share/build/contracts/install.json': b'{}', 'share/build/frontend.json': b'{}',
        }
        for name, value in contents.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(value)
            path.chmod(0o644)
        # Package directory modes are a contract, independent of the caller's umask.
        for directory in root.rglob("*"):
            if directory.is_dir():
                directory.chmod(0o755)
        self.package.write_manifest(root, identity, 'candidate', build_identity['source_tree_sha256'], dirty)

    def test_content_mode_extra_file_and_links_fail_closed(self):
        import os
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            payload = root / 'bin/blog'
            original = payload.read_bytes()
            payload.write_bytes(b'altered')
            with self.assertRaises(ValueError):
                self.package.verify(root)
            payload.write_bytes(original)
            payload.chmod(0o755)
            with self.assertRaises(ValueError):
                self.package.verify(root)
            payload.chmod(0o644)
            extra = root / 'libexec/unmanifested.py'
            extra.write_bytes(b'not in manifest')
            with self.assertRaises(ValueError):
                self.package.verify(root)
            extra.unlink()
            linked = root / 'linked-copy'
            os.link(payload, linked)
            with self.assertRaises(ValueError):
                self.package.verify(root)
            linked.unlink()
            payload.unlink()
            payload.symlink_to('/etc/passwd')
            with self.assertRaises(ValueError):
                self.package.verify(root)

    def test_mutable_state_is_not_payload_but_immutable_additions_are(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            (root / 'state').mkdir(mode=0o700)
            (root / 'state/site.sqlite').write_bytes(b'private state')
            self.package.verify(root)
            (root / 'share/extra-directory').mkdir(mode=0o700)
            with self.assertRaises(ValueError):
                self.package.verify(root)

    def test_dirty_source_cannot_be_published_as_formal_release(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            manifest = self.package.verify(root)
            with self.assertRaises(ValueError):
                self.package.write_manifest(root, manifest['identity'], 'release', 'b' * 64, True)

    def test_candidate_cannot_be_relabelled_release_by_manifest_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root, dirty=False)
            manifest = self.package.verify(root)
            with self.assertRaises(ValueError):
                self.package.write_manifest(root, manifest['identity'], 'release',
                                            manifest['source_tree_sha256'], False)

    def test_rewritten_receipt_and_manifest_cannot_relabel_the_actual_candidate(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root, dirty=False)
            manifest = self.package.verify(root)
            material = root / 'share/build'
            receipt = json.loads((material / 'server.blog-assets.json').read_bytes())
            receipt['identity']['build_mode'] = 'release'
            encoded = release.canonical_build_identity(receipt['identity'])
            receipt['identity_sha256'] = hashlib.sha256(encoded).hexdigest()
            (material / 'blog-build-identity.json').write_bytes(encoded)
            (material / 'server.blog-assets.json').write_text(json.dumps(receipt))
            with self.assertRaises(ValueError):
                self.package.write_manifest(root, manifest['identity'], 'release',
                                            manifest['source_tree_sha256'], False)

    def test_production_root_must_remain_visible_to_systemd_isolation(self):
        spec = importlib.util.spec_from_file_location('blog_installer', ROOT / 'tools/packaging/install.py')
        installer = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(installer)
        for name in ('/tmp', '/tmp/blog', '/var/tmp', '/var/tmp/blog', '/home', '/home/blog',
                     '/root', '/root/blog', '/run/user', '/run/user/1000/blog'):
            with self.assertRaises(ValueError):
                installer.validate_service_root(Path(name))
        installer.validate_service_root(Path('/opt/isarmg/blog'))
        installer.validate_service_root(Path('/var/lib/blog'))
        installer.validate_service_root(Path('/tmp/blog-fixture'), fixture=True)
        installer.validate_service_root(Path('/opt/tmp-blog'))

    def test_offline_environment_removes_only_declared_runtime_overrides(self):
        source = {name: 'pollution' for name in self.package.RUNTIME_OVERRIDE_ENV}
        retained = dict(BLOG_SOURCE_REVISION='a' * 40, BLOG_SOURCE_TREE_SHA256='b' * 64,
                        BLOG_BUILD_MODE='candidate', BLOG_SOURCE_DIRTY='true',
                        BLOG_BUILD_METADATA_DIR='/metadata', PATH='/usr/bin')
        source.update(retained)
        self.assertEqual(self.package.offline_environment(source), retained)
        self.assertEqual(source['BLOG_DATA_DIR'], 'pollution')

    def test_installer_rejects_writable_or_unowned_parent_chain(self):
        import os
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            writable = root / 'writable'
            writable.mkdir()
            writable.chmod(0o777)
            with self.assertRaises(ValueError):
                self.package.trusted_directory(writable)
            writable.chmod(0o770)
            with self.assertRaises(ValueError):
                self.package.trusted_directory(writable)
            target = root / 'safe'
            target.mkdir()
            link = root / 'link'
            link.symlink_to(target)
            # An unprivileged fixture can fail the root-ownership check before
            # reaching the symlink; both paths must reject it.
            with self.assertRaises((OSError, ValueError)):
                self.package.trusted_directory(link)
            if os.geteuid() == 0:
                writable.chmod(0o755)
                os.chown(writable, 65534, 65534)
                with self.assertRaises(ValueError):
                    self.package.trusted_directory(writable)

    def test_duplicate_json_and_traversal_paths_are_rejected(self):
        with self.assertRaises(ValueError):
            self.package.loads(b'{"product":"blog","product":"forged"}')
        for name in ('../bin/blog', '/bin/blog', 'bin/../blog', 'bin//blog', 'state/site.sqlite'):
            with self.assertRaises(ValueError):
                self.package.relative(name)

    def test_staging_accepts_stable_source_and_rejects_material_copy_changes(self):
        spec = importlib.util.spec_from_file_location('blog_stager', ROOT / 'scripts/stage-package.py')
        stager = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(stager)
        previous_umask = os.umask(0o022)  # The stager's command-line entry sets this.
        self.addCleanup(os.umask, previous_umask)
        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary)
            payload = work / 'payload'
            payload.mkdir()
            self.fixture(payload)
            metadata = work / 'metadata'
            shutil.copytree(payload / 'share/build', metadata)
            (metadata / 'server.blog-assets.json').unlink()
            binary = work / 'server'
            shutil.copyfile(payload / 'bin/blog', binary)
            shutil.copyfile(payload / 'share/build/server.blog-assets.json',
                            Path(str(binary) + '.blog-assets.json'))
            source = work / 'source'
            for name in ('package.py', 'install.py', 'maintenance.py'):
                target = source / 'tools/packaging' / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / 'tools/packaging' / name, target)
            for origin in ('src/application/LICENSE', 'src/interface/LICENSE', 'web/styles/LICENSE',
                           'tools/assets/LICENSE', 'src/interface/dom/LICENSE',
                           'src/interface/macros/LICENSE', 'web/styles/web-fonts/CJK-LICENSE.txt',
                           'web/styles/web-fonts/NORMAL-LICENSE.txt', 'LICENSE',
                           'deploy/linux-x86_64/config.json.example', 'deploy/linux-x86_64/README.md',
                           *(f'docs/{name}.md' for name in
                             ('getting-started', 'configuration', 'usage', 'operations', 'contracts'))):
                target = source / origin
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / origin, target)
            manifest = source / 'src/server/Cargo.toml'
            manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text('[package]\nversion = "1.0.1"\n')
            original_license = (source / 'LICENSE').read_bytes()
            stamp = dict(source_revision='a' * 40, source_dirty=True, source_tree_sha256='c' * 64)

            def source_stamp():
                return {**stamp, 'source_tree_sha256':
                        'c' * 64 if (source / 'LICENSE').read_bytes() == original_license else 'f' * 64}

            def build_setup(_identity, output):
                output.mkdir(exist_ok=True)
                for name in ('blog-build-identity.json', 'blog-schema.json'):
                    shutil.copyfile(metadata / name, output / name)
                return binary

            stage_notices = release.stage_original_notices

            def change_material(destination, *, source):
                stage_notices(destination, source=source)
                (source / 'LICENSE').write_bytes(original_license + b'changed while staging')

            with patch.object(stager, 'SOURCE', source), \
                    patch.object(stager, 'current_source_stamp', side_effect=source_stamp), \
                    patch.object(stager, 'build_setup', side_effect=build_setup), \
                    patch.object(stager, 'command', return_value='1700000000\n'), \
                    patch.object(stager, 'module', return_value=release):
                archive = stager.build(binary, metadata, work / 'stable-output')
                with tarfile.open(archive) as packed:
                    prefix = 'blog-1.0.1-linux-x86_64/share/'
                    for name in ('README.md', 'docs/configuration.md', 'docs/operations.md'):
                        self.assertIn(prefix + name, packed.getnames())
                # The real payload is copied; only Cargo and Git commands are
                # substituted. The changed legal input must invalidate staging.
                with patch.object(release, 'stage_original_notices', side_effect=change_material):
                    with self.assertRaisesRegex(ValueError, 'source changed while staging'):
                        stager.build(binary, metadata, work / 'output')
            self.assertFalse(list((work / 'output').glob('*.tar.gz')))
            self.assertFalse((work / 'output/SHA256SUMS').exists())


if __name__ == '__main__':
    unittest.main()
