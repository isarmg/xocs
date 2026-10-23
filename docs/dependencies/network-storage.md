# 网络与存储

接口索引：[network](catalog.md#network)、[storage](catalog.md#storage)、[native](catalog.md#native)。

## HTTP

HTTP 值、Body、传输、请求流水线、路由和中间件分层保存。`http-message` 提供共享请求/响应类型，`body-stream` 处理流式 Body，`http-transport` 承担连接，`request-router` 组织路由和提取器。地址、媒体类型、范围请求与表单解析由独立用途包提供。

路由指南保存在源码旁，并参与 Rust 文档生成：

- [处理函数](../../src/foundation/network/request-router/src/docs/handlers_intro.md)、[提取器](../../src/foundation/network/request-router/src/docs/extract.md)、[响应](../../src/foundation/network/request-router/src/docs/response.md)。
- [错误处理](../../src/foundation/network/request-router/src/docs/error_handling.md)、[中间件](../../src/foundation/network/request-router/src/docs/middleware.md)、[处理函数类型错误](../../src/foundation/network/request-router/src/docs/debugging_handler_type_errors.md)。
- [路由](../../src/foundation/network/request-router/src/docs/routing/route.md)、[嵌套路由](../../src/foundation/network/request-router/src/docs/routing/nest.md)、[状态](../../src/foundation/network/request-router/src/docs/routing/with_state.md)。

服务端和资源开发代理复用同一套 HTTP 类型与传输。开发代理使用 HTTP/1 连接池，保持请求和响应的流式背压；超时覆盖整个转发生命周期。Cookie、CSRF、重定向和逐跳头处理位于产品入口，不能依靠通用客户端的隐式重试。

## SQLite

存储层只服务 SQLite。`sqlite-native` 编译仓库内 SQLite C 源码并导出绑定；`sqlite-access` 提供 Tokio 池、事务、预处理语句、值编解码与行访问；`record-derive` 生成记录映射，`fallible-rows` 提供可失败的行迭代协议。

[访问层 API 概览](../../src/foundation/storage/sqlite-access/src/lib.md)与本地 rustdoc 说明接口。连接池、忙等待和事务取消的行为应按实际调用处理。行可移动到另一线程，其内部解码状态不允许并发共享。

数据库身份与初始化规则见[产品合同](../contracts.md)；实例路径、备份和故障排查见[运维](../operations.md)。这些业务约束由应用层实现，不由通用 SQL 类型推断。

## 原生构建

`native` 层提供 C 编译、编译器能力及目标条件检测。SQLite 构建固定本地源码和产品编译选项，环境仍需 AMD64 Linux GNU/glibc 的 C 编译工具、系统头文件与链接器。Cargo 离线解析不等于环境自带这些系统工具。
