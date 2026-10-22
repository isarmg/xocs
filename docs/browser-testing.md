# 浏览器验证

生产资源由 Rust 构建。Node 26.7.0、Playwright 与 axe 仅提供测试驱动：两个直接 devDependencies、五个非根锁包分别为 `@axe-core/playwright`、`@playwright/test`、`axe-core`、`playwright` 和 `playwright-core`。所有锁包标记为开发依赖；浏览器二进制和系统库另行安装，不能把 node_modules 当前目录当作锁定依赖图。

在 `web` 中执行 `npm ci`，再运行 `npx playwright install --with-deps chromium firefox`。这部分安装有独立的网络与缓存需求，不属于 Rust 的空 Cargo home 离线证明。

| 入口 | 主要范围 | API 边界 |
| --- | --- | --- |
| `npm run test:unit` | 来源/工具链合同、Rust domain 单测、浏览器 fixture 单测 | 不代替真实服务验收 |
| `npm run test:browser:blog-interface` | 实际 WASM、运行时、异步状态、阅读交互及代表性页面冒烟 | 各脚本显式使用 fixture 或自己创建的原生服务 |
| `npm run test:admin` / `npm run test:public` | 管理/公开布局、账号与匿名评论行为 | 主要使用明确的 API fixture |
| `bash scripts/smoke.sh`（根目录） | 真实 Axum/SQLite、管理员与公开页面、重启持久化 | 创建独立测试 ROOT，执行真实写入 |
| CI `installed_candidate` | 精确候选包、安装/维护、凭据与安全合同 | 独立安装与真实后端，不依赖页面 fixture |

不能将需要可丢弃数据的 smoke/管理写入脚本直接指向正在使用的站点。`web/tests/rust-native-domain.mjs` 使用真实生成资源，但 API 和剪贴板有明确 fixture；它的 DOM/Markdown 结论不能描述为真实数据库写入证明。

浏览器需等待 `html[data-blog-runtime="blog-interface"]`，核对实际 `.wasm` 请求、`application/wasm` MIME、magic 字节和前端清单中的原始 SHA-256，并检查 pageerror、登录与 session。缓存 304 只有在同一 URL 已有有效 200、运行时确实执行且直接 GET 能获得正确 WASM 时才可接受，保留 ETag/条件请求证据。HTML 返回 200 不能代替 WASM 或交互验收。

CI 保留 Chromium/Firefox、管理/公开分组及全部已有断言；聚合门禁要求服务端、两组浏览器和独立安装 lane 全部成功。构建、模拟接口浏览器测试、真实后端验收、安装和发行分别报告，不能相互替代。
