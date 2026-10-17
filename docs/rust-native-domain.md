# xocs 前端业务 Rust 化与交互优化

本轮保持 Axum、API 路径、字段和鉴权/幂等约定，前端仍使用 Leptos CSR。生产资源构建的变更见工作区总说明；本文只记录 xocs 的业务状态、Markdown 与交互迁移。

## 明确类型进入实际页面

`web/leptos/src/models.rs` 定义 `Site`、`SiteStats`、`AdminStats`、`Category`、`Label`、`HomeSection`、`ArticleSummary`、`Article`、`ArticleAccess`、`News` 与 `Page<T>`，在网络边界先执行原 `contracts::validate` 和 `matches_page`，再反序列化。缺失的必需可空字段、超出 JS 安全整数范围的数值、错误页码/页大小、错误数字标志等仍会被拒绝。允许后端增加未消费的字段。

类型实际用于首页/推荐/栏目、文章分类与搜索、文章详情和密码解锁、阅读元数据和文章进展、后台统计与文章列表、文章编辑器读取与分类/标签选项、文章进展列表。DTO 只保留页面实际消费的字段；未消费字段仍由原合同校验，避免重复保存/反序列化。`Data<T>` 与分页组件共用同一套请求取消、加载、错误、上一成功状态及重载逻辑。

文章编辑器采用 `ArticleDraft`，从界面状态到提交载荷都按 Rust 字段编译检查，使用 `Serialize` 在发送边界转换为 JSON。写入标志继续使用布尔值，读取标志继续使用后端的 `0/1` 数字协议；空封面、视频、提示、密码继续转为 `null`，空密码仍保留已有密码。新增文章的幂等键、重复提交抑制、上传取消和结果确认路径保留。

随笔、信笺、家庭、链接、资源与审核的通用编辑表单仍使用 `serde_json::Value`。这些路径继续执行原严格响应合同；本轮并未宣称整个应用所有状态都已变为 DTO。

## 阅读与表单行为移入 Rust

删除 `web/leptos/src/browser.js`，移除其 npm Markdown/清理和管理界面 i18n imports。对应功能移入 `browser.rs` 与 `markdown.rs`：

- 使用固定版本 `pulldown-cmark 0.13.4` 解析 CommonMark 结构事件，启用表格和删除线；软换行仍变为换行标签。
- 将所有原始 HTML/内联 HTML 事件变为转义文本。原始 HTML 始终禁用，没有接受任意 HTML 的自制清理器。
- 在上游 HTML renderer 输出前验证链接/图片目标。WASM 使用浏览器标准 URL 解析；原生测试使用 `url 2.5.8`。链接只允许 HTTP、HTTPS、mailto、FTP、FTPS、tel；图片只允许 HTTP、HTTPS 和相对路径。禁止 JavaScript、VBScript、file、data、blob 等目标。
- 使用 `linkify 0.11.0` 保留裸网址、无协议域名和邮箱链接识别；链接文本、代码和图片 alt 不会重复嵌套生成链接。
- 通过 `web-sys` 给 h2–h4 添加稳定目录 ID、收集类型明确的标题、添加代码复制按钮和图片键盘/无障碍属性，保留阅读进度、复制反馈与灯箱。
- 通过 Rust `ResizeObserver` 测量标题滑动，组件销毁时断开观察；字体就绪后的回调先检查组件是否仍有效。
- 表单校验本地化、输入后清除自定义错误、站内链接语言传播改用 Rust 文档事件监听。四个监听器只安装一次，生命周期为当前文档。

生产仍有 wasm-bindgen 绑定、加载入口和公共轮廓绘制兼容模块；这里移除的是 xocs 阅读与表单行为的 JavaScript 桥。

这改变了 Markdown 引擎，CommonMark 边缘语法与 markdown-it 不保证逐字节输出相同。出于明确的 URL 策略，内嵌 data 图片和未列出的自定义协议不再接受；站内上传图片与正常网络图片保留。Markdown 的解析器本身进入 WASM，不能仅依据 JS 变少就宣称总下载体积或启动变快，需以最终产物和浏览器测量为准。

## 修复选择菜单的布局反馈

旧实现把指针悬停与键盘导航写入同一 `active` 信号，其 effect 会重新计算弹层位置并调用 `scrollIntoView`；后者可以滚动列表以外的祖先，进而触发文档 scroll 和再次定位。前次实测记录了 Firefox 从 pointerdown 到 click 期间弹层上移，click 落到 fieldset 的失败。

现在指针悬停只改变活动项外观。打开菜单和键盘导航单独触发选项可见性调整，且只修改弹层自己的 `scrollTop`；列表内部滚动不会再次触发锚点定位。窗口/文档位置真正改变时仍会重新定位，键盘方向键、Home/End、前缀搜索、Enter/Escape/Tab、原生必填校验和禁用状态继续保留。

## 普通 Rust 函数式视图

本项目的 268 个 authored `view!` 调用已转换为普通 Rust DOM/组件 builder，保留 `#[component]`、原业务表达式、受控状态、列表 key、事件和清理机制。使用 `leptos::html`、`leptos::svg`、`leptos::ev` 公开路径及明确导入的组件 props helper；没有以 HTML 字符串替代界面结构。Markdown 内容依然通过独立的安全内容渲染路径进入 `inner_html`。

转换复用 xcss 正式集成宏的 `render_view`，只在维护工具执行一次；应用后续直接维护普通 Rust 源码；builder 和组件 props 适配比 RSX 更长，转换器不是每次构建的步骤。静态 inert HTML 优化关闭是用户选择函数式源码的取舍，不能据此宣称产物更小或更快。正式宏的无 props 组件适配块保留原求值顺序，只在该已生成块局部处理 `unit_arg`/`let_unit_value` lint，没有全文件允许警告。

实际 CSR host fingerprint 不含 internal erasure；首次错误采用 metadata 的跨 target 功能并集，类型检查发现并拒绝该输出。错误阶段源码和诊断保留在本地实验记录，显式恢复只还原生成的 `StaticVec`/闭包适配两种形状，重新验证后通过。

## 验证方式与边界

本次验收曾在 `/mnt/sarmg.org/.local/rust-native-web-2026-10-17` 完成；该临时目录已于 2026-10-17 随迁移收尾清理。下文保留当时 evidence/native-verification 的证据文件名、SHA 与验收结果；源码及结果摘要已合并 main，日志和测试数据不进入产品发行包。

无需 Node 的原生安全/响应合同测试：

```sh
cargo test --locked --offline --manifest-path web/leptos/domain-tests/Cargo.toml
```

这个轻量测试 crate 直接引入生产 `contracts.rs`、`models.rs`、`markdown.rs`，避免维护独立测试实现。13 个测试通过，覆盖必需可空字段、整数上界、分页身份、数字标志、写入空值/布尔字段、原始 HTML/SVG/MathML、URL 协议和实体/控制字符混淆、属性逃逸、代码、目录标题语法、表格/删除线/换行、裸链接与邮箱。

本体已通过裁剪 CSR 的 WASM `cargo check` 与 `cargo clippy --locked --offline --target wasm32-unknown-unknown -- -D warnings`。canonical xcss 的公共组件与裁剪宏修复同步后，最终增量 clippy 含 vendor 一并通过，没有警告。

`web/package.json` 的 `test:domain:rust` 与 `test:leptos` 已纳入上述 13 个原生测试；服务端与发布 CI 也使用 locked Cargo 执行它们。

新增 `web/tests/rust-native-domain.mjs` 对真实 WASM 做 6 组/引擎的 Markdown/DOM/安全 URL/复制和图片交互/菜单固定坐标 pointerdown→pointerup/键盘与原生校验验证。`test:browser:leptos` 与后台/发布 CI 浏览器序列已包含该脚本，原后台和公开页面矩阵、断言与失败传播守卫保留。该脚本的 API 使用明确的可丢弃路由 fixture，系统剪贴板使用 writeText 适配器；它不冒充真实后端数据库写入测试。

该新增脚本已在 Chromium 153.0.8010.12 与 Firefox 155.0 首次运行全部通过，每个引擎各 6 组；固定坐标选择路径的 pointerdown/up 均落在 option 2，弹层 top 均为 250→250、window.scrollY 均为 0，未增加重试。End 键测试同时确认末项可见且只滚动列表。

原有 11 套 `leptos-runtime`、`language`、`contract-fixes`、`management`、`admin-layout`、`account-settings`、`public-layout`、`anonymous-comments`、`admin-cleanup`、`public-footer`、`browser-smoke` 也首轮全部通过。原 smoke 依其设计只使用 Chromium，其余保留原两引擎、宽度与语言矩阵；未增加重试或弱化断言。完整 12 套（含新增 domain）的首轮摘要位于实验目录 `evidence/xocs/browser/first-results.json`，具体范围见 `evidence/xocs/final-domain-summary.json`。上述业务迁移保持 Axum API 合同，未修改线上服务或 React 主分支；生产构建与后端集成的优化另见工作区总说明。

此外，同一生产 WASM 已在独立本地 Axum/SQLite 实例完成首轮真实全栈验证，Chromium 与 Firefox 各 6 组全部通过：实际管理员登录、界面创建分类/标签及依赖菜单、typed 文章写入、公开文章的 Rust Markdown/XSS/URL/代码 DOM、typed 编辑后全页重载与持久化、全流程零 pageError。

该 harness 不拦截/替换请求，也不 mock Response；实例和数据库均为可丢弃测试资源。Axum 的所有发行文件逐字节 HTTP、WASM identity/gzip/ETag/HEAD/304，以及 MP4 的 MIME 也已核验。首轮结果位于实验目录 `native-verification/xocs-provider-first/{results.json,browser.json}`。这是代表性真实写入覆盖，不能替代全部业务与所有浏览器的生产环境验证。

最终集成已将 `SOURCE.base_revision` 更新为 `f6ac3fb1b2d0754d12df0e2d3e0cd29ce10344b8`。移除产品 `web/node_modules`、阻止 Node/npm 等命令后，原生 Rust 前端生产构建通过；34 个发行文件中仅 `assets/leptos-build.json` 的 provenance 变化，其余 33 个文件逐字节相同，原 12 套浏览器断言覆盖的执行代码保持一致。证据为实验目录 `xocs-no-node-final-build.json` 与 `xocs-final-provenance-equivalence.json`。浏览器驱动的开发依赖通过独立原工作树加载，没有恢复产品依赖链接。

刷新后的 Axum 使用最终 CLI `server --mode development --rust-only --offline` 构建，冻结 binary/receipt 后再次 verify 通过。后端 67 个 all-targets 测试通过；相同真实 harness 的 SHA 未变，最终 Chromium/Firefox 各 6 组再次首轮全部通过，MP4 GET/HEAD 为 `video/mp4`，所有 HTTP/编码/ETag 合同通过，进程正常退出并释放监听。结果位于 `native-verification/xocs-final/{results.json,browser.json}`，完整汇总为 `xocs-final-integration.json`，冻结产物为 `artifacts/final-native/xocs/`。这是 `development/unbound` 的独立测试实例验收，未作为签名发布、部署或安装证明。

| 最终证据对象 | SHA256 |
| --- | --- |
| WASM | `10a5d7b90a015e68d9aab5215f27e1836dc20c64d2cbc87822feef80a2d09d9c` |
| Axum binary | `3d00ec8622c9fc927a1354fdf17109ee67c2c815c6ef9a7c2189b3da6270cb05` |
| binary receipt | `3a313f66e63269cd22af319e2508d4ca3ff1b4171c448d541700087be4d8467a` |
| 不变的真实浏览器 harness | `aeecc57e70e79eb887ce32ce9f5374ee0b3c5a92073496aa8bc1b68c14748219` |

本轮原生 release 的最终 WASM 为 3,150,552 B，离线逐文件 gzip9 为 938,549 B，实际随包预压缩文件为 941,140 B；四个 JS 文件共 57,121 B，CSS 为 142,014 B。总发行包还包含原有媒体、字体与预压缩副本，不能将包大小相加当作首屏下载量。本轮没有同条件的启动、内存或时延对照测量。

上游依据：[pulldown-cmark 事件 API](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/enum.Event.html)、[HTML renderer](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/html/fn.push_html.html)、[linkify](https://docs.rs/linkify/0.11.0/linkify/)。
