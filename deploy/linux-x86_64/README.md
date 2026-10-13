# xocs 安装与维护

支持 Linux x86_64 GNU、systemd。Server 只负责运行，安装和维护入口来自独立 xssi 仓库。正式发行物是 `xocs-VERSION-linux-x86_64.tar.gz`，签名清单位于 `share/package.json` 和 `share/package.sig`。

执行发行物内任何代码前，从独立可信渠道取得发行公钥及其 DER SHA256 指纹，核验归档摘要、Ed25519 签名和完整文件清单。包内自带公钥不能自行成为信任来源。

准备私有配置文件，例如复制 `share/config/config.json.example` 后编辑。首次安装：

```bash
sudo ./install.sh --root /opt/isarmg/xocs --config /absolute/config.json \
  --public-key /absolute/release-public.pem --key-sha256 VERIFIED_DER_SHA256
sudo systemctl enable --now xocs.service
curl --fail http://127.0.0.1:8081/readyz
```

管理员密码由安装命令从 stdin 读取，不写入命令行、配置或日志。安装创建固定服务账号、目录、当前数据库及唯一管理员、系统服务和权限声明；完成后不自动启动或启用服务。失败保留明确记录，改正输入后可重试，不覆盖已有状态。

ROOT 默认 `/opt/isarmg/xocs`，包含 `bin/`、`libexec/`、`share/`、`config/`、`state/`、`media/`、`logs/`、`run/`、`maintenance/`。只允许明确外置的实例数据库和媒体目录；systemd 配置按操作系统要求放置。服务账号仅能写数据、媒体、logs、run，不能改程序或维护记录。实际 systemd 可写目录由安装时通过离线配置报告生成。

维护用 `maintenance.sh --help` 查看只读检查、离线密码重置命令。修改凭据前手动停止服务。管理员哈希、会话及审计存储于 `state/site.sqlite` 的公共表；没有管理员 JSON、会员账号或 IP 失败计数表。

升级用 `upgrade.sh --help` 选择明确正式版本、可信公钥及 xssc 二进制。先下载并校验，再停止所有写入服务，取得维护锁、备份并验证实际数据，执行声明的转换、替换签名 payload、启动并核验真实程序和 `/readyz`。恢复意图和备份保存在 ROOT/maintenance；新程序可能已经写数据时，回退需要明确的数据恢复授权。当前仅支持新基线同一数据结构的发行物间升级，拒绝旧结构和 systemd 模板变更。

运行只使用 `bin/xocs --root ROOT`；没有 init、check、status、admin 或 config 子命令。生产保持 Secure Cookie、Host/Origin 来源校验；`development=true` 仅允许回环 HTTP。Server 不读取代理身份或转发协议头。

卸载由用户自行停止、禁用并处理 systemd 配置、目录和账号；工具不自动卸载或清空业务数据。正在运行的服务不能通过删除锁文件绕过维护互斥。
