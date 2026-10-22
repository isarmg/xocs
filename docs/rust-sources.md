# Rust 源码闭包与维护

根 Cargo.toml 为虚拟 workspace，服务端包位于 `src/server`。所有 Rust 包使用唯一根 workspace 和 Cargo.lock，依赖声明使用本仓库内的 `path` 或继承根 workspace 的本地路径。`src/foundation/<layer>/<purpose>` 保存按产品用途维护的基础 Rust 源码，不通过下载缓存、相邻仓库或 registry/Git 源解析。根默认成员覆盖 `blog`、`blog-application` 与 `blog-setup`；`blog-browser` 使用 WASM 目标，集成绑定生成的 `blog-assets` 使用原生目标。

当前维护树由 407 包基线迁移为 243 包：234 个用途基础包与 9 个项目包。根锁完整图为 242 包；仍有 optional 声明的 `cli/terminal-probe` 当前未被选中，保留在受管源码计数中。46 个原始来源已明确退役，397 份原始归档及法律材料仍保留；兼容版本与宏/工具实现的合并不会删除来源。当前职责、已做定制与证据边界见[用途架构与定制](maintained-design.md)，精确映射见[自动生成的用途架构](RUST_SOURCE_ARCHITECTURE.md)。

包数量只作为观察指标，250 是约数，不设置强制上限。全部受管源码和每个目标的完整 metadata 图仍被核验，包含正常依赖、构建脚本、派生宏和测试；入口未选中或仅 optional 的维护源码也保留计数。按实际实现收益决定合并或拆分，原始归档不冒充活动源码包。

当前内部包与上游发布版本分开管理。维护包通常省略 manifest 版本，由 Cargo 记为 0.0.0；依赖声明不使用上游版本范围。主产品与维护工具的 1.0.1 是本项目发行身份。原始上游版本保留在 `compliance/` 和来源文档，用于追溯、法律材料和原始身份审计，不作为维护代码的版本约束。

日常检查在仓库根目录执行：

```sh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
```

来源管理器核验受管文件和来源记录，并在临时空 `CARGO_HOME` 下读取根原生/WASM metadata。独立边界检查另行拒绝 Git/registry 声明、外部路径、符号链接、嵌套 workspace/lock/Git 目录及 metadata 中任何非空 `source`。这两项核验依赖源码与 Cargo 解析结果；它们不证明全部平台、原生库、浏览器或业务行为正确。

本地修改、裁剪、合并和包名变更必须能从当前维护树映射回真实原始来源。原始包名、版本、归档 SHA-256、许可证和版权保留原身份；当前源码摘要单独记录。维护者审查源码差异后显式接纳：

```sh
python3 tools/rust-sources/manage.py refresh
python3 tools/rust-sources/manage.py verify --metadata
```

目录或包身份变化须先更新 `workspaces.json` 用途映射，并向 `compliance/provenance/source-migrations.json` 追加完整消费/生成事件；退役必须明确说明。管理器完整重放历史，拒绝静默丢包、改写原始身份或重写旧事件。旧补丁路径保留历史值，不因目录搬迁改写日志。对仍保留实现的既往退役来源，须显式声明 `restored_origins`、重核原档并恢复当前血缘；本次组件宏保留的 literal helper 已这样恢复，不重写旧事件。具体字段见[维护规则](RUST_SOURCE_MAINTENANCE.md)。

`refresh` 接纳当前本地变更，不表示取得上游支持或修复已知漏洞，也不能用它改写原始来源字节来掩盖修改。来源变化和行为变化都需要相应代码审查与验证。

仓库中的 `.cargo` 配置和正式命令默认离线。首次准备 Rust 工具链、目标标准库、C/C++ 编译器、系统头文件或其他非 Rust 库，仍属于构建环境工作。Node/npm、浏览器执行文件及其系统依赖仅用于独立的[浏览器测试](browser-testing.md)，不属于 Rust 源码闭包。

安全审计使用当前来源记录恢复的原始 registry 包身份：

```sh
python3 tools/rust-sources/audit.py
```

此工具生成临时审计锁并调用独立安装的固定版本 `cargo-audit`，保留 `--deny yanked`；联网更新 advisory 数据和审计工具安装独立于离线产品构建。根 Cargo.lock 的本地路径包不能代替原始名称/版本匹配。原始身份有 advisory 时，本地改动不能自动视为已修复；需评估对应代码与修复证据。未成功取得当前 advisory/yank 信息，不能报告为新的安全审计通过。
