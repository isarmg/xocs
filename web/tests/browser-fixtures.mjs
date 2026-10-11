export const session = { authenticated: true, user_id: 'A'.repeat(43), username: 'admin', role: 'admin', csrf_token: 'A'.repeat(43) };
export const site = { web_name: 'XOCS', web_title: null, notices: null, footer: null, background_image: null, avatar: null, random_cover: null };
const category = { id: 1, sort_name: '中文分类', sort_description: null, priority: 0, article_count: 1 };
export const summary = { id: 1, article_title: '中文文章标题', article_cover: null, sort_id: 1, label_id: 1, sort_name: category.sort_name, label_name: null, view_count: 1, like_count: 0, comment_count: 0, recommend_status: 0, view_status: 1, create_time: '2026-10-08', excerpt: '中文摘要', search_snippet: null };
const { excerpt: _excerpt, search_snippet: _snippet, ...articleFields } = summary;
export const article = { ...articleFields, user_id: 1, article_content: '## 中文正文\n\n中文内容\n\n```js\nconst example = 1;\n```', video_url: null, username: '中文作者', comment_status: 1, password_required: 0, tips: null, update_time: null };
const pageOf = items => ({ items, total: items.length, page: 1, size: 15 });
const sections = [{ id: 1, title: '中文栏目', kind: 'latest', sort_id: null, priority: 0, enabled: true }];
export const authored = ['中文分类', '中文文章标题', '中文摘要', '中文正文', '中文内容', '中文作者', '中文栏目'];
export const schemaPrefixes = ['推送标题：', '推送封面：', '推送链接：'];
export async function mockApi(page, state) {
  await page.route('**/api/v1/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (route.request().method() !== 'GET') {
      if (path === '/api/v1/auth/login') { state.authenticated = true; return route.fulfill({ json: session }); }
      return route.fulfill({ status: 409, json: { code: 'conflict', message: '内部错误 SECRET /private/db', retryable: false, request_id: 'language-409' } });
    }
    if (path === '/api/v1/auth/session') return state.authenticated ? route.fulfill({ json: session }) : route.fulfill({ status: 401, json: { code: 'unauthorized' } });
    const value = path.endsWith('/site') ? site
      : path === '/api/v1/site/stats' ? { article_count: 1, view_count: 1 }
      : path === '/api/v1/content/statistics' ? { articles: 1, comments: 0, members: 0, categories: 1 }
      : path.endsWith('/home-sections') ? sections
      : path.endsWith('/categories') ? [category]
      : path === '/api/v1/articles/1' || path === '/api/v1/content/articles/1' ? article
      : path.endsWith('/articles') || path.endsWith('/articles/search') ? pageOf([summary])
      : path.includes('/news') ? []
      : ['/api/v1/content/users','/api/v1/content/comments','/api/v1/content/resources','/api/v1/content/tree-hole'].includes(path) || path.endsWith('/page') || /^\/api\/v1\/(?:comments|message-comments|love-comments)/.test(path) ? pageOf([])
      : path.endsWith('/notes') ? pageOf([])
      : path === '/api/v1/content/links' ? { items: [], total: 0, page: Number(new URL(route.request().url()).searchParams.get('page')) || 1, size: 20 }
      : [];
    await route.fulfill({ json: value });
  });
}
