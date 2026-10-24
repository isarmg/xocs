# 管理后台样式

管理后台使用声明式几何角色，页面只负责表单和内容管理等业务布局。尺寸与间距的唯一实现入口是 `src/web/styles/admin-ui/geometry.css`，后台领域布局位于 `src/web/src/admin.css`。这些规则限定在后台和登录壳内。

`blog-stack` 和 `blog-flow` 只负责布局；只有显式的 `blog-frame` 绘制面板。`editor-form`、`inline-form`、`blog-content-panel` 和 `blog-table-scroll` 等业务或滚动类不自动产生边框。容器含输入控件、图片或其他表面时，使用 `blog-frame--padded` 保留四周 18px 留白；其内部文字行不再扣除外层边框宽度。

`blog-stack--spaced` 用于多个独立任务或表面的分组，提供 18px 间隔，隔着透明字段集也不贴合；它不再叠加相邻表面的外边距。`blog-form-flow` 只给表单流提供 18px 横向间距，导航与普通操作行仍保持各自布局。只读表格和文字指标卡可以使用无内边距的框；可编辑表格在有内边距的外框中使用透明滚动区。

与后台同名的公开样式限定在 `.original-public` 下，避免分页和状态文字等组件覆盖后台。后台的手写字段标签也使用共享行角色；Markdown 预览使用后台表面和排版，不复用公开文章的布局类。媒体查看器的公开与后台外观分别在各自壳内定义。

公开页面继续使用 `src/web/src/public-original.css` 和 `src/web/src/public-layout.css`，保留其布局、尺寸、字体、配色与原有圆角覆盖范围，仅共享新的 G4 曲线计算。

公开页现有有效轮廓范围不超过 54px。同范围下，两种曲线的双向最大边缘差连同双方各 0.02px 的 SVG 离散误差，上界小于 1.72px；不通过改变元素大小来补偿圆角。

| 角色 | 尺寸与行为 |
| --- | --- |
| 页面 | 普通文档流、通栏内容；左右至少 16px，并计入设备安全区 |
| 内容行、普通操作 | 高度至少 36px，字号 14.4px，行高 1.5 |
| 输入、顶部图标操作 | 高度至少 44px；普通输入字号 17.6px |
| 面板、菜单 | 1px 边框；连续轮廓范围 18px |
| 通知、弹窗 | 无边框面板；连续轮廓范围 18px |
| 输入轮廓 | 范围为控件高度的一半，普通输入为 22px |
| 嵌套表面 | 含控件或子表面的外框四周留白 18px，避免二、三层边框贴合 |
| 相邻面板 | 表面组相隔 18px；未分组的堆叠只对直接相邻表面加距，避免重复计算 |
| 导航 | 自然换行，当前页面用下划线标识 |

后台根字号为 14px。`stack`、`flow`、`frame`、`row` 和 `input` 角色分别表达堆叠、同行、表面、内容行和输入几何；新增后台页面显式组合这些角色，不另设相同用途的尺寸常量。富文本预览也使用有内边距的框，因为正文可能直接以代码块或图片开头。普通输入与复选框在键盘聚焦时显示 2px 轮廓；后台链接使用主题色。登录壳独立维护其 380px 最大宽度、六行表单和至少 44px 的操作区域。

复选框与多行说明保持并排。表格元数据保持不折行，标题及长内容列保留阅读宽度；小屏使用表格容器内的横向滚动呈现操作列，避免将文字压成逐字换行或撑大整页。

连续圆角由 `src/interface/src/server_corners/profile.rs` 与 `geometry.rs` 在 Rust 中计算，采用 G4 过渡曲线、圆弧与镜像过渡，圆弧比例为 0.4。SVG 离散路径带有独立误差界；输出折线本身不宣称具有 G4 连续性。圆形头像与账号入口仍使用真圆。

设计参照为 [zxs 的当前几何源码](https://github.com/isarmg/zxs/blob/8c062902a995e23551662bcf1a723f0c78b5cedd/src/web/public/styles/components/geometry.css)及同一提交的 Rust 曲线实现。保留的来源文件与摘要位于 `compliance/interface-reference/`。本仓库维护实际实现，不依赖运行时读取参考仓库。

`test/browser/style-contract.mjs` 在 Chromium 和 Firefox 中测量后台实际布局、控件尺寸与曲线状态，包括表格操作、编辑器、弹窗、认证错误及下拉菜单的渲染回退。公开页面可使用 `BLOG_PUBLIC_BASELINE_DIST=/absolute/previous-dist node test/browser/public-style-baseline.mjs` 对照修改前产物的布局、字体、配色和圆角边缘；曲线可以保持不变，边缘位移须小于 2px。基线产物不提交仓库。交互、键盘和响应式场景见[浏览器测试](browser-testing.md)。
