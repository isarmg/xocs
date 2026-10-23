# 源码组织与校验

项目使用一个根 Cargo workspace 和一份根 Cargo.lock。项目包位于 `src/application`、`src/server`、`src/browser`、`src/interface`、`tools` 和 `test`，基础 Rust 实现按用途位于 `src/foundation/<layer>/<purpose>`。全部依赖声明使用本仓库的 `path`，或继承根 workspace 中的本地路径。

[架构说明](architecture.md)介绍各层职责，[依赖文档](dependencies/README.md)提供本地参考资料；精确的当前包与来源映射见[源码清单](THIRD_PARTY_RUST_SOURCES.md)和[用途关系](RUST_SOURCE_ARCHITECTURE.md)。

## 输入与身份

| 文件或目录 | 内容 |
| --- | --- |
| `tools/rust-sources/workspaces.json` | 当前包所有者、职责层、用途、来源映射、入口与平台策略 |
| `compliance/rust-sources.json` | 当前源码、manifest、锁文件、来源材料和目标解析图的摘要收据 |
| `compliance/provenance/origins.json` | 原始发布名称、版本、归档及逐文件 SHA-256 |
| `compliance/provenance/` | 原始归档、整合快照与来源身份材料 |
| `compliance/licenses/` | 原始许可证与版权文本 |
| `compliance/provenance/source-migrations.json` | 包身份转换与当前来源归属 |
| `compliance/provenance/local-modifications.json` | 本地源码变化的来源声明 |
| `docs/THIRD_PARTY_RUST_SOURCES.md` | 生成的包与原始来源对应表 |
| `docs/RUST_SOURCE_ARCHITECTURE.md` | 生成的用途层、入口闭包、声明依赖与消费者 |

原始材料和当前源码分别计算摘要。内部基础包通常省略 manifest 版本，Cargo 将其记为 `0.0.0`；原始发布版本用于来源和安全审计，不限制本地 API 或内部版本。产品发行版本由产品包管理。

## 校验当前源码

在根目录执行：

```sh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
```

来源管理器检查原始归档与逐文件摘要、许可证、当前完整源码、manifest、锁文件、依赖和特性声明。`--metadata` 为原生及 WASM 目标使用临时空 `CARGO_HOME` 运行 `cargo metadata --locked --offline`，验证解析结果全部来自仓库内路径，且 Cargo `source=null`。

独立性检查另行检查外部 Git/registry 声明、越界路径、符号链接、嵌套 workspace/lock/Git 目录和完整 metadata 图。入口没有选中的 optional 包也需要登记所有者。真实源码中的 `src/target` 等目录仍属于输入；只排除明确的构建输出目录。

工具生成的入口闭包按 normal、build、proc-macro 和测试用途标注。声明图包含条件和 optional 分支，是受管输入的保守集合。`include_dev: true` 纳入指定测试根的开发依赖，再跟踪其 normal/build 依赖；不递归纳入依赖库自身的测试。具体可执行验证见[开发说明](development.md)和[浏览器验证](browser-testing.md)。

运行依赖使用对应目标的 metadata 图；build 依赖、过程宏及其后续依赖统一切换到 Linux AMD64 宿主图，浏览器 WASM 构建也遵守这一规则。同一包可同时具有运行和编译职责，特性取相关 workspace 图的并集；这仍是保守闭包，不代表最终链接的精确特性集合。

## 接纳文件变动

修改受管源码后，审查实际差异，再更新收据并校验：

```sh
python3 tools/rust-sources/manage.py refresh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
```

`verify` 只检查已有收据；`refresh` 显式接纳当前文件内容，并重新生成源码清单与用途关系。生成文档不手工编辑。原始 origin、归档和许可材料保持原字节；新增来源可追加。

`local-modifications.json` 使用 schema 1 的 `changes` 数组，每项包含 `directory`、非空 `reason`，以及可选的 `file` 或 `files`。新声明使用当前用途目录；已登记项只允许追加。路径必须是安全的仓库相对路径。

## 包与来源字段

`workspaces.json` 的 `owned_sources` 声明项目包，`maintained_sources` 声明基础包。每项包含 `directory`、`layer` 和具体的 `purpose`；基础包另有排序且不重复的 `origins` 数组。合并包可以对应多个来源，同一来源也可以分配给多个当前包。所有 foundation 文件都必须归属当前所有者。

包或目录改名、内部版本变化、合并、拆分、导入和退役使用 `source-migrations.json` 的 schema 1 身份转换：

| 字段 | 约束 |
| --- | --- |
| `baseline` | 首次登记时的完整身份集合 |
| 身份 | `directory`、`name`、`version`、`origins`；项目自身来源可为 `[]` |
| `events[].id` / `reason` | 唯一事件标识和非空原因 |
| `from` / `to` | 精确消费已有身份，再生成完整新身份；不能覆盖未消费的包 |
| `retired_origins` | 当前图完全不再引用的来源，逐项指定 `id` 和 `reason` |
| `restored_origins` | 重新使用已退役实现时，排序列出恢复的原始身份；原档必须存在且校验一致 |

新增来源通过 `from=[]` 的导入事件登记。已退役来源通过显式恢复字段重新加入；恢复集合须与新增血缘一致。管理器重放完整身份图，终态必须与 manifest 和用途映射一致。基线、已有事件和来源声明保持不变，新变化追加登记。

## 平台与构建边界

`workspaces.json` 的 `profiles` 指定原生运行、WASM、宿主工具和测试入口。`platform_guard` 指定共享构建检查 `tools/assets/src/platform_contract.rs` 及各项目包的接线；已有构建脚本先检查平台，再读取产品构建输入。该共享检查同时属于源码收据、构建器摘要和变更监控。

Rust 工具链、标准库、目标组件、C/C++ 编译器和系统构建材料由环境提供。浏览器绑定生成已整合进 `blog-assets`。Node/npm、Playwright、浏览器、字体等非 Rust 输入和审计工具有各自来源与准备步骤；它们不属于 Cargo 本地源码解析图。

## 安全审计

```sh
python3 tools/rust-sources/audit.py
```

审计从当前来源映射恢复活动原始 registry 包名和版本，生成临时锁文件，调用固定版本的 `cargo-audit` 并启用 `--deny yanked`。改名、合并和本地路径依赖都保留原始身份匹配。明确退役且当前无引用的来源保留档案，不作为活动依赖。

审计工具安装、advisory 数据和 yank 信息更新需要独立网络访问。本地定制的漏洞修复需按对应代码核实；来源校验、离线构建和行为测试各自检查不同边界。
