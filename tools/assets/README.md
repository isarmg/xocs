# blog Rust Web 构建器

本项目直接维护的 `blog-assets` 使用锁定 Cargo、wasm-bindgen、Oxc 和 Lightning CSS 构建 `src/browser`，集成 `src/interface` 与 `web/styles`，生成独立 WASM、gzip、CSS、字体、许可、来源记录及可嵌入静态资源。生产构建不执行 Node、Python 或外部 xcss/xssi 下载脚本。

```sh
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- test --product-root . --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- server --product-root . --mode candidate --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- source-stamp --product-root .
```

需要 Rust 1.99.0、wasm32-unknown-unknown 及 Linux 原生编译环境；根 workspace/lock 和 `src/foundation` 提供完整本地 Rust 源码图。浏览器绑定生成器直接编译为 `blog-assets` 内部模块，与 WASM 宏共享本项目的绑定协议，不需要独立 CLI 或注册表源码缓存。JS/CSS 的 AST、语义分析和压缩实现归入本地 `blog-assets-compiler`。WASM 正式构建使用根 `web-release` profile。

`BLOG_WASM_TARGET_DIR` 指定 WASM 构建输出（默认 target/blog-browser），`BLOG_WEB_DIST` 覆盖资源输出；配置为 `web/rust-web.toml` 和根 `blog-assets.json`。工具链、目标标准库和非 Rust 原生依赖由构建环境提供。来源核验见[源码管理](../../docs/rust-sources.md)。

`ORIGIN.json` 标明本地维护身份和修改记录，并绑定 `compliance/web-origins` 中保持原字节的原始 SOURCE 收据与完整导入快照。保留原始许可证；不要求维护代码与旧 xcss 快照相同。构建前后比较当前完整输入摘要，并把构建器编译时嵌入的输入摘要与磁盘源码比较，拒绝过期工具和构建期间漂移。`check-sources` 单独检查本地来源。Node 工具链测试还逐项复核原归档里的所有文件及原 SHA-256。

前端元数据为 `web/pkg/blog-assets.json`、`web/dist/assets/blog-assets.json`；设置 `BLOG_FRONTEND_METADATA_FILE` 可创建实际 runtime receipt，记录模块、输出、WASM Cargo 正常运行依赖图与来源摘要，无需外部构建工具目录。CSR 图限制本地接口、DOM 与组件宏的路径和角色，拒绝服务端/hydration 功能进入浏览器图。

`server --mode development` 使用 source_revision=unbound；`candidate` 使用真实 HEAD 并允许 dirty 源码，`release` 要求 clean checkout。后两者使用 release 优化。构建身份记录 build_mode、source_dirty、source_tree_sha256，通过 BLOG_BUILD_MODE、BLOG_SOURCE_DIRTY、BLOG_SOURCE_TREE_SHA256 与 BLOG_SOURCE_REVISION 传入原生构建；BLOG_BUILD_METADATA_DIR 收集原生的资源/身份记录。

`source-stamp` 对 Git tracked 和非 ignored 的 untracked authored 文件生成按路径排序的 `{sha256,executable}` 映射，紧凑 JSON 编码后计算 SHA-256，包含删除和权限变化，排除 target/dist/pkg/node_modules/运行数据及临时生成目录。server 构建完成必须与开始时相同。最终 `<binary>.blog-assets.json` 绑定实际 binary、所有嵌入资源、身份和源码摘要；`verify --binary PATH --receipt PATH --dist DIR` 可独立核验，不运行候选程序。

JS 资源引用使用语义作用域分析，局部同名 `URL` / `require` 保持普通函数语义。`?url` 导入和重导出通过独立的内容寻址 ESM 提供 URL 值，保留默认、别名及字符串导出名；生成的 URL 求值不受消费模块局部变量影响。CSS `@import` 按每次出现的位置展开并缓存编译结果，保留重复导入的层叠顺序；ESM 对同一 CSS 的副作用导入仅执行一次。CSS 展开在拼接时执行单文件预算，循环导入仍拒绝。

默认预算保持：普通单文件512 KiB，WASM/其gzip4 MiB，字体16 MiB，bundle16 MiB。public媒体使用逐文件真实SHA清单，媒体单文件4 MiB/总8 MiB，未登记文件或变更失败。拒绝输入/输出符号链接、硬链接、特殊文件与越界路径；生成目录以完整来源标识后才可替换，失败恢复之前输出。

`serve --debug` 提供loopback开发服务及API代理，`preview`只读提供已有输出。静态服务检查MIME、nosniff、ETag/HEAD/304和WASM gzip协商。Node/Playwright仅用于浏览器验收。
