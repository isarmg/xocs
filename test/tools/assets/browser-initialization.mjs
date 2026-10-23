import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

// Executed against actual generated bindings and transformed Wasm. The module's
// start function increments an exported global, so skipped or repeated startup
// is observable without matching the generator's source text.
const bytes = await readFile(new URL("browser_bg.wasm", import.meta.url));
const compiled = await WebAssembly.compile(bytes);
let instance = 0;
const fresh = () => import(new URL(`browser.js?instance=${instance++}`, import.meta.url));
const response = (type = "application/wasm") => new Response(bytes, {
  headers: { "Content-Type": type },
});
const fetched = [];
globalThis.fetch = async (input) => {
  fetched.push(input instanceof Request ? input.url : String(input));
  return response();
};

for (const module of [bytes, compiled]) {
  const api = await fresh();
  const exports = api.initSync({ module });
  assert.equal(exports.starts.value, 1);
  assert.equal(api.initSync({ module }), exports);
  assert.equal(await api.default({ module_or_path: module }), exports);
  assert.equal(exports.starts.value, 1);
}

for (const module_or_path of [
  bytes,
  compiled,
  Promise.resolve(bytes),
  response(),
  Promise.resolve(response()),
  "https://example.test/browser.wasm",
  new URL("https://example.test/browser.wasm"),
  new Request("https://example.test/browser.wasm"),
]) {
  const api = await fresh();
  const exports = await api.default({ module_or_path });
  assert.equal(exports.starts.value, 1);
  assert.equal(await api.default({ module_or_path }), exports);
  assert.equal(api.initSync({ module: compiled }), exports);
  assert.equal(exports.starts.value, 1);
}
assert.deepEqual(fetched, Array(3).fill("https://example.test/browser.wasm"));

for (const options of [undefined, {}, { module_or_path: undefined }]) {
  const api = await fresh();
  const exports = await api.default(options);
  assert.equal(exports.starts.value, 1);
  assert.equal(fetched.at(-1), new URL("browser_bg.wasm", import.meta.url).href);
}

// Reject the removed direct-argument API both before and after initialization.
// In particular a direct URL must not silently become a default-URL request.
for (const initialized of [false, true]) {
  const api = await fresh();
  if (initialized) await api.default({ module_or_path: bytes });
  const fetchesBefore = fetched.length;
  for (const legacy of [
    bytes, bytes.buffer, compiled, "browser.wasm", null, 42, [],
    new URL("https://example.test/legacy.wasm"),
    new Request("https://example.test/legacy.wasm"),
    response(), Promise.resolve(bytes),
  ]) {
    assert.throws(() => api.initSync(legacy), TypeError);
    await assert.rejects(api.default(legacy), TypeError);
  }
  assert.throws(() => api.initSync(), TypeError);
  assert.throws(() => api.initSync({ module: bytes }, undefined), TypeError);
  await assert.rejects(api.default({ module_or_path: bytes }, undefined), TypeError);
  assert.equal(fetched.length, fetchesBefore);
}

// Current Response loading remains usable when the server has the wrong MIME
// type or instantiateStreaming is unavailable. HTTP errors remain failures.
const warnings = [];
const warn = console.warn;
console.warn = (...args) => warnings.push(args);
try {
  const api = await fresh();
  assert.equal((await api.default({ module_or_path: response("text/plain") })).starts.value, 1);
  assert.equal(warnings.length, 1);
} finally {
  console.warn = warn;
}
const streaming = WebAssembly.instantiateStreaming;
try {
  WebAssembly.instantiateStreaming = undefined;
  const api = await fresh();
  assert.equal((await api.default({ module_or_path: response() })).starts.value, 1);
} finally {
  WebAssembly.instantiateStreaming = streaming;
}
const api = await fresh();
await assert.rejects(
  api.default({ module_or_path: new Response(null, { status: 404, statusText: "Not Found" }) }),
  /failed to fetch Wasm: 404 Not Found/,
);
