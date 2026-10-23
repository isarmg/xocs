//! Portable "best effort" implementation of `Cmov`/`CmovEq`.
//!
//! This implementation is based on portable arithmetic operations augmented with tactical usage of
//! `core::hint::black_box` based on past observations of where the optimizer has inserted branches
//! (see CVE-2026-23519), but the fully portable implementation cannot guarantee that the resulting
//! generated assembly is free of branch instructions.
//!

use crate::{Cmov, CmovEq, Condition};
use core::ops::{BitAnd, BitOr, Not};

#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for u16 {
    #[inline]
    fn cmovnz(&mut self, value: &u16, condition: Condition) {
        *self = masksel(*self, *value, (masknz32(condition.into()) & 0xFFFF) as u16);
    }

    #[inline]
    fn cmovz(&mut self, value: &u16, condition: Condition) {
        *self = masksel(*self, *value, (!masknz32(condition.into()) & 0xFFFF) as u16);
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for u32 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, masknz32(condition.into()));
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, !masknz32(condition.into()));
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, masknz64(condition.into()));
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, !masknz64(condition.into()));
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl CmovEq for u16 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(
            *output,
            input,
            (masknz32((*self ^ *rhs).into()) & 0xFF) as Condition,
        );
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(
            *output,
            input,
            (!masknz32((*self ^ *rhs).into()) & 0xFF) as Condition,
        );
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskne32(*self, *rhs) & 0xFF) as Condition);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskeq32(*self, *rhs) & 0xFF) as Condition);
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskne64(*self, *rhs) & 0xFF) as Condition);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskeq64(*self, *rhs) & 0xFF) as Condition);
    }
}

/// Returns `u32::MAX` if `x` is equal to `y`, otherwise returns `0` (32-bit version)
#[inline]
fn maskeq32(x: u32, y: u32) -> u32 {
    !maskne32(x, y)
}

/// Returns `u64::MAX` if `x` is equal to `y`, otherwise returns `0` (64-bit version)
#[inline]
fn maskeq64(x: u64, y: u64) -> u64 {
    !maskne64(x, y)
}

/// Returns `0` if `x` is equal to `y`, otherwise returns `u32::MAX` (32-bit version)
#[inline]
fn maskne32(x: u32, y: u32) -> u32 {
    masknz32(x ^ y)
}

/// Returns `0` if `x` is equal to `y`, otherwise returns `u64::MAX` (64-bit version)
#[inline]
fn maskne64(x: u64, y: u64) -> u64 {
    masknz64(x ^ y)
}

/// Return a `u32::MAX` mask if `condition` is non-zero, otherwise return zero for a zero input.
fn masknz32(condition: u32) -> u32 {
    masknz!(condition: u32)
}

/// Return a `u64::MAX` mask if `condition` is non-zero, otherwise return zero for a zero input.
fn masknz64(condition: u64) -> u64 {
    masknz!(condition: u64)
}

/// Given a supplied mask of `0` or all 1-bits (i.e. `u*::MAX`), select `a` if the mask is all-zeros
/// and `b` if the mask is all-ones.
///
/// This function shouldn't be used with a mask that isn't `0` or `u*::MAX`.
#[inline]
fn masksel<T>(a: T, b: T, mask: T) -> T
where
    T: BitAnd<Output = T> + BitOr<Output = T> + Copy + Not<Output = T>,
{
    (a & !mask) | (b & mask)
}

#[path = "../../../../../../test/foundation/crypto/conditional-move/unit/src/backends/soft/tests.rs"]
#[cfg(test)]
mod tests;
