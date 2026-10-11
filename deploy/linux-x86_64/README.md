# xocs Linux AMD64 部署

程序内嵌前台、管理后台资源与当前数据库定义。运行主机使用 Linux AMD64 GNU，并由 HTTPS 反向代理提供访问入口。
本手册也会复制到发行包根目录；在包中从 `bin/xocs` 安装程序。

## 安装程序与数据目录

先按主机规范创建不可登录的专用 `xocs` 用户和同名组。管理员在已验证的发行包根目录执行：

```sh
sudo install -d -o root -g root -m 0755 /opt/xocs/bin
sudo install -o root -g root -m 0755 bin/xocs /opt/xocs/bin/xocs
sudo install -d -o xocs -g xocs -m 0700 /var/lib/xocs
```

使用源码构建时，将第二条命令的 `bin/xocs` 换为构建输出的实际绝对路径。

## 初始化

下面使用 Bash，管理员在私密终端读取密码，再通过 stdin 交给服务用户：

```bash
read -rs -p '管理员密码: ' XOCS_PASSWORD; printf '\n'
printf '%s\n' "$XOCS_PASSWORD" | sudo -u xocs /opt/xocs/bin/xocs \
  --data-dir /var/lib/xocs init --username admin
unset XOCS_PASSWORD
sudo -u xocs /opt/xocs/bin/xocs --data-dir /var/lib/xocs config validate
```

`init` 只创建全新数据库；已有站点直接执行校验。运行用户拥有数据、媒体和日志目录，权限 `0700`，数据库文件为 `0600`。
校验以只读快照核对当前结构，失败时保留原文件。当前源码格式为 `xocs-db-v3`、结构修订 3；实际程序身份和校验结果是部署依据。

## 配置 HTTPS 和评论来源

由本机 nginx 直接接收访客 HTTPS 连接时，在站点的 HTTPS server 中设置：

```nginx
location / {
    proxy_pass http://127.0.0.1:8081;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-Proto https;
}
```

代理覆盖访客提交的 `X-Real-IP`，提供单个有效 IP。后端仅信任配置中的精确对端地址；`Forwarded` 与 `X-Forwarded-For` 不参与评论来源选择。
多级代理需在最后一跳可靠确定真实访客地址后覆盖该头。此设置只影响匿名评论，弹幕和文章密码尝试仍按连接对端限流。

## 启动与检查

```sh
sudo -u xocs /opt/xocs/bin/xocs --data-dir /var/lib/xocs run \
  --bind 127.0.0.1:8081 --trusted-proxies 127.0.0.1
```

保持该进程运行，通过实际 HTTPS 域名打开 `/`，后台在 `/admin`。生产保持默认安全 Cookie；`--development-http` 留给回环开发测试。
在另一终端检查：

```sh
sudo -u xocs /opt/xocs/bin/xocs --data-dir /var/lib/xocs status --json
```

应返回真实 readiness 和服务身份。再登录后台、发布测试文章并查看前台。停止服务使用 SIGTERM 或前台 Ctrl+C，等待进程退出后维护数据。

## 可选 JSON 配置

可在服务用户拥有的 `0700` 私有父目录下创建 `0600`、单硬链接 JSON 文件：

```json
{"data_dir":"/var/lib/xocs","bind":"127.0.0.1:8081","trusted_proxies":["127.0.0.1"]}
```

各命令使用 `--config /absolute/private/config.json`。优先级为 CLI、显式环境映射、文件、默认值；每层均校验，修改后重启。
匿名评论显示“尚未确认”时使用原标签页的确认操作，保持同一提交身份和正文；持久回执会返回原结果，避免重复发表。
完整使用与开发说明见 [xocs 文档](https://github.com/isarmg/xocs/blob/main/docs/README.md)，选择与程序源码 SHA 对应的提交阅读。
