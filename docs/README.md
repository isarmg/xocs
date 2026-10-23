# Blog 文档

服务端和构建工具支持 AMD64 Linux GNU/glibc，浏览器界面使用 Rust/WASM。所有日期、时间输入、显示和日志日界统一使用 UTC。

## 安装与使用

| 文档 | 内容 |
| --- | --- |
| [首次运行](getting-started.md) | 获取程序、初始化实例、打开站点 |
| [使用指南](usage.md) | 发布内容、管理评论和修改管理员账号 |
| [配置参考](configuration.md) | 配置字段、环境变量、实例目录与权限 |
| [生产部署](../deploy/linux-x86_64/README.md) | 安装包校验、systemd、HTTPS 和离线管理 |
| [运行与排障](operations.md) | 就绪检查、日志、备份、登录和提交问题 |
| [数据与请求约定](contracts.md) | 数据库结构、权限、幂等与请求容量 |

## 开发与依赖

| 文档 | 内容 |
| --- | --- |
| [架构](architecture.md) | 入口职责、Rust 业务层与支持范围 |
| [开发说明](development.md) | 工具链、离线构建、开发服务、测试和打包 |
| [浏览器测试](browser-testing.md) | Node/Playwright 环境、测试入口与覆盖范围 |
| [源码布局与校验](source-layout.md) | 本地依赖解析、源码摘要、来源和许可材料 |
| [依赖文档](dependencies/README.md) | 按职责组织的说明、当前包目录、本地来源资料与 API 文档 |
| [Cargo 依赖关系](RUST_SOURCE_ARCHITECTURE.md) | 从当前 manifest 生成的构建入口和直接消费者 |
| [第三方源码清单](THIRD_PARTY_RUST_SOURCES.md) | 当前实现与原始名称、版本、许可证的映射 |

项目入口见[仓库首页](../README.md)，许可证见 [LICENSE](../LICENSE)。版本发布说明由 [GitHub Releases](https://github.com/isarmg/xocs/releases) 提供。
