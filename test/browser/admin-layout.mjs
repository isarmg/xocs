import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { AxeBuilder } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { adminSite, mockApi, waitForApplication } from './browser-fixtures.mjs';
import { paintedSurfaceColor } from './painted-surface.mjs';

const pages = [
  ['dashboard', '总览', 'Overview'], ['articles', '文章管理', 'Articles'], ['categories', '分类管理', 'Categories'],
  ['labels', '标签管理', 'Tags'], ['notes', '随笔管理', 'Notes'], ['letter', '信笺管理', 'Letter'],
  ['comments', '评论管理', 'Comments'], ['tree-hole', '留言板管理', 'Message board'], ['resources', '文件管理', 'Files'],
  ['links', '内容资源', 'Content resources'], ['family', '恋爱笔记', 'Love journal'],
  ['site', '网站设置', 'Site settings'], ['home', '首页栏目', 'Home sections'],
];

async function layout(page) {
  await waitForApplication(page);
  await page.evaluate(() => document.fonts.ready);
  await page.evaluate(() => { scrollTo(0, 0); return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))); });
  const sizes = await page.evaluate(() => {
    const box = element => {
      const value = element.getBoundingClientRect();
      return { top: value.top, bottom: value.bottom, left: value.left, right: value.right,
        height: value.height, center: value.top + value.height / 2 };
    };
    const textBox = element => {
      const range = document.createRange(); range.selectNodeContents(element);
      const value = range.getBoundingClientRect();
      return { top: value.top, center: value.top + value.height / 2 };
    };
    const header = document.querySelector('.blog-admin-shell > .blog-page-header');
    const slot = document.querySelector('.blog-header-navigation-slot');
    const brand = document.querySelector('.blog-product-identity');
    const navigation = document.querySelector('.blog-header-navigation');
    const buttons = [...navigation.querySelectorAll('button')];
    const actions = document.querySelector('.blog-header-actions');
    const actionButtons = [...actions.querySelectorAll('a,button')].map(box);
    const brandBox = box(brand), navBox = box(navigation), slotBox = box(slot), actionBox = box(actions);
    const gap = parseFloat(getComputedStyle(document.querySelector('.blog-header-brand-navigation')).rowGap) || 0;
    const innerWrap = navBox.top >= brandBox.bottom - .5;
    const navRows = [];
    for (const button of buttons.map(box).sort((a, b) => a.top - b.top)) {
      if (!navRows.some(row => Math.abs(row.top - button.top) <= .5)) navRows.push(button);
    }
    const statistics = document.querySelector('.blog-statistics-table')?.closest('.blog-frame').getBoundingClientRect();
    const sections = document.querySelector('.blog-dashboard-sections')?.getBoundingClientRect();
    return { width: innerWidth, scroll: document.documentElement.scrollWidth,
      headerPosition: getComputedStyle(header).position,
      navigationWrap: getComputedStyle(navigation).flexWrap,
      header: box(header), slot: slotBox, brand: brandBox, brandText: textBox(brand.querySelector('strong')),
      navigation: navBox, firstButton: box(buttons[0]), firstButtonText: textBox(buttons[0]), navRows,
      actions: actionBox, actionButtons, sameHeaderRow: actionBox.top < slotBox.bottom - .5,
      innerWrap, innerWrapShift: innerWrap ? brandBox.height + gap : 0,
      selectedPage: navigation.querySelector('[aria-current="page"]')?.textContent,
      dashboardGap: statistics && sections ? sections.top - statistics.bottom : null,
      blockGap: sections ? parseFloat(getComputedStyle(document.querySelector('.blog-dashboard-sections')).rowGap) : null };
  });
  const tolerance = .5, diagnostic = `${page.url()}: ${JSON.stringify(sizes)}`;
  assert.ok(sizes.scroll <= sizes.width, `Body overflows: ${diagnostic}`);
  assert.ok(['static', 'relative'].includes(sizes.headerPosition), `Header must remain in document flow: ${diagnostic}`);
  assert.equal(sizes.navigationWrap, 'wrap', 'Navigation wraps naturally instead of clipping links');
  assert.ok(Math.abs(sizes.brand.top + sizes.innerWrapShift - sizes.navigation.top) <= tolerance,
    `Navigation starts beside the brand or immediately after its intentional narrow-screen wrap: ${diagnostic}`);
  assert.ok(Math.abs(sizes.firstButton.top - sizes.navigation.top) <= tolerance,
    `Measure the actual first button, not only its navigation container: ${diagnostic}`);
  if (!sizes.innerWrap) assert.ok(Math.abs(sizes.brand.center - sizes.firstButton.center) <= tolerance,
    `Brand and first navigation row share a center: ${diagnostic}`);
  if (sizes.sameHeaderRow) {
    const rowCenter = sizes.innerWrap ? sizes.brand.center : sizes.firstButton.center;
    assert.ok(Math.abs(sizes.actionButtons[0].center - rowCenter) <= tolerance,
      `Right actions align to the first header row rather than the middle of wrapped navigation: ${diagnostic}`);
  } else assert.ok(sizes.actions.top >= sizes.slot.bottom - tolerance,
    `Mobile actions may wrap only after the complete navigation slot: ${diagnostic}`);
  for (const button of sizes.actionButtons) {
    const row = sizes.actionButtons.find(candidate => Math.min(candidate.bottom, button.bottom) - Math.max(candidate.top, button.top) > tolerance);
    assert.ok(Math.abs(row.center - button.center) <= tolerance,
      `Action buttons keep their own row centers: ${diagnostic}`);
    assert.ok(button.top >= sizes.header.top - tolerance && button.bottom <= sizes.header.bottom + tolerance,
      `Action controls stay inside the header: ${diagnostic}`);
  }
  if (sizes.dashboardGap !== null) assert.ok(sizes.dashboardGap >= sizes.blockGap - 1,
    `Statistics and management cards have separate surfaces: ${diagnostic}`);
  return sizes;
}

async function headerResizeRegression(page) {
  const route = new URL(page.url()).hash, samples = [];
  for (const width of [1895, 1500, 1895, 1280, 1500, 320, 390, 1895]) {
    await page.setViewportSize({ width, height: 900 });
    const current = await layout(page);
    assert.equal(new URL(page.url()).hash, route, 'Resize the same selected page and action set');
    samples.push(current);
  }
  assert.ok(samples.some(sample => sample.navRows.length === 1), 'Exercise a single navigation row');
  assert.ok(samples.some(sample => sample.navRows.length === 2), 'Exercise the transition to two navigation rows');
  assert.ok(samples.some(sample => sample.width === 320 && sample.innerWrap), 'Exercise the intentional brand/navigation wrap at 320px');
  const anchor = sample => ({
    brandTop: sample.brand.top - sample.header.top,
    brandCenter: sample.brand.center - sample.header.top,
    brandTextTop: sample.brandText.top - sample.header.top,
    brandTextCenter: sample.brandText.center - sample.header.top,
    firstButtonTop: sample.firstButton.top - sample.innerWrapShift - sample.header.top,
    firstButtonCenter: sample.firstButton.center - sample.innerWrapShift - sample.header.top,
    firstTextTop: sample.firstButtonText.top - sample.innerWrapShift - sample.header.top,
    firstTextCenter: sample.firstButtonText.center - sample.innerWrapShift - sample.header.top,
  });
  const reference = anchor(samples[0]);
  for (const sample of samples) {
    assert.equal(sample.selectedPage, samples[0].selectedPage, 'Keep the same selected navigation entry');
    assert.equal(sample.actionButtons.length, samples[0].actionButtons.length, 'Keep the same page-specific actions');
    for (const [name, value] of Object.entries(anchor(sample))) {
      assert.ok(Math.abs(value - reference[name]) <= .5,
        `${route} ${sample.width}px ${name} shifts between single and wrapped rows: ${JSON.stringify({ reference, actual: anchor(sample), rows: sample.navRows.length, innerWrap: sample.innerWrap })}`);
    }
    if (sample.sameHeaderRow) assert.ok(Math.abs(sample.actionButtons[0].center - sample.header.top - samples[0].actionButtons[0].center + samples[0].header.top) <= .5,
      `${route}: first-row right action center must stay fixed when navigation wraps`);
  }
}

async function accessible(page) {
  const { violations } = await new AxeBuilder({ page }).include('.blog-admin-shell')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  assert.deepEqual(violations.map(item => ({ id: item.id, nodes: item.nodes.map(node => node.target) })), [], page.url());
}

async function surfaces(page, selector) {
  await waitForApplication(page);
  await page.locator(selector).first().waitFor({ state: 'visible' });
  const expected = await page.evaluate(() => {
    const probe = document.createElement('span');
    probe.style.backgroundColor = 'var(--blog-bg-panel)';
    document.body.append(probe);
    const expected = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return expected;
  });
  const elements = page.locator(selector).filter({ visible: true }), count = await elements.count();
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
        const english = language === 'en';
        const context = await browser.newContext({ locale: language, colorScheme: 'light', viewport: { width, height: 900 } });
        const page = await context.newPage(), errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await mockApi(page, { authenticated: true });
        await page.goto(`${base}/admin?lang=${language}#categories`);
        const primary = page.locator('.blog-header-navigation');
        await expect(primary.getByRole('button')).toHaveCount(pages.length);
        await expect(primary.locator('[aria-current="page"]')).toHaveText(english ? 'Categories' : '分类管理');
        await expect(primary.locator('[aria-pressed]')).toHaveCount(0);
        await expect(page.locator('.blog-secondary-navigation')).toHaveCount(0);
        for (const [id, chinese, englishTitle] of pages) {
          await primary.getByRole('button', { name: english ? englishTitle : chinese, exact: true }).click();
          await expect(page).toHaveURL(new RegExp(`#${id}$`));
          await expect(page.getByRole('region', { name: english ? englishTitle : chinese, exact: true })).toBeVisible();
          await expect(page.getByRole('heading', { level: 1, name: english ? englishTitle : chinese, exact: true })).toHaveCount(0);
          await expect(primary.locator('[aria-current="page"]')).toHaveText(english ? englishTitle : chinese);
          await layout(page);
        }
        await primary.getByRole('button', { name: english ? 'Articles' : '文章管理', exact: true }).click();
        await page.getByRole('button', { name: english ? 'Add article' : '新增文章', exact: true }).click();
        await expect(page.getByRole('heading', { name: english ? 'Add article' : '新增文章', exact: true })).toBeVisible();
        await expect(primary.locator('[aria-current="page"]')).toHaveText(english ? 'Articles' : '文章管理');
        await surfaces(page, '.blog-frame');
        if (english) await accessible(page);
        await page.getByRole('button', { name: english ? 'Cancel' : '取消', exact: true }).click();
        await expect(page).toHaveURL(/#articles$/);
        await page.goBack();
        await expect(page).toHaveURL(/#new$/);
        await expect(primary.locator('[aria-current="page"]')).toHaveText(english ? 'Articles' : '文章管理');
        await page.goto(`${base}/admin?lang=${language}#edit/1`);
        await page.reload();
        await expect(page.getByRole('heading', { name: english ? 'Edit article' : '编辑文章', exact: true })).toBeVisible();
        await expect(primary.locator('[aria-current="page"]')).toHaveText(english ? 'Articles' : '文章管理');
        await layout(page);
        await primary.getByRole('button', { name: english ? 'Overview' : '总览', exact: true }).click();
        await expect(page.getByRole('table', { name: english ? 'Site statistics' : '网站统计', exact: true })).toBeVisible();
        await surfaces(page, '.blog-frame');
        if (english) await accessible(page);
        await page.getByRole('button', { name: english ? 'Switch to dark mode' : '切换到深色模式', exact: true }).click();
        await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
        await surfaces(page, '.blog-frame');
        if (english) await accessible(page);
        await primary.getByRole('button', { name: english ? 'Categories' : '分类管理', exact: true }).click();
        await expect(page.getByRole('textbox', { name: english ? 'Category name' : '分类名称', exact: true })).toBeEnabled();
        await surfaces(page, '.blog-frame');
        if (english) await accessible(page);
        await primary.getByRole('button', { name: english ? 'Site settings' : '网站设置', exact: true }).click();
        await expect(page.getByLabel(english ? 'Site name' : '网站名称', { exact: true })).toHaveValue('BLOG');
        await surfaces(page, '.blog-settings-form .blog-frame');
        if (english) await accessible(page);
        await layout(page);
        await page.getByRole('button', { name: english ? 'Switch to Chinese' : '切换为英文', exact: true }).click();
        await expect(page.getByRole('region', { name: english ? '网站设置' : 'Site settings', exact: true })).toBeVisible();
        await expect(page).toHaveURL(/#site$/);
        await layout(page);
        assert.deepEqual(errors, []);
        await context.close();
        console.log(`${engine.name()} ${width}px ${language}: navigation, history, forms, themes and layout passed`);
      }
      // Keep the route and its action set fixed while crossing the wrap point
      // in both directions. Comparing container tops alone misses the 4px jump
      // caused by centering a 36px navigation row beside 44px actions.
      for (const language of ['zh-CN', 'en']) {
        const regressionContext = await browser.newContext({ locale: language, viewport: { width: 1895, height: 900 } });
        const regressionPage = await regressionContext.newPage();
        await mockApi(regressionPage, { authenticated: true });
        for (const [id, chinese, english] of [pages[0], pages[1]]) {
          await regressionPage.goto(`${base}/admin?lang=${language}#${id}`);
          await expect(regressionPage.getByRole('region', { name: language === 'en' ? english : chinese, exact: true })).toBeVisible();
          await headerResizeRegression(regressionPage);
        }
        await regressionContext.close();
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
