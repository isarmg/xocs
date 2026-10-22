# blog 文档

所有日期、时间输入、显示与日志日界统一使用 UTC，页面明确标注 UTC；一天固定为 24 小时，不随服务器或浏览器时区、夏令时改变。无时区的 SQLite 时间及日期输入也按 UTC 解释。

从安装站点开始，然后在管理后台发布文章和维护内容。

## 开始使用

1. [安装与首次运行](getting-started.md)：构建或选择程序，初始化站点并打开页面。
2. [发布和管理内容](usage.md)：文章、分类、图片、首页、资源与评论。
3. [配置参考](configuration.md)：JSON、路径、监听地址与实例路径。
4. [Linux 生产部署](../deploy/linux-x86_64/README.md)：服务账号、HTTPS、启动和检查。
5. [运行维护与排障](operations.md)：状态、日志、登录和评论提交问题。

## 开发与参考

- [开发与验证](development.md)：源码构建、自动化检查与发行输入。
- [用途架构与定制](maintained-design.md)：当前目录、内部版本、SQLite/CSR 定制与优化边界。
- [依赖分析与深度定制](dependency-customization.md)：按真实消费者定制实现、评估合并代价与验证优化收益。
- [Rust 源码管理](rust-sources.md)、[维护规则](RUST_SOURCE_MAINTENANCE.md)：来源、完整文件收据和显式迁移。
- [生成的用途关系](RUST_SOURCE_ARCHITECTURE.md)、[原始来源清单](THIRD_PARTY_RUST_SOURCES.md)：实际包映射、入口和依赖关系。
- [历史 xocs Web 迁移记录](xcss-web-migration.md)：改名前的实现与当时验证，当前结构见独立实现说明。
- [当前实现参考](current-guide.md)：数据库身份、公开响应与评论重试语义。
- [账号设置](account-settings.md)、[公共支撑](common-support.md)。
- [项目首页](../README.md)、[MIT 许可证](../LICENSE)。第三方 Rust 许可材料位于 `compliance/licenses/`，整合组件和字体另保留其原始许可证与 `compliance/` 来源记录。

当前源码使用 `blog-db-v1`。程序会校验实际数据结构；不匹配时只读失败，不自动迁移。

当前安装、运行和维护合同见[部署说明](../deploy/linux-x86_64/README.md)与[当前实现参考](current-guide.md)；[1.0.0 发布说明](releases/1.0.0.md)保留改名前的历史合同。

- [独立实现与项目改名](independence.md)：本地源码边界、来源许可证、构建验证与数据身份。
