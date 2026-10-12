#!/usr/bin/env python3
"""Run comment regressions against a freshly initialized local xocs binary.

Only temporary fixture data is used. No browser, installed instance or production
service is contacted. Pass the built binary as the sole argument.
"""
import http.client
import http.server
import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
import uuid


def port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def post(base, content, identity=None, visitor=None):
    body = {'request_id': identity or str(uuid.uuid4()), 'content': content}
    headers = {'Content-Type': 'application/json'}
    if visitor:
        headers.update({'X-Real-IP': visitor, 'X-Forwarded-For': visitor})
    request = urllib.request.Request(base + '/api/v1/message-comments', json.dumps(body).encode(), headers)
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        return error.code, json.load(error)


def main():
    binary = str(Path(sys.argv[1]).resolve())
    with tempfile.TemporaryDirectory(prefix='xocs-comment-regression-') as directory:
        os.chmod(directory, 0o700)
        database = Path(directory) / 'site.sqlite'
        subprocess.run([binary, '--data-dir', directory, 'init'], input='TemporaryTestPassphrase123\n', text=True, check=True)
        with sqlite3.connect(database) as db:
            version, revision, fingerprint = db.execute('SELECT application_version,schema_revision,schema_sha256 FROM product_metadata').fetchone()
            assert (version, revision) == ('xocs-db-v4', 4)
            print('initialized actual DDL:', version, revision, fingerprint)
        address = port()
        base = f'http://127.0.0.1:{address}'
        server = None
        log = open(Path(directory) / 'server.log', 'w+')

        def stop():
            nonlocal server
            if server is not None:
                server.terminate()
                server.wait(timeout=15)
                server = None

        def start(trusted):
            nonlocal server
            command = [binary, '--data-dir', directory, 'run', '--bind', f'127.0.0.1:{address}', '--development-http']
            if trusted:
                command += ['--trusted-proxies', '127.0.0.1']
            server = subprocess.Popen(command, stdout=log, stderr=log)
            for _ in range(100):
                if server.poll() is not None:
                    log.flush(); log.seek(0)
                    raise AssertionError(log.read())
                try:
                    with urllib.request.urlopen(base + '/api/v1/health', timeout=1):
                        return
                except urllib.error.URLError:
                    time.sleep(0.1)
            raise AssertionError('fixture server did not become ready')

        try:
            start(False)
            statuses = [post(base, f'Direct visitor {i}', visitor=f'192.0.2.{i}')[0] for i in range(1, 12)]
            assert statuses == [200] * 10 + [429], statuses
            print('PASS: direct mode ignores forwarded identities and enforces its peer quota')
            stop(); start(True)
            statuses = [post(base, f'Proxy visitor {i}', visitor=f'192.0.2.{i}')[0] for i in range(1, 12)]
            assert statuses == [200] * 11, statuses
            assert post(base, 'Missing trusted header')[0] == 400
            print('PASS: eleven visitors behind the configured proxy each receive independent quota')

            class DropFirstResponse(http.server.BaseHTTPRequestHandler):
                count = 0

                def log_message(self, *_args):
                    pass

                def do_POST(self):
                    payload = self.rfile.read(int(self.headers['Content-Length']))
                    request = urllib.request.Request(base + self.path, payload, {'Content-Type': 'application/json', 'X-Real-IP': '192.0.2.200'})
                    with urllib.request.urlopen(request, timeout=10) as response:
                        data, status = response.read(), response.status
                    type(self).count += 1
                    if self.count == 1:
                        self.close_connection = True
                        return
                    self.send_response(status)
                    self.send_header('Content-Type', 'application/json')
                    self.send_header('Content-Length', str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)

            proxy = http.server.ThreadingHTTPServer(('127.0.0.1', 0), DropFirstResponse)
            worker = threading.Thread(target=proxy.serve_forever, daemon=True)
            worker.start()
            identity = str(uuid.uuid4())
            try:
                with sqlite3.connect(database) as db:
                    before = db.execute('SELECT count(*) FROM comment').fetchone()[0]
                try:
                    post(f'http://127.0.0.1:{proxy.server_port}', 'Committed before disconnection', identity)
                    raise AssertionError('the first successful response should have been lost')
                except http.client.RemoteDisconnected:
                    pass
                stop(); start(True)
                status, comment = post(f'http://127.0.0.1:{proxy.server_port}', 'Committed before disconnection', identity)
                assert status == 200
                with sqlite3.connect(database) as db:
                    assert db.execute('SELECT count(*) FROM comment').fetchone()[0] == before + 1
                    receipt_id = db.execute('SELECT comment_id FROM comment_submission WHERE request_id=?', (identity,)).fetchone()[0]
                    assert comment['id'] == receipt_id
                assert post(base, 'Different content', identity, '192.0.2.200')[0] == 409
                print('PASS: dropped committed response, process restart and retry leave exactly one comment')
                with sqlite3.connect(database) as db:
                    db.execute('PRAGMA foreign_keys=ON')
                    db.execute('DELETE FROM comment WHERE id=?', (comment['id'],))
                assert post(base, 'Committed before disconnection', identity, '192.0.2.200')[0] == 409
                with sqlite3.connect(database) as db:
                    assert db.execute('SELECT count(*) FROM comment').fetchone()[0] == before
                print('PASS: reusing an identity with different content or after deletion never inserts another row')
            finally:
                proxy.shutdown(); proxy.server_close(); worker.join()
        finally:
            stop(); log.close()
        subprocess.run([binary, '--data-dir', directory, 'config', 'validate'], check=True)


if __name__ == '__main__':
    main()
