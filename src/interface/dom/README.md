# blog-dom

浏览器 CSR 运行时，提供类型化视图、响应式状态、控制流、context、Portal 和挂载生命周期，由 `blog-interface` 使用。

`Render::build/rebuild` 创建和更新 DOM，`DomView` 约束视图转换与空视图行为。`AddAnyAttr` 在视图保有具体类型时组合属性；类型擦除保留 CSR 渲染操作。文本和属性直接交给 DOM API。

`For` 使用相等性和哈希确定列表键。事件监听器绑定到元素并在视图清理时移除。响应式 effect 由浏览器或测试执行器调度；跨线程执行使用 `new_sync`。

组件行为测试位于 `src/interface/tests/csr_components.rs`，真实浏览器场景覆盖组件状态、表单及取消行为。构建与测试入口见[开发说明](../../../docs/development.md)和[浏览器测试](../../../docs/browser-testing.md)。API 可由本地 rustdoc 生成，见[依赖文档](../../../docs/dependencies/README.md)。

许可证保存在当前目录，原始来源、归档和许可映射见[第三方源码清单](../../../docs/THIRD_PARTY_RUST_SOURCES.md)。
