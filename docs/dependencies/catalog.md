# 本地包与文档目录

此页由 `python3 tools/rust-sources/docs.py` 按当前 Cargo 包和源码所有者生成。先阅读[依赖指南](README.md)了解支持范围；具体 API 以本地 Rust 文档为准。

## project

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-application](../../src/application/Cargo.toml) | 认证、SQLite 持久化、安装契约和资产响应 | [lib.rs](../../src/application/src/lib.rs) | Rust 文档注释 | 本项目实现 |
| [blog](../../src/server/Cargo.toml) | 博客 HTTP 服务、内容业务与管理员 API | [lib.rs](../../src/server/src/lib.rs) | Rust 文档注释 | 本项目实现 |
| [blog-setup](../../tools/setup/Cargo.toml) | 离线初始化、账户恢复与实例维护 | [main.rs](../../tools/setup/src/main.rs) | [README.md](../../tools/setup/README.md) | 本项目实现 |
| [blog-assets](../../tools/assets/Cargo.toml) | 静态资源编译、源码收据和开发预览 | [lib.rs](../../tools/assets/src/lib.rs) | [README.md](../../tools/assets/README.md) | 本项目实现 |
| [blog-browser](../../src/browser/Cargo.toml) | 浏览器 CSR 博客界面 | [lib.rs](../../src/browser/src/lib.rs) | Rust 文档注释 | 本项目实现 |
| [blog-browser-domain-tests](../../test/browser-domain/Cargo.toml) | 浏览器数据契约与Markdown逻辑的宿主测试 | [lib.rs](../../test/browser-domain/lib.rs) | Rust 文档注释 | 本项目实现 |
| [blog-interface](../../src/interface/Cargo.toml) | 博客组件、浏览器 API、主题和应用挂载 | [lib.rs](../../src/interface/src/lib.rs) | [README.md](../../src/interface/README.md) | 本项目实现 |
| [blog-dom](../../src/interface/dom/Cargo.toml) | 响应式 DOM 视图、组件属性和生命周期 | [lib.rs](../../src/interface/dom/src/lib.rs) | [README.md](../../src/interface/dom/README.md)<br>[src/kernel/erased_value/README.md](../../src/interface/dom/src/kernel/erased_value/README.md) | [any_spawner@0.3.0](origins/any_spawner-0.3.0/README.md)<br>[console_error_panic_hook@0.1.7](origins/console_error_panic_hook-0.1.7/README.md)<br>[drain_filter_polyfill@0.1.3](origins/drain_filter_polyfill-0.1.3/README.md)<br>[either_of@0.1.9](origins/either_of-0.1.9/README.md)<br>[erased@0.1.2](origins/erased-0.1.2/README.md)<br>[gloo-events@0.2.0](origins/gloo-events-0.2.0/README.md)<br>[gloo-timers@0.3.0](origins/gloo-timers-0.3.0/README.md)<br>[guardian@1.3.0](origins/guardian-1.3.0/README.md)<br>[itertools@0.14.0](origins/itertools-0.14.0/README.md)<br>[leptos_dom@0.8.8](origins/leptos_dom-0.8.8/README.md)<br>[next_tuple@0.1.0](origins/next_tuple-0.1.0/README.md)<br>[oco_ref@0.2.1](origins/oco_ref-0.2.1/README.md)<br>[or_poisoned@0.1.0](origins/or_poisoned-0.1.0/README.md)<br>[reactive_graph@0.2.16](origins/reactive_graph-0.2.16/README.md)<br>[send_wrapper@0.6.0](origins/send_wrapper-0.6.0/README.md)<br>[slotmap@1.1.1](origins/slotmap-1.1.1/README.md)<br>[tachys@0.2.19](origins/tachys-0.2.19/README.md)<br>[throw_error@0.3.1](origins/throw_error-0.3.1/README.md)<br>[typed-builder@0.23.2](origins/typed-builder-0.23.2/README.md) |
| [blog-component-macros](../../src/interface/macros/Cargo.toml) | CSR 组件和属性的纯语法编译宏 | [lib.rs](../../src/interface/macros/src/lib.rs) | [README.md](../../src/interface/macros/README.md) | [leptos_hot_reload@0.8.7](origins/leptos_hot_reload-0.8.7/README.md)<br>[paste@1.0.15](origins/paste-1.0.15/README.md)<br>[typed-builder-macro@0.23.2](origins/typed-builder-macro-0.23.2/README.md) |

## assets

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-assets-compiler](../../src/foundation/assets/compiler/Cargo.toml) | 宿主 JavaScript AST、语义分析、代码生成与压缩 | [lib.rs](../../src/foundation/assets/compiler/src/lib.rs) | [parts/regular_expression/src/parser/reader/string_literal_parser/README.md](../../src/foundation/assets/compiler/parts/regular_expression/src/parser/reader/string_literal_parser/README.md)<br>[parts/regular_expression/src/parser/reader/template_literal_parser/README.md](../../src/foundation/assets/compiler/parts/regular_expression/src/parser/reader/template_literal_parser/README.md) | [bytecount@0.6.9](origins/bytecount-0.6.9/README.md)<br>[cow-utils@0.1.3](origins/cow-utils-0.1.3/README.md)<br>[json-escape-simd@3.1.2](origins/json-escape-simd-3.1.2/README.md)<br>[nonmax@0.5.5](origins/nonmax-0.5.5/README.md)<br>[oxc-browserslist@5.0.1](origins/oxc-browserslist-5.0.1/README.md)<br>[oxc@0.153.0](origins/oxc-0.153.0/README.md)<br>[oxc_allocator@0.153.0](origins/oxc_allocator-0.153.0/README.md)<br>[oxc_ast@0.153.0](origins/oxc_ast-0.153.0/README.md)<br>[oxc_ast_visit@0.153.0](origins/oxc_ast_visit-0.153.0/README.md)<br>[oxc_codegen@0.153.0](origins/oxc_codegen-0.153.0/README.md)<br>[oxc_compat@0.153.0](origins/oxc_compat-0.153.0/README.md)<br>[oxc_data_structures@0.153.0](origins/oxc_data_structures-0.153.0/README.md)<br>[oxc_diagnostics@0.153.0](origins/oxc_diagnostics-0.153.0/README.md)<br>[oxc_ecmascript@0.153.0](origins/oxc_ecmascript-0.153.0/README.md)<br>[oxc_estree@0.153.0](origins/oxc_estree-0.153.0/README.md)<br>[oxc_index@5.0.0](origins/oxc_index-5.0.0/README.md)<br>[oxc_mangler@0.153.0](origins/oxc_mangler-0.153.0/README.md)<br>[oxc_minifier@0.153.0](origins/oxc_minifier-0.153.0/README.md)<br>[oxc_parser@0.153.0](origins/oxc_parser-0.153.0/README.md)<br>[oxc_regular_expression@0.153.0](origins/oxc_regular_expression-0.153.0/README.md)<br>[oxc_semantic@0.153.0](origins/oxc_semantic-0.153.0/README.md)<br>[oxc_sourcemap@9.0.0](origins/oxc_sourcemap-9.0.0/README.md)<br>[oxc_span@0.153.0](origins/oxc_span-0.153.0/README.md)<br>[oxc_str@0.153.0](origins/oxc_str-0.153.0/README.md)<br>[oxc_syntax@0.153.0](origins/oxc_syntax-0.153.0/README.md)<br>[oxc_traverse@0.153.0](origins/oxc_traverse-0.153.0/README.md)<br>[unicode-id-start@1.5.0](origins/unicode-id-start-1.5.0/README.md) |
| [blog-assets-markdown-parser](../../src/foundation/assets/markdown-parser/Cargo.toml) | markdown parser | [lib.rs](../../src/foundation/assets/markdown-parser/src/lib.rs) | Rust 文档注释 | [linkify@0.11.0](origins/linkify-0.11.0/README.md)<br>[pulldown-cmark-escape@0.11.0](origins/pulldown-cmark-escape-0.11.0/README.md)<br>[pulldown-cmark@0.13.4](origins/pulldown-cmark-0.13.4/README.md) |
| [blog-assets-style-compiler](../../src/foundation/assets/style-compiler/Cargo.toml) | 宿主 CSS 解析、选择器、浏览器兼容转换与压缩 | [lib.rs](../../src/foundation/assets/style-compiler/src/lib.rs) | [parts/style_optimizer/website/pages/bundling.md](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/bundling.md)<br>[parts/style_optimizer/website/pages/css-modules.md](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/css-modules.md)<br>[parts/style_optimizer/website/pages/docs.md](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/docs.md)<br>[parts/style_optimizer/website/pages/minification.md](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/minification.md)<br>[parts/style_optimizer/website/pages/transforms.md](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/transforms.md)<br>[parts/style_optimizer/website/pages/transpilation.md](../../src/foundation/assets/style-compiler/parts/style_optimizer/website/pages/transpilation.md) | [cssparser-color@0.1.0](origins/cssparser-color-0.1.0/README.md)<br>[cssparser@0.33.0](origins/cssparser-0.33.0/README.md)<br>[dtoa-short@0.3.5](origins/dtoa-short-0.3.5/README.md)<br>[dtoa@1.0.11](origins/dtoa-1.0.11/README.md)<br>[lightningcss@1.0.0-alpha.71](origins/lightningcss-1.0.0-alpha.71/README.md)<br>[parcel_selectors@0.28.2](origins/parcel_selectors-0.28.2/README.md)<br>[pathdiff@0.2.3](origins/pathdiff-0.2.3/README.md)<br>[precomputed-hash@0.1.1](origins/precomputed-hash-0.1.1/README.md) |
| [blog-assets-style-token-macros](../../src/foundation/assets/style-token-macros/Cargo.toml) | style token macros | [lib.rs](../../src/foundation/assets/style-token-macros/lib.rs) | Rust 文档注释 | [cssparser-macros@0.6.1](origins/cssparser-macros-0.6.1/README.md) |
| [blog-assets-style-value-derive](../../src/foundation/assets/style-value-derive/Cargo.toml) | style value derive | [lib.rs](../../src/foundation/assets/style-value-derive/src/lib.rs) | Rust 文档注释 | [lightningcss-derive@1.0.0-alpha.43](origins/lightningcss-derive-1.0.0-alpha.43/README.md) |

## checksum

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-checksum-fast-crc](../../src/foundation/checksum/fast-crc/Cargo.toml) | fast crc | [lib.rs](../../src/foundation/checksum/fast-crc/src/lib.rs) | Rust 文档注释 | [crc32fast@1.5.2](origins/crc32fast-1.5.2/README.md) |

## cli

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-cli-argument-derive](../../src/foundation/cli/argument-derive/Cargo.toml) | argument derive | [lib.rs](../../src/foundation/cli/argument-derive/src/lib.rs) | Rust 文档注释 | [clap_derive@4.6.7](origins/clap_derive-4.6.7/README.md) |
| [blog-cli-arguments](../../src/foundation/cli/arguments/Cargo.toml) | arguments | [lib.rs](../../src/foundation/cli/arguments/src/lib.rs) | Rust 文档注释 | [clap@4.6.7](origins/clap-4.6.7/README.md)<br>[clap_builder@4.6.7](origins/clap_builder-4.6.7/README.md)<br>[clap_lex@1.1.1](origins/clap_lex-1.1.1/README.md) |
| [blog-cli-terminal-format](../../src/foundation/cli/terminal-format/Cargo.toml) | terminal format | [lib.rs](../../src/foundation/cli/terminal-format/src/lib.rs) | Rust 文档注释 | [nu-ansi-term@0.50.3](origins/nu-ansi-term-0.50.3/README.md) |
| [blog-cli-terminal-style](../../src/foundation/cli/terminal-style/Cargo.toml) | 终端样式、颜色策略、ANSI解析与流输出 | [lib.rs](../../src/foundation/cli/terminal-style/src/lib.rs) | Rust 文档注释 | [anstream@1.0.0](origins/anstream-1.0.0/README.md)<br>[anstyle-parse@1.0.0](origins/anstyle-parse-1.0.0/README.md)<br>[anstyle-query@1.1.5](origins/anstyle-query-1.1.5/README.md)<br>[anstyle@1.0.14](origins/anstyle-1.0.14/README.md)<br>[colorchoice@1.0.5](origins/colorchoice-1.0.5/README.md) |

## collections

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-collections-bit-set](../../src/foundation/collections/bit-set/Cargo.toml) | bit set | [lib.rs](../../src/foundation/collections/bit-set/src/lib.rs) | Rust 文档注释 | [bitflags@2.13.2](origins/bitflags-2.13.2/README.md) |
| [blog-collections-compiler-hash](../../src/foundation/collections/compiler-hash/Cargo.toml) | compiler hash | [lib.rs](../../src/foundation/collections/compiler-hash/src/lib.rs) | Rust 文档注释 | [rustc-hash@2.1.3](origins/rustc-hash-2.1.3/README.md) |
| [blog-collections-concurrent-slab](../../src/foundation/collections/concurrent-slab/Cargo.toml) | concurrent slab | [lib.rs](../../src/foundation/collections/concurrent-slab/src/lib.rs) | Rust 文档注释 | [sharded-slab@0.1.7](origins/sharded-slab-0.1.7/README.md) |
| [blog-collections-database-hash-table](../../src/foundation/collections/database-hash-table/Cargo.toml) | database hash table | [lib.rs](../../src/foundation/collections/database-hash-table/src/lib.rs) | Rust 文档注释 | [hashbrown@0.16.1](origins/hashbrown-0.16.1/README.md) |
| [blog-collections-fast-hash](../../src/foundation/collections/fast-hash/Cargo.toml) | fast hash | [lib.rs](../../src/foundation/collections/fast-hash/src/lib.rs) | Rust 文档注释 | [foldhash@0.2.0](origins/foldhash-0.2.0/README.md) |
| [blog-collections-hash-table](../../src/foundation/collections/hash-table/Cargo.toml) | hash table | [lib.rs](../../src/foundation/collections/hash-table/src/lib.rs) | Rust 文档注释 | [hashbrown@0.17.1](origins/hashbrown-0.17.1/README.md) |
| [blog-collections-inline-string](../../src/foundation/collections/inline-string/Cargo.toml) | inline string | [lib.rs](../../src/foundation/collections/inline-string/src/lib.rs) | Rust 文档注释 | [compact_str@0.10.0](origins/compact_str-0.10.0/README.md) |
| [blog-collections-inline-vector](../../src/foundation/collections/inline-vector/Cargo.toml) | inline vector | [lib.rs](../../src/foundation/collections/inline-vector/src/lib.rs) | Rust 文档注释 | [smallvec@1.16.2](origins/smallvec-1.16.2/README.md)<br>[smallvec@1.16.3](origins/smallvec-1.16.3/README.md) |
| [blog-collections-key-equivalence](../../src/foundation/collections/key-equivalence/Cargo.toml) | key equivalence | [lib.rs](../../src/foundation/collections/key-equivalence/src/lib.rs) | Rust 文档注释 | [equivalent@1.0.2](origins/equivalent-1.0.2/README.md)<br>[equivalent@1.0.3](origins/equivalent-1.0.3/README.md) |
| [blog-collections-keyed-hash](../../src/foundation/collections/keyed-hash/Cargo.toml) | keyed hash | [lib.rs](../../src/foundation/collections/keyed-hash/src/lib.rs) | Rust 文档注释 | [ahash@0.8.12](origins/ahash-0.8.12/README.md) |
| [blog-collections-ordered-cache](../../src/foundation/collections/ordered-cache/Cargo.toml) | ordered cache | [lib.rs](../../src/foundation/collections/ordered-cache/src/lib.rs) | Rust 文档注释 | [hashlink@0.11.1](origins/hashlink-0.11.1/README.md) |
| [blog-collections-ordered-map](../../src/foundation/collections/ordered-map/Cargo.toml) | ordered map | [lib.rs](../../src/foundation/collections/ordered-map/src/lib.rs) | Rust 文档注释 | [indexmap@2.14.2](origins/indexmap-2.14.2/README.md) |
| [blog-collections-slot-storage](../../src/foundation/collections/slot-storage/Cargo.toml) | slot storage | [lib.rs](../../src/foundation/collections/slot-storage/src/lib.rs) | Rust 文档注释 | [slab@0.4.12](origins/slab-0.4.12/README.md) |
| [blog-collections-sorted-map](../../src/foundation/collections/sorted-map/Cargo.toml) | sorted map | [lib.rs](../../src/foundation/collections/sorted-map/src/lib.rs) | Rust 文档注释 | [litemap@0.8.3](origins/litemap-0.8.3/README.md) |

## crypto

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-crypto-algorithm-identifiers](../../src/foundation/crypto/algorithm-identifiers/Cargo.toml) | algorithm identifiers | [lib.rs](../../src/foundation/crypto/algorithm-identifiers/src/lib.rs) | Rust 文档注释 | [const-oid@0.10.2](origins/const-oid-0.10.2/README.md) |
| [blog-crypto-conditional-move](../../src/foundation/crypto/conditional-move/Cargo.toml) | conditional move | [lib.rs](../../src/foundation/crypto/conditional-move/src/lib.rs) | Rust 文档注释 | [cmov@0.5.4](origins/cmov-0.5.4/README.md) |
| [blog-crypto-constant-time-comparison](../../src/foundation/crypto/constant-time-comparison/Cargo.toml) | constant time comparison | [lib.rs](../../src/foundation/crypto/constant-time-comparison/src/lib.rs) | Rust 文档注释 | [subtle@2.6.1](origins/subtle-2.6.1/README.md) |
| [blog-crypto-constant-time-utilities](../../src/foundation/crypto/constant-time-utilities/Cargo.toml) | constant time utilities | [lib.rs](../../src/foundation/crypto/constant-time-utilities/src/lib.rs) | Rust 文档注释 | [ctutils@0.4.4](origins/ctutils-0.4.4/README.md) |
| [blog-crypto-digest-traits](../../src/foundation/crypto/digest-traits/Cargo.toml) | digest traits | [lib.rs](../../src/foundation/crypto/digest-traits/src/lib.rs) | Rust 文档注释 | [block-buffer@0.12.1](origins/block-buffer-0.12.1/README.md)<br>[crypto-common@0.2.2](origins/crypto-common-0.2.2/README.md)<br>[digest@0.11.3](origins/digest-0.11.3/README.md) |
| [blog-crypto-memory-hard-hash](../../src/foundation/crypto/memory-hard-hash/Cargo.toml) | memory hard hash | [lib.rs](../../src/foundation/crypto/memory-hard-hash/src/lib.rs) | Rust 文档注释 | [blake2@0.10.6](origins/blake2-0.10.6/README.md) |
| [blog-crypto-message-authentication](../../src/foundation/crypto/message-authentication/Cargo.toml) | message authentication | [lib.rs](../../src/foundation/crypto/message-authentication/src/lib.rs) | Rust 文档注释 | [hmac@0.13.0](origins/hmac-0.13.0/README.md) |
| [blog-crypto-password-algorithm-traits](../../src/foundation/crypto/password-algorithm-traits/Cargo.toml) | password algorithm traits | [lib.rs](../../src/foundation/crypto/password-algorithm-traits/src/lib.rs) | Rust 文档注释 | [crypto-common@0.1.7](origins/crypto-common-0.1.7/README.md) |
| [blog-crypto-password-derivation](../../src/foundation/crypto/password-derivation/Cargo.toml) | password derivation | [lib.rs](../../src/foundation/crypto/password-derivation/src/lib.rs) | Rust 文档注释 | [argon2@0.5.3](origins/argon2-0.5.3/README.md) |
| [blog-crypto-password-digest-traits](../../src/foundation/crypto/password-digest-traits/Cargo.toml) | password digest traits | [lib.rs](../../src/foundation/crypto/password-digest-traits/src/lib.rs) | Rust 文档注释 | [digest@0.10.7](origins/digest-0.10.7/README.md) |
| [blog-crypto-password-format](../../src/foundation/crypto/password-format/Cargo.toml) | password format | [lib.rs](../../src/foundation/crypto/password-format/src/lib.rs) | Rust 文档注释 | [password-hash@0.5.0](origins/password-hash-0.5.0/README.md) |
| [blog-crypto-password-hash-blocks](../../src/foundation/crypto/password-hash-blocks/Cargo.toml) | password hash blocks | [lib.rs](../../src/foundation/crypto/password-hash-blocks/src/lib.rs) | Rust 文档注释 | [block-buffer@0.10.4](origins/block-buffer-0.10.4/README.md) |
| [blog-crypto-secure-hash](../../src/foundation/crypto/secure-hash/Cargo.toml) | secure hash | [lib.rs](../../src/foundation/crypto/secure-hash/src/lib.rs) | [README.md](../../src/foundation/crypto/secure-hash/README.md) | [sha2@0.11.0](origins/sha2-0.11.0/README.md) |

## diagnostics

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-diagnostics-error-context](../../src/foundation/diagnostics/error-context/Cargo.toml) | error context | [lib.rs](../../src/foundation/diagnostics/error-context/src/lib.rs) | Rust 文档注释 | [anyhow@1.0.104](origins/anyhow-1.0.104/README.md) |
| [blog-diagnostics-error-display](../../src/foundation/diagnostics/error-display/Cargo.toml) | error display | [lib.rs](../../src/foundation/diagnostics/error-display/src/lib.rs) | Rust 文档注释 | [displaydoc@0.2.7](origins/displaydoc-0.2.7/README.md) |
| [blog-diagnostics-event-facade](../../src/foundation/diagnostics/event-facade/Cargo.toml) | event facade | [lib.rs](../../src/foundation/diagnostics/event-facade/src/lib.rs) | Rust 文档注释 | [log@0.4.34](origins/log-0.4.34/README.md) |
| [blog-diagnostics-event-instrumentation](../../src/foundation/diagnostics/event-instrumentation/Cargo.toml) | event instrumentation | [lib.rs](../../src/foundation/diagnostics/event-instrumentation/src/lib.rs) | Rust 文档注释 | [tracing-attributes@0.1.31](origins/tracing-attributes-0.1.31/README.md) |
| [blog-diagnostics-object-clone](../../src/foundation/diagnostics/object-clone/Cargo.toml) | object clone | [lib.rs](../../src/foundation/diagnostics/object-clone/src/lib.rs) | Rust 文档注释 | [dyn-clone@1.0.20](origins/dyn-clone-1.0.20/README.md) |
| [blog-diagnostics-structured-events](../../src/foundation/diagnostics/structured-events/Cargo.toml) | structured events | [lib.rs](../../src/foundation/diagnostics/structured-events/src/lib.rs) | Rust 文档注释 | [tracing-core@0.1.36](origins/tracing-core-0.1.36/README.md)<br>[tracing-log@0.2.0](origins/tracing-log-0.2.0/README.md)<br>[tracing-subscriber@0.3.23](origins/tracing-subscriber-0.3.23/README.md)<br>[tracing@0.1.44](origins/tracing-0.1.44/README.md) |
| [blog-diagnostics-symbol-decoder](../../src/foundation/diagnostics/symbol-decoder/Cargo.toml) | symbol decoder | [lib.rs](../../src/foundation/diagnostics/symbol-decoder/src/lib.rs) | Rust 文档注释 | [rustc-demangle@0.1.28](origins/rustc-demangle-0.1.28/README.md) |
| [blog-diagnostics-typed-error](../../src/foundation/diagnostics/typed-error/Cargo.toml) | typed error | [lib.rs](../../src/foundation/diagnostics/typed-error/src/lib.rs) | Rust 文档注释 | [thiserror@2.0.21](origins/thiserror-2.0.21/README.md) |
| [blog-diagnostics-typed-error-derive](../../src/foundation/diagnostics/typed-error-derive/Cargo.toml) | typed error derive | [lib.rs](../../src/foundation/diagnostics/typed-error-derive/src/lib.rs) | Rust 文档注释 | [thiserror-impl@2.0.21](origins/thiserror-impl-2.0.21/README.md) |

## encoding

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-encoding-application-base-encoding](../../src/foundation/encoding/application-base-encoding/Cargo.toml) | application base encoding | [lib.rs](../../src/foundation/encoding/application-base-encoding/src/lib.rs) | Rust 文档注释 | [base64@0.23.1](origins/base64-0.23.1/README.md) |
| [blog-encoding-borrowed-unicode](../../src/foundation/encoding/borrowed-unicode/Cargo.toml) | borrowed unicode | [lib.rs](../../src/foundation/encoding/borrowed-unicode/src/lib.rs) | Rust 文档注释 | [potential_utf@0.1.6](origins/potential_utf-0.1.6/README.md) |
| [blog-encoding-byte-buffer](../../src/foundation/encoding/byte-buffer/Cargo.toml) | byte buffer | [lib.rs](../../src/foundation/encoding/byte-buffer/src/lib.rs) | Rust 文档注释 | [bytes@1.12.1](origins/bytes-1.12.1/README.md) |
| [blog-encoding-byte-search](../../src/foundation/encoding/byte-search/Cargo.toml) | byte search | [lib.rs](../../src/foundation/encoding/byte-search/src/lib.rs) | Rust 文档注释 | [memchr@2.8.3](origins/memchr-2.8.3/README.md) |
| [blog-encoding-compression](../../src/foundation/encoding/compression/Cargo.toml) | DEFLATE、gzip与zlib流和Adler校验 | [lib.rs](../../src/foundation/encoding/compression/src/lib.rs) | Rust 文档注释 | [adler2@2.0.1](origins/adler2-2.0.1/README.md)<br>[flate2@1.1.10](origins/flate2-1.1.10/README.md)<br>[miniz_oxide@0.9.1](origins/miniz_oxide-0.9.1/README.md)<br>[simd-adler32@0.3.10](origins/simd-adler32-0.3.10/README.md) |
| [blog-encoding-constant-time-base-encoding](../../src/foundation/encoding/constant-time-base-encoding/Cargo.toml) | constant time base encoding | [lib.rs](../../src/foundation/encoding/constant-time-base-encoding/src/lib.rs) | Rust 文档注释 | [base64ct@1.8.3](origins/base64ct-1.8.3/README.md) |
| [blog-encoding-decimal-float-format](../../src/foundation/encoding/decimal-float-format/Cargo.toml) | decimal float format | [lib.rs](../../src/foundation/encoding/decimal-float-format/src/lib.rs) | Rust 文档注释 | [zmij@1.0.23](origins/zmij-1.0.23/README.md) |
| [blog-encoding-framing](../../src/foundation/encoding/framing/Cargo.toml) | framing | [lib.rs](../../src/foundation/encoding/framing/src/lib.rs) | Rust 文档注释 | [cobs@0.3.0](origins/cobs-0.3.0/README.md) |
| [blog-encoding-hex-codec](../../src/foundation/encoding/hex-codec/Cargo.toml) | hex codec | [lib.rs](../../src/foundation/encoding/hex-codec/src/lib.rs) | Rust 文档注释 | [hex@0.4.3](origins/hex-0.4.3/README.md) |
| [blog-encoding-integer-format](../../src/foundation/encoding/integer-format/Cargo.toml) | integer format | [lib.rs](../../src/foundation/encoding/integer-format/src/lib.rs) | Rust 文档注释 | [itoa@1.0.18](origins/itoa-1.0.18/README.md) |
| [blog-encoding-radix-codecs](../../src/foundation/encoding/radix-codecs/Cargo.toml) | radix codecs | [lib.rs](../../src/foundation/encoding/radix-codecs/src/lib.rs) | Rust 文档注释 | [data-encoding@2.11.1](origins/data-encoding-2.11.1/README.md) |
| [blog-encoding-roundtrip-float-format](../../src/foundation/encoding/roundtrip-float-format/Cargo.toml) | roundtrip float format | [lib.rs](../../src/foundation/encoding/roundtrip-float-format/src/lib.rs) | Rust 文档注释 | [ryu@1.0.23](origins/ryu-1.0.23/README.md) |
| [blog-encoding-script-float-format](../../src/foundation/encoding/script-float-format/Cargo.toml) | script float format | [lib.rs](../../src/foundation/encoding/script-float-format/src/lib.rs) | Rust 文档注释 | [dragonbox_ecma@0.1.12](origins/dragonbox_ecma-0.1.12/README.md) |
| [blog-encoding-signed-variable-integer](../../src/foundation/encoding/signed-variable-integer/Cargo.toml) | signed variable integer | [lib.rs](../../src/foundation/encoding/signed-variable-integer/src/lib.rs) | Rust 文档注释 | [leb128@0.2.7](origins/leb128-0.2.7/README.md) |
| [blog-encoding-text-writer](../../src/foundation/encoding/text-writer/Cargo.toml) | text writer | [lib.rs](../../src/foundation/encoding/text-writer/src/lib.rs) | Rust 文档注释 | [writeable@0.6.4](origins/writeable-0.6.4/README.md) |
| [blog-encoding-unicode-decoder](../../src/foundation/encoding/unicode-decoder/Cargo.toml) | unicode decoder | [lib.rs](../../src/foundation/encoding/unicode-decoder/src/lib.rs) | Rust 文档注释 | [utf8parse@0.2.2](origins/utf8parse-0.2.2/README.md) |
| [blog-encoding-unicode-iterator](../../src/foundation/encoding/unicode-iterator/Cargo.toml) | unicode iterator | [lib.rs](../../src/foundation/encoding/unicode-iterator/src/lib.rs) | Rust 文档注释 | [utf8_iter@1.0.4](origins/utf8_iter-1.0.4/README.md) |
| [blog-encoding-variable-integer-writer](../../src/foundation/encoding/variable-integer-writer/Cargo.toml) | variable integer writer | [lib.rs](../../src/foundation/encoding/variable-integer-writer/src/lib.rs) | Rust 文档注释 | [leb128fmt@0.1.0](origins/leb128fmt-0.1.0/README.md) |
| [blog-encoding-vector-base-encoding](../../src/foundation/encoding/vector-base-encoding/Cargo.toml) | vector base encoding | [lib.rs](../../src/foundation/encoding/vector-base-encoding/src/lib.rs) | Rust 文档注释 | [base64-simd@0.8.0](origins/base64-simd-0.8.0/README.md) |
| [blog-encoding-vector-escaping](../../src/foundation/encoding/vector-escaping/Cargo.toml) | vector escaping | [lib.rs](../../src/foundation/encoding/vector-escaping/src/lib.rs) | Rust 文档注释 | [escape-simd@0.1.0](origins/escape-simd-0.1.0/README.md) |
| [blog-encoding-vector-primitives](../../src/foundation/encoding/vector-primitives/Cargo.toml) | vector primitives | [lib.rs](../../src/foundation/encoding/vector-primitives/src/lib.rs) | Rust 文档注释 | [vsimd@0.8.0](origins/vsimd-0.8.0/README.md) |

## filesystem

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-filesystem-temporary-storage](../../src/foundation/filesystem/temporary-storage/Cargo.toml) | temporary storage | [lib.rs](../../src/foundation/filesystem/temporary-storage/src/lib.rs) | Rust 文档注释 | [tempfile@3.27.0](origins/tempfile-3.27.0/README.md) |

## format

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-format-binary-record](../../src/foundation/format/binary-record/Cargo.toml) | binary record | [lib.rs](../../src/foundation/format/binary-record/src/lib.rs) | Rust 文档注释 | [postcard@1.1.3](origins/postcard-1.1.3/README.md) |
| [blog-format-configuration](../../src/foundation/format/configuration/Cargo.toml) | TOML配置解析、保留格式编辑、字节跨度与日期类型 | [lib.rs](../../src/foundation/format/configuration/src/lib.rs) | [retained/docs/components/toml/datetime.md](../../src/foundation/format/configuration/retained/docs/components/toml/datetime.md)<br>[retained/docs/components/toml/editor.md](../../src/foundation/format/configuration/retained/docs/components/toml/editor.md)<br>[retained/docs/components/toml/overview.md](../../src/foundation/format/configuration/retained/docs/components/toml/overview.md)<br>[retained/docs/components/toml/span.md](../../src/foundation/format/configuration/retained/docs/components/toml/span.md)<br>[retained/docs/components/toml/writer.md](../../src/foundation/format/configuration/retained/docs/components/toml/writer.md) | [serde_spanned@0.6.9](origins/serde_spanned-0.6.9/README.md)<br>[toml@0.8.23](origins/toml-0.8.23/README.md)<br>[toml_datetime@0.6.11](origins/toml_datetime-0.6.11/README.md)<br>[toml_edit@0.22.27](origins/toml_edit-0.22.27/README.md)<br>[toml_write@0.1.2](origins/toml_write-0.1.2/README.md) |
| [blog-format-data-model](../../src/foundation/format/data-model/Cargo.toml) | data model | [lib.rs](../../src/foundation/format/data-model/src/lib.rs) | Rust 文档注释 | [serde@1.0.229](origins/serde-1.0.229/README.md)<br>[serde_core@1.0.229](origins/serde_core-1.0.229/README.md)<br>[serde_path_to_error@0.1.20](origins/serde_path_to_error-0.1.20/README.md) |
| [blog-format-data-model-derive](../../src/foundation/format/data-model-derive/Cargo.toml) | data model derive | [lib.rs](../../src/foundation/format/data-model-derive/src/lib.rs) | Rust 文档注释 | [serde_derive@1.0.229](origins/serde_derive-1.0.229/README.md) |
| [blog-format-data-model-syntax](../../src/foundation/format/data-model-syntax/Cargo.toml) | data model syntax | [lib.rs](../../src/foundation/format/data-model-syntax/lib.rs) | Rust 文档注释 | [serde_derive_internals@0.30.0](origins/serde_derive_internals-0.30.0/README.md) |
| [blog-format-date-time](../../src/foundation/format/date-time/Cargo.toml) | date time | [lib.rs](../../src/foundation/format/date-time/src/lib.rs) | [README.md](../../src/foundation/format/date-time/README.md) | [chrono@0.4.45](origins/chrono-0.4.45/README.md) |
| [blog-format-json](../../src/foundation/format/json/Cargo.toml) | json | [lib.rs](../../src/foundation/format/json/src/lib.rs) | Rust 文档注释 | [serde_json@1.0.151](origins/serde_json-1.0.151/README.md)<br>[serde_json@1.0.154](origins/serde_json-1.0.154/README.md) |
| [blog-format-json-schema](../../src/foundation/format/json-schema/Cargo.toml) | json schema | [lib.rs](../../src/foundation/format/json-schema/src/lib.rs) | Rust 文档注释 | [schemars@1.2.3](origins/schemars-1.2.3/README.md) |
| [blog-format-json-schema-derive](../../src/foundation/format/json-schema-derive/Cargo.toml) | json schema derive | [lib.rs](../../src/foundation/format/json-schema-derive/src/lib.rs) | Rust 文档注释 | [schemars_derive@1.2.3](origins/schemars_derive-1.2.3/README.md) |
| [blog-format-parser-combinators](../../src/foundation/format/parser-combinators/Cargo.toml) | parser combinators | [lib.rs](../../src/foundation/format/parser-combinators/src/lib.rs) | Rust 文档注释 | [winnow@0.7.15](origins/winnow-0.7.15/README.md) |
| [blog-format-release-identity](../../src/foundation/format/release-identity/Cargo.toml) | release identity | [lib.rs](../../src/foundation/format/release-identity/src/lib.rs) | Rust 文档注释 | [semver@1.0.28](origins/semver-1.0.28/README.md) |

## interface

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-interface-browser-api](../../src/foundation/interface/browser-api/Cargo.toml) | browser api | [lib.rs](../../src/foundation/interface/browser-api/src/lib.rs) | Rust 文档注释 | [web-sys@0.3.106](origins/web-sys-0.3.106/README.md) |
| [blog-interface-javascript-api](../../src/foundation/interface/javascript-api/Cargo.toml) | javascript api | [lib.rs](../../src/foundation/interface/javascript-api/src/lib.rs) | Rust 文档注释 | [js-sys@0.3.106](origins/js-sys-0.3.106/README.md) |

## javascript

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-javascript-syntax-derive](../../src/foundation/javascript/syntax-derive/Cargo.toml) | syntax derive | [lib.rs](../../src/foundation/javascript/syntax-derive/src/lib.rs) | Rust 文档注释 | [oxc_ast_macros@0.153.0](origins/oxc_ast_macros-0.153.0/README.md) |

## lookup

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-lookup-script-table](../../src/foundation/lookup/script-table/Cargo.toml) | script table | [lib.rs](../../src/foundation/lookup/script-table/src/lib.rs) | Rust 文档注释 | [phf@0.14.0](origins/phf-0.14.0/README.md) |
| [blog-lookup-script-table-generator](../../src/foundation/lookup/script-table-generator/Cargo.toml) | script table generator | [lib.rs](../../src/foundation/lookup/script-table-generator/src/lib.rs) | Rust 文档注释 | [phf_generator@0.14.0](origins/phf_generator-0.14.0/README.md) |
| [blog-lookup-script-table-hashing](../../src/foundation/lookup/script-table-hashing/Cargo.toml) | script table hashing | [lib.rs](../../src/foundation/lookup/script-table-hashing/src/lib.rs) | Rust 文档注释 | [phf_shared@0.14.0](origins/phf_shared-0.14.0/README.md) |
| [blog-lookup-script-table-macros](../../src/foundation/lookup/script-table-macros/Cargo.toml) | script table macros | [lib.rs](../../src/foundation/lookup/script-table-macros/src/lib.rs) | Rust 文档注释 | [phf_macros@0.14.0](origins/phf_macros-0.14.0/README.md) |
| [blog-lookup-style-table](../../src/foundation/lookup/style-table/Cargo.toml) | style table | [lib.rs](../../src/foundation/lookup/style-table/src/lib.rs) | Rust 文档注释 | [phf@0.11.3](origins/phf-0.11.3/README.md) |
| [blog-lookup-style-table-generator](../../src/foundation/lookup/style-table-generator/Cargo.toml) | style table generator | [lib.rs](../../src/foundation/lookup/style-table-generator/src/lib.rs) | Rust 文档注释 | [phf_generator@0.11.3](origins/phf_generator-0.11.3/README.md) |
| [blog-lookup-style-table-hashing](../../src/foundation/lookup/style-table-hashing/Cargo.toml) | style table hashing | [lib.rs](../../src/foundation/lookup/style-table-hashing/src/lib.rs) | Rust 文档注释 | [phf_shared@0.11.3](origins/phf_shared-0.11.3/README.md) |
| [blog-lookup-style-table-macros](../../src/foundation/lookup/style-table-macros/Cargo.toml) | style table macros | [lib.rs](../../src/foundation/lookup/style-table-macros/src/lib.rs) | Rust 文档注释 | [phf_macros@0.11.3](origins/phf_macros-0.11.3/README.md) |

## math

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-math-constant-array](../../src/foundation/math/constant-array/Cargo.toml) | constant array | [lib.rs](../../src/foundation/math/constant-array/src/lib.rs) | Rust 文档注释 | [hybrid-array@0.4.15](origins/hybrid-array-0.4.15/README.md) |
| [blog-math-fast-random](../../src/foundation/math/fast-random/Cargo.toml) | fast random | [lib.rs](../../src/foundation/math/fast-random/src/lib.rs) | Rust 文档注释 | [fastrand@2.5.0](origins/fastrand-2.5.0/README.md) |
| [blog-math-integer-traits](../../src/foundation/math/integer-traits/Cargo.toml) | integer traits | [lib.rs](../../src/foundation/math/integer-traits/src/lib.rs) | Rust 文档注释 | [num-integer@0.1.47](origins/num-integer-0.1.47/README.md) |
| [blog-math-keyed-fingerprint](../../src/foundation/math/keyed-fingerprint/Cargo.toml) | keyed fingerprint | [lib.rs](../../src/foundation/math/keyed-fingerprint/src/lib.rs) | Rust 文档注释 | [siphasher@1.0.4](origins/siphasher-1.0.4/README.md) |
| [blog-math-large-integer](../../src/foundation/math/large-integer/Cargo.toml) | large integer | [lib.rs](../../src/foundation/math/large-integer/src/lib.rs) | Rust 文档注释 | [num-bigint@0.5.1](origins/num-bigint-0.5.1/README.md) |
| [blog-math-lookup-random-generator](../../src/foundation/math/lookup-random-generator/Cargo.toml) | lookup random generator | [lib.rs](../../src/foundation/math/lookup-random-generator/src/lib.rs) | Rust 文档注释 | [rand@0.8.8](origins/rand-0.8.8/README.md) |
| [blog-math-numeric-traits](../../src/foundation/math/numeric-traits/Cargo.toml) | numeric traits | [lib.rs](../../src/foundation/math/numeric-traits/src/lib.rs) | Rust 文档注释 | [num-traits@0.2.19](origins/num-traits-0.2.19/README.md) |
| [blog-math-password-random-traits](../../src/foundation/math/password-random-traits/Cargo.toml) | password random traits | [lib.rs](../../src/foundation/math/password-random-traits/src/lib.rs) | Rust 文档注释 | [rand_core@0.6.4](origins/rand_core-0.6.4/README.md) |
| [blog-math-type-integers](../../src/foundation/math/type-integers/Cargo.toml) | type integers | [lib.rs](../../src/foundation/math/type-integers/src/lib.rs) | Rust 文档注释 | [typenum@1.20.1](origins/typenum-1.20.1/README.md) |
| [blog-math-typed-array](../../src/foundation/math/typed-array/Cargo.toml) | typed array | [lib.rs](../../src/foundation/math/typed-array/src/lib.rs) | Rust 文档注释 | [generic-array@0.14.7](origins/generic-array-0.14.7/README.md) |

## memory

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-memory-allocator](../../src/foundation/memory/allocator/Cargo.toml) | allocator | [lib.rs](../../src/foundation/memory/allocator/src/lib.rs) | Rust 文档注释 | [allocator-api2@0.2.21](origins/allocator-api2-0.2.21/README.md) |
| [blog-memory-arena](../../src/foundation/memory/arena/Cargo.toml) | arena | [lib.rs](../../src/foundation/memory/arena/src/lib.rs) | Rust 文档注释 | [bumpalo@3.20.3](origins/bumpalo-3.20.3/README.md) |
| [blog-memory-byte-layout](../../src/foundation/memory/byte-layout/Cargo.toml) | byte layout | [lib.rs](../../src/foundation/memory/byte-layout/src/lib.rs) | Rust 文档注释 | [zerocopy@0.8.63](origins/zerocopy-0.8.63/README.md) |
| [blog-memory-indexed-arena](../../src/foundation/memory/indexed-arena/Cargo.toml) | indexed arena | [lib.rs](../../src/foundation/memory/indexed-arena/src/lib.rs) | Rust 文档注释 | [id-arena@2.3.0](origins/id-arena-2.3.0/README.md) |
| [blog-memory-output-reference](../../src/foundation/memory/output-reference/Cargo.toml) | output reference | [lib.rs](../../src/foundation/memory/output-reference/src/lib.rs) | Rust 文档注释 | [outref@0.5.2](origins/outref-0.5.2/README.md) |
| [blog-memory-owned-borrow](../../src/foundation/memory/owned-borrow/Cargo.toml) | owned borrow | [lib.rs](../../src/foundation/memory/owned-borrow/src/lib.rs) | Rust 文档注释 | [self_cell@1.3.0](origins/self_cell-1.3.0/README.md) |
| [blog-memory-reference-cast](../../src/foundation/memory/reference-cast/Cargo.toml) | reference cast | [lib.rs](../../src/foundation/memory/reference-cast/src/lib.rs) | Rust 文档注释 | [ref-cast@1.0.27](origins/ref-cast-1.0.27/README.md) |
| [blog-memory-reference-cast-derive](../../src/foundation/memory/reference-cast-derive/Cargo.toml) | reference cast derive | [lib.rs](../../src/foundation/memory/reference-cast-derive/src/lib.rs) | Rust 文档注释 | [ref-cast-impl@1.0.27](origins/ref-cast-impl-1.0.27/README.md) |
| [blog-memory-stable-address](../../src/foundation/memory/stable-address/Cargo.toml) | stable address | [lib.rs](../../src/foundation/memory/stable-address/src/lib.rs) | Rust 文档注释 | [stable_deref_trait@1.2.1](origins/stable_deref_trait-1.2.1/README.md) |
| [blog-memory-type-specialization](../../src/foundation/memory/type-specialization/Cargo.toml) | type specialization | [lib.rs](../../src/foundation/memory/type-specialization/src/lib.rs) | Rust 文档注释 | [castaway@0.2.4](origins/castaway-0.2.4/README.md) |

## native

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-native-c-compiler](../../src/foundation/native/c-compiler/Cargo.toml) | c compiler | [lib.rs](../../src/foundation/native/c-compiler/src/lib.rs) | Rust 文档注释 | [cc@1.6.0](origins/cc-1.6.0/README.md) |
| [blog-native-compiler-capabilities](../../src/foundation/native/compiler-capabilities/Cargo.toml) | compiler capabilities | [lib.rs](../../src/foundation/native/compiler-capabilities/src/lib.rs) | Rust 文档注释 | [autocfg@1.5.1](origins/autocfg-1.5.1/README.md) |
| [blog-native-compiler-conditions](../../src/foundation/native/compiler-conditions/Cargo.toml) | compiler conditions | [lib.rs](../../src/foundation/native/compiler-conditions/src/lib.rs) | Rust 文档注释 | [rustversion@1.0.23](origins/rustversion-1.0.23/README.md) |
| [blog-native-target-conditions](../../src/foundation/native/target-conditions/Cargo.toml) | target conditions | [lib.rs](../../src/foundation/native/target-conditions/src/lib.rs) | Rust 文档注释 | [cfg-if@1.0.5](origins/cfg-if-1.0.5/README.md) |

## network

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-network-body-stream](../../src/foundation/network/body-stream/Cargo.toml) | body stream | [lib.rs](../../src/foundation/network/body-stream/src/lib.rs) | Rust 文档注释 | [http-body-util@0.1.5](origins/http-body-util-0.1.5/README.md)<br>[http-body@1.1.0](origins/http-body-1.1.0/README.md) |
| [blog-network-byte-ranges](../../src/foundation/network/byte-ranges/Cargo.toml) | byte ranges | [lib.rs](../../src/foundation/network/byte-ranges/src/lib.rs) | Rust 文档注释 | [http-range-header@0.4.2](origins/http-range-header-0.4.2/README.md) |
| [blog-network-form-data](../../src/foundation/network/form-data/Cargo.toml) | form data | [lib.rs](../../src/foundation/network/form-data/src/lib.rs) | Rust 文档注释 | [serde_urlencoded@0.7.1](origins/serde_urlencoded-0.7.1/README.md) |
| [blog-network-http-date](../../src/foundation/network/http-date/Cargo.toml) | http date | [lib.rs](../../src/foundation/network/http-date/src/lib.rs) | Rust 文档注释 | [httpdate@1.0.3](origins/httpdate-1.0.3/README.md) |
| [blog-network-http-message](../../src/foundation/network/http-message/Cargo.toml) | http message | [lib.rs](../../src/foundation/network/http-message/src/lib.rs) | Rust 文档注释 | [http@1.5.0](origins/http-1.5.0/README.md) |
| [blog-network-http-middleware](../../src/foundation/network/http-middleware/Cargo.toml) | http middleware | [lib.rs](../../src/foundation/network/http-middleware/src/lib.rs) | Rust 文档注释 | [tower-http@0.6.11](origins/tower-http-0.6.11/README.md) |
| [blog-network-http-parser](../../src/foundation/network/http-parser/Cargo.toml) | http parser | [lib.rs](../../src/foundation/network/http-parser/src/lib.rs) | Rust 文档注释 | [httparse@1.10.1](origins/httparse-1.10.1/README.md) |
| [blog-network-http-transport](../../src/foundation/network/http-transport/Cargo.toml) | http transport | [lib.rs](../../src/foundation/network/http-transport/src/lib.rs) | Rust 文档注释 | [hyper-util@0.1.21](origins/hyper-util-0.1.21/README.md)<br>[hyper@1.12.0](origins/hyper-1.12.0/README.md) |
| [blog-network-media-detection](../../src/foundation/network/media-detection/Cargo.toml) | media detection | [lib.rs](../../src/foundation/network/media-detection/src/lib.rs) | Rust 文档注释 | [mime_guess@2.0.5](origins/mime_guess-2.0.5/README.md) |
| [blog-network-media-type](../../src/foundation/network/media-type/Cargo.toml) | media type | [lib.rs](../../src/foundation/network/media-type/src/lib.rs) | Rust 文档注释 | [mime@0.3.17](origins/mime-0.3.17/README.md) |
| [blog-network-request-pipeline](../../src/foundation/network/request-pipeline/Cargo.toml) | request pipeline | [lib.rs](../../src/foundation/network/request-pipeline/src/lib.rs) | Rust 文档注释 | [tower-layer@0.3.3](origins/tower-layer-0.3.3/README.md)<br>[tower-service@0.3.3](origins/tower-service-0.3.3/README.md)<br>[tower@0.5.3](origins/tower-0.5.3/README.md) |
| [blog-network-request-router](../../src/foundation/network/request-router/Cargo.toml) | request router | [lib.rs](../../src/foundation/network/request-router/src/lib.rs) | [src/docs/debugging_handler_type_errors.md](../../src/foundation/network/request-router/src/docs/debugging_handler_type_errors.md)<br>[src/docs/error_handling.md](../../src/foundation/network/request-router/src/docs/error_handling.md)<br>[src/docs/extract.md](../../src/foundation/network/request-router/src/docs/extract.md)<br>[src/docs/handlers_intro.md](../../src/foundation/network/request-router/src/docs/handlers_intro.md)<br>[src/docs/method_routing/fallback.md](../../src/foundation/network/request-router/src/docs/method_routing/fallback.md)<br>[src/docs/method_routing/layer.md](../../src/foundation/network/request-router/src/docs/method_routing/layer.md)<br>[src/docs/method_routing/merge.md](../../src/foundation/network/request-router/src/docs/method_routing/merge.md)<br>[src/docs/method_routing/route_layer.md](../../src/foundation/network/request-router/src/docs/method_routing/route_layer.md)<br>[src/docs/middleware.md](../../src/foundation/network/request-router/src/docs/middleware.md)<br>[src/docs/response.md](../../src/foundation/network/request-router/src/docs/response.md)<br>[src/docs/routing/fallback.md](../../src/foundation/network/request-router/src/docs/routing/fallback.md)<br>[src/docs/routing/into_make_service_with_connect_info.md](../../src/foundation/network/request-router/src/docs/routing/into_make_service_with_connect_info.md)<br>[src/docs/routing/layer.md](../../src/foundation/network/request-router/src/docs/routing/layer.md)<br>[src/docs/routing/merge.md](../../src/foundation/network/request-router/src/docs/routing/merge.md)<br>[src/docs/routing/method_not_allowed_fallback.md](../../src/foundation/network/request-router/src/docs/routing/method_not_allowed_fallback.md)<br>[src/docs/routing/nest.md](../../src/foundation/network/request-router/src/docs/routing/nest.md)<br>[src/docs/routing/route.md](../../src/foundation/network/request-router/src/docs/routing/route.md)<br>[src/docs/routing/route_layer.md](../../src/foundation/network/request-router/src/docs/routing/route_layer.md)<br>[src/docs/routing/route_service.md](../../src/foundation/network/request-router/src/docs/routing/route_service.md)<br>[src/docs/routing/with_state.md](../../src/foundation/network/request-router/src/docs/routing/with_state.md)<br>[src/docs/routing/without_v07_checks.md](../../src/foundation/network/request-router/src/docs/routing/without_v07_checks.md) | [axum-core@0.5.6](origins/axum-core-0.5.6/README.md)<br>[axum@0.8.9](origins/axum-0.8.9/README.md) |
| [blog-network-route-matcher](../../src/foundation/network/route-matcher/Cargo.toml) | route matcher | [lib.rs](../../src/foundation/network/route-matcher/src/lib.rs) | Rust 文档注释 | [matchit@0.8.4](origins/matchit-0.8.4/README.md) |
| [blog-network-web-address](../../src/foundation/network/web-address/Cargo.toml) | web address | [lib.rs](../../src/foundation/network/web-address/src/lib.rs) | Rust 文档注释 | [form_urlencoded@1.2.2](origins/form_urlencoded-1.2.2/README.md)<br>[idna@1.1.0](origins/idna-1.1.0/README.md)<br>[idna_adapter@1.2.2](origins/idna_adapter-1.2.2/README.md)<br>[percent-encoding@2.3.2](origins/percent-encoding-2.3.2/README.md)<br>[url@2.5.8](origins/url-2.5.8/README.md) |

## platform

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-platform-error-code](../../src/foundation/platform/error-code/Cargo.toml) | error code | [lib.rs](../../src/foundation/platform/error-code/src/lib.rs) | Rust 文档注释 | [errno@0.3.14](origins/errno-0.3.14/README.md) |
| [blog-platform-linux-abi](../../src/foundation/platform/linux-abi/Cargo.toml) | linux abi | [lib.rs](../../src/foundation/platform/linux-abi/src/lib.rs) | Rust 文档注释 | [linux-raw-sys@0.12.1](origins/linux-raw-sys-0.12.1/README.md) |
| [blog-platform-local-time-zone](../../src/foundation/platform/local-time-zone/Cargo.toml) | local time zone | [lib.rs](../../src/foundation/platform/local-time-zone/src/lib.rs) | Rust 文档注释 | [iana-time-zone@0.1.65](origins/iana-time-zone-0.1.65/README.md) |
| [blog-platform-password-processor-features](../../src/foundation/platform/password-processor-features/Cargo.toml) | password processor features | [lib.rs](../../src/foundation/platform/password-processor-features/src/lib.rs) | Rust 文档注释 | [cpufeatures@0.2.17](origins/cpufeatures-0.2.17/README.md) |
| [blog-platform-processor-features](../../src/foundation/platform/processor-features/Cargo.toml) | processor features | [lib.rs](../../src/foundation/platform/processor-features/src/lib.rs) | Rust 文档注释 | [cpufeatures@0.3.1](origins/cpufeatures-0.3.1/README.md) |
| [blog-platform-socket-options](../../src/foundation/platform/socket-options/Cargo.toml) | socket options | [lib.rs](../../src/foundation/platform/socket-options/src/lib.rs) | Rust 文档注释 | [socket2@0.6.5](origins/socket2-0.6.5/README.md) |
| [blog-platform-system-abi](../../src/foundation/platform/system-abi/Cargo.toml) | system abi | [lib.rs](../../src/foundation/platform/system-abi/src/lib.rs) | Rust 文档注释 | [libc@0.2.190](origins/libc-0.2.190/README.md)<br>[libc@0.2.191](origins/libc-0.2.191/README.md) |
| [blog-platform-system-calls](../../src/foundation/platform/system-calls/Cargo.toml) | system calls | [lib.rs](../../src/foundation/platform/system-calls/src/lib.rs) | Rust 文档注释 | [rustix@1.1.5](origins/rustix-1.1.5/README.md) |
| [blog-platform-system-entropy](../../src/foundation/platform/system-entropy/Cargo.toml) | system entropy | [lib.rs](../../src/foundation/platform/system-entropy/src/lib.rs) | [README.md](../../src/foundation/platform/system-entropy/README.md) | [getrandom@0.4.3](origins/getrandom-0.4.3/README.md) |

## runtime

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-runtime-async-toolkit](../../src/foundation/runtime/async-toolkit/Cargo.toml) | async toolkit | [lib.rs](../../src/foundation/runtime/async-toolkit/src/lib.rs) | Rust 文档注释 | [futures-channel@0.3.34](origins/futures-channel-0.3.34/README.md)<br>[futures-core@0.3.34](origins/futures-core-0.3.34/README.md)<br>[futures-executor@0.3.34](origins/futures-executor-0.3.34/README.md)<br>[futures-io@0.3.34](origins/futures-io-0.3.34/README.md)<br>[futures-sink@0.3.34](origins/futures-sink-0.3.34/README.md)<br>[futures-task@0.3.34](origins/futures-task-0.3.34/README.md)<br>[futures-util@0.3.34](origins/futures-util-0.3.34/README.md)<br>[futures@0.3.34](origins/futures-0.3.34/README.md) |
| [blog-runtime-future-macros](../../src/foundation/runtime/future-macros/Cargo.toml) | future macros | [lib.rs](../../src/foundation/runtime/future-macros/src/lib.rs) | Rust 文档注释 | [futures-macro@0.3.34](origins/futures-macro-0.3.34/README.md) |
| [blog-runtime-io-poller](../../src/foundation/runtime/io-poller/Cargo.toml) | io poller | [lib.rs](../../src/foundation/runtime/io-poller/src/lib.rs) | Rust 文档注释 | [mio@1.2.4](origins/mio-1.2.4/README.md) |
| [blog-runtime-parallel-iterator](../../src/foundation/runtime/parallel-iterator/Cargo.toml) | parallel iterator | [lib.rs](../../src/foundation/runtime/parallel-iterator/src/lib.rs) | [src/iter/plumbing/README.md](../../src/foundation/runtime/parallel-iterator/src/iter/plumbing/README.md) | [rayon@1.12.0](origins/rayon-1.12.0/README.md) |
| [blog-runtime-readiness-signal](../../src/foundation/runtime/readiness-signal/Cargo.toml) | readiness signal | [lib.rs](../../src/foundation/runtime/readiness-signal/src/lib.rs) | Rust 文档注释 | [want@0.3.2](origins/want-0.3.2/README.md) |
| [blog-runtime-scheduled-io](../../src/foundation/runtime/scheduled-io/Cargo.toml) | scheduled io | [lib.rs](../../src/foundation/runtime/scheduled-io/src/lib.rs) | Rust 文档注释 | [tokio-util@0.7.19](origins/tokio-util-0.7.19/README.md)<br>[tokio-util@0.7.20](origins/tokio-util-0.7.20/README.md) |
| [blog-runtime-signal-registry](../../src/foundation/runtime/signal-registry/Cargo.toml) | signal registry | [lib.rs](../../src/foundation/runtime/signal-registry/src/lib.rs) | Rust 文档注释 | [signal-hook-registry@1.4.8](origins/signal-hook-registry-1.4.8/README.md) |
| [blog-runtime-task-entry](../../src/foundation/runtime/task-entry/Cargo.toml) | task entry | [lib.rs](../../src/foundation/runtime/task-entry/src/lib.rs) | Rust 文档注释 | [tokio-macros@2.7.2](origins/tokio-macros-2.7.2/README.md) |
| [blog-runtime-task-scheduler](../../src/foundation/runtime/task-scheduler/Cargo.toml) | task scheduler | [lib.rs](../../src/foundation/runtime/task-scheduler/src/lib.rs) | Rust 文档注释 | [tokio@1.53.2](origins/tokio-1.53.2/README.md) |
| [blog-runtime-task-waker](../../src/foundation/runtime/task-waker/Cargo.toml) | task waker | [lib.rs](../../src/foundation/runtime/task-waker/src/lib.rs) | Rust 文档注释 | [atomic-waker@1.1.2](origins/atomic-waker-1.1.2/README.md) |
| [blog-runtime-worker-pool](../../src/foundation/runtime/worker-pool/Cargo.toml) | worker pool | [lib.rs](../../src/foundation/runtime/worker-pool/src/lib.rs) | [src/sleep/README.md](../../src/foundation/runtime/worker-pool/src/sleep/README.md) | [rayon-core@1.13.0](origins/rayon-core-1.13.0/README.md) |

## storage

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-storage-fallible-rows](../../src/foundation/storage/fallible-rows/Cargo.toml) | fallible rows | [lib.rs](../../src/foundation/storage/fallible-rows/src/lib.rs) | Rust 文档注释 | [fallible-iterator@0.3.0](origins/fallible-iterator-0.3.0/README.md) |
| [blog-storage-record-derive](../../src/foundation/storage/record-derive/Cargo.toml) | record derive | [lib.rs](../../src/foundation/storage/record-derive/src/lib.rs) | Rust 文档注释 | [sqlx-macros-core@0.9.0](origins/sqlx-macros-core-0.9.0/README.md)<br>[sqlx-macros@0.9.0](origins/sqlx-macros-0.9.0/README.md) |
| [blog-storage-sqlite-access](../../src/foundation/storage/sqlite-access/Cargo.toml) | sqlite access | [lib.rs](../../src/foundation/storage/sqlite-access/src/lib.rs) | [src/lib.md](../../src/foundation/storage/sqlite-access/src/lib.md)<br>[tests/README.md](../../src/foundation/storage/sqlite-access/tests/README.md) | [sqlx-core@0.9.0](origins/sqlx-core-0.9.0/README.md)<br>[sqlx-sqlite@0.9.0](origins/sqlx-sqlite-0.9.0/README.md)<br>[sqlx@0.9.0](origins/sqlx-0.9.0/README.md) |
| [blog-storage-sqlite-native](../../src/foundation/storage/sqlite-native/Cargo.toml) | sqlite native | [lib.rs](../../src/foundation/storage/sqlite-native/src/lib.rs) | Rust 文档注释 | [libsqlite3-sys@0.37.0](origins/libsqlite3-sys-0.37.0/README.md) |

## sync

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-sync-atomic-lock](../../src/foundation/sync/atomic-lock/Cargo.toml) | atomic lock | [lib.rs](../../src/foundation/sync/atomic-lock/src/lib.rs) | Rust 文档注释 | [try-lock@0.2.5](origins/try-lock-0.2.5/README.md) |
| [blog-sync-concurrent-collections](../../src/foundation/sync/concurrent-collections/Cargo.toml) | 并发工具、队列、epoch回收与工作窃取 | [lib.rs](../../src/foundation/sync/concurrent-collections/src/lib.rs) | Rust 文档注释 | [crossbeam-deque@0.8.8](origins/crossbeam-deque-0.8.8/README.md)<br>[crossbeam-epoch@0.9.21](origins/crossbeam-epoch-0.9.21/README.md)<br>[crossbeam-queue@0.3.14](origins/crossbeam-queue-0.3.14/README.md)<br>[crossbeam-utils@0.8.23](origins/crossbeam-utils-0.8.23/README.md) |
| [blog-sync-exclusive-value](../../src/foundation/sync/exclusive-value/Cargo.toml) | exclusive value | [lib.rs](../../src/foundation/sync/exclusive-value/src/lib.rs) | Rust 文档注释 | [sync_wrapper@1.0.2](origins/sync_wrapper-1.0.2/README.md) |
| [blog-sync-initialization](../../src/foundation/sync/initialization/Cargo.toml) | initialization | [lib.rs](../../src/foundation/sync/initialization/src/lib.rs) | Rust 文档注释 | [once_cell@1.21.4](origins/once_cell-1.21.4/README.md) |
| [blog-sync-lazy-global](../../src/foundation/sync/lazy-global/Cargo.toml) | lazy global | [lib.rs](../../src/foundation/sync/lazy-global/src/lib.rs) | Rust 文档注释 | [lazy_static@1.5.1](origins/lazy_static-1.5.1/README.md) |
| [blog-sync-message-channel](../../src/foundation/sync/message-channel/Cargo.toml) | message channel | [lib.rs](../../src/foundation/sync/message-channel/src/lib.rs) | Rust 文档注释 | [flume@0.12.0](origins/flume-0.12.0/README.md) |
| [blog-sync-mutex](../../src/foundation/sync/mutex/Cargo.toml) | mutex | [lib.rs](../../src/foundation/sync/mutex/src/lib.rs) | Rust 文档注释 | [lock_api@0.4.14](origins/lock_api-0.4.14/README.md)<br>[parking_lot@0.12.5](origins/parking_lot-0.12.5/README.md)<br>[parking_lot_core@0.9.12](origins/parking_lot_core-0.9.12/README.md) |
| [blog-sync-notification](../../src/foundation/sync/notification/Cargo.toml) | notification | [lib.rs](../../src/foundation/sync/notification/src/lib.rs) | Rust 文档注释 | [event-listener@5.4.2](origins/event-listener-5.4.2/README.md) |
| [blog-sync-pinned-projection](../../src/foundation/sync/pinned-projection/Cargo.toml) | pinned projection | [lib.rs](../../src/foundation/sync/pinned-projection/src/lib.rs) | Rust 文档注释 | [pin-project-lite@0.2.17](origins/pin-project-lite-0.2.17/README.md) |
| [blog-sync-scope-cleanup](../../src/foundation/sync/scope-cleanup/Cargo.toml) | scope cleanup | [lib.rs](../../src/foundation/sync/scope-cleanup/src/lib.rs) | Rust 文档注释 | [scopeguard@1.2.0](origins/scopeguard-1.2.0/README.md) |
| [blog-sync-spin-lock](../../src/foundation/sync/spin-lock/Cargo.toml) | spin lock | [lib.rs](../../src/foundation/sync/spin-lock/src/lib.rs) | Rust 文档注释 | [spin@0.9.9](origins/spin-0.9.9/README.md) |
| [blog-sync-thread-storage](../../src/foundation/sync/thread-storage/Cargo.toml) | thread storage | [lib.rs](../../src/foundation/sync/thread-storage/src/lib.rs) | Rust 文档注释 | [thread_local@1.1.10](origins/thread_local-1.1.10/README.md) |
| [blog-sync-thread-wait](../../src/foundation/sync/thread-wait/Cargo.toml) | thread wait | [lib.rs](../../src/foundation/sync/thread-wait/src/lib.rs) | Rust 文档注释 | [parking@2.2.1](origins/parking-2.2.1/README.md) |

## syntax

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-syntax-alternative](../../src/foundation/syntax/alternative/Cargo.toml) | alternative | [lib.rs](../../src/foundation/syntax/alternative/src/lib.rs) | Rust 文档注释 | [either@1.19.0](origins/either-1.19.0/README.md) |
| [blog-syntax-async-trait](../../src/foundation/syntax/async-trait/Cargo.toml) | async trait | [lib.rs](../../src/foundation/syntax/async-trait/src/lib.rs) | Rust 文档注释 | [async-trait@0.1.92](origins/async-trait-0.1.92/README.md) |
| [blog-syntax-compile-assertions](../../src/foundation/syntax/compile-assertions/Cargo.toml) | compile assertions | [lib.rs](../../src/foundation/syntax/compile-assertions/src/lib.rs) | Rust 文档注释 | [static_assertions@1.1.0](origins/static_assertions-1.1.0/README.md) |
| [blog-syntax-compiler-iterators](../../src/foundation/syntax/compiler-iterators/Cargo.toml) | compiler iterators | [lib.rs](../../src/foundation/syntax/compiler-iterators/src/lib.rs) | Rust 文档注释 | [itertools@0.15.0](origins/itertools-0.15.0/README.md) |
| [blog-syntax-derive-structure](../../src/foundation/syntax/derive-structure/Cargo.toml) | derive structure | [lib.rs](../../src/foundation/syntax/derive-structure/src/lib.rs) | Rust 文档注释 | [synstructure@0.14.0](origins/synstructure-0.14.0/README.md) |
| [blog-syntax-index-expansion](../../src/foundation/syntax/index-expansion/Cargo.toml) | index expansion | [lib.rs](../../src/foundation/syntax/index-expansion/src/lib.rs) | Rust 文档注释 | [seq-macro@0.3.6](origins/seq-macro-0.3.6/README.md) |
| [blog-syntax-record-derive-parser](../../src/foundation/syntax/record-derive-parser/Cargo.toml) | record derive parser | [lib.rs](../../src/foundation/syntax/record-derive-parser/src/lib.rs) | Rust 文档注释 | [syn@2.0.119](origins/syn-2.0.119/README.md) |
| [blog-syntax-rust-parser](../../src/foundation/syntax/rust-parser/Cargo.toml) | rust parser | [lib.rs](../../src/foundation/syntax/rust-parser/src/lib.rs) | Rust 文档注释 | [syn@3.0.6](origins/syn-3.0.6/README.md)<br>[syn@3.0.9](origins/syn-3.0.9/README.md) |
| [blog-syntax-style-derive-parser](../../src/foundation/syntax/style-derive-parser/Cargo.toml) | style derive parser | [lib.rs](../../src/foundation/syntax/style-derive-parser/src/lib.rs) | Rust 文档注释 | [syn@1.0.109](origins/syn-1.0.109/README.md) |
| [blog-syntax-style-iterators](../../src/foundation/syntax/style-iterators/Cargo.toml) | style iterators | [lib.rs](../../src/foundation/syntax/style-iterators/src/lib.rs) | Rust 文档注释 | [itertools@0.10.5](origins/itertools-0.10.5/README.md) |
| [blog-syntax-token-concat](../../src/foundation/syntax/token-concat/Cargo.toml) | token concat | [lib.rs](../../src/foundation/syntax/token-concat/src/lib.rs) | Rust 文档注释 | [pastey@0.1.1](origins/pastey-0.1.1/README.md) |
| [blog-syntax-token-stream](../../src/foundation/syntax/token-stream/Cargo.toml) | token stream | [lib.rs](../../src/foundation/syntax/token-stream/src/lib.rs) | Rust 文档注释 | [proc-macro2@1.0.107](origins/proc-macro2-1.0.107/README.md) |
| [blog-syntax-token-templates](../../src/foundation/syntax/token-templates/Cargo.toml) | token templates | [lib.rs](../../src/foundation/syntax/token-templates/src/lib.rs) | Rust 文档注释 | [quote@1.0.47](origins/quote-1.0.47/README.md) |

## text

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-text-case-insensitive-text](../../src/foundation/text/case-insensitive-text/Cargo.toml) | case insensitive text | [lib.rs](../../src/foundation/text/case-insensitive-text/src/lib.rs) | Rust 文档注释 | [unicase@2.10.0](origins/unicase-2.10.0/README.md) |
| [blog-text-compile-time-text](../../src/foundation/text/compile-time-text/Cargo.toml) | compile time text | [lib.rs](../../src/foundation/text/compile-time-text/src/lib.rs) | Rust 文档注释 | [const-str@0.3.2](origins/const-str-0.3.2/README.md) |
| [blog-text-compile-time-text-macros](../../src/foundation/text/compile-time-text-macros/Cargo.toml) | compile time text macros | [lib.rs](../../src/foundation/text/compile-time-text-macros/src/lib.rs) | Rust 文档注释 | [const-str-proc-macro@0.3.2](origins/const-str-proc-macro-0.3.2/README.md) |
| [blog-text-constant-pattern](../../src/foundation/text/constant-pattern/Cargo.toml) | constant pattern | [lib.rs](../../src/foundation/text/constant-pattern/src/lib.rs) | Rust 文档注释 | [lazy-regex@3.6.2](origins/lazy-regex-3.6.2/README.md) |
| [blog-text-constant-pattern-macros](../../src/foundation/text/constant-pattern-macros/Cargo.toml) | constant pattern macros | [mod.rs](../../src/foundation/text/constant-pattern-macros/mod.rs) | Rust 文档注释 | [lazy-regex-proc_macros@3.6.2](origins/lazy-regex-proc_macros-3.6.2/README.md) |
| [blog-text-inline-ascii](../../src/foundation/text/inline-ascii/Cargo.toml) | inline ascii | [lib.rs](../../src/foundation/text/inline-ascii/src/lib.rs) | Rust 文档注释 | [tinystr@0.8.4](origins/tinystr-0.8.4/README.md) |
| [blog-text-line-layout](../../src/foundation/text/line-layout/Cargo.toml) | line layout | [lib.rs](../../src/foundation/text/line-layout/src/lib.rs) | Rust 文档注释 | [textwrap@0.16.4](origins/textwrap-0.16.4/README.md) |
| [blog-text-macro-identifier-case](../../src/foundation/text/macro-identifier-case/Cargo.toml) | macro identifier case | [lib.rs](../../src/foundation/text/macro-identifier-case/src/lib.rs) | Rust 文档注释 | [heck@0.5.0](origins/heck-0.5.0/README.md) |
| [blog-text-monotone-minima](../../src/foundation/text/monotone-minima/Cargo.toml) | monotone minima | [lib.rs](../../src/foundation/text/monotone-minima/src/lib.rs) | Rust 文档注释 | [smawk@0.3.3](origins/smawk-0.3.3/README.md) |
| [blog-text-pattern-search](../../src/foundation/text/pattern-search/Cargo.toml) | 正则语法、自动机、文字多模式与Unicode匹配 | [lib.rs](../../src/foundation/text/pattern-search/src/lib.rs) | Rust 文档注释 | [aho-corasick@1.1.5](origins/aho-corasick-1.1.5/README.md)<br>[regex-automata@0.4.18](origins/regex-automata-0.4.18/README.md)<br>[regex-syntax@0.8.11](origins/regex-syntax-0.8.11/README.md)<br>[regex@1.13.1](origins/regex-1.13.1/README.md) |
| [blog-text-rust-identifiers](../../src/foundation/text/rust-identifiers/Cargo.toml) | rust identifiers | [lib.rs](../../src/foundation/text/rust-identifiers/src/lib.rs) | Rust 文档注释 | [unicode-ident@1.0.26](origins/unicode-ident-1.0.26/README.md)<br>[unicode-ident@1.0.27](origins/unicode-ident-1.0.27/README.md) |
| [blog-text-shell-words](../../src/foundation/text/shell-words/Cargo.toml) | shell words | [lib.rs](../../src/foundation/text/shell-words/src/lib.rs) | [src/quoting_warning.md](../../src/foundation/text/shell-words/src/quoting_warning.md) | [shlex@2.0.1](origins/shlex-2.0.1/README.md) |
| [blog-text-style-identifier-case](../../src/foundation/text/style-identifier-case/Cargo.toml) | style identifier case | [lib.rs](../../src/foundation/text/style-identifier-case/src/lib.rs) | Rust 文档注释 | [convert_case@0.6.0](origins/convert_case-0.6.0/README.md) |
| [blog-text-terminal-width](../../src/foundation/text/terminal-width/Cargo.toml) | terminal width | [lib.rs](../../src/foundation/text/terminal-width/src/lib.rs) | Rust 文档注释 | [unicode-width@0.2.2](origins/unicode-width-0.2.2/README.md) |
| [blog-text-text-boundaries](../../src/foundation/text/text-boundaries/Cargo.toml) | text boundaries | [lib.rs](../../src/foundation/text/text-boundaries/src/lib.rs) | Rust 文档注释 | [unicode-segmentation@1.13.3](origins/unicode-segmentation-1.13.3/README.md) |
| [blog-text-text-similarity](../../src/foundation/text/text-similarity/Cargo.toml) | text similarity | [lib.rs](../../src/foundation/text/text-similarity/src/lib.rs) | Rust 文档注释 | [strsim@0.11.1](origins/strsim-0.11.1/README.md) |

## unicode

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-unicode-borrow-conversion](../../src/foundation/unicode/borrow-conversion/Cargo.toml) | borrow conversion | [lib.rs](../../src/foundation/unicode/borrow-conversion/src/lib.rs) | Rust 文档注释 | [zerofrom@0.1.8](origins/zerofrom-0.1.8/README.md) |
| [blog-unicode-borrow-conversion-derive](../../src/foundation/unicode/borrow-conversion-derive/Cargo.toml) | borrow conversion derive | [lib.rs](../../src/foundation/unicode/borrow-conversion-derive/src/lib.rs) | Rust 文档注释 | [zerofrom-derive@0.1.8](origins/zerofrom-derive-0.1.8/README.md) |
| [blog-unicode-borrowed-data](../../src/foundation/unicode/borrowed-data/Cargo.toml) | borrowed data | [lib.rs](../../src/foundation/unicode/borrowed-data/src/lib.rs) | Rust 文档注释 | [yoke@0.8.3](origins/yoke-0.8.3/README.md) |
| [blog-unicode-borrowed-data-derive](../../src/foundation/unicode/borrowed-data-derive/Cargo.toml) | borrowed data derive | [lib.rs](../../src/foundation/unicode/borrowed-data-derive/src/lib.rs) | Rust 文档注释 | [yoke-derive@0.8.4](origins/yoke-derive-0.8.4/README.md) |
| [blog-unicode-borrowed-vector](../../src/foundation/unicode/borrowed-vector/Cargo.toml) | borrowed vector | [lib.rs](../../src/foundation/unicode/borrowed-vector/src/lib.rs) | Rust 文档注释 | [zerovec@0.11.8](origins/zerovec-0.11.8/README.md) |
| [blog-unicode-borrowed-vector-derive](../../src/foundation/unicode/borrowed-vector-derive/Cargo.toml) | borrowed vector derive | [lib.rs](../../src/foundation/unicode/borrowed-vector-derive/src/lib.rs) | Rust 文档注释 | [zerovec-derive@0.11.6](origins/zerovec-derive-0.11.6/README.md) |
| [blog-unicode-boundary-analysis](../../src/foundation/unicode/boundary-analysis/Cargo.toml) | boundary analysis | [lib.rs](../../src/foundation/unicode/boundary-analysis/src/lib.rs) | Rust 文档注释 | [icu_segmenter@2.3.0](origins/icu_segmenter-2.3.0/README.md) |
| [blog-unicode-boundary-data](../../src/foundation/unicode/boundary-data/Cargo.toml) | boundary data | [lib.rs](../../src/foundation/unicode/boundary-data/src/lib.rs) | Rust 文档注释 | [icu_segmenter_data@2.3.0](origins/icu_segmenter_data-2.3.0/README.md) |
| [blog-unicode-character-properties](../../src/foundation/unicode/character-properties/Cargo.toml) | character properties | [lib.rs](../../src/foundation/unicode/character-properties/src/lib.rs) | Rust 文档注释 | [icu_properties@2.3.0](origins/icu_properties-2.3.0/README.md)<br>[icu_properties_data@2.3.0](origins/icu_properties_data-2.3.0/README.md) |
| [blog-unicode-compact-trie](../../src/foundation/unicode/compact-trie/Cargo.toml) | compact trie | [lib.rs](../../src/foundation/unicode/compact-trie/src/lib.rs) | Rust 文档注释 | [zerotrie@0.2.5](origins/zerotrie-0.2.5/README.md) |
| [blog-unicode-data-collections](../../src/foundation/unicode/data-collections/Cargo.toml) | data collections | [lib.rs](../../src/foundation/unicode/data-collections/src/lib.rs) | Rust 文档注释 | [icu_collections@2.3.0](origins/icu_collections-2.3.0/README.md) |
| [blog-unicode-data-provider](../../src/foundation/unicode/data-provider/Cargo.toml) | data provider | [lib.rs](../../src/foundation/unicode/data-provider/src/lib.rs) | Rust 文档注释 | [icu_provider@2.3.1](origins/icu_provider-2.3.1/README.md) |
| [blog-unicode-locale-identifiers](../../src/foundation/unicode/locale-identifiers/Cargo.toml) | locale identifiers | [lib.rs](../../src/foundation/unicode/locale-identifiers/src/lib.rs) | Rust 文档注释 | [icu_locale_core@2.3.0](origins/icu_locale_core-2.3.0/README.md) |
| [blog-unicode-locale-selection](../../src/foundation/unicode/locale-selection/Cargo.toml) | locale selection | [lib.rs](../../src/foundation/unicode/locale-selection/src/lib.rs) | Rust 文档注释 | [icu_locale_fallback@2.3.0](origins/icu_locale_fallback-2.3.0/README.md) |
| [blog-unicode-locale-selection-data](../../src/foundation/unicode/locale-selection-data/Cargo.toml) | locale selection data | [lib.rs](../../src/foundation/unicode/locale-selection-data/src/lib.rs) | Rust 文档注释 | [icu_locale_fallback_data@2.3.0](origins/icu_locale_fallback_data-2.3.0/README.md) |
| [blog-unicode-text-normalizer](../../src/foundation/unicode/text-normalizer/Cargo.toml) | text normalizer | [lib.rs](../../src/foundation/unicode/text-normalizer/src/lib.rs) | Rust 文档注释 | [icu_normalizer@2.3.0](origins/icu_normalizer-2.3.0/README.md)<br>[icu_normalizer_data@2.3.0](origins/icu_normalizer_data-2.3.0/README.md) |

## wasm

| 包 | 用途 | API 入口 | 本地说明 | 原始参考 |
| --- | --- | --- | --- | --- |
| [blog-wasm-browser-bridge](../../src/foundation/wasm/browser-bridge/Cargo.toml) | browser bridge | [lib.rs](../../src/foundation/wasm/browser-bridge/src/lib.rs) | Rust 文档注释 | [wasm-bindgen@0.2.129](origins/wasm-bindgen-0.2.129/README.md) |
| [blog-wasm-browser-bridge-entry](../../src/foundation/wasm/browser-bridge-entry/Cargo.toml) | browser bridge entry | [lib.rs](../../src/foundation/wasm/browser-bridge-entry/src/lib.rs) | Rust 文档注释 | [wasm-bindgen-macro-support@0.2.129](origins/wasm-bindgen-macro-support-0.2.129/README.md)<br>[wasm-bindgen-macro@0.2.129](origins/wasm-bindgen-macro-0.2.129/README.md) |
| [blog-wasm-browser-bridge-schema](../../src/foundation/wasm/browser-bridge-schema/Cargo.toml) | browser bridge schema | [lib.rs](../../src/foundation/wasm/browser-bridge-schema/src/lib.rs) | Rust 文档注释 | [wasm-bindgen-shared@0.2.129](origins/wasm-bindgen-shared-0.2.129/README.md) |
| [blog-wasm-browser-futures](../../src/foundation/wasm/browser-futures/Cargo.toml) | browser futures | [lib.rs](../../src/foundation/wasm/browser-futures/src/lib.rs) | Rust 文档注释 | [wasm-bindgen-futures@0.4.79](origins/wasm-bindgen-futures-0.4.79/README.md) |
| [blog-wasm-browser-module](../../src/foundation/wasm/browser-module/Cargo.toml) | browser module | [lib.rs](../../src/foundation/wasm/browser-module/src/lib.rs) | Rust 文档注释 | [wasm-bindgen-cli-support@0.2.129](origins/wasm-bindgen-cli-support-0.2.129/README.md) |
| [blog-wasm-debug-information](../../src/foundation/wasm/debug-information/Cargo.toml) | debug information | [lib.rs](../../src/foundation/wasm/debug-information/src/lib.rs) | Rust 文档注释 | [gimli@0.32.3](origins/gimli-0.32.3/README.md) |
| [blog-wasm-module-decoder](../../src/foundation/wasm/module-decoder/Cargo.toml) | module decoder | [lib.rs](../../src/foundation/wasm/module-decoder/src/lib.rs) | Rust 文档注释 | [wasmparser@0.245.1](origins/wasmparser-0.245.1/README.md) |
| [blog-wasm-module-encoder](../../src/foundation/wasm/module-encoder/Cargo.toml) | module encoder | [lib.rs](../../src/foundation/wasm/module-encoder/src/lib.rs) | Rust 文档注释 | [wasm-encoder@0.245.1](origins/wasm-encoder-0.245.1/README.md) |
| [blog-wasm-module-transform](../../src/foundation/wasm/module-transform/Cargo.toml) | module transform | [lib.rs](../../src/foundation/wasm/module-transform/src/lib.rs) | Rust 文档注释 | [walrus@0.27.2](origins/walrus-0.27.2/README.md) |
| [blog-wasm-module-transform-derive](../../src/foundation/wasm/module-transform-derive/Cargo.toml) | module transform derive | [lib.rs](../../src/foundation/wasm/module-transform-derive/src/lib.rs) | Rust 文档注释 | [walrus-macro@0.26.0](origins/walrus-macro-0.26.0/README.md) |
