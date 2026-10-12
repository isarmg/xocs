# xocs

## 项目简要介绍

自托管个人博客与管理后台。Rust 服务内嵌 React 前台和后台资源，使用 SQLite 保存站点数据。

## 项目功能

- 文章、分类、全文搜索、Markdown 目录、收藏和友链
- 音乐、旅拍、恋爱笔记及可配置首页栏目
- 匿名评论、回复、祝福板与留言弹幕
- 内容审核、随笔与信笺编辑、站点设置、图片上传和删除及资源管理

## 适用平台

服务端仅支持 Linux AMD64 GNU（`x86_64-unknown-linux-gnu`）。生产环境需要 HTTPS 反向代理；浏览器页面支持手机和桌面布局。

## 如何快速部署

[下载页](https://github.com/isarmg/xocs/releases) 提供 Linux 归档和 `.sha256` 文件。公开 v1.0.0 归档早于当前匿名评论与数据库合同；部署当前源码功能时，先按下节编译，并使用对应程序执行以下步骤。

以专用站点用户运行，使用该用户拥有的私有绝对路径。下面在源码根目录初始化全新的数据目录：

```sh
XOCS_BIN="$PWD/target/x86_64-unknown-linux-gnu/release/xocs"
XOCS_DATA="$HOME/.local/share/xocs"
install -d -m 0700 "$XOCS_DATA"
read -rs -p '管理员密码: ' XOCS_PASSWORD; printf '\n'
printf '%s\n' "$XOCS_PASSWORD" | "$XOCS_BIN" \
  --data-dir "$XOCS_DATA" init --username admin
unset XOCS_PASSWORD
"$XOCS_BIN" --data-dir "$XOCS_DATA" config validate
"$XOCS_BIN" --data-dir "$XOCS_DATA" run \
  --bind 127.0.0.1:8081 --trusted-proxies 127.0.0.1
```

本机 HTTPS 代理须覆盖访客提供的 `X-Real-IP`，填入实际访客 IP；只信任实际代理地址。前台在 `/`，后台在 `/admin`。生产不使用 `--development-http`；普通启动不创建数据库，已有数据只做校验。

## 如何编译部署

在 Linux AMD64 的干净源码目录准备 Rust 1.99.0、Node.js 26.7.0 与 C 编译工具：

```sh
rustup target add --toolchain 1.99.0 x86_64-unknown-linux-gnu
npm ci --prefix web
web/node_modules/.bin/xcss-build-server --config xcss-web-build.json \
  --mode release --no-install --source-revision "$(git rev-parse HEAD)"
```

输出 `target/x86_64-unknown-linux-gnu/release/xocs`。将二进制交给站点用户，按上面步骤初始化和运行；运行时不需要源码或单独的 Web 目录。

[详细文档](docs/README.md)
