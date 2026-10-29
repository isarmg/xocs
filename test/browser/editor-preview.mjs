import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { article, fixtureImageBytes, mockApi, waitForApplication } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = server.resolvedUrls.local[0];
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ locale: 'en' }), errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await mockApi(page, { authenticated: true });
      await page.goto(`${base}admin?lang=en#new`);
      await waitForApplication(page);
      await expect(page).toHaveTitle('Add article · blog');
      const body = page.getByRole('textbox', { name: 'Markdown content', exact: true });
      await body.fill('# Initial preview');
      await page.getByRole('button', { name: 'Preview content', exact: true }).click();
      const rendered = page.locator('.editor-markdown-preview');
      await expect(rendered).toContainText('Initial preview');
      await page.clock.install({ time: new Date("2026-01-01T00:00:00Z") });
      await page.clock.pauseAt(new Date("2026-01-01T00:00:01Z"));
      await rendered.evaluate(node => {
        window.previewUpdates = 0;
        // Measure the fixture's rendered heading, independently of decorative
        // SVG nodes that the corner controller restores after an HTML update.
        window.previewObserver = new MutationObserver(records => {
          if (records.some(record => [...record.addedNodes, ...record.removedNodes]
            .some(node => node.nodeName === 'H1'))) window.previewUpdates++;
        });
        window.previewObserver.observe(node, { childList: true, subtree: true });
      });
      for (let index = 0; index < 10; index++) {
        await body.fill(`# Latest draft ${index}`);
        await page.clock.runFor(20);
        await expect(rendered).toContainText('Initial preview');
      }
      await page.clock.runFor(179);
      await expect(rendered).toContainText('Initial preview');
      await page.clock.runFor(2);
      await expect(rendered).toContainText('Latest draft 9');
      assert.equal(await page.evaluate(() => window.previewUpdates), 1, 'The input burst updates the visible preview once');
      await page.getByRole('textbox', { name: 'Title', exact: true }).fill('Metadata edit');
      await page.clock.runFor(250);
      assert.equal(await page.evaluate(() => window.previewUpdates), 1, 'Metadata edits leave rendered Markdown intact');
      await body.fill('# Hidden draft');
      await page.getByRole('button', { name: 'Hide preview', exact: true }).click();
      await page.clock.runFor(250);
      await expect(rendered).toHaveCount(0);
      await page.getByRole('button', { name: 'Preview content', exact: true }).click();
      await expect(rendered).toContainText('Hidden draft');
      await body.fill('# Unmounted draft');
      await page.evaluate(() => { location.hash = 'articles'; });
      await expect(rendered).toHaveCount(0);
      await page.clock.runFor(1_000);
      await expect(page).toHaveTitle('Articles · blog');
      await page.clock.resume();
      let imageRequests = 0;
      await page.route('**/api/v1/articles/1', route => route.fulfill({ json: {
        ...article, article_content: 'Paragraph\n\n'.repeat(150) + '![Below fold](/__test-lazy.png)',
      } }));
      await page.route('**/__test-lazy.png', route => {
        imageRequests++;
        return route.fulfill({ contentType: 'image/png', body: fixtureImageBytes });
      });
      await page.goto(`${base}article/1?lang=en`);
      await waitForApplication(page);
      await expect(page).toHaveTitle(`${article.article_title} · BLOG`);
      const image = page.locator('.article-markdown img');
      await expect(image).toHaveAttribute('loading', 'lazy');
      await expect(image).toHaveAttribute('decoding', 'async');
      await page.waitForLoadState('networkidle');
      assert.equal(imageRequests, 0, 'An offscreen article image stays unfetched, including during markup decoration');
      await image.scrollIntoViewIfNeeded();
      await expect.poll(() => imageRequests).toBe(1);
      await expect.poll(() => image.evaluate(node => node.complete && node.naturalWidth > 0)).toBe(true);
      await page.goto(`${base}?lang=en`);
      await waitForApplication(page);
      await expect(page).toHaveTitle('BLOG');
      assert.deepEqual(errors, []);
      console.log(`${engine.name()}: 10 input edits/200 ms produce one preview update; hide/unmount cancel timers; document titles follow pages; offscreen images load on scroll`);
    } finally { await browser.close(); }
  }
} finally { await server.close(); }
