# xocs

Rust 博客与内容服务，含文章、随笔、匿名评论、媒体和管理后台，支持 Linux x86_64 GNU。Web 使用 Rust / xcss-web，资源与 WebAssembly 内嵌 Axum 运行二进制。构建与验收见 [Web 迁移记录](docs/xcss-web-migration.md)。

安装使用独立 [xssi](https://github.com/isarmg/xssi) 仓库提供的当前发行包及 `install.sh`。Server 仅接受 `xocs --root /opt/isarmg/xocs` 运行；初始化、只读检查和离线密码重置由 `libexec/setup` 执行。升级由 `upgrade.sh` 调用独立 xssc，下载校验后自动停服、备份、检查数据、执行受支持的数据升级、切换并启动验收。

程序、配置、数据库、管理员凭据、媒体、日志和维护记录默认位于同一 ROOT。数据库或媒体可明确指定外置目录，日志和临时文件留在 ROOT。唯一管理员及会话持久化于 `state/site.sqlite` 的公共管理表，密码保存 Argon2id 哈希。

服务不识别代理或访问者 IP，不读取转发头用于限流。认证、CSRF、内容权限与幂等仍保留；匿名写入最多 4 个同时执行和 32 个等待，文章密码计算最多 2 个任务，超出容量返回 503。

当前配置和数据库合同不接受旧 CLI、旧路径、旧字段或历史结构，也不自动转换旧数据。详见[部署与维护](deploy/linux-x86_64/README.md)、[配置](docs/configuration.md)和[当前行为](docs/current-guide.md)。


生产前端构建使用仓库内 Rust 工具，无需安装 npm 依赖。准备 Rust 1.99.0、wasm32-unknown-unknown target 和 wasm-bindgen-cli 0.2.129 后，在产品根目录执行：

```sh
cargo +1.99.0 run --quiet --locked --manifest-path tools/web-build/Cargo.toml -- build --product-root .
```

开发监听、Axum 集成和测试边界见 [Rust 原生 Web 构建与测试](docs/rust-native-web.md)。Node/Playwright 仅用于浏览器验收；已退役的 TypeScript/React 对照实现与 Vite 转换依赖可通过 Git 历史查阅。
