# Web 原始来源与本地维护边界

此目录的五组 `*.SOURCE.json` 和 `*.snapshot.tar.gz` 保留重构前已经逐文件验证一致的原始导入快照。归档包含原始清单及该清单登记的全部文件，保持真实原名称和LICENSE版权；属于导入时的完整组件快照，不冒充完整上游Git仓库归档。

本地派生目录：`web/ui`（xcss-web），`web/ui/vendor/blog-ui-core`（xcss-web-core，Leptos派生），`web/ui/vendor/blog-ui-macro`（xcss-web-macro，Leptos派生），`web/styles`（@xcss/web必要CSS/字体）以及 `tools/web-build`（xcss-web-build）。各自 `ORIGIN.json` 记录原归档/原清单的SHA-256与2026-10-22整合修改。Maple字体更早来源保留在 `web/styles/web-fonts/provenance.json`；字体及OFL许可证未改字节。

这些历史清单不用于宣称当前维护代码未修改。当前代码可独立维护，构建器为实际源码生成独立清单并在构建前后校验；Node工具链测试逐项核对原归档与原始逐文件SHA。构建、测试与发行无需访问或同步xcss/xssi仓库。
