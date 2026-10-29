# 测试

产品、基础实现和构建工具源码位于 [`src/`](../src)，测试实现、测试专用数据和验证脚本集中在本目录。所有命令从仓库根目录执行。

| 目录 | 内容 |
| --- | --- |
| `application`、`server`、`interface` | 对应组件的 Rust 单元测试和集成测试 |
| `foundation/<层>/<包>` | 基础实现的单元测试、集成测试和专用数据 |
| `browser-domain` | 浏览器业务合同与 Markdown 的原生 Rust 测试包 |
| `browser` | Chromium、Firefox 场景和浏览器 fixture |
| `tools` | 构建器、来源管理和文档工具的测试 |
| `operations` | 真实后端冒烟、安装验收、打包和 CI 门禁测试 |

Rust 单元测试通过 `#[cfg(test)]` 和模块路径声明挂接到原业务模块，继续访问私有实现，不要求为测试公开接口。独立集成测试使用各包 Cargo manifest 的 `[[test]]` 入口。Cargo 仍由根 workspace 和唯一的 `Cargo.lock` 管理，不需要切换到测试目录运行。

```sh
python3 test/check-layout.py
python3 -I -B test/tools/test-layout.py
python3 -I -B test/server/query-indexes.py
cargo +1.99.0 test --locked --offline
cargo +1.99.0 test --locked --offline -p blog-assets -p blog-browser-domain-tests
bash test/run-maintained.sh
bash test/operations/smoke.sh
python3 -I -B test/operations/test-package-release.py
python3 -I -B test/tools/rust-sources/test_manage.py
node --test test/operations/check-ci-parallelism.test.mjs
```

浏览器测试依赖由 `src/web/package.json` 和锁文件固定。先在 `src/web` 执行 `npm ci` 并安装 Playwright 的 Chromium、Firefox，再运行 `npm --prefix src/web run test:unit` 或对应页面测试命令。完整环境和测试边界见[开发说明](../docs/development.md)及[浏览器测试](../docs/browser-testing.md)。

测试会创建可丢弃实例和数据。页面写入测试应连接专用测试实例；生产实例的运行与管理方法见[部署说明](../deploy/linux-x86_64/README.md)。
