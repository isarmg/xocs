import { createRequire } from 'node:module';

// Browser tooling has one dependency installation, beside its npm manifest.
// Central tests resolve it explicitly instead of depending on their cwd.
const require = createRequire(new URL('../../src/web/package.json', import.meta.url));
export const { chromium, firefox, expect } = require('@playwright/test');
export const { default: AxeBuilder } = require('@axe-core/playwright');
