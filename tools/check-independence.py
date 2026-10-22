#!/usr/bin/env python3
"""Verify one offline Cargo workspace whose resolved packages are local sources."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

SKIP_ROOTS = {'.git', 'target', 'compliance', '.local-run', 'web/node_modules', 'web/dist', 'web/pkg'}
ROOT = Path(__file__).resolve().parents[1]
WORKSPACES = [('Cargo.toml', target) for target in
              ('x86_64-unknown-linux-gnu', 'wasm32-unknown-unknown')]
DEPENDENCIES = ('dependencies', 'dev-dependencies', 'build-dependencies')


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def local_path(path: str | Path, context: str) -> Path:
    path = Path(path)
    require(path.resolve().is_relative_to(ROOT.resolve()),
            f'{context} escapes this checkout: {path}')
    for ancestor in (path, *path.parents):
        require(not ancestor.is_symlink(), f'{context} contains a symbolic link: {ancestor}')
    return path.resolve()


def manifests() -> list[Path]:
    result = []
    for directory, children, files in os.walk(ROOT):
        directory = Path(directory)
        if directory != ROOT:
            require('.git' not in children and '.git' not in files,
                    f'Nested Git repository is forbidden: {directory}')
        package_root = ('Cargo.toml' in files and
                        'package' in tomllib.loads((directory / 'Cargo.toml').read_text()))
        children[:] = sorted(child for child in children
                             if (directory / child).relative_to(ROOT).as_posix() not in SKIP_ROOTS
                             and not (package_root and child == 'target'))
        for child in children:
            require(not (directory / child).is_symlink(),
                    f'Symbolic source directory is forbidden: {directory / child}')
        if 'Cargo.lock' in files:
            require(directory == ROOT, f'Only the root Cargo.lock may be active: {directory}')
        if 'Cargo.toml' in files:
            result.append(directory / 'Cargo.toml')
    return result


def check_declarations() -> None:
    root_manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    require('workspace' in root_manifest, 'The root Cargo.toml must declare the sole workspace')
    workspace_dependencies = root_manifest['workspace'].get('dependencies', {})
    for path in manifests():
        local_path(path, 'Cargo manifest')
        manifest = tomllib.loads(path.read_text())
        require(path == ROOT / 'Cargo.toml' or 'workspace' not in manifest,
                f'Nested Cargo workspace is forbidden: {path}')
        package = manifest.get('package', {})
        require(not package.get('name', '').startswith('xcss'), f'Old dependency package remains: {path}')

        def declaration(name: str, value: object, *, inherited: bool = False) -> None:
            require(not name.startswith('xcss'), f'Old dependency declaration {name}: {path}')
            require(isinstance(value, dict), f'Registry dependency must use a local path: {path}: {name}')
            require('git' not in value and 'registry' not in value and 'registry-index' not in value,
                    f'External Git or registry dependency is forbidden: {path}: {name}')
            if value.get('workspace') is True:
                require(not inherited and name in workspace_dependencies,
                        f'Unknown inherited workspace dependency: {path}: {name}')
                declaration(name, workspace_dependencies[name], inherited=True)
                return
            require('path' in value, f'Dependency must use a local path: {path}: {name}')
            # Inherited declarations are relative to the root manifest.
            parent = ROOT if inherited else path.parent
            dependency = local_path(parent / value['path'], f'{path}: {name}')
            require((dependency / 'Cargo.toml').is_file(), f'Local dependency manifest is missing: {dependency}')

        def tables(table: dict, *, inherited: bool = False) -> None:
            for group in DEPENDENCIES:
                for name, value in table.get(group, {}).items():
                    declaration(name, value, inherited=inherited)

        tables(manifest)
        for target in manifest.get('target', {}).values():
            tables(target)
        if path == ROOT / 'Cargo.toml':
            tables(root_manifest['workspace'], inherited=True)
        for replacements in manifest.get('patch', {}).values():
            for name, value in replacements.items():
                declaration(name, value)
        for name, value in manifest.get('replace', {}).items():
            declaration(name, value)
    lock = ROOT / 'Cargo.lock'
    if lock.exists():
        local_path(lock, 'Cargo lockfile')
        for package in tomllib.loads(lock.read_text()).get('package', []):
            require('source' not in package and 'checksum' not in package,
                    f'Root lockfile contains an external source: {package.get("name")}')


def check_graph(manifest: str, target: str, offline: bool = True) -> dict:
    require(manifest == 'Cargo.toml', 'Only the root Cargo workspace may be resolved')
    command = ['cargo', '+1.99.0', 'metadata', '--offline', '--locked', '--format-version', '1',
               '--manifest-path', str(ROOT / manifest), '--filter-platform', target]
    completed = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, check=True)
    metadata = json.loads(completed.stdout)
    require(Path(metadata['workspace_root']).resolve() == ROOT.resolve(),
            f'{manifest}: resolved workspace differs from this checkout')
    for package in metadata['packages']:
        name = package['name']
        require(not name.startswith('xcss'), f'{manifest}: shared crate remains: {name}')
        require(package.get('source') is None,
                f'{manifest}: external source is forbidden: {name}: {package.get("source")}')
        local_path(package['manifest_path'], f'{manifest}: {name}')
        for target_entry in package.get('targets', []):
            local_path(target_entry['src_path'], f'{manifest}: {name} target {target_entry["name"]}')
    return {'manifest': manifest, 'target': target, 'packages': len(metadata['packages'])}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--offline', action='store_true',
                        help='Compatibility flag; Cargo metadata is always locked and offline')
    parser.parse_args()
    check_declarations()
    graphs = [check_graph(manifest, target) for manifest, target in WORKSPACES]
    print(json.dumps({'independent': True, 'single_workspace': True, 'single_lockfile': True,
                      'registry_dependencies_allowed': False, 'graphs': graphs}, indent=2))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f'check-independence: {error}', file=sys.stderr)
        raise SystemExit(1)
