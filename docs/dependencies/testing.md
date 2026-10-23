# 测试与审计工具

浏览器测试使用 Node 26.7.0。npm 包只用于测试驱动、断言和无障碍检查；生产资源由 Rust 构建。依赖及完整性摘要由 [package.json](../../src/web/package.json) 和 [package-lock.json](../../src/web/package-lock.json) 固定。

## 锁定的 npm 包

| 包 | 版本 | 依赖关系 | 本地说明 | 许可 |
| --- | --- | --- | --- | --- |
| `@playwright/test` | 1.63.0 | 直接开发依赖，依赖 `playwright` | [断言与测试入口](#playwrighttest) | Apache-2.0 |
| `playwright` | 1.63.0 | 由 `@playwright/test` 引入，依赖 `playwright-core` | [浏览器驱动](#playwright) | Apache-2.0 |
| `playwright-core` | 1.63.0 | 由 `playwright` 引入，同时满足 axe 适配器的 peer 依赖 | [协议与浏览器文件](#playwright-core) | Apache-2.0 |
| `@axe-core/playwright` | 4.12.1 | 直接开发依赖，依赖 `axe-core` | [无障碍适配器](#axe-coreplaywright) | MPL-2.0 |
| `axe-core` | 4.12.1 | 由 `@axe-core/playwright` 引入 | [规则引擎与结果](#axe-core) | MPL-2.0 |

`@axe-core/playwright` 声明的 `axe-core ~4.12.1` 和 `playwright-core >=1.0.0` 由锁文件解析到表中精确版本。新增脚本继续使用这组依赖，安装使用 `npm ci`。

## 安装与执行

准备 Rust 工具链与 WASM 目标后，在仓库根目录执行：

```sh
cd src/web
npm ci
npx playwright install --with-deps chromium firefox
npm run test:unit
npm run test:browser:blog-interface
npm run test:admin
npm run test:public
```

`npm ci` 安装锁定工具包；浏览器安装准备匹配的执行文件和 Linux 系统依赖，可能联网并需要安装系统包的权限。浏览器脚本通过 Rust 预览服务读取实际资源，API 可以由脚本 fixture 提供；测试边界见[浏览器验证](../browser-testing.md)。

## @playwright/test

本项目从 `@playwright/test` 导入 `chromium`、`firefox` 和 `expect`。页面脚本是由 Node 直接执行的 `.mjs`，命令由 npm scripts 和 [CI](../../.github/workflows/ci.yml) 组织。

常用操作包括：

- `expect(locator).toBeVisible()`、`toHaveText()`、`toHaveValue()`：自动等待页面满足断言。
- `expect(page).toHaveURL()`：检查页面路由；`expect.poll()`：等待异步计算结果。
- `page.getByRole()`、`getByLabel()`：按可访问角色和表单名称定位；`page.locator()`：按 CSS 定位。

示例见[管理布局测试](../../test/browser/admin-layout.mjs)和[界面运行测试](../../test/browser/blog-interface-runtime.mjs)。交互前等待应用运行标记或实际可见状态，使用新的 browser context 隔离测试会话。

## playwright

`playwright` 提供浏览器自动化 API，并导出给测试包使用。典型生命周期为 `engine.launch()` → `browser.newContext()` → `context.newPage()`；在 `finally` 中关闭 context 和 browser，释放进程与页面资源。

项目同时执行 Chromium 与 Firefox，按场景设置 `locale`、`viewport` 和 `colorScheme`。`page.route()` 拦截 API 并返回 fixture，`page.on('pageerror', ...)` 收集页面错误；监听响应可检查实际 WASM 的 MIME、内容和缓存行为。可复用的 fixture 入口见 [browser-fixtures.mjs](../../test/browser/browser-fixtures.mjs)。

## playwright-core

`playwright-core` 保存浏览器协议实现、类型声明、CLI 和所需浏览器修订信息。浏览器执行文件由安装命令单独准备；包版本与执行文件共同决定测试环境。项目通过上层包使用它，不需要新增直接依赖。

预览适配器 [test-preview.mjs](../../test/tools/assets/test-preview.mjs) 启动 Rust `blog-assets preview`，读取监听地址并在结束时清理子进程。添加场景时沿用此入口，保持浏览器驱动与资源服务的生命周期一致。

## @axe-core/playwright

适配器将 axe 规则引擎注入受测页面及相关 frame，提供可链式配置的 `AxeBuilder`。本项目使用默认导入：

```js
import assert from 'node:assert/strict';
import AxeBuilder from '@axe-core/playwright';

async function checkAccessibility(page) {
  const { violations } = await new AxeBuilder({ page })
    .include('.blog-admin-shell')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
    .analyze();
  assert.deepEqual(violations, []);
}
```

`.include(selector)` 限定检查区域，`.exclude(selector)` 排除区域，`.withTags(tags)` 选择规则标签，`.analyze()` 返回检查结果。区域和排除规则会改变覆盖范围，应在具体测试中保持明确。实际调用见[管理布局测试](../../test/browser/admin-layout.mjs)。

## axe-core

`axe-core` 是在页面中执行的无障碍规则引擎，由适配器管理加载。其 `axe.run(context, options)` 返回结果，主要字段包括 `violations`、`passes`、`incomplete` 和 `inapplicable`。违规项包含规则 `id`、影响级别和受影响 `nodes`；节点的 `target` 用于定位具体元素。

项目从 `violations` 提取规则和节点目标以报告失败。`incomplete` 表示规则无法自动作出结论，需结合页面检查；自动扫描通过只能说明已执行规则的结果。可访问名称、键盘、焦点、对比度与交互行为还由对应浏览器场景检查。

## Cargo 审计工具

| 工具 | 固定版本 | 用途与入口 |
| --- | --- | --- |
| `cargo-audit` | 0.22.2 | [audit.py](../../src/tools/rust-sources/audit.py) 从当前来源映射恢复原始 registry 身份，调用审计并启用 `--deny yanked` |

CI 中的工具安装命令为：

```sh
CARGO_NET_OFFLINE=false cargo install cargo-audit --version 0.22.2 --locked
python3 src/tools/rust-sources/audit.py
```

工具安装、advisory 数据和 yank 信息更新使用网络。产品的离线 Cargo 源码解析、浏览器工具安装、浏览器执行文件及系统依赖分别准备。来源与审计身份说明见[源码布局与校验](../source-layout.md)。
