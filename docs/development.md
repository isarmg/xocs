# 从源码开发运行

准备 Rust 1.99.0（含 rustfmt、Clippy）、`wasm32-unknown-unknown` 目标和 AMD64 Linux GNU/glibc 原生编译环境（含 C 编译器）。所有项目入口直接检查 HOST/TARGET；原生只支持 `x86_64-unknown-linux-gnu`，浏览器使用 `wasm32-unknown-unknown`。根 Cargo.toml 是虚拟 workspace，只有根 Cargo.lock；服务端包位于 `src/server`，浏览器应用在 `src/browser`，本地 UI 在 `src/interface`，构建器 `blog-assets` 在 `tools/assets`，浏览器 WASM 绑定生成已整合到该工具。所有 Rust 依赖通过本仓库路径解析，默认离线，不需要相邻项目、预装 wasm-bindgen CLI 或注册表源码缓存。工具链、目标标准库和原生依赖不属于这份 Rust 源码闭包。

```sh
python3 tools/rust-sources/manage.py verify --metadata
python3 tools/check-independence.py
cargo +1.99.0 run --locked --offline -p blog-assets -- build --product-root . --offline
cargo +1.99.0 build --locked --offline --bin blog
cargo +1.99.0 build --locked --offline -p blog-setup
```

首次运行使用新的独立 ROOT。以下回环 HTTP 配置仅用于本机开发，密码由 `blog-setup` 从标准输入读取：

```sh
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

访问 `http://127.0.0.1:18091/` 和 `/admin`。对已有 ROOT 使用 `blog-setup --root ROOT check`，不要重复初始化。`src/server/examples/smoke-fixture.rs` 只提供测试账号，真实站点使用自选密码的 setup 工具。

验证按支持的入口分开运行：默认成员是服务端、`blog-application` 与 setup；Web 应用使用 WASM 目标，构建器和 domain 合同使用原生目标。不能用跨目标 `--workspace --all-features` 联合替代这些实际配置。

`blog-assets` 的 Rust 测试会调用 PATH 中的 Node 26.7.0，执行实际生成的 JavaScript/WASM 初始化契约；没有 Node 时明确失败。该测试不需要 npm 依赖或浏览器安装。

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

`scripts/test-maintained.sh` 注册实际基础实现与消费者回归，包含并发 panic、密码熵源、加密向量、编译器字符串边界和 Linux 时区。哈希库原有的数千万输入统计质量扫测不在常规 CI；保留的上游测试材料不等于已支持的测试入口。日期 JSON 和时区通过 `blog` 的 `date_time_contracts` 验证，不恢复未使用的上游 bincode 测试依赖。

浏览器测试另需 Node 26.7.0、npm 锁定依赖和浏览器执行环境：

```sh
cd web
npm ci
npx playwright install --with-deps chromium firefox
npm run test:unit
npm run test:browser:blog-interface
```

`web/package.json` 只有两个直接 devDependencies，`web/package-lock.json` 包含五个非根锁包，均是测试工具。首次安装这些 npm 包、浏览器和系统依赖可能联网；不能将 Rust 离线构建的结果扩大到这部分。浏览器验证入口及真实后端测试边界见[浏览器验证](browser-testing.md)。

`BLOG_WEB_DIST` 指向实际生成资源，`CARGO_TARGET_DIR` 与 `BLOG_WASM_TARGET_DIR` 分别选择原生和 WASM 构建输出；它们是可重建产物，不是源码来源。前端使用 `web-release` profile（继承 release，体积优化、LTO、panic abort）。候选模式明确记录未提交源码；正式发布仍要求干净源码和匹配的正式 tag，不能用修改版本字段代替。

当前用途目录、内部版本、宏与界面裁剪见[用途架构与定制](maintained-design.md)。所有 Cargo 操作仍从根 workspace 进行；不为工具或测试目录重建独立锁文件。

修改本地 Rust 源码后，先审查变动，再显式执行 `python3 tools/rust-sources/manage.py refresh` 接纳新的维护摘要；随后重新执行 `verify --metadata` 和对应验证。该操作不更新或伪造原始上游归档。具体规则见[源码管理说明](rust-sources.md)。
