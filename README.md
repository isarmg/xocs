# blog

Rust 博客与内容服务，包含文章、随笔、匿名评论、媒体与管理后台，支持 Linux x86_64 GNU。服务端、Rust/WASM 界面、样式字体和构建器都在本仓库维护；构建与运行不需要 xcss、xssi 或相邻项目检出。上游代码保留原许可证和来源记录，见[源码管理说明](docs/rust-sources.md)和[独立实现说明](docs/independence.md)。

主服务端位于 `src/server`，应用基础在 `src/application`，浏览器界面在 `src/browser`，组件、DOM 和宏在 `src/interface`。整个 Rust 项目使用根 Cargo workspace 和唯一的根 Cargo.lock，基础源码按职责纳入 `src/foundation/<layer>/<purpose>`，依赖全部指向本仓库路径。准备 Rust 1.99.0、`wasm32-unknown-unknown` target、Linux 编译工具和所需原生库后，在仓库根目录执行：

```sh
python3 tools/rust-sources/manage.py verify --metadata
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 build --locked --offline --bin blog
cargo +1.99.0 build --locked --offline -p blog-setup
```

`tools/assets` 的 `blog-assets` 同时负责前端构建和浏览器 WASM 绑定生成；绑定实现已整合为同一工具中的模块，无需另建或安装 wasm-bindgen CLI，也不依赖 npm 生产构建或 Rust 注册表源码缓存。Rust 工具链、目标标准库、系统 SDK 和非 Rust 原生库仍由构建环境提供。

当前受管源码包含 240 个包：231 个用途基础包和 9 个项目包；根 Cargo.lock 完整图为 239 包。250 仅是优化规模的参考，不设包数硬上限；以正确性、实际资源开销、职责边界与维护复杂度决定合并或拆分。内部维护包不沿用上游版本约束；省略 manifest 版本时，Cargo 将其记为 0.0.0。主产品 `blog` 和维护工具 `blog-setup` 的发行版本单独管理，当前为 1.0.1。原始发布身份、归档与许可证保存在 `compliance/` 和来源文档中。目录、迁移规则与已做优化见[用途架构与定制](docs/maintained-design.md)，逐职责合并分析见[依赖分析与深度定制](docs/dependency-customization.md)。

初始化使用仓库内的 `blog-setup`，密码从标准输入读取；服务运行命令为 `blog --root ROOT`。完整本地启动步骤见[开发说明](docs/development.md)，独立包安装见[部署说明](deploy/linux-x86_64/README.md)。生产保持 HTTPS、Secure Cookie 与同源检查，本地开发仅允许回环 HTTP。

配置、SQLite、媒体、日志和临时快照位于明确的 ROOT。管理员密码使用 Argon2id；会话、CSRF、文章权限、匿名提交幂等与容量限制保留。当前数据库身份是 `blog-db-v1`，不会自动将旧 xocs 数据标记成新结构；升级前应备份并验证明确的数据迁移方案。

Node 26.7.0、锁定 npm 包和 Chromium/Firefox 仅用于浏览器测试，独立于 Rust 源码闭包管理。构建成功、浏览器验收、独立包安装和正式发行分别提供证据；本地改造不代表正式发布或线上部署。历史发行记录保留原始 xocs 身份。
