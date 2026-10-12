# xocs 部署与运维

支持 Linux AMD64 GNU。Web 资源内嵌单个二进制。生产是普通 HTTP 后端，由部署环境提供 HTTPS 并保留原始 Host 和 Origin；无需特定代理或 X-Forwarded-Proto。默认保持 Secure Cookie 与来源校验，`development=true` 仅用于回环 HTTP 开发。不要将明文后端暴露给不可信网络。

## 首次安装

核对发行包校验和并解压。在包根目录准备自己的 JSON，选择 `data_dir`、监听地址等参数。唯一安装入口为 `install.sh setup CONFIG_JSON [USERNAME]`，用户名默认 admin，密码只从 stdin 读取。

```bash
umask 077
cp config/config.json.example ./site.json
${EDITOR:-vi} ./site.json
read -rs -p '管理员密码: ' password; printf '\n'
printf '%s\n' "$password" | sudo ./install.sh setup "$PWD/site.json" admin
unset password
sudo systemctl enable --now xocs.service
sudo -u isarmg-xocs /opt/isarmg/xocs/bin/xocs status --json
```

脚本创建不可登录的普通 `isarmg-xocs` 用户和同名组，安装 `/opt/isarmg/xocs/bin/xocs`、私有配置 `/etc/isarmg/xocs/config.json` 以及普通 systemd unit；读取已选择配置中的 `data_dir` 后创建私有目录，执行 `config validate`、`init` 和 `check`。初始化成功后只提示启动，不自动启动服务。密码不放入配置、环境或命令参数。

配置文件由运行用户拥有，为单硬链接普通文件、权限 0600，直接父目录同属该用户且为 0700。`data_dir` 是唯一内部状态根，默认 `/var/lib/isarmg/xocs`；数据库固定为 `site.sqlite`，日志固定为 `logs/`。媒体默认 `data_dir/media`，可通过独立 `media` 字段指定其他私有业务目录。安装器创建外部媒体目录并只设置该目录所有权，不递归修改已有文件；systemd 写入白名单由所选状态和媒体路径生成。

`init` 只接受全新或空的私有状态根，拒绝覆盖；`run` 从不隐式初始化。`config validate` 只校验配置，初始化前可成功；`check` 离线只读检查数据库、schema、管理员及目录；`status` 只查询正在运行的实例是否就绪及其身份。

## 升级

核对并解压新发行包，在新包根目录运行：

```sh
sudo ./install.sh upgrade
```

同一个脚本读取已安装的配置并校验，只停止原本正在运行的服务；替换二进制和 unit 后检查状态，且仅在原服务运行时重新启动；随后用 `status` 确认就绪。原本停止的服务保持停止。JSON、数据库、媒体及日志保持原位；不会自动初始化或迁移。校验失败即停止，不改数据库来迎合不匹配的 schema。升级后的业务验证包括首页、登录、文章和媒体读取。

## 重置管理员密码

先停服，再输入新密码。此操作同时撤销该管理员的全部会话：

```bash
sudo systemctl stop xocs.service
read -rs -p '新管理员密码: ' password; printf '\n'
printf '%s\n' "$password" | sudo -u isarmg-xocs /opt/isarmg/xocs/bin/xocs admin reset-password --username admin
unset password
sudo systemctl start xocs.service
sudo -u isarmg-xocs /opt/isarmg/xocs/bin/xocs status --json
```

## 配置与日志

默认配置为 `/etc/isarmg/xocs/config.json`；缺失的隐式默认文件可跳过，显式 `--config` 或 `XOCS_CONFIG` 缺失即失败。优先级是 CLI > `XOCS_` 环境 > JSON > 默认值，每层分别校验。`bind` 默认为 `127.0.0.1:8081`，生产可用内网或通配地址；`development` 默认 false。

`log_level` 默认 info，支持 trace/debug/info/warn/error/off，环境变量只有 `XOCS_LOG_LEVEL`，CLI 使用 `--log-level`。前台和 systemd 使用同一套 xcss UTC JSON 结构日志，状态就绪后写入 `data_dir/logs` 的滚动文件，启动前诊断输出 stderr，默认每文件 8 MiB、保留 4 份归档。

`trusted_proxies` 默认空，只使用真实连接对端。可显式配置最多 32 个受控代理 IP/CIDR；公共中间件从对端开始，由右向左解析 `X-Forwarded-For` 链，跳过可信代理，在首个不可信地址停止。最多 32 跳，可信代理传入非法链返回 400，头缺失时回退连接对端。边缘代理必须覆盖或安全追加来源，不能直接透传用户伪造链；不读取 `X-Real-IP` 或 `Forwarded`。管理员认证、评论、弹幕及文章密码限流共用该地址。此设置不改变 HTTPS 或 Cookie 模式。

完整说明见[文档](https://github.com/isarmg/xocs/tree/main/docs)。
