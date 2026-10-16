# React 行为对照

这里保留迁移前的 React/TypeScript 源码，供原 HookHost 与纯函数测试核对历史行为。它不属于 Vite 的生产入口，不要求安装 React runtime，也不能证明 Leptos 运行时通过。

生产业务代码位于 `web/leptos/src`。对应的新运行时要求由 Rust 单测和真实浏览器脚本验证，重点包括响应模型及页身份、UTC/UTF-8 边界、匿名提交恢复、重复写入、脏草稿重试、过期请求取消、键盘选择器、权限与全部页面操作。新增或修改行为时，应改 Rust 与新运行时测试，不应把这里的代码重新接回生产入口。

原浏览器端到端脚本保留真实界面断言；`tests/leptos-runtime.mjs` 还要求实际 WASM 请求和 `data-xcss-runtime=leptos`。
