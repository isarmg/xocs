import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
const submission = { request_id: '12345678-1234-4123-8123-123456789abc', content: 'The original pending comment', parent_comment_id: null };
const cases = [
  { name: 'article target differs', page: '/article/1', endpoint: '/api/v1/comments', article: 1, payload: { ...submission, article_id: 2 } },
  { name: 'article target missing', page: '/article/1', endpoint: '/api/v1/comments', article: 1, payload: submission },
  { name: 'message target added', page: '/message', endpoint: '/api/v1/message-comments', article: 0, payload: { ...submission, article_id: 1 } },
  { name: 'love target added', page: '/love', endpoint: '/api/v1/love-comments', article: 0, payload: { ...submission, article_id: 1 } },
  { name: 'message target null', page: '/message', endpoint: '/api/v1/message-comments', article: 0, payload: { ...submission, article_id: null } },
  { name: 'unknown field', page: '/article/1', endpoint: '/api/v1/comments', article: 1, payload: { ...submission, article_id: 1, unexpected: true } },
  { name: 'article target matches', page: '/article/1', endpoint: '/api/v1/comments', article: 1, payload: { ...submission, article_id: 1 }, valid: true },
  { name: 'message target matches', page: '/message', endpoint: '/api/v1/message-comments', article: 0, payload: submission, valid: true },
  { name: 'love target matches', page: '/love', endpoint: '/api/v1/love-comments', article: 0, payload: submission, valid: true },
];
const results = [];
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const scenario of cases) {
        const page = await browser.newPage();
        const requests = [], errors = [];
        page.on('pageerror', error => errors.push(error.message));
        try {
          await mockApi(page, { authenticated: false });
          await page.route(`**${scenario.endpoint}`, route => {
            assert.equal(route.request().method(), 'POST');
            requests.push(route.request().postDataJSON());
            return route.fulfill({ json: { id: 1, user_id: null, username: null, avatar: null, comment_content: scenario.payload.content, create_time: null, parent_username: null, reply_count: 0 } });
          });
          const key = `blog.comment-submission:${scenario.endpoint}:${scenario.article}`;
          const raw = JSON.stringify(scenario.payload);
          await page.addInitScript(({ key, raw }) => sessionStorage.setItem(key, raw), { key, raw });
          await page.goto(`${base}${scenario.page}?lang=en`);
          if (scenario.page === '/love') await page.locator('.love-tabs').getByRole('button', { name: /^Wishes/ }).click();
          const comments = page.locator('.article-comments');
          await expect(comments).toBeVisible();
          if (scenario.valid) {
            const confirm = comments.getByRole('button', { name: 'Confirm original submission', exact: true });
            await expect(confirm).toBeEnabled();
            await expect(comments.getByRole('textbox')).toHaveValue(scenario.payload.content);
            await confirm.click();
            await expect(comments.getByText('Your comment has been posted.', { exact: true })).toBeVisible();
            assert.deepEqual(requests, [scenario.payload]);
            assert.equal(await page.evaluate(key => sessionStorage.getItem(key), key), null);
          } else {
            const confirm = comments.getByRole('button', { name: 'Confirm original submission', exact: true });
            if (await confirm.count()) {
              await confirm.click();
              await expect.poll(() => requests.length).toBe(1);
            }
            assert.deepEqual(requests, [], 'A mismatched pending submission must not be sent');
            await expect(comments.getByRole('alert')).toContainText('The pending submission could not be read or saved.');
            await expect(comments.getByRole('button', { name: 'Post anonymous comment', exact: true })).toBeDisabled();
            assert.equal(await page.evaluate(key => sessionStorage.getItem(key), key), raw);
          }
          assert.deepEqual(errors, []);
          results.push({ engine: engine.name(), scenario: scenario.name, passed: true });
        } finally {
          await page.close();
        }
      }
    } finally {
      await browser.close();
    }
  }
  console.log(JSON.stringify(results));
} finally {
  await new Promise(resolve => server.httpServer.close(resolve));
}
