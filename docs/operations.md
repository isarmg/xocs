# 运行维护与排障

生产安装、HTTPS 和服务用户要求见[Linux 部署](../deploy/linux-x86_64/README.md)。下面使用实际程序路径和与服务相同的配置执行检查。

## 检查站点

```sh
xocs --config /absolute/private/config.json config validate --json
xocs --config /absolute/private/config.json check --json
xocs --config /absolute/private/config.json status --json
xocs release-identity
```

配置校验无需数据库，离线 check 校验已初始化状态；在线 status 应报告当前服务已就绪。再从公开 HTTPS 地址检查首页、文章和后台登录。
程序存在或端口打开仅说明进程层状态，不能代替 readiness 与页面检查。

前台与 systemd 使用相同 xcss UTC JSON 日志，状态就绪后写入 `data_dir/logs`，启动前诊断输出 stderr。`log_level` 默认 info，通过 JSON、`XOCS_LOG_LEVEL` 或 `--log-level` 配置；默认每文件 8 MiB、保留 4 份归档。查看最近错误时保留时间、错误码、请求 ID 和程序身份；分享前移除账号、Cookie、私人路径和内容。停止服务使用 SIGTERM 或前台 Ctrl+C，等待进程退出后再维护状态文件。

## 按症状排查

| 症状 | 先检查 | 正常结果和下一步 |
|---|---|---|
| `init` 报状态已存在 | 数据目录和所选程序 | 已初始化站点改用 `check`；全新站点选择新的空目录 |
| 配置校验失败 | 错误中的字段、路径和权限 | JSON 类型正确，目录属运行用户且为 `0700`；按[配置参考](configuration.md)修正 |
| `status` 失败 | 与服务相同的配置、进程日志、监听地址 | 服务身份为 xocs 且 readiness 成功；修正日志中首个错误后重新检查 |
| 前台正常，后台登录循环 | HTTPS、Cookie、Host/Origin、主机时间 | 浏览器经同一 HTTPS 域名访问；生产保持 `development=false` |
| 代理后评论失败 | 可信代理 IP/CIDR 与 `X-Forwarded-For` 链 | 只信任受控代理，使用合法链；按部署说明配置后再提交 |
| 评论显示“尚未确认” | 原标签页的原稿和提交状态 | 点击“确认原提交”，保持原身份和内容，确认后再写新评论 |
| 评论返回 429 | 最近 15 分钟该来源 IP 的提交量 | 三类评论共用每来源 10 条额度；等待窗口恢复 |
| 图片或页面资源异常 | 图片上传结果、程序资源清单、代理路由 | 上传地址可读，页面资源来自同一程序；核对是否使用了预期构建 |

数据库格式不符时保留原文件并停止排查写入，用 `release-identity` 和校验错误核对程序与数据。当前格式为 `xocs-db-v4`，不能靠修改元数据或删除锁文件使不同结构共用同一目录。

## 更新程序与重置密码

生产首次使用发行包中的 `sudo ./install.sh setup /absolute/config.json admin`，密码走 stdin。升级使用 `sudo ./install.sh upgrade`，保留配置、`data_dir` 与独立媒体目录，不初始化或迁移数据；只有原本运行的服务才会恢复运行。升级后验证状态和实际功能。

忘记管理员密码时先停止服务，再通过 stdin 向 `xocs admin reset-password --username admin` 输入新密码；不要放入参数、配置或环境。新密码生效时全部现有管理员会话被撤销。重启服务后使用新密码登录。操作示例见部署手册。

## 安全事件

先限制受影响入口，保存脱敏日志和程序身份，再根据暴露范围更换管理员密码及主机或代理凭据。向维护者提供症状、复现条件、版本和安全错误信息；生产数据库、媒体和私有配置保留在受控主机。
