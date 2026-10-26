import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi, waitForApplication } from './browser-fixtures.mjs';

const surfaces = await readFile(new URL('../../src/browser/src/corner_surfaces.txt', import.meta.url), 'utf8');
const results = [];
const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;

async function coverage(page) {
  const missing = await page.evaluate(selector => [...document.querySelectorAll('*')].filter(node => {
    if (node.namespaceURI !== 'http://www.w3.org/1999/xhtml' || node.hasAttribute('data-blog-server-corner-decoration')) return false;
    const extent = node.getAttribute('data-blog-server-corner-extent') ?? getComputedStyle(node).getPropertyValue('--blog-server-corner-extent');
    return parseFloat(extent) > 0 && !node.matches(selector);
  }).map(node => ({ tag: node.tagName, classes: node.className })), surfaces);
  assert.deepEqual(missing, [], `${page.url()}: every positive extent belongs to an explicit surface role`);
}

try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ locale: 'en', viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await mockApi(page, { authenticated: true });
      for (const route of ['/', '/sort', '/travel', '/favorite', '/jotting', '/love', '/article/1', '/admin#articles', '/admin#site']) {
        const url = new URL(route, base);
        url.searchParams.set('lang', 'en');
        await page.goto(url.href);
        await waitForApplication(page);
        await expect(page.locator('main').first()).toBeVisible();
        if (url.hash === '#site') await expect(page.getByRole('textbox', { name: 'Site name', exact: true })).toBeVisible();
        await coverage(page);
      }
      await page.evaluate(() => {
        const host = document.createElement('aside');
        host.id = 'corner-regression-host';
        const surface = document.createElement('div');
        surface.id = 'corner-regression-surface';
        surface.setAttribute('data-blog-server-corner-extent', '24');
        surface.style.cssText = 'width:180px;height:100px;border:1px solid black;background:white';
        host.append(surface);
        document.body.append(host);
      });
      const surface = page.locator('#corner-regression-surface');
      await expect(surface).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
      const originalPaint = await surface.evaluate(node => node.style.borderImageSource);
      await surface.evaluate(node => node.setAttribute('data-blog-server-corner-extent', '12'));
      await expect.poll(() => surface.evaluate(node => node.style.borderImageSource)).not.toBe(originalPaint);
      await surface.evaluate(node => node.removeAttribute('data-blog-server-corner-extent'));
      await expect(surface).not.toHaveAttribute('data-blog-server-corner-state');
      await surface.evaluate(node => node.className = 'blog-button');
      await expect(surface).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
      await page.evaluate(() => {
        const style = document.createElement('style');
        style.textContent = '.corner-transition-initial { transition: background-color 1s, opacity .1s }'
          + '.corner-transition-updated { transition: background-color .2s, transform .25s }';
        document.head.append(style);
        document.querySelector('#corner-regression-surface').classList.add('corner-transition-initial');
      });
      await expect.poll(() => surface.evaluate(node => node.style.transitionProperty)).toBe('opacity');
      await page.evaluate(() => {
        customElements.define('corner-reentrant-surface', class extends HTMLElement {
          static observedAttributes = ['data-blog-server-corner-state'];
          attributeChangedCallback() {
            if (this.querySelector('#corner-reentrant-child')) return;
            const child = document.createElement('div');
            child.id = 'corner-reentrant-child';
            child.setAttribute('data-blog-server-corner-extent', '8');
            child.style.cssText = 'position:absolute;width:40px;height:30px;background:white;border:1px solid black';
            this.append(child);
            this.querySelector('#corner-reentrant-promoted').setAttribute('data-blog-server-corner-extent', '6');
            document.querySelector('#corner-regression-surface').classList.replace('corner-transition-initial', 'corner-transition-updated');
          }
        });
        const parent = document.createElement('corner-reentrant-surface');
        parent.style.cssText = 'display:block;width:160px;height:90px;background:white;border:1px solid black';
        const promoted = document.createElement('div');
        promoted.id = 'corner-reentrant-promoted';
        promoted.style.cssText = 'position:absolute;width:30px;height:20px;background:white;border:1px solid black';
        parent.append(promoted);
        parent.setAttribute('data-blog-server-corner-extent', '16');
        document.querySelector('#corner-regression-host').append(parent);
      });
      await expect(page.locator('#corner-reentrant-child')).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
      await expect(page.locator('#corner-reentrant-promoted')).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
      await expect.poll(() => surface.evaluate(node => node.style.transitionProperty)).toBe('transform');
      await surface.evaluate(node => node.remove());
      await expect(page.locator('#corner-regression-surface')).toHaveCount(0);
      await page.evaluate(() => {
        const host = document.querySelector('#corner-regression-host');
        for (let i = 0; i < 5000; i++) host.append(document.createElement('span'));
      });
      const measurement = await page.evaluate(selector => {
        const html = node => node.namespaceURI === 'http://www.w3.org/1999/xhtml'
          && !node.hasAttribute('data-blog-server-corner-decoration');
        const positive = node => parseFloat(node.getAttribute('data-blog-server-corner-extent')
          ?? getComputedStyle(node).getPropertyValue('--blog-server-corner-extent')) > 0;
        const all = () => [...document.querySelectorAll('*')].filter(html).filter(positive);
        const candidates = [...document.querySelectorAll(selector)].filter(html);
        const selected = () => candidates.filter(positive);
        const expected = all(), actual = selected();
        assertEqual(expected.length, actual.length);
        if (!expected.every(node => actual.includes(node))) throw new Error('candidate selection changed decoration');
        function assertEqual(left, right) { if (left !== right) throw new Error(`surface counts differ: ${left} vs ${right}`); }
        const measure = read => {
          const samples = [];
          for (let i = 0; i < 11; i++) {
            const begin = performance.now();
            for (let repeat = 0; repeat < 5; repeat++) read();
            samples.push((performance.now() - begin) / 5);
          }
          return samples.sort((a, b) => a - b)[5];
        };
        return { allElements: document.querySelectorAll('*').length, candidates: candidates.length,
          surfaces: actual.length, baselineMedianMs: measure(all), cachedCandidatesMedianMs: measure(selected),
          note: 'local style-discovery microbenchmark; excludes geometry and paint; 5000 inert elements; 11 x 5 iterations' };
      }, surfaces);
      await coverage(page);
      assert.deepEqual(errors, [], 'candidate changes preserve browser execution');
      results.push({ engine: browser.browserType().name(), ...measurement });
      console.log(`${browser.browserType().name()}: explicit candidate coverage, dynamic styles, insertion and removal passed; ${JSON.stringify(measurement)}`);
    } finally { await browser.close(); }
  }
  if (process.env.BLOG_CORNER_BENCHMARK_OUT) {
    await writeFile(process.env.BLOG_CORNER_BENCHMARK_OUT, `${JSON.stringify(results, null, 2)}\n`);
  }
} finally { await server.close(); }
