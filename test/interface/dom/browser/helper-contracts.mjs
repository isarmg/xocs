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
const transitionFailures = [];
const timerCleanupFailures = [];
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
      const transitions = await page.evaluate(() => {
        customElements.define('dom-attribute-probe', class extends HTMLElement {
          static observedAttributes = ['data-first', 'data-second'];
          attributeChangedCallback() { this.dispatchEvent(new MouseEvent('click')); }
        });
        return [
          'either-right', 'either-left', 'either-dynamic-key', 'erased-type', 'erased-same-key',
          'erased-dynamic-key', 'erased-retained-keys', 'erased-property',
          'erased-inner-html', 'erased-listener', 'either-property',
          'either-inner-html', 'either-listener', 'vector-listener',
        ].map(operation => {
          const element = document.createElement('dom-attribute-probe');
          let called = 0;
          fixture.attribute_transition(operation, element, () => { called++; });
          const first = element.getAttribute('data-first');
          const second = element.getAttribute('data-second');
          let passed;
          switch (operation) {
            case 'either-right':
            case 'either-left': passed = first === null; break;
            case 'erased-type': passed = first === null && second === ''; break;
            case 'erased-same-key': passed = first === 'new'; break;
            case 'either-dynamic-key':
            case 'erased-dynamic-key': passed = first === null && second === 'same'; break;
            case 'erased-retained-keys': passed = first === null && second === null; break;
            case 'either-property':
            case 'erased-property': passed = element.contractProperty === undefined; break;
            case 'either-inner-html':
            case 'erased-inner-html': passed = element.childNodes.length === 0; break;
            case 'vector-listener':
            case 'either-listener':
            case 'erased-listener': passed = called === 0 && second === 'new'; break;
          }
          return { operation, passed, first, second, called };
        });
      });
      for (const result of transitions) {
        if (!result.passed) transitionFailures.push({ engine: engine.name(), ...result });
      }
      console.log(`${engine.name()}: attribute transitions ${JSON.stringify(transitions)}`);
      assert.deepEqual(await page.evaluate(async () => {
        const results = [];
        for (const operation of ['either', 'erased']) {
          const element = document.createElement('div');
          await fixture.reactive_attribute_transition(operation, element);
          results.push({ operation, value: element.getAttribute('data-reactive') });
        }
        return results;
      }), [{ operation: 'either', value: null }, { operation: 'erased', value: null }], 'A removed reactive attribute must stay removed after its signal changes');
      assert.equal(await page.evaluate(() => {
        const element = document.createElement('dom-attribute-probe');
        const remove = element.removeAttribute;
        const source = new DOMException('attribute vector removal', 'InvalidStateError');
        let called = 0;
        element.removeAttribute = () => { throw source; };
        let error;
        try { fixture.attribute_transition('vector-listener', element, () => { called++; }); }
        catch (caught) { error = caught; }
        finally { element.removeAttribute = remove; }
        element.dispatchEvent(new MouseEvent('click'));
        return error === source && called === 0;
      }), true, 'A failed vector key removal must preserve its source and cancel every old listener first');
      const timerCleanup = await page.evaluate(() => ['timeout', 'interval', 'timeout-range', 'interval-range'].map(operation => {
        const method = operation.startsWith('timeout') ? 'setTimeout' : 'setInterval';
        const original = window[method];
        const source = new DOMException('timer cleanup', 'InvalidStateError');
        let dropped = 0;
        let error;
        window[method] = () => { throw source; };
        try { fixture.timer_failure_cleanup(operation, () => { dropped++; }); }
        catch (caught) { error = caught; }
        finally { window[method] = original; }
        const expectedError = operation.endsWith('-range')
          ? error instanceof RangeError && error.message === 'timer delay exceeds i32 milliseconds'
          : error === source;
        return { operation, expectedError, dropped };
      }));
      for (const result of timerCleanup) {
        if (!result.expectedError || result.dropped !== 1) timerCleanupFailures.push({ engine: engine.name(), ...result });
      }
      console.log(`${engine.name()}: timer registration cleanup ${JSON.stringify(timerCleanup)}`);
      console.log(`${engine.name()}: timer failure sources and ranges, window listener RAII, explicit cleanup and destructor failure contracts passed`);
      await page.close();
    } finally {
      await browser.close();
    }
  }
} finally {
  await new Promise(resolve => server.close(resolve));
}
assert.deepEqual(transitionFailures, [], 'Attribute transitions must remove the old effect, listener and DOM keys before building the new state');
assert.deepEqual(timerCleanupFailures, [], 'Failed timer registration must drop its Rust callback capture once and preserve the original host error');
