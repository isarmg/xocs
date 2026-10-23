# 系统与运行时

[本地包目录](catalog.md#platform)提供系统层入口；[runtime](catalog.md#runtime)、[sync](catalog.md#sync)、[filesystem](catalog.md#filesystem)与[diagnostics](catalog.md#diagnostics)列出相关实现。

## Linux 接口

系统层固定 AMD64 Linux ABI，提供错误码、系统调用、socket 选项、时间区域发现和处理器能力探测。`system-abi` 与 `linux-abi` 分别承担 C ABI 声明和底层 Linux 常量，`system-calls` 封装实际系统调用。产品及构建工具的入口检查原生 HOST/TARGET。

[系统随机数](../../src/foundation/platform/system-entropy/README.md)在 Linux 直接使用内核随机源，处理短读和中断，向调用者保留失败信息；浏览器使用 Web Crypto。密码相关消费者使用同一熵源。

## 异步 I/O 与线程

`task-scheduler` 提供运行时、任务、时间和异步 I/O；`io-poller` 使用 Linux epoll，`readiness-signal` 提供唤醒。等待就绪后仍须处理 `WouldBlock` 并按实际事件清除就绪状态，取消等待时释放注册。实现入口见 [I/O driver](../../src/foundation/runtime/task-scheduler/src/runtime/io)。

`AsyncFd` 的 `register` 与 `register_with_interest` 是 unsafe 注册入口：调用者须保证底层文件描述符在整个注册期间保持有效、打开且不变，包括所有别名和可变访问。服务监听器独占持有标准库 `TcpListener`，只共享借用它，注销后才关闭；`AsFd` 借用直接来自实际支持 I/O safety 的内部对象。

并行迭代与异步任务有不同调度接口。CPU 数据并行使用 `parallel-iterator` 与 `worker-pool`，其内部协议分别见[迭代器 plumbing](../../src/foundation/runtime/parallel-iterator/src/iter/plumbing/README.md)和[线程休眠](../../src/foundation/runtime/worker-pool/src/sleep/README.md)。阻塞任务需要明确并发上限，不能占用运行时的 I/O 工作线程执行无界计算。

## 共享状态与诊断

`sync` 层按用途提供互斥锁、条件通知、一次初始化、消息通道、线程局部存储和 pinned projection。跨线程所有权、取消及用户回调 panic 都属于同步边界；调用者仍须遵守每种类型的借用和锁顺序。

`diagnostics` 层提供结构化事件、错误上下文、错误派生和符号解码。`filesystem/temporary-storage` 提供临时文件与目录；权限和生命周期要求由实际使用它的安装、快照和构建流程决定。
