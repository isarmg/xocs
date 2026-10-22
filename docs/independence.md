# 独立实现与项目改名

项目的程序、Cargo 包和源码目录改为 blog。服务端、浏览器 UI、静态资源、构建器与 setup 工具均由本仓库维护，日常构建、初始化、测试和打包不需要 xcss、xssi 或相邻工作区源码。

| 原实现 | 当前维护位置 | 使用范围 |
| --- | --- | --- |
| 服务端业务 | src/server | Axum API、数据结构、启动与业务验收 |
| xcss 原生所需模块 | src/application | 配置、认证、SQLite、私有文件、HTTP 合同、生命周期、嵌入资源 |
| xcss-web 与 CSR 核心/宏 | src/interface | 本地 blog-interface 组件、浏览器状态和视图构建 |
| Web 业务实现 | src/browser | 博客公开页面、管理员页面、评论和 Markdown |
| CSS、字体与启动样式 | web/styles | 本项目实际使用的静态资源 |
| Rust 资产工具 | tools/assets | WASM 编译与绑定生成、资源校验、开发服务和服务端嵌入校验 |
| 用途基础源码 | src/foundation/&lt;layer&gt;/&lt;purpose&gt; | 本项目维护的协议、存储、异步、编码、密码与构建实现 |
| 本地初始化/维护 CLI | tools/setup | 复用服务端当前结构及凭据校验，避免另一套 SQL |

原始来源和可维护实现分开记录：`compliance/` 保存固定上游来源、原始许可证和摘要，派生映射与修改记录说明本地整合。MIT、Apache-2.0 和字体许可按原文保留；根 LICENSE 继续保留原版权，不因改名改写历史作者。来源中出现 xcss 不代表构建时依赖该仓库，也不能删除来源记录来制造“独立”结论。

所有 Rust 包现统一使用根 workspace 和唯一根 Cargo.lock，基础源码按用途纳入 `src/foundation/<layer>/<purpose>`。当前受管源码 243 个包由 234 个基础包和 9 个项目包组成，根锁完整图为 242 包，数量仅作为观察指标，250 不是硬上限；重命名、合并、拆分与退役由不可改写历史的显式迁移日志记录。内部版本与原始发布身份分开，详见[用途架构与定制](maintained-design.md)。`python3 tools/rust-sources/manage.py verify --metadata` 在临时空 Cargo home 中核验来源及原生/WASM 本地图；`python3 tools/check-independence.py` 独立拒绝外部 Git/registry 输入、越界路径、嵌套 workspace/lock/Git 和非空 Cargo source。默认离线，不要求预装 wasm-bindgen CLI 或注册表源码缓存。详见[源码管理说明](rust-sources.md)。

这些检查证明受检 Rust 源码的闭包边界，不表示全部平台、原生依赖、Node 测试工具已内置，也不表示新的漏洞审计或浏览器验收已通过。改名不能替代实际实现整合、来源保存与行为验证。

根程序使用 BLOG_ 环境变量、blog CLI 名与 blog-db-v1 当前格式身份。旧 xocs 数据和已安装服务不在改名过程中自动转换；正式迁移需要明确的备份、转换和验收流程。本次开发实例应使用新的 ROOT。

历史发行记录与迁移报告保留原 xocs 身份及当时数据。新构建、真实浏览器、独立包安装和发布状态分别验证；只有实际执行过的结果才可作为本次验收证据。
