import assert from 'node:assert/strict';
import { chromium, firefox, expect } from '../../../browser/dependencies.mjs';
import { mockApi } from '../../../browser/browser-fixtures.mjs';
import { preview } from '../../../tools/assets/test-preview.mjs';

// Faults are injected at the browser's public DOM boundary; the WASM renderer
// and startup controller run unchanged. Observe the original exception object.
const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const operation of ['attribute', 'insert', 'listener', 'property', 'readonly-property']) {
        const context = await browser.newContext({ locale: 'en-US' });
        try {
          const page = await context.newPage();
          await mockApi(page, { authenticated: operation === 'property' || operation === 'readonly-property' });
          await page.addInitScript(operation => {
            const source = new DOMException(`DOM boundary ${operation} fixture`, 'InvalidStateError');
            const state = { injected: false, original: false, message: null };
            window.domBoundaryFailure = state;
            const observe = error => {
              if (error === source) {
                state.original = true;
                state.message = error.message;
              } else if (operation === 'readonly-property' && error instanceof TypeError) {
                state.message = error.message;
              }
            };
            window.addEventListener('error', event => observe(event.error));
            window.addEventListener('unhandledrejection', event => observe(event.reason));
            const log = console.error;
            console.error = (...args) => {
              args.forEach(observe);
              log.apply(console, args);
            };

            if (operation === 'attribute') {
              const setAttribute = Element.prototype.setAttribute;
              Element.prototype.setAttribute = function (key, value) {
                if (key === 'class') {
                  Element.prototype.setAttribute = setAttribute;
                  state.injected = true;
                  throw source;
                }
                return setAttribute.call(this, key, value);
              };
            } else if (operation === 'insert') {
              const insertBefore = Node.prototype.insertBefore;
              Node.prototype.insertBefore = function (child, marker) {
                if (this instanceof Element && this.id === 'root') {
                  Node.prototype.insertBefore = insertBefore;
                  state.injected = true;
                  throw source;
                }
                return insertBefore.call(this, child, marker);
              };
            } else if (operation === 'listener') {
              const addEventListener = EventTarget.prototype.addEventListener;
              EventTarget.prototype.addEventListener = function (name, callback, options) {
                if (this instanceof Element) {
                  EventTarget.prototype.addEventListener = addEventListener;
                  state.injected = true;
                  throw source;
                }
                return addEventListener.call(this, name, callback, options);
              };
            } else {
              const set = Reflect.set;
              Reflect.set = function (target, key, value, ...receiver) {
                if (target instanceof Element && key === 'value') {
                  Reflect.set = set;
                  state.injected = true;
                  if (operation === 'readonly-property') return false;
                  throw source;
                }
                return set(target, key, value, ...receiver);
              };
            }
          }, operation);
          await page.goto(new URL('/admin?lang=en#labels', server.resolvedUrls.local[0]).href);
          await expect.poll(() => page.evaluate(() => window.domBoundaryFailure.message)).not.toBeNull();
          const failure = await page.evaluate(() => window.domBoundaryFailure);
          assert.equal(failure.injected, true, 'The real WASM renderer reached the failing DOM operation');
          if (operation === 'readonly-property') {
            assert.equal(failure.message, 'DOM property assignment was rejected');
          } else {
            assert.equal(failure.original, true, 'The exact original JS exception reaches the outer boundary');
          }
          if (['attribute', 'insert'].includes(operation)) {
            await expect(page.locator('#blog-startup-error')).toBeVisible();
            await expect(page.locator('html')).toHaveAttribute('data-blog-startup', 'failed');
            assert.equal(await page.locator('html').getAttribute('data-blog-runtime'), null);
          }
          console.log(`${engine.name()}: ${operation} failure reaches the JS boundary without silent success`);
        } finally {
          await context.close();
        }
      }
    } finally {
      await browser.close();
    }
  }
} finally {
  await server.close();
}
