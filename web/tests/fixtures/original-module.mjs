import { readFile } from 'node:fs/promises';
import { registerHooks } from 'node:module';
import { fileURLToPath } from 'node:url';
import { transformWithOxc } from 'vite';
import { registerNativeSelectBoundary } from './hook-host.mjs';

// Transform the original TSX, replacing only explicitly named module boundaries.
// The shared UI is loaded from the installed package. No browser or DOM is used.
export async function originalModule(entry, replacements = {}, exports = []) {
  const source = await readFile(entry, 'utf8');
  const exposed = exports.length ? `\nexport { ${exports.join(', ')} };` : '';
  const { code } = await transformWithOxc(source + exposed, fileURLToPath(entry), {
    jsx: { runtime: 'automatic' },
  });
  const hookHost = new URL('./hook-host.mjs', import.meta.url).href;
  const hooks = registerHooks({
    resolve(specifier, context, next) {
      if (specifier === 'react' || specifier === 'react/jsx-runtime') {
        return { url: hookHost, shortCircuit: true };
      }
      if (Object.hasOwn(replacements, specifier)) {
        return { url: `data:text/javascript,${encodeURIComponent(replacements[specifier])}`, shortCircuit: true };
      }
      return next(specifier, context);
    },
    load(url, context, next) {
      if (url === entry.href) return { format: 'module', source: code, shortCircuit: true };
      return next(url, context);
    },
  });
  try {
    const module = await import(entry.href);
    const ui = await import('@xcss/web/admin-ui');
    registerNativeSelectBoundary(ui.Select);
    return module;
  } finally {
    hooks.deregister();
  }
}
