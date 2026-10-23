//! `likely` and `unlikely` branch prediction hint functions.
//!
//! Also re-exports [`std::hint::cold_path`].

/// Re-export of [`std::hint::cold_path`].
pub use std::hint::cold_path;

/// Hint to the compiler that the condition `cond` is likely to be `true`.
///
/// No-op at runtime. Just hints to compiler how to arrange code so the likely branch
/// is the default. Any alternative branch is a "cold path".
///
/// Returns `cond` unchanged.
///
/// This is a wrapper around [`std::hint::cold_path`].
/// See documentation for [`std::hint::cold_path`] for more details.
///
/// # Example
/// ```
/// use oxc_data_structures::branch_hints::likely;
///
/// fn char_width(ch: char) -> usize {
///     // Source text is expected to be mostly ASCII
///     if likely(ch.is_ascii()) {
///         1
///     } else {
///         ch.len_utf8()
///     }
/// }
/// ```
//
// `#[inline(always)]` because is no-op at runtime, and would have no effect unless inlined
#[expect(clippy::inline_always)]
#[inline(always)]
pub const fn likely(cond: bool) -> bool {
    if !cond {
        cold_path();
    }
    cond
}

/// Hint to the compiler that the condition `cond` is likely to be `false`.
///
/// No-op at runtime. Just hints to compiler how to arrange code so the likely branch
/// is the default. Any alternative branch is a "cold path".
///
/// Returns `cond` unchanged.
///
/// This is a wrapper around [`std::hint::cold_path`].
/// See documentation for [`std::hint::cold_path`] for more details.
///
/// # Example
/// ```
/// use oxc_data_structures::branch_hints::unlikely;
///
/// fn contains_zero(bytes: &[u8]) -> bool {
///     for &byte in bytes {
///         // Most bytes are expected to be non-zero
///         if unlikely(byte == 0) {
///             return true;
///         }
///     }
///     false
/// }
/// ```
//
// `#[inline(always)]` because is no-op at runtime, and would have no effect unless inlined
#[expect(clippy::inline_always)]
#[inline(always)]
pub const fn unlikely(cond: bool) -> bool {
    if cond {
        cold_path();
    }
    cond
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/branch_hints/test.rs"]
#[cfg(test)]
mod test;
