# blog-dom

本项目维护的浏览器 CSR 核心运行时，由 `src/interface` 的 `blog-interface` facade 使用。代码、Cargo 包名、生成宏路径均在本仓库维护，不读取或同步外部 xcss 仓库。

原实现为 xcss 中已经裁剪的 Leptos 0.8.19 派生代码。原始 MIT 许可证及版权通知完整保留在 `LICENSE`。上游逐文件摘要、原始 README 中的更早来源与修改记录，以及导入时的完整字节归档均保存在仓库 `compliance/web-origins/xcss-web-core.*`；这些历史文件保持原字节。当前本地修改及原归档 SHA-256 映射见 `ORIGIN.json`。当前代码摘要由 `blog-assets` 在实际构建时生成，允许正常维护，不把本地改动伪装成未修改上游快照。

当前实现仅维护浏览器 CSR：保留组件与 typed props、children、控制流、context、mount/unmount 与 portal；移除未使用的 SSR、hydration、island、lazy、RSX/template/parser 和独立 slot/memo/slice/Params 宏。来源归档及原始许可证保持原样。实际组件行为测试由 facade 的 `src/interface/tests/csr_components.rs` 运行。

当前视图模型由 `Render::build/rebuild` 创建和更新浏览器 DOM；`DomView` 只约束拥有输入数据的转换和空视图行为；属性组合由独立的 `AddAnyAttr` 能力提供，必须在视图仍有具体类型时完成。类型擦除只保留这些 CSR 渲染操作，不再携带 HTML 字符串输出、异步流、hydration cursor、模板缓存或服务器分支协议。`For` 的键只要求相等性和哈希，列表输入无需为 islands 序列化保留可空状态。事件监听器直接绑定到元素并在视图清理时移除。

响应式 effect 在浏览器及测试执行器中始终创建；不再保留禁用 effects 时的空外壳或重复的 isomorphic 入口。线程安全执行使用 `new_sync`。原模板拼接 `constant_text` 和 HTML 字符串转义 `html_escaping` 已随服务端渲染退役；CSR 文本和属性直接传给 DOM API，其历史来源、归档和许可证仍保留在来源账本中。

生产博客业务由 `src/browser/src` 和 `src/interface/src` 的 Rust/WASM 实现。`web/src/main.js` 仅导入资源与生成的 WASM glue；`web/runtime/startup.js` 负责 WASM 自身加载失败时仍能显示的错误与手动重载界面。浏览器回归覆盖真实 Chromium/Firefox 中的组件、异步状态、表单与传输取消行为。
