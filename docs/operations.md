# 运行维护

运行程序使用 `blog --root ROOT`；初始化、离线结构检查和密码重置由仓库内编译的 `blog-setup` 执行。安装与权限见[部署说明](../deploy/linux-x86_64/README.md)。

- 新站点：`blog-setup --root ROOT initialize --username admin`，密码从标准输入读取。
- 停服后检查：`blog-setup --root ROOT check`。
- 停服后重置：`blog-setup --root ROOT admin-reset --username admin`，新密码从标准输入读取并撤销现有管理员会话。

离线工具只以显式 ROOT/config/config.json 决定状态目录，忽略 BLOG_DATA_DIR、BLOG_MEDIA 等运行时环境覆盖，避免维护另一实例。需要外置状态目录时，将真实路径写入该 JSON，再启动和维护同一实例。

配置位于 ROOT/config/config.json，数据库位于 ROOT/state/site.sqlite，媒体默认 ROOT/media，日志 ROOT/logs，验证快照 ROOT/run。配置、状态和媒体的权限及物理路径检查由本地实现保留；不要通过复制版本字段绕过数据库结构验证。

实际检查 `/readyz`、首页、文章和后台登录。HTTP 200、进程存在、构建成功、候选包通过和正式发布是不同层次的证据。生产需要 HTTPS 与同源 Cookie；开发 HTTP 只允许回环地址。

本项目不再调用外部多产品安装/升级框架；当前不提供旧 xocs 数据或任意版本之间的自动迁移。升级前停服、备份数据与配置，先验证目标程序的结构和业务行为，不能宣称保留了原外部升级器的全部事务恢复能力。
