# blog-interface

Rust/WASM 浏览器组件、主题、HTTP 客户端、应用挂载和曲面绘制。`src/browser` 通过本地路径依赖本包；DOM 核心在 `dom/`，过程宏在 `macros/`。

组件使用类型化 props、响应式状态和普通 Rust DOM builder，支持表单、受控输入、弹层、键盘交互及挂载清理。业务页面、路由和业务数据状态位于 `src/browser`。

- [界面与浏览器依赖](../../docs/dependencies/README.md)
- [架构](../../docs/architecture.md)
- [开发与测试](../../docs/development.md)
- [DOM 核心](dom/README.md)与[组件宏](macros/README.md)

本包许可证位于当前目录，来源材料保存在 `compliance/`。构建器核对当前源码摘要并记录实际构建输入。
