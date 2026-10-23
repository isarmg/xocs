#!/usr/bin/env python3
"""Regression checks for documentation paths, anchors and build inputs."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('check_docs', (Path(__file__).resolve().parents[2] / 'src/tools/check-docs.py'))
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)


class DocumentationTests(unittest.TestCase):
    def test_code_and_comments_are_not_links(self):
        source = '''[good](ok.md)
`[inline](missing.md)`
````markdown
[example](missing.md)
```
````
~~~
[fenced](missing.md)
~~~
<!-- [comment](missing.md) -->
'''
        self.assertEqual(check.links(source), [(1, 'ok.md')])

    def test_balanced_paths_titles_and_references(self):
        source = '''[nested](image(a).svg "Diagram")
[angle](<with space.md>)
[ref][ A   B ] and [shortcut]
[A B]: target.md
[shortcut]: other.md "Title"
'''
        self.assertEqual(sorted(target for _, target in check.links(source)),
                         ['image(a).svg', 'other.md', 'target.md', 'with space.md'])

    def test_github_anchors_code_unicode_duplicates_and_setext(self):
        source = '''# Hello, `Rust`!
## 中文 API / 路径
## Hello, Rust!
A setext title
----
<a name="explicit"></a>
```
# not-a-heading
```
'''
        self.assertEqual(check.anchors(source), {'hello-rust', 'hello-rust-1', '中文-api--路径', 'a-setext-title', 'explicit'})

    def test_heading_suffixes_do_not_collide_with_other_headings(self):
        source = '# Entry\n# Entry-1\n# Entry\n# Entry\n# Entry-1\n'
        self.assertEqual(check.anchors(source),
                         {'entry', 'entry-1', 'entry-2', 'entry-3', 'entry-1-1'})

    def test_example_anchors_and_invalid_fence_closers_are_hidden(self):
        source = '''# Visible
<!-- <a name="comment"></a> -->
`<a name="inline"></a>`
````markdown
<a name="example"></a>
# Example `code`
````python
[example](missing.md)
````
[visible](real.md)
'''
        self.assertEqual(check.anchors(source), {'visible'})
        self.assertEqual(check.links(source), [(10, 'real.md')])

    def test_missing_files_and_fragments_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'README.md').write_text('[ok](#title) [file](missing.md) [anchor](#missing)\n# Title\n')
            errors, count = check.check_links(root)
            self.assertEqual(count, 3)
            self.assertEqual(len(errors), 2)
            self.assertTrue(any('target does not exist' in error for error in errors))
            self.assertTrue(any('heading does not exist' in error for error in errors))

    def test_relative_paths_percent_encoding_and_repository_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'docs').mkdir()
            (root / 'README.md').write_text('[ok](docs/space%20file.md#中文) [bad](../outside.md)\n')
            (root / 'docs/space file.md').write_text('# 中文\n[home](../README.md)\n')
            errors, count = check.check_links(root)
            self.assertEqual(count, 3)
            self.assertEqual(len(errors), 1)
            self.assertIn('escapes repository', errors[0])

    def test_directory_fragments_require_a_local_readme(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'repo'
            root.mkdir()
            (root / 'docs').mkdir()
            (root / 'README.md').write_text('[guide](docs#title)\n')
            errors, count = check.check_links(root)
            self.assertEqual(count, 1)
            self.assertEqual(len(errors), 1)
            self.assertIn('directory has no README.md', errors[0])

            outside = root.parent / 'outside.md'
            outside.write_text('# Title\n')
            (root / 'docs/README.md').symlink_to(outside)
            errors, _ = check.check_links(root)
            self.assertEqual(len(errors), 1)
            self.assertIn('escapes repository', errors[0])

    def test_only_original_prose_is_exempt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            origin = root / 'docs/dependencies/origins/original-1.0'
            origin.mkdir(parents=True)
            (root / 'README.md').write_text('')
            (origin / 'README.md').write_text('[old](missing.md)')
            (origin.parent / 'README.md').write_text('[index](missing.md)')
            errors, _ = check.check_links(root)
            self.assertEqual(len(errors), 1)
            self.assertIn('origins/README.md', errors[0])

    def test_rust_literals_ignore_comments_and_string_examples(self):
        source = r'''// include_str!("absent.md")
/* outer /* nested */ include_bytes!("absent.md") */
const EXAMPLE: &str = "include_str!(\"absent.md\")";
const RAW: &str = r##"include_str!("absent.md")"##;
const QUOTE: char = '"';
const ESCAPED: char = '\'';
const UNICODE: &str = "\u{1234} include_str!(\"absent.md\")";
#![doc = include_str!(r#"guide.md"#)]
const B: &[u8] = include_bytes!(
    "../other.md"
);
'''
        self.assertEqual([name for _, name in check.rust_includes(source)], ['guide.md', '../other.md'])

    def test_missing_documentation_build_inputs_fail_even_in_original_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'src/foundation/test'
            source.mkdir(parents=True)
            (source / 'lib.rs').write_text('#![doc = include_str!("missing.md")]\ninclude_bytes!("fixture.bin");\n')
            (source / 'Cargo.toml').write_text('[package]\nname = "test"\nreadme = "other.md"\n')
            errors, count = check.check_inputs(root)
            self.assertEqual(count, 2)
            self.assertEqual(len(errors), 2)


if __name__ == '__main__':
    unittest.main()
