# 密码学

[crypto 目录](catalog.md#crypto)列出当前摘要、密码格式、密码派生和常量时间接口；随机数入口见[系统熵源](../../src/foundation/platform/system-entropy/README.md)。

## 密码与共享接口

产品管理员密码使用 Argon2id。`password-derivation`、`memory-hard-hash`、密码格式和专用 trait 组成密码消费者的类型闭包；盐和随机值使用操作系统或 Web Crypto 熵源，失败会向上返回。

`digest-traits`、`message-authentication` 和 `secure-hash` 为消息摘要与认证提供接口；算法标识、固定数组和常量时间操作由明确的基础包提供。通用摘要与密码消费者的接口按真实版本和类型关系分开，不能通过名字相近的 trait 随意混用。基础组合方式见[摘要接口参考](guides/cryptography.md)。

## SHA-2 与处理器能力

[SHA-2 本地说明](../../src/foundation/crypto/secure-hash/README.md)给出算法、Digest 示例、后端配置与 CPU 条件。Linux AMD64 正常构建按运行时能力选择 SHA-NI、AVX2 或便携 Rust；WASM 按目标特性选择对应实现。强制硬件后端需要同时满足编译目标和实际 CPU 条件。

常量时间比较、条件移动与编码包只保证其文档所述操作，不自动使整个密码流程成为常量时间。密码计算的内存、并发和阻塞调度由应用层限制；行为见[产品合同](../contracts.md)。

## 许可与安全信息

本地来源参考与完整归档用于核对原始身份，安全审计按当前所有者恢复来源身份。定制实现需要单独核对 advisory 涉及的代码路径；操作说明见[源码布局](../source-layout.md)。
