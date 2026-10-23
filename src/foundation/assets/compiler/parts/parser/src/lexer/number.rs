//! Parsing utilities for converting Javascript numbers to Rust f64.
//! Code copied originally from
//! [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/parser/src/numeric_value.rs)
//! but iterated on since.

use std::borrow::Cow;

use crate::borrowed_text::CowUtils;
use num_bigint::BigInt;
use num_traits::Num;

use crate::syntax_arena::Allocator;
use crate::compiler_strings::{Str, format_str};

use super::kind::Kind;

pub fn parse_int(s: &str, kind: Kind, has_sep: bool) -> Result<f64, &'static str> {
    match kind {
        Kind::Decimal => {
            Ok(if has_sep { parse_decimal_with_underscores(s) } else { parse_decimal(s) })
        }
        Kind::Binary => {
            let s = &s[2..];
            Ok(if has_sep { parse_binary_with_underscores(s) } else { parse_binary(s) })
        }
        Kind::Octal => {
            // Octals always begin with `0`. Trim off leading `0`, `0o` or `0O`.
            let second_byte = s.as_bytes()[1];
            let s = if second_byte == b'o' || second_byte == b'O' {
                // SAFETY: We just checked that 2nd byte is ASCII, so slicing off 2 bytes
                // must be in bounds and on a UTF-8 character boundary.
                unsafe { s.get_unchecked(2..) }
            } else {
                &s[1..] // legacy octal
            };
            Ok(if has_sep { parse_octal_with_underscores(s) } else { parse_octal(s) })
        }
        Kind::Hex => {
            let s = &s[2..];
            Ok(if has_sep { parse_hex_with_underscores(s) } else { parse_hex(s) })
        }
        _ => unreachable!(),
    }
}

pub fn parse_float(s: &str, has_sep: bool) -> Result<f64, &'static str> {
    let s = if has_sep { s.cow_replace('_', "") } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));
    s.parse::<f64>().map_err(|_| "invalid float")
}

// ==================================== DECIMAL ====================================

/// b'0' is 0x30 and b'9' is 0x39.
///
/// So we can convert from any decimal digit to its value with `b & 15`.
/// This produces more compact assembly than `b - b'0'`.
///
/// <https://godbolt.org/z/WMarz15sq>
#[inline]
const fn decimal_byte_to_value(b: u8) -> u8 {
    debug_assert!(b >= b'0' && b <= b'9');
    b & 15
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_decimal(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    /// `u64::MAX + 1` in decimal is 18446744073709551616 (20 chars).
    const MAX_FAST_DECIMAL_LEN: usize = 19;

    debug_assert_ne!(s, "");
    if s.len() > MAX_FAST_DECIMAL_LEN {
        return parse_decimal_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        // The latency of the multiplication can be hidden by issuing it
        // before the result is needed to improve performance on
        // modern out-of-order CPU as multiplication here is slower
        // than the other instructions, we can get the end result faster
        // doing multiplication first and let the CPU spends other cycles
        // doing other computation and get multiplication result later.
        result *= 10;
        let n = decimal_byte_to_value(b);
        result += n as u64;
    }
    result as f64
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_decimal_with_underscores(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    /// `u64::MAX + 1` in decimal is 18446744073709551616 (20 chars).
    const MAX_FAST_DECIMAL_LEN: usize = 19;

    debug_assert_ne!(s, "");
    if s.len() > MAX_FAST_DECIMAL_LEN {
        return parse_decimal_slow(&s.cow_replace('_', ""));
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            result *= 10;
            let n = decimal_byte_to_value(b);
            result += n as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_decimal_slow(s: &str) -> f64 {
    s.parse::<f64>().unwrap()
}

// ==================================== BINARY ====================================

/// b'0' is 0x30 and b'1' is 0x31.
///
/// So we can convert from binary digit to its value with `b & 1`.
/// This is produces more compact assembly than `b - b'0'`.
///
/// <https://godbolt.org/z/1vvrK78jf>
#[inline]
const fn binary_byte_to_value(b: u8) -> u8 {
    debug_assert!(b == b'0' || b == b'1');
    b & 1
}

/// NOTE: bit shifting here is safe and much faster than f64::mul_add.
/// It's safe because we're sure this number is an integer - if it wasn't, it
/// would be a [`Kind::Float`] instead. It's fast because shifting usually takes
/// 1 clock cycle on the ALU, while multiplication+addition uses the FPU and is
/// much slower. Additionally, this loop often gets unrolled by rustc since
/// these numbers are usually not long. On x84_64, FMUL has a latency of 4 clock
/// cycles, which doesn't include addition. Some platforms support mul + add in a
/// single instruction, but many others do not.
///
/// Unfortunately, this approach has the chance to overflow for excessively
/// large numbers. In such cases, we fall back to `parse_nondecimal_slow`.
/// Leading zeros count toward this length; checking and stripping them may not
/// be worth the performance cost. Further experimentation could be useful.
#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_binary(s: &str) -> f64 {
    /// binary literals longer than this many characters have the chance to
    /// overflow a u64, forcing us to take the slow path.
    const MAX_FAST_BINARY_LEN: usize = 64;

    debug_assert_ne!(s, "");
    debug_assert!(!s.starts_with("0b") && !s.starts_with("0B"));

    if s.len() > MAX_FAST_BINARY_LEN {
        return parse_binary_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        result <<= 1;
        result |= binary_byte_to_value(b) as u64;
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_binary_slow(s: &str) -> f64 {
    parse_nondecimal_slow::<1>(s.bytes().map(binary_byte_to_value))
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_binary_with_underscores(s: &str) -> f64 {
    /// binary literals longer than this many characters have the chance to
    /// overflow a u64, forcing us to take the slow path.
    const MAX_FAST_BINARY_LEN: usize = 64;

    debug_assert_ne!(s, "");
    debug_assert!(!s.starts_with("0b") && !s.starts_with("0B"));

    if s.len() > MAX_FAST_BINARY_LEN {
        return parse_binary_with_underscores_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            result <<= 1;
            result |= binary_byte_to_value(b) as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_binary_with_underscores_slow(s: &str) -> f64 {
    parse_nondecimal_slow::<1>(s.bytes().filter(|&b| b != b'_').map(binary_byte_to_value))
}

// ==================================== OCTAL ====================================

/// b'0' is 0x30 and b'7' is 0x37.
///
/// So we can convert from any octal digit to its value with `b & 7`.
/// This is produces more compact assembly than `b - b'0'`.
///
/// <https://godbolt.org/z/9rYTsMoMM>
#[inline]
const fn octal_byte_to_value(b: u8) -> u8 {
    debug_assert!(b >= b'0' && b <= b'7');
    b & 7
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_octal(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    const MAX_FAST_OCTAL_LEN: usize = 21;

    debug_assert_ne!(s, "");
    debug_assert!(!s.starts_with("0o") && !s.starts_with("0O"));
    if s.len() > MAX_FAST_OCTAL_LEN {
        return parse_octal_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        let n = octal_byte_to_value(b);
        result <<= 3;
        result |= n as u64;
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_octal_slow(s: &str) -> f64 {
    parse_nondecimal_slow::<3>(s.bytes().map(octal_byte_to_value))
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_octal_with_underscores(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    const MAX_FAST_OCTAL_LEN: usize = 21;

    debug_assert_ne!(s, "");
    debug_assert!(!s.starts_with("0o") && !s.starts_with("0O"));
    if s.len() > MAX_FAST_OCTAL_LEN {
        return parse_octal_with_underscores_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let n = octal_byte_to_value(b);
            result <<= 3;
            result |= n as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_octal_with_underscores_slow(s: &str) -> f64 {
    parse_nondecimal_slow::<3>(s.bytes().filter(|&b| b != b'_').map(octal_byte_to_value))
}

// ==================================== HEX ====================================

/// - b'0' is 0x30 and b'9' is 0x39.
/// - b'A' is 0x41 and b'F' is 0x46.
/// - b'a' is 0x61 and b'f' is 0x66.
///
/// So we can convert from a digit to its value with `b & 15`,
/// and from `A-F` or `a-f` to its value with `(b & 15) + 9`.
/// We could use `(b & 7) + 9` for `A-F`, but we use `(b & 15) + 9`
/// so that both branches share the same `b & 15` operation.
///
/// This is produces more slightly more assembly than explicitly matching all possibilities,
/// but only because compiler unrolls the loop.
///
/// <https://godbolt.org/z/5fsdv8rGo>
#[inline]
const fn hex_byte_to_value(b: u8) -> u8 {
    debug_assert!((b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'F') || (b >= b'a' && b <= b'f'));
    if b < b'A' {
        b & 15 // 0-9
    } else {
        (b & 15) + 9 // A-F or a-f
    }
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_hex(s: &str) -> f64 {
    /// Hex strings longer than this have the chance to overflow u64.
    const MAX_FAST_HEX_LEN: usize = 16;

    debug_assert_ne!(s, "");
    debug_assert!(!s.starts_with("0x"));

    if s.len() > MAX_FAST_HEX_LEN {
        return parse_hex_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        let n = hex_byte_to_value(b);
        result <<= 4;
        result |= n as u64;
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_hex_slow(s: &str) -> f64 {
    parse_nondecimal_slow::<4>(s.bytes().map(hex_byte_to_value))
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_hex_with_underscores(s: &str) -> f64 {
    /// Hex strings longer than this have the chance to overflow u64.
    const MAX_FAST_HEX_LEN: usize = 16;

    debug_assert_ne!(s, "");
    debug_assert!(!s.starts_with("0x"));

    if s.len() > MAX_FAST_HEX_LEN {
        return parse_hex_with_underscores_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let n = hex_byte_to_value(b);
            result <<= 4;
            result |= n as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_hex_with_underscores_slow(s: &str) -> f64 {
    parse_nondecimal_slow::<4>(s.bytes().filter(|&b| b != b'_').map(hex_byte_to_value))
}

/// Convert power-of-two radix digits with a single rounding to nearest, ties to even.
#[cold]
#[expect(clippy::cast_precision_loss)]
fn parse_nondecimal_slow<const BITS: u32>(digits: impl Iterator<Item = u8>) -> f64 {
    let mut significand = 0_u64;
    let mut exponent = 0_u32;
    let mut sticky = 0_u8;
    for digit in digits {
        if significand < (1 << 54) {
            significand = (significand << BITS) | u64::from(digit);
        } else {
            exponent += BITS;
            if exponent > 1023 {
                return f64::INFINITY;
            }
            sticky |= digit;
        }
    }

    // Retain at least 55 significant bits: the 53 f64 bits, a rounding bit,
    // and a sticky bit. Any nonzero discarded digit makes the lowest retained
    // bit sticky, distinguishing an exact halfway value from one just above it.
    // The integer cast rounds once; scaling by a power of two is exact unless
    // the result overflows to infinity.
    significand |= u64::from(sticky != 0);
    (significand as f64) * f64::from_bits(u64::from(exponent + 1023) << 52)
}

// ==================================== BIGINT ====================================

pub fn parse_big_int<'a>(
    s: &'a str,
    kind: Kind,
    has_sep: bool,
    allocator: &'a Allocator,
) -> Str<'a> {
    let s = if has_sep { s.cow_replace('_', "") } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));

    let radix = match kind {
        // Skip parsing with `BigInt` - it's already in decimal form, and underscores are removed
        Kind::Decimal => return Str::from_cow_in(&s, &allocator),
        Kind::Binary => 2,
        Kind::Octal => 8,
        Kind::Hex => 16,
        _ => unreachable!(),
    };

    let s = &s[2..];

    // NOTE: BigInt::from_bytes does a UTF8 check, then uses from_str_radix under the hood.
    // We already have a string, so we can just use that directly.
    // Lexer already checked `s` represents a valid BigInt, so `unwrap` cannot fail.
    let bigint = BigInt::from_str_radix(s, radix).unwrap();
    format_str!(allocator, "{bigint}")
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/parser/src/lexer/number/test.rs"]
#[cfg(test)]
#[expect(clippy::unreadable_literal, clippy::mixed_case_hex_literals)]
mod test;
