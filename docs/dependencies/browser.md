# 界面与绑定

[项目包](catalog.md#project)、[interface](catalog.md#interface)、[wasm](catalog.md#wasm)与[javascript](catalog.md#javascript)列出当前所有者。

## 浏览器界面

`blog-interface` 是应用使用的组件和浏览器适配门面；`blog-dom` 实现响应式状态、DOM、类型化视图和生命周期；`blog-component-macros` 提供组件和属性的编译期生成。组件、DOM 与宏的说明位于各包源码旁：

- [组件门面](../../src/interface/README.md)。
- [响应式 DOM](../../src/interface/dom/README.md)。
- [组件宏](../../src/interface/macros/README.md)。

视图协议用于 CSR 的构建、更新、挂载与拆卸，支持动态列表、context、Portal 和错误边界。事件监听、观察器和异步工作应随所有者清理；改变列表键会改变子树身份。业务页面和请求状态位于 `src/browser`，共享组件不承担产品业务规则。

## WASM 边界

`browser-api` 和 `javascript-api` 提供类型化浏览器与 JavaScript 接口。`browser-bridge`、入口宏、元数据及 schema 配合 `browser-module` 生成绑定；模块的编码、解码、变换和调试信息由 `wasm` 层其余包提供。绑定生成直接由 `blog-assets` 调用。元数据字段、枚举顺序和描述符操作码通过 `browser-bridge-schema` 的协议快照测试核验；协议变化需更新 `SCHEMA_VERSION`，注释、文档和辅助函数变化不改变协议身份。

应用使用 `wasm32-unknown-unknown`，WASM 必须与同次构建生成的 JS 绑定一起使用。手写 JS 负责加载和启动失败反馈，页面、请求、表单及 Markdown 规则在 Rust 中实现。构建、资源校验与本地预览见[开发说明](../development.md)，真实浏览器验证见[浏览器测试](../browser-testing.md)。

## 样式与字体

样式和设计变量位于 `src/web/styles`。字体以本地 WOFF2 文件随资源分发，CSS 字体族为 `Sarmg Maple`，使用 400 和 700 两个字重；页面不需要访问外部字体服务。字体以 `font-display: swap` 加载，系统字体立即显示；字体下载或解码失败不影响应用挂载。字体声明见 [fonts.css](../../src/web/styles/web-fonts/fonts.css)，原始版本、文件摘要及字符覆盖见[字体来源](../../src/web/styles/web-fonts/provenance.json)。

字体使用 SIL Open Font License，许可文本随源码和资源保留：[OFL](../../src/web/styles/web-fonts/OFL.txt)、[CJK 许可](../../src/web/styles/web-fonts/CJK-LICENSE.txt)、[Latin 许可](../../src/web/styles/web-fonts/NORMAL-LICENSE.txt)。样式来源与许可证分别见 [ORIGIN.json](../../src/web/styles/ORIGIN.json) 和 [LICENSE](../../src/web/styles/LICENSE)。构建器校验实际字体和样式输入，输出路径由生成资源清单决定。
