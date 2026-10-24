import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import { adminSite, fixtureImage, mockApi, pageOf, summary, waitForApplication } from './browser-fixtures.mjs';
import { paintedSurfaceColor } from './painted-surface.mjs';

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

async function color(page, property) {
  return page.evaluate(property => {
    const probe = document.createElement('span');
    probe.style.color = `var(${property})`;
    document.body.append(probe);
    const result = getComputedStyle(probe).color;
    probe.remove();
    return result;
  }, property);
}

async function selectFallback(page, trigger, declaration, reason) {
  const panel = page.locator(`#${await trigger.getAttribute('aria-controls')}`);
  await panel.evaluate((node, declaration) => Object.assign(node.style, declaration), declaration);
  await trigger.click();
  await expect(panel).toHaveAttribute('data-blog-server-corner-state', 'fallback');
  await expect(panel).toHaveAttribute('data-blog-server-corner-reason', reason);
  await expect(panel).toBeVisible();
  await expect(panel.getByRole('option').first()).toBeVisible();
  // A completed decorator fallback must leave the native popup usable. Exercise
  // its actual keyboard state, rather than changing the state attributes by hand.
  await page.keyboard.press('End');
  await page.keyboard.press('Enter');
  await expect(panel).not.toBeVisible();
  await expect(trigger).toBeFocused();
  await expect(trigger).toContainText('中文分类');
  await panel.evaluate((node, declaration) => {
    for (const property of Object.keys(declaration)) node.style[property] = '';
  }, declaration);
}

async function row(locator, label) {
  await expect(locator).toBeVisible();
  const geometry = await locator.evaluate(node => {
    const css = getComputedStyle(node);
    return { height: node.getBoundingClientRect().height, font: parseFloat(css.fontSize), margin: css.marginBlock };
  });
  assert.ok(geometry.height >= 35.9, `${label} retains a full 36px row: ${JSON.stringify(geometry)}`);
  near(geometry.font, 14.4, `${label} uses body typography`);
  assert.equal(geometry.margin, '0px', `${label} has no inherited public margin`);
}

async function fixture(page) {
  const state = { authenticated: true, adminSite: { ...adminSite, media: { ...adminSite.media, logo: fixtureImage } } };
  await mockApi(page, state);
  await page.route('**/api/v1/content/articles?*', route => route.fulfill({
    json: pageOf(Array.from({ length: 21 }, (_, index) => ({ ...summary, id: index + 1,
      article_title: `${summary.article_title} · Rust service design review` })), new URL(route.request().url())),
  }));
  await page.route('**/api/v1/content/tree-hole?*', route => route.fulfill({ json: pageOf([
    { id: 1, user_id: 1, username: 'User', avatar: null, message: 'A message with a photograph', image_path: fixtureImage, create_time: '2026-10-24' },
  ], new URL(route.request().url())) }));
  return state;
}

async function businessComponents(page, base, dark, state) {
  // Empty fixtures used to let most of the old component styles escape the
  // geometry checks. Exercise populated rows, pagination and media as well.
  await page.goto(`${base}/admin?lang=en#articles`);
  const articleRow = page.getByRole('row').filter({ hasText: summary.article_title }).first();
  await control(articleRow.getByRole('button', { name: 'Edit', exact: true }), 36, 18, 0);
  const remove = articleRow.getByRole('button', { name: 'Delete', exact: true });
  await control(remove, 36, 18, 0);
  assert.equal(await remove.evaluate(node => getComputedStyle(node).color), await color(page, '--blog-text-danger'),
    'Dangerous actions keep their semantic color instead of inheriting ordinary button color');
  const table = page.locator('.blog-table-scroll').filter({ has: page.getByRole('table', { name: 'Article list', exact: true }) });
  const readability = await articleRow.evaluate(node => {
    const title = document.createRange();
    title.selectNodeContents(node.cells[0].querySelector('strong'));
    return { width: node.cells[0].getBoundingClientRect().width, height: node.getBoundingClientRect().height,
      line: parseFloat(getComputedStyle(node.cells[0]).lineHeight),
      titleLines: new Set([...title.getClientRects()].filter(rect => rect.width > 0).map(rect => Math.round(rect.top))).size };
  });
  assert.ok(readability.width >= 139.9, `Article titles keep a readable column: ${JSON.stringify(readability)}`);
  assert.ok(readability.titleLines <= 4 && readability.height <= readability.line * 5 + 2,
    `A medium title uses at most four lines plus its separate identifier: ${JSON.stringify(readability)}`);
  if (page.viewportSize().width <= 768) {
    const overflow = await table.evaluate(node => ({ content: node.scrollWidth, available: node.clientWidth }));
    assert.ok(overflow.content > overflow.available, 'Small screens scroll a readable table instead of crushing every column');
  }
  await table.evaluate(node => { node.scrollLeft = node.scrollWidth; });
  const actionPosition = await remove.evaluate(node => {
    const button = node.getBoundingClientRect(), table = node.closest('.blog-table-scroll').getBoundingClientRect();
    return { left: button.left, right: button.right, tableLeft: table.left, tableRight: table.right };
  });
  assert.ok(actionPosition.left >= actionPosition.tableLeft - 1 && actionPosition.right <= actionPosition.tableRight + 1,
    `Scrolling reaches table actions: ${JSON.stringify(actionPosition)}`);
  let confirmed = false;
  page.once('dialog', async dialog => { confirmed = true; await dialog.dismiss(); });
  await remove.click();
  assert.equal(confirmed, true, 'The visible Delete action still opens its real confirmation');
  await articleRow.getByRole('button', { name: 'Edit', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Edit article', exact: true })).toBeVisible();
  await page.goto(`${base}/admin?lang=en#articles`);
  const pager = page.getByRole('navigation', { name: 'Pagination', exact: true });
  await control(pager.getByRole('button', { name: 'Next page', exact: true }), 36, 18, 0);
  await pager.getByRole('button', { name: 'Next page', exact: true }).click();
  await expect(pager).toContainText('Page 2');

  await page.goto(`${base}/admin?lang=en#edit/1`);
  const markdown = page.getByRole('textbox', { name: 'Markdown content', exact: true });
  await expect(markdown).toBeVisible();
  near((await markdown.boundingBox()).height, 308, 'Markdown editor retains a useful seven-row writing surface');
  const checks = page.locator('.check-row > label');
  await expect(checks).toHaveCount(3);
  for (const check of await checks.all()) {
    await row(check, 'Editor checkbox label');
    const alignment = await check.evaluate(node => {
      const input = node.querySelector('input').getBoundingClientRect();
      const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
      let text;
      while ((text = walker.nextNode()) && !text.textContent.trim()) {}
      if (!text) throw new Error('A checkbox requires a visible text label');
      const range = document.createRange();
      range.selectNodeContents(text);
      const first = [...range.getClientRects()].find(rect => rect.width > 0 && rect.height > 0);
      if (!first) throw new Error('Checkbox text has no visible line box');
      return { inputRight: input.right, textLeft: first.left };
    });
    assert.ok(alignment.textLeft >= alignment.inputRight - .1,
      `Checkbox and the start of its label stay side by side: ${JSON.stringify(alignment)}`);
  }
  const fieldLabels = page.locator('.editor-form .blog-field-label');
  for (const label of await fieldLabels.all()) await row(label, 'Editor field label');
  await page.getByRole('button', { name: 'Preview content', exact: true }).click();
  const rendered = page.locator('.editor-markdown-preview');
  await expect(rendered).toBeVisible();
  await expect(rendered).toHaveAttribute('data-blog-server-corner-state', 'clip');
  const previewGeometry = await rendered.evaluate(node => {
    const css = getComputedStyle(node);
    return { topInset: parseFloat(css.paddingTop), leftInset: parseFloat(css.paddingLeft), rightInset: parseFloat(css.paddingRight),
      margin: css.margin, line: parseFloat(css.lineHeight), font: parseFloat(css.fontSize),
      border: css.borderTopWidth, extent: parseFloat(css.getPropertyValue('--blog-server-corner-extent')) };
  });
  near(previewGeometry.topInset, 0, 'Markdown preview uses natural content rows');
  near(previewGeometry.leftInset, 9.8, 'Markdown preview retains the shared frame text clearance');
  near(previewGeometry.rightInset, 9.8, 'Markdown preview retains symmetric text clearance');
  assert.equal(previewGeometry.margin, '0px');
  assert.equal(previewGeometry.border, '1px');
  near(previewGeometry.font, 14.4, 'Markdown preview typography');
  near(previewGeometry.line, 21.6, 'Markdown preview line height');
  near(previewGeometry.extent, 18, 'Markdown preview G4 extent');
  await expect.poll(() => paintedSurfaceColor(rendered)).toBe(await color(page, '--blog-bg-panel'));

  await page.goto(`${base}/admin?lang=en#notes`);
  const note = page.getByRole('textbox', { name: 'Note content', exact: true });
  await expect(note).toBeVisible();
  near((await note.boundingBox()).height, 110, 'Ordinary multiline fields retain the intended writing height');
  near(await note.evaluate(node => parseFloat(getComputedStyle(node).fontSize)), 17.6, 'Multiline field typography');

  await page.goto(`${base}/admin?lang=en#site`);
  await row(page.locator('.blog-settings-form summary'), 'Push format disclosure');
  const logo = page.getByRole('img', { name: 'Site logo preview', exact: true });
  await expect(logo).toBeVisible();
  await expect(logo).toHaveAttribute('data-blog-server-corner-state', 'clip');
  near(await logo.evaluate(node => parseFloat(getComputedStyle(node).getPropertyValue('--blog-server-corner-extent'))), 18,
    'Configured media preview uses the shared panel contour');
  const panels = page.locator('.blog-settings-form .blog-content-panel');
  for (const panel of await panels.all()) {
    await expect(panel).toHaveAttribute('data-blog-server-corner-state', 'clip');
    await expect.poll(() => paintedSurfaceColor(panel)).toBe(await color(page, '--blog-bg-panel'));
  }

  await page.goto(`${base}/admin?lang=en#tree-hole`);
  const photograph = page.getByRole('button').filter({ has: page.getByRole('img', { name: 'View message image', exact: true }) });
  await expect(photograph).toBeVisible();
  near((await photograph.boundingBox()).height, 56, 'Image preview trigger has no inline-image baseline gap');
  await expect(photograph.getByRole('img')).toHaveAttribute('data-blog-server-corner-state', 'clip');
  near(await photograph.getByRole('img').evaluate(node => parseFloat(getComputedStyle(node).getPropertyValue('--blog-server-corner-extent'))),
    18, 'Moderation thumbnail follows the shared panel contour');
  await photograph.click();
  const lightbox = page.getByRole('dialog', { name: 'View image', exact: true });
  await expect(lightbox).toBeVisible();
  await control(lightbox.getByRole('button', { name: 'Close image', exact: true }), 44, 22, 0);
  await page.keyboard.press('Escape');
  await expect(lightbox).toHaveCount(0);
  await expect(photograph).toBeFocused();

  // Notifications are driven through the same document event used by actual
  // asynchronous saves, so their portal location and inherited roles are real.
  await page.evaluate(() => dispatchEvent(new CustomEvent('blog-notify', { detail: 'Component style verification' })));
  const notification = page.getByRole('status').filter({ hasText: 'Component style verification' });
  await expect(notification).toBeVisible();
  await expect(notification).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
  await expect.poll(() => paintedSurfaceColor(notification)).toBe(await color(page, '--blog-bg-panel'));
  await control(notification.getByRole('button', { name: 'Dismiss notification', exact: true }), 36, 18, 0);
  await notification.getByRole('button', { name: 'Dismiss notification', exact: true }).click();
  await expect(notification).toHaveCount(0);

  for (const hash of ['dashboard', 'categories', 'labels', 'letter', 'comments', 'resources', 'links', 'family', 'home']) {
    await page.goto(`${base}/admin?lang=en#${hash}`);
    await waitForApplication(page);
    const labels = page.locator('.blog-admin-workspace .blog-field-label');
    for (const label of await labels.all()) await row(label, `${hash} field label`);
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${hash} components fit the viewport`);
  }
  await expect(page.locator('html')).toHaveAttribute('data-theme', dark ? 'dark' : 'light');

  let pending;
  await page.route('**/api/v1/content/comments?*', route => { pending = route; });
  await page.goto(`${base}/admin?lang=en#comments`);
  const loading = page.locator('.blog-admin-workspace .blog-loading');
  await expect(loading).toBeVisible();
  await row(loading, 'Loading state');
  await expect(loading).toHaveAttribute('data-blog-server-corner-state', /^(paint|clip)$/);
  await expect.poll(() => paintedSurfaceColor(loading)).toBe(await color(page, '--blog-bg-panel'));
  await expect.poll(() => Boolean(pending)).toBe(true);
  await pending.fulfill({ status: 503, json: { code: 'unavailable', retryable: true, request_id: 'style-503' } });
  const error = page.getByRole('alert');
  await expect(error).toBeVisible();
  await row(error, 'Error state');
  near(await error.evaluate(node => parseFloat(getComputedStyle(node).getPropertyValue('--blog-server-corner-extent'))), 18,
    'Business error uses the shared notification contour');
  await expect.poll(() => paintedSurfaceColor(error)).toBe(await color(page, '--blog-bg-panel'));
  await control(error.getByRole('button', { name: 'Try again', exact: true }), 36, 18, 0);

  state.authenticated = false;
  await page.route('**/api/v1/auth/login', route => route.fulfill({ status: 401,
    json: { code: 'unauthorized', retryable: false, request_id: 'style-login-401' } }));
  await page.goto(`${base}/admin?lang=en`);
  await page.getByRole('textbox', { name: 'Username', exact: true }).fill('administrator');
  await page.getByLabel('Password', { exact: true }).fill('incorrect-password');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  const loginError = page.getByRole('alert');
  await expect(loginError).toContainText('Sign in failed');
  const loginGeometry = await loginError.evaluate(node => {
    const css = getComputedStyle(node);
    return { border: css.borderTopWidth, padding: css.padding, font: css.fontSize,
      formFont: getComputedStyle(node.closest('form')).fontSize, extent: css.getPropertyValue('--blog-server-corner-extent') };
  });
  assert.equal(loginGeometry.border, '0px', 'Authentication errors do not restore the old outlined box');
  assert.equal(loginGeometry.padding, '0px');
  assert.equal(loginGeometry.font, loginGeometry.formFont, 'Authentication errors follow the six-row form typography');
  near(parseFloat(loginGeometry.extent), 0, 'Authentication error stays inside its existing card contour');
  assert.equal(await paintedSurfaceColor(loginError), await color(page, '--blog-bg-panel'));
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Failed authentication fits the viewport');
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
          const state = await fixture(page);
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
          await selectFallback(page, category, { maskImage: 'linear-gradient(#000,#000)' }, 'existing-mask');
          await selectFallback(page, category, { boxShadow: 'inset 0 0 1px #000' }, 'inset-or-unsupported-shadow');
          await category.click();
          await expect(page.getByRole('listbox', { name: 'Category', exact: true })).toHaveAttribute('data-blog-server-corner-state', 'clip');
          await page.keyboard.press('Escape');

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
          await businessComponents(page, base, dark, state);
          assert.deepEqual(errors, []);
          console.log(`${engine.name()} ${width}px ${dark ? 'dark' : 'light'}: full admin navigation, populated tables/pagination, forms/Markdown, media/lightbox, notifications, and decorated/fallback selectors passed`);
        } finally { await context.close(); }
      }
    } finally { await browser.close(); }
  }
} finally { await new Promise(resolve => server.httpServer.close(resolve)); }
