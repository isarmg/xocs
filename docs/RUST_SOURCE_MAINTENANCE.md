# Rust 源码维护

所有产品和 Rust 构建工具共用根 Cargo workspace 和 Cargo.lock。`src/foundation/` 是本项目维护的第三方实现；原始 `.crate`、精确发布身份、归档 SHA-256、原始逐文件 SHA-256 与原许可文本独立存放于 `compliance/provenance/`、`compliance/licenses/`，不能用当前修改后的源码冒充原始发布包。

`tools/rust-sources/workspaces.json` 声明当前源码所有者和 Linux/WASM 入口。`compliance/rust-sources.json` 是当前源码收据；`docs/THIRD_PARTY_RUST_SOURCES.md` 和 `docs/RUST_SOURCE_ARCHITECTURE.md` 由管理器生成，不手工改写。修改后先执行 `python3 tools/rust-sources/manage.py refresh`，审查源码、收据及文档差异，再执行 `python3 tools/rust-sources/manage.py verify --metadata`。

`refresh` 会显式接纳当前文件变动，但不能改写或删除已登记的 origin、原始归档和许可材料。包或目录改名、内部版本变化、合并、拆分和退役必须有下述明确迁移事件；不能仅修改当前收据让身份消失。新增 origin 允许追加；后续本地补丁写入 `compliance/provenance/local-modifications.json` 的 schema 1 `changes` 列表，每项含 `directory`、可选 `file` 及非空 `reason`。该日志允许在 refresh 时追加，但拒绝改写或删除既有项；原始 origin 记录始终保持不变。原始发布版本不限制本项目的内部 Cargo 版本；内部实现可省略 manifest 的 version，让 Cargo 记为 0.0.0。产品与维护工具可保留自身发行版本。原始来源中的 directory 只描述导入时的位置，迁移后不能用它判断当前源码是否活动。

`verify` 只核对现有收据，不更新哈希。它重读原始归档并验证每个原始文件摘要，再核对当前完整基础包、产品包、manifest、唯一锁文件和静态依赖/特性边。所有声明的依赖，包括目标条件、dev、build 和 optional 依赖，都必须通过本仓库内 path 指向真实包。未启用或不支持平台的依赖不能悄悄保留外部地址；裁剪必须记录其原始发布身份和具体改动。

`verify --metadata` 还为 Linux amd64 和 Web WASM 使用临时空 Cargo home 运行 `cargo metadata --locked --offline`，确认所有已解析包 `source=null` 且所有源码和目标路径都在仓库内。闭包按统一 workspace 特性图标注 normal 运行候选和 build/proc-macro 构建用途，是保守超集，不是精确单入口链接结果，也不能作为删除源码的唯一依据。测试入口以布尔字段 `include_dev: true` 明确纳入所选根包的 dev 依赖，再跟踪其 normal/build 依赖；不递归纳入依赖包自身的测试。`workspace-tests` 覆盖全部项目测试根，生产入口保持不选 dev 边。该清单证明测试构建输入的覆盖，不表示这些测试已执行。

包数量是观察指标，250 是早期给出的约数，不设硬上限。snapshot 继续统计全部当前源码包，包括未选中的 optional 包；metadata 核验保留每个目标完整图及入口闭包，正常、build、proc-macro、测试和宿主工具都纳入清单。合理拆分导致包数增加不应让 refresh/verify 失败；来源、摘要、依赖边界与迁移历史的校验继续强制执行。合并和拆分需说明消费者、实现复杂度、编译粒度、支持面及实际验证；数量下降本身不构成优化收益。

Rust 工具链（rustc、Cargo、标准库、目标组件、rustfmt、Clippy 等）、C/C++ 编译器以及原生系统构建材料属于单独边界。浏览器 WASM 绑定生成属于 `tools/assets` 的 `blog-assets` 内部模块，与资产工具一起从本地 Rust 源码构建，不另行安装 wasm-bindgen CLI。审计数据库、cargo-audit/cyclonedx、Node/Playwright/浏览器和非 Rust Web 资源应明确标注其独立供应来源；不能把“Rust 源码闭包”描述成整套开发及运行环境无外部依赖。

`compliance/provenance/integrations/` 另存 ZXS 精确历史 Git revision 的输入档案与逐文件摘要，以及 Blog 维护树退役前的独立快照。后者明确标注取样时间，不能冒充早先复制瞬间或原始发布字节。当前输出始终由当前源码收据绑定；改动和消费者验证另行登记。

完整内置源码只证明来源和本地可构建性。依赖安全审计仍须按原始上游名字与版本匹配，不能因为 Cargo 已改为 path 或包改了名就漏扫。原许可声明不意味着维护保障、无漏洞或完成线上发布。正式验收仍需原生单元测试、WASM 构建、真实浏览器、候选包完整性、TLS/API/CSRF、持久化、实例锁、密码重置和污染环境隔离。

## 用途映射与显式迁移

`workspaces.json` 的 `owned_sources` 声明项目自身或已有独立来源的当前包；`maintained_sources` 声明当前第三方维护包。每项都必须有 `directory`、职责层 `layer` 和具体用途 `purpose`；第三方包还必须有按原始身份排序且不重复的 `origins` 数组。一个合并包可对应多个来源，一个来源可拆分为多个当前包。基础包允许按 `src/foundation/<layer>/<purpose>` 嵌套组织；所有包必须登记，所有 foundation 文件必须有当前所有者。真实源码中的 `src/target` 仍核验，只排除根 target 和准确包根 target 构建输出。

`compliance/provenance/source-migrations.json` 使用 schema 1，包含 `baseline` 与 `events`。首次启用时，baseline 必须与启用前现有 `rust-sources.json` 的完整包身份一致，包括项目包；管理器不能为重构重新造一份空基线。每个身份包含 directory/name/version/origins，旧收据的单值 origin 转换成一元素数组，项目实现为 []。

每个事件包含唯一 `id`、非空 `reason`、`from` 和 `to` 完整身份数组，以及 `retired_origins`。`from` 必须精确消费当时存在的身份，`to` 不得覆盖未消费的身份。重命名、内部版本调整、合并和拆分都使用相同的消费/生成规则。整个当前图不再引用的原始来源必须在 `retired_origins` 中逐项写 `{id, reason}`；仍被另一个当前包引用的来源不能误标退役。新导入的原始来源通过 from=[] 的专门事件生成，不能借现有来源为未消费的新包虚构血缘。退役来源不能通过普通导入事件复活。确实仍保留其代码或重新采用其实现时，事件可另含按原始身份排序且不重复的 `restored_origins` 数组：每个身份必须已被明确退役，原始 archive 仍存在且重新计算 SHA-256 一致；新增血缘必须恰好等于该恢复集合。事件照常精确消费/生成当前所有者，恢复后的来源从终态退役集合中移除并重新参加审计。未知、未退役、未守恒或原档变化的恢复均拒绝；旧事件保持不变。

管理器每次完整重放 baseline 与全部 events，终态必须与实际 manifest 和用途映射一致。首次 refresh 校验基线，后续 refresh 只允许追加事件，禁止重写基线、修改旧事件或删除迁移历史。原始 origin、归档和法律材料仍单独不可变，本地补丁日志也保留追加约束。追加迁移事件是维护者对变化原因和来源对应关系的明确声明，工具不证明语义等价或代码已经修复漏洞。

管理器生成 `docs/RUST_SOURCE_ARCHITECTURE.md`：按用途层汇总当前包、明确职责、实际入口闭包、声明的跨层边与直接消费者，并指向原始身份。层次与职责来自维护者审查的映射，依赖关系从真实 path 声明生成；不能用上游名字的机械改写代替用途分析。声明图包含条件、optional、dev/build 分支，仍需测试证明受支持行为。

`audit.py` 从当前收据的 origins 数组（兼容旧单值 origin）取所有活动原始发布身份，合并和目录变化不能漏扫。它不以历史 origin.directory 是否仍存在为过滤条件；明确退役且当前无引用的来源保留档案，但不作为当前实现冒充运行依赖。联网 advisory/yank 取得失败仍是审计失败，不能用离线构建通过代替。

历史补丁的 directory 可能已经搬迁、合并或退役。管理器不要求该历史目录或 file/files 仍存在；它要求目录出现在已验证迁移日志的 baseline 或事件身份中（未启用迁移时必须是当前包），并对 file 及 files 数组的每条相对路径执行安全检查。历史记录不改写为新路径，继续遵守追加约束；后续新补丁追加当前用途目录。包级迁移记录证明历史所有者身份，不能代替对补丁内容的代码审查。
