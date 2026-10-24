import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { AxeBuilder } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { adminSite, mockApi } from './browser-fixtures.mjs';
import { paintedSurfaceColor } from './painted-surface.mjs';

const groups = [
  { labels: ['总览', 'Overview'], pages: [['dashboard', '总览', 'Overview']] },
  { labels: ['内容', 'Content'], pages: [['articles', '文章管理', 'Articles'], ['categories', '分类管理', 'Categories'], ['labels', '标签管理', 'Tags'], ['notes', '随笔管理', 'Notes'], ['letter', '信笺管理', 'Letter']] },
  { labels: ['社区', 'Community'], pages: [['comments', '评论管理', 'Comments'], ['tree-hole', '留言板管理', 'Message board']] },
  { labels: ['资源', 'Resources'], pages: [['resources', '文件管理', 'Files'], ['links', '内容资源', 'Content resources'], ['family', '恋爱笔记', 'Love journal']] },
  { labels: ['设置', 'Settings'], pages: [['site', '网站设置', 'Site settings'], ['home', '首页栏目', 'Home sections']] },
];

async function layout(page) {
  await page.evaluate(() => document.fonts.ready);
  const sizes = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth,
    headerPosition: getComputedStyle(document.querySelector('.blog-admin-shell > .blog-page-header')).position,
    navigationWrap: getComputedStyle(document.querySelector('.blog-header-navigation')).flexWrap }));
  assert.ok(sizes.scroll <= sizes.width, `${page.url()}: body overflows: ${JSON.stringify(sizes)}`);
  assert.ok(['static', 'relative'].includes(sizes.headerPosition), `Header must remain in document flow: ${JSON.stringify(sizes)}`);
  assert.equal(sizes.navigationWrap, 'wrap', 'Navigation wraps naturally instead of clipping links');
}

async function accessible(page) {
  const { violations } = await new AxeBuilder({ page }).include('.blog-admin-shell')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  assert.deepEqual(violations.map(item => ({ id: item.id, nodes: item.nodes.map(node => node.target) })), [], page.url());
}

async function surfaces(page, selector) {
  await page.locator(selector).first().waitFor({ state: 'visible' });
  const expected = await page.evaluate(() => {
    const probe = document.createElement('span');
    probe.style.backgroundColor = 'var(--blog-bg-panel)';
    document.body.append(probe);
    const expected = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return expected;
  });
  const elements = page.locator(selector), count = await elements.count();
  assert.ok(count > 0, `No surfaces found: ${selector}`);
  await expect.poll(() => Promise.all(Array.from({ length: count }, (_, index) =>
    paintedSurfaceColor(elements.nth(index))))).toEqual(Array(count).fill(expected));
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const width of [390, 1280]) for (const language of ['zh-CN', 'en']) {
        const english = language === 'en', labelIndex = english ? 1 : 0;
        const context = await browser.newContext({ locale: language, colorScheme: 'light', viewport: { width, height: 900 } });
        const page = await context.newPage(), errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await mockApi(page, { authenticated: true });
        await page.goto(`${base}/admin?lang=${language}#categories`);
        const primary = page.locator('.blog-header-navigation');
        const secondary = page.locator('.blog-secondary-navigation');
        await expect(primary.getByRole('button')).toHaveCount(5);
        await expect(secondary.getByRole('button', { pressed: true })).toHaveText(english ? 'Categories' : '分类管理');
        for (const group of groups) {
          await primary.getByRole('button', { name: group.labels[labelIndex], exact: true }).click();
          await expect(primary.getByRole('button', { pressed: true })).toHaveText(group.labels[labelIndex]);
          for (const [id, chinese, englishTitle] of group.pages) {
            if (group.pages.length > 1) await secondary.getByRole('button', { name: english ? englishTitle : chinese, exact: true }).click();
            await expect(page).toHaveURL(new RegExp(`#${id}$`));
            await expect(page.getByRole('heading', { level: 1, name: english ? englishTitle : chinese, exact: true })).toBeVisible();
            if (group.pages.length > 1) await expect(secondary.getByRole('button', { pressed: true })).toHaveText(english ? englishTitle : chinese);
            await layout(page);
          }
        }
        await primary.getByRole('button', { name: english ? 'Content' : '内容', exact: true }).click();
        await page.getByRole('button', { name: english ? 'Add article' : '新增文章', exact: true }).click();
        await expect(page.getByRole('heading', { name: english ? 'Add article' : '新增文章', exact: true })).toBeVisible();
        await expect(secondary.getByRole('button', { pressed: true })).toHaveText(english ? 'Articles' : '文章管理');
        await surfaces(page, '.editor-form');
        if (english) await accessible(page);
        await page.getByRole('button', { name: english ? 'Cancel' : '取消', exact: true }).click();
        await expect(page).toHaveURL(/#articles$/);
        await page.goBack();
        await expect(page).toHaveURL(/#new$/);
        await expect(secondary.getByRole('button', { pressed: true })).toHaveText(english ? 'Articles' : '文章管理');
        await page.goto(`${base}/admin?lang=${language}#edit/1`);
        await page.reload();
        await expect(page.getByRole('heading', { name: english ? 'Edit article' : '编辑文章', exact: true })).toBeVisible();
        await expect(primary.getByRole('button', { pressed: true })).toHaveText(english ? 'Content' : '内容');
        await layout(page);
        await primary.getByRole('button', { name: english ? 'Overview' : '总览', exact: true }).click();
        await expect(page.getByRole('table', { name: english ? 'Site statistics' : '网站统计', exact: true })).toBeVisible();
        await surfaces(page, '.blog-table-scroll, .blog-dashboard-sections > section');
        if (english) await accessible(page);
        await page.getByRole('button', { name: english ? 'Switch to dark mode' : '切换到深色模式', exact: true }).click();
        await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
        await surfaces(page, '.blog-table-scroll, .blog-dashboard-sections > section');
        if (english) await accessible(page);
        await primary.getByRole('button', { name: english ? 'Content' : '内容', exact: true }).click();
        await secondary.getByRole('button', { name: english ? 'Categories' : '分类管理', exact: true }).click();
        await surfaces(page, '.inline-form');
        if (english) await accessible(page);
        await primary.getByRole('button', { name: english ? 'Settings' : '设置', exact: true }).click();
        await expect(page.getByLabel(english ? 'Site name' : '网站名称', { exact: true })).toHaveValue('BLOG');
        await surfaces(page, '.blog-settings-form .blog-content-panel');
        if (english) await accessible(page);
        await layout(page);
        await page.getByRole('button', { name: english ? 'Switch to Chinese' : '切换为英文', exact: true }).click();
        await expect(page.getByRole('heading', { level: 1, name: english ? '网站设置' : 'Site settings', exact: true })).toBeVisible();
        await expect(page).toHaveURL(/#site$/);
        await layout(page);
        assert.deepEqual(errors, []);
        await context.close();
        console.log(`${engine.name()} ${width}px ${language}: navigation, history, forms, themes and layout passed`);
      }
      const context = await browser.newContext({ locale: 'en' }), page = await context.newPage();
      await mockApi(page, { authenticated: true });
      let failStatistics = true;
      await page.route('**/api/v1/content/statistics', route => failStatistics
        ? route.fulfill({ status: 503, json: { code: 'unavailable', retryable: true, request_id: 'layout-503' } })
        : route.fulfill({ json: { articles: 1, comments: 0, members: 0, categories: 1 } }));
      await page.goto(`${base}/admin?lang=en#dashboard`);
      await expect(page.getByRole('alert')).toContainText('Unable to load statistics');
      await expect(page.locator('.blog-loading')).toHaveCount(0);
      failStatistics = false;
      await page.getByRole('button', { name: 'Try again', exact: true }).click();
      await expect(page.getByRole('table', { name: 'Site statistics', exact: true })).toBeVisible();
      await expect(page.getByRole('alert')).toHaveCount(0);
      let saved;
      await page.route('**/api/v1/content/site', route => {
        if (route.request().method() === 'PUT') saved = route.request().postDataJSON();
        return route.fulfill({ json: saved || adminSite });
      });
      await page.goto(`${base}/admin?lang=en#site`);
      await page.getByLabel('Site name', { exact: true }).fill('Layout test');
      await page.getByLabel('Notices', { exact: true }).fill('["First notice"]');
      await page.getByLabel('Random covers', { exact: true }).fill('https://example.com/cover.jpg');
      await page.getByRole('button', { name: 'Save settings', exact: true }).click();
      await expect(page.getByText('Site settings saved', { exact: true })).toBeVisible();
      assert.deepEqual(saved, { ...adminSite, web_name: 'Layout test', notices: '["First notice"]', random_cover: 'https://example.com/cover.jpg' });
      await context.close();
      console.log(`${engine.name()}: statistics retry and settings save contract passed`);
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
