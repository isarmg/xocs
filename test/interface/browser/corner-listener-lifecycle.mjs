import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, firefox } from '../../browser/dependencies.mjs';

const output = fileURLToPath(new URL('../../../target/verification/anti-defensive-review/corner-listener-fixture/', import.meta.url));
const packageRoot = resolve(output, 'pkg');
const server = createServer(async (request, response) => {
  try {
    const path = new URL(request.url, 'http://fixture.invalid').pathname;
    if (path === '/') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end('<!doctype html><title>Corner listener consumer</title><body>Public interface consumer</body>');
      return;
    }
    assert.ok(path.startsWith('/pkg/'));
    const file = resolve(packageRoot, decodeURIComponent(path.slice('/pkg/'.length)));
    assert.ok(file.startsWith(`${packageRoot}/`));
    const bytes = await readFile(file);
    response.writeHead(200, { 'content-type': file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
    response.end(bytes);
  } catch (error) {
    response.writeHead(404);
    response.end(String(error));
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const results = [];
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const operation of ['stop-before-cross-origin', 'pagehide-before-cross-origin', 'failed-removal-keeps-callback']) {
        const page = await browser.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        try {
          await page.goto(base);
          const result = await page.evaluate(async operation => {
            const fixture = await import('/pkg/dom_contracts.js');
            await fixture.default({ module_or_path: '/pkg/dom_contracts_bg.wasm' });
            const frame = document.createElement('iframe');
            const ready = new Promise(resolve => frame.addEventListener('load', resolve, { once: true }));
            document.body.append(frame);
            await ready;
            const view = frame.contentWindow;
            const documentInFrame = frame.contentDocument;
            documentInFrame.body.innerHTML = '<div data-corner-listener-surface data-blog-server-corner-extent="12" style="display:block;width:160px;height:100px;border:1px solid black;background:white">Surface</div>';
            const surface = documentInFrame.querySelector('[data-corner-listener-surface]');
            const originalAdd = view.addEventListener;
            const originalRemove = view.removeEventListener;
            let callback;
            let removals = 0;
            view.addEventListener = function (name, listener, options) {
              if (name === 'pagehide') callback = listener;
              return originalAdd.call(this, name, listener, options);
            };
            view.removeEventListener = function (name, listener, options) {
              if (name === 'pagehide') removals++;
              return originalRemove.call(this, name, listener, options);
            };
            fixture.start_corner_controller(documentInFrame);
            const initialPaint = surface.getAttribute('data-blog-server-corner-state');
            const failure = error => ({ name: error.name, message: error.message });
            const attempt = run => {
              try { run(); return null; } catch (error) { return failure(error); }
            };
            if (operation === 'failed-removal-keeps-callback') {
              const source = new DOMException('corner listener removal fixture', 'InvalidStateError');
              let originalObserved = false;
              const warn = console.warn;
              const error = console.error;
              const observe = values => { originalObserved ||= values.some(value => value === source); };
              console.warn = (...values) => { observe(values); warn.apply(console, values); };
              console.error = (...values) => { observe(values); error.apply(console, values); };
              view.removeEventListener = function (name, listener, options) {
                if (name === 'pagehide') { removals++; throw source; }
                return originalRemove.call(this, name, listener, options);
              };
              try {
                const stopError = attempt(() => fixture.stop_corner_controller());
                const dropError = attempt(() => fixture.drop_corner_controller());
                const callbackError = attempt(() => callback(new view.Event('pagehide')));
                return { initialPaint, stopError, dropError, callbackError, originalObserved, removals };
              } finally {
                console.warn = warn;
                console.error = error;
                view.removeEventListener = originalRemove;
                originalRemove.call(view, 'pagehide', callback, false);
                frame.remove();
              }
            }
            const stopError = operation === 'stop-before-cross-origin'
              ? attempt(() => fixture.stop_corner_controller()) : null;
            const navigation = new Promise(resolve => frame.addEventListener('load', resolve, { once: true }));
            frame.src = 'data:text/html,<!doctype html><title>Opaque frame</title><body>Cross origin</body>';
            await navigation;
            const sameWindowProxy = view === frame.contentWindow;
            const restoredBeforeDrop = !surface.hasAttribute('data-blog-server-corner-state');
            const removalsBeforeDrop = removals;
            const dropError = attempt(() => fixture.drop_corner_controller());
            frame.remove();
            return { initialPaint, stopError, dropError, sameWindowProxy, restoredBeforeDrop, removalsBeforeDrop, removals };
          }, operation);
          const failures = [];
          if (!/^(paint|clip)$/.test(result.initialPaint ?? '')) failures.push('The public controller did not paint the iframe surface');
          if (result.stopError) failures.push(`stop threw ${JSON.stringify(result.stopError)}`);
          if (result.dropError) failures.push(`drop threw ${JSON.stringify(result.dropError)}`);
          if (operation === 'failed-removal-keeps-callback') {
            if (!result.originalObserved) failures.push('The original browser exception did not reach the cleanup boundary');
            if (result.callbackError) failures.push(`retained listener callback threw ${JSON.stringify(result.callbackError)}`);
          } else {
            if (!result.sameWindowProxy) failures.push('Frame navigation did not retain the WindowProxy');
            if (!result.restoredBeforeDrop) failures.push('Explicit stop or pagehide did not restore the iframe surface before drop');
          }
          if (errors.length) failures.push(`Unhandled browser errors: ${JSON.stringify(errors)}`);
          const record = { engine: engine.name(), operation, ...result, errors, failures };
          results.push(record);
          console.log(JSON.stringify(record));
        } finally {
          await page.close();
        }
      }
    } finally {
      await browser.close();
    }
  }
  await writeFile(resolve(output, 'results.json'), `${JSON.stringify(results, null, 2)}\n`);
  assert.deepEqual(results.flatMap(result => result.failures), [], 'Public corner-controller cleanup remains safe after navigation and browser failures');
} finally {
  await new Promise(resolve => server.close(resolve));
}
