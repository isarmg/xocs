import assert from 'node:assert/strict';
import { createServer, request } from 'node:http';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi } from './browser-fixtures.mjs';

// Keep the real HTTP body open after sending headers the Rust API must reject.
// A UI error alone is insufficient: the server must observe request cancellation
// while the page remains open, without waiting for its request timeout.
const assets = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const assetBase = `http://127.0.0.1:${assets.httpServer.address().port}`;
const pending = new Set();
let opened = 0;
let canceled = 0;
const server = createServer((incoming, outgoing) => {
  if (incoming.url === '/api/v1/categories') {
    opened += 1;
    pending.add(outgoing);
    outgoing.writeHead(200, { 'content-type': 'text/html', 'cache-control': 'no-store' });
    outgoing.write('<html>invalid API response');
    const interval = setInterval(() => outgoing.write(' '), 50);
    outgoing.once('close', () => {
      clearInterval(interval);
      pending.delete(outgoing);
      if (!outgoing.writableFinished) canceled += 1;
    });
    return;
  }
  const upstream = request(new URL(incoming.url, assetBase), {
    method: incoming.method,
    headers: { ...incoming.headers, host: new URL(assetBase).host },
  }, response => {
    outgoing.writeHead(response.statusCode, response.headers);
    response.pipe(outgoing);
  });
  upstream.on('error', error => outgoing.destroy(error));
  incoming.pipe(upstream);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;

try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ locale: 'en' });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await mockApi(page, { authenticated: true });
      await page.route('**/api/v1/categories', route => route.continue());
      const expected = opened + 1;
      await page.goto(`${base}/admin?lang=en#new`);
      await expect(page.locator('html')).toHaveAttribute('data-blog-runtime', 'blog-interface');
      await expect.poll(() => opened).toBe(expected);
      await expect(page.getByRole('alert').first()).toBeVisible();
      await expect.poll(() => canceled, { timeout: 3_000 }).toBe(expected);
      assert.equal(pending.size, 0, 'rejected headers close the still-streaming response');
      assert.deepEqual(errors, []);
      console.log(`${engine.name()}: rejected API headers cancel the live HTTP body`);
    } finally {
      await browser.close();
    }
  }
} finally {
  for (const response of pending) response.destroy();
  await new Promise(resolve => server.close(resolve));
  await new Promise(resolve => assets.httpServer.close(resolve));
}
