# blog-interface

本项目维护的 Rust/WASM 浏览器组件、主题、HTTP 合同、启动与曲面绘制实现。应用位于 `src/browser`，通过本地 path 直接依赖 `blog-interface`。核心与过程宏分别位于 `vendor/blog-dom`、`vendor/blog-component-macros`，完整实现和 Cargo 图均在本仓库，不需要外部 xcss 仓库或同步工具。

原始 Apache-2.0/MIT 许可证保留在各组件目录。`ORIGIN.json` 记录原始 xcss 来源、精确修订、原逐文件摘要与完整导入快照归档以及本地修改。`compliance/web-origins` 中原始 `SOURCE.json` 与归档保持原字节；构建器记录当前代码摘要并检查构建期间源文件未变化。
