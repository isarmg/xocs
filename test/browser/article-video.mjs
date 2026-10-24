import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { chromium, firefox, expect } from './dependencies.mjs';
import { preview } from '../tools/assets/test-preview.mjs';
import {
  article, fixtureVideoBytes, mockApi, pageOf, session, waitForApplication,
} from './browser-fixtures.mjs';

const youtube = 'https://www.youtube.com/watch?v=dQw4w9WgXcQ&autoplay=1';
const youtubeEmbed = 'https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ';
const bilibili = 'https://www.bilibili.com/video/BV1xx411c7mD/?p=2';
const bilibiliEmbed = 'https://player.bilibili.com/player.html?bvid=BV1xx411c7mD&page=2&autoplay=0';
const directVideo = '/__test-media/background.webm';
// MP4 bytes exercise the browser's MIME-preserving upload path. Actual native
// playback below uses the valid WebM fixture; container validation is a native
// server responsibility and has separate tests.
const mp4UploadBytes = Buffer.from('article-video MP4 upload transport fixture');
const labels = {
  en: { body: 'Markdown content', url: 'Video URL', contentUrl: 'Content video URL', insert: 'Insert video',
    bodyUpload: 'Choose file: Upload content video', mainUpload: 'Choose file: Upload article video',
    preview: 'Preview content', save: 'Save article', cancel: 'Cancel', articles: 'Articles', title: 'Title' },
  'zh-CN': { body: 'Markdown 正文', url: '视频地址', contentUrl: '正文视频地址', insert: '插入视频',
    bodyUpload: '选择文件：上传正文视频', mainUpload: '选择文件：上传文章视频',
    preview: '预览正文', save: '保存文章', cancel: '取消', articles: '文章管理', title: '标题' },
};
const artifacts = process.env.BLOG_ARTICLE_VIDEO_ARTIFACTS;
if (artifacts) await mkdir(artifacts, { recursive: true });

async function screenshot(page, filename) {
  if (!artifacts) return;
  await page.evaluate(() => scrollTo(0, 0));
  await page.screenshot({ path: join(artifacts, filename), fullPage: true });
}

async function chooseFile(page, name, file) {
  const waiting = page.waitForEvent('filechooser');
  await page.getByRole('button', { name, exact: true }).click();
  await (await waiting).setFiles(file);
}

async function selectText(textarea, start, end = start) {
  await textarea.focus();
  await textarea.evaluate((node, [start, end]) => {
    node.setSelectionRange(start, end);
    node.dispatchEvent(new Event('select', { bubbles: true }));
  }, [start, end]);
}

async function noOverflow(page) {
  await page.evaluate(() => document.fonts.ready);
  const size = await page.evaluate(() => ({ viewport: innerWidth, content: document.documentElement.scrollWidth }));
  assert.ok(size.content <= size.viewport, `${page.url()}: ${JSON.stringify(size)}`);
}

async function mediaMarkup(scope, nativeCount, sources) {
  await expect(scope.locator('video')).toHaveCount(nativeCount);
  await expect(scope.locator('iframe')).toHaveCount(sources.length);
  for (const video of await scope.locator('video').all()) {
    await expect(video).toHaveAttribute('controls', '');
    await expect(video).toHaveAttribute('playsinline', '');
    await expect(video).toHaveAttribute('preload', 'metadata');
    await expect(video).toHaveJSProperty('autoplay', false);
  }
  const actual = await scope.locator('iframe').evaluateAll(nodes => nodes.map(node => node.src));
  assert.deepEqual(actual, sources);
  for (const frame of await scope.locator('iframe').all()) {
    await expect(frame).toHaveAttribute('title', /^(YouTube|Bilibili) video player$/);
    await expect(frame).toHaveAttribute('loading', 'lazy');
    await expect(frame).toHaveAttribute('referrerpolicy', 'strict-origin-when-cross-origin');
    await expect(frame).toHaveAttribute('sandbox', 'allow-scripts allow-same-origin allow-presentation');
    await expect(frame).toHaveAttribute('allowfullscreen', '');
  }
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const base = `http://127.0.0.1:${server.httpServer.address().port}`;
try {
  for (const [engine, language, width] of [[chromium, 'en', 1280], [firefox, 'zh-CN', 390]]) {
    const browser = await engine.launch();
    const context = await browser.newContext({ locale: language, viewport: { width, height: 900 }, reducedMotion: 'reduce' });
    const page = await context.newPage(), names = labels[language], errors = [], uploads = [], writes = [], externalRequests = [];
    page.setDefaultTimeout(30_000);
    page.on('pageerror', error => errors.push(error.message));
    let saved = { ...structuredClone(article), article_title: 'Article videos', article_content: '甲😀乙尾' };
    let mode = 'success', pending = null;
    async function releasePending() {
      const held = pending; pending = null;
      if (!held) return;
      try { await held.route.fulfill({ json: held.value }); }
      catch (error) { assert.match(error.message, /closed|aborted|already|intercept|invalid/i); }
      finally { held.resolve(); }
    }
    try {
      await page.addInitScript(() => { window.__articleVideoXss = 0; });
      await context.route('**/*', route => {
        const url = new URL(route.request().url());
        if (url.origin === base) return route.fallback();
        externalRequests.push(url.href);
        if (['https://www.youtube-nocookie.com', 'https://player.bilibili.com'].includes(url.origin)) {
          return route.fulfill({ contentType: 'text/html', body: '<!doctype html><html lang="en"><title>Video fixture</title><body><p data-video-fixture>Player loaded</p></body></html>' });
        }
        return route.abort('blockedbyclient');
      });
      await mockApi(page, { authenticated: true });
      await page.route('**/media/**', route => route.fulfill({ contentType: 'video/webm', body: fixtureVideoBytes }));
      await page.route('**/api/v1/**', async route => {
        const request = route.request(), url = new URL(request.url()), path = url.pathname;
        if (path === '/api/v1/content/labels') return route.fulfill({ json: [{ id: 1, sort_id: 1, label_name: 'Video tag', label_description: null }] });
        if (path === '/api/v1/articles/1') return route.fulfill({ json: saved });
        if (path === '/api/v1/content/articles/1') {
          if (request.method() === 'PUT') {
            assert.equal(request.headers()['x-csrf-token'], session.csrf_token);
            const input = request.postDataJSON(); writes.push(input);
            saved = { ...saved, ...input, view_status: Number(input.view_status),
              recommend_status: Number(input.recommend_status), comment_status: Number(input.comment_status) };
          }
          return route.fulfill({ json: saved });
        }
        if (path === '/api/v1/content/articles' && request.method() === 'GET') {
          const { article_content: _content, video_url: _video, ...summary } = saved;
          return route.fulfill({ json: pageOf([{ ...summary, excerpt: 'Article videos', search_snippet: null }], url) });
        }
        if (path === '/api/v1/content/upload-video') {
          assert.equal(request.method(), 'POST');
          assert.equal(request.headers()['x-csrf-token'], session.csrf_token);
          const mime = request.headers()['content-type'];
          assert.ok(['video/mp4', 'video/webm'].includes(mime));
          const body = request.postDataBuffer();
          assert.deepEqual(body, mime === 'video/mp4' ? mp4UploadBytes : fixtureVideoBytes);
          const value = { id: uploads.length + 80, path: `/media/${(uploads.length + 10).toString(16).padStart(64, '0')}.${mime.split('/')[1]}`, mime_type: mime, size: body.length };
          uploads.push({ value, mime });
          if (mode === 'error') return route.fulfill({ status: 503, json: { code: 'unavailable', message: 'PRIVATE video failure', retryable: true, request_id: 'video-upload-test' } });
          if (mode === 'hold') return new Promise(resolve => { pending = { route, resolve, value }; });
          return route.fulfill({ json: value });
        }
        return route.fallback();
      });

      await page.goto(`${base}/admin?lang=${language}#edit/1`);
      await waitForApplication(page);
      const body = page.getByRole('textbox', { name: names.body, exact: true });
      const contentUrl = page.getByRole('textbox', { name: names.contentUrl, exact: true });
      const mainUrl = page.getByRole('textbox', { name: names.url, exact: true });
      const insert = page.getByRole('button', { name: names.insert, exact: true });
      await expect(body).toHaveValue('甲😀乙尾');
      await selectText(body, 3);
      await contentUrl.fill(youtube);
      await insert.focus(); await page.keyboard.press('Enter');
      await expect(body).toHaveValue(/甲😀[\s\S]*```video\n[^\n]+\n```[\s\S]*乙尾/);
      assert.equal((await body.inputValue()).includes('\uFFFD'), false, 'UTF-16 selection must preserve the emoji');
      await waitForApplication(page);
      await expect(body).toBeFocused();
      assert.deepEqual(await body.evaluate(node => [node.selectionStart, node.selectionEnd]),
        Array(2).fill((await body.inputValue()).indexOf('乙')), 'Caret remains before the original suffix after the reactive render');

      const beforeReplace = await body.inputValue(), replaceAt = beforeReplace.indexOf('乙');
      await selectText(body, replaceAt, replaceAt + 1);
      await contentUrl.fill(directVideo); await insert.click();
      const replaced = await body.inputValue();
      assert.ok(!replaced.includes('乙') && replaced.startsWith('甲😀') && replaced.endsWith('尾'), replaced);
      assert.ok(replaced.indexOf('youtube') < replaced.indexOf(directVideo), 'Video blocks follow insertion order');

      const unchanged = await body.inputValue();
      await contentUrl.fill('javascript:alert(1)'); await insert.click();
      await expect(page.getByRole('alert')).toBeVisible();
      await expect(body).toHaveValue(unchanged);
      await selectText(body, unchanged.length);
      await contentUrl.fill(bilibili); await insert.click();
      await expect(page.getByRole('alert')).toHaveCount(0);

      // Real file selection sends WebM bytes to the existing upload endpoint.
      // Edits made while awaiting the response survive insertion at the saved
      // cursor instead of being replaced by the old upload-time document.
      const beforeUpload = await body.inputValue();
      await selectText(body, beforeUpload.length);
      mode = 'hold';
      await chooseFile(page, names.bodyUpload, { name: 'body.webm', mimeType: 'video/webm', buffer: fixtureVideoBytes });
      await expect.poll(() => Boolean(pending)).toBe(true);
      await expect(page.locator('form.editor-form').first().locator('button[type="submit"]')).toBeDisabled();
      await expect(page.getByRole('button', { name: names.mainUpload, exact: true })).toBeDisabled();
      await body.press('ControlOrMeta+Home'); await body.pressSequentially('New draft 😀'); await body.press('Enter');
      const concurrentDraft = await body.inputValue(), inlineUpload = uploads.at(-1).value;
      await releasePending(); mode = 'success';
      await expect.poll(() => body.inputValue()).toContain(inlineUpload.path);
      assert.ok((await body.inputValue()).startsWith('New draft 😀\n'), 'Upload preserves newer text before the saved cursor');
      assert.ok((await body.inputValue()).includes(concurrentDraft.trimEnd()), 'Upload preserves the latest body');

      await chooseFile(page, names.mainUpload, { name: 'article.mp4', mimeType: 'video/mp4', buffer: mp4UploadBytes });
      await expect.poll(() => uploads.length).toBe(2);
      await expect(mainUrl).toHaveValue(uploads.at(-1).value.path);
      assert.deepEqual(uploads.slice(0, 2).map(upload => upload.mime), ['video/webm', 'video/mp4']);

      const beforeError = await body.inputValue(); mode = 'error';
      await chooseFile(page, names.bodyUpload, { name: 'retry.webm', mimeType: 'video/webm', buffer: fixtureVideoBytes });
      await expect(page.getByRole('alert')).toBeVisible();
      await expect(body).toHaveValue(beforeError);
      await expect(page.locator('body')).not.toContainText('PRIVATE');
      await expect(page.getByRole('button', { name: names.mainUpload, exact: true })).toBeEnabled();
      mode = 'success';

      // The same renderer serves editor previews and public articles. Raw HTML
      // and unsupported video blocks stay escaped, including platform-looking
      // hostile hosts, and never gain arbitrary iframe capabilities.
      const attack = '\n\n<iframe src="https://evil.example/player" onload="window.__articleVideoXss=1"></iframe>\n\n<video src="x" onerror="window.__articleVideoXss=2"></video>\n\n```video\nhttps://www.youtube.com.evil.example/watch?v=dQw4w9WgXcQ\n```\n';
      await body.fill((await body.inputValue()) + attack);
      await mainUrl.fill(youtube);
      await page.getByRole('button', { name: names.preview, exact: true }).click();
      const previewBody = page.locator('.editor-markdown-preview');
      await mediaMarkup(previewBody, 2, [youtubeEmbed, bilibiliEmbed]);
      await expect(previewBody.locator('pre code')).toContainText('https://www.youtube.com.evil.example/');
      await expect(previewBody.locator('script,object,embed,video[onerror],iframe[onload]')).toHaveCount(0);
      await expect(previewBody).toContainText('<iframe src="https://evil.example/player"');
      await noOverflow(page);
      await screenshot(page, `${engine.name()}-${language}-${width}-editor.png`);
      const submitted = await body.inputValue();
      await page.getByRole('button', { name: names.save, exact: true }).click();
      await expect(page).toHaveURL(/#articles$/);
      assert.equal(writes.length, 1); assert.equal(writes[0].article_content, submitted); assert.equal(writes[0].video_url, youtube);
      await page.goto(`${base}/admin?lang=${language}#edit/1`); await waitForApplication(page);
      await expect(page.getByRole('textbox', { name: names.body, exact: true })).toHaveValue(submitted);

      await page.goto(`${base}/article/1?lang=${language}`); await waitForApplication(page);
      const publicBody = page.locator('.article-markdown');
      await mediaMarkup(publicBody, 2, [youtubeEmbed, bilibiliEmbed]);
      const leading = page.locator('.article-featured-media > iframe');
      await expect(leading).toHaveAttribute('src', youtubeEmbed);
      const native = publicBody.locator('video').first();
      await native.evaluate(node => node.load());
      await expect.poll(() => native.evaluate(node => node.videoWidth)).toBe(16);
      for (const frame of await publicBody.locator('iframe').all()) {
        await frame.scrollIntoViewIfNeeded();
        await expect(frame.contentFrame().locator('[data-video-fixture]')).toHaveText('Player loaded');
      }
      await noOverflow(page);
      await screenshot(page, `${engine.name()}-${language}-${width}-public.png`);
      assert.equal(await page.evaluate(() => window.__articleVideoXss), 0);
      assert.ok(externalRequests.length >= 2, 'Players load through actual browser frames');
      assert.ok(externalRequests.every(value => [new URL(youtubeEmbed).origin, new URL(bilibiliEmbed).origin].includes(new URL(value).origin)), JSON.stringify(externalRequests));

      // Canceling an article aborts its pending upload. A late response cannot
      // leak a block or featured URL into a fresh editor mounted afterwards.
      await page.goto(`${base}/admin?lang=${language}#edit/1`); await waitForApplication(page);
      mode = 'hold';
      await chooseFile(page, names.bodyUpload, { name: 'cancelled.webm', mimeType: 'video/webm', buffer: fixtureVideoBytes });
      await expect.poll(() => Boolean(pending)).toBe(true);
      const cancelledPath = uploads.at(-1).value.path;
      await page.getByRole('button', { name: names.cancel, exact: true }).click();
      await expect(page).toHaveURL(/#articles$/);
      await releasePending(); mode = 'success';
      await page.goto(`${base}/admin?lang=${language}#new`); await waitForApplication(page);
      const newBody = page.getByRole('textbox', { name: names.body, exact: true });
      await expect(newBody).toHaveValue('');
      await expect(page.getByRole('textbox', { name: names.url, exact: true })).toHaveValue('');
      await page.getByRole('textbox', { name: names.contentUrl, exact: true }).fill(directVideo);
      await page.getByRole('button', { name: names.insert, exact: true }).click();
      await expect(newBody).toHaveValue(/```video\n\/__test-media\/background\.webm\n```/);
      assert.equal((await newBody.inputValue()).includes(cancelledPath), false);
      assert.equal(writes.length, 1, 'Uploads, previews and insertions never save the article implicitly');
      assert.deepEqual(errors, []);
      console.log(`${engine.name()} ${language} ${width}px: cursor/UTF-16, multiple videos, safe platform frames, native playback, WebM/MP4 uploads, draft preservation, errors, cancellation and save/reload passed`);
    } finally {
      await releasePending();
      await context.close(); await browser.close();
    }
  }
} finally { await server.close(); }
