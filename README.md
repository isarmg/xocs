# blog

Rust 博客与内容服务，包含文章、随笔、匿名评论、媒体与管理后台。服务端和构建工具只支持 AMD64 Linux（`x86_64-unknown-linux-gnu`、64 位指针、glibc）；界面编译为浏览器 WASM。直接执行 Cargo 也会检查构建宿主和目标。

服务端和浏览器业务逻辑均使用 Rust，手写 JavaScript 仅负责 WASM 加载和无法启动时的错误提示。

所有 Rust 依赖及 SQLite 源码都在本仓库维护，按实际用途定制；整个项目共用根 Cargo workspace 和唯一的 Cargo.lock。准备 Rust 1.99.0、`wasm32-unknown-unknown` target 和 Linux C 编译工具后，在仓库根目录执行：

```sh
python3 tools/rust-sources/manage.py verify --metadata
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 build --locked --offline --bin blog
cargo +1.99.0 build --locked --offline -p blog-setup
```

`blog-assets` 负责 JS/CSS 编译、WASM 绑定生成和资源校验。Rust 构建离线解析本地路径，不需要相邻仓库、独立 wasm-bindgen CLI 或 npm 生产构建；Rust 工具链、目标标准库、C 编译器和系统运行库由环境提供。

服务端位于 `src/server`，应用基础在 `src/application`，浏览器应用在 `src/browser`，组件、DOM 和宏在 `src/interface`，内部依赖在 `src/foundation/<layer>/<purpose>`。职责和支持面见[架构说明](docs/maintained-design.md)，修改依据和测量见[依赖定制](docs/dependency-customization.md)。当前包数和来源映射统一由[源码清单](docs/THIRD_PARTY_RUST_SOURCES.md)生成，原始归档和许可证保存在 `compliance/`。

初始化使用仓库内的 `blog-setup`，密码从标准输入读取；服务运行命令为 `blog --root ROOT`。完整本地启动步骤见[开发说明](docs/development.md)，独立包安装见[部署说明](deploy/linux-x86_64/README.md)。生产保持 HTTPS、Secure Cookie 与同源检查，本地开发仅允许回环 HTTP。

配置、SQLite、媒体、日志和临时快照位于明确的 ROOT。管理员密码使用 Argon2id；会话、CSRF、文章权限、匿名提交幂等与容量限制保留。当前数据库身份是 `blog-db-v1`，不会自动将旧 xocs 数据标记成新结构；升级前应备份并验证明确的数据迁移方案。

Node 26.7.0、锁定 npm 包和 Chromium/Firefox 用于浏览器测试，详见[开发与验证](docs/development.md)。安装、使用和运维入口见[文档目录](docs/README.md)。
