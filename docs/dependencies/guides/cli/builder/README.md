# 参数 Builder

Builder 与解析运行时共同位于 `blog-cli-arguments`。`Command` 管理应用、子命令与帮助，`Arg` 管理单个参数，`ArgGroup` 表达参数组关系；`ArgAction` 决定值存储、计数、布尔标志或帮助版本行为。

1. 创建 `Command`，定义程序名、说明及子命令。
2. 添加 `Arg`，指定名称、短/长选项、位置、动作和参数值解析器。
3. 使用 `try_get_matches_from` 解析显式输入，或使用 `get_matches` 读取进程参数。
4. 通过 `ArgMatches::get_one`、`get_many`、`get_flag` 等与动作匹配的接口读取结果。

使用 `value_parser!` 选择 Rust 类型对应的解析器；额外转换和验证由 `TypedValueParser` 完成。`get_matches` 会按 CLI 行为退出进程，嵌入式调用和测试应选择返回 `Result` 的 `try_*` 方法。

- [运行时与特性](../runtime/README.md)
- [Builder 入门交互示例](../runtime/examples/tutorial_builder/01_quick.md)
- [完整 Builder 教程源码](../../../../../src/foundation/cli/arguments/src/_tutorial.rs)
- [API 实现入口](../../../../../src/foundation/cli/arguments/src/builder/mod.rs)

许可：[Apache-2.0](../../../../../compliance/licenses/upstream/clap_builder-4.6.7/LICENSE-APACHE) 或 [MIT](../../../../../compliance/licenses/upstream/clap_builder-4.6.7/LICENSE-MIT)。
