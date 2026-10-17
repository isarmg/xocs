# xocs 开发与验证

当前生产 Web 构建使用 `cargo +1.99.0 run --quiet --locked --manifest-path tools/web-build/Cargo.toml -- build --product-root .`（产品根目录），无需 npm 安装。开发监听与 Node 浏览器测试见 [Rust 原生 Web 构建](rust-native-web.md)；下述旧前端流程仅作历史验证参考。

## 构建

后台的[账号设置](account-settings.md)作为普通后台菜单页面，使用随仓库记录来源的 xcss Leptos 账号组件提供用户名、当前密码、新密码、确认新密码和保存按钮。

需要精确的 Node.js 26.7.0、Rust 1.99.0、wasm32-unknown-unknown 和 wasm-bindgen-cli 0.2.129。从项目根目录运行：

```bash
rustup toolchain install 1.99.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown
cargo +1.99.0 install wasm-bindgen-cli --version 0.2.129 --locked
cd web
npm ci
npm run build
cd ..
export XOCS_SOURCE_REVISION="$(git rev-parse HEAD)"
web/node_modules/.bin/xcss-build-server \
  --config xcss-web-build.json --mode release --no-install \
  --rust-only --source-revision "$XOCS_SOURCE_REVISION"
```

Rust 公共依赖固定官方 xcss 仓库的版本 `=1.0.0` 与完整 revision `b34b3861e15edd2b1348e7d03b9c9a93e97d6521`；一个 @xcss/web 包固定同版 `v1.0.0` 官方发行 URL 和真实归档的 lockfile integrity，不读取相邻工作区。Xocs 软件版本为 `1.0.1`，数据库格式身份为 `xocs-db-v1`，软件版本与数据格式版本分别管理。

服务端 Cargo workspace 与 lock 位于根目录；Leptos 浏览器工作区及独立锁位于 `web/leptos`，公共 Rust UI 快照位于其 `vendor/xcss-web` 并带逐文件来源校验。共同 builder 验证 Linux AMD64 GNU target、真实源码 revision 和实际二进制资源清单，报告真实输出路径；默认正式输出为 `target/x86_64-unknown-linux-gnu/release/xocs`。独立缓存通过 `CARGO_TARGET_DIR` 选择，打包脚本从精确源码提交生成私有快照，并使用固定 xssi 修订编译同源 setup。正式包要求干净源码、annotated `v1.0.1` 精确指向 HEAD 和匹配该 HEAD 的二进制身份，不允许 unbound 程序打包。

## 验证

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 scripts/build-leptos.py check
python3 scripts/build-leptos.py test
python3 -I -B scripts/test-leptos-build.py
python3 scripts/check-comment-retries.py "${CARGO_TARGET_DIR:-target}/debug/xocs"
cd web
npm run build
npm run test:unit
node tests/language.mjs
node tests/admin-layout.mjs
node tests/leptos-runtime.mjs
node tests/contract-fixes.mjs
node tests/public-layout.mjs
node tests/anonymous-comments.mjs
cd ..
bash scripts/smoke.sh
```

浏览器检查需要安装 Chromium 和 Firefox（`cd web && npx playwright install chromium firefox`）。语言与布局检查使用临时 Web 预览和模拟 API；后台检查覆盖导航、浏览器历史、手机布局、深色模式对比度、加载重试和设置保存。前台检查覆盖手机至 2560 像素宽屏、家页面的相册筛选与分页、计时、深色模式、空状态和匿名评论。`scripts/smoke.sh` 在本机回环地址启动临时服务，创建临时数据库和媒体目录，执行 API 与浏览器检查，结束时清理。

首次启动按[安装指南](getting-started.md)准备隔离的测试数据。配置、页面操作和评论重试分别见[配置](configuration.md)、[使用](usage.md)和[实现参考](current-guide.md)。

集成测试使用仅供测试的当前 DDL fixture，并运行仅接受 --root 的实际程序，验证未初始化拒绝、重复实例拒绝、管理员结构损坏拒绝与真实就绪；xssi 独立验证安装、密码重置和 xssc 的升级恢复事务。系统级服务验收仍需部署主机执行。

Web 业务代码和前后端边界说明见[Leptos 迁移记录](leptos-migration.md)。浏览器验收必须观察实际 WASM 请求以及 Rust 启动时设置的 `data-xcss-runtime=leptos`；旧实现与 HookHost 只用于历史行为对照。
