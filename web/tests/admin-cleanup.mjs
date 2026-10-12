import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { chromium, firefox, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { preview } from 'vite';
import { createServerCornerGeometry } from '@xcss/web/admin-ui/server-corners';
import { mockApi, session } from './browser-fixtures.mjs';

const output = process.env.XOCS_UI_ARTIFACT_DIR;
if (output) await mkdir(output, { recursive: true });
const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
const image = { name: '旅行图片.png', mimeType: 'image/png', buffer: Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=', 'base64') };

async function assertProjectCurve(button) {
  await expect(button).toHaveAttribute('data-xcss-server-corner-state', 'paint');
  const paint = await button.evaluate(async node => {
    const css = getComputedStyle(node);
    const dimension = (name, sides) => parseFloat(css[name]) + (css.boxSizing === 'border-box' ? 0 :
      sides.reduce((sum, side) => sum + parseFloat(css.getPropertyValue(`padding-${side}`)) + parseFloat(css.getPropertyValue(`border-${side}-width`)), 0));
    const property = node.style.borderImageSource === 'none' ? 'backgroundImage' : 'borderImageSource';
    const match = /url\(["']?(.*?)["']?\)/.exec(node.style[property]);
    if (!match?.[1].startsWith('blob:')) throw new Error('Button must use the actual shared SVG paint');
    const image = new Image();
    await new Promise((resolve, reject) => { image.onload = resolve; image.onerror = reject; image.src = match[1]; });
    const svg = new DOMParser().parseFromString(await (await fetch(match[1])).text(), 'image/svg+xml');
    const full = svg.querySelector('clipPath#inside path');
    return { width: dimension('width', ['left', 'right']), height: dimension('height', ['top', 'bottom']), extent: parseFloat(css.getPropertyValue('--xcss-server-corner-extent')),
      radius: css.borderRadius, compact: !full, actualPath: (full ?? svg.querySelector('svg > path')).getAttribute('d') };
  });
  assert.equal(paint.radius, '0px');
  assert.equal(paint.extent, 27);
  const extent = Math.min(paint.extent, paint.width / 2, paint.height / 2);
  const expected = createServerCornerGeometry({ width: paint.compact ? extent * 2 + 1 : paint.width,
    height: paint.compact ? extent * 2 + 1 : paint.height, extent, tolerance: .02 });
  assert.equal(paint.actualPath, expected.path, 'The painted button must use the existing project curve');
  assert.ok(expected.corners.every(corner => corner.extent > 0));
}

async function choose(page, button, keyboard = false) {
  const dialog = page.waitForEvent('filechooser');
  if (keyboard) { await button.focus(); await button.press('Enter'); }
  else await button.click();
  await (await dialog).setFiles(image);
}

try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const language of ['zh-CN', 'en']) {
        const english = language === 'en', width = english ? 1280 : 390;
        const context = await browser.newContext({ locale: language, viewport: { width, height: 900 }, colorScheme: english ? 'dark' : 'light' });
        try {
          const page = await context.newPage(), errors = [], removedCalls = [], uploads = [], pending = [];
          page.on('pageerror', error => errors.push(error.message));
          page.on('request', request => { if (/\/api\/v1\/content\/users(?:\/|\?|$)/.test(request.url())) removedCalls.push(request.url()); });
          await mockApi(page, { authenticated: true });
          await page.route('**/api/v1/content/upload', route => {
            const request = route.request();
            assert.equal(request.method(), 'POST');
            assert.equal(request.headers()['content-type'], image.mimeType);
            assert.equal(request.headers()['x-file-name'], encodeURIComponent(image.name));
            assert.equal(request.headers()['x-csrf-token'], session.csrf_token);
            assert.deepEqual(request.postDataBuffer(), image.buffer);
            uploads.push(request);
            pending.push(() => route.fulfill({ json: { id: uploads.length, path: `/media/${String(uploads.length).repeat(64)}.png`, mime_type: image.mimeType, size: image.buffer.length } }));
          });
          const fieldLabel = english ? 'Upload image (PNG, JPEG, GIF, WebP)' : '上传图片（PNG、JPEG、GIF、WebP）';
          const triggerName = label => (english ? 'Choose file: ' : '选择文件：') + label;
          await page.goto(`${base}/admin?lang=${language}#dashboard`);
          await expect(page.getByRole('heading', { name: english ? 'Overview' : '总览', exact: true })).toBeVisible();
          await expect(page.locator('.xocs-dashboard-sections a[href="/admin#users"]')).toHaveCount(0);
          await page.locator('.xcss-header-navigation').getByRole('button', { name: english ? 'Community' : '社区', exact: true }).click();
          await expect(page.locator('.xcss-secondary-navigation').getByRole('button')).toHaveCount(2);
          await expect(page.getByRole('button', { name: english ? 'Users' : '用户管理', exact: true })).toHaveCount(0);
          await page.getByRole('button', { name: english ? 'Account settings' : '账号设置', exact: true }).click();
          await expect(page.getByRole('region', { name: english ? 'Account settings' : '账号设置', exact: true })).toBeVisible();
          await expect(page.getByLabel(english ? 'Current password' : '当前密码', { exact: true })).toBeVisible();

          await page.goto(`${base}/admin?lang=${language}#resources`);
          const file = page.getByLabel(fieldLabel, { exact: true });
          const button = page.getByRole('button', { name: triggerName(fieldLabel), exact: true });
          await expect(file).toBeHidden();
          await expect(file).toHaveAttribute('accept', 'image/png,image/jpeg,image/gif,image/webp');
          await expect(button).toHaveAttribute('type', 'button');
          await assertProjectCurve(button);
          if (output) await button.screenshot({ path: path.join(output, `file-button-${engine.name()}-${language}.png`) });
          await choose(page, button, true);
          assert.equal(uploads.length, 0, 'Choosing a resource file must not submit the form');
          await expect(page.locator('.xocs-file-picker-name')).toHaveText(image.name);
          const upload = page.getByRole('button', { name: english ? 'Upload' : '上传', exact: true });
          await upload.click(); await expect.poll(() => pending.length).toBe(1);
          await expect(button).toBeDisabled(); await expect(file).toBeDisabled();
          await expect(page.getByRole('button', { name: english ? 'Uploading…' : '上传中…', exact: true })).toBeDisabled();
          await pending.shift()();
          await expect(button).toBeEnabled();
          await expect(page.locator('.xocs-file-picker-name')).toHaveText(english ? 'No file chosen' : '未选择文件');
          assert.equal(await file.evaluate(node => node.files.length), 0);
          await choose(page, button);
          await expect(page.locator('.xocs-file-picker-name')).toHaveText(image.name);
          assert.equal(uploads.length, 1, 'The same file is selectable again after the native input is cleared');
          assert.deepEqual((await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze()).violations, []);

          await page.goto(`${base}/admin?lang=${language}#new`);
          const coverLabel = english ? 'Upload cover image' : '上传封面图片', contentLabel = english ? 'Insert content image' : '插入正文图片';
          const cover = page.getByRole('button', { name: triggerName(coverLabel), exact: true });
          const content = page.getByRole('button', { name: triggerName(contentLabel), exact: true });
          await assertProjectCurve(cover); await assertProjectCurve(content);
          const body = page.locator('textarea.editor-textarea');
          await expect(body).toHaveAccessibleName(english ? 'Markdown content' : 'Markdown 正文');
          await body.fill('Original draft');
          await choose(page, cover); await expect.poll(() => pending.length).toBe(1);
          await expect(cover).toBeDisabled(); await expect(content).toBeDisabled();
          assert.equal(await page.getByLabel(coverLabel, { exact: true }).evaluate(node => node.files.length), 0);
          await pending.shift()();
          await expect(page.getByLabel(english ? 'Cover image URL' : '封面图片地址', { exact: true })).toHaveValue(`/media/${'2'.repeat(64)}.png`);
          await choose(page, content, true); await expect.poll(() => pending.length).toBe(1);
          await pending.shift()();
          await expect(body).toHaveValue(`Original draft\n\n![${image.name}](/media/${'3'.repeat(64)}.png)\n`);
          assert.equal(await page.getByLabel(contentLabel, { exact: true }).evaluate(node => node.files.length), 0);
          assert.equal(uploads.length, 3);
          for (const removed of ['/about', '/weiYan']) {
            await page.goto(`${base}${removed}?lang=${language}`);
            await expect(page.getByRole('heading', { name: english ? 'Page not found' : '页面不存在', exact: true }).first()).toBeVisible();
            await expect(page.getByRole('navigation', { name: english ? 'Main navigation' : '主导航', exact: true })
              .locator(`a[href^="${removed}"]`)).toHaveCount(0);
          }
          await page.goto(`${base}/jotting?lang=${language}`);
          await expect(page.getByRole('heading', { name: 'xocs', exact: true }).first()).toBeVisible();
          assert.deepEqual(removedCalls, []);
          assert.deepEqual(errors, []);
          assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
          console.log(`${engine.name()} ${width}px ${language}: removed menus/pages, avatar account, three native file pickers and shared curve paint passed`);
        } finally { await context.close(); }
      }
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
