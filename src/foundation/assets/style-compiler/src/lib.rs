//! Host stylesheet compiler: CSS parsing, optimization and serialization.
#![recursion_limit = "512"]
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
extern crate self as blog_assets_style_compiler;
#[path = "../parts/style_color/lib.rs"]
pub mod style_color;
#[path = "../parts/style_optimizer/src/lib.rs"]
pub mod style_optimizer;
#[path = "../parts/style_parser/src/lib.rs"]
#[macro_use]
pub mod style_parser;
#[path = "../parts/style_selectors/lib.rs"]
pub mod style_selectors;
#[path = "../parts/cached_hash/src/lib.rs"]
pub mod cached_hash;
#[path = "../parts/float_format/src/lib.rs"]
pub mod float_format;
#[path = "../parts/short_float_format/src/lib.rs"]
pub mod short_float_format;
#[path = "../parts/relative_path/src/lib.rs"]
pub mod relative_path;
