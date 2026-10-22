# 启动第一个站点

源码启动见[开发说明](development.md)。前端与服务端都从本仓库构建，初始化工具是本地 `blog-setup`；无需检出或下载 xcss/xssi。

使用独立包时，先核对 SHA256SUMS 和包内完整文件清单，再按[部署说明](../deploy/linux-x86_64/README.md)调用 `install.sh`。首次初始化从标准输入接收自选的唯一管理员密码。安装不等于启用服务，启动方式以实际生成的 blog systemd 单元为准。

通过配置监听地址的 `/readyz` 确认就绪，再打开首页和 `/admin`。生产保持 development=false 与 HTTPS；本地测试可以在回环地址配置 development=true。

现有旧 xocs 的站点数据保留原身份，不会由本次改名自动迁移。不要将旧数据库重新初始化或改写格式字段来绕过检查。
