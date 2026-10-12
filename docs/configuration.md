# 配置参考

默认读取 `/etc/isarmg/xocs/config.json`，隐式默认文件不存在时使用环境和默认值；显式配置缺失、权限或格式错误均报错。所有状态命令共享严格 JSON 配置：`xocs --config /absolute/private/config.json ...`。
优先级为显式 CLI、下表环境变量、文件、默认值。每层都验证字段与类型；修改配置后重启进程。

| JSON 字段 | 默认值 | 用途与取值 | CLI / 环境变量 |
|---|---|---|---|
| `data_dir` | `/var/lib/isarmg/xocs` | 唯一内部私有状态根，固定保存 `site.sqlite`、`logs/` | `--data-dir` / `XOCS_DATA_DIR` |
| `media` | `data_dir/media` | 可独立配置的私有媒体业务目录 | `--media` / `XOCS_MEDIA` |
| `bind` | `127.0.0.1:8081` | IP 和端口 | `--bind` / `XOCS_BIND` |
| `trusted_proxies` | `[]` | 最多 32 个代理 IP/CIDR | `--trusted-proxies` / `XOCS_TRUSTED_PROXIES` |
| `development` | `false` | 仅允许本机回环 HTTP 开发 | `--development` / `XOCS_DEVELOPMENT` |
| `log_level` | `info` | trace/debug/info/warn/error/off | `--log-level` / `XOCS_LOG_LEVEL` |
| `web` | 无，使用内嵌资源 | 开发模式的资源绝对目录 | `--web` / `XOCS_DEV_WEB_DIR` |

`XOCS_CONFIG` 可指定配置文件。代理环境变量使用 JSON 数组，CLI 使用逗号分隔 IP/CIDR。数据库位置不提供独立配置入口。密码不放入 JSON、环境变量或 CLI。

```json
{
  "data_dir": "/var/lib/isarmg/xocs",
  "bind": "127.0.0.1:8081",
  "trusted_proxies": ["127.0.0.1"],
  "development": false,
  "log_level": "info"
}
```

配置文件由运行用户拥有，权限 0600，位于同用户拥有的 0700 私有父目录，使用单硬链接普通文件。所有路径均为绝对路径，不含穿越组件。媒体目录不能覆盖配置或内部状态、数据库和日志。

## HTTPS 与访客来源

生产可绑定回环、内网或通配地址，由部署环境保护后端并提供 HTTPS。`trusted_proxies` 默认空，仅采用真实连接对端，不信任任何访客转发头。显式列出的代理 IP/CIDR 可提供 `X-Forwarded-For`：公共中间件从连接对端开始，由右向左跳过可信代理，在首个不可信地址停止，最多解析 32 跳。头缺失时回退连接对端；可信代理发送非法地址或过长链返回 400。

边缘代理必须覆盖或安全追加真实来源，不能原样转发用户伪造链。只信任实际受控代理的最小网络范围；不要无条件信任所有来源。`X-Real-IP` 和 `Forwarded` 不参与选择。管理员认证、评论、弹幕和文章访问密码均使用同一个公共解析结果，此配置不影响 HTTPS 与 Cookie 模式。

## 配置、离线检查与在线状态

```sh
xocs --config /absolute/private/config.json config validate --json
xocs --config /absolute/private/config.json check --json
xocs --config /absolute/private/config.json status --json
```

`config validate` 是纯配置检查，尚未初始化也可成功，不创建状态。`check` 只读检查已初始化的数据库、schema 和目录，无需服务在线；缺失状态会失败。`status` 只探测实时 readiness 与服务身份，离线会失败。退出码为 0 才表示对应检查成功。

`init` 用于首次创建，`run` 用于前台运行，`admin reset-password --username admin` 从 stdin 读取新密码并撤销会话。`--help` 和 `--version` 不需要配置或状态。

前台与 systemd 均写同一套 xcss UTC 结构日志，状态就绪后写入 `data_dir/logs`，启动前诊断输出 stderr，不读取 RUST_LOG。详见[运维](operations.md)。
