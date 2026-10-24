import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { mockApi } from './browser-fixtures.mjs';

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
const near = (actual, expected, label, tolerance = .1) => {
  assert.ok(Math.abs(actual - expected) <= tolerance, `${label}: expected ${expected}, got ${actual}`);
};

async function theme(page, dark) {
  const switchTo = page.getByRole('button', { name: dark ? 'Switch to dark mode' : 'Switch to light mode', exact: true });
  if (await switchTo.count()) await switchTo.click();
  await expect(page.getByRole('button', { name: dark ? 'Switch to light mode' : 'Switch to dark mode', exact: true })).toBeVisible();
}

async function shell(page, rootSelector, headerSelector, mainSelector) {
  await page.evaluate(() => document.fonts.ready);
  const geometry = await page.evaluate(({ rootSelector, headerSelector, mainSelector }) => {
    const root = document.querySelector(rootSelector), header = document.querySelector(headerSelector), main = document.querySelector(mainSelector);
    const rootBox = root.getBoundingClientRect(), headerBox = header.getBoundingClientRect(), mainBox = main.getBoundingClientRect();
    const rootStyle = getComputedStyle(root), headerStyle = getComputedStyle(header), mainStyle = getComputedStyle(main);
    return { viewport: innerWidth, available: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
      root: { left: rootBox.left, width: rootBox.width, font: parseFloat(rootStyle.fontSize), line: parseFloat(rootStyle.lineHeight) },
      header: { left: headerBox.left, width: headerBox.width, bottom: headerBox.bottom, position: headerStyle.position,
        transform: headerStyle.transform, leftInset: parseFloat(headerStyle.paddingLeft), rightInset: parseFloat(headerStyle.paddingRight) },
      main: { left: mainBox.left, top: mainBox.top, width: mainBox.width, maxWidth: mainStyle.maxWidth,
        leftInset: parseFloat(mainStyle.paddingLeft), rightInset: parseFloat(mainStyle.paddingRight), bottomInset: parseFloat(mainStyle.paddingBottom) } };
  }, { rootSelector, headerSelector, mainSelector });
  assert.ok(geometry.scroll <= geometry.viewport, `Page overflows: ${JSON.stringify(geometry)}`);
  near(geometry.root.left, 0, 'Shell starts at the viewport edge');
  near(geometry.root.width, geometry.available, 'Shell fills the available width');
  near(geometry.root.font, 14.4, 'Business body font');
  near(geometry.root.line, 21.6, 'Business body line height');
  assert.ok(['static', 'relative'].includes(geometry.header.position), 'Header remains in ordinary document flow');
  assert.equal(geometry.header.transform, 'none');
  near(geometry.header.left, 0, 'Header starts at the viewport edge');
  near(geometry.header.width, geometry.available, 'Header spans the viewport');
  near(geometry.main.left, 0, 'Main starts at the viewport edge');
  near(geometry.main.width, geometry.available, 'Main spans the viewport');
  assert.equal(geometry.main.maxWidth, 'none', 'Wide screens have no content-width cap');
  assert.ok(geometry.main.top >= geometry.header.bottom - 1, 'Main follows the header instead of sitting beneath an overlay');
  // Desktop browser engines expose zero safe-area insets. Verify the 16px
  // baseline here without pretending to emulate a device with a display cutout.
  for (const [label, value] of Object.entries({ headerLeft: geometry.header.leftInset, headerRight: geometry.header.rightInset,
    mainLeft: geometry.main.leftInset, mainRight: geometry.main.rightInset, mainBottom: geometry.main.bottomInset })) {
    near(value, 16, `${label} safe-area baseline`);
  }
}

async function control(locator, height, extent, border) {
  await expect(locator).toBeVisible();
  await expect(locator).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
  const geometry = await locator.evaluate(node => {
    const css = getComputedStyle(node);
    return { height: node.getBoundingClientRect().height, font: parseFloat(css.fontSize), radius: css.borderRadius,
      extent: parseFloat(css.getPropertyValue('--blog-server-corner-extent')), border: parseFloat(css.borderTopWidth) };
  });
  near(geometry.height, height, 'Control height');
  near(geometry.font, height * .4, 'Control typography follows its row height');
  near(geometry.extent, extent, 'G4 corner extent');
  near(geometry.border, border, 'Control border');
  assert.equal(geometry.radius, '0px', 'The shared G4 renderer owns the corner');
}

try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      for (const width of [390, 1280, 2560]) for (const dark of [false, true]) {
        const context = await browser.newContext({ locale: 'en', viewport: { width, height: 900 }, reducedMotion: 'reduce' });
        try {
          const page = await context.newPage(), errors = [];
          page.on('pageerror', error => errors.push(error.message));
          await mockApi(page, { authenticated: true });
          await page.goto(`${base}/admin?lang=en#categories`);
          const name = page.getByRole('textbox', { name: 'Category name', exact: true });
          await expect(name).toBeVisible();
          await theme(page, dark);
          await shell(page, '.blog-admin-shell', '.blog-page-header', '.blog-shell-main');
          await control(name, 44, 22, 1);
          await control(page.getByRole('button', { name: 'Create category', exact: true }), 36, 18, 0);
          const navigationButtons = page.locator('.blog-header-navigation').getByRole('button');
          await expect(navigationButtons).toHaveCount(5);
          for (const button of await navigationButtons.all()) await control(button, 36, 18, 0);
          const accountButton = page.getByRole('button', { name: 'Account settings', exact: true });
          near((await accountButton.boundingBox()).height, 44, 'Header actions retain comfortable targets');

          await page.goto(`${base}/admin?lang=en#labels`);
          const category = page.getByRole('combobox', { name: 'Category', exact: true });
          await expect(category).toBeEnabled();
          await control(category, 36, 18, 1);
          await category.click();
          const options = page.getByRole('listbox', { name: 'Category', exact: true }).getByRole('option');
          await expect(options.first()).toBeVisible();
          for (const option of await options.all()) near((await option.boundingBox()).height, 36, 'Select option row');
          await page.keyboard.press('Escape');
          await expect(category).toBeFocused();
          assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'The open and closed selector never expands the viewport');

          await page.goto(`${base}/admin?lang=en#account`);
          const account = page.getByRole('region', { name: 'Account settings', exact: true });
          await control(account.getByRole('textbox', { name: 'Username', exact: true }), 36, 18, 1);
          const accountLayout = await account.evaluate(node => ({
            width: node.querySelector('.blog-content-panel').getBoundingClientRect().width,
            available: document.documentElement.clientWidth - 32,
            columns: getComputedStyle(node.querySelector('.blog-account-fields')).gridTemplateColumns.split(' ').length,
          }));
          near(accountLayout.width, Math.min(accountLayout.available, 504), 'Account form remains a compact task surface');
          assert.equal(accountLayout.columns, width <= 768 ? 1 : 2, 'Account labels stack on small screens');

          await page.goto(`${base}/admin?lang=en#new`);
          const editor = page.locator('.blog-admin-workspace form.editor-form').first();
          await expect(editor).toBeVisible();
          const editorLayout = await editor.evaluate(node => {
            const main = node.closest('.blog-shell-main'), css = getComputedStyle(main), box = main.getBoundingClientRect();
            const leftInset = parseFloat(css.paddingLeft), rightInset = parseFloat(css.paddingRight);
            return { left: node.getBoundingClientRect().left, width: node.getBoundingClientRect().width,
              maxWidth: getComputedStyle(node).maxWidth, contentLeft: box.left + main.clientLeft + leftInset,
              contentWidth: main.clientWidth - leftInset - rightInset };
          });
          assert.equal(editorLayout.maxWidth, 'none', 'The article editor has no inherited public width cap');
          near(editorLayout.left, editorLayout.contentLeft, 'Article editor aligns with the main content');
          near(editorLayout.width, editorLayout.contentWidth, 'Article editor fills the main content width');
          assert.deepEqual(errors, []);
          console.log(`${engine.name()} ${width}px ${dark ? 'dark' : 'light'}: admin full-width flow navigation, 16px insets and 36/44px G4 controls passed`);
        } finally { await context.close(); }
      }
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
