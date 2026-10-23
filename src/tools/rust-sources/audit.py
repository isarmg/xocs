#!/usr/bin/env python3
"""Audit maintained Rust source using its preserved upstream registry identities.

This online security check is separate from the product's offline Cargo graph.
Upstream advisories remain relevant to local forks until reviewed fix evidence
shows otherwise; a changed name or local path does not suppress the identity.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]
ORIGINS = 'compliance/provenance/origins.json'
CATALOG = 'compliance/rust-sources.json'
AUDIT_VERSION = '0.22.2'
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'
HEX = re.compile(r'[0-9a-f]{64}')
NAME = re.compile(r'[A-Za-z0-9_-]+')
VERSION = re.compile(r'[0-9]+\.[0-9]+\.[0-9]+(?:-[A-Za-z0-9.-]+)?(?:\+[A-Za-z0-9.-]+)?')


def physical_path(root, name):
    path = PurePosixPath(name)
    if path.is_absolute() or not path.parts or any(part in ('', '.', '..') for part in path.parts):
        raise ValueError(f'unsafe source path: {name}')
    current = root
    for part in path.parts:
        current /= part
        if current.is_symlink():
            raise ValueError(f'symlink in source path: {name}')
    current.resolve().relative_to(root.resolve())
    return current


def identities(root):
    document = json.loads(physical_path(root, ORIGINS).read_text())
    current = json.loads(physical_path(root, CATALOG).read_text())
    if document.get('schema') != 1 or current.get('schema') != 1 or not isinstance(current.get('packages'), dict):
        raise ValueError('unsupported original/current source schema')
    origins = {}
    for origin in document['packages']:
        key = origin.get('id', origin['name'] + '@' + origin['version'])
        if key in origins:
            raise ValueError(f'duplicate original identity: {key}')
        origins[key] = origin
    active = set()
    covered = set(current['packages'])
    for directory, package in current['packages'].items():
        source = physical_path(root, directory)
        if package.get('directory') != directory:
            raise ValueError(f'current source directory differs from catalog key: {directory}')
        manifest = tomllib.loads((source / 'Cargo.toml').read_text())['package']
        name, version = manifest['name'], manifest.get('version', '0.0.0')
        if isinstance(version, dict) and version.get('workspace') is True:
            version = tomllib.loads(physical_path(root, 'Cargo.toml').read_text())['workspace']['package']['version']
        if (name, version) != (package['name'], package['version']):
            raise ValueError(f'current package identity differs from current receipt: {directory}')
        lineage = package.get('origins')
        if lineage is None:
            lineage = [package['origin']] if package.get('origin') else []
            if directory.startswith('src/foundation/') and not lineage:
                raise ValueError(f'maintained source lacks an audit identity: {directory}')
        if (not isinstance(lineage, list) or any(not isinstance(item, str) for item in lineage)
                or len(lineage) != len(set(lineage)) or any(item not in origins for item in lineage)):
            raise ValueError(f'current source has unknown or duplicate original identities: {directory}')
        active.update(lineage)
    # A source receipt may map one merged package to many registry identities.
    # Historical origin directories are never used as activity indicators.
    foundation = physical_path(root, 'src/foundation')
    for parent, children, names in os.walk(foundation, followlinks=False):
        parent = Path(parent)
        directory = parent.relative_to(root).as_posix()
        if directory in covered:
            children[:] = []  # Source fixtures and package output are not roots.
            continue
        if 'Cargo.toml' in names:
            raise ValueError(f'maintained source lacks an audit identity: {directory}')
        for child in children:
            if (parent / child).is_symlink():
                raise ValueError(f'symlink in source path: {directory}/{child}')
    records = {}
    for key in sorted(active):
        origin = origins[key]
        upstream = origin['upstream']
        name, version, checksum = upstream['name'], upstream['version'], upstream['archive_sha256']
        if not NAME.fullmatch(name) or not VERSION.fullmatch(version) or not HEX.fullmatch(checksum):
            raise ValueError(f'invalid upstream registry identity: {key}')
        archive_name = upstream['archive']
        if not archive_name.startswith('compliance/provenance/archives/') or not archive_name.endswith('.crate'):
            raise ValueError(f'invalid original archive path: {key}')
        archive = physical_path(root, archive_name)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != checksum:
            raise ValueError(f'original archive checksum differs: {key}')
        identity = (name, version)
        if identity in records and records[identity] != checksum:
            raise ValueError(f'conflicting original archive identity: {name} {version}')
        records[identity] = checksum
    if not records:
        raise ValueError('no maintained upstream registry identities to audit')
    return records


def audit_lock(records):
    lines = ['# Temporary upstream-identity audit projection; not a product lockfile.', 'version = 4', '']
    for (name, version), checksum in sorted(records.items()):
        lines += ['[[package]]', f'name = {json.dumps(name)}', f'version = {json.dumps(version)}',
                  f'source = {json.dumps(REGISTRY)}', f'checksum = {json.dumps(checksum)}', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    try:
        # The source manager binds current identities to immutable provenance.
        subprocess.run([sys.executable, '-I', '-B', str(root / 'src/tools/rust-sources/manage.py'),
                        'verify', '--root', str(root)], check=True, cwd=root)
        records = identities(root)
        with tempfile.TemporaryDirectory(prefix='blog-rust-audit-') as temporary:
            directory = Path(temporary)
            lock = directory / 'Cargo.lock'
            lock.write_text(audit_lock(records))
            version = subprocess.check_output(['cargo-audit', '--version'], text=True, cwd=directory).strip()
            if version != f'cargo-audit {AUDIT_VERSION}':
                raise ValueError(f'expected cargo-audit {AUDIT_VERSION}, got {version!r}')
            environment = os.environ.copy()
            environment['CARGO_NET_OFFLINE'] = 'false'
            print(f'Auditing {len(records)} original registry identities; local changes do not imply advisory fixes.', flush=True)
            subprocess.run(['cargo', 'audit', '--file', str(lock), '--deny', 'yanked'],
                           check=True, cwd=directory, env=environment)
    except (ValueError, OSError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        print(f'Rust upstream-identity audit failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
