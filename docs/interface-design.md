# 管理后台样式

管理后台使用声明式几何角色，页面只负责表单和内容管理等业务布局。尺寸与间距的唯一实现入口是 `src/web/styles/admin-ui/geometry.css`，后台领域布局位于 `src/web/src/admin.css`。这些规则限定在后台和登录壳内。

公开页面继续使用 `src/web/src/public-original.css` 和 `src/web/src/public-layout.css`，保留其布局、尺寸、字体、配色与原有圆角覆盖范围，仅共享新的 G4 曲线计算。

公开页现有有效轮廓范围不超过 54px。同范围下，两种曲线的双向最大边缘差连同双方各 0.02px 的 SVG 离散误差，上界小于 1.72px；不通过改变元素大小来补偿圆角。

| 角色 | 尺寸与行为 |
| --- | --- |
| 页面 | 普通文档流、通栏内容；左右至少 16px，并计入设备安全区 |
| 内容行、普通操作 | 高度至少 36px，字号 14.4px，行高 1.5 |
| 输入、顶部图标操作 | 高度至少 44px；普通输入字号 17.6px |
| 面板、菜单、通知、弹窗 | 1px 边框；连续轮廓范围 18px |
| 输入轮廓 | 范围为控件高度的一半，普通输入为 22px |
| 相邻面板 | 直接接触的可见表面之间相隔 18px；已有文字行分隔时不重复加距 |
| 导航 | 自然换行，当前页面用下划线标识 |

后台根字号为 14px。`stack`、`flow`、`frame`、`row` 和 `input` 角色分别表达堆叠、同行、表面、内容行和输入几何；新增后台页面复用这些角色，不另设相同用途的尺寸常量。登录壳独立维护其 380px 最大宽度、六行表单和至少 44px 的操作区域。

连续圆角由 `src/interface/src/server_corners/profile.rs` 与 `geometry.rs` 在 Rust 中计算，采用 G4 过渡曲线、圆弧与镜像过渡，圆弧比例为 0.4。SVG 离散路径带有独立误差界；输出折线本身不宣称具有 G4 连续性。圆形头像与账号入口仍使用真圆。

设计参照为 [zxs 的当前几何源码](https://github.com/isarmg/zxs/blob/8c062902a995e23551662bcf1a723f0c78b5cedd/src/web/public/styles/components/geometry.css)及同一提交的 Rust 曲线实现。保留的来源文件与摘要位于 `compliance/interface-reference/`。本仓库维护实际实现，不依赖运行时读取参考仓库。

`test/browser/style-contract.mjs` 在 Chromium 和 Firefox 中测量后台实际布局、控件尺寸与曲线状态。公开页面可使用 `BLOG_PUBLIC_BASELINE_DIST=/absolute/previous-dist node test/browser/public-style-baseline.mjs` 对照修改前产物的布局、字体、配色和圆角边缘；基线产物不提交仓库。交互、键盘和响应式场景见[浏览器测试](browser-testing.md)。
