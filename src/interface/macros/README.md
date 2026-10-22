# blog-component-macros

本项目维护的组件和视图过程宏，由 `src/interface` 的 `blog-interface` facade 使用。代码、Cargo 包名、生成宏路径均在本仓库维护，不读取或同步外部 xcss 仓库。

原实现为 xcss 中已经裁剪的 Leptos 0.8.19 派生代码。原始 MIT 许可证及版权通知完整保留在 `LICENSE`。上游逐文件摘要、原始 README 中的更早来源与修改记录，以及导入时的完整字节归档均保存在仓库 `compliance/web-origins/xcss-web-macro.*`；这些历史文件保持原字节。当前本地修改及原归档 SHA-256 映射见 `ORIGIN.json`。当前代码摘要由 `blog-assets` 在实际构建时生成，允许正常维护，不把本地改动伪装成未修改上游快照。

当前实现仅维护浏览器 CSR：保留组件与 typed props、children、控制流、context、mount/unmount 与 portal；移除未使用的 SSR、hydration、island、lazy、RSX/template/parser 和独立 slot/memo/slice/Params 宏。来源归档及原始许可证保持原样。实际组件行为测试由 facade 的 `src/interface/tests/csr_components.rs` 运行。
