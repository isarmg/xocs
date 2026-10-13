# 当前配置

唯一配置位置是 `ROOT/config/config.json`，文件必须为私有普通文件。Server CLI 只有 `--root`；服务参数通过当前 JSON 与已声明环境变量读取，没有旧 CLI 覆盖、目录别名或未知字段兼容。

| 字段 | 默认 | 约束 |
| --- | --- | --- |
| data_dir | ROOT/state | 物理绝对私有目录；包含 site.sqlite 和实例锁 |
| media | ROOT/media | 物理绝对私有目录，可明确外置 |
| bind | 127.0.0.1:8081 | 明确的监听地址 |
| log_level | info | trace/debug/info/warn/error/off |
| development | false | true 仅允许回环 HTTP |
| web | null | 仅开发模式可选绝对 Web 目录；生产内嵌资源 |

日志固定 ROOT/logs，验证快照固定 ROOT/run。data_dir 与媒体不能覆盖配置、程序或维护区域；不允许符号链接别名。`trusted_proxies` 等旧字段直接拒绝。

账号密码不属于配置。管理员的 Argon2id 哈希、会话及审计位于 site.sqlite；文章密码由业务表独立保存。绑定地址中的 IP 是实际网络配置，不用来识别访问者或创建频率桶。
