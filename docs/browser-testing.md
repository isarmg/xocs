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

`test/browser/pending-comment-binding.mjs` 检查会话存储中的待确认评论与当前文章或评论区绑定，拒绝缺失、错绑及未知字段，同时保留有效提交的原幂等标识。

`test/browser/editor-preview.mjs` 用真实输入和暂停时钟验证预览的 200ms 防抖、关闭与卸载取消，公开页和后台页的文档标题，以及屏外正文图片滚动到视口前后的实际请求。`test/browser/startup-failure.mjs` 验证字体失败或持续等待时页面仍可用，同时保留 WASM 加载失败、超时和手动重载恢复。

`test/browser/article-video.mjs` 在真实编辑器和公开文章中检查视频上传、正文光标插入、多播放器预览、保存与重新加载、受限平台嵌入和上传失败恢复。平台播放器请求使用本地响应，测试不依赖第三方服务；实际编码与媒体响应由视频输入和原生服务测试覆盖。

`test/browser/style-contract.mjs` 在 390px、1280px、2560px 的明暗主题中检查后台全部页面，包括有数据的表格和横向滚动操作、分页、多行字段、Markdown 预览、媒体查看器、通知与认证错误，以及下拉菜单的正常绘制和回退交互。它同时测量实际页面边距、控件尺寸、自然换行导航和 G4 轮廓状态。`test/browser/public-style-baseline.mjs` 可通过 `BLOG_PUBLIC_BASELINE_DIST` 指定修改前的独立产物，对照公开页的布局、尺寸、字体、颜色与圆角边缘。

`test/browser/corner-candidates.mjs` 检查公开页和管理页的正圆角尺寸均属于已登记样式角色，以及动态插入、属性和类变化、移除后的绘制状态。它还对相同 DOM 测量全元素样式发现与缓存候选的局部成本；设置 `BLOG_CORNER_BENCHMARK_OUT` 可保存结果，这项数据不代表整页渲染性能。

DOM 边界回归先运行 `bash test/interface/dom/browser/build.sh`，再执行 `node test/interface/dom/browser/helper-contracts.mjs` 与 `node test/interface/dom/browser/dom-errors.mjs`。前者通过测试专用 WASM 消费者验证计时、微任务、监听器的失败与清理，以及条件和类型擦除属性切换时旧属性与资源的移除；后者向生产 WASM 的浏览器操作注入真实异常，验证错误到达外层且启动失败能显示错误页。CI 与发行验证均执行这组双浏览器回归。

在仓库根运行 `bash test/interface/browser/build.sh` 和 `node test/interface/browser/corner-listener-lifecycle.mjs`，可通过公开圆角 API 验证 iframe 导航、停止与析构。用例覆盖同源窗口转为跨源后释放，以及浏览器移除监听器失败时的原始错误和回调生命周期。

页面脚本会执行管理写入或创建可丢弃数据，应使用专用测试实例。`test/browser/rust-native-domain.mjs` 使用真实生成资源，API 和剪贴板使用 fixture；持久化与真实数据库行为由后端和安装测试覆盖。

## WASM 启动检查

测试等待 `html[data-blog-runtime="blog-interface"]`，检查实际 `.wasm` 请求、`application/wasm` MIME、WASM magic 字节、资源清单中的原始 SHA-256，以及 pageerror、登录和 session 行为。

条件请求返回 304 时，需要同一 URL 先有有效 200，运行时已执行，直接 GET 可取得正确 WASM，并保存 ETag 与条件请求证据。HTML 页面可访问和 WASM 交互成功分别检查。

## CI 分组

`.github/workflows/ci.yml` 分别运行服务端、Chromium/Firefox 的管理与公开页面组，以及独立安装测试，聚合门禁要求各组成功。构建检查、fixture 页面测试、真实后端写入和安装测试各自验证对应边界；具体命令以工作流和 `src/web/package.json` 为准。
