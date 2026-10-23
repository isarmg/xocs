#!/usr/bin/env python3
"""Test the original-identity projection without network or advisory claims."""
from contextlib import redirect_stderr, redirect_stdout
import io
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('audit_projection', (Path(__file__).resolve().parents[3] / 'src/tools/rust-sources' / 'audit.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class AuditProjection(unittest.TestCase):
    def fixture(self, root):
        directory = root / 'src/foundation/local-name'
        directory.mkdir(parents=True)
        (directory / 'Cargo.toml').write_text('[package]\nname="maintained-name"\nversion="1.2.3"\n')
        archive = root / 'compliance/provenance/archives/upstream-1.2.3.crate'
        archive.parent.mkdir(parents=True)
        archive.write_bytes(b'preserved original archive')
        origin = dict(id='upstream-name@1.2.3', directory='src/foundation/original-old-directory', name='upstream-name', version='1.2.3',
                      upstream=dict(name='upstream-name', version='1.2.3', archive=archive.relative_to(root).as_posix(),
                                    archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest()))
        path = root / audit.ORIGINS
        path.write_text(json.dumps(dict(schema=1, packages=[origin])))
        (root / audit.CATALOG).write_text(json.dumps(dict(schema=1, packages={'src/foundation/local-name': dict(directory='src/foundation/local-name', name='maintained-name', version='1.2.3', origin='upstream-name@1.2.3')})))
        return path, archive, origin

    def test_original_name_and_checksum_survive_local_rename(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            _, _, origin = self.fixture(root)
            records = audit.identities(root)
            data = tomllib.loads(audit.audit_lock(records))
            self.assertEqual(len(data['package']), 1)
            self.assertEqual(data['package'][0], dict(name='upstream-name', version='1.2.3', source=audit.REGISTRY,
                                                    checksum=origin['upstream']['archive_sha256']))
            self.assertNotIn('maintained-name', audit.audit_lock(records))

    def test_unrecorded_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.fixture(root)
            extra = root / 'src/foundation/unrecorded'
            extra.mkdir()
            (extra / 'Cargo.toml').write_text('[package]\nname="unrecorded"\nversion="1.0.0"\n')
            with self.assertRaisesRegex(ValueError, 'lacks an audit identity'):
                audit.identities(root)

    def test_archive_tampering_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            _, archive, _ = self.fixture(root)
            archive.write_bytes(b'tampered')
            with self.assertRaisesRegex(ValueError, 'checksum differs'):
                audit.identities(root)

    def test_active_identity_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.fixture(root)
            path = root / audit.CATALOG
            current = json.loads(path.read_text())
            current['packages']['src/foundation/local-name']['version'] = '9.9.9'
            path.write_text(json.dumps(current))
            with self.assertRaisesRegex(ValueError, 'current package identity differs'):
                audit.identities(root)

    def test_nested_merged_source_audits_every_original_identity(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            path, archive, origin = self.fixture(root)
            destination = root / 'src/foundation/network/message'
            destination.parent.mkdir(parents=True)
            (root / 'src/foundation/local-name').rename(destination)
            (destination / 'Cargo.toml').write_text('[package]\nname="blog-message"\npublish=false\n')
            second_archive = archive.with_name('second-2.0.0.crate')
            second_archive.write_bytes(b'second original archive')
            second = dict(id='second@2.0.0', directory='src/foundation/second-2.0.0', name='second', version='2.0.0', upstream=dict(name='second', version='2.0.0', archive=second_archive.relative_to(root).as_posix(), archive_sha256=hashlib.sha256(second_archive.read_bytes()).hexdigest()))
            path.write_text(json.dumps(dict(schema=1, packages=[origin, second])))
            current = dict(schema=1, packages={'src/foundation/network/message': dict(directory='src/foundation/network/message', name='blog-message', version='0.0.0', origins=['second@2.0.0', 'upstream-name@1.2.3'], origin=None)})
            (root / audit.CATALOG).write_text(json.dumps(current))
            self.assertEqual(set(audit.identities(root)), {('upstream-name', '1.2.3'), ('second', '2.0.0')})

    def test_unknown_origin_mapping_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.fixture(root)
            path = root / audit.CATALOG
            current = json.loads(path.read_text())
            current['packages']['src/foundation/local-name']['origins'] = ['missing@1.0.0']
            path.write_text(json.dumps(current))
            with self.assertRaisesRegex(ValueError, 'unknown or duplicate'):
                audit.identities(root)

    def test_symlink_archive_cannot_replace_evidence(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            _, archive, _ = self.fixture(root)
            original = archive.with_suffix('.saved')
            archive.rename(original)
            archive.symlink_to(original)
            with self.assertRaisesRegex(ValueError, 'symlink'):
                audit.identities(root)


    def test_main_runs_online_audit_with_yank_denial(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.fixture(root)
            with patch('sys.argv', ['audit.py', '--root', str(root)]), \
                    patch.object(audit.subprocess, 'run') as run, \
                    patch.object(audit.subprocess, 'check_output', return_value='cargo-audit 0.22.2\n'), \
                    redirect_stdout(io.StringIO()):
                self.assertEqual(audit.main(), 0)
            self.assertEqual(run.call_count, 2)
            command, options = run.call_args
            self.assertEqual(command[0][:3], ['cargo', 'audit', '--file'])
            self.assertEqual(command[0][-2:], ['--deny', 'yanked'])
            self.assertTrue(options['check'])
            self.assertEqual(options['env']['CARGO_NET_OFFLINE'], 'false')

    def test_audit_failure_remains_failure(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.fixture(root)
            with patch('sys.argv', ['audit.py', '--root', str(root)]), \
                    patch.object(audit.subprocess, 'run', side_effect=[None, audit.subprocess.CalledProcessError(7, ['cargo', 'audit'])]), \
                    patch.object(audit.subprocess, 'check_output', return_value='cargo-audit 0.22.2\n'), \
                    redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                self.assertEqual(audit.main(), 1)


if __name__ == '__main__':
    unittest.main()
