import assert from 'node:assert/strict';
import { chromium, firefox, expect } from '@playwright/test';
import { preview } from 'vite';

import { authored, mockApi, schemaPrefixes, summary } from './browser-fixtures.mjs';

const routes = [
  ['/', 'Believe in the power of stories'], ['/search', 'Search articles'], ['/sort', 'Article categories'],
  ['/weiYan', 'Posts'], ['/jotting', 'xocs'], ['/menory', 'xocs'], ['/favorite', 'Toolbox'],
  ['/favorite?tab=friends', 'Toolbox'], ['/favorite?tab=music', 'Toolbox'], ['/travel', 'Photo album'], ['/love', 'Our journey together'],
  ['/message', 'Message wall'], ['/about', 'About'], ['/letter', 'To Ming'],
  ['/article/1', summary.article_title],
  ...[['dashboard','Overview'],['articles','Articles'],['categories','Categories'],['labels','Tags'],['comments','Comments'],
    ['notes','Posts'],['tree-hole','Message board'],['users','Users'],['resources','Files'],['links','Content resources'],
    ['family','Love journal'],['site','Site settings'],['home','Home sections'],['new','Add article'],['edit/1','Edit article']].map(([hash,heading])=>['/admin#'+hash,heading]),
];
const adminEmptyStates = {
  users: 'No users yet', comments: 'No comments yet', resources: 'No files yet',
  'tree-hole': 'No messages yet', notes: 'No posts yet', links: 'No content resources yet',
};

async function assertEnglish(page) {
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  const body = await page.locator('body').innerText();
  const attributes = await page.locator('[aria-label],[title],[placeholder],[alt]').evaluateAll(nodes => nodes.flatMap(node => ['aria-label','title','placeholder','alt'].map(name => node.getAttribute(name) || '')).join('\n'));
  const strip = text => [...authored, ...schemaPrefixes].reduce((result, value) => result.replaceAll(value, ''), text);
  assert.doesNotMatch(strip(body), /\p{Script=Han}/u, body);
  assert.doesNotMatch(strip(attributes), /\p{Script=Han}/u, attributes);
  const layout = await page.evaluate(() => ({ width: innerWidth, scrollWidth: document.documentElement.scrollWidth,
    overflowing: [...document.querySelectorAll('body *')].filter(element => element.getBoundingClientRect().right > innerWidth + 1).slice(0, 8).map(element => ({ tag: element.tagName, class: element.className, right: element.getBoundingClientRect().right })) }));
  assert.ok(layout.scrollWidth <= layout.width, `${page.url()}: ${JSON.stringify(layout)}`);
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const context = await browser.newContext({ locale: 'zh-CN', viewport: { width: 390, height: 844 } });
      const page = await context.newPage(), errors = [], state = { authenticated: true };
      page.on('pageerror', error => errors.push(error.message));
      await mockApi(page, state);
      await page.goto(base + '/?lang=zh-CN');
      await page.getByRole('button', { name: '切换为英文', exact: true }).click();
      await expect(page.getByRole('button', { name: 'Switch to Chinese', exact: true })).toBeVisible();
      for (const [path, heading] of routes) {
        const url = new URL(path, base); url.searchParams.set('lang', 'en');
        await page.goto(url.href);
        await page.waitForLoadState('networkidle');
        await expect(page.getByRole('heading', { name: heading, exact: true }).first()).toBeVisible();
        // Hash navigation can reuse the old document's network-idle state.
        // Wait for the actual successful list result, not just its heading.
        const adminPage = url.pathname === '/admin' ? url.hash.slice(1) : '';
        if (adminEmptyStates[adminPage]) {
          await expect(page.locator('.xcss-admin-shell .xcss-empty')).toHaveText(adminEmptyStates[adminPage]);
          await expect(page.locator('.xcss-admin-shell .xcss-loading')).toHaveCount(0);
        } else if (adminPage === 'labels') {
          await expect(page.getByRole('combobox', { name: 'Category', exact: true })).toBeEnabled();
          const category = page.getByRole('combobox', { name: 'Category', exact: true });
          await category.click();
          await expect(page.getByRole('option', { name: '中文分类', exact: true })).toBeVisible();
          await category.press('Escape');
          await expect(page.locator('.xcss-admin-shell .xcss-loading')).toHaveCount(0);
        }
        assert.equal(await page.getByRole('alert').count(), 0, `${path}: ${await page.getByRole('alert').allTextContents()}`);
        await assertEnglish(page);
      }
      await page.goto(base + '/article/1?lang=en');
      await expect(page.getByRole('button', { name: 'Copy code', exact: true })).toBeVisible();
      await expect(page.locator('.article-markdown')).toContainText('中文正文');
      await page.getByRole('button', { name: 'Copyright notice', exact: true }).click();
      await assertEnglish(page);
      await page.getByRole('button', { name: 'Close copyright notice', exact: true }).click();
      await page.goto(base + '/admin?lang=en#categories');
      await page.getByLabel('Category name', { exact: true }).fill('New category');
      await page.getByRole('button', { name: 'Create category', exact: true }).click();
      await expect(page.getByRole('alert')).toContainText('The content or state has changed');
      await expect(page.locator('body')).not.toContainText('SECRET');
      await page.goto(base + '/admin?lang=en#home');
      await page.getByLabel('Section 1 title', { exact: true }).fill('');
      await page.getByRole('button', { name: 'Save sections', exact: true }).click();
      await expect(page.getByRole('alert')).toContainText('Enter a section title');
      await page.goto(base + '/?lang=en');
      await page.getByRole('button', { name: 'Switch to dark mode', exact: true }).click();
      await expect(page.locator('.original-public')).toHaveClass(/theme-dark/);
      await expect(page.getByRole('button', { name: 'Switch to light mode', exact: true })).toHaveAttribute('aria-pressed', 'true');
      await assertEnglish(page);
      await page.reload();
      await expect(page.locator('.original-public')).toHaveClass(/theme-dark/);
      await page.getByRole('button', { name: 'Switch to Chinese', exact: true }).click();
      await expect(page.getByRole('heading', { name: '相信记录的力量', exact: true })).toBeVisible();
      await page.getByRole('button', { name: '切换到浅色模式', exact: true }).click();
      await expect(page.locator('.original-public')).not.toHaveClass(/theme-dark/);
      await expect(page.getByRole('button', { name: '切换到深色模式', exact: true })).toHaveAttribute('aria-pressed', 'false');
      await context.close();

      const restricted = await browser.newContext({ locale: 'en', viewport: { width: 1280, height: 900 } });
      await restricted.addInitScript(() => { Object.defineProperty(window, 'localStorage', { get() { throw new Error('blocked'); } }); });
      const restrictedPage = await restricted.newPage(), restrictedState = { authenticated: false };
      await mockApi(restrictedPage, restrictedState);
      await restrictedPage.goto(base + '/?lang=zh-CN');
      await restrictedPage.locator('.public-header nav > a[href="/sort"]').click();
      await expect(restrictedPage).toHaveURL(/\/sort\?lang=zh-CN/);
      await expect(restrictedPage.getByRole('heading', { name: '文章分类', exact: true })).toBeVisible();
      await restrictedPage.goto(base + '/admin?lang=zh-CN');
      await restrictedPage.getByLabel('用户名', { exact: true }).fill('admin');
      await restrictedPage.getByLabel('密码', { exact: true }).fill('valid-password');
      await restrictedPage.getByRole('button', { name: '登录', exact: true }).click();
      await expect(restrictedPage).toHaveURL(/\/admin\?lang=zh-CN#articles$/);
      await restrictedPage.reload();
      await expect(restrictedPage.getByRole('heading', { name: '文章管理', exact: true })).toBeVisible();
      await restrictedPage.getByRole('link', { name: '返回网站', exact: true }).click();
      await expect(restrictedPage).toHaveURL(/lang=zh-CN/);
      await expect(restrictedPage.getByRole('heading', { name: '相信记录的力量', exact: true })).toBeVisible();
      await restricted.close();
      assert.deepEqual(errors, []);
      console.log(`${engine.name()}: bilingual public/admin routes, error text, authored content, and restricted storage passed`);
    } finally { await browser.close(); }
  }
} finally { await new Promise((resolve, reject) => server.httpServer.close(error => error ? reject(error) : resolve())); }
