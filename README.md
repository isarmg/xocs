# Blog

使用 Rust 构建的博客与内容服务，提供文章、随笔、分类标签、搜索、媒体、匿名评论和管理后台。服务端与构建工具支持 AMD64 Linux GNU/glibc（`x86_64-unknown-linux-gnu`），浏览器应用使用 Rust/WASM。

## 安装与使用

从 [GitHub Releases](https://github.com/isarmg/xocs/releases) 下载 Linux AMD64 安装包及 `SHA256SUMS`，核对后按[部署说明](deploy/linux-x86_64/README.md)安装。初始化和离线检查使用安装包中的 `blog-setup`，服务运行入口为 `blog --root ROOT`。

- [首次运行](docs/getting-started.md)
- [发布内容与管理账号](docs/usage.md)
- [配置参考](docs/configuration.md)
- [运行与排障](docs/operations.md)

站点头像、背景、栏目封面和百宝箱横幅在后台配置，图片和视频可上传至实例媒体目录，前端包不包含默认照片或视频。

配置、SQLite 数据库、媒体、日志和临时快照由明确的实例目录 `ROOT` 管理。生产使用 HTTPS、Secure Cookie 和同源检查；本机开发可使用回环 HTTP。管理员密码使用 Argon2id，会话、CSRF、内容权限及匿名提交幂等由服务端验证。日期、时间和日志日界统一使用 UTC。

## 从源码构建

准备 Rust 1.99.0、`wasm32-unknown-unknown` target、Linux C 编译工具及 Python 3.11 或更新版本，然后在仓库根目录执行：

```sh
python3 src/tools/rust-sources/manage.py verify --metadata
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 build --locked --offline --bin blog
cargo +1.99.0 build --locked --offline -p blog-setup
```

全部 Rust 依赖和 SQLite 源码位于本仓库，共用根 Cargo workspace 与唯一的 `Cargo.lock`。`blog-assets` 完成 JS/CSS 编译、WASM 绑定生成和资源校验。Node 与浏览器仅用于测试。环境准备、本机启动和验证命令见[开发说明](docs/development.md)。

服务端位于 `src/server`，应用基础位于 `src/application`，浏览器应用和组件位于 `src/browser`、`src/interface`，依赖实现按职责放在 `src/foundation`。工具与浏览器资源位于 `src/tools`、`src/operations` 和 `src/web`，测试代码、fixture 与验证脚本集中在 [`test`](test/README.md)。参阅[架构](docs/architecture.md)、[依赖文档](docs/dependencies/README.md)和[完整文档目录](docs/README.md)。

修改代码遵循根目录的 [Rust 编码规范](AGENTS.md)。

项目使用 [MIT 许可证](LICENSE)。依赖许可证与原始来源材料保存在 `compliance/`，对应关系见[第三方源码清单](docs/THIRD_PARTY_RUST_SOURCES.md)。
