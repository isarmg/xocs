# xocs 文档

从安装站点开始，然后在管理后台发布文章和维护内容。

## 开始使用

1. [安装与首次运行](getting-started.md)：构建或选择程序，初始化站点并打开页面。
2. [发布和管理内容](usage.md)：文章、分类、图片、首页、资源与评论。
3. [配置参考](configuration.md)：JSON、路径、监听地址与可信代理。
4. [Linux 生产部署](../deploy/linux-x86_64/README.md)：服务账号、HTTPS、启动和检查。
5. [运行维护与排障](operations.md)：状态、日志、登录和评论提交问题。

## 开发与参考

- [开发与验证](development.md)：源码构建、自动化检查与发行输入。
- [当前实现参考](current-guide.md)：数据库身份、公开响应与评论重试语义。
- [账号设置](account-settings.md)、[公共支撑](common-support.md)、[1.0.0 发行记录](releases/1.0.0.md)。
- [项目首页](../README.md)、[MIT 许可证](../LICENSE)。第三方前端许可证位于 `web/licenses/`。

当前源码使用 `xocs-db-v4`。公开 v1.0.0 归档早于当前匿名评论、来源识别和持久回执实现；使用这些功能时应构建对应源码，并以该程序的身份与状态校验结果为准。
