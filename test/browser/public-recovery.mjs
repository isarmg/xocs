import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { article, mockApi, pageOf, site, summary, waitForApplication } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
const unavailable = route => route.fulfill({ status: 503, json: { code: 'service_unavailable', retryable: true } });
const absent = route => route.fulfill({ status: 404, json: { code: 'not_found', retryable: false } });

async function articleRecovery(browser) {
  const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 } });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await mockApi(page, { authenticated: false });
  let scenario = 'unavailable', detailReads = 0, accessReads = 0, unlocks = 0;
  await page.route('**/api/v1/articles/1', route => {
    detailReads++;
    if (scenario === 'unavailable') return unavailable(route);
    if (scenario === 'invalid') return route.fulfill({ json: { article_title: 'Incomplete detail' } });
    if (scenario === 'available') return route.fulfill({ json: article });
    return absent(route);
  });
  await page.route('**/api/v1/articles/1/access', route => {
    accessReads++;
    if (scenario === 'access-unavailable') return unavailable(route);
    if (scenario === 'missing') return absent(route);
    return route.fulfill({ json: { password_required: scenario === 'restricted' ? 1 : 0, tips: 'A private story' } });
  });
  await page.route('**/api/v1/articles/1/unlock', route => {
    unlocks++;
    assert.deepEqual(route.request().postDataJSON(), { password: 'correct password' });
    return route.fulfill({ json: { ...article, article_title: 'Protected story', password_required: 1, view_status: 0 } });
  });
  try {
    for (const failure of ['unavailable', 'invalid']) {
      scenario = failure;
      detailReads = 0; accessReads = 0;
      await page.goto(`${base}/article/1?lang=en`);
      await expect(page.getByRole('alert')).toBeVisible();
      await waitForApplication(page);
      await expect(page.getByText('Loading article…', { exact: true })).toHaveCount(0);
      assert.equal(accessReads, 0, 'Transport/contract failures must not probe article visibility');
      scenario = 'available';
      await page.getByRole('button', { name: 'Try again', exact: true }).click();
      await expect(page.locator('.article-markdown')).toContainText('中文正文');
      await expect(page.getByRole('alert')).toHaveCount(0);
      assert.equal(detailReads, 2, 'Retry repeats the failed detail read');
    }

    scenario = 'access-unavailable'; accessReads = 0;
    await page.goto(`${base}/article/1?lang=en`);
    await expect(page.getByRole('alert')).toContainText('temporarily unavailable');
    await expect(page.getByText('Article not found', { exact: true })).toHaveCount(0);
    scenario = 'restricted';
    await page.getByRole('button', { name: 'Try again', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'This article requires an access password', exact: true })).toBeVisible();
    assert.equal(accessReads, 2, 'Access recovery repeats the probe rather than the detail');
    await page.getByLabel('Access password', { exact: true }).fill('correct password');
    await page.getByRole('button', { name: 'Read article', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Protected story', exact: true })).toBeVisible();
    await expect(page.locator('.article-markdown')).toContainText('中文正文');
    await expect(page.locator('.password-form')).toHaveCount(0);
    assert.equal(unlocks, 1);

    scenario = 'public-after-404';
    await page.goto(`${base}/article/1?lang=en`);
    await expect(page.getByRole('alert')).toContainText('Unable to load the article');
    await expect(page.getByText('Loading article…', { exact: true })).toHaveCount(0);
    scenario = 'available';
    await page.getByRole('button', { name: 'Try again', exact: true }).click();
    await expect(page.locator('.article-markdown')).toBeVisible();

    scenario = 'missing';
    await page.goto(`${base}/article/1?lang=en`);
    await expect(page.getByRole('alert')).toContainText('Article not found');
    scenario = 'available';
    await page.getByRole('button', { name: 'Try again', exact: true }).click();
    await expect(page.locator('.article-markdown')).toBeVisible();
    assert.deepEqual(errors, []);
  } finally { await page.close(); }
}

async function photoRecovery(browser) {
  const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 } });
  const errors = [], reads = [];
  let failSecond = true, shortPage = false;
  page.on('pageerror', error => errors.push(error.message));
  await mockApi(page, { authenticated: false });
  await page.route('**/api/v1/links/classes?*', route => route.fulfill({ json: [] }));
  const photo = id => ({ id, title: `Photo ${id}`, classify: null, cover: null, url: null, introduction: null, link_type: 'lovePhoto', create_time: null });
  await page.route('**/api/v1/links/page?*', route => {
    const params = new URL(route.request().url()).searchParams;
    assert.equal(params.get('kind'), 'lovePhoto');
    const number = Number(params.get('page')), size = Number(params.get('size'));
    reads.push(number);
    if (number === 2 && failSecond) return unavailable(route);
    // A new leading photo shifts page 2 after page 1 has already loaded.
    const ids = number === 1 ? Array.from({ length: 12 }, (_, index) => 14 - index)
      : shortPage ? [] : [3, 2, 1];
    return route.fulfill({ json: { items: ids.map(photo), total: shortPage ? 100 : number === 1 ? 14 : 15, page: number, size } });
  });
  try {
    await page.goto(`${base}/travel?lang=en`);
    await expect(page.locator('.travel-photo-card')).toHaveCount(12);
    await page.getByRole('button', { name: 'Next page', exact: true }).click();
    await expect(page.getByRole('alert')).toBeVisible();
    await expect(page.locator('.travel-photo-card')).toHaveCount(12);
    failSecond = false;
    await page.getByRole('button', { name: 'Try again', exact: true }).click();
    await expect(page.locator('.travel-photo-card')).toHaveCount(14);
    await expect(page.getByRole('button', { name: 'Next page', exact: true })).toHaveCount(0);
    await expect(page.locator('.travel-end')).toBeVisible();
    assert.deepEqual(reads, [1, 2, 2]);

    shortPage = true; reads.length = 0;
    await page.reload();
    await expect(page.locator('.travel-photo-card')).toHaveCount(12);
    await page.getByRole('button', { name: 'Next page', exact: true }).click();
    await expect(page.locator('.travel-end')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Next page', exact: true })).toHaveCount(0);
    assert.deepEqual(reads, [1, 2], 'An empty acknowledged page ends browsing even with a stale larger total');
    assert.deepEqual(errors, []);
  } finally { await page.close(); }
}

async function renderedLanguage(browser) {
  const context = await browser.newContext({ locale: 'en', viewport: { width: 1280, height: 900 } });
  const errors = [];
  context.on('page', page => page.on('pageerror', error => errors.push(error.message)));
  await context.addInitScript(() => { Object.defineProperty(window, 'localStorage', { get() { throw new Error('blocked'); } }); });
  // Context routes cover the new tab's first API requests as well as its opener.
  await mockApi({ route: context.route.bind(context) }, { authenticated: false });
  try {
    const page = await context.newPage();
    await page.goto(`${base}/?lang=zh-CN`);
    const articles = page.locator('.public-header nav > a').filter({ hasText: '记录' });
    await expect(articles).toHaveAttribute('href', '/sort?lang=zh-CN');
    await expect(page.locator('.public-header .brand')).toHaveAttribute('href', '/?lang=zh-CN');
    await expect(page.locator('.home-content .article-card').first()).toHaveAttribute('href', '/article/1?lang=zh-CN');
    assert.ok((await page.locator('.public-header nav > a').evaluateAll(nodes => nodes.map(node => new URL(node.href).searchParams.get('lang')))).every(locale => locale === 'zh-CN'));
    // Exercise the rendered href through native new-tab navigation without
    // depending on the host's modifier-click shortcut or headless tab settings.
    await articles.evaluate(node => { node.target = '_blank'; });
    const [opened] = await Promise.all([context.waitForEvent('page'), articles.click()]);
    await opened.bringToFront();
    await opened.waitForURL(`${base}/sort?lang=zh-CN`, { waitUntil: 'domcontentloaded', timeout: 30_000 });
    await expect(opened).toHaveURL(`${base}/sort?lang=zh-CN`);
    await expect(opened.locator('html')).toHaveAttribute('lang', 'zh-CN');
    await expect(opened.getByRole('heading', { name: '文章分类', exact: true })).toBeVisible();
    await expect(page).toHaveURL(`${base}/?lang=zh-CN`);
    assert.deepEqual(errors, []);
  } finally { await context.close(); }
}

async function correctedPage(browser) {
  const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 } });
  const errors = [], reads = [];
  page.on('pageerror', error => errors.push(error.message));
  await mockApi(page, { authenticated: false });
  const articles = Array.from({ length: 13 }, (_, index) => ({ ...summary, id: index + 1, article_title: `Recovery article ${index + 1}` }));
  await page.route('**/api/v1/articles?*', route => {
    const url = new URL(route.request().url());
    reads.push(Number(url.searchParams.get('page')));
    return route.fulfill({ json: pageOf(url.searchParams.get('search') === 'missing' ? [] : articles, url) });
  });
  try {
    await page.goto(`${base}/message?lang=en`);
    await expect(page.getByRole('heading', { name: 'Message wall', exact: true })).toBeVisible();
    const length = await page.evaluate(() => history.length);
    await page.goto(`${base}/sort?lang=en&page=100000`);
    await expect(page).toHaveURL(`${base}/sort?lang=en&page=2`);
    await expect(page.locator('.original-list-card')).toHaveCount(1);
    await expect(page.locator('.original-list-card')).toContainText('Recovery article 13');
    assert.equal(await page.evaluate(() => history.length), length + 1, 'Correction replaces the bad entry');
    assert.deepEqual(reads, [100000, 2]);
    await page.goBack();
    await expect(page).toHaveURL(`${base}/message?lang=en`);
    await page.goForward();
    await expect(page).toHaveURL(`${base}/sort?lang=en&page=2`);
    await expect(page.locator('.original-list-card')).toContainText('Recovery article 13');
    assert.equal(reads.filter(number => number === 100000).length, 1, 'History never returns to the invalid page');

    await page.goto(`${base}/search?lang=en&q=missing&page=99`);
    await expect(page).toHaveURL(`${base}/search?lang=en&q=missing`);
    await expect(page.getByRole('heading', { name: 'No matching articles', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Previous page', exact: true })).toBeDisabled();
    assert.deepEqual(errors, []);
  } finally { await page.close(); }
}

async function modalFocus(page, dialog, first, last) {
  await expect(dialog).toBeVisible();
  await expect.poll(() => dialog.evaluate(node => node.matches(':modal'))).toBe(true);
  await expect(first).toBeFocused();
  await page.keyboard.press('Shift+Tab');
  await expect(last).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(first).toBeFocused();
  await page.keyboard.press('Tab');
  assert.equal(await dialog.evaluate(node => node.contains(document.activeElement)), true);
  await page.keyboard.press('Shift+Tab');
  await expect(first).toBeFocused();
}

async function publicModals(browser) {
  const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await mockApi(page, { authenticated: false });
  await page.route('**/api/v1/articles/1', route => route.fulfill({ json: {
    ...article, article_content: `${article.article_content}\n\n![Modal illustration](/__test-media/image.png)`,
  } }));
  try {
    await page.goto(`${base}/article/1?lang=en`);
    const opener = page.getByRole('button', { name: 'Copyright notice', exact: true });
    await opener.focus();
    await opener.click();
    const copyright = page.getByRole('dialog', { name: 'Copyright notice', exact: true });
    await modalFocus(page, copyright,
      copyright.getByRole('button', { name: 'Close copyright notice', exact: true }),
      copyright.getByRole('link', { name: 'View license', exact: true }));
    await page.keyboard.press('Escape');
    await expect(copyright).toHaveCount(0);
    await expect(opener).toBeFocused();

    const illustration = page.getByRole('button', { name: 'View image: Modal illustration', exact: true });
    await illustration.focus();
    await illustration.press('Enter');
    const lightbox = page.getByRole('dialog', { name: 'View image', exact: true });
    const closeImage = lightbox.getByRole('button', { name: 'Close image', exact: true });
    await modalFocus(page, lightbox, closeImage, closeImage);
    await page.keyboard.press('Escape');
    await expect(lightbox).toHaveCount(0);
    await expect(illustration).toBeFocused();

    let pendingSite;
    await page.route('**/api/v1/site', route => { pendingSite = route; });
    await page.goto(`${base}/?lang=en`);
    const search = page.locator('#home-search');
    await expect(search).toBeVisible();
    await search.focus();
    await expect.poll(() => Boolean(pendingSite)).toBe(true);
    // Release the site read after focusing a connected opener, before the
    // recommendation's owned two-second timer can create its native modal.
    await pendingSite.fulfill({ json: { ...site, notices: JSON.stringify([
      '推送标题：Keyboard recommendation', '推送链接：https://example.test/recommendation',
    ]) } });
    const recommendation = page.getByRole('dialog', { name: 'Daily recommendation', exact: true });
    await modalFocus(page, recommendation,
      recommendation.getByRole('button', { name: 'Close daily recommendation', exact: true }),
      recommendation.getByRole('link', { name: 'Visit now →', exact: true }));
    await page.keyboard.press('Escape');
    await expect(recommendation).toHaveCount(0);
    await expect(search).toBeFocused();
    assert.deepEqual(errors, []);
  } finally { await page.close(); }
}


try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      await articleRecovery(browser);
      await photoRecovery(browser);
      await renderedLanguage(browser);
      await correctedPage(browser);
      await publicModals(browser);
      console.log(`${engine.name()}: article errors/access/retry, overlapping photos, rendered locale/new tabs, corrected page history and native public modal focus passed`);
    } finally { await browser.close(); }
  }
} finally { await server.close(); }
