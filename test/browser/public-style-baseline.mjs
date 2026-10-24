// Optional before/after visual contract. The baseline is a separately built
// public dist, supplied explicitly; normal CI keeps using the public suites.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi, pageOf, summary, waitForApplication } from './browser-fixtures.mjs';

const baselineDist = process.env.BLOG_PUBLIC_BASELINE_DIST;
if (!baselineDist) throw new Error('Set BLOG_PUBLIC_BASELINE_DIST to the previous public dist');
const output = process.env.BLOG_UI_ARTIFACT_DIR;
if (output) await mkdir(output, { recursive: true });
const servers = [];
const reports = [];
const common = ['.original-public', '.public-header', '.brand', '.public-header > nav', '.public-header nav > a',
  '.public-header-controls button', '.public-menu-toggle', '.hero', '.hero h1', '.hero-subtitle', '.public-main'];
const routes = [
  ['home', '/', '.article-card', ['.home-layout', '.home-aside', '.aside-search', '.aside-search input', '.aside-search button', '.article-card', '.article-card h3']],
  ['article', '/article/1', '.article-markdown h2', ['.reading-layout', '.reading-card', '.article-markdown h2', '.article-markdown p', '.article-markdown pre', '.article-comments', '.comment-form textarea', '.comment-form > button']],
  ['message', '/message', '.message-comment-wrap', ['.message-page', '.message-hero', '.message-title', '.message-hero form', '.message-hero input', '.message-hero button', '.message-comment-wrap', '.comment-form textarea', '.comment-form > button']],
  ['love', '/love', '.love-story-card', ['.love-hero-inner', '.love-title', '.love-content', '.love-pair-card', '.love-letter', '.love-story-card', '.love-tabs', '.love-tabs button']],
];
const family = { id: 1, man_name: 'Lin', woman_name: 'Yu', timing: '2026-10-01', bg_cover: null,
  man_cover: null, woman_cover: null, family_info: 'An everyday story and a familiar place.', countdown_title: 'A special day', countdown_time: '2026-10-24' };

async function fixtures(page) {
  await mockApi(page, { authenticated: false });
  await page.route('**/api/v1/family', route => route.fulfill({ json: [family] }));
  await page.route('**/api/v1/articles?*', route => route.fulfill({ json: pageOf(Array.from({ length: 6 }, (_, index) => ({
    ...summary, id: index + 1, article_title: `A familiar story ${index + 1}`,
  })), new URL(route.request().url())) }));
}

async function snapshot(page, selectors) {
  await page.evaluate(() => document.fonts.ready);
  return page.evaluate(async selectors => {
    const result = {};
    const properties = ['display', 'position', 'max-width', 'min-height', 'padding-top', 'padding-right', 'padding-bottom', 'padding-left',
      'margin-top', 'margin-right', 'margin-bottom', 'margin-left', 'gap', 'grid-template-columns', 'flex-direction', 'flex-wrap',
      'font-family', 'font-size', 'font-weight', 'line-height', 'letter-spacing', 'text-align', 'color', 'background-color',
      'border-top-width', 'border-right-width', 'border-bottom-width', 'border-left-width', 'backdrop-filter'];
    for (const selector of selectors) {
      const nodes = [...document.querySelectorAll(selector)];
      result[selector] = await Promise.all(nodes.map(async node => {
        const css = getComputedStyle(node), box = node.getBoundingClientRect();
        const style = Object.fromEntries(properties.map(property => [property, css.getPropertyValue(property)]));
        const state = node.getAttribute('data-blog-server-corner-state');
        let surface = null;
        const source = /^url\((["']?)(.*?)\1\)$/.exec(css.borderImageSource)?.[2];
        if ((state === 'paint' || state === 'clip') && source?.startsWith('blob:')) {
          const image = new Image();
          await new Promise((resolve, reject) => { image.onload = resolve; image.onerror = reject; image.src = source; });
          const canvas = document.createElement('canvas');
          canvas.width = canvas.height = 1;
          const context = canvas.getContext('2d');
          context.drawImage(image, Math.floor(image.naturalWidth / 2), Math.floor(image.naturalHeight / 2), 1, 1, 0, 0, 1, 1);
          surface = [...context.getImageData(0, 0, 1, 1).data];
        }
        return { box: { x: box.x + scrollX, y: box.y + scrollY, width: box.width, height: box.height }, style, surface,
          extent: css.getPropertyValue('--blog-server-corner-extent').trim() };
      }));
    }
    return result;
  }, selectors);
}

function compare(before, after, label) {
  assert.deepEqual(Object.keys(after), Object.keys(before), `${label}: measured elements`);
  for (const [selector, expected] of Object.entries(before)) {
    const actual = after[selector];
    assert.equal(actual.length, expected.length, `${label} ${selector}: element count`);
    for (let index = 0; index < expected.length; index++) {
      const prefix = `${label} ${selector}[${index}]`;
      for (const [key, value] of Object.entries(expected[index].box)) {
        assert.ok(Math.abs(actual[index].box[key] - value) <= .1,
          `${prefix}: ${key} changed from ${value} to ${actual[index].box[key]}`);
      }
      assert.deepEqual(actual[index].style, expected[index].style, `${prefix}: layout, typography and CSS colors`);
      assert.deepEqual(actual[index].surface, expected[index].surface, `${prefix}: actual SVG surface color`);
      assert.equal(actual[index].extent, expected[index].extent, `${prefix}: public corner size`);
    }
  }
}

async function corner(locator) {
  await expect(locator).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
  return locator.evaluate(async node => {
    const css = getComputedStyle(node);
    const source = /^url\((["']?)(.*?)\1\)$/.exec(css.borderImageSource)?.[2];
    if (!source?.startsWith('blob:')) throw new Error('Public corner must use its actual SVG paint');
    const svg = new DOMParser().parseFromString(await (await fetch(source)).text(), 'image/svg+xml');
    const full = svg.querySelector('clipPath#inside path');
    const path = (full ?? svg.querySelector('svg > path')).getAttribute('d');
    const dimension = (name, sides) => parseFloat(css[name]) + (css.boxSizing === 'border-box' ? 0 : sides.reduce((sum, side) =>
      sum + parseFloat(css.getPropertyValue(`padding-${side}`)) + parseFloat(css.getPropertyValue(`border-${side}-width`)), 0));
    const extent = Math.min(parseFloat(css.getPropertyValue('--blog-server-corner-extent')),
      dimension('width', ['left', 'right']) / 2, dimension('height', ['top', 'bottom']) / 2);
    const width = full ? dimension('width', ['left', 'right']) : extent * 2 + 1;
    const points = [...path.matchAll(/[ML](-?[\d.]+) (-?[\d.]+)/g)]
      .map(match => ({ x: Number(match[1]) - width + extent, y: Number(match[2]) }))
      .filter(point => point.x >= -.001 && point.x <= extent + .001 && point.y >= -.001 && point.y <= extent + .001);
    return { extent, points };
  });
}

function curveDistance(before, after, label) {
  assert.equal(after.extent, before.extent, `${label}: effective corner extent`);
  for (const curve of [before, after]) assert.ok(curve.points.length > 10, `${label}: actual SVG corner vertices`);
  const pointDistance = (p, a, b) => {
    const dx = b.x - a.x, dy = b.y - a.y, length = dx * dx + dy * dy;
    const t = length ? Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / length)) : 0;
    return Math.hypot(p.x - a.x - t * dx, p.y - a.y - t * dy);
  };
  const directed = (a, b) => Math.max(...a.flatMap((point, index) => index ? [point,
    { x: (point.x + a[index - 1].x) / 2, y: (point.y + a[index - 1].y) / 2 }] : [point])
    .map(point => Math.min(...b.slice(1).map((end, index) => pointDistance(point, b[index], end)))));
  const error = Math.max(directed(before.points, after.points), directed(after.points, before.points));
  assert.ok(error < 2, `${label}: visible corner displacement is ${error}px (limit 2px)`);
  return error;
}

try {
  servers.push(await preview({ build: { outDir: baselineDist }, preview: { host: '127.0.0.1', port: 0 } }));
  servers.push(await preview({ preview: { host: '127.0.0.1', port: 0 } }));
  const bases = servers.map(server => `http://127.0.0.1:${server.httpServer.address().port}`);
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const width of [390, 1280]) for (const dark of [false, true]) {
        const context = await browser.newContext({ locale: 'en', viewport: { width, height: 900 }, colorScheme: dark ? 'dark' : 'light', reducedMotion: 'reduce' });
        try {
          await context.clock.install({ time: new Date('2026-10-24T08:00:00Z') });
          const pages = await Promise.all(bases.map(async () => {
            const page = await context.newPage();
            await fixtures(page);
            return page;
          }));
          const errors = [];
          for (const page of pages) page.on('pageerror', error => errors.push(error.message));
          for (const [name, route, ready, selectors] of routes) {
            const label = `${engine.name()} ${width}px ${dark ? 'dark' : 'light'} ${name}`;
            await Promise.all(pages.map(async (page, index) => {
              await page.goto(`${bases[index]}${route}?lang=en`);
              await expect(page.locator(ready).first()).toBeVisible();
              await waitForApplication(page);
              const switchTo = page.getByRole('button', { name: dark ? 'Switch to dark mode' : 'Switch to light mode', exact: true });
              if (await switchTo.count()) await switchTo.click();
              // Keep both header hover and reading scroll state identical.
              await page.mouse.move(0, 880);
              await page.evaluate(() => scrollTo(0, 0));
              if (dark) await expect(page.locator('.original-public')).toHaveClass(/theme-dark/);
              else await expect(page.locator('.original-public')).not.toHaveClass(/theme-dark/);
              await waitForApplication(page);
            }));
            const samples = await Promise.all(pages.map(page => snapshot(page, [...common, ...selectors])));
            compare(samples[0], samples[1], label);
            const report = { label, baseline: samples[0], candidate: samples[1] };
            if (name === 'home' || name === 'article') {
              const selector = name === 'home' ? '.article-card' : '.article-copyright-dialog';
              if (name === 'article') {
                await Promise.all(pages.map(page => page.getByRole('button', { name: 'Copyright notice', exact: true }).click()));
                await Promise.all(pages.map(page => expect(page.locator(selector))
                  .toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/)));
                const dialogs = await Promise.all(pages.map(page => snapshot(page, [selector, `${selector} h2`, `${selector} button`])));
                compare(dialogs[0], dialogs[1], `${label} dialog`);
              }
              const curves = await Promise.all(pages.map(page => corner(page.locator(selector).first())));
              report.corner = { extent: curves[1].extent, maximumDisplacement: curveDistance(curves[0], curves[1], label) };
            }
            reports.push(report);
            if (output) await Promise.all(pages.map((page, index) => page.screenshot({ path: path.join(output,
              `${engine.name()}-${width}-${dark ? 'dark' : 'light'}-${name}-${index ? 'candidate' : 'baseline'}.png`), fullPage: true })));
            console.log(`${label}: public positions, sizes, typography, colors and extents preserved${report.corner ? `; actual corner displacement ${report.corner.maximumDisplacement.toFixed(3)}px` : ''}`);
          }
          assert.deepEqual(errors, []);
        } finally { await context.close(); }
      }
    } finally { await browser.close(); }
  }
} finally {
  if (output) await writeFile(path.join(output, 'public-style-comparison.json'), JSON.stringify(reports, null, 2));
  await Promise.all(servers.map(server => server.close()));
}
