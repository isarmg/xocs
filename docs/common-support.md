# 本地实现与第三方代码

服务端使用仓库内 `src/application` 的配置、SQLite、管理员认证、HTTP 合同和生命周期实现；浏览器使用本地 `src/interface`；样式字体位于 `web/styles`，构建器位于 `tools/assets`。这些实现由原 xcss 的实际所需模块整合而来。原始来源、固定修订、许可证、原文件摘要及派生映射保存在 `compliance/`，维护代码可以在明确记录下演进。

全部 Rust 依赖已纳入根 workspace/lock 与按用途分层的 `src/foundation` 本地路径图。浏览器绑定生成直接整合在 `blog-assets`，没有独立绑定 CLI。来源管理器保留原始上游身份、归档和许可证，当前受管源码摘要与历史材料分开记录。新改动由本项目负责审查、测试和维护，不能把复制代码描述为另行取得上游安全支持。

使用 `python3 tools/rust-sources/manage.py verify --metadata` 核验来源和空 Cargo home 下的本地图，使用 `python3 tools/check-independence.py` 另行检查源码边界。Rust 工具链、系统原生依赖、Node 浏览器测试工具和浏览器执行环境分别管理。详见[源码管理说明](rust-sources.md)和[独立实现说明](independence.md)。
