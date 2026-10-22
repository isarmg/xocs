#!/usr/bin/env python3
"""Blog package integrity checks shared by staging and installed tooling."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat

MANIFEST = 'share/package.json'
CHECKSUM = 'share/package.sha256'
MAX_FILES = 8192
MAX_BYTES = 2 * 1024 * 1024 * 1024


RUNTIME_OVERRIDE_ENV = (
    'BLOG_DATA_DIR', 'BLOG_LOG_LEVEL', 'BLOG_MEDIA', 'BLOG_BIND',
    'BLOG_DEVELOPMENT', 'BLOG_DEV_WEB_DIR',
)


def offline_environment(source=None):
    # Offline operations name one explicit root/configuration. Clear only the
    # runtime overrides; preserve build identity, toolchain and ordinary env.
    result = dict(os.environ if source is None else source)
    for name in RUNTIME_OVERRIDE_ENV:
        result.pop(name, None)
    return result


BUILD_IDENTITY_FIELDS = (
    "format", "product", "version", "source_revision", "target", "web_assets_sha256",
    "source_dirty", "source_tree_sha256", "build_mode", "schema_sha256", "core_source_sha256",
)


def canonical_build_identity(identity: dict) -> bytes:
    if not isinstance(identity, dict) or set(identity) != set(BUILD_IDENTITY_FIELDS):
        raise ValueError("unexpected compiled Blog identity fields")
    if (identity["format"] != "blog-build-v1" or identity["product"] != "blog"
            or identity["target"] != "x86_64-unknown-linux-gnu"
            or not isinstance(identity["version"], str)
            or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", identity["version"])
            or not isinstance(identity["source_revision"], str)
            or not re.fullmatch(r"[0-9a-f]{40}", identity["source_revision"])
            or type(identity["source_dirty"]) is not bool
            or identity["build_mode"] not in ("candidate", "release")
            or (identity["build_mode"] == "release" and identity["source_dirty"])):
        raise ValueError("invalid packaged Blog build identity")
    for name in ("web_assets_sha256", "source_tree_sha256", "schema_sha256", "core_source_sha256"):
        if not isinstance(identity[name], str) or not re.fullmatch(r"[0-9a-f]{64}", identity[name]):
            raise ValueError("invalid compiled Blog identity digest")
    ordered = {name: identity[name] for name in BUILD_IDENTITY_FIELDS}
    return json.dumps(ordered, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def require_compiled_identity(binary: bytes, identity: dict, encoded: bytes, expected_sha256: str) -> None:
    # This is the same canonical byte sequence retained by the startup checker.
    # A self-consistent sidecar alone does not prove which code was compiled.
    if encoded != canonical_build_identity(identity):
        raise ValueError("build identity metadata is not the compiled canonical JSON")
    actual = hashlib.sha256(encoded).hexdigest()
    if (actual != expected_sha256 or encoded not in binary
            or actual.encode("ascii") not in binary):
        raise ValueError("executable does not contain the exact compiled Blog identity and digest")


def loads(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('duplicate JSON field')
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=unique)


def open_directory(path, *, trusted=False):
    """Walk directory descriptors without following any ancestor link.

    A trusted chain is root owned and cannot be renamed by another user. A
    root-owned sticky directory (such as /tmp) may contain a root-owned next
    component: sticky semantics prevent another user from replacing that child.
    """
    path = Path(path)
    if not path.is_absolute() or '..' in path.parts:
        raise ValueError('a normalized absolute directory is required')
    descriptor = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        for part in (None, *path.parts[1:]):
            if part is not None:
                following = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC,
                                    dir_fd=descriptor)
                os.close(descriptor)
                descriptor = following
            info = os.fstat(descriptor)
            if trusted and (info.st_uid != 0 or
                            (info.st_mode & 0o022 and not info.st_mode & stat.S_ISVTX)):
                raise ValueError('installation ancestors must be root owned and protected from other writers')
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def physical_directory(path):
    descriptor = open_directory(path)
    os.close(descriptor)
    return Path(path)


def trusted_directory(path):
    descriptor = open_directory(path, trusted=True)
    os.close(descriptor)
    return Path(path)


def read_regular(path, maximum=16 * 1024 * 1024, *, allow_hardlinks=False):
    path = Path(path)
    parent = open_directory(path.parent)
    try:
        before = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or (before.st_nlink != 1 and not allow_hardlinks):
            raise ValueError('a single-link regular file is required')
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=parent)
    finally:
        os.close(parent)
    try:
        opened = os.fstat(descriptor)
        def identity(info):
            return (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
                    info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
        if (identity(before) != identity(opened) or not stat.S_ISREG(opened.st_mode)
                or (opened.st_nlink != 1 and not allow_hardlinks) or opened.st_size > maximum):
            raise ValueError('a stable single-link regular file is required')
        with os.fdopen(os.dup(descriptor), 'rb') as stream:
            value = stream.read(maximum + 1)
        if (len(value) > maximum or identity(os.fstat(descriptor)) != identity(opened)
                or identity(path.lstat()) != identity(opened)):
            raise ValueError('file changed while reading or exceeded its bound')
        return value
    finally:
        os.close(descriptor)


def relative(value):
    if not isinstance(value, str):
        raise ValueError('package path must be text')
    path = PurePosixPath(value)
    if (path.is_absolute() or str(path) != value or not path.parts
            or any(part in ('', '.', '..') for part in path.parts)
            or any(ord(char) < 32 or ord(char) == 127 for char in value)):
        raise ValueError('package path must be normalized and relative')
    if value in ('install.sh', 'maintenance.sh'):
        return value
    if path.parts[0] not in ('bin', 'libexec', 'share') or value in (MANIFEST, CHECKSUM):
        raise ValueError('unexpected package namespace')
    return value


def inventory(root):
    physical_directory(root)
    entries = []
    total = 0
    paths = []
    for child in root.iterdir():
        if child.name in ('config', 'state', 'media', 'logs', 'run', 'maintenance'):
            continue
        paths.append(child)
        if child.is_dir() and not child.is_symlink():
            paths.extend(child.rglob('*'))
    for path in sorted(paths):
        info = path.lstat()
        if stat.S_ISDIR(info.st_mode):
            if stat.S_IMODE(info.st_mode) != 0o755:
                raise ValueError('immutable package directories must be mode 0755')
            continue
        name = path.relative_to(root).as_posix()
        if name in (MANIFEST, CHECKSUM):
            continue
        relative(name)
        value = read_regular(path, MAX_BYTES)
        mode = stat.S_IMODE(info.st_mode)
        if mode not in (0o644, 0o755):
            raise ValueError('immutable package file mode is invalid')
        total += len(value)
        entries.append({'path': name, 'size': len(value), 'mode': mode,
                        'sha256': hashlib.sha256(value).hexdigest()})
        if len(entries) > MAX_FILES or total > MAX_BYTES:
            raise ValueError('package exceeds its inventory bound')
    return entries


def write_manifest(root, identity, kind, source_tree_sha256, source_dirty):
    manifest = {'format': 'blog-package-v1', 'kind': kind, 'identity': identity,
                'source_tree_sha256': source_tree_sha256, 'source_dirty': source_dirty,
                'files': inventory(root)}
    value = (json.dumps(manifest, sort_keys=True, indent=2) + '\n').encode()
    (root / MANIFEST).write_bytes(value)
    (root / MANIFEST).chmod(0o644)
    (root / CHECKSUM).write_text(hashlib.sha256(value).hexdigest() + '\n', encoding='ascii')
    (root / CHECKSUM).chmod(0o644)
    verify(root)


def verify_build_material(root, manifest):
    """Cross-check self-description against both actual compiled executables.

    This is not a publisher signature. The archive still needs an externally
    trusted checksum; these checks prevent relabelling an unchanged candidate
    artifact by editing only its adjacent metadata and manifest.
    """
    material = root / 'share/build'
    encoded = read_regular(material / 'blog-build-identity.json')
    identity = loads(encoded)
    if canonical_build_identity(identity) != encoded:
        raise ValueError('packaged build identity must use its compiled canonical JSON')
    receipt = loads(read_regular(material / 'server.blog-assets.json'))
    if (not isinstance(receipt, dict) or receipt.get('format') != 'blog-assets-receipt-v1'
            or receipt.get('identity') != identity
            or manifest['kind'] != identity.get('build_mode')
            or manifest['source_dirty'] != identity.get('source_dirty')
            or manifest['source_tree_sha256'] != identity.get('source_tree_sha256')):
        raise ValueError('package provenance differs from its compiled build identity')
    native = loads(read_regular(material / 'blog-identity.json'))
    if (native != manifest['identity'] or any(native.get(key) != identity.get(key)
            for key in ('product', 'version', 'source_revision', 'target'))):
        raise ValueError('package release identity differs from its compiled build identity')
    schema = loads(read_regular(material / 'blog-schema.json'))
    state_bytes = read_regular(material / 'blog-state-contract.json')
    state = loads(state_bytes)
    if (schema.get('application') != 'blog' or schema.get('schema_sha256') != identity.get('schema_sha256')
            or native.get('state_contract_sha256') != hashlib.sha256(state_bytes).hexdigest()
            or state.get('application') != identity.get('product')
            or state.get('application_version') != identity.get('version')
            or state.get('source_revision') != identity.get('source_revision')
            or state.get('schema', {}).get('sha256') != identity.get('schema_sha256')
            or state.get('schema', {}).get('revision') != schema.get('schema_revision')):
        raise ValueError('packaged schema or state contract differs from its compiled build identity')
    inventory_bytes = read_regular(material / 'blog-browser-assets.json')
    inventory = loads(inventory_bytes)
    inventory_sha256 = hashlib.sha256(inventory_bytes).hexdigest()
    if (inventory != receipt.get('inventory') or inventory_sha256 != receipt.get('inventory_sha256')
            or inventory_sha256 != identity.get('web_assets_sha256')):
        raise ValueError('packaged web inventory differs from its compiled build identity')
    server = read_regular(root / 'bin/blog', 1024 * 1024 * 1024)
    if hashlib.sha256(server).hexdigest() != receipt.get('binary_sha256'):
        raise ValueError('packaged server differs from its build receipt')
    for binary in (server, read_regular(root / 'libexec/blog-setup', 1024 * 1024 * 1024)):
        require_compiled_identity(binary, identity, encoded, receipt.get('identity_sha256'))


def verify(root):
    root = Path(root)
    value = read_regular(root / MANIFEST)
    expected = read_regular(root / CHECKSUM, 65)
    if expected != (hashlib.sha256(value).hexdigest() + '\n').encode('ascii'):
        raise ValueError('package manifest checksum differs')
    manifest = loads(value)
    if (not isinstance(manifest, dict) or manifest.get('format') != 'blog-package-v1'
            or manifest.get('kind') not in ('candidate', 'release')
            or type(manifest.get('source_dirty')) is not bool
            or not re.fullmatch(r'[0-9a-f]{64}', manifest.get('source_tree_sha256', ''))
            or not isinstance(manifest.get('identity'), dict)
            or manifest['identity'].get('product') != 'blog'
            or manifest['identity'].get('target') != 'x86_64-unknown-linux-gnu'
            or not re.fullmatch(r'[0-9a-f]{40}', manifest['identity'].get('source_revision', ''))):
        raise ValueError('invalid Blog package identity')
    if manifest['kind'] == 'release' and manifest['source_dirty']:
        raise ValueError('a formal release cannot contain dirty source')
    if manifest.get('files') != inventory(root):
        raise ValueError('package contents differ from the exact manifest')
    required = {'bin/blog', 'libexec/blog-setup', 'libexec/package.py',
                'libexec/install.py', 'libexec/maintenance.py',
                'install.sh', 'maintenance.sh', 'share/config/config.json.example',
                'share/build/blog-identity.json', 'share/build/blog-schema.json',
                'share/build/blog-state-contract.json', 'share/build/blog-browser-assets.json',
                'share/build/contracts/install.json', 'share/build/frontend.json',
                'share/build/blog-build-identity.json', 'share/build/server.blog-assets.json'}
    if not required <= {entry['path'] for entry in manifest['files']}:
        raise ValueError('package is missing required runtime or installation material')
    verify_build_material(root, manifest)
    for name in (MANIFEST, CHECKSUM):
        if stat.S_IMODE((root / name).lstat().st_mode) != 0o644:
            raise ValueError('manifest files must be mode 0644')
    return manifest


def verify_installed(root):
    """Immutable installed code must remain administrator controlled."""
    root = Path(root)
    trusted_directory(root)
    manifest = verify(root)
    for path in [root, root / MANIFEST, root / CHECKSUM,
                 *(root / entry['path'] for entry in manifest['files'])]:
        for current in (path, *path.parents):
            if current != root and root not in current.parents:
                break
            info = current.lstat()
            if info.st_uid != 0 or info.st_mode & 0o022 or stat.S_ISLNK(info.st_mode):
                raise ValueError('installed program paths must be root owned and not writable by other users')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    args = parser.parse_args()
    manifest = verify(args.root)
    print(json.dumps({'product': 'blog', 'kind': manifest['kind'], 'status': 'verified'}))


if __name__ == '__main__':
    main()
