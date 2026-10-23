#!/usr/bin/env python3
"""Adapter contract regressions; the installed CI lane runs the real server."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('adapter', ROOT / 'test/operations/installed-lifecycle-adapter.py')
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


class AdapterContract(unittest.TestCase):
    def test_production_configuration_and_source_bound_retained_files(self):
        config = json.loads((ROOT / 'deploy/linux-x86_64/config.json.example').read_text())
        adapter.configure(config, Path('/installed'), Path('/shared'), 12345)
        self.assertEqual(config['development'], False)
        self.assertEqual(config['bind'], '127.0.0.1:12345')
        self.assertEqual(config['data_dir'], '/installed/state')
        source = (ROOT / 'src/server/src/installation_definition.rs').read_text()
        self.assertIn('database["name"] = json!("site");', source)
        self.assertIn('path["suffix"] = json!("site.sqlite");', source)
        self.assertIn('material["secrets"] = json!([]);', source)
        self.assertEqual(adapter.RETAINED_FILES, ('state/site.sqlite',))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with self.assertRaisesRegex(RuntimeError, 'retain private database'):
                adapter.retained(root, None, None, require)
            (root / 'state').mkdir()
            (root / 'state/site.sqlite').touch()
            adapter.retained(root, None, None, require)

    def test_errors_never_disclose_response_body(self):
        for status, body in ((401, b'private-response'), (200, b'private-response')):
            with self.assertRaises(RuntimeError) as error:
                adapter.api(lambda *args: (status, body), '/site')
            self.assertNotIn('private-response', str(error.exception))

    def test_business_and_persisted_session_contract(self):
        session = {'csrf_token': 'csrf', 'authenticated': True, 'user_id': 'admin-id'}
        state = {}
        calls = []

        def request(path, method, body, headers):
            calls.append((path, method))
            data = None if body is None else json.loads(body)
            if method != 'GET':
                self.assertEqual(headers['X-CSRF-Token'], 'csrf')
            if path == '/api/v1/auth/session':
                result = session
            elif path == adapter.SITE:
                if method == 'PUT':
                    state['site'] = data
                result = state['site']
            elif path == '/api/v1/site':
                result = {k: v for k, v in state['site'].items() if k != 'author_name'}
            elif path == adapter.CATEGORIES:
                self.assertEqual(data, {'name': 'Installed category'})
                result = {'id': 13, 'sort_name': data['name'], 'article_count': 0}
            elif path == adapter.LABELS:
                self.assertEqual(data, {'sort_id': 13, 'name': 'Installed label'})
                result = {'id': 17, 'sort_id': 13, 'label_name': data['name']}
            elif path == adapter.ARTICLES:
                self.assertEqual(data['sort_id'], 13)
                self.assertEqual(data['label_id'], 17)
                self.assertIs(data['view_status'], True)
                self.assertIsNone(data['password'])
                state['article'] = {**data, 'id': 23}
                result = state['article']
            elif path == '/api/v1/categories':
                result = [{'id': 13, 'sort_name': 'Installed category', 'article_count': 1}]
            else:
                self.assertEqual(path, '/api/v1/articles/23')
                result = state['article']
            return 200, json.dumps(result).encode()

        expected = adapter.exercise(request, session, [], require)
        adapter.persisted(request, session, expected, require)
        self.assertEqual(len(calls), 12)
        state['article']['article_content'] = 'lost'
        with self.assertRaisesRegex(RuntimeError, 'Published article differs'):
            adapter.persisted(request, session, expected, require)
        session['authenticated'] = False
        with self.assertRaisesRegex(RuntimeError, 'session did not survive'):
            adapter.persisted(request, session, expected, require)

    def test_routes_are_current_product_routes(self):
        routes = (ROOT / 'src/server/src/content/mod.rs').read_text() + (ROOT / 'src/server/src/admin.rs').read_text()
        for path in (adapter.SITE, adapter.CATEGORIES, adapter.LABELS, adapter.ARTICLES,
                     '/api/v1/site', '/api/v1/categories', '/api/v1/articles/{id}'):
            self.assertIn('"' + path + '"', routes)


if __name__ == '__main__':
    unittest.main()
