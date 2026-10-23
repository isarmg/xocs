# 安装与首次运行

blog 的服务端和构建工具运行于 AMD64 Linux GNU/glibc。浏览器界面使用 WebAssembly；访问者无需安装 Rust 或其他客户端。

## 选择运行方式

| 用途 | 入口 |
| --- | --- |
| 安装生产站点 | 从 [GitHub Releases](https://github.com/isarmg/xocs/releases) 下载 Linux x86_64 安装包，按[部署说明](../deploy/linux-x86_64/README.md)安装 |
| 修改源码或本机试用 | 按[开发说明](development.md)准备工具链、构建和启动 |

生产安装前，通过可信渠道核对 `SHA256SUMS`；安装器还会核验包内完整文件清单。为新站点选择一个独立的绝对路径作为 `ROOT`，将配置放在 `ROOT/config/config.json`。配置字段和目录要求见[配置参考](configuration.md)。

## 初始化并打开页面

安装器使用包内的 `blog-setup` 创建数据库和唯一管理员；源码运行则需手动执行初始化。密码从标准输入读取，不写入命令行或配置文件。初始化只用于尚无数据库的新站点。

安装成功后，按部署说明启用并启动 `blog.service`。源码构建的服务使用 `blog --root ROOT` 启动。服务启动后：

1. 请求监听地址的 `/readyz`，确认返回成功。
2. 打开站点首页，检查页面和图片能否加载。
3. 打开 `/admin`，使用初始化时的管理员账号登录。
4. 按[内容管理说明](usage.md)设置站点信息并发布第一篇文章。

生产站点通过 HTTPS 对外提供服务，并保持 `development=false`。本机试用可以设置 `development=true`，监听地址必须为回环地址，如 `127.0.0.1:18091`。

已有站点的检查、备份和账号恢复见[运行与排障](operations.md)。
