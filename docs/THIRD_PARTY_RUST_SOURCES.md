# Rust 第三方源码与离线构建

当前源码按产品用途组织；Cargo 内部包名与版本不是原始上游身份。原始发布版本只用于来源、法律材料和按原始身份执行的安全审计。
compliance/provenance/origins.json 与原始 .crate 档案保持原始身份及逐文件摘要，compliance/rust-sources.json 单独记录当前源码摘要。
compliance/provenance/source-migrations.json 从首次启用前的完整包身份基线出发，逐事件记录重命名、版本变化、合并、拆分、导入和退役。
verify 完整重放日志并核对实际 manifest 与当前映射；refresh 只允许追加迁移事件，不能重写历史或原始来源。
源码收据与空 Cargo home 的 Linux/WASM metadata 是构建输入证据，不替代编译、原生测试、浏览器、安装或正式发行验收。
Rust 工具链、系统原生构建材料、审计数据库和非 Rust 测试工具另行管理。

用途架构、入口和实际声明的包依赖关系见 [Rust 用途架构](RUST_SOURCE_ARCHITECTURE.md)。

## 当前源码与原始来源

| 当前用途路径 | 内部 Cargo 包 | 原始身份 | 原始许可 |
| --- | --- | --- | --- |
| src/application | blog-application | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |
| src/browser | blog-browser | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |
| src/foundation/assets/compiler | blog-assets-compiler | bytecount@0.6.9, cow-utils@0.1.3, cssparser-color@0.1.0, cssparser@0.33.0, dtoa-short@0.3.5, dtoa@1.0.11, json-escape-simd@3.1.2, lightningcss@1.0.0-alpha.71, nonmax@0.5.5, oxc-browserslist@5.0.1, oxc@0.153.0, oxc_allocator@0.153.0, oxc_ast@0.153.0, oxc_ast_visit@0.153.0, oxc_codegen@0.153.0, oxc_compat@0.153.0, oxc_data_structures@0.153.0, oxc_diagnostics@0.153.0, oxc_ecmascript@0.153.0, oxc_estree@0.153.0, oxc_index@5.0.0, oxc_mangler@0.153.0, oxc_minifier@0.153.0, oxc_parser@0.153.0, oxc_regular_expression@0.153.0, oxc_semantic@0.153.0, oxc_sourcemap@9.0.0, oxc_span@0.153.0, oxc_str@0.153.0, oxc_syntax@0.153.0, oxc_traverse@0.153.0, parcel_selectors@0.28.2, pathdiff@0.2.3, precomputed-hash@0.1.1, unicode-id-start@1.5.0 | bytecount@0.6.9: Apache-2.0/MIT; cow-utils@0.1.3: MIT; cssparser-color@0.1.0: MPL-2.0；发布包无独立许可文本，待复核; cssparser@0.33.0: MPL-2.0; dtoa-short@0.3.5: MPL-2.0; dtoa@1.0.11: MIT OR Apache-2.0; json-escape-simd@3.1.2: MIT；发布包无独立许可文本，待复核; lightningcss@1.0.0-alpha.71: MPL-2.0; nonmax@0.5.5: MIT OR Apache-2.0; oxc-browserslist@5.0.1: MIT；发布包无独立许可文本，待复核; oxc@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_allocator@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_ast@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_ast_visit@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_codegen@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_compat@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_data_structures@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_diagnostics@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_ecmascript@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_estree@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_index@5.0.0: MIT；发布包无独立许可文本，待复核; oxc_mangler@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_minifier@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_parser@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_regular_expression@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_semantic@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_sourcemap@9.0.0: BSD-3-Clause; oxc_span@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_str@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_syntax@0.153.0: MIT；发布包无独立许可文本，待复核; oxc_traverse@0.153.0: MIT；发布包无独立许可文本，待复核; parcel_selectors@0.28.2: MPL-2.0; pathdiff@0.2.3: MIT/Apache-2.0; precomputed-hash@0.1.1: MIT; unicode-id-start@1.5.0: (MIT OR Apache-2.0) AND Unicode-3.0 |
| src/foundation/assets/markdown-parser | blog-assets-markdown-parser | linkify@0.11.0, pulldown-cmark-escape@0.11.0, pulldown-cmark@0.13.4 | linkify@0.11.0: MIT OR Apache-2.0; pulldown-cmark-escape@0.11.0: MIT; pulldown-cmark@0.13.4: MIT |
| src/foundation/assets/style-token-macros | blog-assets-style-token-macros | cssparser-macros@0.6.1 | cssparser-macros@0.6.1: MPL-2.0 |
| src/foundation/assets/style-value-derive | blog-assets-style-value-derive | lightningcss-derive@1.0.0-alpha.43 | lightningcss-derive@1.0.0-alpha.43: MPL-2.0；发布包无独立许可文本，待复核 |
| src/foundation/checksum/fast-crc | blog-checksum-fast-crc | crc32fast@1.5.2 | crc32fast@1.5.2: MIT OR Apache-2.0 |
| src/foundation/cli/argument-derive | blog-cli-argument-derive | clap_derive@4.6.7 | clap_derive@4.6.7: MIT OR Apache-2.0 |
| src/foundation/cli/arguments | blog-cli-arguments | clap@4.6.7, clap_builder@4.6.7, clap_lex@1.1.1 | clap@4.6.7: MIT OR Apache-2.0; clap_builder@4.6.7: MIT OR Apache-2.0; clap_lex@1.1.1: MIT OR Apache-2.0 |
| src/foundation/cli/terminal-format | blog-cli-terminal-format | nu-ansi-term@0.50.3 | nu-ansi-term@0.50.3: MIT |
| src/foundation/cli/terminal-probe | blog-cli-terminal-probe | is_terminal_polyfill@1.70.2 | is_terminal_polyfill@1.70.2: MIT OR Apache-2.0 |
| src/foundation/cli/terminal-style | blog-cli-terminal-style | anstream@1.0.0, anstyle-parse@1.0.0, anstyle-query@1.1.5, anstyle@1.0.14, colorchoice@1.0.5 | anstream@1.0.0: MIT OR Apache-2.0; anstyle-parse@1.0.0: MIT OR Apache-2.0; anstyle-query@1.1.5: MIT OR Apache-2.0; anstyle@1.0.14: MIT OR Apache-2.0; colorchoice@1.0.5: MIT OR Apache-2.0 |
| src/foundation/collections/bit-set | blog-collections-bit-set | bitflags@2.13.2 | bitflags@2.13.2: MIT OR Apache-2.0 |
| src/foundation/collections/compiler-hash | blog-collections-compiler-hash | rustc-hash@2.1.3 | rustc-hash@2.1.3: Apache-2.0 OR MIT |
| src/foundation/collections/concurrent-slab | blog-collections-concurrent-slab | sharded-slab@0.1.7 | sharded-slab@0.1.7: MIT |
| src/foundation/collections/database-hash-table | blog-collections-database-hash-table | hashbrown@0.16.1 | hashbrown@0.16.1: MIT OR Apache-2.0 |
| src/foundation/collections/fast-hash | blog-collections-fast-hash | foldhash@0.2.0 | foldhash@0.2.0: Zlib |
| src/foundation/collections/hash-table | blog-collections-hash-table | hashbrown@0.17.1 | hashbrown@0.17.1: MIT OR Apache-2.0 |
| src/foundation/collections/inline-string | blog-collections-inline-string | compact_str@0.10.0 | compact_str@0.10.0: MIT |
| src/foundation/collections/inline-vector | blog-collections-inline-vector | smallvec@1.16.2, smallvec@1.16.3 | smallvec@1.16.2: MIT OR Apache-2.0; smallvec@1.16.3: MIT OR Apache-2.0 |
| src/foundation/collections/key-equivalence | blog-collections-key-equivalence | equivalent@1.0.2, equivalent@1.0.3 | equivalent@1.0.2: Apache-2.0 OR MIT; equivalent@1.0.3: Apache-2.0 OR MIT |
| src/foundation/collections/keyed-hash | blog-collections-keyed-hash | ahash@0.8.12 | ahash@0.8.12: MIT OR Apache-2.0 |
| src/foundation/collections/ordered-cache | blog-collections-ordered-cache | hashlink@0.11.1 | hashlink@0.11.1: MIT OR Apache-2.0 |
| src/foundation/collections/ordered-map | blog-collections-ordered-map | indexmap@2.14.2 | indexmap@2.14.2: Apache-2.0 OR MIT |
| src/foundation/collections/slot-storage | blog-collections-slot-storage | slab@0.4.12 | slab@0.4.12: MIT |
| src/foundation/collections/sorted-map | blog-collections-sorted-map | litemap@0.8.3 | litemap@0.8.3: Unicode-3.0 |
| src/foundation/crypto/algorithm-identifiers | blog-crypto-algorithm-identifiers | const-oid@0.10.2 | const-oid@0.10.2: Apache-2.0 OR MIT |
| src/foundation/crypto/conditional-move | blog-crypto-conditional-move | cmov@0.5.4 | cmov@0.5.4: Apache-2.0 OR MIT |
| src/foundation/crypto/constant-time-comparison | blog-crypto-constant-time-comparison | subtle@2.6.1 | subtle@2.6.1: BSD-3-Clause |
| src/foundation/crypto/constant-time-utilities | blog-crypto-constant-time-utilities | ctutils@0.4.3 | ctutils@0.4.3: Apache-2.0 OR MIT |
| src/foundation/crypto/digest-traits | blog-crypto-digest-traits | block-buffer@0.12.1, crypto-common@0.2.2, digest@0.11.3 | block-buffer@0.12.1: MIT OR Apache-2.0; crypto-common@0.2.2: MIT OR Apache-2.0; digest@0.11.3: MIT OR Apache-2.0 |
| src/foundation/crypto/memory-hard-hash | blog-crypto-memory-hard-hash | blake2@0.10.6 | blake2@0.10.6: MIT OR Apache-2.0 |
| src/foundation/crypto/message-authentication | blog-crypto-message-authentication | hmac@0.13.0 | hmac@0.13.0: MIT OR Apache-2.0 |
| src/foundation/crypto/password-algorithm-traits | blog-crypto-password-algorithm-traits | crypto-common@0.1.7 | crypto-common@0.1.7: MIT OR Apache-2.0 |
| src/foundation/crypto/password-derivation | blog-crypto-password-derivation | argon2@0.5.3 | argon2@0.5.3: MIT OR Apache-2.0 |
| src/foundation/crypto/password-digest-traits | blog-crypto-password-digest-traits | digest@0.10.7 | digest@0.10.7: MIT OR Apache-2.0 |
| src/foundation/crypto/password-format | blog-crypto-password-format | password-hash@0.5.0 | password-hash@0.5.0: MIT OR Apache-2.0 |
| src/foundation/crypto/password-hash-blocks | blog-crypto-password-hash-blocks | block-buffer@0.10.4 | block-buffer@0.10.4: MIT OR Apache-2.0 |
| src/foundation/crypto/secure-hash | blog-crypto-secure-hash | sha2@0.11.0 | sha2@0.11.0: MIT OR Apache-2.0 |
| src/foundation/diagnostics/error-context | blog-diagnostics-error-context | anyhow@1.0.104 | anyhow@1.0.104: MIT OR Apache-2.0 |
| src/foundation/diagnostics/error-display | blog-diagnostics-error-display | displaydoc@0.2.7 | displaydoc@0.2.7: MIT OR Apache-2.0 |
| src/foundation/diagnostics/event-facade | blog-diagnostics-event-facade | log@0.4.34 | log@0.4.34: MIT OR Apache-2.0 |
| src/foundation/diagnostics/event-instrumentation | blog-diagnostics-event-instrumentation | tracing-attributes@0.1.31 | tracing-attributes@0.1.31: MIT |
| src/foundation/diagnostics/object-clone | blog-diagnostics-object-clone | dyn-clone@1.0.20 | dyn-clone@1.0.20: MIT OR Apache-2.0 |
| src/foundation/diagnostics/structured-events | blog-diagnostics-structured-events | tracing-core@0.1.36, tracing-log@0.2.0, tracing-subscriber@0.3.23, tracing@0.1.44 | tracing-core@0.1.36: MIT; tracing-log@0.2.0: MIT; tracing-subscriber@0.3.23: MIT; tracing@0.1.44: MIT |
| src/foundation/diagnostics/symbol-decoder | blog-diagnostics-symbol-decoder | rustc-demangle@0.1.28 | rustc-demangle@0.1.28: MIT/Apache-2.0 |
| src/foundation/diagnostics/typed-error | blog-diagnostics-typed-error | thiserror@2.0.21 | thiserror@2.0.21: MIT OR Apache-2.0 |
| src/foundation/diagnostics/typed-error-derive | blog-diagnostics-typed-error-derive | thiserror-impl@2.0.21 | thiserror-impl@2.0.21: MIT OR Apache-2.0 |
| src/foundation/encoding/application-base-encoding | blog-encoding-application-base-encoding | base64@0.23.1 | base64@0.23.1: MIT OR Apache-2.0 |
| src/foundation/encoding/borrowed-unicode | blog-encoding-borrowed-unicode | potential_utf@0.1.6 | potential_utf@0.1.6: Unicode-3.0 |
| src/foundation/encoding/byte-buffer | blog-encoding-byte-buffer | bytes@1.12.1 | bytes@1.12.1: MIT |
| src/foundation/encoding/byte-search | blog-encoding-byte-search | memchr@2.8.3 | memchr@2.8.3: Unlicense OR MIT |
| src/foundation/encoding/compression | blog-encoding-compression | adler2@2.0.1, flate2@1.1.10, miniz_oxide@0.9.1, simd-adler32@0.3.10 | adler2@2.0.1: 0BSD OR MIT OR Apache-2.0; flate2@1.1.10: MIT OR Apache-2.0; miniz_oxide@0.9.1: MIT OR Zlib OR Apache-2.0; simd-adler32@0.3.10: MIT |
| src/foundation/encoding/constant-time-base-encoding | blog-encoding-constant-time-base-encoding | base64ct@1.8.3 | base64ct@1.8.3: Apache-2.0 OR MIT |
| src/foundation/encoding/decimal-float-format | blog-encoding-decimal-float-format | zmij@1.0.23 | zmij@1.0.23: MIT |
| src/foundation/encoding/framing | blog-encoding-framing | cobs@0.3.0 | cobs@0.3.0: MIT OR Apache-2.0 |
| src/foundation/encoding/hex-codec | blog-encoding-hex-codec | hex@0.4.3 | hex@0.4.3: MIT OR Apache-2.0 |
| src/foundation/encoding/http-base-encoding | blog-encoding-http-base-encoding | base64@0.22.1 | base64@0.22.1: MIT OR Apache-2.0 |
| src/foundation/encoding/integer-format | blog-encoding-integer-format | itoa@1.0.18 | itoa@1.0.18: MIT OR Apache-2.0 |
| src/foundation/encoding/integer-parser | blog-encoding-integer-parser | atoi@2.0.0 | atoi@2.0.0: MIT |
| src/foundation/encoding/radix-codecs | blog-encoding-radix-codecs | data-encoding@2.11.1 | data-encoding@2.11.1: MIT |
| src/foundation/encoding/roundtrip-float-format | blog-encoding-roundtrip-float-format | ryu@1.0.23 | ryu@1.0.23: Apache-2.0 OR BSL-1.0 |
| src/foundation/encoding/script-float-format | blog-encoding-script-float-format | dragonbox_ecma@0.1.12 | dragonbox_ecma@0.1.12: Apache-2.0 WITH LLVM-exception OR BSL-1.0 |
| src/foundation/encoding/signed-variable-integer | blog-encoding-signed-variable-integer | leb128@0.2.7 | leb128@0.2.7: MIT OR Apache-2.0 |
| src/foundation/encoding/text-writer | blog-encoding-text-writer | writeable@0.6.4 | writeable@0.6.4: Unicode-3.0 |
| src/foundation/encoding/unicode-decoder | blog-encoding-unicode-decoder | utf8parse@0.2.2 | utf8parse@0.2.2: Apache-2.0 OR MIT |
| src/foundation/encoding/unicode-iterator | blog-encoding-unicode-iterator | utf8_iter@1.0.4 | utf8_iter@1.0.4: Apache-2.0 OR MIT |
| src/foundation/encoding/variable-integer-writer | blog-encoding-variable-integer-writer | leb128fmt@0.1.0 | leb128fmt@0.1.0: MIT OR Apache-2.0 |
| src/foundation/encoding/vector-base-encoding | blog-encoding-vector-base-encoding | base64-simd@0.8.0 | base64-simd@0.8.0: MIT；发布包无独立许可文本，待复核 |
| src/foundation/encoding/vector-escaping | blog-encoding-vector-escaping | escape-simd@0.1.0 | escape-simd@0.1.0: MIT；发布包无独立许可文本，待复核 |
| src/foundation/encoding/vector-primitives | blog-encoding-vector-primitives | vsimd@0.8.0 | vsimd@0.8.0: MIT；发布包无独立许可文本，待复核 |
| src/foundation/filesystem/temporary-storage | blog-filesystem-temporary-storage | tempfile@3.27.0 | tempfile@3.27.0: MIT OR Apache-2.0 |
| src/foundation/format/binary-record | blog-format-binary-record | postcard@1.1.3 | postcard@1.1.3: MIT OR Apache-2.0 |
| src/foundation/format/configuration | blog-format-configuration | serde_spanned@0.6.9, toml@0.8.23, toml_datetime@0.6.11, toml_edit@0.22.27, toml_write@0.1.2 | serde_spanned@0.6.9: MIT OR Apache-2.0; toml@0.8.23: MIT OR Apache-2.0; toml_datetime@0.6.11: MIT OR Apache-2.0; toml_edit@0.22.27: MIT OR Apache-2.0; toml_write@0.1.2: MIT OR Apache-2.0 |
| src/foundation/format/data-model | blog-format-data-model | serde@1.0.229, serde_core@1.0.229, serde_path_to_error@0.1.20 | serde@1.0.229: MIT OR Apache-2.0; serde_core@1.0.229: MIT OR Apache-2.0; serde_path_to_error@0.1.20: MIT OR Apache-2.0 |
| src/foundation/format/data-model-derive | blog-format-data-model-derive | serde_derive@1.0.229 | serde_derive@1.0.229: MIT OR Apache-2.0 |
| src/foundation/format/data-model-syntax | blog-format-data-model-syntax | serde_derive_internals@0.30.0 | serde_derive_internals@0.30.0: MIT OR Apache-2.0 |
| src/foundation/format/date-time | blog-format-date-time | chrono@0.4.45 | chrono@0.4.45: MIT OR Apache-2.0 |
| src/foundation/format/json | blog-format-json | serde_json@1.0.151, serde_json@1.0.154 | serde_json@1.0.151: MIT OR Apache-2.0; serde_json@1.0.154: MIT OR Apache-2.0 |
| src/foundation/format/json-schema | blog-format-json-schema | schemars@1.2.3 | schemars@1.2.3: MIT |
| src/foundation/format/json-schema-derive | blog-format-json-schema-derive | schemars_derive@1.2.3 | schemars_derive@1.2.3: MIT |
| src/foundation/format/parser-combinators | blog-format-parser-combinators | winnow@0.7.15 | winnow@0.7.15: MIT |
| src/foundation/format/release-identity | blog-format-release-identity | semver@1.0.28 | semver@1.0.28: MIT OR Apache-2.0 |
| src/foundation/interface/browser-api | blog-interface-browser-api | web-sys@0.3.106 | web-sys@0.3.106: MIT OR Apache-2.0 |
| src/foundation/interface/javascript-api | blog-interface-javascript-api | js-sys@0.3.106 | js-sys@0.3.106: MIT OR Apache-2.0 |
| src/foundation/io/portable-stream | blog-io-portable-stream | embedded-io@0.6.1 | embedded-io@0.6.1: MIT OR Apache-2.0 |
| src/foundation/io/record-stream | blog-io-record-stream | embedded-io@0.4.0 | embedded-io@0.4.0: MIT OR Apache-2.0 |
| src/foundation/javascript/syntax-derive | blog-javascript-syntax-derive | oxc_ast_macros@0.153.0 | oxc_ast_macros@0.153.0: MIT；发布包无独立许可文本，待复核 |
| src/foundation/lookup/script-table | blog-lookup-script-table | phf@0.14.0 | phf@0.14.0: MIT |
| src/foundation/lookup/script-table-generator | blog-lookup-script-table-generator | phf_generator@0.14.0 | phf_generator@0.14.0: MIT |
| src/foundation/lookup/script-table-hashing | blog-lookup-script-table-hashing | phf_shared@0.14.0 | phf_shared@0.14.0: MIT |
| src/foundation/lookup/script-table-macros | blog-lookup-script-table-macros | phf_macros@0.14.0 | phf_macros@0.14.0: MIT |
| src/foundation/lookup/style-table | blog-lookup-style-table | phf@0.11.3 | phf@0.11.3: MIT |
| src/foundation/lookup/style-table-generator | blog-lookup-style-table-generator | phf_generator@0.11.3 | phf_generator@0.11.3: MIT |
| src/foundation/lookup/style-table-hashing | blog-lookup-style-table-hashing | phf_shared@0.11.3 | phf_shared@0.11.3: MIT |
| src/foundation/lookup/style-table-macros | blog-lookup-style-table-macros | phf_macros@0.11.3 | phf_macros@0.11.3: MIT |
| src/foundation/math/constant-array | blog-math-constant-array | hybrid-array@0.4.15 | hybrid-array@0.4.15: MIT OR Apache-2.0 |
| src/foundation/math/fast-random | blog-math-fast-random | fastrand@2.5.0 | fastrand@2.5.0: Apache-2.0 OR MIT |
| src/foundation/math/integer-traits | blog-math-integer-traits | num-integer@0.1.47 | num-integer@0.1.47: MIT OR Apache-2.0 |
| src/foundation/math/keyed-fingerprint | blog-math-keyed-fingerprint | siphasher@1.0.4 | siphasher@1.0.4: MIT OR Apache-2.0 |
| src/foundation/math/large-integer | blog-math-large-integer | num-bigint@0.5.1 | num-bigint@0.5.1: MIT OR Apache-2.0 |
| src/foundation/math/lookup-random-generator | blog-math-lookup-random-generator | rand@0.8.8 | rand@0.8.8: MIT OR Apache-2.0 |
| src/foundation/math/numeric-traits | blog-math-numeric-traits | num-traits@0.2.19 | num-traits@0.2.19: MIT OR Apache-2.0 |
| src/foundation/math/password-random-traits | blog-math-password-random-traits | rand_core@0.6.4 | rand_core@0.6.4: MIT OR Apache-2.0 |
| src/foundation/math/type-integers | blog-math-type-integers | typenum@1.20.1 | typenum@1.20.1: MIT OR Apache-2.0 |
| src/foundation/math/typed-array | blog-math-typed-array | generic-array@0.14.7 | generic-array@0.14.7: MIT |
| src/foundation/memory/allocator | blog-memory-allocator | allocator-api2@0.2.21 | allocator-api2@0.2.21: MIT OR Apache-2.0 |
| src/foundation/memory/arena | blog-memory-arena | bumpalo@3.20.3 | bumpalo@3.20.3: MIT OR Apache-2.0 |
| src/foundation/memory/byte-layout | blog-memory-byte-layout | zerocopy@0.8.63 | zerocopy@0.8.63: BSD-2-Clause OR Apache-2.0 OR MIT |
| src/foundation/memory/indexed-arena | blog-memory-indexed-arena | id-arena@2.3.0 | id-arena@2.3.0: MIT/Apache-2.0 |
| src/foundation/memory/output-reference | blog-memory-output-reference | outref@0.5.2 | outref@0.5.2: MIT |
| src/foundation/memory/owned-borrow | blog-memory-owned-borrow | self_cell@1.3.0 | self_cell@1.3.0: Apache-2.0 OR GPL-2.0-only |
| src/foundation/memory/reference-cast | blog-memory-reference-cast | ref-cast@1.0.27 | ref-cast@1.0.27: MIT OR Apache-2.0 |
| src/foundation/memory/reference-cast-derive | blog-memory-reference-cast-derive | ref-cast-impl@1.0.27 | ref-cast-impl@1.0.27: MIT OR Apache-2.0 |
| src/foundation/memory/stable-address | blog-memory-stable-address | stable_deref_trait@1.2.1 | stable_deref_trait@1.2.1: MIT OR Apache-2.0 |
| src/foundation/memory/type-specialization | blog-memory-type-specialization | castaway@0.2.4 | castaway@0.2.4: MIT |
| src/foundation/native/c-compiler | blog-native-c-compiler | cc@1.6.0 | cc@1.6.0: MIT OR Apache-2.0 |
| src/foundation/native/compiler-capabilities | blog-native-compiler-capabilities | autocfg@1.5.1 | autocfg@1.5.1: Apache-2.0 OR MIT |
| src/foundation/native/compiler-conditions | blog-native-compiler-conditions | rustversion@1.0.23 | rustversion@1.0.23: MIT OR Apache-2.0 |
| src/foundation/native/compiler-probe | blog-native-compiler-probe | version_check@0.9.5 | version_check@0.9.5: MIT/Apache-2.0 |
| src/foundation/native/system-library-discovery | blog-native-system-library-discovery | pkg-config@0.3.34 | pkg-config@0.3.34: MIT OR Apache-2.0 |
| src/foundation/native/target-conditions | blog-native-target-conditions | cfg-if@1.0.5 | cfg-if@1.0.5: MIT OR Apache-2.0 |
| src/foundation/native/windows-compiler | blog-native-windows-compiler | find-msvc-tools@0.1.14 | find-msvc-tools@0.1.14: MIT OR Apache-2.0 |
| src/foundation/native/windows-library-discovery | blog-native-windows-library-discovery | vcpkg@0.2.15 | vcpkg@0.2.15: MIT/Apache-2.0 |
| src/foundation/network/body-stream | blog-network-body-stream | http-body-util@0.1.5, http-body@1.1.0 | http-body-util@0.1.5: MIT; http-body@1.1.0: MIT |
| src/foundation/network/byte-ranges | blog-network-byte-ranges | http-range-header@0.4.2 | http-range-header@0.4.2: MIT |
| src/foundation/network/form-data | blog-network-form-data | serde_urlencoded@0.7.1 | serde_urlencoded@0.7.1: MIT/Apache-2.0 |
| src/foundation/network/http-client | blog-network-http-client | reqwest@0.13.5 | reqwest@0.13.5: MIT OR Apache-2.0 |
| src/foundation/network/http-date | blog-network-http-date | httpdate@1.0.3 | httpdate@1.0.3: MIT OR Apache-2.0 |
| src/foundation/network/http-message | blog-network-http-message | http@1.5.0 | http@1.5.0: MIT OR Apache-2.0 |
| src/foundation/network/http-middleware | blog-network-http-middleware | tower-http@0.6.11 | tower-http@0.6.11: MIT |
| src/foundation/network/http-parser | blog-network-http-parser | httparse@1.10.1 | httparse@1.10.1: MIT OR Apache-2.0 |
| src/foundation/network/http-transport | blog-network-http-transport | hyper-util@0.1.21, hyper@1.12.0 | hyper-util@0.1.21: MIT; hyper@1.12.0: MIT |
| src/foundation/network/media-detection | blog-network-media-detection | mime_guess@2.0.5 | mime_guess@2.0.5: MIT |
| src/foundation/network/media-type | blog-network-media-type | mime@0.3.17 | mime@0.3.17: MIT OR Apache-2.0 |
| src/foundation/network/request-pipeline | blog-network-request-pipeline | tower-layer@0.3.3, tower-service@0.3.3, tower@0.5.3 | tower-layer@0.3.3: MIT; tower-service@0.3.3: MIT; tower@0.5.3: MIT |
| src/foundation/network/request-router | blog-network-request-router | axum-core@0.5.6, axum@0.8.9 | axum-core@0.5.6: MIT; axum@0.8.9: MIT |
| src/foundation/network/route-matcher | blog-network-route-matcher | matchit@0.8.4 | matchit@0.8.4: MIT AND BSD-3-Clause |
| src/foundation/network/web-address | blog-network-web-address | form_urlencoded@1.2.2, idna@1.1.0, idna_adapter@1.2.2, percent-encoding@2.3.2, url@2.5.8 | form_urlencoded@1.2.2: MIT OR Apache-2.0; idna@1.1.0: MIT OR Apache-2.0; idna_adapter@1.2.2: Apache-2.0 OR MIT; percent-encoding@2.3.2: MIT OR Apache-2.0; url@2.5.8: MIT OR Apache-2.0 |
| src/foundation/platform/compiler-entropy | blog-platform-compiler-entropy | getrandom@0.3.4 | getrandom@0.3.4: MIT OR Apache-2.0 |
| src/foundation/platform/error-code | blog-platform-error-code | errno@0.3.14 | errno@0.3.14: MIT OR Apache-2.0 |
| src/foundation/platform/ip-addresses | blog-platform-ip-addresses | ipnet@2.12.2 | ipnet@2.12.2: MIT OR Apache-2.0 |
| src/foundation/platform/linux-abi | blog-platform-linux-abi | linux-raw-sys@0.12.1 | linux-raw-sys@0.12.1: Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| src/foundation/platform/local-time-zone | blog-platform-local-time-zone | iana-time-zone@0.1.65 | iana-time-zone@0.1.65: MIT OR Apache-2.0 |
| src/foundation/platform/password-processor-features | blog-platform-password-processor-features | cpufeatures@0.2.17 | cpufeatures@0.2.17: MIT OR Apache-2.0 |
| src/foundation/platform/processor-features | blog-platform-processor-features | cpufeatures@0.3.1 | cpufeatures@0.3.1: MIT OR Apache-2.0 |
| src/foundation/platform/random-seed | blog-platform-random-seed | getrandom@0.2.17 | getrandom@0.2.17: MIT OR Apache-2.0 |
| src/foundation/platform/socket-options | blog-platform-socket-options | socket2@0.6.5 | socket2@0.6.5: MIT OR Apache-2.0 |
| src/foundation/platform/system-abi | blog-platform-system-abi | libc@0.2.190, libc@0.2.191 | libc@0.2.190: MIT OR Apache-2.0; libc@0.2.191: MIT OR Apache-2.0 |
| src/foundation/platform/system-calls | blog-platform-system-calls | rustix@1.1.5 | rustix@1.1.5: Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| src/foundation/platform/system-entropy | blog-platform-system-entropy | getrandom@0.4.3 | getrandom@0.4.3: MIT OR Apache-2.0 |
| src/foundation/runtime/async-toolkit | blog-runtime-async-toolkit | futures-channel@0.3.34, futures-core@0.3.34, futures-executor@0.3.34, futures-io@0.3.34, futures-sink@0.3.34, futures-task@0.3.34, futures-util@0.3.34, futures@0.3.34 | futures-channel@0.3.34: MIT OR Apache-2.0; futures-core@0.3.34: MIT OR Apache-2.0; futures-executor@0.3.34: MIT OR Apache-2.0; futures-io@0.3.34: MIT OR Apache-2.0; futures-sink@0.3.34: MIT OR Apache-2.0; futures-task@0.3.34: MIT OR Apache-2.0; futures-util@0.3.34: MIT OR Apache-2.0; futures@0.3.34: MIT OR Apache-2.0 |
| src/foundation/runtime/future-macros | blog-runtime-future-macros | futures-macro@0.3.34 | futures-macro@0.3.34: MIT OR Apache-2.0 |
| src/foundation/runtime/intrusive-futures | blog-runtime-intrusive-futures | futures-intrusive@0.5.0 | futures-intrusive@0.5.0: MIT OR Apache-2.0 |
| src/foundation/runtime/io-poller | blog-runtime-io-poller | mio@1.2.4 | mio@1.2.4: MIT |
| src/foundation/runtime/parallel-iterator | blog-runtime-parallel-iterator | rayon@1.12.0 | rayon@1.12.0: MIT OR Apache-2.0 |
| src/foundation/runtime/readiness-signal | blog-runtime-readiness-signal | want@0.3.2 | want@0.3.2: MIT |
| src/foundation/runtime/scheduled-io | blog-runtime-scheduled-io | tokio-util@0.7.19, tokio-util@0.7.20 | tokio-util@0.7.19: MIT; tokio-util@0.7.20: MIT |
| src/foundation/runtime/scheduled-stream | blog-runtime-scheduled-stream | tokio-stream@0.1.19 | tokio-stream@0.1.19: MIT |
| src/foundation/runtime/signal-registry | blog-runtime-signal-registry | signal-hook-registry@1.4.8 | signal-hook-registry@1.4.8: MIT OR Apache-2.0 |
| src/foundation/runtime/task-entry | blog-runtime-task-entry | tokio-macros@2.7.2 | tokio-macros@2.7.2: MIT |
| src/foundation/runtime/task-scheduler | blog-runtime-task-scheduler | tokio@1.53.2 | tokio@1.53.2: MIT |
| src/foundation/runtime/task-waker | blog-runtime-task-waker | atomic-waker@1.1.2 | atomic-waker@1.1.2: Apache-2.0 OR MIT |
| src/foundation/runtime/worker-pool | blog-runtime-worker-pool | rayon-core@1.13.0 | rayon-core@1.13.0: MIT OR Apache-2.0 |
| src/foundation/storage/fallible-rows | blog-storage-fallible-rows | fallible-iterator@0.3.0 | fallible-iterator@0.3.0: MIT/Apache-2.0 |
| src/foundation/storage/record-derive | blog-storage-record-derive | sqlx-macros-core@0.9.0, sqlx-macros@0.9.0 | sqlx-macros-core@0.9.0: MIT OR Apache-2.0; sqlx-macros@0.9.0: MIT OR Apache-2.0 |
| src/foundation/storage/sqlite-access | blog-storage-sqlite-access | sqlx-core@0.9.0, sqlx-sqlite@0.9.0, sqlx@0.9.0 | sqlx-core@0.9.0: MIT OR Apache-2.0; sqlx-sqlite@0.9.0: MIT OR Apache-2.0; sqlx@0.9.0: MIT OR Apache-2.0 |
| src/foundation/storage/sqlite-native | blog-storage-sqlite-native | libsqlite3-sys@0.37.0 | libsqlite3-sys@0.37.0: MIT |
| src/foundation/sync/atomic-lock | blog-sync-atomic-lock | try-lock@0.2.5 | try-lock@0.2.5: MIT |
| src/foundation/sync/concurrent-collections | blog-sync-concurrent-collections | crossbeam-deque@0.8.8, crossbeam-epoch@0.9.21, crossbeam-queue@0.3.14, crossbeam-utils@0.8.23 | crossbeam-deque@0.8.8: MIT OR Apache-2.0; crossbeam-epoch@0.9.21: MIT OR Apache-2.0; crossbeam-queue@0.3.14: MIT OR Apache-2.0; crossbeam-utils@0.8.23: MIT OR Apache-2.0 |
| src/foundation/sync/exclusive-value | blog-sync-exclusive-value | sync_wrapper@1.0.2 | sync_wrapper@1.0.2: Apache-2.0 |
| src/foundation/sync/initialization | blog-sync-initialization | once_cell@1.21.4 | once_cell@1.21.4: MIT OR Apache-2.0 |
| src/foundation/sync/lazy-global | blog-sync-lazy-global | lazy_static@1.5.1 | lazy_static@1.5.1: MIT OR Apache-2.0 |
| src/foundation/sync/message-channel | blog-sync-message-channel | flume@0.12.0 | flume@0.12.0: Apache-2.0/MIT |
| src/foundation/sync/mutex | blog-sync-mutex | lock_api@0.4.14, parking_lot@0.12.5, parking_lot_core@0.9.12 | lock_api@0.4.14: MIT OR Apache-2.0; parking_lot@0.12.5: MIT OR Apache-2.0; parking_lot_core@0.9.12: MIT OR Apache-2.0 |
| src/foundation/sync/notification | blog-sync-notification | event-listener@5.4.2 | event-listener@5.4.2: Apache-2.0 OR MIT |
| src/foundation/sync/pinned-projection | blog-sync-pinned-projection | pin-project-lite@0.2.17 | pin-project-lite@0.2.17: Apache-2.0 OR MIT |
| src/foundation/sync/scope-cleanup | blog-sync-scope-cleanup | scopeguard@1.2.0 | scopeguard@1.2.0: MIT OR Apache-2.0 |
| src/foundation/sync/spin-lock | blog-sync-spin-lock | spin@0.9.9 | spin@0.9.9: MIT |
| src/foundation/sync/thread-storage | blog-sync-thread-storage | thread_local@1.1.10 | thread_local@1.1.10: MIT OR Apache-2.0 |
| src/foundation/sync/thread-wait | blog-sync-thread-wait | parking@2.2.1 | parking@2.2.1: Apache-2.0 OR MIT |
| src/foundation/syntax/alternative | blog-syntax-alternative | either@1.19.0 | either@1.19.0: MIT OR Apache-2.0 |
| src/foundation/syntax/async-trait | blog-syntax-async-trait | async-trait@0.1.92 | async-trait@0.1.92: MIT OR Apache-2.0 |
| src/foundation/syntax/compile-assertions | blog-syntax-compile-assertions | static_assertions@1.1.0 | static_assertions@1.1.0: MIT OR Apache-2.0 |
| src/foundation/syntax/compiler-iterators | blog-syntax-compiler-iterators | itertools@0.15.0 | itertools@0.15.0: MIT OR Apache-2.0 |
| src/foundation/syntax/derive-structure | blog-syntax-derive-structure | synstructure@0.14.0 | synstructure@0.14.0: MIT |
| src/foundation/syntax/index-expansion | blog-syntax-index-expansion | seq-macro@0.3.6 | seq-macro@0.3.6: MIT OR Apache-2.0 |
| src/foundation/syntax/record-derive-parser | blog-syntax-record-derive-parser | syn@2.0.119 | syn@2.0.119: MIT OR Apache-2.0 |
| src/foundation/syntax/rust-parser | blog-syntax-rust-parser | syn@3.0.6, syn@3.0.9 | syn@3.0.6: MIT OR Apache-2.0; syn@3.0.9: MIT OR Apache-2.0 |
| src/foundation/syntax/style-derive-parser | blog-syntax-style-derive-parser | syn@1.0.109 | syn@1.0.109: MIT OR Apache-2.0 |
| src/foundation/syntax/style-iterators | blog-syntax-style-iterators | itertools@0.10.5 | itertools@0.10.5: MIT/Apache-2.0 |
| src/foundation/syntax/token-concat | blog-syntax-token-concat | pastey@0.1.1 | pastey@0.1.1: MIT OR Apache-2.0 |
| src/foundation/syntax/token-stream | blog-syntax-token-stream | proc-macro2@1.0.107 | proc-macro2@1.0.107: MIT OR Apache-2.0 |
| src/foundation/syntax/token-templates | blog-syntax-token-templates | quote@1.0.47 | quote@1.0.47: MIT OR Apache-2.0 |
| src/foundation/text/case-insensitive-text | blog-text-case-insensitive-text | unicase@2.10.0 | unicase@2.10.0: MIT OR Apache-2.0 |
| src/foundation/text/compile-time-text | blog-text-compile-time-text | const-str@0.3.2 | const-str@0.3.2: MIT；发布包无独立许可文本，待复核 |
| src/foundation/text/compile-time-text-macros | blog-text-compile-time-text-macros | const-str-proc-macro@0.3.2 | const-str-proc-macro@0.3.2: MIT；发布包无独立许可文本，待复核 |
| src/foundation/text/constant-pattern | blog-text-constant-pattern | lazy-regex@3.6.2 | lazy-regex@3.6.2: MIT |
| src/foundation/text/constant-pattern-macros | blog-text-constant-pattern-macros | lazy-regex-proc_macros@3.6.2 | lazy-regex-proc_macros@3.6.2: MIT；发布包无独立许可文本，待复核 |
| src/foundation/text/inline-ascii | blog-text-inline-ascii | tinystr@0.8.4 | tinystr@0.8.4: Unicode-3.0 |
| src/foundation/text/line-layout | blog-text-line-layout | textwrap@0.16.4 | textwrap@0.16.4: MIT |
| src/foundation/text/macro-identifier-case | blog-text-macro-identifier-case | heck@0.5.0 | heck@0.5.0: MIT OR Apache-2.0 |
| src/foundation/text/monotone-minima | blog-text-monotone-minima | smawk@0.3.3 | smawk@0.3.3: MIT |
| src/foundation/text/pattern-search | blog-text-pattern-search | aho-corasick@1.1.5, regex-automata@0.4.18, regex-syntax@0.8.11, regex@1.13.1 | aho-corasick@1.1.5: Unlicense OR MIT; regex-automata@0.4.18: MIT OR Apache-2.0; regex-syntax@0.8.11: MIT OR Apache-2.0; regex@1.13.1: MIT OR Apache-2.0 |
| src/foundation/text/rust-identifiers | blog-text-rust-identifiers | unicode-ident@1.0.26, unicode-ident@1.0.27 | unicode-ident@1.0.26: (MIT OR Apache-2.0) AND Unicode-3.0; unicode-ident@1.0.27: (MIT OR Apache-2.0) AND Unicode-3.0 |
| src/foundation/text/shell-words | blog-text-shell-words | shlex@2.0.1 | shlex@2.0.1: MIT OR Apache-2.0 |
| src/foundation/text/style-identifier-case | blog-text-style-identifier-case | convert_case@0.6.0 | convert_case@0.6.0: MIT |
| src/foundation/text/terminal-width | blog-text-terminal-width | unicode-width@0.2.2 | unicode-width@0.2.2: MIT OR Apache-2.0 |
| src/foundation/text/text-boundaries | blog-text-text-boundaries | unicode-segmentation@1.13.3 | unicode-segmentation@1.13.3: MIT OR Apache-2.0 |
| src/foundation/text/text-similarity | blog-text-text-similarity | strsim@0.11.1 | strsim@0.11.1: MIT |
| src/foundation/unicode/borrow-conversion | blog-unicode-borrow-conversion | zerofrom@0.1.8 | zerofrom@0.1.8: Unicode-3.0 |
| src/foundation/unicode/borrow-conversion-derive | blog-unicode-borrow-conversion-derive | zerofrom-derive@0.1.8 | zerofrom-derive@0.1.8: Unicode-3.0 |
| src/foundation/unicode/borrowed-data | blog-unicode-borrowed-data | yoke@0.8.3 | yoke@0.8.3: Unicode-3.0 |
| src/foundation/unicode/borrowed-data-derive | blog-unicode-borrowed-data-derive | yoke-derive@0.8.4 | yoke-derive@0.8.4: Unicode-3.0 |
| src/foundation/unicode/borrowed-vector | blog-unicode-borrowed-vector | zerovec@0.11.8 | zerovec@0.11.8: Unicode-3.0 |
| src/foundation/unicode/borrowed-vector-derive | blog-unicode-borrowed-vector-derive | zerovec-derive@0.11.6 | zerovec-derive@0.11.6: Unicode-3.0 |
| src/foundation/unicode/boundary-analysis | blog-unicode-boundary-analysis | icu_segmenter@2.3.0 | icu_segmenter@2.3.0: Unicode-3.0 |
| src/foundation/unicode/boundary-data | blog-unicode-boundary-data | icu_segmenter_data@2.3.0 | icu_segmenter_data@2.3.0: Unicode-3.0 |
| src/foundation/unicode/character-properties | blog-unicode-character-properties | icu_properties@2.3.0, icu_properties_data@2.3.0 | icu_properties@2.3.0: Unicode-3.0; icu_properties_data@2.3.0: Unicode-3.0 |
| src/foundation/unicode/compact-trie | blog-unicode-compact-trie | zerotrie@0.2.5 | zerotrie@0.2.5: Unicode-3.0 |
| src/foundation/unicode/data-collections | blog-unicode-data-collections | icu_collections@2.3.0 | icu_collections@2.3.0: Unicode-3.0 |
| src/foundation/unicode/data-provider | blog-unicode-data-provider | icu_provider@2.3.1 | icu_provider@2.3.1: Unicode-3.0 |
| src/foundation/unicode/locale-identifiers | blog-unicode-locale-identifiers | icu_locale_core@2.3.0 | icu_locale_core@2.3.0: Unicode-3.0 |
| src/foundation/unicode/locale-selection | blog-unicode-locale-selection | icu_locale_fallback@2.3.0 | icu_locale_fallback@2.3.0: Unicode-3.0 |
| src/foundation/unicode/locale-selection-data | blog-unicode-locale-selection-data | icu_locale_fallback_data@2.3.0 | icu_locale_fallback_data@2.3.0: Unicode-3.0 |
| src/foundation/unicode/text-normalizer | blog-unicode-text-normalizer | icu_normalizer@2.3.0, icu_normalizer_data@2.3.0 | icu_normalizer@2.3.0: Unicode-3.0; icu_normalizer_data@2.3.0: Unicode-3.0 |
| src/foundation/wasm/browser-bridge | blog-wasm-browser-bridge | wasm-bindgen@0.2.129 | wasm-bindgen@0.2.129: MIT OR Apache-2.0 |
| src/foundation/wasm/browser-bridge-entry | blog-wasm-browser-bridge-entry | wasm-bindgen-macro-support@0.2.129, wasm-bindgen-macro@0.2.129 | wasm-bindgen-macro-support@0.2.129: MIT OR Apache-2.0; wasm-bindgen-macro@0.2.129: MIT OR Apache-2.0 |
| src/foundation/wasm/browser-bridge-schema | blog-wasm-browser-bridge-schema | wasm-bindgen-shared@0.2.129 | wasm-bindgen-shared@0.2.129: MIT OR Apache-2.0 |
| src/foundation/wasm/browser-futures | blog-wasm-browser-futures | wasm-bindgen-futures@0.4.79 | wasm-bindgen-futures@0.4.79: MIT OR Apache-2.0 |
| src/foundation/wasm/browser-module | blog-wasm-browser-module | wasm-bindgen-cli-support@0.2.129 | wasm-bindgen-cli-support@0.2.129: MIT OR Apache-2.0 |
| src/foundation/wasm/debug-information | blog-wasm-debug-information | gimli@0.32.3 | gimli@0.32.3: MIT OR Apache-2.0 |
| src/foundation/wasm/module-decoder | blog-wasm-module-decoder | wasmparser@0.245.1 | wasmparser@0.245.1: Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT；发布包无独立许可文本，待复核 |
| src/foundation/wasm/module-encoder | blog-wasm-module-encoder | wasm-encoder@0.245.1 | wasm-encoder@0.245.1: Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT；发布包无独立许可文本，待复核 |
| src/foundation/wasm/module-transform | blog-wasm-module-transform | walrus@0.27.2 | walrus@0.27.2: MIT/Apache-2.0 |
| src/foundation/wasm/module-transform-derive | blog-wasm-module-transform-derive | walrus-macro@0.26.0 | walrus-macro@0.26.0: MIT/Apache-2.0 |
| src/interface | blog-interface | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |
| src/interface/dom | blog-dom | any_spawner@0.3.0, console_error_panic_hook@0.1.7, const_str_slice_concat@0.1.0, drain_filter_polyfill@0.1.3, either_of@0.1.9, erased@0.1.2, gloo-events@0.2.0, gloo-timers@0.3.0, guardian@1.3.0, html-escape@0.2.15, itertools@0.14.0, leptos_dom@0.8.8, next_tuple@0.1.0, oco_ref@0.2.1, or_poisoned@0.1.0, reactive_graph@0.2.16, send_wrapper@0.6.0, slotmap@1.1.1, tachys@0.2.19, throw_error@0.3.1, typed-builder@0.23.2 | any_spawner@0.3.0: MIT；发布包无独立许可文本，待复核; console_error_panic_hook@0.1.7: Apache-2.0/MIT; const_str_slice_concat@0.1.0: MIT；发布包无独立许可文本，待复核; drain_filter_polyfill@0.1.3: MIT OR Apache-2.0；发布包无独立许可文本，待复核; either_of@0.1.9: MIT；发布包无独立许可文本，待复核; erased@0.1.2: MIT；发布包无独立许可文本，待复核; gloo-events@0.2.0: MIT OR Apache-2.0；发布包无独立许可文本，待复核; gloo-timers@0.3.0: MIT OR Apache-2.0；发布包无独立许可文本，待复核; guardian@1.3.0: MIT OR Apache-2.0; html-escape@0.2.15: MIT; itertools@0.14.0: MIT OR Apache-2.0; leptos_dom@0.8.8: MIT；发布包无独立许可文本，待复核; next_tuple@0.1.0: MIT；发布包无独立许可文本，待复核; oco_ref@0.2.1: MIT；发布包无独立许可文本，待复核; or_poisoned@0.1.0: MIT；发布包无独立许可文本，待复核; reactive_graph@0.2.16: MIT；发布包无独立许可文本，待复核; send_wrapper@0.6.0: MIT/Apache-2.0; slotmap@1.1.1: Zlib; tachys@0.2.19: MIT；发布包无独立许可文本，待复核; throw_error@0.3.1: MIT；发布包无独立许可文本，待复核; typed-builder@0.23.2: MIT OR Apache-2.0 |
| src/interface/macros | blog-component-macros | leptos_hot_reload@0.8.7, paste@1.0.15, typed-builder-macro@0.23.2 | leptos_hot_reload@0.8.7: MIT；发布包无独立许可文本，待复核; paste@1.0.15: MIT OR Apache-2.0; typed-builder-macro@0.23.2: MIT OR Apache-2.0 |
| src/server | blog | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |
| test/browser-domain | blog-browser-domain-tests | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |
| tools/assets | blog-assets | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |
| tools/setup | blog-setup | 项目实现或已有独立来源记录 | 沿用组件独立法律材料 |

## 已退役来源

退役只移除当前实现；原始归档、许可材料及历史仍完整保留。

| 原始身份 | 迁移事件 | 退役原因 |
| --- | --- | --- |
| async-lock@3.4.2 | browser-csr-module-specialization | Remove unselected async resource/transition functionality from the browser CSR kernel; no implementation from this package remains. |
| async-once-cell@0.5.4 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| attribute-derive-macro@0.11.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| attribute-derive@0.11.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| base16@0.2.1 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| byteorder@1.5.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| camino@1.2.6 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| chacha20@0.10.2 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| collection_literals@1.0.3 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| convert_case@0.11.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| convert_case_extras@0.2.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| crc-catalog@2.5.0 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| crc@3.4.0 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| derive-where@1.7.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| dotenvy@0.15.7 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| event-listener-strategy@0.5.4 | browser-csr-module-specialization | Remove unselected async resource/transition functionality from the browser CSR kernel; no implementation from this package remains. |
| hkdf@0.13.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| interpolator@0.5.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| manyhow-macros@0.14.1 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| manyhow@0.14.1 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| md-5@0.11.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| phf_codegen@0.11.3 | browser-csr-module-specialization | No retained product, browser, compiler or test manifest declares this package; retain its original archive and historical maintained-source snapshot. |
| prettyplease@0.3.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| proc-macro-utils@0.10.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| proc-macro2-diagnostics@0.11.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| quote-use-macros@0.9.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| quote-use@0.9.0 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| rand@0.10.3 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| rand_core@0.10.1 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| rstml@0.13.1 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| rustc_version@0.4.1 | browser-csr-module-specialization | Replace the unused macro/casing/import support with the scoped CSR component compiler and independent Unicode name conversion; no implementation from this origin remains. |
| same-file@1.0.6 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| sha1@0.11.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| sha2@0.10.9 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| sqlx-mysql@0.9.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| sqlx-postgres@0.9.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| stringprep@0.1.5 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| tinyvec@1.13.3 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| unicode-bidi@0.3.18 | unused-optional-source-retirement | No supported native, browser, build or test graph selects this source; all incoming unused optional declarations have been removed. |
| unicode-normalization@0.1.25 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| unicode-properties@0.1.4 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| walkdir@2.5.0 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| wasm_split_helpers@0.2.3 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| wasm_split_macros@0.2.2 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| whoami@2.1.3 | purpose-specialization | No remaining product, browser, build or test declaration consumes this implementation after SQLite/CSR specialization. |
| zlib-rs@0.6.8 | browser-csr-module-specialization | No retained product, browser, compiler or test manifest declares this package; retain its original archive and historical maintained-source snapshot. |
