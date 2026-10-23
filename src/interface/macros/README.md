# blog-component-macros

供 `blog-interface` 使用的 Rust 过程宏，将组件函数和类型化属性转换为 CSR 组件入口及 props builder。宏在原生构建宿主运行，生成的组件代码用于浏览器 WASM。

当前实现以语法变换为职责，运行状态与 DOM 生命周期由 [blog-dom](../dom/README.md) 提供。项目使用普通 Rust 视图 builder 构造页面。

构建、示例与回归入口见[开发说明](../../../docs/development.md)及 `src/interface/tests/csr_components.rs`，本地 API 文档入口见[依赖文档](../../../docs/dependencies/README.md)。许可证保存在当前目录，原始来源材料保存在 `compliance/`。
