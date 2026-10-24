import assert from 'node:assert/strict';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import {
  adminSite, emptySiteMedia, fixtureImageBytes, fixtureVideoBytes, mockApi,
  site, siteMediaKeys, waitForApplication,
} from './browser-fixtures.mjs';

const families = [{ id: 1, man_name: 'Lin', woman_name: 'Yu', timing: '2026-10-01',
  bg_cover: null, man_cover: null, woman_cover: null, family_info: 'Our home.',
  countdown_title: null, countdown_time: null }];
const image = name => `/__test-media/${name}.png`;
const videoPath = '/__test-media/background.webm';
const imageUpload = { id: 81, path: `/media/${'c'.repeat(64)}.png`, mime_type: 'image/png', size: fixtureImageBytes.length };
const videoUpload = { id: 82, path: `/media/${'d'.repeat(64)}.webm`, mime_type: 'video/webm', size: fixtureVideoBytes.length };
const labels = {
  logo: 'Site logo', journey_background: 'Journey background', journal_background: 'Journal background',
  guestbook_background: 'Message board background', love_letter_background: 'Love letter background',
  love_moments_cover: 'Love moments cover', love_photos_cover: 'Love photos cover', love_wishes_cover: 'Love wishes cover',
  toolbox_background: 'Toolbox background image', toolbox_video: 'Toolbox background video',
};
const mediaResponses = new WeakMap();

async function chooseFile(page, label, payload) {
  const waiting = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: `Choose file: Upload ${label}`, exact: true }).click();
  await (await waiting).setFiles(payload);
}
async function background(page, selector, path) {
  const expected = new URL(path, page.url()).href;
  const target = page.locator(selector);
  await target.scrollIntoViewIfNeeded();
  // Rounded surfaces paint native CSS backgrounds on their visible decoration
  // plane. The host deliberately becomes transparent beneath its SVG border.
  await expect.poll(() => target.evaluate((host, url) => {
    const bounds = host.getBoundingClientRect();
    return [host, ...host.querySelectorAll(':scope > [data-blog-server-corner-decoration]')].some(node => {
      const css = getComputedStyle(node), box = node.getBoundingClientRect();
      return css.backgroundImage.includes(`url("${url}")`) && css.display !== 'none' && css.visibility === 'visible'
        && Number(css.opacity) > 0 && box.width >= bounds.width * .98 && box.height >= bounds.height * .98;
    });
  }, expected), { message: `${selector} paints configured image ${path}` }).toBe(true);
  await expect.poll(() => mediaResponses.get(page)?.get(expected), { message: `Configured image loaded: ${path}` })
    .toEqual({ status: 200, mime: 'image/png' });
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    const context = await browser.newContext({ locale: 'en-US', viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });
    const page = await context.newPage(), errors = [], removedRequests = [], saved = [];
    const responses = new Map(); mediaResponses.set(page, responses);
    page.setDefaultTimeout(30_000);
    page.on('pageerror', error => errors.push(error.message));
    page.on('response', response => {
      if (new URL(response.url()).pathname.startsWith('/__test-media/')) {
        responses.set(response.url(), { status: response.status(), mime: response.headers()['content-type'] });
      }
    });
    page.on('request', request => {
      const path = new URL(request.url()).pathname;
      if (/^\/(?:legacy|live)\//.test(path)) removedRequests.push(path);
    });
    const state = { authenticated: true, site: structuredClone(site), adminSite: structuredClone(adminSite) };
    let pendingVideo = null, holdVideo = false;
    try {
      await mockApi(page, state);
      await page.route('**/api/v1/family', route => route.fulfill({ json: families }));
      await page.route('**/media/**', route => route.fulfill(new URL(route.request().url()).pathname.endsWith('.webm')
        ? { contentType: 'video/webm', body: fixtureVideoBytes }
        : { contentType: 'image/png', body: fixtureImageBytes }));
      await page.route('**/api/v1/content/site', route => {
        if (route.request().method() === 'PUT') {
          const body = route.request().postDataJSON();
          assert.deepEqual(Object.keys(body.media).sort(), [...siteMediaKeys].sort());
          assert.ok(Object.values(body.media).every(value => value === null || typeof value === 'string'));
          saved.push(structuredClone(body)); state.adminSite = structuredClone(body);
          const { author_name: _author, ...publicSite } = body; state.site = publicSite;
        }
        return route.fulfill({ json: state.adminSite });
      });
      await page.route('**/api/v1/content/upload', route => {
        assert.deepEqual(route.request().postDataBuffer(), fixtureImageBytes);
        return route.fulfill({ json: imageUpload });
      });
      await page.route('**/api/v1/content/upload-video', async route => {
        assert.deepEqual(route.request().postDataBuffer(), fixtureVideoBytes);
        if (holdVideo) await new Promise(resolve => { pendingVideo = { route, resolve }; });
        else await route.fulfill({ json: videoUpload });
      });

      // Empty settings are a real empty state, without requests for photographs,
      // avatars or videos removed from the production asset inventory.
      for (const path of ['/', '/menory', '/jotting', '/message', '/love', '/favorite?tab=friends']) {
        await page.goto(`${base}${path}${path.includes('?') ? '&' : '?'}lang=en`);
        await waitForApplication(page);
        await expect(page.locator('img[src=""], video[src=""], video[poster=""]')).toHaveCount(0);
        await expect(page.locator('video')).toHaveCount(0);
      }
      assert.deepEqual(removedRequests, [], 'Unconfigured pages never fetch retired bundled media');

      // The real settings form writes the explicit media object and uses native
      // file selection for both image and video uploads.
      await page.goto(`${base}/admin?lang=en#site`);
      await expect(page.getByRole('textbox', { name: 'Site name', exact: true })).toHaveValue('BLOG');
      for (const key of siteMediaKeys) await page.getByRole('textbox', { name: labels[key], exact: true }).fill(key === 'toolbox_video' ? videoPath : image(key));
      await page.getByRole('textbox', { name: 'Background image', exact: true }).fill(image('home'));
      await page.getByRole('textbox', { name: 'Site owner avatar', exact: true }).fill(image('owner'));
      await page.getByRole('button', { name: 'Save settings', exact: true }).click();
      await expect.poll(() => saved.length).toBe(1);
      assert.deepEqual(saved[0].media, Object.fromEntries(siteMediaKeys.map(key => [key, key === 'toolbox_video' ? videoPath : image(key)])));
      await page.reload();
      for (const key of siteMediaKeys) await expect(page.getByRole('textbox', { name: labels[key], exact: true })).toHaveValue(saved[0].media[key]);

      for (const [path, selector, value] of [
        ['/', '.hero', image('home')], ['/menory', '.hero', image('journey_background')],
        ['/jotting', '.hero', image('journal_background')], ['/message', '.message-hero', image('guestbook_background')],
        ['/love', '.love-letter', image('love_letter_background')],
      ]) {
        await page.goto(`${base}${path}?lang=en`); await waitForApplication(page);
        await background(page, selector, value);
        await expect(page.locator('.brand img')).toHaveAttribute('src', base + image('logo'));
        if (path === '/') await expect(page.locator('.aside-avatar img')).toHaveAttribute('src', base + image('owner'));
        if (path === '/love') {
          for (const key of ['love_moments_cover', 'love_photos_cover', 'love_wishes_cover']) {
            await expect(page.locator(`.love-tab-icon img[src="${base + image(key)}"], img.love-tab-icon[src="${base + image(key)}"]`)).toHaveCount(1);
          }
        }
      }
      await page.goto(`${base}/favorite?tab=friends&lang=en`); await waitForApplication(page);
      await background(page, '.favorite-banner', image('toolbox_background'));
      const banner = page.locator('.favorite-banner video');
      await expect(banner).toHaveAttribute('src', base + videoPath);
      await expect(banner).toHaveAttribute('poster', base + image('toolbox_background'));
      await expect(banner).toHaveJSProperty('autoplay', false);
      await expect(banner).toHaveJSProperty('controls', true);
      await banner.evaluate(node => { node.preload = 'metadata'; node.load(); });
      await expect.poll(() => banner.evaluate(node => node.videoWidth)).toBe(16);
      state.site.media.toolbox_video = null;
      await page.reload(); await waitForApplication(page);
      await expect(page.locator('.favorite-banner video')).toHaveCount(0);
      await background(page, '.favorite-banner', image('toolbox_background'));

      await page.goto(`${base}/admin?lang=en#site`);
      await expect(page.getByRole('textbox', { name: labels.logo, exact: true })).toHaveValue(image('logo'));
      await chooseFile(page, labels.logo, { name: 'site-logo.png', mimeType: 'image/png', buffer: fixtureImageBytes });
      await expect(page.getByRole('textbox', { name: labels.logo, exact: true })).toHaveValue(imageUpload.path);
      await chooseFile(page, labels.toolbox_video, { name: 'background.webm', mimeType: 'video/webm', buffer: fixtureVideoBytes });
      await expect(page.getByRole('textbox', { name: labels.toolbox_video, exact: true })).toHaveValue(videoUpload.path);
      await page.getByRole('button', { name: 'Save settings', exact: true }).click();
      await expect.poll(() => saved.length).toBe(2);
      assert.equal(saved[1].media.logo, imageUpload.path); assert.equal(saved[1].media.toolbox_video, videoUpload.path);

      holdVideo = true;
      await chooseFile(page, labels.toolbox_video, { name: 'cancelled.webm', mimeType: 'video/webm', buffer: fixtureVideoBytes });
      await expect.poll(() => Boolean(pendingVideo)).toBe(true);
      await expect(page.getByRole('button', { name: 'Save settings', exact: true })).toBeDisabled();
      await expect(page.getByRole('textbox', { name: labels.toolbox_video, exact: true })).toBeDisabled();
      await page.getByRole('button', { name: 'Cancel upload', exact: true }).click();
      await expect(page.getByRole('textbox', { name: labels.toolbox_video, exact: true })).toBeEnabled();
      await page.getByRole('textbox', { name: labels.toolbox_video, exact: true }).fill(videoPath);
      const pending = pendingVideo; pendingVideo = null;
      try { await pending.route.fulfill({ json: videoUpload }); } catch (error) {
        assert.match(error.message, /closed|aborted|already|intercept|invalid/i);
      } finally { pending.resolve(); }
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      await expect(page.getByRole('textbox', { name: labels.toolbox_video, exact: true })).toHaveValue(videoPath);

      for (const key of siteMediaKeys) await page.getByRole('button', { name: `Clear ${labels[key]}`, exact: true }).click();
      await page.getByRole('button', { name: 'Save settings', exact: true }).click();
      await expect.poll(() => saved.length).toBe(3);
      assert.deepEqual(saved[2].media, emptySiteMedia());
      await page.goto(`${base}/favorite?tab=friends&lang=en`); await waitForApplication(page);
      await expect(page.locator('.favorite-banner video')).toHaveCount(0);
      await expect(page.locator('.favorite-banner')).not.toHaveAttribute('style', /__test-media/);
      assert.deepEqual(removedRequests, []); assert.deepEqual(errors, []);
      console.log(`${engine.name()}: empty media, configured backgrounds/logo/avatar/video, upload/save/reload, cancellation and clearing passed`);
    } finally {
      if (pendingVideo) { pendingVideo.resolve(); await pendingVideo.route.abort().catch(() => {}); }
      await context.close(); await browser.close();
    }
  }
} finally { await server.close(); }
