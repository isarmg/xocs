# 配置参考

服务通过 `blog --root ROOT` 选择实例。`ROOT` 必须是规范化的绝对目录，配置固定为 `ROOT/config/config.json`。配置采用严格 JSON；未知字段或类型错误会使启动失败。

## 配置字段

| 字段 | 默认值 | 说明 |
| --- | --- | --- |
| `data_dir` | `ROOT/state` | 数据库与实例锁所在的绝对私有目录 |
| `media` | `null`，使用 `ROOT/media` | 上传媒体的绝对私有目录 |
| `bind` | `127.0.0.1:8081` | IP 地址和端口，IPv6 示例为 `[::1]:8081` |
| `log_level` | `info` | `trace`、`debug`、`info`、`warn`、`error` 或 `off` |
| `development` | `false` | 设为 `true` 时使用本机开发 HTTP，且只允许监听回环地址 |
| `web` | `null` | 外部 Web 资源目录；仅允许未绑定发行源码身份的 debug 构建在开发模式下使用 |

新建本机开发实例的最小配置：

```json
{
  "bind": "127.0.0.1:18091",
  "development": true
}
```

生产保持 `development=false`，由 HTTPS 反向代理对外提供服务。发行程序使用内嵌 Web 资源。开发时指定 `web` 必须使用绝对路径，且目录中须有构建后的 `index.html`。

## 目录与权限

| 位置 | 内容 |
| --- | --- |
| `ROOT/config/config.json` | 实例配置 |
| `data_dir/site.sqlite` | 业务数据、管理员密码哈希、会话和安全审计 |
| `media` 或 `ROOT/media` | 上传图片 |
| `ROOT/logs` | 结构化日志 |
| `ROOT/run` | 数据库检查的临时快照 |

配置必须为受管普通文件。生产安装器设置配置为 `root:isarmg-blog` 的 `0640`，配置目录为 `0750`；源码开发使用当前用户所有的 `0600` 文件和 `0700` 配置目录。状态、媒体、日志及临时目录由运行账号所有，权限为 `0700`。

数据和媒体路径必须为真实绝对路径，不能通过符号链接跳转，也不能覆盖配置、程序或实例管理区域。运行程序支持明确配置外置数据和媒体目录；随包安装器及其 systemd 单元使用 `ROOT` 内的布局。外置布局需同时配置正确的属主、权限和 systemd 可写目录，详见[部署说明](../deploy/linux-x86_64/README.md)。

管理员密码通过初始化、账号设置或离线重置管理，不放入 JSON。文章访问密码由业务数据单独保存。

## 服务环境变量

服务按“默认值 → JSON → 环境变量”的顺序解析配置：

| 环境变量 | 对应字段 |
| --- | --- |
| `BLOG_DATA_DIR` | `data_dir` |
| `BLOG_MEDIA` | `media` |
| `BLOG_BIND` | `bind` |
| `BLOG_LOG_LEVEL` | `log_level` |
| `BLOG_DEVELOPMENT` | `development`，只接受 `true` 或 `false` |
| `BLOG_DEV_WEB_DIR` | `web` |

`blog-setup` 的初始化、检查和密码重置只使用所选 `ROOT` 的 JSON，不读取上述运行时覆盖。将实例的数据与媒体位置写入 JSON，使服务和离线操作始终指向同一实例。
