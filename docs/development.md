# 开发与验证

在仓库根目录执行本文命令。源码职责见[架构](architecture.md)，依赖实现和本地参考资料见[依赖文档](dependencies/README.md)。

## 构建环境

准备 Rust 1.99.0（含 rustfmt、Clippy）、`wasm32-unknown-unknown` 标准库、Python 3.11+ 以及 AMD64 Linux GNU/glibc 的 C 编译环境。原生目标为 `x86_64-unknown-linux-gnu`，浏览器目标为 `wasm32-unknown-unknown`；构建脚本会检查宿主和目标。

所有 Rust 源码通过本仓库路径解析，共用根 workspace 和唯一 Cargo.lock。Cargo 默认离线。Rust 工具链、目标标准库和系统构建材料需由环境提供；完整边界见[源码组织与校验](source-layout.md)。

```sh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 build --locked --offline --bin blog
cargo +1.99.0 build --locked --offline -p blog-setup
```

## 本机运行

创建独立的开发 ROOT。以下配置允许回环 HTTP，初始化密码从标准输入读取：

```bash
BLOG_ROOT="$(pwd)/.local-run/site"
install -d -m 0700 "$BLOG_ROOT/config"
printf '%s\n' '{"bind":"127.0.0.1:18091","development":true}' > "$BLOG_ROOT/config/config.json"
chmod 0600 "$BLOG_ROOT/config/config.json"
read -r -s -p "管理员密码: " blog_password
printf '\n'
printf '%s\n' "$blog_password" | cargo +1.99.0 run --locked --offline -p blog-setup -- --root "$BLOG_ROOT" initialize --username admin
unset blog_password
./target/debug/blog --root "$BLOG_ROOT"
```

打开 `http://127.0.0.1:18091/` 和 `/admin`。已有实例使用 `blog-setup --root ROOT check` 检查；初始化只用于新实例。配置字段见[配置参考](configuration.md)。

## 资源构建与预览

`blog-assets` 编译 JS/CSS、调用 Cargo 构建浏览器 WASM，并在进程中生成绑定。生产构建不需要 npm、Node 或独立 wasm-bindgen CLI。WASM 使用根 `web-release` profile，启用体积优化、LTO 和 panic abort。

```sh
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 run --locked --offline -p blog-assets -- serve --product-root . --debug --bind 127.0.0.1:5173
cargo +1.99.0 run --locked --offline -p blog-assets -- server --product-root . --config blog-assets.json --mode development --offline
```

`serve` 提供开发资源和变更监控；需要后台 API 时启动本机服务，并用 `--proxy /api=http://127.0.0.1:18091` 配置转发。`preview` 只读提供已有资源。`server` 按根 `blog-assets.json` 协调前端与服务端构建，具体选项见[构建器说明](../tools/assets/README.md)。

| 输入或输出 | 用途 |
| --- | --- |
| `web/rust-web.toml` | 前端入口、资源和预算配置 |
| `blog-assets.json` | 前端与服务端构建入口 |
| `web/pkg` | 生成的 WASM、浏览器绑定与元数据 |
| `web/dist` | 可分发静态资源 |
| `web/dist/assets/blog-assets.json` | 资源与源码摘要收据 |
| `CARGO_TARGET_DIR` | 原生 Cargo 构建输出 |
| `BLOG_WASM_TARGET_DIR` | WASM Cargo 输出，默认 `target/blog-browser` |
| `BLOG_WEB_DIST` 或 `--dist` | 指定静态资源输出 |

生成目录可重建，不能作为依赖源码来源。浏览器运行标记为 `html[data-blog-runtime="blog-interface"]`。

## 验证

先检查文档链接、Rust 文档输入和依赖资料索引：

```sh
python3 tools/check-docs.py
python3 tools/rust-sources/docs.py --check
```

CI 还会生成原生与 WASM 的完整 API 文档，并将文档警告作为错误处理；本地生成命令和输出位置见[依赖参考](dependencies/README.md#api-文档)。

按实际消费者和目标运行检查：根默认成员包括服务端、application 和 setup；构建器及浏览器域测试使用原生目标，浏览器应用使用 WASM 目标。

```sh
cargo +1.99.0 fmt -p blog -p blog-application -p blog-setup -p blog-assets -p blog-browser -p blog-browser-domain-tests -p blog-interface -p blog-dom -p blog-component-macros -- --check
cargo +1.99.0 clippy --locked --offline --all-targets -- -D warnings
cargo +1.99.0 clippy --locked --offline -p blog-assets -p blog-browser-domain-tests --all-targets -- -D warnings
cargo +1.99.0 clippy --locked --offline -p blog-browser --target wasm32-unknown-unknown -- -D warnings
cargo +1.99.0 test --locked --offline
cargo +1.99.0 test --locked --offline -p blog-assets -p blog-browser-domain-tests
bash scripts/test-maintained.sh
cargo +1.99.0 run --locked --offline -p blog-assets -- test --product-root . --offline
bash scripts/smoke.sh
```

`blog-assets` 的 Rust 测试会调用 PATH 中的 Node 26.7.0，检查实际生成的 JavaScript/WASM 初始化合同；这部分不需要 npm 包或浏览器。`scripts/test-maintained.sh` 运行已登记的基础库和消费者回归。`smoke.sh` 创建可丢弃实例，执行真实 HTTP、SQLite 写入和重启检查。

浏览器验证还需安装锁定测试工具及 Chromium、Firefox：

```sh
cd web
npm ci
npx playwright install --with-deps chromium firefox
npm run test:unit
npm run test:browser:blog-interface
```

完整入口和测试边界见[浏览器验证](browser-testing.md)。

## 修改源码和构建发行包

修改受管源码后，审查代码差异并接纳当前文件摘要，再执行相应验证：

```sh
python3 tools/rust-sources/manage.py refresh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
```

新增、合并或拆分包时同时更新用途映射和来源身份，字段见[源码组织与校验](source-layout.md)。

`blog-assets server --mode candidate` 使用真实 HEAD 并记录未提交源码；`--mode release` 要求干净源码。正式打包脚本 `scripts/package-linux-x86_64.sh --output-dir ABSOLUTE` 还要求与 HEAD、产品版本一致的注解 tag `vVERSION`。包内服务、初始化工具、资源和源码摘要共同绑定发行身份。安装与验证见[Linux 部署说明](../deploy/linux-x86_64/README.md)。

推送版本 tag 后，发布工作流执行验证并上传安装包及 `SHA256SUMS`，在 GitHub Releases 生成版本说明。仓库文档描述当前用法，版本变更记录由 [GitHub Releases](https://github.com/isarmg/xocs/releases) 提供。
