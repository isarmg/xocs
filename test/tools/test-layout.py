#!/usr/bin/env python3
"""Exercise source/test layout validation without depending on this checkout."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('check_layout', Path(__file__).resolve().parents[1] / 'check-layout.py')
layout = importlib.util.module_from_spec(spec)
spec.loader.exec_module(layout)


class LayoutTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='blog-layout-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / 'test').mkdir()
        self.write('src/component/Cargo.toml', '[package]\nname="component"\nversion="0.0.0"\n')
        self.write('src/component/src/lib.rs', 'pub fn business() {}\n')

    def write(self, name, value):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value)
        return path

    def manifest(self, value):
        with (self.root / 'src/component/Cargo.toml').open('a') as stream:
            stream.write(value)

    def test_central_tests_and_generated_outputs_are_accepted(self):
        self.write('test/component/integration/contract.rs', '#[test] fn contract() {}\n')
        self.manifest('[[test]]\nname="contract"\npath="../../test/component/integration/contract.rs"\n')
        self.write('src/web/node_modules/dependency/tests/spec.test.mjs', 'generated dependency')
        self.write('src/web/pkg/generated.test.mjs', 'generated bindings')
        self.write('src/web/dist/test-bundle.js', 'generated bundle')
        self.write('src/component/src/lib.rs',
                   '#[cfg(test)]\n#[path="../../../test/component/unit.rs"]\nmod tests;\n'
                   '/// A documented #[test] example is prose.\n')
        self.assertEqual(layout.check(self.root), [])

    def test_inline_tests_are_rejected_even_in_business_named_files(self):
        self.write('src/component/src/business.rs',
                   '#[test]\nfn plain() {}\n'
                   '#[tokio::test]\nasync fn namespaced() {}\n'
                   '#[some::runtime::test(flavor = "current_thread")]\nasync fn configured() {}\n')
        errors = layout.check(self.root)
        self.assertEqual(len(errors), 3)
        self.assertTrue(all('test implementations belong under test/' in error for error in errors))

    def test_test_directories_and_test_filenames_are_rejected(self):
        self.write('src/component/tests/contract.rs', '')
        self.write('src/component/src/test_helpers.rs', '')
        self.write('src/component/src/runtime.test.mjs', '')
        errors = layout.check(self.root)
        self.assertEqual(len(errors), 3)
        self.assertTrue(any('test directories belong' in error for error in errors))

    def test_unmigrated_roots_are_rejected(self):
        self.write('tools/source.py', '')
        self.assertTrue(any('tools/:' in error for error in layout.check(self.root)))

    def test_missing_implicit_and_outside_test_entrypoints_are_rejected(self):
        self.manifest('[[test]]\nname="implicit"\n[[test]]\nname="missing"\npath="../../test/missing.rs"\n'
                      '[[bench]]\nname="outside"\npath="src/lib.rs"\n')
        errors = layout.check(self.root)
        self.assertEqual(len(errors), 3)
        self.assertTrue(any('declare an explicit path' in error for error in errors))
        self.assertTrue(any('entrypoint does not exist' in error for error in errors))
        self.assertTrue(any('entrypoint is outside' in error for error in errors))

    def test_symbolic_entrypoint_cannot_escape_test_root(self):
        self.write('test/component/inside.rs', '')
        (self.root / 'test/component/escape.rs').symlink_to(self.root / 'src/component/src/lib.rs')
        self.manifest('[[test]]\nname="escape"\npath="../../test/component/escape.rs"\n')
        self.assertTrue(any('entrypoint is outside' in error for error in layout.check(self.root)))


if __name__ == '__main__':
    unittest.main()
