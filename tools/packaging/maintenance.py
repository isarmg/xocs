#!/usr/bin/env python3
"""Invoke Blog-owned offline checks and credential reset as the service user."""
import argparse
import os
from pathlib import Path
import pwd
import subprocess
import sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import package


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('check')
    reset = commands.add_parser('admin-reset')
    reset.add_argument('--username', required=True)
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise SystemExit('Installed maintenance requires root.')
    package.verify_installed(args.root)
    identity = pwd.getpwnam('isarmg-blog')
    if identity.pw_uid == 0 or identity.pw_gid == 0:
        raise SystemExit('Blog service account must be non-root.')
    command = ['runuser', '-u', 'isarmg-blog', '--', str(args.root / 'libexec/blog-setup'),
               '--root', str(args.root), args.command]
    if args.command == 'admin-reset':
        command += ['--username', args.username]
    raise SystemExit(subprocess.run(command, env=package.offline_environment(), check=False).returncode)


if __name__ == '__main__':
    main()
