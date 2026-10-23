# 依赖参考

全部 Rust 依赖、SQLite C 源码、浏览器组件和绑定生成实现都在本仓库。原生目标为 AMD64 Linux GNU/glibc，浏览器目标为 `wasm32-unknown-unknown`。依赖按产品用途组合，当前包名与原始发布名分开管理。

从下表选择用途，再查[本地包目录](catalog.md)中的 API 入口、相邻文档和来源资料。目录覆盖全部当前源码所有者，按 `src/foundation/<layer>/<purpose>` 分层；包之间的准确依赖见[用途关系](../RUST_SOURCE_ARCHITECTURE.md)。

| 用途 | 阅读入口 | 对应层 |
| --- | --- | --- |
| 系统接口、异步、并发与诊断 | [系统与运行时](runtime.md) | platform、runtime、sync、filesystem、diagnostics |
| HTTP、SQLite 与原生构建 | [网络与存储](network-storage.md) | network、storage、native |
| 浏览器组件、DOM 与 WASM | [界面与绑定](browser.md) | interface、wasm、javascript |
| JS/CSS、Markdown 与命令行 | [资源和命令行](assets-cli.md) | assets、cli |
| 序列化、日期、配置与字节编码 | [数据格式](formats.md) | format、encoding、checksum |
| 集合、内存、数字与静态查找 | [数据结构](data-structures.md) | collections、memory、math、lookup |
| 字符串、Unicode 与区域数据 | [文本与 Unicode](text.md) | text、unicode |
| Rust 语法和派生宏 | [编译期语法](syntax.md) | syntax |
| 密码、熵源与摘要 | [密码学](cryptography.md) | crypto |
| 浏览器验证工具 | [测试依赖](testing.md) | Node、Playwright、axe-core |

## API 文档

Rust 文档注释与 `include_str!` 引用的 Markdown 是当前 API 说明。先按[开发说明](../development.md)准备工具链并构建 Web 资源（服务端构建脚本需要嵌入这些资源），再从仓库根目录离线生成实际消费者的本地 HTML 文档：

```sh
cargo +1.99.0 doc --locked --offline -p blog -p blog-setup -p blog-assets
cargo +1.99.0 doc --locked --offline -p blog-browser --target wasm32-unknown-unknown
```

原生入口为 `target/doc/blog/index.html` 和 `target/doc/blog_assets/index.html`，浏览器入口为 `target/wasm32-unknown-unknown/doc/blog_browser/index.html`；`CARGO_TARGET_DIR` 可改变根目录。不使用 `--no-deps`，这样依赖接口也会生成；各目标使用实际启用的特性组合。

浏览器测试使用的 Node/npm 工具需独立安装，不属于 Rust 离线源码闭包。安装与验证入口见[测试依赖](testing.md)。

## 来源资料

[来源参考索引](origins/README.md)将每个当前来源映射到本地包和本地资料。保留的原始 README、设计和格式说明描述其原始发布版本，帮助理解算法与 API 背景。本项目的支持范围、包名、安装命令和启用特性以上面的主题指南及本地源码为准。没有独立手册的来源链接到实现所有者与文档注释。

[原始来源清单](../THIRD_PARTY_RUST_SOURCES.md)记录许可身份；完整原始归档和许可证保存在 `compliance/`。源码组织、内容摘要及离线核验见[源码布局](../source-layout.md)。

目录由当前所有者生成，增删或移动包后更新并检查：

```sh
python3 tools/rust-sources/docs.py
python3 tools/rust-sources/docs.py --check
```
