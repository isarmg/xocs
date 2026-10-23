import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { AxeBuilder } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { article, mockApi, pageOf, summary, waitForApplication } from './browser-fixtures.mjs';

const families = [
  { id: 1, man_name: 'Lin', woman_name: 'Yu', timing: '2026-10-01', bg_cover: null, man_cover: null, woman_cover: null, family_info: 'We keep the little moments.\nOne photo and one story at a time.', countdown_title: 'A special day', countdown_time: '2026-10-08' },
];
const photos = Array.from({ length: 14 }, (_, index) => ({ id: index + 1, title: `Memory ${index + 1}`, classify: index === 13 ? 'City' : 'Beach', cover: '/live/blog-current-hero.jpg', create_time: '2026-10-08', introduction: null, url: null, link_type: 'lovePhoto' }));
const notes = Array.from({ length: 11 }, (_, index) => ({ id: index + 1, username: 'Lin', content: `Moment ${index + 1}: an ordinary day worth keeping.`, image_path: index === 0 ? '/legacy/avatar.jpg' : null, create_time: '2026-10-08', user_id: 1, is_public: 1, like_count: 0 }));

async function fixtures(page, state) {
  await mockApi(page, { authenticated: true });
  await page.route('**/api/v1/family', route => route.fulfill({ json: state.empty ? [] : families }));
  await page.route('**/api/v1/links/classes?*', route => route.fulfill({ json: state.empty ? [] : [{ classify: 'Beach', count: 13 }, { classify: 'City', count: 1 }] }));
  await page.route('**/api/v1/links/page?*', route => {
    if (state.failPhotos) return route.fulfill({ status: 503, json: { code: 'service_unavailable', retryable: true, request_id: 'public-layout-503' } });
    const url = new URL(route.request().url()), query = url.searchParams;
    const items = state.empty || query.get('kind') !== 'lovePhoto' ? [] : photos.filter(item => !query.get('classify') || item.classify === query.get('classify'));
    return route.fulfill({ json: pageOf(items, url) });
  });
  await page.route('**/api/v1/notes/page?*', route => {
    return route.fulfill({ json: pageOf(state.empty ? [] : notes, new URL(route.request().url())) });
  });
  await page.route('**/api/v1/articles?*', route => {
    const url = new URL(route.request().url()), size = Number(url.searchParams.get('size') || 12);
    const items = Array.from({ length: size === 20 ? 15 : 6 }, (_, index) => ({ ...summary, id: index + 1, article_title: index === 0 ? 'A very long title, with all of its words visible as it slides across the card instead of wrapping onto a second line' : `A day worth keeping ${index + 1}`, article_cover: '/live/blog-current-hero.jpg' }));
    return route.fulfill({ json: pageOf(items, url) });
  });
}
// Run in real browsers in CI: URL state must survive native reload and history.
async function articleBrowseHistory(browser, base) {
  const context = await browser.newContext({ locale: 'zh-CN', viewport: { width: 1280, height: 900 } });
  await context.addInitScript(() => { Object.defineProperty(window, 'localStorage', { get() { throw new Error('blocked'); } }); });
  const page = await context.newPage();
  try {
    await mockApi(page, { authenticated: false });
    await page.route('**/api/v1/articles/13', route => route.fulfill({ json: { ...article, id: 13 } }));
    await page.route('**/api/v1/categories', route => route.fulfill({ json: [
      { id: 1, sort_name: 'Stories', sort_description: null, priority: 0, article_count: 25 },
      { id: 2, sort_name: 'Trips', sort_description: null, priority: 0, article_count: 1 },
    ] }));
    await page.route('**/api/v1/labels?*', route => route.fulfill({ json: [
      { id: 1, sort_id: 1, label_name: 'Tagged', label_description: null, article_count: 25 },
    ] }));
    await page.route('**/api/v1/articles?*', route => {
      const url = new URL(route.request().url());
      const term = url.searchParams.get('search'), category = url.searchParams.get('sort_id');
      const rows = Array.from({ length: category === '2' ? 1 : 25 }, (_, index) => ({
        ...summary, id: index + 1, article_title: `${term || 'Article'} ${index + 1}`,
      }));
      return route.fulfill({ json: pageOf(rows, url) });
    });
    await page.goto(`${base}/sort?sort_id=1&lang=en`);
    await page.getByRole('button', { name: /^Tagged / }).click();
    await expect(page).toHaveURL(/label_id=1/);
    await page.getByRole('button', { name: 'Next page', exact: true }).click();
    await expect(page).toHaveURL(/page=2/);
    await expect(page.locator('.original-list-card').first()).toContainText('Article 13');
    await page.reload();
    await expect(page.locator('.original-list-card').first()).toContainText('Article 13');
    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
    await page.locator('.original-list-card').first().click();
    await waitForApplication(page);
    await expect(page.locator('.article-markdown')).toBeVisible();
    await expect(page).toHaveURL(/article\/13\?lang=en/);
    await page.goBack();
    await expect(page.locator('.original-list-card').first()).toContainText('Article 13');
    await page.getByRole('button', { name: /^Trips / }).click();
    await expect(page).not.toHaveURL(/page=|label_id=/);
    await expect(page.locator('.original-list-card')).toHaveCount(1);
    await page.goBack();
    await expect(page.locator('.original-list-card').first()).toContainText('Article 13');
    await expect(page.getByRole('button', { name: /^Tagged / })).toHaveClass('active');
    await page.goForward();
    await expect(page.getByRole('button', { name: /^Trips / })).toHaveClass('active');
    await expect(page.locator('.original-list-card')).toHaveCount(1);

    await page.goto(`${base}/?lang=en`);
    await page.locator('.aside-search').getByRole('textbox').fill('first');
    await page.locator('.aside-search').getByRole('button', { name: 'Search', exact: true }).click();
    await expect(page).toHaveURL(/\/search\?.*lang=en/);
    await expect(page.locator('.original-list-card').first()).toContainText('first 1');
    await page.getByRole('textbox', { name: 'Search titles or content', exact: true }).fill('second');
    await expect(page).toHaveURL(/q=first/);
    await page.getByRole('button', { name: 'Search', exact: true }).click();
    await expect(page.locator('.original-list-card').first()).toContainText('second 1');
    await page.getByRole('button', { name: 'Next page', exact: true }).click();
    await expect(page.locator('.original-list-card').first()).toContainText('second 13');
    await page.reload();
    await expect(page.locator('.original-list-card').first()).toContainText('second 13');
    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
    await page.goBack();
    await expect(page.locator('.original-list-card').first()).toContainText('second 1');
    await page.goBack();
    await expect(page.locator('.original-list-card').first()).toContainText('first 1');
    await expect(page.getByRole('textbox', { name: 'Search titles or content', exact: true })).toHaveValue('first');
    await page.goForward();
    await expect(page.locator('.original-list-card').first()).toContainText('second 1');
    console.log(`${browser.browserType().name()}: article filters, search, reload, history and explicit language passed`);
  } finally { await context.close(); }
}

async function widthCheck(page, wideSelector) {
  await page.evaluate(() => document.fonts.ready);
  const result = await page.evaluate(selector => ({ viewport: innerWidth, scroll: document.documentElement.scrollWidth,
    content: selector ? document.querySelector(selector)?.getBoundingClientRect().width : null }), wideSelector);
  assert.ok(result.scroll <= result.viewport, `${page.url()}: ${JSON.stringify(result)}`);
  if (result.viewport >= 1440 && wideSelector) assert.ok(result.content >= Math.min(result.viewport, wideSelector === '.love-hero-inner' ? 1200 : wideSelector === '.love-content' ? 1600 : 1920) * .87, `${wideSelector} is too narrow: ${JSON.stringify(result)}`);
}
async function accessible(page) {
  const { violations } = await new AxeBuilder({ page }).include('.love-page').withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  assert.deepEqual(violations.map(item => ({ id: item.id, nodes: item.nodes.map(node => node.target) })), [], page.url());
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const width of [390, 1280, 1920, 2560]) {
        const context = await browser.newContext({ locale: 'en', viewport: { width, height: 1080 }, reducedMotion: 'reduce' });
        const page = await context.newPage(), state = { empty: false, failPhotos: false }, errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await fixtures(page, state);
        await page.clock.install({ time: new Date('2026-10-08T04:34:56Z') });
        // Wait for fixture content to render; autoplay media and lazy resources
        // are independent of whether each route is ready for layout assertions.
        for (const [path, selector, readySelector] of [
          ['/', '.home-layout', '.home-content .article-card:nth-child(6)'],
          ['/sort', '.original-article-list', '.original-list-card:nth-of-type(6)'],
          ['/menory', '.journey-page', '.journey-item:nth-child(6)'],
          ['/jotting', '.wall-layout', '.wall-card:nth-child(10)'],
          ['/travel', '.travel-page', '.travel-content .travel-photo-card:nth-child(12)'],
          ['/favorite', '.favorite-page', '.favorite-empty'],
          ['/article/1', '.reading-layout', '.article-markdown h2'],
          ['/love', '.love-content', '.love-content .travel-photo-card:nth-child(12)'],
        ]) {
          await page.goto(`${base}${path}?lang=en`);
          await page.locator(readySelector).waitFor({ state: 'visible', timeout: 30000 });
          await widthCheck(page, selector);
          if (path === '/') {
            await expect(page.locator('.article-card-excerpt')).toHaveCount(0);
            await expect(page.locator('.article-card h3 .sliding-title').first()).toHaveClass(/is-overflowing/);
            const height = await page.locator('.article-card h3').first().evaluate(node => ({ height: node.getBoundingClientRect().height, line: parseFloat(getComputedStyle(node).lineHeight) }));
            assert.ok(Math.abs(height.height - height.line) < 2, 'Home title stays on one line');
          }
          if (path === '/menory') {
            await expect(page.locator('.journey-item')).toHaveCount(15);
            const geometry = await page.locator('.journey-grid').evaluate(grid => ({ columns: getComputedStyle(grid).gridTemplateColumns.split(' ').length, pictures: [...grid.querySelectorAll('.journey-image')].map(node => ({ ratio: node.clientWidth / node.clientHeight, fit: getComputedStyle(node.querySelector('img')).objectFit })), rows: [...grid.querySelectorAll('.journey-item')].map(node => node.getBoundingClientRect().top) }));
            assert.equal(geometry.columns, width >= 1001 ? 5 : 2);
            assert.ok(geometry.pictures.every(item => Math.abs(item.ratio - 1.5) < .02 && item.fit === 'cover'), 'Journey pictures fill 3:2 frames with uniform scaling');
            if (width >= 1001) assert.equal(geometry.rows.filter(top => Math.abs(top - geometry.rows[0]) < 2).length, 5);
          }
          assert.equal(await page.getByRole('alert').count(), 0, path);
        }
        await widthCheck(page, '.love-hero-inner');
        await expect(page.locator('.love-clock [data-unit=days] strong')).toHaveText('7');
        await expect(page.locator('.love-clock [data-unit=years] strong')).toHaveText('0');
        await expect(page.locator('.love-wave')).toBeVisible();
        await expect(page.locator('.love-tab-icon')).toHaveCount(3);
        await expect(page.getByRole('link', { name: 'A letter for you', exact: true })).toHaveAttribute('href', '/letter');
        await expect(page.locator('.love-countdown')).toContainText('Today is the day');
        await expect(page.locator('.love-story-card')).toContainText(families[0].family_info);
        await expect(page.locator('.travel-photo-card')).toHaveCount(12);
        await expect(page.locator('.love-photo-filters').getByRole('button', { name: /^All/ })).toHaveText('All 14');
        const caption = await page.locator('.travel-photo-card').first().evaluate(card => ({
          imageBottom: card.querySelector('.travel-photo-image').getBoundingClientRect().bottom,
          titleTop: card.querySelector('strong').getBoundingClientRect().top,
          titleBottom: card.querySelector('strong').getBoundingClientRect().bottom,
          dateTop: card.querySelector('small').getBoundingClientRect().top,
        }));
        assert.ok(caption.titleTop >= caption.imageBottom && caption.dateTop >= caption.titleBottom, `Photo captions overlap: ${JSON.stringify(caption)}`);
        await page.getByRole('button', { name: 'Load more photos', exact: true }).click();
        await expect(page.locator('.travel-photo-card')).toHaveCount(14);
        await expect(page.getByRole('button', { name: 'Load more photos', exact: true })).toHaveCount(0);
        await page.locator('.love-photo-filters').getByRole('button', { name: /^City/ }).click();
        await expect(page.locator('.travel-photo-card')).toHaveCount(1);
        const photoTrigger = page.getByRole('button', { name: 'View photo: Memory 14', exact: true });
        for (let opening = 0; opening < 2; opening++) {
          await photoTrigger.focus();
          await photoTrigger.click();
          const lightbox = page.getByRole('dialog', { name: 'View image', exact: true });
          await expect(lightbox).toBeVisible();
          await expect(lightbox.getByRole('button', { name: 'Close image', exact: true })).toBeFocused();
          await page.keyboard.press('Escape');
          await expect(lightbox).toHaveCount(0);
          await expect(photoTrigger).toBeFocused();
        }
        if (width === 1920) await accessible(page);
        const tabs = page.locator('.love-tabs');
        await tabs.getByRole('button', { name: /^Moments/ }).click();
        await expect(page.locator('.love-moment-card')).toHaveCount(10);
        await page.getByRole('navigation', { name: 'Entry pages', exact: true }).getByRole('button', { name: 'Next page', exact: true }).click();
        await expect(page.locator('.love-moment-card')).toHaveCount(1);
        await expect(page.locator('.love-moments-grid')).toContainText('Moment 11');
        await expect(tabs.getByRole('button')).toHaveCount(3);
        await expect(page.locator('.love-story-picker, .family-grid')).toHaveCount(0);
        await expect(page.getByRole('button', { name: /Love wall|Apply|Move in/ })).toHaveCount(0);
        await expect(page.locator('.love-pair')).toContainText('Lin');
        await tabs.getByRole('button', { name: /^Photos/ }).click();
        await expect(tabs.getByRole('button', { name: /^Photos/ })).toHaveAttribute('aria-pressed', 'true');
        await tabs.getByRole('button', { name: /^Wishes/ }).click();
        await expect(page.locator('.love-wishes').getByRole('heading', { name: /^Wishes/ })).toBeVisible();
        await expect(page.locator('.love-wishes').getByRole('button', { name: 'Post anonymous comment', exact: true })).toBeVisible();
        await widthCheck(page, '.love-content');
        if (width === 1920) await accessible(page);
        await page.evaluate(async () => {
          scrollTo(0, 0);
          await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        });
        await expect(page.locator('.public-header')).toBeVisible();
        await page.getByRole('button', { name: 'Switch to dark mode', exact: true }).click();
        await expect(page.locator('.original-public')).toHaveClass(/theme-dark/);
        await page.keyboard.press('Escape');
        await widthCheck(page, '.love-content');
        if (width === 1920) {
          await tabs.getByRole('button', { name: /^Photos/ }).click();
          await expect(page.locator('.travel-photo-card')).toHaveCount(1);
          await accessible(page);
          await tabs.getByRole('button', { name: /^Wishes/ }).click();
          await accessible(page);
        }
        state.empty = true;
        await page.reload();
        await expect(page.getByRole('heading', { name: 'No photos in this collection yet', exact: true })).toBeVisible();
        await expect(page.locator('.love-timer')).toContainText('Start with your first memory');
        await expect(tabs.getByRole('button')).toHaveCount(3);
        await expect(page.locator('.love-story-card')).toContainText('Make a home for everyday moments');
        await widthCheck(page, '.love-content');
        assert.deepEqual(errors, []);
        await context.close();
        console.log(`${engine.name()} ${width}px: wide layouts, photos, entries, private home, themes and empty states passed`);
      }
      const context = await browser.newContext({ locale: 'en', viewport: { width: 1280, height: 900 } }), page = await context.newPage(), state = { empty: false, failPhotos: true };
      await fixtures(page, state);
      await page.goto(`${base}/love?lang=en`);
      await expect(page.getByRole('alert')).toBeVisible();
      await expect(page.locator('.public-loading')).toHaveCount(0);
      state.failPhotos = false;
      await page.getByRole('button', { name: 'Try again', exact: true }).click();
      await expect(page.locator('.travel-photo-card')).toHaveCount(12);
      await expect(page.getByRole('alert')).toHaveCount(0);
      await context.close();
      console.log(`${engine.name()}: photo retry passed`);
      await articleBrowseHistory(browser, base);
      const animated = await browser.newContext({ locale: 'en', viewport: { width: 1440, height: 900 }, reducedMotion: 'no-preference' });
      const animatedPage = await animated.newPage();await fixtures(animatedPage, { empty: false, failPhotos: false });
      for (const path of ['/', '/menory']) {
        await animatedPage.goto(`${base}${path}?lang=en`);
        const title = animatedPage.locator(path === '/' ? '.article-card .sliding-title.is-overflowing' : '.journey-item .sliding-title.is-overflowing').first();
        await expect(title).toBeVisible();
        const movement = await title.evaluate(async node => {
          const label = node.querySelector('.sliding-title-text'), animation = label.getAnimations()[0];
          if (!animation) throw new Error('Long title has no sliding animation');
          animation.pause();animation.currentTime = 0;
          await new Promise(resolve => requestAnimationFrame(resolve));
          const beginning = label.getBoundingClientRect().left;
          animation.currentTime = animation.effect.getTiming().duration;
          await new Promise(resolve => requestAnimationFrame(resolve));
          return { distance: beginning - label.getBoundingClientRect().left, expected: parseFloat(node.style.getPropertyValue('--title-travel')) };
        });
        assert.ok(movement.distance > 0 && Math.abs(movement.distance - movement.expected) < 2, `${path}: ${JSON.stringify(movement)}`);
      }
      await animated.close();console.log(`${engine.name()}: overflowing titles slide across their full text`);

    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
