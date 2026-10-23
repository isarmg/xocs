# blog-setup

离线初始化、检查实例和重置管理员凭据的 Rust 工具，直接调用 `blog-application` 的实例管理 API。

在仓库根目录构建：

```sh
cargo +1.99.0 build --locked --offline --release -p blog-setup
```

先准备私有的 `ROOT/config/config.json`，以实例服务账号执行：

```sh
systemd-ask-password 'Initial administrator password:' | blog-setup --root /absolute/instance initialize --username admin
blog-setup --root /absolute/instance check
systemd-ask-password 'New administrator password:' | blog-setup --root /absolute/instance admin-reset --username admin
```

密码从标准输入读取，限制为有长度上限的单行 UTF-8；不放入命令行、配置或日志。运行前停止服务，工具与服务端使用同一个实例锁。初始化拒绝已有数据库，检查验证数据库结构、实际数据和凭据，重置密码以事务更新账号并撤销旧会话。

离线操作仅读取明确指定的 `ROOT/config/config.json`，运行时 `BLOG_*` 环境覆盖不能将操作重定向到另一实例。工具先核对编译内的产品、数据库和应用源码身份；打包同时校验服务端与 setup 的身份一致。

安装包使用方式见[生产部署](../../../deploy/linux-x86_64/README.md)，源码实例配置见[首次运行](../../../docs/getting-started.md)。
