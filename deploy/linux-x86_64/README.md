# Blog 安装与维护

支持 Linux x86_64 GNU、systemd。服务、当前数据库初始化、安装、完整性检查和离线密码重置代码均由 Blog 仓库维护，不再下载 xcss 或 xssi。发行归档包含 `bin/blog`、`libexec/blog-setup`、`install.sh`、`maintenance.sh` 和构建元数据；`share/package.json` 与 `share/package.sha256` 记录完整不可变文件清单及 SHA-256、大小和权限。

执行归档内程序前，先通过可信渠道核对 `SHA256SUMS`。候选包的 `share/package.json` 明确标记 `kind=candidate`，记录真实 Git HEAD、源码树哈希及工作树是否有修改；候选包不代表正式发布。正式发行需干净源码树和绑定当前提交的 annotated `vVERSION` tag。

准备配置（示例省略的数据目录自动跟随 ROOT），然后首次安装到不存在的新 ROOT：

```bash
cp share/config/config.json.example /absolute/private/config.json
chmod 600 /absolute/private/config.json
systemd-ask-password 'Initial administrator password:' | sudo ./install.sh \
  --package /absolute/verified-package --root /opt/isarmg/blog \
  --config /absolute/private/config.json --username admin
sudo systemctl enable --now blog.service
curl --fail http://127.0.0.1:8081/readyz
```

安装工具先验证完整包，再创建固定非 root 账号 `isarmg-blog`。程序文件由 root 管理；配置为 `root:isarmg-blog` 的 `0640`，配置目录 `0750`；可写的 `state/`、`media/`、`logs/`、`run/` 为服务账号的 `0700`。初始化通过服务账号执行本地 `blog-setup`，使用当前 schema、私有文件验证及服务器相同实例锁，拒绝已有数据库。密码仅从有界的 stdin 输入，不写命令行、配置或日志。安装成功不自动启动或启用服务；失败保留已创建的部分安装供检查，不能直接覆盖重试。

生产 ROOT 必须位于 systemd 隔离后仍可见的位置；安装拒绝 `/tmp`、`/var/tmp`、`/home`、`/root`、`/run/user` 及其后代，因为服务启用 `PrivateTmp=true` 和 `ProtectHome=true`。默认 `/opt/isarmg/blog` 不受影响。临时验收实例不安装 unit，可使用 `/tmp`。

此安装器只接受 ROOT 下的 state/media，systemd 只允许写入上述四个目录。高级外置布局可先人工准备受管配置和权限，再使用 `blog-setup`；需同步由运维明确配置服务可写目录。开发源码使用 `cargo build --locked --offline -p blog-setup`，以实例账号执行：

```bash
blog-setup --root /absolute/instance initialize --username admin
blog-setup --root /absolute/instance check
blog-setup --root /absolute/instance admin-reset --username admin
```

`initialize` 与 `admin-reset` 从 stdin 读取密码。配置固定为 `ROOT/config/config.json`；可选 `--config FILE` 只接受这一物理位置。服务占用实例锁时，检查、初始化和重置均拒绝运行。检查核验当前数据库 schema、实际数据和凭据；密码重置执行事务并撤销旧会话。

安装实例维护需先停止服务，再检查或重置密码：

```bash
sudo systemctl stop blog.service
sudo ./maintenance.sh --root /opt/isarmg/blog check
systemd-ask-password 'New administrator password:' | \
  sudo ./maintenance.sh --root /opt/isarmg/blog admin-reset --username admin
sudo systemctl start blog.service
```

当前不提供自动版本升级、旧结构迁移或自动卸载。替换版本前停止服务并备份配置、数据库和媒体；由运维核实目标版本的数据兼容性。不要删除锁文件绕过正在运行的服务器。服务运行入口仍为 `bin/blog --root ROOT`，生产保持 Secure Cookie、Host/Origin 来源校验；`development=true` 仅允许回环 HTTP。
