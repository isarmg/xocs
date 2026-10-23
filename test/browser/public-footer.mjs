import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
// Covers every surviving public page and both additional Toolbox tabs.
const pages = ['/', '/search', '/sort', '/jotting', '/menory', '/favorite', '/favorite?tab=friends', '/favorite?tab=music', '/travel', '/love', '/message', '/letter', '/article/1'];
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const width of [390, 1280]) {
        const context = await browser.newContext({ locale: 'en', viewport: { width, height: 800 }, reducedMotion: 'reduce' });
        try {
          const page = await context.newPage(), errors = [];
          page.on('pageerror', error => errors.push(error.message));
          await mockApi(page, { authenticated: false });
          for (const pathname of pages) {
            const url = new URL(pathname, base); url.searchParams.set('lang', 'en');
            await page.goto(url.href);
            const root = page.locator('.original-public');
            await expect(root).toBeVisible();
            await expect(page.locator('.public-footer, footer')).toHaveCount(0);
            await expect(page.locator('body')).not.toContainText('Powered by blog');
            const sheets = await page.evaluate(async () => Promise.all(Array.from(document.querySelectorAll('link[rel="stylesheet"]'), async node => (await fetch(node.href)).text())));
            assert.ok(sheets.every(css => !css.includes('.public-footer')), 'No removed footer background/spacing CSS remains');
            // Empty fixtures may be short. Add content inside the real main
            // region to exercise the existing scroll handler on every page.
            await page.locator('#public-content').evaluate(node => { node.style.minHeight = '1800px'; });
            const gap = await root.evaluate(node => node.getBoundingClientRect().bottom - node.querySelector('main').getBoundingClientRect().bottom);
            assert.ok(Math.abs(gap) <= 2, `${pathname}: no reserved footer block below main (${gap})`);
            await page.evaluate(() => window.scrollTo(0, 1000));
            const top = page.getByRole('button', { name: 'Back to top', exact: true });
            await expect(top).toBeVisible();
            const style = await top.evaluate(node => ({ radius: getComputedStyle(node).borderRadius,
              width: node.getBoundingClientRect().width, height: node.getBoundingClientRect().height,
              position: getComputedStyle(node.parentElement).position, bottom: node.getBoundingClientRect().bottom,
              right: node.getBoundingClientRect().right, text: node.textContent }));
            assert.equal(style.radius, '50%'); assert.equal(style.width, style.height); assert.equal(style.position, 'fixed');
            assert.equal(style.text, '↑'); assert.ok(style.bottom <= 800 && style.right <= width);
            await top.click(); await expect.poll(() => page.evaluate(() => window.scrollY)).toBe(0);
          }
          assert.deepEqual(errors, []);
          console.log(`${engine.name()} ${width}px: ${pages.length} public views have no footer or reserved footer block; independent circular Back to top works`);
        } finally { await context.close(); }
      }
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
