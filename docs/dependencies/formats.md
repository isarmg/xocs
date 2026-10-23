# 数据格式

接口索引：[format](catalog.md#format)、[encoding](catalog.md#encoding)、[checksum](catalog.md#checksum)。

## 序列化和配置

`format/data-model` 提供共享序列化 trait，派生器和语法支持分别独立；`format/json`、`binary-record` 与 `json-schema` 使用这套类型。需要跨前后端共享的数据规则应在网络边界显式验证，不能把成功反序列化等同于业务合法。

`format/configuration` 组合 TOML 的读取、保留格式编辑、输出和日期类型，资料在源码旁并进入 rustdoc：

- [概览](../../src/foundation/format/configuration/retained/docs/components/toml/overview.md)、[编辑](../../src/foundation/format/configuration/retained/docs/components/toml/editor.md)、[写入](../../src/foundation/format/configuration/retained/docs/components/toml/writer.md)。
- [日期](../../src/foundation/format/configuration/retained/docs/components/toml/datetime.md)、[跨度](../../src/foundation/format/configuration/retained/docs/components/toml/span.md)。

产品站点配置使用 JSON；TOML 接口用于构建配置与依赖消费者，不改变[站点配置格式](../configuration.md)。

## 日期与版本

[date-time](../../src/foundation/format/date-time/README.md)提供日历计算、检查式构造、解析、格式化和时间偏移。Linux 本地时区由系统文件和 `TZ` 解析；浏览器本地时间通过 JavaScript `Date`。产品日期输入、存储解释和显示统一使用 UTC，不采用浏览器本地时区作为业务日界。

`release-identity` 处理版本身份，`parser-combinators` 提供文本解析组合器。格式与错误位置应保留原输入边界，特别是 UTF-8、CRLF 与多字节字符。

## 字节、文本编码与压缩

`encoding` 层按使用场景保留字节缓冲、搜索、Base64/十六进制、整数与浮点格式化、Unicode 解码、压缩、帧与变长整数。编码过程的容量、合法输入、溢出和错误语义以每种接口的文档为准。`checksum/fast-crc` 提供传输和归档校验；CRC 不承担密码学完整性保证。
