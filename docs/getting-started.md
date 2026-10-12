# 安装并打开第一个站点

目标是在 Linux AMD64 GNU 主机上启动一个全新的 xocs 站点，并能登录 `/admin`。
以下源码命令在仓库根目录执行，使用 Bash。

## 1. 准备程序

准备 Rust 1.99.0、Node.js 26.7.0、C 编译工具及 Linux GNU target：

```bash
rustup target add --toolchain 1.99.0 x86_64-unknown-linux-gnu
npm ci --prefix web
web/node_modules/.bin/xcss-build-server --config xcss-web-build.json \
  --mode release --no-install --source-revision "$(git rev-parse HEAD)"
```

成功后得到 `target/x86_64-unknown-linux-gnu/release/xocs`。前台和后台资源已内嵌，运行主机只需要程序及其系统运行库。
从发行归档部署时，先核对同名 `.sha256`，再使用包中的 `bin/xocs`，按需准备 JSON，通过 stdin 提供管理员密码并执行包内 `sudo ./install.sh setup /absolute/config.json admin`；脚本完成校验和初始化后，再启动 systemd。升级使用同一入口 `sudo ./install.sh upgrade`。

## 2. 初始化自己的测试站点

以将要运行服务的同一个用户执行。下面使用该用户主目录中的私有状态目录：

```bash
XOCS_BIN="$PWD/target/x86_64-unknown-linux-gnu/release/xocs"
XOCS_DATA="$HOME/.local/share/xocs"
install -d -m 0700 "$XOCS_DATA"
"$XOCS_BIN" --data-dir "$XOCS_DATA" config validate
read -rs -p '管理员密码: ' XOCS_PASSWORD; printf '\n'
printf '%s\n' "$XOCS_PASSWORD" | "$XOCS_BIN" \
  --data-dir "$XOCS_DATA" init --username admin
unset XOCS_PASSWORD
"$XOCS_BIN" --data-dir "$XOCS_DATA" check
```

`init` 只创建全新数据库，完成后数据目录包含 `site.sqlite`、媒体及日志目录。
已初始化站点使用 `check` 检查状态，不能再次 `init`。路径须为绝对路径，由运行用户拥有；目录权限为 `0700`，数据库为 `0600`。

## 3. 本机打开并登录

```bash
"$XOCS_BIN" --data-dir "$XOCS_DATA" run \
  --bind 127.0.0.1:8081 --development
```

在这台主机的浏览器打开 `http://127.0.0.1:8081/`，后台为 `http://127.0.0.1:8081/admin`。
使用 `admin` 和刚设置的密码登录。保持终端运行；按 Ctrl+C 停止服务。
`--development` 只用于本机回环测试。对外提供站点时，按 [Linux 部署](../deploy/linux-x86_64/README.md)配置 HTTPS，并去掉此选项。

下一步：[发布第一篇文章](usage.md)。
