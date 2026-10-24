"""BLOG business assertions for the real production installed lifecycle."""
import json

PRODUCT = 'blog'
SERVICES = ('blog.service',)
ACCOUNT = 'isarmg-blog'
# installation_definition.rs declares no separate secret files.
RETAINED_FILES = ('state/site.sqlite',)
SITE = '/api/v1/content/site'
CATEGORIES = '/api/v1/content/categories'
LABELS = '/api/v1/content/labels'
ARTICLES = '/api/v1/content/articles'


def configure(config, root, shared, port):
    config.update(bind=f'127.0.0.1:{port}', development=False, data_dir=str(root / 'state'))


def api(request, path, method='GET', body=None, csrf=None):
    headers = {'Content-Type': 'application/json'}
    if csrf:
        headers['X-CSRF-Token'] = csrf
    status, response = request(path, method, None if body is None else json.dumps(body).encode(), headers)
    if status != 200:
        raise RuntimeError('Unexpected business API status')
    try:
        return json.loads(response)
    except (ValueError, UnicodeError):
        raise RuntimeError('Invalid business API JSON') from None


def verify_public(request, state, require):
    require(api(request, '/api/v1/site') == state['public_site'], 'Public site settings differ')
    categories = api(request, '/api/v1/categories')
    require(any(category['id'] == state['category_id']
                and category['sort_name'] == 'Installed category'
                and category['article_count'] == 1 for category in categories),
            'Published category differs')
    article = api(request, '/api/v1/articles/' + str(state['article_id']))
    require(article['article_title'] == 'Installed article'
            and article['article_content'] == '## Installed lifecycle\nPersistent public content'
            and article['sort_id'] == state['category_id']
            and article['label_id'] == state['label_id'], 'Published article differs')


def exercise(request, session, sensitive, require):
    csrf = session['csrf_token']
    site = api(request, SITE)
    if not isinstance(site, dict):
        raise RuntimeError('Invalid site settings')
    site.update(author_name='Installed writer', web_name='Installed blog',
                web_title='Production lifecycle')
    require(api(request, SITE, 'PUT', site, csrf) == site, 'Site update differs')
    category = api(request, CATEGORIES, 'POST', {'name': 'Installed category'}, csrf)
    label = api(request, LABELS, 'POST', {'sort_id': category['id'], 'name': 'Installed label'}, csrf)
    article = api(request, ARTICLES, 'POST', {
        'article_title': 'Installed article',
        'article_content': '## Installed lifecycle\nPersistent public content',
        'article_cover': None, 'video_url': None,
        'sort_id': category['id'], 'label_id': label['id'],
        'view_status': True, 'recommend_status': True, 'comment_status': True,
        'password': None, 'tips': None,
    }, csrf)
    state = {'site': site, 'public_site': {key: value for key, value in site.items() if key != 'author_name'},
             'category_id': category['id'], 'label_id': label['id'], 'article_id': article['id']}
    verify_public(request, state, require)
    return state


def persisted(request, session, state, require):
    current = api(request, '/api/v1/auth/session')
    require(current['authenticated'] is True and current['user_id'] == session['user_id'],
            'Administrator session did not survive restart')
    require(api(request, SITE) == state['site'], 'Site settings did not survive restart')
    verify_public(request, state, require)


def retained(root, shared, state, require):
    require(all((root / relative).is_file() for relative in RETAINED_FILES),
            'Operator uninstall did not retain private database')
