# Rust Web 构建

前端生产链由 `tools/assets` 维护，工具名 `blog-assets`。应用包 `blog-browser` 位于 `src/browser`，组件门面、DOM 核心与宏分别位于 `src/interface`、`src/interface/dom` 和 `src/interface/macros`，CSS 和字体位于 `web/styles`。所有 Rust 包使用根 workspace/lock 和本地路径源码；固定来源与许可证见 `compliance/` 和[源码管理说明](rust-sources.md)。

```sh
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- serve --product-root . --debug --bind 127.0.0.1:5173
cargo +1.99.0 run --locked --offline -p blog-assets -- server --product-root . --config blog-assets.json --mode development --offline
```

需要 Rust 1.99.0、`wasm32-unknown-unknown` 和原生编译环境。`tools/assets/src/browser_module.rs` 在 `blog-assets` 进程中直接生成浏览器 WASM 绑定；没有独立绑定工具包或 CLI 安装步骤。绑定相关实现通过本地用途包参与构建，不依赖 PATH 中的 wasm-bindgen 命令或注册表源码缓存。WASM 的正式优化使用根 `web-release` profile。

默认生成 `web/pkg` 与 `web/dist`；`--dist` 或 `BLOG_WEB_DIST` 可指定独立输出。生产资源编译不运行 npm、Node 或相邻项目脚本。构建器保留实际依赖图、源码变更检测、资源预算、许可、MIME、gzip 与嵌入摘要校验。

根 `blog-assets.json` 指定服务端与前端构建入口；分发资源内的 `assets/blog-assets.json` 记录本次真实代码与资源哈希。运行时标记为 `html[data-blog-runtime="blog-interface"]`。来源记录描述代码的上游历史；当前可维护源码的哈希与当次构建绑定，不伪装为未修改的上游源码。

当前组件实现只保留本项目的浏览器 CSR 用途；RSX 解析、服务端渲染、水合、岛屿与拆分加载等不再属于支持面。具体裁剪和证据边界见[用途架构与定制](maintained-design.md)。

浏览器工具仍使用独立的 Node/Playwright 依赖，覆盖真实 WASM、后台、公开页面、评论与 Markdown 安全边界，详见[浏览器验证](browser-testing.md)。原生单测、模拟接口页面测试、真实后端与实际安装验收分别报告。
