# 参数词法层

词法层是 `blog-cli-arguments` 的私有实现，按操作系统字符串处理原始参数，不提供独立 Cargo 包或外部公开模块。公共消费者使用 [Builder 与解析运行时](../runtime/README.md)。

`RawArgs` 遍历原始参数并提供游标，`ParsedArg` 区分长选项、短选项、选项终止符和值，`ShortFlags` 逐项遍历组合短标志。命令行词法依赖上下文：值是否接受前导连字符、一个选项消费多少个值等规则由上层解析器决定。

源码入口为 [lexer/mod.rs](../../../../../src/foundation/cli/arguments/src/lexer/mod.rs)，操作系统字符串工具在 [lexer/ext.rs](../../../../../src/foundation/cli/arguments/src/lexer/ext.rs)。非法 UTF-8、Unicode 选项与参数边界应按这些类型的返回值处理。

许可：[Apache-2.0](../../../../../compliance/licenses/upstream/clap_lex-1.1.1/LICENSE-APACHE) 或 [MIT](../../../../../compliance/licenses/upstream/clap_lex-1.1.1/LICENSE-MIT)。
