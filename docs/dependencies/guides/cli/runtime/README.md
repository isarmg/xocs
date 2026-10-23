# 命令行解析

`blog-cli-arguments` 提供 `blog_cli_arguments` 库，包含 builder、参数匹配、错误处理及可选派生宏。实现位于[本地包](../../../../../src/foundation/cli/arguments/Cargo.toml)，供 Blog 服务与 setup 工具使用。

## 选择接口

- Builder：使用 `Command`、`Arg` 和 `ArgAction` 定义参数，通过 `ArgMatches` 读取类型化值，见 [builder 说明](../builder/README.md)。
- 派生：在本地路径依赖上启用 `derive`，使用 `Parser`、`Args`、`Subcommand` 和 `ValueEnum`。宏生成的路径使用 `clap` 作为消费者引用名，示例显式声明 `extern crate blog_cli_arguments as clap;`。

基础 builder 示例：

```rust
use blog_cli_arguments::{Arg, Command};

let matches = Command::new("blog-tool")
    .arg(Arg::new("root").long("root").required(true))
    .try_get_matches_from(["blog-tool", "--root", "/srv/blog"])?;
assert_eq!(matches.get_one::<String>("root").map(String::as_str), Some("/srv/blog"));
# Ok::<(), blog_cli_arguments::Error>(())
```

示例程序保存在[源码 examples](../../../../../src/foundation/cli/arguments/examples)，终端交互示例位于[本地教程目录](examples/)。产品命令以 `blog --help`、`blog-setup --help` 和 `blog-assets --help` 为准。

## 特性

默认提供标准库支持、彩色输出、帮助、用法、错误上下文与建议。`derive` 开启派生宏，`cargo` 开启 Cargo 元信息宏，`env` 支持环境默认值，`string` 支持动态构造的字符串。`unstable-doc` 将完整教程和参考模块纳入 rustdoc；它包含额外文档示例所需特性。

生成当前 API 的命令见[依赖参考](../../../README.md#api-文档)。许可证：[Apache-2.0](../../../../../compliance/licenses/upstream/clap-4.6.7/LICENSE-APACHE) 或 [MIT](../../../../../compliance/licenses/upstream/clap-4.6.7/LICENSE-MIT)。
