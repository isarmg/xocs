#!/usr/bin/env python3
"""Install a verified Blog package into a fresh root without overwriting state."""
import argparse
import grp
import json
import os
from pathlib import Path
import pwd
import re
import shutil
import stat
import subprocess
import sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import package

ACCOUNT = 'isarmg-blog'


def account():
    try:
        identity = pwd.getpwnam(ACCOUNT)
    except KeyError:
        subprocess.run(['useradd', '--system', '--user-group', '--no-create-home',
                        '--home-dir', '/nonexistent', '--shell', '/usr/sbin/nologin', ACCOUNT], check=True)
        identity = pwd.getpwnam(ACCOUNT)
    if identity.pw_uid == 0 or identity.pw_gid == 0 or grp.getgrgid(identity.pw_gid).gr_name != ACCOUNT:
        raise ValueError('Blog account must have its own non-root group')
    if identity.pw_shell not in ('/usr/sbin/nologin', '/sbin/nologin', '/bin/false'):
        raise ValueError('Blog account must not have an interactive login shell')
    return identity


def write(path, value, mode, uid=0, gid=0):
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC, mode)
    try:
        os.fchmod(descriptor, mode)
        os.fchown(descriptor, uid, gid)
        with os.fdopen(os.dup(descriptor), 'wb') as stream:
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
    finally:
        os.close(descriptor)


def validate_service_root(root, *, fixture=False):
    # These directories are hidden by the fixed systemd isolation policy.
    # Temporary test instances skip unit installation and can use /tmp.
    if fixture:
        return
    for masked in ('/tmp', '/var/tmp', '/home', '/root', '/run/user'):
        path = Path(masked)
        if root == path or path in root.parents:
            raise ValueError('production root is hidden by systemd PrivateTmp/ProtectHome')


def install(source, root, config, username, *, fixture=False):
    if os.geteuid() != 0:
        raise ValueError('installation requires root')
    manifest = package.verify(source)
    if not re.fullmatch(r'/[A-Za-z0-9_./-]+', str(root)) or '..' in root.parts or root == Path('/'):
        raise ValueError('installation root must be a normalized absolute path without spaces')
    validate_service_root(root, fixture=fixture)
    parent_descriptor = package.open_directory(root.parent, trusted=True)
    if root.exists() or root.is_symlink() or root == source or source in root.parents:
        raise ValueError('installation requires a fresh root outside the package')
    package.physical_directory(config.parent)
    configuration = package.loads(package.read_regular(config, 1024 * 1024))
    if not isinstance(configuration, dict):
        raise ValueError('configuration must be an object')
    # The installer declares exactly these writable directories in systemd.
    # Advanced external layouts can be provisioned explicitly with blog-setup.
    for name, expected in [('data_dir', root / 'state'), ('media', root / 'media')]:
        if name in configuration and configuration[name] is not None and configuration[name] != str(expected):
            raise ValueError('installer configuration must keep state and media within its root')
    identity = account()
    unit_path = Path('/etc/systemd/system/blog.service')
    if not fixture:
        for directory in ('/etc/systemd/system', '/run/systemd/system', '/run/systemd/transient',
                          '/usr/local/lib/systemd/system', '/usr/lib/systemd/system', '/lib/systemd/system'):
            unit = Path(directory) / 'blog.service'
            dropin = unit.with_name(unit.name + '.d')
            if any(path.exists() or path.is_symlink() for path in (unit, dropin)):
                raise ValueError('an existing Blog systemd unit must be preserved')
    password = sys.stdin.buffer.read(4097)
    if len(password) > 4096:
        raise ValueError('password input exceeds its bound')
    os.mkdir(root.name, mode=0o750, dir_fd=parent_descriptor)
    root_descriptor = os.open(root.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC,
                              dir_fd=parent_descriptor)
    os.fchmod(root_descriptor, 0o750)
    os.fchown(root_descriptor, 0, identity.pw_gid)
    created_root = os.fstat(root_descriptor)
    def verify_root():
        current = root.lstat()
        if (current.st_dev, current.st_ino) != (created_root.st_dev, created_root.st_ino):
            raise ValueError('installation root changed identity')
        package.trusted_directory(root)
    verify_root()
    for directory in ('bin', 'libexec', 'share'):
        (root / directory).mkdir(mode=0o755)
        (root / directory).chmod(0o755)
    # Copy only previously checked regular bytes; links and unmanifested files
    # never enter the installed immutable tree.
    for entry in manifest['files']:
        target = root / entry['path']
        target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
        for directory in (target.parent, *target.parent.parents):
            if directory == root:
                break
            directory.chmod(0o755)
        verify_root()
        write(target, package.read_regular(source / entry['path'], package.MAX_BYTES), entry['mode'])
    for name in (package.MANIFEST, package.CHECKSUM):
        write(root / name, package.read_regular(source / name), 0o644)
    if package.verify_installed(root) != manifest:
        raise ValueError('package source changed during installation')
    (root / 'config').mkdir(mode=0o750)
    (root / 'config').chmod(0o750)
    os.chown(root / 'config', 0, identity.pw_gid)
    write(root / 'config/config.json', (json.dumps(configuration) + '\n').encode(), 0o640, 0, identity.pw_gid)
    for name in ('state', 'media', 'logs', 'run'):
        (root / name).mkdir(mode=0o700)
        (root / name).chmod(0o700)
        os.chown(root / name, identity.pw_uid, identity.pw_gid)
    command = ['runuser', '-u', ACCOUNT, '--', str(root / 'libexec/blog-setup'), '--root', str(root)]
    subprocess.run([*command, 'initialize', '--username', username], input=password,
                   env=package.offline_environment(), check=True)
    subprocess.run([*command, 'check'], stdin=subprocess.DEVNULL,
                   env=package.offline_environment(), check=True)
    if not fixture:
        unit = f'''[Unit]
Description=Blog
After=network.target

[Service]
Type=simple
User={ACCOUNT}
Group={ACCOUNT}
ExecStart={root}/bin/blog --root {root}
WorkingDirectory={root}
Restart=on-failure
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths={root}/state {root}/media {root}/logs {root}/run

[Install]
WantedBy=multi-user.target
'''
        package.trusted_directory(unit_path.parent)
        verify_root()
        write(unit_path, unit.encode(), 0o644)
        subprocess.run(['systemctl', 'daemon-reload'], stdin=subprocess.DEVNULL, check=True)
    verify_root()
    os.close(root_descriptor)
    os.close(parent_descriptor)
    print('Blog installed. The server has not been started or enabled.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--root', type=Path, default=Path('/opt/isarmg/blog'))
    parser.add_argument('--config', required=True, type=Path)
    parser.add_argument('--username', default='admin')
    parser.add_argument('--fixture', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    try:
        install(args.package, args.root, args.config, args.username, fixture=args.fixture)
    except Exception:
        print('Blog installation failed. Existing files and any partial installation are retained for inspection.', file=sys.stderr)
        raise SystemExit(1) from None


if __name__ == '__main__':
    main()
