#!/usr/bin/env python3
"""Build Blog-owned setup and package an exact, independently verified server."""
import argparse
import gzip
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
sys.dont_write_bytecode = True
SOURCE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SOURCE / 'tools/packaging'))
import package

TARGET = 'x86_64-unknown-linux-gnu'


def command(arguments, *, env=None):
    return subprocess.check_output(arguments, cwd=SOURCE, env=env, text=True)


def module(path):
    spec = importlib.util.spec_from_file_location('blog_release', path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def write(path, value, mode=0o644):
    path.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    with path.open('xb') as stream:
        stream.write(value)
    path.chmod(mode)


def build_setup(identity, metadata):
    environment = os.environ.copy()
    environment.update(BLOG_SOURCE_REVISION=identity['source_revision'],
                       BLOG_BUILD_METADATA_DIR=str(metadata),
                       BLOG_WEB_DIST=str(SOURCE / 'web/dist'),
                       BLOG_BUILD_MODE=identity['build_mode'],
                       BLOG_SOURCE_DIRTY=str(identity['source_dirty']).lower(),
                       BLOG_SOURCE_TREE_SHA256=identity['source_tree_sha256'])
    result = command(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--release', '--target', TARGET,
                      '-p', 'blog-setup', '--message-format=json-render-diagnostics'], env=environment)
    executables = []
    for line in result.splitlines():
        event = json.loads(line)
        if event.get('reason') == 'compiler-artifact' and event.get('target', {}).get('name') == 'blog-setup' and event.get('executable'):
            executables.append(Path(event['executable']))
    if len(executables) != 1:
        raise ValueError('Cargo did not return exactly one Blog setup executable')
    return executables[0]


def current_source_stamp():
    # Kept in the Rust builder, which owns the source identity algorithm.
    return json.loads(command(['cargo', '+1.99.0', 'run', '--quiet', '--locked', '--offline', '-p', 'blog-assets',
                               '--manifest-path', str(SOURCE / 'Cargo.toml'),
                               '--', 'source-stamp', '--product-root', str(SOURCE)]))


def build(binary, metadata, output, *, kind='candidate'):
    for directory in (SOURCE, metadata, output.parent):
        package.physical_directory(directory)
    if not binary.is_absolute() or not metadata.is_absolute() or not output.is_absolute():
        raise ValueError('all candidate paths must be absolute')
    if output.exists() or output.is_symlink() or output == SOURCE or SOURCE in output.parents or output == metadata or metadata in output.parents:
        raise ValueError('output must be a fresh directory outside the source and metadata')
    receipt_file = Path(str(binary) + '.blog-assets.json')
    receipt = package.loads(package.read_regular(receipt_file))
    if receipt.get('format') != 'blog-assets-receipt-v1':
        raise ValueError('server receipt format differs')
    identity = receipt['identity']
    if (identity.get('build_mode') != kind or type(identity.get('source_dirty')) is not bool
            or (kind == 'release' and identity['source_dirty'])):
        raise ValueError('candidate and formal release build identities must remain distinct')
    expected = {key: identity[key] for key in ('product', 'version', 'source_revision', 'target')}
    if expected['product'] != 'blog' or expected['target'] != TARGET:
        raise ValueError('server does not identify Blog on the supported target')
    version = tomllib.loads((SOURCE / 'src/server/Cargo.toml').read_text())['package']['version']
    if version != expected['version']:
        raise ValueError('server version differs from the current source')
    before = current_source_stamp()
    if any(identity.get(key) != before.get(key) for key in ('source_revision', 'source_dirty', 'source_tree_sha256')):
        raise ValueError('server receipt differs from the exact current source tree')
    command(['cargo', '+1.99.0', 'run', '--quiet', '--locked', '--offline', '-p', 'blog-assets', '--manifest-path',
             str(SOURCE / 'Cargo.toml'), '--', 'verify', '--product-root', str(SOURCE),
             '--binary', str(binary), '--receipt', str(receipt_file), '--dist', str(SOURCE / 'web/dist')])
    material = {}
    for path in sorted(metadata.rglob('*')):
        if path.is_dir() and not path.is_symlink():
            package.physical_directory(path)
            continue
        name = path.relative_to(metadata).as_posix()
        package.relative('share/build/' + name)
        material[name] = package.read_regular(path)
    required = ('blog-identity.json', 'blog-build-identity.json', 'blog-schema.json',
                'blog-state-contract.json', 'blog-browser-assets.json', 'contracts/install.json', 'frontend.json')
    if any(name not in material for name in required):
        raise ValueError('exact native/frontend installation metadata is incomplete')
    native = package.loads(material['blog-identity.json'])
    if any(native.get(key) != value for key, value in expected.items()):
        raise ValueError('native installation identity differs from the exact server')
    release = module(SOURCE / 'scripts/package-release.py')
    canonical_identity = material['blog-build-identity.json']
    release.require_compiled_identity(package.read_regular(binary, 1024 * 1024 * 1024, allow_hardlinks=True),
                                      identity, canonical_identity, receipt['identity_sha256'])
    if package.loads(canonical_identity) != identity:
        raise ValueError('native build identity differs from the server receipt')
    if package.loads(material['blog-schema.json']).get('schema_sha256') != identity['schema_sha256']:
        raise ValueError('native current schema differs from the identity bound to the server')
    if package.loads(material['blog-browser-assets.json']) != receipt['inventory']:
        raise ValueError('native asset inventory differs from the server receipt')
    state_contract = hashlib.sha256(material['blog-state-contract.json']).hexdigest()
    if native.get('state_contract_sha256') != state_contract:
        raise ValueError('installation state contract digest differs')
    output.mkdir(mode=0o700)
    with tempfile.TemporaryDirectory(prefix='.blog-package-', dir=output) as temporary:
        work = Path(temporary)
        setup_metadata = work / 'setup-metadata'
        setup_metadata.mkdir(mode=0o700)
        setup = build_setup(identity, setup_metadata)
        if before != current_source_stamp():
            raise ValueError('source changed while building Blog setup')
        setup_identity = package.read_regular(setup_metadata / 'blog-build-identity.json')
        if setup_identity != canonical_identity:
            raise ValueError('Blog setup was compiled from a different build identity')
        if package.read_regular(setup_metadata / 'blog-schema.json') != material['blog-schema.json']:
            raise ValueError('Blog setup was compiled with a different current schema')
        setup_bytes = package.read_regular(setup, 1024 * 1024 * 1024, allow_hardlinks=True)
        release.require_compiled_identity(setup_bytes, identity, setup_identity, receipt['identity_sha256'])
        name = f'blog-{version}-linux-x86_64'
        stage = work / name
        for directory in (stage, stage / 'bin', stage / 'libexec', stage / 'share'):
            directory.mkdir(mode=0o755)
            directory.chmod(0o755)
        release.stage_verified_server(binary, stage / 'bin/blog', identity['source_revision'])
        release.stage_original_notices(stage / 'share', source=SOURCE)
        # Cargo hardlinks named executables to deps artifacts; copy bytes into
        # exclusive package files, then verify the complete single-link payload.
        write(stage / 'libexec/blog-setup', setup_bytes, 0o755)
        for name, value in material.items():
            write(stage / 'share/build' / name, value)
        write(stage / 'share/build/server.blog-assets.json', package.read_regular(receipt_file))
        for name in ('package.py', 'install.py', 'maintenance.py'):
            write(stage / 'libexec' / name, package.read_regular(SOURCE / 'tools/packaging' / name))
        for name, program in (('install.sh', 'install.py'), ('maintenance.sh', 'maintenance.py')):
            value = ('#!/usr/bin/env bash\nset -euo pipefail\nroot="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"\n'
                     'exec python3 -I -B "$root/libexec/' + program + '" "$@"\n')
            write(stage / name, value.encode(), 0o755)
        write(stage / 'share/config/config.json.example', package.read_regular(SOURCE / 'deploy/linux-x86_64/config.json.example'))
        write(stage / 'share/README.md', package.read_regular(SOURCE / 'deploy/linux-x86_64/README.md'))
        for directory in ('compliance',):
            for path in sorted((SOURCE / directory).rglob('*')):
                if path.is_file():
                    write(stage / 'share' / directory / path.relative_to(SOURCE / directory), package.read_regular(path))
        for origin in ('src/application/LICENSE', 'src/interface/LICENSE', 'web/styles/LICENSE',
                       'tools/assets/LICENSE', 'src/interface/dom/LICENSE',
                       'src/interface/macros/LICENSE', 'web/styles/web-fonts/CJK-LICENSE.txt',
                       'web/styles/web-fonts/NORMAL-LICENSE.txt'):
            write(stage / 'share/licenses' / origin, package.read_regular(SOURCE / origin))
        for path in sorted((SOURCE / 'web/licenses').glob('*')):
            if path.is_file():
                write(stage / 'share/licenses/web/licenses' / path.name, package.read_regular(path))
        package.write_manifest(stage, native, kind, identity['source_tree_sha256'], identity['source_dirty'])
        archive = output / f'blog-{version}-linux-x86_64.tar.gz'
        epoch = int(command(['git', 'show', '-s', '--format=%ct', identity['source_revision']]).strip())
        with archive.open('xb') as raw, gzip.GzipFile(filename='', fileobj=raw, mode='wb', mtime=epoch) as compressed:
            with tarfile.open(fileobj=compressed, mode='w', format=tarfile.PAX_FORMAT) as tar:
                for path in [stage, *sorted(stage.rglob('*'))]:
                    entry = tar.gettarinfo(str(path), arcname=str(path.relative_to(work)))
                    entry.uid = entry.gid = 0
                    entry.uname = entry.gname = ''
                    entry.mtime = epoch
                    if entry.isfile():
                        with path.open('rb') as stream:
                            tar.addfile(entry, stream)
                    else:
                        tar.addfile(entry)
        write(output / 'SHA256SUMS', (hashlib.sha256(package.read_regular(archive, package.MAX_BYTES)).hexdigest()
                                     + '  ' + archive.name + '\n').encode('ascii'))
    return archive


def main():
    os.umask(0o022)
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('binary', 'metadata', 'output'):
        parser.add_argument(name, type=Path)
    parser.add_argument('--kind', choices=('candidate', 'release'), default='candidate')
    args = parser.parse_args()
    print(build(args.binary, args.metadata, args.output, kind=args.kind))


if __name__ == '__main__':
    main()
