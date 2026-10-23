#!/usr/bin/env python3
"""Accept a real Blog package through installation, TLS APIs and offline reset."""
import argparse
import hashlib
import http.client
from http.cookiejar import CookieJar
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
import json
import os
import pwd
from pathlib import Path, PurePosixPath
import secrets
import signal
import socket
import ssl
import stat
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
from urllib.error import HTTPError
from urllib.request import build_opener, HTTPCookieProcessor, HTTPSHandler, Request
sys.dont_write_bytecode = True
SOURCE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(SOURCE / 'src/tools/packaging'))
import package


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def extract(archive, destination):
    package.physical_directory(archive.parent)
    digest = hashlib.sha256(package.read_regular(archive, package.MAX_BYTES)).hexdigest()
    require(package.read_regular(archive.parent / 'SHA256SUMS') ==
            (digest + '  ' + archive.name + '\n').encode(), 'Archive checksum differs')
    prefix = archive.name.removesuffix('.tar.gz')
    count = total = 0
    seen = set()
    with tarfile.open(archive, 'r:gz') as source:
        for item in source:
            name = PurePosixPath(item.name)
            require(not name.is_absolute() and '..' not in name.parts and
                    name.parts and name.parts[0] == prefix and str(name) == item.name.rstrip('/'),
                    'Unsafe archive path')
            require(str(name) not in seen, 'Duplicate archive path')
            seen.add(str(name))
            require(0 <= item.size <= package.MAX_BYTES, 'Archive member size exceeds its bound')
            target = destination / name
            count += 1
            total += item.size
            require(count <= package.MAX_FILES * 2 and total <= package.MAX_BYTES,
                    'Archive exceeds extraction bounds')
            if item.isdir():
                require(item.mode == 0o755, 'Archive directory mode differs')
                target.mkdir(mode=0o755, parents=True, exist_ok=True)
                target.chmod(0o755)
            else:
                require(item.isfile() and item.mode in (0o644, 0o755), 'Archive links or special files are forbidden')
                target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
                with target.open('xb') as output:
                    stream = source.extractfile(item)
                    require(stream is not None, 'Missing archive bytes')
                    remaining = item.size
                    while remaining:
                        block = stream.read(min(remaining, 1024 * 1024))
                        require(bool(block), 'Truncated archive')
                        output.write(block)
                        remaining -= len(block)
                target.chmod(item.mode)
    return destination / prefix


def free_port():
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 0))
        return probe.getsockname()[1]


class Proxy(BaseHTTPRequestHandler):
    backend = 0
    def log_message(self, *_args):
        pass
    def forward(self):
        length = int(self.headers.get('Content-Length', '0'))
        require(0 <= length <= 1024 * 1024, 'Fixture request too large')
        body = self.rfile.read(length) if length else None
        connection = http.client.HTTPConnection('127.0.0.1', self.backend, timeout=10)
        try:
            headers = {key: value for key, value in self.headers.items() if key.lower() != 'connection'}
            connection.request(self.command, self.path, body=body, headers=headers)
            response = connection.getresponse()
            data = response.read(16 * 1024 * 1024)
            self.send_response(response.status)
            for key, value in response.getheaders():
                if key.lower() not in ('connection', 'transfer-encoding', 'content-length'):
                    self.send_header(key, value)
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        finally:
            connection.close()
    do_GET = do_POST = do_PUT = do_DELETE = forward


def accept(archive):
    require(os.geteuid() == 0, 'Installed candidate checks require root')
    spec = importlib.util.spec_from_file_location('blog_adapter', SOURCE / 'test/operations/installed-lifecycle-adapter.py')
    adapter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(adapter)
    process = proxy = log = None
    with tempfile.TemporaryDirectory(prefix='blog-installed-') as temporary:
        work = Path(temporary)
        work.chmod(0o755)  # The service account must traverse this private fixture parent.
        payload = extract(archive, work)
        package.verify(payload)
        original = package.read_regular(payload / 'share/LICENSE')
        (payload / 'share/LICENSE').write_bytes(original + b'corruption')
        try:
            package.verify(payload)
        except ValueError:
            pass
        else:
            raise RuntimeError('Package corruption was accepted')
        (payload / 'share/LICENSE').write_bytes(original)
        package.verify(payload)
        root = work / 'instance'
        backend = free_port()
        configuration = work / 'config.json'
        configuration.write_text(json.dumps({'bind': f'127.0.0.1:{backend}', 'development': False}))
        configuration.chmod(0o600)
        initial = 'InitialPassphrase-' + secrets.token_urlsafe(24)
        replacement = 'ReplacementPassphrase-' + secrets.token_urlsafe(24)
        intermediate = 'IntermediatePassphrase-' + secrets.token_urlsafe(24)
        other_password = 'OtherRootPassphrase-' + secrets.token_urlsafe(24)
        installed = subprocess.run(['python3', '-I', '-B', str(payload / 'libexec/install.py'),
                                    '--package', str(payload), '--root', str(root),
                                    '--config', str(configuration), '--username', 'admin', '--fixture'],
                                   input=(initial + '\n').encode(), capture_output=True)
        require(installed.returncode == 0, 'Local installer failed')
        package.verify(root)
        setup = ['runuser', '-u', 'isarmg-blog', '--', str(root / 'libexec/blog-setup'), '--root', str(root)]
        maintenance = ['python3', '-I', '-B', str(root / 'libexec/maintenance.py'), '--root', str(root)]
        require(subprocess.run([*setup, 'initialize', '--username', 'admin'], input=(initial + '\n').encode(),
                               capture_output=True).returncode != 0, 'Initialization overwrote an existing database')
        require(subprocess.run([*maintenance, 'check'], capture_output=True).returncode == 0,
                'Installed offline state check failed')
        certificate, key = work / 'fixture.crt', work / 'fixture.key'
        subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
                        '-subj', '/CN=localhost', '-keyout', str(key), '-out', str(certificate)],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
        Proxy.backend = backend
        proxy = ThreadingHTTPServer(('127.0.0.1', 0), Proxy)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(certificate, key)
        proxy.socket = context.wrap_socket(proxy.socket, server_side=True)
        thread = threading.Thread(target=proxy.serve_forever, daemon=True)
        thread.start()
        base = f'https://127.0.0.1:{proxy.server_address[1]}'
        jar = CookieJar()
        opener = build_opener(HTTPCookieProcessor(jar), HTTPSHandler(context=ssl._create_unverified_context()))
        anonymous = build_opener(HTTPSHandler(context=ssl._create_unverified_context()))
        def request(path, method='GET', body=None, headers=None, *, client=None):
            actual = {'Origin': base, 'Sec-Fetch-Site': 'same-origin', **(headers or {})}
            value = Request(base + path, data=body, headers=actual, method=method)
            try:
                with (client or opener).open(value, timeout=10) as response:
                    return response.status, response.read()
            except HTTPError as response:
                return response.code, response.read()
        def login(password):
            status, response = request('/api/v1/auth/login', 'POST',
                                       json.dumps({'username': 'admin', 'password': password}).encode(),
                                       {'Content-Type': 'application/json'})
            return status, json.loads(response)
        log_path = work / 'server.log'
        def start():
            nonlocal process, log
            log = log_path.open('ab')
            process = subprocess.Popen(['runuser', '-u', 'isarmg-blog', '--',
                                        str(root / 'bin/blog'), '--root', str(root)],
                                       stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True,
                                       env=package.offline_environment())
            for _ in range(100):
                require(process.poll() is None, 'Installed server exited before readiness')
                try:
                    if request('/readyz', client=anonymous)[0] == 200:
                        return
                except OSError:
                    pass
                time.sleep(0.1)
            raise RuntimeError('Installed server readiness timed out')
        def stop():
            nonlocal process, log
            if process is not None and process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=8)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=8)
            process = None
            if log is not None:
                log.close()
                log = None
        other_process = other_log = None
        try:
            start()
            require(request('/api/v1/content/statistics', client=anonymous)[0] == 401,
                    'Unauthenticated management API was accepted')
            require(login('WrongPassphrase-' + secrets.token_urlsafe(12))[0] == 401,
                    'Wrong password was accepted')
            status, session = login(initial)
            require(status == 200 and 'csrf_token' in session, 'Initial administrator login failed')
            require(any(cookie.secure and cookie.has_nonstandard_attr('HttpOnly') for cookie in jar),
                    'Production administrator cookie lacks Secure/HttpOnly')
            require(request(adapter.SITE, 'PUT', b'{}', {'Content-Type': 'application/json'})[0] == 403,
                    'CSRF token omission was accepted')
            require(request(adapter.SITE, 'PUT', b'{}', {'Content-Type': 'application/json',
                            'X-CSRF-Token': session['csrf_token'], 'Origin': 'https://foreign.invalid'})[0] == 403,
                    'Foreign Origin was accepted')
            state = adapter.exercise(request, session, [], require)
            require(subprocess.run([*maintenance, 'check'], capture_output=True).returncode != 0,
                    'Offline check bypassed a running writer lock')
            require(subprocess.run([*maintenance, 'admin-reset', '--username', 'admin'],
                                   input=(replacement + '\n').encode(), capture_output=True).returncode != 0,
                    'Offline credential reset bypassed a running writer lock')
            stop()
            start()
            adapter.persisted(request, session, state, require)
            stop()
            # Keep another independently passworded instance running. Runtime
            # overrides must never redirect offline operations into its state.
            other_root = work / 'other-instance'
            other_backend = free_port()
            while other_backend == backend:
                other_backend = free_port()
            other_configuration = work / 'other-config.json'
            other_configuration.write_text(json.dumps({'bind': f'127.0.0.1:{other_backend}', 'development': False}))
            other_configuration.chmod(0o600)
            require(subprocess.run(['python3', '-I', '-B', str(payload / 'libexec/install.py'),
                                    '--package', str(payload), '--root', str(other_root),
                                    '--config', str(other_configuration), '--username', 'admin', '--fixture'],
                                   input=(other_password + '\n').encode(), capture_output=True).returncode == 0,
                    'Second independent instance installation failed')
            other_log = (work / 'other-server.log').open('ab')
            other_process = subprocess.Popen(['runuser', '-u', 'isarmg-blog', '--',
                                              str(other_root / 'bin/blog'), '--root', str(other_root)],
                                             stdin=subprocess.DEVNULL, stdout=other_log, stderr=other_log,
                                             start_new_session=True, env=package.offline_environment())
            Proxy.backend = other_backend
            for _ in range(100):
                require(other_process.poll() is None, 'Second instance exited before readiness')
                try:
                    if request('/readyz', client=anonymous)[0] == 200:
                        break
                except OSError:
                    pass
                time.sleep(0.1)
            else:
                raise RuntimeError('Second instance readiness timed out')
            polluted = {**os.environ, 'BLOG_DATA_DIR': str(other_root / 'state'),
                        'BLOG_MEDIA': str(other_root / 'media'), 'BLOG_BIND': 'invalid-socket',
                        'BLOG_LOG_LEVEL': 'invalid-level', 'BLOG_DEVELOPMENT': 'not-a-boolean',
                        'BLOG_DEV_WEB_DIR': str(other_root / 'state')}
            require(subprocess.run([*setup, 'check'], env=polluted, capture_output=True).returncode == 0,
                    'Runtime environment redirected direct offline check')
            # A third fresh root exercises initialize without wrapper cleaning.
            fresh = work / 'fresh-offline-instance'
            identity = pwd.getpwnam('isarmg-blog')
            fresh.mkdir(mode=0o750)
            fresh.chmod(0o750)
            os.chown(fresh, 0, identity.pw_gid)
            (fresh / 'config').mkdir(mode=0o750)
            (fresh / 'config').chmod(0o750)
            os.chown(fresh / 'config', 0, identity.pw_gid)
            config = fresh / 'config/config.json'
            config.write_text(json.dumps({'bind': f'127.0.0.1:{free_port()}', 'development': False}))
            config.chmod(0o640)
            os.chown(config, 0, identity.pw_gid)
            for name in ('state', 'media', 'logs', 'run'):
                (fresh / name).mkdir(mode=0o700)
                (fresh / name).chmod(0o700)
                os.chown(fresh / name, identity.pw_uid, identity.pw_gid)
            direct = ['runuser', '-u', 'isarmg-blog', '--', str(root / 'libexec/blog-setup'), '--root', str(fresh)]
            require(subprocess.run([*direct, 'initialize', '--username', 'admin'],
                                   input=(initial + '\n').encode(), env=polluted, capture_output=True).returncode == 0
                    and (fresh / 'state/site.sqlite').is_file(),
                    'Runtime environment redirected direct offline initialization')
            require(subprocess.run([*setup, 'admin-reset', '--username', 'admin'],
                                   input=(intermediate + '\n').encode(), env=polluted, capture_output=True).returncode == 0,
                    'Runtime environment redirected direct offline password reset')
            require(subprocess.run([*maintenance, 'check'], env=polluted, capture_output=True).returncode == 0,
                    'Runtime environment redirected installed offline check')
            require(subprocess.run([*maintenance, 'admin-reset', '--username', 'admin'],
                                   input=(replacement + '\n').encode(), env=polluted, capture_output=True).returncode == 0,
                    'Runtime environment redirected installed password reset')
            require(request('/api/v1/auth/login', 'POST',
                            json.dumps({'username': 'admin', 'password': other_password}).encode(),
                            {'Content-Type': 'application/json'}, client=anonymous)[0] == 200,
                    'Offline operations changed the other instance password')
            require(request('/api/v1/auth/login', 'POST',
                            json.dumps({'username': 'admin', 'password': intermediate}).encode(),
                            {'Content-Type': 'application/json'}, client=anonymous)[0] == 401,
                    'Offline password reset targeted the other instance')
            Proxy.backend = backend
            start()
            status, body = request('/api/v1/auth/session')
            require(status != 200 or json.loads(body).get('authenticated') is not True,
                    'Password reset retained a previous session')
            require(login(initial)[0] == 401, 'Previous password survived the reset')
            require(login(intermediate)[0] == 401, 'Intermediate password survived the final reset')
            require(login(replacement)[0] == 200, 'Replacement password failed')
            adapter.verify_public(request, state, require)
            require(stat.S_IMODE((root / 'state').stat().st_mode) == 0o700 and
                    stat.S_IMODE((root / 'state/site.sqlite').stat().st_mode) == 0o600,
                    'Installed private state permissions differ')
            package.verify(root)
            print('Installed Blog candidate passed integrity, initialization, TLS/API/CSRF, restart, instance-lock, polluted-environment isolation and password-reset checks.')
        finally:
            stop()
            if other_process is not None and other_process.poll() is None:
                os.killpg(other_process.pid, signal.SIGTERM)
                try:
                    other_process.wait(timeout=8)
                except subprocess.TimeoutExpired:
                    os.killpg(other_process.pid, signal.SIGKILL)
                    other_process.wait(timeout=8)
            if other_log is not None:
                other_log.close()
            proxy.shutdown()
            proxy.server_close()
            diagnostics = os.environ.get('BLOG_INSTALL_DIAGNOSTICS')
            if diagnostics:
                destination = Path(diagnostics)
                package.physical_directory(destination.parent)
                destination.mkdir(mode=0o700, exist_ok=True)
                package.physical_directory(destination)
                (destination / 'server.log').write_bytes(log_path.read_bytes())
                if (work / 'other-server.log').exists():
                    (destination / 'other-server.log').write_bytes((work / 'other-server.log').read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    args = parser.parse_args()
    accept(args.archive)


if __name__ == '__main__':
    main()
