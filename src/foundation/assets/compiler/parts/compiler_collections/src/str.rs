//! String utility functions.

/// Compare two strings for equality in a `const` context.
///
/// `str`'s `==` operator is not `const` (it goes through the non-`const` [`PartialEq`] impl),
/// so this byte-for-byte comparison stands in where a compile-time string equality is needed.
///
/// Do not use this except in const context - standard `==` ([`PartialEq`]) has much better performance.
///
/// ```
/// use oxc_data_structures::str::const_str_eq;
///
/// const _: () = assert!(const_str_eq("foo", "foo"));
/// const _: () = assert!(!const_str_eq("foo", "bar"));
/// ```
pub const fn const_str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();

    let len = a.len();

    if b.len() != len {
        return false;
    }

    let mut i = 0;
    while i < len {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }

    true
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/str/tests.rs"]
#[cfg(test)]
mod tests;
