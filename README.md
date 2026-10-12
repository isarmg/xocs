# xocs

Rust 博客与内容服务，含文章、匿名评论、媒体与管理后台。支持 Linux AMD64 GNU，Web 资源内嵌二进制。

快速部署：解压已验证的发行包，准备配置，执行 `sudo ./install.sh setup /absolute/config.json admin`（密码从 stdin 输入），初始化完成后用 systemd 启动；升级执行 `sudo ./install.sh upgrade`，保留配置和状态。

源码部署：准备 Rust 1.99.0、Node.js 26.7.0，执行 `npm ci --prefix web`，再运行 `web/node_modules/.bin/xcss-build-server --config xcss-web-build.json --mode release --no-install --source-revision "$(git rev-parse HEAD)"`；开发用 `xocs run --development` 前台运行。

初始化、配置、安装步骤及开发说明见[文档](docs/README.md)。
