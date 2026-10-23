// Track application requests separately from media and browser lifecycle events.
const applicationRequests=new WeakMap();
function observeApplicationRequests(page){
  if(applicationRequests.has(page)||typeof page.on!=='function')return;
  const pending=new Set();applicationRequests.set(page,pending);
  page.on('framenavigated',frame=>{if(frame===page.mainFrame())pending.clear();});
  page.on('request',request=>{if(new URL(request.url()).pathname.startsWith('/api/v1/'))pending.add(request);});
  page.on('requestfinished',request=>pending.delete(request));
  page.on('requestfailed',request=>pending.delete(request));
}
export async function waitForApplication(page){
  const {expect}=await import('./dependencies.mjs');
  await expect(page.locator('html')).toHaveAttribute('data-blog-runtime','blog-interface');
  for(let pass=0;pass<2;pass++){
    await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    await expect.poll(()=>[...(applicationRequests.get(page)??[])].map(request=>request.url()),{message:`Current application reads: ${page.url()}`}).toEqual([]);
  }
}
export const session = { authenticated: true, user_id: 'A'.repeat(43), username: 'admin', role: 'admin', csrf_token: 'A'.repeat(43) };
export const site = { web_name: 'BLOG', web_title: null, notices: null, background_image: null, avatar: null, random_cover: null };
export const adminSite = { ...site, author_name: 'Site owner' };
export const letter = { recipient: 'Reader', sender: 'Writer', content: 'A letter from the site.\nEveryday stories belong here.' };
const category = { id: 1, sort_name: '中文分类', sort_description: null, priority: 0, article_count: 1 };
export const summary = { id: 1, article_title: '中文文章标题', article_cover: null, sort_id: 1, label_id: 1, sort_name: category.sort_name, label_name: null, view_count: 1, like_count: 0, comment_count: 0, recommend_status: 0, view_status: 1, create_time: '2026-10-08', excerpt: '中文摘要', search_snippet: null };
const { excerpt: _excerpt, search_snippet: _snippet, ...articleFields } = summary;
export const article = { ...articleFields, user_id: 1, article_content: '## 中文正文\n\n中文内容\n\n```js\nconst example = 1;\n```', video_url: null, username: '中文作者', comment_status: 1, password_required: 0, tips: null, update_time: null };
const adminPagePaths = new Set(['comments', 'resources', 'tree-hole', 'notes', 'links'].map(name => `/api/v1/content/${name}`));
// Mirror the request's pagination, including production defaults and bounds.
// Keep total before slicing so a later page never repeats the first-page rows.
export function pageOf(items, url) {
  const comments = /^\/api\/v1\/(?:comments|message-comments|love-comments)(?:\/|$)/.test(url.pathname);
  const notes = url.pathname === '/api/v1/notes/page';
  const defaultSize = adminPagePaths.has(url.pathname) || /^\/api\/v1\/content\/articles\/\d+\/news$/.test(url.pathname) ? 20 : comments && url.pathname.endsWith('/replies') ? 5 : comments || notes ? 10 : 12;
  const integer = (name, fallback, maximum) => {
    const value = Number(url.searchParams.get(name) ?? fallback);
    if (!Number.isSafeInteger(value)) throw new TypeError(`Invalid fixture ${name}: ${url.href}`);
    return Math.max(1, Math.min(maximum, value));
  };
  const page = integer('page', 1, 100_000), size = integer('size', defaultSize, comments || notes ? 50 : 100);
  return { items: items.slice((page - 1) * size, page * size), total: items.length, page, size };
}
const sections = [{ id: 1, title: '中文栏目', kind: 'latest', sort_id: null, priority: 0, enabled: true }];
export const authored = ['中文分类', '中文文章标题', '中文摘要', '中文正文', '中文内容', '中文作者', '中文栏目'];
export const schemaPrefixes = ['推送标题：', '推送封面：', '推送链接：'];
export async function mockApi(page, state) {
  observeApplicationRequests(page);
  await page.route('**/api/v1/**', async route => {
    const url = new URL(route.request().url()), path = url.pathname;
    if (route.request().method() !== 'GET') {
      if (path === '/api/v1/auth/login') { state.authenticated = true; return route.fulfill({ json: session }); }
      return route.fulfill({ status: 409, json: { code: 'conflict', message: '内部错误 SECRET /private/db', retryable: false, request_id: 'language-409' } });
    }
    if (path === '/api/v1/content/categories') return route.fulfill({status:405,json:{code:'method_not_allowed'}});
    if (path === '/api/v1/auth/session') return state.authenticated ? route.fulfill({ json: session }) : route.fulfill({ status: 401, json: { code: 'unauthorized' } });
    const value = path === '/api/v1/content/site' ? adminSite : path.endsWith('/site') ? site
      : path === '/api/v1/letter' ? letter
      : path === '/api/v1/content/letter' ? { ...letter, is_public: true }
      : path === '/api/v1/site/stats' ? { article_count: 1, view_count: 1 }
      : path === '/api/v1/content/statistics' ? { articles: 1, comments: 0, members: 0, categories: 1 }
      : path.endsWith('/home-sections') ? sections
      : path === '/api/v1/categories' ? [category]
      : path === '/api/v1/articles/1' || path === '/api/v1/content/articles/1' ? article
      : path.endsWith('/articles') || path.endsWith('/articles/search') ? pageOf([summary], url)
      : path.includes('/news') ? path.startsWith('/api/v1/content/') ? pageOf([], url) : []
      : adminPagePaths.has(path) || path.endsWith('/page') || /^\/api\/v1\/(?:comments|message-comments|love-comments)/.test(path) ? pageOf([], url)
      : [];
    await route.fulfill({ json: value });
  });
}
