# 资源和命令行

接口索引：[assets](catalog.md#assets)、[cli](catalog.md#cli)。

## 资源编译

`blog-assets` 直接消费两个独立编译器：`assets/compiler` 处理 JavaScript 的解析、AST、语义、生成与压缩；`assets/style-compiler` 处理 CSS 的解析、选择器、目标浏览器转换和压缩。各自的类型与派生宏归属对应管线，修改一条管线不要求另一条重新编译。

CSS 资料与实现相邻：

- [编译接口](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/transforms.md)、[语法转换](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/transpilation.md)、[压缩](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/minification.md)。
- [模块](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/css-modules.md)、[打包](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/bundling.md)。

这些组件资料中的 Node 调用示例属于原始接口背景。本项目的生产入口使用 Rust `blog-assets`，命令、配置、输出目录和资源收据见[构建工具](../../tools/assets/README.md)。构建输入使用内容摘要，每次构建建立新快照；进程内复用解析结果仍必须对应同一源码快照。

## Markdown

`assets/markdown-parser` 组合 CommonMark 事件解析、HTML 转义与裸链接识别。产品的原始 HTML、链接协议、图片和目录策略由 `src/browser/src/markdown.rs` 决定；不能把解析器支持的全部语法直接当成公开内容权限。

## 命令行

`cli/arguments` 提供参数模型、builder、解析和派生接口，`argument-derive` 在编译期生成解析代码；`terminal-format` 与 `terminal-style` 处理帮助文字和终端样式。详细教程已纳入本仓库：

- [解析入口](guides/cli/runtime/README.md)、[Builder](guides/cli/builder/README.md)、[词法层](guides/cli/lexer/README.md)。
- [Builder 入门](guides/cli/runtime/examples/tutorial_builder/01_quick.md)、[派生入门](guides/cli/runtime/examples/tutorial_derive/01_quick.md)。
- [子命令示例](guides/cli/runtime/examples/git.md)、[类型解析示例](guides/cli/runtime/examples/typed-derive/custom.md)。

教程中的通用示例用于理解依赖接口；产品实际命令以 `blog --help`、`blog-setup --help` 和 `blog-assets --help` 为准。
