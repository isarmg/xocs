> 历史记录：本文描述改名前 xocs 的迁移与验证。当前 blog 的目录、独立维护方式与新验证边界见 [独立实现说明](independence.md) 和 [开发说明](development.md)。历史数据不代表改名后的发行或验收结果。

# Web 前端迁移记录

当前入口为 `web/xcss-web`，仅通过 `xcss-web` 路径 crate 接入；主分支使用 Rust 生产构建。构建与验收命令见 [Rust 原生 Web 构建与测试](rust-native-web.md)。本文保留首次迁移时的流程和测量记录，旧 Node/Vite 命令与体积数字不代表本次改名后的验证结果。

Web 业务界面位于 `web/xcss-web/src` 的 Rust / xcss-web CSR，服务端仍为 Axum，使用当前 `/api/v1`、Cookie 会话、CSRF、数据库和媒体存储合同。Rust CLI 编译独立 WebAssembly 并整理样式、字体、启动资源和构建清单。

当前 `main` 已合并 xcss-web CSR 与 Rust 原生构建；直接在主分支构建和验收。下面的首次迁移流程与数字作为历史记录，正式发行和线上部署继续使用产品发布流程。

## 源码位置

| 模块 | 功能 |
| --- | --- |
| `public.rs` | 首页栏目、文章阅读/解锁/目录/复制、分类标签、搜索与历史、游记、随笔、相册、百宝箱、恋爱计时与页签、信笺 |
| `admin.rs` | 后台导航、总览、文章列表、分类标签、评论/留言/文件、站点设置、首页栏目、信笺、家的资料、内容资源 |
| `editor.rs` | Markdown 编辑器、上传与预览、访问密码、文章进展、随笔编辑及可见性 |
| `comments.rs` | 三类匿名评论、线程回复、分页、待确认提交持久化和同收据重试 |
| `select.rs` | 原自定义选择器的 Rust 移植，包括键盘、ARIA、top-layer 列表、禁用和表单验证 |
| `contracts.rs` | 页面与写入响应合同、分页身份和安全整数校验 |
| `dates.rs` | SQLite/ISO UTC 时间、周年/月末计时、UTF-8 密码和文件名边界 |
| `common.rs` | 页面读写、取消、错误文案、表单、分页、图片对话框、文件选择 |
| `markdown.rs` / `browser.rs` | Rust Markdown/URL 安全策略、浏览器布局测量与文档级原生事件适配 |
| `vendor/xcss-web` | 独立构建所需公共 xcss-web UI、认证及有界网络请求快照 |

公共快照由 `SOURCE.json` 标记来源和逐文件哈希。不要直接修改 vendored 公共代码；修改公共来源后重新同步快照。构建器还校验编译前后输入一致，防止产物与源码错配。

## 当时的构建配置（历史记录）

以下安装命令与独立锁文件是首次迁移时的环境记录，不是当前 blog 的准备条件。当前单工作区离线构建自动编译本地绑定工具，见[当前构建命令](rust-native-web.md)。

生产构建需要 Rust 1.99.0、`wasm32-unknown-unknown` 和 wasm-bindgen-cli 0.2.129。浏览器验证另需 Node.js 26.7 和锁定 npm 依赖。

```sh
rustup toolchain install 1.99.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown
cargo +1.99.0 install wasm-bindgen-cli --version 0.2.129 --locked
cargo +1.99.0 run --quiet --locked --manifest-path tools/assets/Cargo.toml -- build --product-root .
```

`web/pkg` 是生成目录，不提交。独立前端 Cargo 工作区及 Cargo.lock 位于 `web/xcss-web`。当前 Rust CLI 的 `cargo +1.99.0 run --locked --manifest-path tools/assets/Cargo.toml -- check --product-root .` 检查 WASM 目标；将 `check` 改为 `test` 执行原生 Rust 合同与时间边界单测。退役的 Python 构建器及其专属测试已移除。可显式设置 `CARGO_TARGET_DIR` 共享编译缓存。

## 验收规则

必须检查 `dist/assets` 中实际存在独立 `.wasm`，并在浏览器等待 `html[data-xcss-runtime="xcss-web"]`。仅设置前端环境变量不足以证明 xcss-web 入口已经执行。原浏览器脚本的行为断言保留，新 `tests/xcss-web-runtime.mjs` 补充 Rust 运行时的键盘选择器、写入幂等、重复提交、脏草稿重试、分页取消和刷新后的提交恢复。

测试夹具同步真实响应：JSON null 明确携带 application/json，音频使用有效的短 WAV；页面就绪通过 Rust 标记和当前 API 请求完成判定，URL 条件直接轮询浏览器当前地址。页面、交互、语言、错误恢复和可访问性断言均保留。

默认 `npm run build` / `npm run dev` 调用 Rust CLI；开发命令监听 Rust/资源变化。`npm run test:unit` 执行当前原生 Rust、domain 与浏览器夹具测试；退役的 React 对照实现、Hook 模拟器和专属 JavaScript 单测已删除，历史行为可从 Git 历史查阅。现行浏览器验收与新增 `tests/async-state.mjs` 直接验证当前 WASM 的取消、分页、失败恢复和副作用边界。

Rust CLI 输出 `assets/xcss-web-build.json`，记录 Rust、公共快照和生成文件哈希；真实运行时测试核对浏览器读取的 WASM 与清单完全一致。生产构建拒绝 React 进入模块依赖图。

## Axum 适配

WASM 在构建时生成 gzip 变体并加入内嵌资源清单，HTML 预加载 WASM。Axum 按 `Accept-Encoding` 协商，设置 `application/wasm`、`Content-Encoding: gzip` 和 `Vary: Accept-Encoding`，不支持 gzip 时回退原始文件。不同表示保留各自强 ETag；HEAD、304、HTML 的 no-store 和其它资源的重验证规则保持一致。API、认证、Cookie、CSRF、媒体和数据结构继续使用原合同。

协商与缓存语义由公共 `xcss::web_assets::wasm` 实现；`src/static_assets.rs` 负责产品资源接入。真实 Axum 冒烟验证解压后的哈希、HEAD/304 与显式禁用 gzip。

## 依赖审计和清单

CI 的独立安装验收及发布打包 job 使用固定版本 cargo-audit 对 `web/xcss-web/Cargo.lock` 执行 `--deny yanked` 审计。服务端原有 RustSec 封存、隔离审计及远端 packager 的固定来源保持一致，不增加忽略规则。

`assets/xcss-web-build.json` 包含前端 Rust 的锁定包、版本、来源及校验和，作为随二进制内嵌的补充清单。合并服务端与前端的 CycloneDX 通过固定版本的发行工具生成，并按当前正式包与源码身份独立核验；历史构建数字不能替代当前包的证据。

## 本轮验证结果（2026-10-16）

- 默认 release 构建、WASM 严格 Clippy（`-D warnings`）、Rust 格式检查及 Axum 重新嵌入通过；前端原生 Rust 单测 9 项、后端库测试 65 项通过。构建器 3 项与 CI 分支门禁 3 项通过，公共来源快照与适配器校验通过。
- 原语言、管理布局、账号设置、卸载清理、管理操作、合同修复、公开页脚、公开布局与匿名评论浏览器断言已在 Chromium、Firefox 执行；新 `xcss-web-runtime.mjs` 在两个引擎验证实际 WASM 哈希、选择器键盘/禁用/重置、主题同步、幂等创建、独立重试、分类标签并发删除与草稿保留、资料读取失败禁写、匿名评论持久化恢复。
- 原公开/后台 Chromium 真实 Axum 冒烟、重启持久化、文章解锁与评论审核通过。`management-native.mjs` 在 Chromium、Firefox 各通过 25 笔真实 UI 写请求，覆盖上传/引用保护、随笔、站点、信笺、家的资料连续保存、内容资源编辑审核、分类标签创建编辑、文章创建、UTC 进展编辑与访问密码提示。每个引擎恢复原配置并清理自身记录，页面异常为零；预期的引用保护 409 保留断言。
- 分类读取使用 `/api/v1/categories`，写入使用 `/api/v1/content/categories`；夹具准确拒绝后者的 GET。分类与标签按各自 DTO 选择字段，家的资料把读取响应的数字状态转换为布尔表单值，连续保存不携带只读字段。
- 清洁 `npm ci --offline` 与默认构建通过，76 个非根、非链接锁定 npm 条目均保留来源和 integrity，生产依赖中没有 React runtime。310 项历史对照断言在禁止实际导入 React 的检查下通过；这些对照不替代实际 xcss-web 验收。
- 最终 WASM 为 3,023,038 字节，gzip 为 903,172 字节，较原始 WASM 减少 70.12%。真实 Axum 验证解压后字节/哈希、MIME、Vary、表示各自的 ETag、HEAD、304 和禁用 gzip 回退。此数字衡量压缩效果，不表示已测量 React 与 xcss-web 的启动或交互性能。
- 前端 Cargo.lock 的 187 个依赖经 cargo-audit 审计，未报漏洞或 yanked 错误；保留上游 `paste 1.0.15` 的未维护提示 `RUSTSEC-2024-0436`，没有增加忽略规则。

一次并发负载下的 Firefox 真实流程出现菜单点击后未选中。独立真实 Axum 选择重复 25 次、受限高度菜单滚动探针均正常，随后 Firefox 原管理流程定向复测的全部断言通过；未给失败点击添加自动重试，尚未证实具体根因。测试保留事件目标、坐标及滚动诊断，后续若复现应依据该记录定位。管理脚本可用 `XOCS_SMOKE_ENGINES=firefox` 选择引擎，默认仍执行 Chromium 和 Firefox。

功能验收已在临时回环服务中完成；迁移源码单独提交和推送到本文所述分支，尚未发布或部署。`web/dist`、`web/pkg`、编译缓存均为可重建产物，清理后按本文构建命令重新生成。
