# Rust 用途架构

本表由 workspaces.json 的明确职责映射、当前 manifest 本地路径边和已验证的构建入口生成。
层名用于说明职责，不自动证明依赖方向或无环。依赖列包含目标条件、optional、dev 和 build 声明；入口列来自统一 workspace 特性图的保守闭包，不能当作精确链接结果。
每个当前包都必须有 layer、purpose 和可重放的身份；合并包保留多个原始来源，拆分包共享来源，完全不再活动的来源必须有明确退役事件。

## 构建入口

| 入口 | 目标 | 职责 | 根包 |
| --- | --- | --- | --- |
| native | x86_64-unknown-linux-gnu | 服务端、本地基础实现与离线维护工具 | src/server/Cargo.toml, src/application/Cargo.toml, tools/setup/Cargo.toml |
| web-release | wasm32-unknown-unknown | 浏览器 CSR 正式界面 | src/browser/Cargo.toml |
| host-tools | x86_64-unknown-linux-gnu | Web 构建及本地 WASM 绑定生成 | tools/assets/Cargo.toml |
| domain-tests | x86_64-unknown-linux-gnu | Web 域逻辑的宿主测试 | test/browser-domain/Cargo.toml |

## 职责层及实际声明关系

| 职责层 | 包数 | 构建入口 | 依赖的其他层 |
| --- | --- | --- | --- |
| application | 1 | native | crypto, diagnostics, encoding, filesystem, format, network, platform, runtime, storage, syntax |
| assets | 4 | domain-tests, host-tools, web-release | collections, diagnostics, encoding, format, interface, javascript, lookup, math, memory, network, platform, sync, syntax, text |
| build | 1 | host-tools | assets, crypto, diagnostics, encoding, filesystem, format, network, runtime, text, wasm |
| checksum | 1 | host-tools | native |
| cli | 5 | host-tools, native | assets, collections, encoding, format, syntax, text |
| collections | 14 | domain-tests, host-tools, native, web-release | encoding, format, memory, native, platform, runtime, storage, sync, syntax, unicode |
| crypto | 13 | host-tools, native | encoding, math, native, platform |
| diagnostics | 9 | domain-tests, host-tools, native, web-release | cli, collections, format, sync, syntax, text |
| encoding | 23 | domain-tests, host-tools, native, web-release | checksum, diagnostics, format, math, memory, syntax, unicode |
| filesystem | 1 | host-tools, native | math, platform, sync |
| format | 11 | domain-tests, host-tools, native, web-release | cli, collections, diagnostics, encoding, interface, io, math, memory, network, platform, syntax, wasm |
| interface | 5 | host-tools, web-release | collections, diagnostics, format, native, runtime, sync, syntax, text, wasm |
| io | 2 | host-tools | diagnostics, runtime |
| javascript | 1 | host-tools | lookup, syntax |
| lookup | 8 | host-tools | format, math, syntax, text |
| math | 10 | host-tools, native, web-release | crypto, format, memory, native, platform |
| memory | 10 | domain-tests, host-tools, native, web-release | format, native, runtime, syntax |
| native | 8 | host-tools, native, web-release | platform, text |
| network | 15 | domain-tests, host-tools, native | collections, diagnostics, encoding, format, interface, platform, runtime, sync, text, unicode, wasm |
| operations | 1 | native | cli, diagnostics, product, runtime |
| platform | 12 | host-tools, native | collections, diagnostics, format, interface, native, wasm |
| product | 2 | native, web-release | application, assets, cli, crypto, diagnostics, encoding, filesystem, format, interface, network, runtime, storage, syntax, text, wasm |
| runtime | 13 | host-tools, native, web-release | collections, diagnostics, encoding, platform, sync, syntax |
| storage | 4 | host-tools, native | collections, diagnostics, encoding, format, native, network, platform, runtime, sync, syntax, text |
| sync | 13 | host-tools, native, web-release | collections, format, math, native, platform, runtime |
| syntax | 13 | domain-tests, host-tools, native, web-release | format, text |
| text | 16 | domain-tests, host-tools, native, web-release | diagnostics, encoding, format, sync, syntax, unicode |
| unicode | 16 | domain-tests, host-tools, native | collections, diagnostics, encoding, format, memory, syntax, text |
| verification | 1 | domain-tests | assets, format, network |
| wasm | 10 | host-tools, web-release | collections, diagnostics, encoding, format, interface, memory, native, runtime, storage, sync, syntax, text |

## application

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-application (`src/application`) | 认证、SQLite 持久化、安装契约和资产响应 | native | src/server | 项目/独立来源 |

## assets

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-assets-compiler (`src/foundation/assets/compiler`) | 宿主 JS/CSS AST、语义分析、资源链接与压缩 | host-tools | tools/assets | bytecount@0.6.9, cow-utils@0.1.3, cssparser-color@0.1.0, cssparser@0.33.0, dtoa-short@0.3.5, dtoa@1.0.11, json-escape-simd@3.1.2, lightningcss@1.0.0-alpha.71, nonmax@0.5.5, oxc-browserslist@5.0.1, oxc@0.153.0, oxc_allocator@0.153.0, oxc_ast@0.153.0, oxc_ast_visit@0.153.0, oxc_codegen@0.153.0, oxc_compat@0.153.0, oxc_data_structures@0.153.0, oxc_diagnostics@0.153.0, oxc_ecmascript@0.153.0, oxc_estree@0.153.0, oxc_index@5.0.0, oxc_mangler@0.153.0, oxc_minifier@0.153.0, oxc_parser@0.153.0, oxc_regular_expression@0.153.0, oxc_semantic@0.153.0, oxc_sourcemap@9.0.0, oxc_span@0.153.0, oxc_str@0.153.0, oxc_syntax@0.153.0, oxc_traverse@0.153.0, parcel_selectors@0.28.2, pathdiff@0.2.3, precomputed-hash@0.1.1, unicode-id-start@1.5.0 |
| blog-assets-markdown-parser (`src/foundation/assets/markdown-parser`) | markdown parser | web-release, domain-tests | src/browser, src/foundation/cli/argument-derive, test/browser-domain | linkify@0.11.0, pulldown-cmark-escape@0.11.0, pulldown-cmark@0.13.4 |
| blog-assets-style-token-macros (`src/foundation/assets/style-token-macros`) | style token macros | host-tools | src/foundation/assets/compiler | cssparser-macros@0.6.1 |
| blog-assets-style-value-derive (`src/foundation/assets/style-value-derive`) | style value derive | host-tools | src/foundation/assets/compiler | lightningcss-derive@1.0.0-alpha.43 |

## build

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-assets (`tools/assets`) | 静态资源编译、源码收据和开发预览 | host-tools | 入口或保留实现 | 项目/独立来源 |

## checksum

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-checksum-fast-crc (`src/foundation/checksum/fast-crc`) | fast crc | host-tools | src/foundation/encoding/compression | crc32fast@1.5.2 |

## cli

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-cli-argument-derive (`src/foundation/cli/argument-derive`) | argument derive | native | src/foundation/cli/arguments | clap_derive@4.6.7 |
| blog-cli-arguments (`src/foundation/cli/arguments`) | arguments | native | src/server, tools/setup | clap@4.6.7, clap_builder@4.6.7, clap_lex@1.1.1 |
| blog-cli-terminal-format (`src/foundation/cli/terminal-format`) | terminal format | native, host-tools | src/foundation/diagnostics/structured-events | nu-ansi-term@0.50.3 |
| blog-cli-terminal-probe (`src/foundation/cli/terminal-probe`) | terminal probe | 当前入口未选用 | src/foundation/format/parser-combinators | is_terminal_polyfill@1.70.2 |
| blog-cli-terminal-style (`src/foundation/cli/terminal-style`) | 终端样式、颜色策略、ANSI解析与流输出 | native | src/foundation/cli/argument-derive, src/foundation/cli/arguments, src/foundation/format/parser-combinators | anstream@1.0.0, anstyle-parse@1.0.0, anstyle-query@1.1.5, anstyle@1.0.14, colorchoice@1.0.5 |

## collections

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-collections-bit-set (`src/foundation/collections/bit-set`) | bit set | native, web-release, host-tools, domain-tests | src/foundation/assets/compiler, src/foundation/assets/markdown-parser, src/foundation/network/http-middleware, src/foundation/platform/system-calls, src/foundation/wasm/module-decoder | bitflags@2.13.2 |
| blog-collections-compiler-hash (`src/foundation/collections/compiler-hash`) | compiler hash | web-release, host-tools | src/foundation/assets/compiler, src/interface/dom | rustc-hash@2.1.3 |
| blog-collections-concurrent-slab (`src/foundation/collections/concurrent-slab`) | concurrent slab | native, host-tools | src/foundation/diagnostics/structured-events | sharded-slab@0.1.7 |
| blog-collections-database-hash-table (`src/foundation/collections/database-hash-table`) | database hash table | native, host-tools | src/foundation/collections/ordered-cache, src/foundation/storage/sqlite-access, src/foundation/wasm/module-decoder | hashbrown@0.16.1 |
| blog-collections-fast-hash (`src/foundation/collections/fast-hash`) | fast hash | native, host-tools | src/foundation/collections/database-hash-table, src/foundation/collections/hash-table, src/foundation/format/json | foldhash@0.2.0 |
| blog-collections-hash-table (`src/foundation/collections/hash-table`) | hash table | native, web-release, host-tools | src/foundation/assets/compiler, src/foundation/collections/ordered-map | hashbrown@0.17.1 |
| blog-collections-inline-string (`src/foundation/collections/inline-string`) | inline string | host-tools | src/foundation/assets/compiler | compact_str@0.10.0 |
| blog-collections-inline-vector (`src/foundation/collections/inline-vector`) | inline vector | native, host-tools, domain-tests | src/foundation/assets/compiler, src/foundation/cli/terminal-style, src/foundation/collections/inline-string, src/foundation/diagnostics/structured-events, src/foundation/format/json-schema, src/foundation/network/http-transport, src/foundation/network/web-address, src/foundation/storage/sqlite-access, src/foundation/sync/mutex, src/foundation/unicode/boundary-analysis, src/foundation/unicode/text-normalizer | smallvec@1.16.2, smallvec@1.16.3 |
| blog-collections-key-equivalence (`src/foundation/collections/key-equivalence`) | key equivalence | native, web-release, host-tools | src/foundation/collections/database-hash-table, src/foundation/collections/hash-table, src/foundation/collections/ordered-map | equivalent@1.0.2, equivalent@1.0.3 |
| blog-collections-keyed-hash (`src/foundation/collections/keyed-hash`) | keyed hash | host-tools | src/foundation/assets/compiler | ahash@0.8.12 |
| blog-collections-ordered-cache (`src/foundation/collections/ordered-cache`) | ordered cache | native | src/foundation/storage/sqlite-access | hashlink@0.11.1 |
| blog-collections-ordered-map (`src/foundation/collections/ordered-map`) | ordered map | native, web-release, host-tools | src/foundation/assets/compiler, src/foundation/format/configuration, src/foundation/format/json, src/foundation/format/json-schema, src/foundation/network/request-pipeline, src/foundation/storage/sqlite-access, src/foundation/wasm/debug-information, src/foundation/wasm/module-decoder, src/interface/dom | indexmap@2.14.2 |
| blog-collections-slot-storage (`src/foundation/collections/slot-storage`) | slot storage | native, web-release, host-tools | src/foundation/network/request-pipeline, src/foundation/runtime/async-toolkit, src/foundation/runtime/scheduled-io, src/foundation/runtime/task-scheduler | slab@0.4.12 |
| blog-collections-sorted-map (`src/foundation/collections/sorted-map`) | sorted map | native, host-tools, domain-tests | src/foundation/unicode/compact-trie, src/foundation/unicode/locale-identifiers | litemap@0.8.3 |

## crypto

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-crypto-algorithm-identifiers (`src/foundation/crypto/algorithm-identifiers`) | algorithm identifiers | native, host-tools | src/foundation/crypto/digest-traits | const-oid@0.10.2 |
| blog-crypto-conditional-move (`src/foundation/crypto/conditional-move`) | conditional move | native, host-tools | src/foundation/crypto/constant-time-utilities | cmov@0.5.4 |
| blog-crypto-constant-time-comparison (`src/foundation/crypto/constant-time-comparison`) | constant time comparison | native | src/application, src/foundation/crypto/constant-time-utilities, src/foundation/crypto/password-digest-traits, src/foundation/crypto/password-format, src/foundation/math/constant-array | subtle@2.6.1 |
| blog-crypto-constant-time-utilities (`src/foundation/crypto/constant-time-utilities`) | constant time utilities | native, host-tools | src/foundation/crypto/digest-traits, src/foundation/math/constant-array | ctutils@0.4.3 |
| blog-crypto-digest-traits (`src/foundation/crypto/digest-traits`) | digest traits | native, host-tools | src/foundation/crypto/message-authentication, src/foundation/crypto/secure-hash | block-buffer@0.12.1, crypto-common@0.2.2, digest@0.11.3 |
| blog-crypto-memory-hard-hash (`src/foundation/crypto/memory-hard-hash`) | memory hard hash | native | src/foundation/crypto/password-derivation | blake2@0.10.6 |
| blog-crypto-message-authentication (`src/foundation/crypto/message-authentication`) | message authentication | native | src/application | hmac@0.13.0 |
| blog-crypto-password-algorithm-traits (`src/foundation/crypto/password-algorithm-traits`) | password algorithm traits | native | src/foundation/crypto/password-digest-traits | crypto-common@0.1.7 |
| blog-crypto-password-derivation (`src/foundation/crypto/password-derivation`) | password derivation | native | src/application | argon2@0.5.3 |
| blog-crypto-password-digest-traits (`src/foundation/crypto/password-digest-traits`) | password digest traits | native | src/foundation/crypto/memory-hard-hash | digest@0.10.7 |
| blog-crypto-password-format (`src/foundation/crypto/password-format`) | password format | native | src/foundation/crypto/password-derivation | password-hash@0.5.0 |
| blog-crypto-password-hash-blocks (`src/foundation/crypto/password-hash-blocks`) | password hash blocks | native | src/foundation/crypto/password-digest-traits | block-buffer@0.10.4 |
| blog-crypto-secure-hash (`src/foundation/crypto/secure-hash`) | secure hash | native, host-tools | src/application, src/server, tools/assets | sha2@0.11.0 |

## diagnostics

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-diagnostics-error-context (`src/foundation/diagnostics/error-context`) | error context | native, host-tools | src/application, src/foundation/wasm/browser-module, src/foundation/wasm/module-transform, src/server, tools/assets, tools/setup | anyhow@1.0.104 |
| blog-diagnostics-error-display (`src/foundation/diagnostics/error-display`) | error display | native, host-tools, domain-tests | src/foundation/text/inline-ascii, src/foundation/unicode/character-properties, src/foundation/unicode/compact-trie, src/foundation/unicode/data-collections, src/foundation/unicode/data-provider, src/foundation/unicode/locale-identifiers | displaydoc@0.2.7 |
| blog-diagnostics-event-facade (`src/foundation/diagnostics/event-facade`) | event facade | native, host-tools | src/foundation/assets/compiler, src/foundation/diagnostics/structured-events, src/foundation/encoding/byte-search, src/foundation/io/record-stream, src/foundation/network/http-client, src/foundation/platform/local-time-zone, src/foundation/runtime/io-poller, src/foundation/storage/sqlite-access, src/foundation/text/pattern-search, src/foundation/unicode/character-properties, src/foundation/unicode/data-provider, src/foundation/wasm/browser-module, src/foundation/wasm/module-transform | log@0.4.34 |
| blog-diagnostics-event-instrumentation (`src/foundation/diagnostics/event-instrumentation`) | event instrumentation | native, host-tools | src/foundation/diagnostics/structured-events | tracing-attributes@0.1.31 |
| blog-diagnostics-object-clone (`src/foundation/diagnostics/object-clone`) | object clone | native | src/foundation/format/json-schema | dyn-clone@1.0.20 |
| blog-diagnostics-structured-events (`src/foundation/diagnostics/structured-events`) | structured events | native, host-tools | src/application, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-pipeline, src/foundation/network/request-router, src/foundation/runtime/scheduled-io, src/foundation/runtime/task-scheduler, src/foundation/storage/sqlite-access, src/server | tracing-core@0.1.36, tracing-log@0.2.0, tracing-subscriber@0.3.23, tracing@0.1.44 |
| blog-diagnostics-symbol-decoder (`src/foundation/diagnostics/symbol-decoder`) | symbol decoder | host-tools | src/foundation/wasm/browser-module | rustc-demangle@0.1.28 |
| blog-diagnostics-typed-error (`src/foundation/diagnostics/typed-error`) | typed error | native, web-release, host-tools | src/application, src/foundation/assets/compiler, src/foundation/encoding/framing, src/foundation/storage/sqlite-access, src/interface/dom | thiserror@2.0.21 |
| blog-diagnostics-typed-error-derive (`src/foundation/diagnostics/typed-error-derive`) | typed error derive | native, web-release, host-tools | src/foundation/diagnostics/typed-error | thiserror-impl@2.0.21 |

## encoding

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-encoding-application-base-encoding (`src/foundation/encoding/application-base-encoding`) | application base encoding | native, host-tools | src/application, src/foundation/network/http-client, src/foundation/network/http-transport | base64@0.23.1 |
| blog-encoding-borrowed-unicode (`src/foundation/encoding/borrowed-unicode`) | borrowed unicode | native, host-tools, domain-tests | src/foundation/unicode/boundary-analysis, src/foundation/unicode/data-collections, src/foundation/unicode/locale-selection | potential_utf@0.1.6 |
| blog-encoding-byte-buffer (`src/foundation/encoding/byte-buffer`) | byte buffer | native, host-tools | src/application, src/foundation/collections/inline-string, src/foundation/format/json-schema, src/foundation/network/body-stream, src/foundation/network/http-client, src/foundation/network/http-message, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-router, src/foundation/runtime/scheduled-io, src/foundation/runtime/task-scheduler, src/foundation/storage/sqlite-access | bytes@1.12.1 |
| blog-encoding-byte-search (`src/foundation/encoding/byte-search`) | byte search | native, web-release, host-tools, domain-tests | src/foundation/assets/compiler, src/foundation/assets/markdown-parser, src/foundation/format/json, src/foundation/format/parser-combinators, src/foundation/network/request-router, src/foundation/runtime/async-toolkit, src/foundation/storage/sqlite-access, src/foundation/text/pattern-search, tools/assets | memchr@2.8.3 |
| blog-encoding-compression (`src/foundation/encoding/compression`) | DEFLATE、gzip与zlib流和Adler校验 | host-tools | src/foundation/assets/compiler, tools/assets | adler2@2.0.1, flate2@1.1.10, miniz_oxide@0.9.1, simd-adler32@0.3.10 |
| blog-encoding-constant-time-base-encoding (`src/foundation/encoding/constant-time-base-encoding`) | constant time base encoding | native | src/foundation/crypto/password-derivation, src/foundation/crypto/password-format | base64ct@1.8.3 |
| blog-encoding-decimal-float-format (`src/foundation/encoding/decimal-float-format`) | decimal float format | native, web-release, host-tools, domain-tests | src/foundation/collections/inline-string, src/foundation/format/json | zmij@1.0.23 |
| blog-encoding-framing (`src/foundation/encoding/framing`) | framing | host-tools | src/foundation/format/binary-record | cobs@0.3.0 |
| blog-encoding-hex-codec (`src/foundation/encoding/hex-codec`) | hex codec | native | src/server | hex@0.4.3 |
| blog-encoding-http-base-encoding (`src/foundation/encoding/http-base-encoding`) | http base encoding | native, host-tools | src/foundation/network/http-middleware, src/foundation/storage/sqlite-access, src/foundation/wasm/browser-module | base64@0.22.1 |
| blog-encoding-integer-format (`src/foundation/encoding/integer-format`) | integer format | native, web-release, host-tools, domain-tests | src/foundation/assets/compiler, src/foundation/collections/inline-string, src/foundation/format/data-model, src/foundation/format/json, src/foundation/network/form-data, src/foundation/network/http-message, src/foundation/network/http-transport, src/foundation/network/request-router | itoa@1.0.18 |
| blog-encoding-integer-parser (`src/foundation/encoding/integer-parser`) | integer parser | native | src/foundation/storage/sqlite-access | atoi@2.0.0 |
| blog-encoding-radix-codecs (`src/foundation/encoding/radix-codecs`) | radix codecs | host-tools | src/foundation/assets/compiler | data-encoding@2.11.1 |
| blog-encoding-roundtrip-float-format (`src/foundation/encoding/roundtrip-float-format`) | roundtrip float format | native, host-tools | src/foundation/network/form-data | ryu@1.0.23 |
| blog-encoding-script-float-format (`src/foundation/encoding/script-float-format`) | script float format | host-tools | src/foundation/assets/compiler | dragonbox_ecma@0.1.12 |
| blog-encoding-signed-variable-integer (`src/foundation/encoding/signed-variable-integer`) | signed variable integer | host-tools | src/foundation/wasm/browser-module, src/foundation/wasm/module-transform | leb128@0.2.7 |
| blog-encoding-text-writer (`src/foundation/encoding/text-writer`) | text writer | native, host-tools, domain-tests | src/foundation/encoding/borrowed-unicode, src/foundation/unicode/data-provider, src/foundation/unicode/locale-identifiers | writeable@0.6.4 |
| blog-encoding-unicode-decoder (`src/foundation/encoding/unicode-decoder`) | unicode decoder | native | src/foundation/cli/terminal-style | utf8parse@0.2.2 |
| blog-encoding-unicode-iterator (`src/foundation/encoding/unicode-iterator`) | unicode iterator | native, host-tools, domain-tests | src/foundation/network/web-address, src/foundation/unicode/boundary-analysis, src/foundation/unicode/data-collections, src/foundation/unicode/text-normalizer | utf8_iter@1.0.4 |
| blog-encoding-variable-integer-writer (`src/foundation/encoding/variable-integer-writer`) | variable integer writer | host-tools | src/foundation/wasm/module-encoder | leb128fmt@0.1.0 |
| blog-encoding-vector-base-encoding (`src/foundation/encoding/vector-base-encoding`) | vector base encoding | host-tools | src/foundation/assets/compiler | base64-simd@0.8.0 |
| blog-encoding-vector-escaping (`src/foundation/encoding/vector-escaping`) | vector escaping | host-tools | src/foundation/assets/compiler | escape-simd@0.1.0 |
| blog-encoding-vector-primitives (`src/foundation/encoding/vector-primitives`) | vector primitives | host-tools | src/foundation/encoding/vector-base-encoding | vsimd@0.8.0 |

## filesystem

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-filesystem-temporary-storage (`src/foundation/filesystem/temporary-storage`) | temporary storage | native, host-tools | src/application, src/server, tools/assets | tempfile@3.27.0 |

## format

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-format-binary-record (`src/foundation/format/binary-record`) | binary record | host-tools | src/foundation/assets/compiler, src/foundation/unicode/data-provider | postcard@1.1.3 |
| blog-format-configuration (`src/foundation/format/configuration`) | TOML配置解析、保留格式编辑、字节跨度与日期类型 | host-tools | src/foundation/storage/sqlite-access, tools/assets | serde_spanned@0.6.9, toml@0.8.23, toml_datetime@0.6.11, toml_edit@0.22.27, toml_write@0.1.2 |
| blog-format-data-model (`src/foundation/format/data-model`) | data model | native, web-release, host-tools, domain-tests | src/application, src/browser, src/foundation/assets/compiler, src/foundation/assets/markdown-parser, src/foundation/cli/terminal-format, src/foundation/collections/bit-set, src/foundation/collections/database-hash-table, src/foundation/collections/hash-table, src/foundation/collections/inline-string, src/foundation/collections/inline-vector, src/foundation/collections/keyed-hash, src/foundation/collections/ordered-cache, src/foundation/collections/ordered-map, src/foundation/collections/slot-storage, src/foundation/collections/sorted-map, src/foundation/diagnostics/event-facade, src/foundation/diagnostics/structured-events, src/foundation/encoding/borrowed-unicode, src/foundation/encoding/byte-buffer, src/foundation/encoding/compression, src/foundation/encoding/framing, src/foundation/encoding/hex-codec, src/foundation/format/binary-record, src/foundation/format/configuration, src/foundation/format/date-time, src/foundation/format/json, src/foundation/format/json-schema, src/foundation/format/release-identity, src/foundation/lookup/script-table, src/foundation/lookup/style-table, src/foundation/math/constant-array, src/foundation/math/keyed-fingerprint, src/foundation/math/large-integer, src/foundation/math/lookup-random-generator, src/foundation/math/password-random-traits, src/foundation/math/typed-array, src/foundation/memory/allocator, src/foundation/memory/arena, src/foundation/network/form-data, src/foundation/network/http-client, src/foundation/network/request-router, src/foundation/network/web-address, src/foundation/platform/ip-addresses, src/foundation/storage/sqlite-access, src/foundation/sync/mutex, src/foundation/syntax/alternative, src/foundation/text/inline-ascii, src/foundation/unicode/borrowed-vector, src/foundation/unicode/boundary-analysis, src/foundation/unicode/character-properties, src/foundation/unicode/compact-trie, src/foundation/unicode/data-collections, src/foundation/unicode/data-provider, src/foundation/unicode/locale-identifiers, src/foundation/unicode/locale-selection, src/foundation/unicode/text-normalizer, src/foundation/wasm/browser-bridge, src/foundation/wasm/browser-module, src/foundation/wasm/module-decoder, src/interface, src/interface/dom, src/server, test/browser-domain, tools/assets | serde@1.0.229, serde_core@1.0.229, serde_path_to_error@0.1.20 |
| blog-format-data-model-derive (`src/foundation/format/data-model-derive`) | data model derive | native, web-release, host-tools, domain-tests | src/foundation/format/data-model | serde_derive@1.0.229 |
| blog-format-data-model-syntax (`src/foundation/format/data-model-syntax`) | data model syntax | native | src/foundation/format/json-schema-derive | serde_derive_internals@0.30.0 |
| blog-format-date-time (`src/foundation/format/date-time`) | date time | native, web-release | src/application, src/browser, src/foundation/format/json-schema, src/foundation/storage/sqlite-access, src/server | chrono@0.4.45 |
| blog-format-json (`src/foundation/format/json`) | json | native, web-release, host-tools, domain-tests | src/application, src/browser, src/foundation/assets/compiler, src/foundation/diagnostics/structured-events, src/foundation/format/json-schema, src/foundation/math/keyed-fingerprint, src/foundation/network/http-client, src/foundation/network/request-router, src/foundation/storage/sqlite-access, src/foundation/unicode/data-provider, src/foundation/wasm/browser-bridge, src/foundation/wasm/browser-module, src/interface, src/server, test/browser-domain, tools/assets | serde_json@1.0.151, serde_json@1.0.154 |
| blog-format-json-schema (`src/foundation/format/json-schema`) | json schema | native | src/application, src/foundation/collections/inline-string, src/foundation/platform/ip-addresses, src/foundation/unicode/borrowed-vector, src/server | schemars@1.2.3 |
| blog-format-json-schema-derive (`src/foundation/format/json-schema-derive`) | json schema derive | native | src/foundation/format/json-schema | schemars_derive@1.2.3 |
| blog-format-parser-combinators (`src/foundation/format/parser-combinators`) | parser combinators | host-tools | src/foundation/format/configuration | winnow@0.7.15 |
| blog-format-release-identity (`src/foundation/format/release-identity`) | release identity | host-tools | src/foundation/format/json-schema, src/foundation/wasm/module-decoder | semver@1.0.28 |

## interface

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-interface-browser-api (`src/foundation/interface/browser-api`) | browser api | web-release | src/browser, src/foundation/network/http-client, src/interface, src/interface/dom | web-sys@0.3.106 |
| blog-interface-javascript-api (`src/foundation/interface/javascript-api`) | javascript api | web-release, host-tools | src/browser, src/foundation/assets/compiler, src/foundation/format/date-time, src/foundation/interface/browser-api, src/foundation/network/http-client, src/foundation/platform/compiler-entropy, src/foundation/platform/local-time-zone, src/foundation/platform/random-seed, src/foundation/platform/system-entropy, src/foundation/wasm/browser-futures, src/interface, src/interface/dom | js-sys@0.3.106 |
| blog-interface (`src/interface`) | 博客组件、浏览器 API、主题和应用挂载 | web-release | src/browser | 项目/独立来源 |
| blog-dom (`src/interface/dom`) | 响应式 DOM 视图、组件属性和生命周期 | web-release | src/interface | any_spawner@0.3.0, console_error_panic_hook@0.1.7, const_str_slice_concat@0.1.0, drain_filter_polyfill@0.1.3, either_of@0.1.9, erased@0.1.2, gloo-events@0.2.0, gloo-timers@0.3.0, guardian@1.3.0, html-escape@0.2.15, itertools@0.14.0, leptos_dom@0.8.8, next_tuple@0.1.0, oco_ref@0.2.1, or_poisoned@0.1.0, reactive_graph@0.2.16, send_wrapper@0.6.0, slotmap@1.1.1, tachys@0.2.19, throw_error@0.3.1, typed-builder@0.23.2 |
| blog-component-macros (`src/interface/macros`) | CSR 组件和属性的纯语法编译宏 | web-release | src/interface/dom | leptos_hot_reload@0.8.7, paste@1.0.15, typed-builder-macro@0.23.2 |

## io

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-io-portable-stream (`src/foundation/io/portable-stream`) | portable stream | host-tools | src/foundation/format/binary-record | embedded-io@0.6.1 |
| blog-io-record-stream (`src/foundation/io/record-stream`) | record stream | host-tools | src/foundation/format/binary-record | embedded-io@0.4.0 |

## javascript

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-javascript-syntax-derive (`src/foundation/javascript/syntax-derive`) | syntax derive | host-tools | src/foundation/assets/compiler | oxc_ast_macros@0.153.0 |

## lookup

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-lookup-script-table (`src/foundation/lookup/script-table`) | script table | host-tools | src/foundation/assets/compiler, src/foundation/javascript/syntax-derive | phf@0.14.0 |
| blog-lookup-script-table-generator (`src/foundation/lookup/script-table-generator`) | script table generator | host-tools | src/foundation/lookup/script-table-macros | phf_generator@0.14.0 |
| blog-lookup-script-table-hashing (`src/foundation/lookup/script-table-hashing`) | script table hashing | host-tools | src/foundation/lookup/script-table, src/foundation/lookup/script-table-generator, src/foundation/lookup/script-table-macros | phf_shared@0.14.0 |
| blog-lookup-script-table-macros (`src/foundation/lookup/script-table-macros`) | script table macros | host-tools | src/foundation/lookup/script-table | phf_macros@0.14.0 |
| blog-lookup-style-table (`src/foundation/lookup/style-table`) | style table | host-tools | src/foundation/assets/compiler | phf@0.11.3 |
| blog-lookup-style-table-generator (`src/foundation/lookup/style-table-generator`) | style table generator | host-tools | src/foundation/lookup/style-table-macros | phf_generator@0.11.3 |
| blog-lookup-style-table-hashing (`src/foundation/lookup/style-table-hashing`) | style table hashing | host-tools | src/foundation/lookup/style-table, src/foundation/lookup/style-table-generator, src/foundation/lookup/style-table-macros | phf_shared@0.11.3 |
| blog-lookup-style-table-macros (`src/foundation/lookup/style-table-macros`) | style table macros | host-tools | src/foundation/lookup/style-table | phf_macros@0.11.3 |

## math

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-math-constant-array (`src/foundation/math/constant-array`) | constant array | native, host-tools | src/foundation/crypto/digest-traits | hybrid-array@0.4.15 |
| blog-math-fast-random (`src/foundation/math/fast-random`) | fast random | native, host-tools | src/foundation/filesystem/temporary-storage, src/foundation/lookup/script-table-generator, src/foundation/sync/message-channel | fastrand@2.5.0 |
| blog-math-integer-traits (`src/foundation/math/integer-traits`) | integer traits | host-tools | src/foundation/math/large-integer | num-integer@0.1.47 |
| blog-math-keyed-fingerprint (`src/foundation/math/keyed-fingerprint`) | keyed fingerprint | host-tools | src/foundation/lookup/script-table-hashing, src/foundation/lookup/style-table-hashing | siphasher@1.0.4 |
| blog-math-large-integer (`src/foundation/math/large-integer`) | large integer | host-tools | src/foundation/assets/compiler | num-bigint@0.5.1 |
| blog-math-lookup-random-generator (`src/foundation/math/lookup-random-generator`) | lookup random generator | host-tools | src/foundation/lookup/style-table-generator | rand@0.8.8 |
| blog-math-numeric-traits (`src/foundation/math/numeric-traits`) | numeric traits | native, web-release, host-tools | src/foundation/assets/compiler, src/foundation/encoding/integer-parser, src/foundation/format/date-time, src/foundation/math/integer-traits, src/foundation/math/large-integer | num-traits@0.2.19 |
| blog-math-password-random-traits (`src/foundation/math/password-random-traits`) | password random traits | native, host-tools | src/foundation/crypto/password-algorithm-traits, src/foundation/crypto/password-format, src/foundation/math/lookup-random-generator | rand_core@0.6.4 |
| blog-math-type-integers (`src/foundation/math/type-integers`) | type integers | native, host-tools | src/foundation/crypto/password-algorithm-traits, src/foundation/math/constant-array, src/foundation/math/typed-array | typenum@1.20.1 |
| blog-math-typed-array (`src/foundation/math/typed-array`) | typed array | native | src/foundation/crypto/password-algorithm-traits, src/foundation/crypto/password-hash-blocks | generic-array@0.14.7 |

## memory

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-memory-allocator (`src/foundation/memory/allocator`) | allocator | native, web-release, host-tools | src/foundation/assets/compiler, src/foundation/collections/database-hash-table, src/foundation/collections/hash-table, src/foundation/memory/arena | allocator-api2@0.2.21 |
| blog-memory-arena (`src/foundation/memory/arena`) | arena | web-release, host-tools | src/foundation/wasm/browser-bridge-entry | bumpalo@3.20.3 |
| blog-memory-byte-layout (`src/foundation/memory/byte-layout`) | byte layout | host-tools | src/foundation/collections/keyed-hash, src/foundation/math/constant-array | zerocopy@0.8.63 |
| blog-memory-indexed-arena (`src/foundation/memory/indexed-arena`) | indexed arena | host-tools | src/foundation/wasm/module-transform | id-arena@2.3.0 |
| blog-memory-output-reference (`src/foundation/memory/output-reference`) | output reference | host-tools | src/foundation/encoding/vector-base-encoding | outref@0.5.2 |
| blog-memory-owned-borrow (`src/foundation/memory/owned-borrow`) | owned borrow | host-tools | src/foundation/assets/compiler | self_cell@1.3.0 |
| blog-memory-reference-cast (`src/foundation/memory/reference-cast`) | reference cast | native | src/foundation/format/json-schema | ref-cast@1.0.27 |
| blog-memory-reference-cast-derive (`src/foundation/memory/reference-cast-derive`) | reference cast derive | native | src/foundation/memory/reference-cast | ref-cast-impl@1.0.27 |
| blog-memory-stable-address (`src/foundation/memory/stable-address`) | stable address | native, host-tools, domain-tests | src/foundation/unicode/borrowed-data, src/foundation/unicode/data-provider, src/foundation/wasm/debug-information | stable_deref_trait@1.2.1 |
| blog-memory-type-specialization (`src/foundation/memory/type-specialization`) | type specialization | host-tools | src/foundation/collections/inline-string | castaway@0.2.4 |

## native

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-native-c-compiler (`src/foundation/native/c-compiler`) | c compiler | native | src/foundation/storage/sqlite-native | cc@1.6.0 |
| blog-native-compiler-capabilities (`src/foundation/native/compiler-capabilities`) | compiler capabilities | native, web-release, host-tools | src/foundation/math/numeric-traits | autocfg@1.5.1 |
| blog-native-compiler-conditions (`src/foundation/native/compiler-conditions`) | compiler conditions | web-release, host-tools | src/foundation/memory/type-specialization, src/foundation/wasm/browser-bridge | rustversion@1.0.23 |
| blog-native-compiler-probe (`src/foundation/native/compiler-probe`) | compiler probe | native, host-tools | src/foundation/collections/keyed-hash, src/foundation/math/typed-array | version_check@0.9.5 |
| blog-native-system-library-discovery (`src/foundation/native/system-library-discovery`) | system library discovery | native | src/foundation/storage/sqlite-native | pkg-config@0.3.34 |
| blog-native-target-conditions (`src/foundation/native/target-conditions`) | target conditions | native, web-release, host-tools | src/foundation/checksum/fast-crc, src/foundation/collections/inline-string, src/foundation/collections/keyed-hash, src/foundation/crypto/secure-hash, src/foundation/interface/javascript-api, src/foundation/platform/compiler-entropy, src/foundation/platform/random-seed, src/foundation/platform/system-entropy, src/foundation/storage/sqlite-access, src/foundation/sync/mutex, src/foundation/sync/thread-storage, src/foundation/wasm/browser-bridge, src/interface/dom | cfg-if@1.0.5 |
| blog-native-windows-compiler (`src/foundation/native/windows-compiler`) | windows compiler | native | src/foundation/native/c-compiler | find-msvc-tools@0.1.14 |
| blog-native-windows-library-discovery (`src/foundation/native/windows-library-discovery`) | windows library discovery | native | src/foundation/storage/sqlite-native | vcpkg@0.2.15 |

## network

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-network-body-stream (`src/foundation/network/body-stream`) | body stream | native, host-tools | src/application, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-router | http-body-util@0.1.5, http-body@1.1.0 |
| blog-network-byte-ranges (`src/foundation/network/byte-ranges`) | byte ranges | native, host-tools | src/foundation/network/http-middleware | http-range-header@0.4.2 |
| blog-network-form-data (`src/foundation/network/form-data`) | form data | native, host-tools | src/foundation/network/http-client, src/foundation/network/request-router | serde_urlencoded@0.7.1 |
| blog-network-http-client (`src/foundation/network/http-client`) | http client | host-tools | tools/assets | reqwest@0.13.5 |
| blog-network-http-date (`src/foundation/network/http-date`) | http date | native, host-tools | src/foundation/network/http-middleware, src/foundation/network/http-transport | httpdate@1.0.3 |
| blog-network-http-message (`src/foundation/network/http-message`) | http message | native, host-tools | src/application, src/foundation/network/body-stream, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-router | http@1.5.0 |
| blog-network-http-middleware (`src/foundation/network/http-middleware`) | http middleware | native, host-tools | src/foundation/network/http-client, src/server | tower-http@0.6.11 |
| blog-network-http-parser (`src/foundation/network/http-parser`) | http parser | native, host-tools | src/foundation/network/http-transport | httparse@1.10.1 |
| blog-network-http-transport (`src/foundation/network/http-transport`) | http transport | native, host-tools | src/application, src/foundation/network/http-client, src/foundation/network/request-router | hyper-util@0.1.21, hyper@1.12.0 |
| blog-network-media-detection (`src/foundation/network/media-detection`) | media detection | native, host-tools | src/foundation/network/http-client, src/foundation/network/http-middleware, tools/assets | mime_guess@2.0.5 |
| blog-network-media-type (`src/foundation/network/media-type`) | media type | native, host-tools | src/application, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/media-detection, src/foundation/network/request-router | mime@0.3.17 |
| blog-network-request-pipeline (`src/foundation/network/request-pipeline`) | request pipeline | native, host-tools | src/application, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-router, src/server, tools/assets | tower-layer@0.3.3, tower-service@0.3.3, tower@0.5.3 |
| blog-network-request-router (`src/foundation/network/request-router`) | request router | native, host-tools | src/application, src/server, tools/assets | axum-core@0.5.6, axum@0.8.9 |
| blog-network-route-matcher (`src/foundation/network/route-matcher`) | route matcher | native, host-tools | src/foundation/network/request-router | matchit@0.8.4 |
| blog-network-web-address (`src/foundation/network/web-address`) | web address | native, host-tools, domain-tests | src/application, src/browser, src/foundation/assets/compiler, src/foundation/format/json-schema, src/foundation/network/form-data, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-router, src/foundation/storage/sqlite-access, src/server, test/browser-domain, tools/assets | form_urlencoded@1.2.2, idna@1.1.0, idna_adapter@1.2.2, percent-encoding@2.3.2, url@2.5.8 |

## operations

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-setup (`tools/setup`) | 离线初始化、账户恢复与实例维护 | native | 入口或保留实现 | 项目/独立来源 |

## platform

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-platform-compiler-entropy (`src/foundation/platform/compiler-entropy`) | compiler entropy | host-tools | src/foundation/assets/compiler, src/foundation/collections/keyed-hash | getrandom@0.3.4 |
| blog-platform-error-code (`src/foundation/platform/error-code`) | error code | native, host-tools | src/foundation/platform/system-calls, src/foundation/runtime/signal-registry | errno@0.3.14 |
| blog-platform-ip-addresses (`src/foundation/platform/ip-addresses`) | ip addresses | native, host-tools | src/foundation/network/http-transport, src/foundation/storage/sqlite-access | ipnet@2.12.2 |
| blog-platform-linux-abi (`src/foundation/platform/linux-abi`) | linux abi | native, host-tools | src/foundation/platform/system-calls | linux-raw-sys@0.12.1 |
| blog-platform-local-time-zone (`src/foundation/platform/local-time-zone`) | local time zone | native | src/foundation/format/date-time | iana-time-zone@0.1.65 |
| blog-platform-password-processor-features (`src/foundation/platform/password-processor-features`) | password processor features | native | src/foundation/crypto/password-derivation | cpufeatures@0.2.17 |
| blog-platform-processor-features (`src/foundation/platform/processor-features`) | processor features | native, host-tools | src/foundation/crypto/secure-hash | cpufeatures@0.3.1 |
| blog-platform-random-seed (`src/foundation/platform/random-seed`) | random seed | native, host-tools | src/foundation/math/password-random-traits | getrandom@0.2.17 |
| blog-platform-socket-options (`src/foundation/platform/socket-options`) | socket options | native, host-tools | src/application, src/foundation/network/http-transport, src/foundation/runtime/task-scheduler | socket2@0.6.5 |
| blog-platform-system-abi (`src/foundation/platform/system-abi`) | system abi | native, host-tools | src/application, src/foundation/assets/compiler, src/foundation/math/lookup-random-generator, src/foundation/native/c-compiler, src/foundation/network/http-transport, src/foundation/platform/compiler-entropy, src/foundation/platform/error-code, src/foundation/platform/password-processor-features, src/foundation/platform/processor-features, src/foundation/platform/random-seed, src/foundation/platform/socket-options, src/foundation/platform/system-calls, src/foundation/platform/system-entropy, src/foundation/runtime/io-poller, src/foundation/runtime/scheduled-io, src/foundation/runtime/signal-registry, src/foundation/runtime/task-scheduler, src/foundation/sync/mutex | libc@0.2.190, libc@0.2.191 |
| blog-platform-system-calls (`src/foundation/platform/system-calls`) | system calls | native, host-tools | src/application, src/foundation/filesystem/temporary-storage | rustix@1.1.5 |
| blog-platform-system-entropy (`src/foundation/platform/system-entropy`) | system entropy | native, host-tools | src/application, src/foundation/filesystem/temporary-storage, src/foundation/math/fast-random | getrandom@0.4.3 |

## product

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-browser (`src/browser`) | 浏览器 CSR 博客界面 | web-release | 入口或保留实现 | 项目/独立来源 |
| blog (`src/server`) | 博客 HTTP 服务、内容业务与管理员 API | native | tools/setup | 项目/独立来源 |

## runtime

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-runtime-async-toolkit (`src/foundation/runtime/async-toolkit`) | async toolkit | native, web-release, host-tools | src/application, src/foundation/interface/javascript-api, src/foundation/io/record-stream, src/foundation/network/body-stream, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-pipeline, src/foundation/network/request-router, src/foundation/runtime/intrusive-futures, src/foundation/runtime/scheduled-io, src/foundation/runtime/scheduled-stream, src/foundation/storage/sqlite-access, src/foundation/sync/exclusive-value, src/foundation/sync/message-channel, src/interface, src/interface/dom | futures-channel@0.3.34, futures-core@0.3.34, futures-executor@0.3.34, futures-io@0.3.34, futures-sink@0.3.34, futures-task@0.3.34, futures-util@0.3.34, futures@0.3.34 |
| blog-runtime-future-macros (`src/foundation/runtime/future-macros`) | future macros | native, web-release, host-tools | src/foundation/runtime/async-toolkit | futures-macro@0.3.34 |
| blog-runtime-intrusive-futures (`src/foundation/runtime/intrusive-futures`) | intrusive futures | native | src/foundation/storage/sqlite-access | futures-intrusive@0.5.0 |
| blog-runtime-io-poller (`src/foundation/runtime/io-poller`) | io poller | native, host-tools | src/foundation/runtime/task-scheduler | mio@1.2.4 |
| blog-runtime-parallel-iterator (`src/foundation/runtime/parallel-iterator`) | parallel iterator | host-tools | src/foundation/collections/database-hash-table, src/foundation/collections/hash-table, src/foundation/collections/ordered-map, src/foundation/memory/indexed-arena, src/foundation/wasm/module-transform | rayon@1.12.0 |
| blog-runtime-readiness-signal (`src/foundation/runtime/readiness-signal`) | readiness signal | native, host-tools | src/foundation/network/http-transport | want@0.3.2 |
| blog-runtime-scheduled-io (`src/foundation/runtime/scheduled-io`) | scheduled io | native, host-tools | src/application, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/request-pipeline, src/foundation/runtime/scheduled-stream | tokio-util@0.7.19, tokio-util@0.7.20 |
| blog-runtime-scheduled-stream (`src/foundation/runtime/scheduled-stream`) | scheduled stream | native | src/foundation/storage/sqlite-access | tokio-stream@0.1.19 |
| blog-runtime-signal-registry (`src/foundation/runtime/signal-registry`) | signal registry | native, host-tools | src/foundation/runtime/task-scheduler | signal-hook-registry@1.4.8 |
| blog-runtime-task-entry (`src/foundation/runtime/task-entry`) | task entry | native, host-tools | src/foundation/runtime/task-scheduler | tokio-macros@2.7.2 |
| blog-runtime-task-scheduler (`src/foundation/runtime/task-scheduler`) | task scheduler | native, host-tools | src/application, src/foundation/io/record-stream, src/foundation/network/body-stream, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-pipeline, src/foundation/network/request-router, src/foundation/runtime/scheduled-io, src/foundation/runtime/scheduled-stream, src/foundation/storage/sqlite-access, src/foundation/wasm/browser-futures, src/server, tools/assets, tools/setup | tokio@1.53.2 |
| blog-runtime-task-waker (`src/foundation/runtime/task-waker`) | task waker | native, host-tools | src/foundation/network/http-transport | atomic-waker@1.1.2 |
| blog-runtime-worker-pool (`src/foundation/runtime/worker-pool`) | worker pool | host-tools | src/foundation/runtime/parallel-iterator | rayon-core@1.13.0 |

## storage

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-storage-fallible-rows (`src/foundation/storage/fallible-rows`) | fallible rows | host-tools | src/foundation/wasm/debug-information | fallible-iterator@0.3.0 |
| blog-storage-record-derive (`src/foundation/storage/record-derive`) | record derive | native | src/foundation/storage/sqlite-access | sqlx-macros-core@0.9.0, sqlx-macros@0.9.0 |
| blog-storage-sqlite-access (`src/foundation/storage/sqlite-access`) | sqlite access | native | src/application, src/foundation/collections/inline-string, src/server | sqlx-core@0.9.0, sqlx-sqlite@0.9.0, sqlx@0.9.0 |
| blog-storage-sqlite-native (`src/foundation/storage/sqlite-native`) | sqlite native | native | src/application, src/foundation/storage/sqlite-access | libsqlite3-sys@0.37.0 |

## sync

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-sync-atomic-lock (`src/foundation/sync/atomic-lock`) | atomic lock | native, host-tools | src/foundation/runtime/readiness-signal | try-lock@0.2.5 |
| blog-sync-concurrent-collections (`src/foundation/sync/concurrent-collections`) | 并发工具、队列、epoch回收与工作窃取 | native, host-tools | src/foundation/runtime/worker-pool, src/foundation/storage/sqlite-access | crossbeam-deque@0.8.8, crossbeam-epoch@0.9.21, crossbeam-queue@0.3.14, crossbeam-utils@0.8.23 |
| blog-sync-exclusive-value (`src/foundation/sync/exclusive-value`) | exclusive value | native, host-tools | src/foundation/network/http-client, src/foundation/network/request-pipeline, src/foundation/network/request-router | sync_wrapper@1.0.2 |
| blog-sync-initialization (`src/foundation/sync/initialization`) | initialization | native, web-release, host-tools | src/foundation/collections/keyed-hash, src/foundation/diagnostics/structured-events, src/foundation/filesystem/temporary-storage, src/foundation/network/http-client, src/foundation/text/constant-pattern, src/foundation/wasm/browser-bridge | once_cell@1.21.4 |
| blog-sync-lazy-global (`src/foundation/sync/lazy-global`) | lazy global | native, host-tools | src/foundation/assets/compiler, src/foundation/collections/concurrent-slab | lazy_static@1.5.1 |
| blog-sync-message-channel (`src/foundation/sync/message-channel`) | message channel | native | src/foundation/storage/sqlite-access | flume@0.12.0 |
| blog-sync-mutex (`src/foundation/sync/mutex`) | mutex | native | src/foundation/diagnostics/structured-events, src/foundation/runtime/intrusive-futures, src/foundation/runtime/task-scheduler, src/foundation/sync/initialization, src/foundation/sync/spin-lock | lock_api@0.4.14, parking_lot@0.12.5, parking_lot_core@0.9.12 |
| blog-sync-notification (`src/foundation/sync/notification`) | notification | native | src/foundation/storage/sqlite-access | event-listener@5.4.2 |
| blog-sync-pinned-projection (`src/foundation/sync/pinned-projection`) | pinned projection | native, web-release, host-tools | src/foundation/diagnostics/structured-events, src/foundation/network/body-stream, src/foundation/network/http-client, src/foundation/network/http-middleware, src/foundation/network/http-transport, src/foundation/network/request-pipeline, src/foundation/network/request-router, src/foundation/runtime/async-toolkit, src/foundation/runtime/scheduled-io, src/foundation/runtime/scheduled-stream, src/foundation/runtime/task-scheduler, src/foundation/sync/notification, src/interface/dom | pin-project-lite@0.2.17 |
| blog-sync-scope-cleanup (`src/foundation/sync/scope-cleanup`) | scope cleanup | native | src/foundation/sync/mutex | scopeguard@1.2.0 |
| blog-sync-spin-lock (`src/foundation/sync/spin-lock`) | spin lock | native | src/foundation/sync/lazy-global, src/foundation/sync/message-channel | spin@0.9.9 |
| blog-sync-thread-storage (`src/foundation/sync/thread-storage`) | thread storage | native, host-tools | src/foundation/diagnostics/structured-events | thread_local@1.1.10 |
| blog-sync-thread-wait (`src/foundation/sync/thread-wait`) | thread wait | native | src/foundation/sync/notification | parking@2.2.1 |

## syntax

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-syntax-alternative (`src/foundation/syntax/alternative`) | alternative | native, web-release, host-tools | src/foundation/encoding/text-writer, src/foundation/format/json-schema, src/foundation/runtime/parallel-iterator, src/foundation/storage/sqlite-access, src/foundation/syntax/compiler-iterators, src/foundation/syntax/style-iterators, src/interface/dom | either@1.19.0 |
| blog-syntax-async-trait (`src/foundation/syntax/async-trait`) | async trait | native, web-release | src/application, src/interface/dom, src/server | async-trait@0.1.92 |
| blog-syntax-compile-assertions (`src/foundation/syntax/compile-assertions`) | compile assertions | host-tools | src/foundation/collections/inline-string | static_assertions@1.1.0 |
| blog-syntax-compiler-iterators (`src/foundation/syntax/compiler-iterators`) | compiler iterators | host-tools | src/foundation/assets/compiler | itertools@0.15.0 |
| blog-syntax-derive-structure (`src/foundation/syntax/derive-structure`) | derive structure | native, host-tools, domain-tests | src/foundation/unicode/borrow-conversion-derive, src/foundation/unicode/borrowed-data-derive | synstructure@0.14.0 |
| blog-syntax-index-expansion (`src/foundation/syntax/index-expansion`) | index expansion | host-tools | src/foundation/assets/compiler | seq-macro@0.3.6 |
| blog-syntax-record-derive-parser (`src/foundation/syntax/record-derive-parser`) | record derive parser | native, web-release, host-tools | src/foundation/assets/style-token-macros, src/foundation/diagnostics/event-instrumentation, src/foundation/lookup/script-table-macros, src/foundation/lookup/style-table-macros, src/foundation/storage/record-derive, src/foundation/storage/sqlite-native, src/foundation/wasm/module-transform-derive, src/interface/macros | syn@2.0.119 |
| blog-syntax-rust-parser (`src/foundation/syntax/rust-parser`) | rust parser | native, web-release, host-tools, domain-tests | src/foundation/cli/argument-derive, src/foundation/diagnostics/error-display, src/foundation/diagnostics/typed-error-derive, src/foundation/format/data-model-derive, src/foundation/format/data-model-syntax, src/foundation/format/json-schema-derive, src/foundation/javascript/syntax-derive, src/foundation/memory/reference-cast-derive, src/foundation/runtime/future-macros, src/foundation/runtime/task-entry, src/foundation/syntax/async-trait, src/foundation/syntax/derive-structure, src/foundation/text/constant-pattern-macros, src/foundation/unicode/borrow-conversion-derive, src/foundation/unicode/borrowed-data-derive, src/foundation/unicode/borrowed-vector-derive, src/foundation/wasm/browser-bridge-entry, src/interface/macros | syn@3.0.6, syn@3.0.9 |
| blog-syntax-style-derive-parser (`src/foundation/syntax/style-derive-parser`) | style derive parser | host-tools | src/foundation/assets/style-value-derive, src/foundation/text/compile-time-text-macros | syn@1.0.109 |
| blog-syntax-style-iterators (`src/foundation/syntax/style-iterators`) | style iterators | host-tools | src/foundation/assets/compiler | itertools@0.10.5 |
| blog-syntax-token-concat (`src/foundation/syntax/token-concat`) | token concat | host-tools | src/foundation/assets/compiler | pastey@0.1.1 |
| blog-syntax-token-stream (`src/foundation/syntax/token-stream`) | token stream | native, web-release, host-tools, domain-tests | src/foundation/assets/style-value-derive, src/foundation/cli/argument-derive, src/foundation/diagnostics/error-display, src/foundation/diagnostics/event-instrumentation, src/foundation/diagnostics/typed-error-derive, src/foundation/format/data-model-derive, src/foundation/format/data-model-syntax, src/foundation/format/json-schema-derive, src/foundation/javascript/syntax-derive, src/foundation/lookup/script-table-macros, src/foundation/lookup/style-table-macros, src/foundation/memory/reference-cast-derive, src/foundation/runtime/future-macros, src/foundation/runtime/task-entry, src/foundation/storage/record-derive, src/foundation/syntax/async-trait, src/foundation/syntax/derive-structure, src/foundation/syntax/record-derive-parser, src/foundation/syntax/rust-parser, src/foundation/syntax/style-derive-parser, src/foundation/syntax/token-templates, src/foundation/text/compile-time-text-macros, src/foundation/text/constant-pattern-macros, src/foundation/unicode/borrow-conversion-derive, src/foundation/unicode/borrowed-data-derive, src/foundation/unicode/borrowed-vector-derive, src/foundation/wasm/browser-bridge-entry, src/foundation/wasm/module-transform-derive, src/interface/macros | proc-macro2@1.0.107 |
| blog-syntax-token-templates (`src/foundation/syntax/token-templates`) | token templates | native, web-release, host-tools, domain-tests | src/foundation/assets/style-token-macros, src/foundation/assets/style-value-derive, src/foundation/cli/argument-derive, src/foundation/diagnostics/error-display, src/foundation/diagnostics/event-instrumentation, src/foundation/diagnostics/typed-error-derive, src/foundation/format/data-model-derive, src/foundation/format/data-model-syntax, src/foundation/format/json-schema-derive, src/foundation/javascript/syntax-derive, src/foundation/lookup/script-table-macros, src/foundation/lookup/style-table-macros, src/foundation/memory/reference-cast-derive, src/foundation/runtime/future-macros, src/foundation/runtime/task-entry, src/foundation/storage/record-derive, src/foundation/storage/sqlite-native, src/foundation/syntax/async-trait, src/foundation/syntax/derive-structure, src/foundation/syntax/record-derive-parser, src/foundation/syntax/rust-parser, src/foundation/syntax/style-derive-parser, src/foundation/text/compile-time-text-macros, src/foundation/text/constant-pattern-macros, src/foundation/unicode/borrow-conversion-derive, src/foundation/unicode/borrowed-data-derive, src/foundation/unicode/borrowed-vector-derive, src/foundation/wasm/browser-bridge-entry, src/foundation/wasm/module-transform-derive, src/interface/macros | quote@1.0.47 |

## text

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-text-case-insensitive-text (`src/foundation/text/case-insensitive-text`) | case insensitive text | native, web-release, host-tools, domain-tests | src/foundation/assets/markdown-parser, src/foundation/lookup/script-table-hashing, src/foundation/lookup/script-table-macros, src/foundation/lookup/style-table-hashing, src/foundation/lookup/style-table-macros, src/foundation/network/media-detection | unicase@2.10.0 |
| blog-text-compile-time-text (`src/foundation/text/compile-time-text`) | compile time text | host-tools | src/foundation/assets/compiler | const-str@0.3.2 |
| blog-text-compile-time-text-macros (`src/foundation/text/compile-time-text-macros`) | compile time text macros | host-tools | src/foundation/text/compile-time-text | const-str-proc-macro@0.3.2 |
| blog-text-constant-pattern (`src/foundation/text/constant-pattern`) | constant pattern | host-tools | src/foundation/assets/compiler | lazy-regex@3.6.2 |
| blog-text-constant-pattern-macros (`src/foundation/text/constant-pattern-macros`) | constant pattern macros | host-tools | src/foundation/text/constant-pattern | lazy-regex-proc_macros@3.6.2 |
| blog-text-inline-ascii (`src/foundation/text/inline-ascii`) | inline ascii | native, host-tools, domain-tests | src/foundation/unicode/locale-identifiers, src/foundation/unicode/locale-selection | tinystr@0.8.4 |
| blog-text-line-layout (`src/foundation/text/line-layout`) | line layout | host-tools | src/foundation/assets/compiler | textwrap@0.16.4 |
| blog-text-macro-identifier-case (`src/foundation/text/macro-identifier-case`) | macro identifier case | native, host-tools | src/foundation/cli/argument-derive, src/foundation/storage/record-derive, src/foundation/wasm/module-transform-derive | heck@0.5.0 |
| blog-text-monotone-minima (`src/foundation/text/monotone-minima`) | monotone minima | host-tools | src/foundation/text/line-layout | smawk@0.3.3 |
| blog-text-pattern-search (`src/foundation/text/pattern-search`) | 正则语法、自动机、文字多模式与Unicode匹配 | web-release, host-tools | src/browser, src/foundation/diagnostics/structured-events, src/foundation/storage/sqlite-access, src/foundation/text/compile-time-text-macros, src/foundation/text/constant-pattern, src/foundation/text/constant-pattern-macros, tools/assets | aho-corasick@1.1.5, regex-automata@0.4.18, regex-syntax@0.8.11, regex@1.13.1 |
| blog-text-rust-identifiers (`src/foundation/text/rust-identifiers`) | rust identifiers | native, web-release, host-tools, domain-tests | src/foundation/syntax/record-derive-parser, src/foundation/syntax/rust-parser, src/foundation/syntax/style-derive-parser, src/foundation/syntax/token-stream, src/foundation/wasm/browser-bridge-schema | unicode-ident@1.0.26, unicode-ident@1.0.27 |
| blog-text-shell-words (`src/foundation/text/shell-words`) | shell words | native | src/foundation/native/c-compiler | shlex@2.0.1 |
| blog-text-style-identifier-case (`src/foundation/text/style-identifier-case`) | style identifier case | host-tools | src/foundation/assets/style-value-derive | convert_case@0.6.0 |
| blog-text-terminal-width (`src/foundation/text/terminal-width`) | terminal width | host-tools | src/foundation/assets/compiler, src/foundation/text/line-layout | unicode-width@0.2.2 |
| blog-text-text-boundaries (`src/foundation/text/text-boundaries`) | text boundaries | web-release, host-tools | src/foundation/assets/compiler, src/foundation/text/style-identifier-case, src/interface/macros | unicode-segmentation@1.13.3 |
| blog-text-text-similarity (`src/foundation/text/text-similarity`) | text similarity | native | src/foundation/cli/arguments | strsim@0.11.1 |

## unicode

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-unicode-borrow-conversion (`src/foundation/unicode/borrow-conversion`) | borrow conversion | native, host-tools, domain-tests | src/foundation/unicode/borrowed-data, src/foundation/unicode/borrowed-vector, src/foundation/unicode/compact-trie, src/foundation/unicode/data-collections, src/foundation/unicode/data-provider | zerofrom@0.1.8 |
| blog-unicode-borrow-conversion-derive (`src/foundation/unicode/borrow-conversion-derive`) | borrow conversion derive | native, host-tools, domain-tests | src/foundation/unicode/borrow-conversion | zerofrom-derive@0.1.8 |
| blog-unicode-borrowed-data (`src/foundation/unicode/borrowed-data`) | borrowed data | native, host-tools, domain-tests | src/foundation/collections/sorted-map, src/foundation/unicode/borrowed-vector, src/foundation/unicode/compact-trie, src/foundation/unicode/data-collections, src/foundation/unicode/data-provider | yoke@0.8.3 |
| blog-unicode-borrowed-data-derive (`src/foundation/unicode/borrowed-data-derive`) | borrowed data derive | native, host-tools, domain-tests | src/foundation/unicode/borrowed-data | yoke-derive@0.8.4 |
| blog-unicode-borrowed-vector (`src/foundation/unicode/borrowed-vector`) | borrowed vector | native, host-tools, domain-tests | src/foundation/encoding/borrowed-unicode, src/foundation/text/inline-ascii, src/foundation/unicode/boundary-analysis, src/foundation/unicode/character-properties, src/foundation/unicode/compact-trie, src/foundation/unicode/data-collections, src/foundation/unicode/data-provider, src/foundation/unicode/locale-identifiers, src/foundation/unicode/locale-selection, src/foundation/unicode/text-normalizer | zerovec@0.11.8 |
| blog-unicode-borrowed-vector-derive (`src/foundation/unicode/borrowed-vector-derive`) | borrowed vector derive | native, host-tools, domain-tests | src/foundation/unicode/borrowed-vector | zerovec-derive@0.11.6 |
| blog-unicode-boundary-analysis (`src/foundation/unicode/boundary-analysis`) | boundary analysis | host-tools | src/foundation/text/line-layout | icu_segmenter@2.3.0 |
| blog-unicode-boundary-data (`src/foundation/unicode/boundary-data`) | boundary data | host-tools | src/foundation/unicode/boundary-analysis | icu_segmenter_data@2.3.0 |
| blog-unicode-character-properties (`src/foundation/unicode/character-properties`) | character properties | native, host-tools, domain-tests | src/foundation/network/web-address, src/foundation/unicode/text-normalizer | icu_properties@2.3.0, icu_properties_data@2.3.0 |
| blog-unicode-compact-trie (`src/foundation/unicode/compact-trie`) | compact trie | native, host-tools, domain-tests | src/foundation/unicode/character-properties, src/foundation/unicode/data-provider | zerotrie@0.2.5 |
| blog-unicode-data-collections (`src/foundation/unicode/data-collections`) | data collections | native, host-tools, domain-tests | src/foundation/unicode/boundary-analysis, src/foundation/unicode/character-properties, src/foundation/unicode/text-normalizer | icu_collections@2.3.0 |
| blog-unicode-data-provider (`src/foundation/unicode/data-provider`) | data provider | native, host-tools, domain-tests | src/foundation/unicode/boundary-analysis, src/foundation/unicode/character-properties, src/foundation/unicode/locale-selection, src/foundation/unicode/text-normalizer | icu_provider@2.3.1 |
| blog-unicode-locale-identifiers (`src/foundation/unicode/locale-identifiers`) | locale identifiers | native, host-tools, domain-tests | src/foundation/unicode/character-properties, src/foundation/unicode/data-provider, src/foundation/unicode/locale-selection | icu_locale_core@2.3.0 |
| blog-unicode-locale-selection (`src/foundation/unicode/locale-selection`) | locale selection | host-tools | src/foundation/unicode/boundary-analysis | icu_locale_fallback@2.3.0 |
| blog-unicode-locale-selection-data (`src/foundation/unicode/locale-selection-data`) | locale selection data | host-tools | src/foundation/unicode/locale-selection | icu_locale_fallback_data@2.3.0 |
| blog-unicode-text-normalizer (`src/foundation/unicode/text-normalizer`) | text normalizer | native, host-tools, domain-tests | src/foundation/network/web-address | icu_normalizer@2.3.0, icu_normalizer_data@2.3.0 |

## verification

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-browser-domain-tests (`test/browser-domain`) | 浏览器数据契约与Markdown逻辑的宿主测试 | domain-tests | 入口或保留实现 | 项目/独立来源 |

## wasm

| 当前包与路径 | 具体职责 | 构建入口 | 直接消费者 | 原始来源 |
| --- | --- | --- | --- | --- |
| blog-wasm-browser-bridge (`src/foundation/wasm/browser-bridge`) | browser bridge | web-release, host-tools | src/browser, src/foundation/format/date-time, src/foundation/interface/browser-api, src/foundation/interface/javascript-api, src/foundation/network/http-client, src/foundation/platform/compiler-entropy, src/foundation/platform/local-time-zone, src/foundation/platform/random-seed, src/foundation/platform/system-entropy, src/foundation/wasm/browser-futures, src/interface, src/interface/dom | wasm-bindgen@0.2.129 |
| blog-wasm-browser-bridge-entry (`src/foundation/wasm/browser-bridge-entry`) | browser bridge entry | web-release, host-tools | src/foundation/wasm/browser-bridge | wasm-bindgen-macro-support@0.2.129, wasm-bindgen-macro@0.2.129 |
| blog-wasm-browser-bridge-schema (`src/foundation/wasm/browser-bridge-schema`) | browser bridge schema | web-release, host-tools | src/foundation/wasm/browser-bridge, src/foundation/wasm/browser-bridge-entry, src/foundation/wasm/browser-module | wasm-bindgen-shared@0.2.129 |
| blog-wasm-browser-futures (`src/foundation/wasm/browser-futures`) | browser futures | web-release | src/browser, src/foundation/network/http-client, src/interface, src/interface/dom | wasm-bindgen-futures@0.4.79 |
| blog-wasm-browser-module (`src/foundation/wasm/browser-module`) | browser module | host-tools | tools/assets | wasm-bindgen-cli-support@0.2.129 |
| blog-wasm-debug-information (`src/foundation/wasm/debug-information`) | debug information | host-tools | src/foundation/wasm/module-transform | gimli@0.32.3 |
| blog-wasm-module-decoder (`src/foundation/wasm/module-decoder`) | module decoder | host-tools | src/foundation/wasm/browser-module, src/foundation/wasm/module-encoder, src/foundation/wasm/module-transform | wasmparser@0.245.1 |
| blog-wasm-module-encoder (`src/foundation/wasm/module-encoder`) | module encoder | host-tools | src/foundation/wasm/module-transform | wasm-encoder@0.245.1 |
| blog-wasm-module-transform (`src/foundation/wasm/module-transform`) | module transform | host-tools | src/foundation/wasm/browser-module | walrus@0.27.2 |
| blog-wasm-module-transform-derive (`src/foundation/wasm/module-transform-derive`) | module transform derive | host-tools | src/foundation/wasm/module-transform | walrus-macro@0.26.0 |

原始档案和许可清单见 [第三方 Rust 来源](THIRD_PARTY_RUST_SOURCES.md)。内部版本可能省略并由 Cargo 记为 0.0.0；这不改变原始发布版本或安全审计身份。
