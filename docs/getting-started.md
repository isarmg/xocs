# 安装第一个站点

下载当前发行包和 SHA256SUMS，核验归档摘要以及包内完整文件清单。复制 `share/config/config.json.example` 为自己的私有配置，执行[部署说明](../deploy/linux-x86_64/README.md)中的 `install.sh`，从 stdin 提供唯一管理员密码。

安装完成后自行执行 `sudo systemctl enable --now xocs.service`，通过配置监听地址的 `/readyz` 确认真实就绪，再打开公开首页和 `/admin`。前台和后台资源内嵌同一 Server，运行命令只有 `bin/xocs --root ROOT`。

生产保持 `development=false` 与 Secure Cookie。本机隔离测试可以把配置中的 development 设为 true，监听回环地址。源码测试使用 `cargo test` 的 public offline 适配器；初始化和维护不进入运行程序 CLI。

数据库格式、配置和目录只支持当前基线；已有旧站点的数据不能由修改版本字段或重新初始化来适配。下一步见[发布与管理内容](usage.md)。
