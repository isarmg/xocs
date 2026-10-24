# blog-assets

`blog-assets` 编译浏览器应用、生成 WASM 绑定并打包静态资源。工具支持 AMD64 Linux GNU/glibc；输入为仓库内的 Rust、JavaScript、CSS、字体和媒体文件。

## 常用命令

在仓库根目录执行：

```sh
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- test --product-root . --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- serve --product-root . --debug --bind 127.0.0.1:5173
cargo +1.99.0 run --locked --offline -p blog-assets -- server --product-root . --mode candidate --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- source-stamp --product-root .
```

准备 Rust 1.99.0、`wasm32-unknown-unknown` target 和原生 C 编译环境。根 workspace 与 `Cargo.lock` 解析全部本地 Rust 依赖；生产资源构建直接使用 Rust 工具，不运行 npm。Node 26.7.0 用于构建器的生成代码测试，Node/Playwright 用于浏览器测试。

JS 编译由 `blog-assets-compiler` 提供，CSS 编译由 `blog-assets-style-compiler` 提供。浏览器绑定生成在构建器内部运行，与本地 WASM 宏共享绑定协议。浏览器正式构建使用根 `web-release` profile。实现职责和参考资料见[依赖文档](../../../docs/dependencies/README.md)。

## 配置与输出

| 配置 | 用途 |
| --- | --- |
| `src/web/rust-web.toml` | Web 输入、输出、资源预算与构建设置 |
| `blog-assets.json` | 服务端和前端的联合构建入口 |
| `BLOG_WASM_TARGET_DIR` | WASM 编译目录，默认 `target/blog-browser` |
| `BLOG_WEB_DIST` | 静态资源输出目录，默认 `src/web/dist` |
| `BLOG_FRONTEND_METADATA_FILE` | 额外输出前端构建收据 |
| `BLOG_BUILD_METADATA_DIR` | 联合构建时收集原生程序的构建身份与资源记录 |

绑定默认生成到 `src/web/pkg`。资源包含 WASM、JS、CSS、字体、许可文本及预压缩文件。`src/web/pkg/blog-assets.json` 与 `src/web/dist/assets/blog-assets.json` 记录实际构建输入、依赖图和输出摘要。

`BLOG_FRONTEND_METADATA_FILE` 必须指定一个尚不存在的绝对文件路径，并位于 `src/web/pkg`、资源输出目录之外；父目录必须已存在且不经过符号链接。构建器先完整写入并同步临时收据，再发布输出与新收据。收据名称冲突或输出发布失败时保留原有输出与已有收据。

## 构建模式与校验

`server --mode development` 使用未绑定提交的开发身份；`candidate` 绑定真实 HEAD 并允许工作树修改；`release` 要求干净工作树。候选和正式构建均使用 release 优化。正式打包还要求与版本、提交匹配的 annotated tag，见[开发说明](../../../docs/development.md)。

构建器编译时自动绑定自身全部源码，并检查当前源码、依赖文档和来源材料的内容摘要，拒绝过期构建器或构建期间的输入变化。`source-stamp` 包含受 Git 管理及未被忽略的新源码文件，按路径记录 SHA-256 和执行权限；构建输出与运行数据不参与。来源校验见[源码布局与校验](../../../docs/source-layout.md)。

同一产品的 Web 构建和联合原生构建共用 `target/.blog-assets/build.lock`，所有权覆盖构建、发布及收据导出。已有构建持锁时，新构建明确返回占用错误；锁文件不随产物目录替换，调用方不应删除或修复另一个构建的锁。

外部命令并发读取并限制 stdout、stderr，在超时或错误时清理所属进程组。CLI 收到 SIGINT 或 SIGTERM 后停止新任务、回收正在执行的命令并关闭预览服务，分别以 130 或 143 退出；预览最多等待两秒排空响应，随后关闭剩余连接。Cargo 查询上限为 120 秒，本地 Git 查询为 30 秒，编译与测试动作为 45 分钟；输出仍受各调用方的字节预算限制。

服务端收据 `<binary>.blog-assets.json` 绑定程序字节、嵌入资源和构建身份，可独立验证：

```sh
cargo +1.99.0 run --locked --offline -p blog-assets -- verify \
  --binary /absolute/blog --receipt /absolute/blog.blog-assets.json --dist /absolute/web-dist
```

资源处理保留 JS 语义作用域、CSS 导入层叠顺序与重复导入规则。默认预算为普通文件 512 KiB、WASM 及其 gzip 各 4 MiB、字体各 16 MiB、bundle 共 16 MiB；public 媒体按清单校验，单文件 4 MiB、总量 8 MiB。仓库相对路径使用正斜杠分隔，文件名不允许含反斜杠，防止不同 Linux 文件被写入同一收据键。许可证、复制资源和导入统一使用 Web 根目录内的规范路径。已编写的自定义子目录同样进入源码快照及变更监控；`pkg`、`dist` 和 `node_modules` 是明确的生成目录。工具拒绝越界路径、符号链接、硬链接和特殊文件，预算在累计输出时检查，失败时恢复原输出。

`serve` 提供回环开发服务及 API 代理，目标可使用 `localhost`、IPv4 或 IPv6 回环地址；`preview` 提供已有输出。构建期间发生的输入变化会继续触发下一次构建；失败不会吞掉该变化。静态响应支持 MIME、nosniff、ETag、HEAD、304 与 WASM gzip 协商。开发服务和完整验证步骤见[开发说明](../../../docs/development.md)与[浏览器测试](../../../docs/browser-testing.md)。
