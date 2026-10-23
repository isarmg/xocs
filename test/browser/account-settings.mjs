import assert from 'node:assert/strict';
import {chromium, firefox, expect} from './dependencies.mjs';
import { AxeBuilder } from './dependencies.mjs';
import {preview} from '../tools/assets/test-preview.mjs';
import {mockApi, session} from './browser-fixtures.mjs';
import {paintedSurfaceColor} from './painted-surface.mjs';

const server = await preview({build: {outDir: process.env.BLOG_WEB_DIST || 'dist'}, preview: {host: '127.0.0.1', port: 0, strictPort: true}});
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
const appearance = element => {
  const style = getComputedStyle(element), form = getComputedStyle(element.querySelector('form'));
  return {radius: style.borderRadius, background: style.backgroundColor, font: form.fontFamily};
};
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const language of ['zh-CN', 'en']) for (const width of [320, 1280]) {
        const english = language === 'en';
        const context = await browser.newContext({locale: language, viewport: {width, height: 800}, colorScheme: width === 320 ? 'light' : 'dark'});
        try {
          const page = await context.newPage(), state = {authenticated: false}, errors = [];
          let updates = 0;
          page.on('pageerror', error => errors.push(error.message));
          await mockApi(page, state);
          await page.route('**/api/v1/platform/administrators/self', route => {
            updates++;
            const request = route.request(), input = request.postDataJSON();
            assert.equal(request.method(), 'POST');
            assert.equal(request.headers()['x-csrf-token'], session.csrf_token);
            assert.equal(input.username, 'renamed');
            if (input.current_password !== 'correct-password') return route.fulfill({status: 403, headers: {'x-request-id':'private-blog-id'}, json: {code: 'admin.current_password_invalid', message: 'SECRET internal details', retryable: false, request_id: 'private-blog-id'}});
            assert.equal(input.new_password, 'updated-password');
            state.authenticated = false;
            return route.fulfill({status: 204});
          });
          await page.goto(`${base}/admin?lang=${language}`);
          const login = page.locator('.blog-auth-card');
          await expect(login).toBeVisible();
          await expect(page.locator('html')).toHaveAttribute('data-blog-fonts', 'ready');
          await page.evaluate(() => document.fonts.ready);
          const loginStyle = await login.evaluate(appearance);
          loginStyle.background = await paintedSurfaceColor(login);
          state.authenticated = true;
          await page.reload();
          const trigger = page.getByRole('button', {name: english ? 'Account settings' : '账号设置', exact: true});
          const icon = await trigger.evaluate(node => ({image:node.querySelector('svg').innerHTML,decoration:getComputedStyle(node).textDecorationLine}));
          await trigger.click();
          assert.deepEqual(await trigger.evaluate(node => ({image:node.querySelector('svg').innerHTML,decoration:getComputedStyle(node).textDecorationLine})),icon);
          await expect(page.locator('.blog-header-navigation [aria-pressed="true"]')).toHaveCount(0);
          await expect(page.locator('.blog-secondary-navigation')).toHaveCount(0);
          const dialog = page.getByRole('region', {name: english ? 'Account settings' : '账号设置', exact: true});
          await page.evaluate(() => document.fonts.ready);
          await expect(page.getByRole('dialog')).toHaveCount(0);
          await expect(page).toHaveURL(/#account$/);
          const panel = dialog.locator('.blog-content-panel');
          const panelStyle = await panel.evaluate(element => ({background:getComputedStyle(element).backgroundColor,font:getComputedStyle(element).fontFamily}));
          await expect.poll(() => paintedSurfaceColor(panel)).toBe(loginStyle.background);
          assert.equal(panelStyle.font, loginStyle.font);
          const fields = dialog.locator('.blog-form-field');
          const labels = english ? ['Username', 'Current password', 'New password', 'Confirm new password'] : ['用户名', '当前密码', '新密码', '确认新密码'];
          for (const label of labels) await expect(dialog.getByLabel(label, {exact: true})).toBeVisible();
          await expect(fields).toHaveCount(4);
          await expect(dialog.getByRole('button')).toHaveCount(1);
          const save = dialog.getByRole('button', {name: english ? 'Save' : '保存', exact: true});
          await save.click();
          await expect(dialog.getByRole('alert')).toBeVisible();
          assert.equal(updates, 0);
          assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
          assert.deepEqual((await new AxeBuilder({page}).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze()).violations, []);
          await dialog.getByLabel(labels[0], {exact: true}).fill('renamed');
          await dialog.getByLabel(labels[1], {exact: true}).fill('incorrect-password');
          await dialog.getByLabel(labels[2], {exact: true}).fill('updated-password');
          await dialog.getByLabel(labels[3], {exact: true}).fill('mismatched-password');
          await save.click();
          await expect(dialog.getByRole('alert')).toContainText(english ? 'do not match' : '不一致');
          assert.equal(updates, 0);
          await dialog.getByLabel(labels[3], {exact: true}).fill('updated-password');
          await save.click();
          await expect(dialog.getByRole('alert')).toContainText(english ? 'password is incorrect' : '密码不正确');
          for (const label of labels.slice(1)) await expect(dialog.getByLabel(label, {exact: true})).toHaveValue('');
          await expect(page.locator('body')).not.toContainText('SECRET');
          await expect(dialog).not.toContainText('private-blog-id');
          await dialog.getByLabel(labels[1], {exact: true}).fill('correct-password');
          for (const label of labels.slice(2)) await dialog.getByLabel(label, {exact: true}).fill('updated-password');
          await save.click();
          await expect(page.getByRole('heading', {name: english ? 'Administrator sign in' : '管理员登录', exact: true})).toBeVisible();
          await expect(page.getByRole('status')).toContainText(english ? 'Account updated' : '账号已更新');
          assert.equal(updates, 2);
          assert.deepEqual(errors, []);
          console.log(`${engine.name()} ${width}px ${language}: ordinary account page, shared palette/font, password validation and sign-in passed`);
        } finally { await context.close(); }
      }
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
