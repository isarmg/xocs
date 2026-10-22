# Native source integration

`crates/blog-core` is maintained inside this repository. Its original source is
xcss commit `d9f6f2a900d85eac0bb8c53575ccd4ca09890514`; it is not downloaded or
linked as an external Git package. `manifest.json` records each selected original
file, its original SHA-256, the local destination, current SHA-256, and adaptation notes.
`modifications.patch` records the exact original-to-local changes. The
Apache-2.0 license is retained byte for byte in `crates/blog-core/LICENSE`.
No root NOTICE file exists in that pinned source.

Authentication, CSRF and origin enforcement, private-file identity checks,
SQLite limits and snapshot validation, instance locks, bounded HTTP transport,
shutdown draining, and embedded-resource verification remain implemented by the
selected original modules. The local crate is Linux x86_64 GNU only. Testkit is
compiled only for its own unit tests. Unused platform lifecycle, encrypted secret
handling, Hyper admin adapter, operations client and Windows logging are omitted.

The product uses `blog_core::` directly. Installation source descriptors reference
local paths; Web inventories are `blog-web-assets.rs`; generated Rust refers to
`blog_core::web_assets`. Local adaptations and additional modifications must stay
recorded in the manifest. The original upstream author and license notices remain
in the copied source and license.
