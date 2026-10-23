# Date and time

`blog-format-date-time` maintains calendar arithmetic, parsing, formatting, and
local-time conversion for this repository. It is derived from Chrono 0.4.45 and
uses the workspace's Rust 1.99.0 toolchain. The supported product targets are
Linux AMD64 GNU/glibc and browser `wasm32-unknown-unknown`.

The default features are `clock`, `std`, and `wasmbind`. `std` enables `alloc`,
and `clock` enables `now`. The optional `serde` feature uses the repository's
data-model package. `core-error` exposes error trait implementations when `std`
is disabled; `__internal_bench` is an internal benchmark hook. These switches
do not expand the product's supported target set.

On Linux, `Local` reads `TZ` or the system timezone files and parses timezone
transitions in Rust. Timezone discovery falls back to UTC when no usable data is
available. On browsers, `wasmbind` uses JavaScript `Date`; its local-time mapping
does not distinguish ambiguous or nonexistent times at daylight-saving
transitions. UTC and fixed offsets avoid that browser-specific limitation.

Formatting uses the built-in English month and weekday names. Localized
formatting, rkyv, arbitrary-value generation, and the oldtime feature are not
available. The package ships no bundled timezone database and makes no promise
of compatibility with upstream extension crates or old public APIs.

Use `TimeDelta`, `DateTime`, and the checked date/time constructors in new
consumers. Remaining inherited aliases and deprecated methods are not a
compatibility commitment. The Rust source documents the maintained API; the
README under `docs/upstream/chrono-0.4.45` describes the original upstream
release only. Original archives and licenses remain under `compliance/`.
