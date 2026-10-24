# 浏览器验证

浏览器测试使用 Node 26.7.0、Playwright 和 axe，执行 Chromium 与 Firefox。版本由 `src/web/package.json` 和 `src/web/package-lock.json` 锁定；浏览器二进制及系统库单独安装。生产资源由 Rust 构建，测试工具只用于驱动和检查页面。

## 准备与执行

先按[开发说明](development.md)准备 Rust、WASM 目标和 Linux 编译环境，再从根目录执行：

```sh
cd src/web
npm ci
npx playwright install --with-deps chromium firefox
npm run test:unit
npm run test:browser:blog-interface
```

`npm ci` 和浏览器安装可能联网。固定版本、依赖关系及常用 API 见[测试工具说明](dependencies/testing.md)。

| 入口 | 检查范围 | 数据来源 |
| --- | --- | --- |
| `npm run test:unit` | 工具链合同、Rust 域测试、浏览器 fixture 单测 | 本地测试输入 |
| `npm run test:browser:blog-interface` | WASM 运行、异步状态、流取消、阅读交互和页面冒烟 | 脚本指定的 API fixture 或独立原生服务 |
| `npm run test:admin` | 管理布局、账号设置、幂等提交确认与表单恢复 | API fixture |
| `npm run test:public` | 公开布局、读取错误恢复、分页、语言、弹窗和匿名评论行为 | API fixture |
| `bash test/operations/smoke.sh`（根目录） | 真实 HTTP、SQLite、登录、公开页面与重启持久化 | 独立可丢弃 ROOT |
| CI `installed_candidate` | 候选包完整性、安装、凭据、权限与实例隔离 | 独立安装和真实后端 |

`test/browser/admin-recovery.mjs` 和 `test/browser/public-recovery.mjs` 覆盖丢失响应、原提交确认、可编辑草稿、并发分页、语言链接及原生弹窗的键盘行为。

`test/browser/style-contract.mjs` 测量后台实际页面边距、控件尺寸、自然换行导航和 G4 轮廓状态。`test/browser/public-style-baseline.mjs` 可通过 `BLOG_PUBLIC_BASELINE_DIST` 指定修改前的独立产物，对照公开页的布局、尺寸、字体、颜色与圆角边缘。

页面脚本会执行管理写入或创建可丢弃数据，应使用专用测试实例。`test/browser/rust-native-domain.mjs` 使用真实生成资源，API 和剪贴板使用 fixture；持久化与真实数据库行为由后端和安装测试覆盖。

## WASM 启动检查

测试等待 `html[data-blog-runtime="blog-interface"]`，检查实际 `.wasm` 请求、`application/wasm` MIME、WASM magic 字节、资源清单中的原始 SHA-256，以及 pageerror、登录和 session 行为。

条件请求返回 304 时，需要同一 URL 先有有效 200，运行时已执行，直接 GET 可取得正确 WASM，并保存 ETag 与条件请求证据。HTML 页面可访问和 WASM 交互成功分别检查。

## CI 分组

`.github/workflows/ci.yml` 分别运行服务端、Chromium/Firefox 的管理与公开页面组，以及独立安装测试，聚合门禁要求各组成功。构建检查、fixture 页面测试、真实后端写入和安装测试各自验证对应边界；具体命令以工作流和 `src/web/package.json` 为准。
