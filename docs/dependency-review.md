# 内建依赖实现审查

本次以博客的 Linux 服务、浏览器 CSR 和资源编译器为使用边界，检查完整本地依赖图、既有定制记录和主要消费者，并对具体问题修改实现。目标是行为正确、资源有界和可维护；不保留上游兼容性作为重构前提，也不以包数作为验收指标。未对每个基础包做逐行证明，未声称整个项目已经达到全局最优。

## 实施结果

| 范围 | 确认的问题 | 实现及回归 |
| --- | --- | --- |
| 普通 Base64 | 两份常规实现重复；流式编码短写可能误报失败，零进展可能循环，finish 重试可能重复末尾字节 | HTTP 与 WASM 消费者统一到 application-base-encoding；只移动剩余缓冲，正确转移尾部所有权，专门实现 write_all；RFC4648、故障注入和 SIMD 契约测试 |
| SQLite | `$0` 越界、数字前缀错绑、缺参后漏绑；非法 collation 名称泄漏回调 | 完整正整数解析；保留 NULL 缺参行为并继续绑定；验证名称后才转移回调所有权；真实连接及回调引用计数回归 |
| 行派生宏 | 合法的 `__row`、`__default` 字段捕获生成局部变量 | mixed-site 局部标识符和完整 Default 路径；实际 FromRow 消费者 |
| HTTP body | 单次轮询无界处理 ready 帧，空数据和 trailers 可阻止超时重新获调度 | 每次最多 32 帧并自唤醒；验证取消、数据/trailers 完整性与小请求单次完成 |
| HTTP Range | 空文件产生非法部分响应；过长后缀被拒绝；HEAD 错误套用 Range；重叠检测平方复杂度 | 空文件拒绝、后缀截为完整文件、HEAD 返回完整元数据；仅排序边界副本，保留请求顺序；参考算法和 ServeFile 消费回归 |
| 响应式 DOM | 异常后 owner 未恢复；清理回调重入 TLS 借用；嵌套 untracked 状态丢失；历史子节点不断累积 | RAII 完整状态恢复；借用外析构；新 owner 继承暂停状态；摊销剔除失效弱引用；实际 memo、cleanup 与 10,000 次节点增删回归 |
| 浏览器请求 | 拒绝响应头或丢弃 future 后，未读取响应仍可继续下载 | 请求生命周期持有 AbortOnDrop；真实持续 HTTP 响应在页面仍打开时被取消，Chromium/Firefox 验证 |
| JavaScript 资产链接 | 把局部 URL/require 当全局；手写导出破坏 default/字符串导出名并受 URL 遮蔽；压缩器模板字面量输出不能再次扫描 | 使用语义作用域；生成独立 URL ESM 模块，仅改写 source 字面量；识别静态 cooked 模板值，保留拒绝计算导入的规则 |
| CSS | 全局去重破坏重复 @import 的层叠位置 | 按每次引用展开，独立保留 ESM 的一次执行语义；缓存总字节受预算限制，防止长链重复文本累积 |
| 验证闭包 | 源码清单入口跳过所有 dev 边，保留的基础库测试不自动执行 | 测试入口显式 include_dev，仅遍历选定根的开发依赖；增加 maintained 测试脚本并接入 CI/release |

三个不再需要的实现已退役：`http-base-encoding`、`integer-parser`、`scheduled-stream`。当前受管源码为 240 包（231 基础包、9 项目包），根锁 239 包，Linux/WASM metadata 分别 239/233 包。397 份原始归档和许可材料保留，348 个来源仍活动，49 个明确退役。变更原因及退役身份采用追加记录，不改写历史来源。

Base64 合并前对四种标准/URL/无填充引擎、519 种输入长度及逐位置全部字节值做差异验证。旧版本的错误类型表现与新版本不完全相同，迁移的消费者只使用编码接口。恒时密码 Base64 与编译器 SIMD 编码仍有独立的调用契约和职责，继续保留。

## 验证入口

```sh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
python3 -I -B tools/rust-sources/test_manage.py
python3 -I -B tools/rust-sources/test_audit.py
cargo test --locked --offline
cargo test --locked --offline -p blog-assets -p blog-browser-domain-tests
bash scripts/test-maintained.sh
cargo run --locked --offline -p blog-assets -- test --product-root . --offline
cargo run --locked --offline -p blog-assets -- build --product-root . --offline
node web/tests/transport-lifecycle.mjs
node web/tests/blog-interface-runtime.mjs
node web/tests/async-state.mjs
bash scripts/smoke.sh
```

密码验证补充公开标准向量：SHA-256 FIPS（含跨块和百万字节输入）、HMAC-SHA256 RFC4231、Argon2id RFC9106（含 secret、关联数据和多 lane）。这比仅验证“本实现生成、本实现校验”更能发现定制后的组合偏差，不等价于密码算法的完整安全证明。

本轮执行结果（Rust 1.99.0、Node 26.7.0，Linux x86_64）：

| 验证 | 结果 |
| --- | --- |
| 应用单元与新增 HTTP/密码集成测试 | 212 通过，3 项原有忽略测试未执行 |
| 服务端、部署/维护、SQLite、派生宏与媒体消费者 | 79 通过，1 项原有忽略测试未执行 |
| DOM/界面单元与集成测试 | 53 通过；另 476 文档测试通过，4 项原有忽略测试未执行 |
| 浏览器域与浏览器 Rust 测试 | 13 + 18 通过 |
| 资产构建器 | 50 通过；真实生成 ESM 经 Node 加载，6 项运行断言通过 |
| Range 与 Base64 显式依赖测试 | 36 + 7 通过 |
| 来源管理器、审计映射工具、独立性工具回归 | 35 + 9 + 13 通过；审计映射工具回归不代表在线 advisory 扫描 |
| 来源及离线解析 | 240 包与原始档案核验通过，Linux/WASM 空 Cargo home metadata 通过 |
| 构建及静态检查 | 正式 web-release WASM、原生服务/维护工具构建、项目 fmt 与 native/tools/WASM Clippy 均通过；内建依赖仍有既有编译告警 |
| Chromium 与 Firefox | transport-lifecycle、blog-interface-runtime、async-state、rust-native-domain 四脚本均通过 |
| 实际服务端 smoke | 登录、权限、内容/媒体操作、两浏览器管理交互和重启持久化通过 |

文件属主/ACL 测试及 Firefox 需要真实执行环境：命令沙箱会虚拟化属主、拒绝 ACL 设置或用户命名空间。相关用例在获准的真实云执行环境重跑通过，没有削弱应用权限断言或关闭浏览器安全机制。未执行的忽略测试不计入通过数，也未把遗留上游测试文件计入结果。

## 资源与维护边界

Range 校验在相同 Rust 1.99.0、优化参数和输入下比较原实现与当前实现。8192 个逆序、不重叠区间，20 次平均从约 12.69 ms 降为 0.033 ms；计时不含输入构造，先核对结果完全相同。这是函数微基准，不能推导为整个 HTTP 服务的延迟改善。

DOM 回归固定 128 个存活子节点并反复创建销毁 10,000 个节点，验证登记数量不超过 256，且存活节点清理恰好执行一次。该界限是特定工作负载的结构验证，不是全应用峰值内存测量。Base64 缓冲移动减少了明确多余的复制，但没有单独测量吞吐收益。

所有基础包仍排除在 workspace 成员外，且禁用自动集成测试发现。源码目录保留测试文件不表示它们已运行；本次修正的关键行为通过项目消费者或明确注册的测试执行。资产编译器中保留的全部上游算法测试尚未建立完整独立执行契约。`workspace-tests` 清点的是依赖图，实际测试通过仍需运行对应命令。

35 个编译阶段合并后的冷构建、增量重编译和峰值内存仍缺少同环境对比。不同代际语法树、宏/运行库、协议类型和不同用途编码不会仅因名字相似而合并。后续调整这些边界应先补对应消费者、基准和测试入口；本次不把没有证据的结构改动称为优化。
