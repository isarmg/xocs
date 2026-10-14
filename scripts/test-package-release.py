#!/usr/bin/env python3
"""The formal and candidate path share exact binary staging, not two packagers."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

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
            receipt = {'format': 'xssi-web-build-receipt-v1',
                       'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                       'identity': {'source_revision': 'a' * 40, 'product': 'xocs',
                                    'target': 'x86_64-unknown-linux-gnu'}}
            path = Path(str(binary) + '.xssi-web-build.json')
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


if __name__ == '__main__':
    unittest.main()
