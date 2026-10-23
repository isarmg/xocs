#!/usr/bin/env python3
"""Keep maintained production files in src/ and test entrypoints in test/."""
from __future__ import annotations

import os
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
GENERATED = {'target', 'node_modules', '__pycache__'}
TEST_NAME = re.compile(r'(?:tests?\.rs$|test[_-]|.*\.(?:test|spec)\.)')
TEST_ATTRIBUTE = re.compile(r'^[ \t]*#\[\s*(?:[A-Za-z_]\w*::)*test\b', re.M)


def check(root: Path) -> list[str]:
    root = root.resolve()
    errors = []
    source = root / 'src'
    tests = root / 'test'
    for directory in (source, tests):
        if not directory.is_dir() or directory.is_symlink():
            errors.append(f'{directory.name}/: a regular maintained directory is required')
    for old in ('tools', 'web', 'scripts'):
        if (root / old).exists():
            errors.append(f'{old}/: maintained source and tests must live in src/ or test/')
    for current, children, names in os.walk(source, followlinks=False):
        parent = Path(current)
        children[:] = sorted(child for child in children
                             if child not in GENERATED and
                             (parent / child).relative_to(root).as_posix() not in
                             {'src/web/pkg', 'src/web/dist'})
        for child in children:
            path = parent / child
            if path.is_symlink():
                errors.append(f'{path.relative_to(root)}: source directories must not be symbolic links')
            if child in {'test', 'tests', 'benches'}:
                errors.append(f'{path.relative_to(root)}: test directories belong under test/')
        for name in names:
            path = parent / name
            if TEST_NAME.match(name):
                errors.append(f'{path.relative_to(root)}: test files belong under test/')
            if path.suffix == '.rs':
                text = path.read_text()
                for match in TEST_ATTRIBUTE.finditer(text):
                    line = text.count('\n', 0, match.start()) + 1
                    errors.append(f'{path.relative_to(root)}:{line}: test implementations belong under test/')
            if name != 'Cargo.toml':
                continue
            document = tomllib.loads(path.read_text())
            for kind in ('test', 'bench'):
                for target in document.get(kind, []):
                    target_path = target.get('path')
                    label = f'{path.relative_to(root)}: {kind} {target.get("name", "<unnamed>")}'
                    if not isinstance(target_path, str):
                        errors.append(f'{label}: declare an explicit path under test/')
                        continue
                    entry = (parent / target_path).resolve()
                    if not entry.is_relative_to(tests):
                        errors.append(f'{label}: entrypoint is outside test/: {target_path}')
                    elif not entry.is_file():
                        errors.append(f'{label}: entrypoint does not exist: {target_path}')
    return errors


def main() -> int:
    errors = check(ROOT)
    for error in errors:
        print(f'check-layout: {error}', file=sys.stderr)
    if errors:
        return 1
    print('Layout: production source is in src/; test files and Cargo test entrypoints are in test/.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
