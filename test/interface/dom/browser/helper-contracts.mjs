import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import { chromium, firefox } from '../../../browser/dependencies.mjs';

const directory = resolve(process.argv[2] ?? 'target/verification/anti-defensive-review/dom-fixture/pkg');
const files = new Map(await Promise.all([
  ['dom_contracts.js', 'text/javascript'],
  ['dom_contracts_bg.wasm', 'application/wasm'],
].map(async ([name, type]) => [`/${name}`, { type, body: await readFile(resolve(directory, name)) }])));
const server = createServer((request, response) => {
  const path = new URL(request.url, 'http://127.0.0.1').pathname;
  if (path === '/') {
    response.writeHead(200, { 'content-type': 'text/html' });
    response.end('<!doctype html><script type="module">window.fixture=await import("/dom_contracts.js");await fixture.default({module_or_path:"/dom_contracts_bg.wasm"});window.fixtureReady=true;</script>');
  } else if (files.has(path)) {
    const { type, body } = files.get(path);
    response.writeHead(200, { 'content-type': type });
    response.end(body);
  } else {
    response.writeHead(404);
    response.end();
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage();
      await page.goto(base);
      await page.waitForFunction(() => window.fixtureReady);
      const results = await page.evaluate(() => {
        const operations = {
          raf: 'requestAnimationFrame',
          'cancel-raf': 'cancelAnimationFrame',
          idle: 'requestIdleCallback',
          microtask: 'queueMicrotask',
          timeout: 'setTimeout',
          interval: 'setInterval',
          'callback-timeout': 'setTimeout',
          'callback-interval': 'setInterval',
        };
        const results = [];
        for (const [operation, method] of Object.entries(operations)) {
          const original = window[method];
          const source = new DOMException(operation, 'InvalidStateError');
          window[method] = () => { throw source; };
          let error;
          try { fixture.call_helper(operation); } catch (caught) { error = caught; }
          finally { window[method] = original; }
          results.push({ operation, original: error === source });
        }
        for (const operation of ['timeout-range', 'interval-range', 'callback-timeout-range', 'callback-interval-range']) {
          let error;
          try { fixture.call_helper(operation); } catch (caught) { error = caught; }
          results.push({ operation, range: error instanceof RangeError, message: error?.message });
        }
        for (let index = 0; index < 50; index++) {
          let called = false;
          fixture.listen(() => { called = true; });
          fixture.drop_listener();
          window.dispatchEvent(new Event('dom-contract'));
          if (called) throw Error('Dropping the window listener must unregister its callback');
        }
        let active = false;
        fixture.listen(() => { active = true; });
        window.dispatchEvent(new Event('dom-contract'));
        if (!active) throw Error('A retained window listener must deliver events');
        fixture.remove_listener();
        active = false;
        window.dispatchEvent(new Event('dom-contract'));
        if (active) throw Error('Explicit removal must unregister the callback');
        return results;
      });
      for (const result of results) {
        if ('original' in result) assert.equal(result.original, true, result.operation);
        else {
          assert.equal(result.range, true, result.operation);
          assert.equal(result.message, 'timer delay exceeds i32 milliseconds');
        }
      }
      assert.equal(await page.evaluate(async () => {
        const input = document.createElement('input');
        input.value = 'before';
        const original = window.queueMicrotask;
        const source = new DOMException('queued DOM write', 'InvalidStateError');
        window.queueMicrotask = () => { throw source; };
        let error;
        try { fixture.queue_value(input, 'failed'); } catch (caught) { error = caught; }
        finally { window.queueMicrotask = original; }
        if (error !== source || input.value !== 'before') return false;
        fixture.queue_value(input, 'after');
        await new Promise(resolve => queueMicrotask(resolve));
        return input.value === 'after';
      }), true, 'A failed scheduler must release queued writes and allow future DOM work');
      for (const mode of ['registration', 'drop', 'remove']) {
        const result = await page.evaluate(mode => {
          const method = mode === 'registration' ? 'addEventListener' : 'removeEventListener';
          const original = window[method];
          const log = console.error;
          const source = new DOMException(mode, 'InvalidStateError');
          let logged = false;
          console.error = (...args) => { logged ||= args.includes(source); };
          let callbackAlive = false;
          if (mode !== 'registration') fixture.listen(() => { callbackAlive = true; });
          window[method] = () => { throw source; };
          let error;
          try {
            if (mode === 'registration') fixture.listen(() => {});
            else if (mode === 'drop') fixture.drop_listener();
            else fixture.remove_listener();
          } catch (caught) { error = caught; }
          finally { window[method] = original; console.error = log; }
          if (mode !== 'registration') window.dispatchEvent(new Event('dom-contract'));
          return { mode, original: error === source, logged, callbackAlive, threw: error !== undefined };
        }, mode);
        if (mode === 'drop') {
          assert.equal(result.threw, false, 'Destructor fallback must not throw');
          assert.equal(result.logged, true, 'Destructor fallback reports the original host error');
        } else {
          assert.equal(result.original, true, 'Explicit operations preserve the original host error');
          assert.equal(result.logged, false, 'Explicit operations leave reporting to the caller');
        }
        if (mode !== 'registration') assert.equal(result.callbackAlive, true, 'Failed removal must leave a registered WASM callback callable');
      }
      console.log(`${engine.name()}: timer failure sources and ranges, window listener RAII, explicit cleanup and destructor failure contracts passed`);
      await page.close();
    } finally {
      await browser.close();
    }
  }
} finally {
  await new Promise(resolve => server.close(resolve));
}
