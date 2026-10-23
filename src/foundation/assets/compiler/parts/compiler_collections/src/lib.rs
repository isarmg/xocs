//! Data structures used across other oxc crates.

#[cfg(all())]
mod assert_unchecked;

#[cfg(any())]
pub mod box_macros;

#[cfg(all())]
pub mod branch_hints;

#[cfg(all())]
pub mod code_buffer;

#[cfg(all())]
pub mod fieldless_enum;

#[cfg(all())]
pub mod inline_string;

#[cfg(any())]
pub mod non_null;

#[cfg(any())]
pub mod rope;

#[cfg(all())]
pub mod slice_iter;

#[cfg(all())]
pub mod stack;

#[cfg(any())]
pub mod str;

#[cfg(all())]
pub mod string_ext;

// Gated on `any(test, ...)` because `implements!` is used in unit tests for `non_null` and `stack`,
// and unit tests cannot enable features.
// This enables:
// * `cargo test -p oxc_data_structures --features non_null`
// * `cargo test -p oxc_data_structures --features stack`
#[cfg(any(test, any()))]
pub mod types;

#[doc(hidden)]
pub use crate::{assert_unchecked, fieldless_enum};
#[cfg(test)]
include!("../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/lib/849.rs");
