# Rust 原生 Web 构建与测试

生产前端编译、资源整理、gzip、清单、预算/许可证检查和后端集成使用本仓库 `tools/web-build`，不需要 `npm ci`、Node 或 Vite。后端仍为 Axum。前端只使用裁剪的 xcss-web CSR；框架源码与公共组件由 xcss 统一维护与同步。

在产品根目录执行：

```sh
cargo +1.99.0 run --quiet --locked --manifest-path tools/web-build/Cargo.toml -- build --product-root .
cargo +1.99.0 run --quiet --locked --manifest-path tools/web-build/Cargo.toml -- serve --product-root . --debug --bind 127.0.0.1:5173
cargo +1.99.0 run --quiet --locked --manifest-path tools/web-build/Cargo.toml -- server --product-root . --config xcss-web-build.json --mode development
```

需要 Rust 1.99.0、wasm32-unknown-unknown target 和 wasm-bindgen-cli 0.2.129。默认输出 `web/dist`；`--dist` / `XCSS_WEB_DIST` 用于隔离验收构建。`serve` 监听 Rust/资源文件变化；`preview` 只提供已有资源，不重新构建。Cargo 缓存齐备时可显式传 `--offline`，默认允许获取锁定依赖。

package.json 的 build/dev/check/test Rust 入口只是这些命令的便利别名。Node、Playwright 与 axe 只承担浏览器和测试夹具验证；生产构建不读取 node_modules。已退役的 React/TypeScript 对照实现、Hook 模拟器、Vite 转换器、MarkdownIt 与 DOMPurify 已删除。当前 Markdown 解析与 URL/HTML 边界由 Rust 实现。

`npm run test:unit --prefix web` 执行当前前端 Rust 单测、独立 Rust domain 测试及 4 个仍被真实浏览器脚本使用的夹具合同测试。现行浏览器脚本全部保留；`tests/async-state.mjs` 在 Chromium/Firefox 直接加载当前 WASM，补充后台分页失败重试、删除串行与末页回退、页面离开后的迟到响应、上传取消和读恢复不覆盖未知写入状态。原对照断言可从 Git 历史查阅，不能作为当前实现通过的证明。

`tools/web-build/test-preview.mjs` 是测试专用适配器：浏览器脚本仍通过 Node 执行，但 HTTP 资源由 Rust CLI `preview` 提供。测试默认申请端口 0，读取 listening JSON 后返回实际端口，并在清理时结束 Rust 子进程。可设置 `XCSS_WEB_BUILD_BIN` 复用预编译工具，避免每个测试重复启动 Cargo。

CI 与正式打包脚本的生产 build/server 直接调用 Cargo，保留原 source revision、receipt、精确安装器与包校验守卫。浏览器测试仍安装其锁定 devDependencies；它们属于验证工具，不属于 Axum 运行时或生产前端构建依赖。
