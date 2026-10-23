# 文本与 Unicode

接口索引：[text](catalog.md#text)、[unicode](catalog.md#unicode)。

## 字符串处理

`text` 层提供 ASCII、忽略大小写、标识符、词边界、模式搜索、文本相似度、终端宽度与排版。Shell 参数的引用和拆词是独立接口，不能作为执行任意命令的输入验证；[引用说明](../../src/foundation/text/shell-words/src/quoting_warning.md)解释具体限制。

终端宽度与 Rust 字符串字节偏移不同，组合字符和宽字符应通过对应宽度/边界 API 处理。排版使用 `line-layout` 和 `monotone-minima`，其配置和算法见各自本地来源参考。

## Unicode 数据

`unicode` 层保留规范化、字符属性、文本边界、区域标识及区域选择数据，并提供紧凑 trie、借用数据、向量和提供者协议。数据与读取类型属于同一接口组合；增加语言或区域能力时，应同时核对数据覆盖和消费者启用特性。

域名与 URL 处理使用这些 Unicode 接口，但站点是否允许某个协议、图片来源或提交内容仍由产品验证决定。浏览器与原生的 URL API 边界见本地 `network/web-address` 和实际调用者文档。
