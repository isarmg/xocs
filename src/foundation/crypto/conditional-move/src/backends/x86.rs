//! From "AMD64 Architecture Programmer’s Manual, Volume 1: Application Programming"
//! (Rev. 3.23 - October 2020) page 46:
//!
//! <https://www.amd.com/system/files/TechDocs/24592.pdf>
//!
//! > The CMOVcc instructions conditionally copy a word, doubleword, or
//! > quadword from a register or memory location to a register location.
//! > The source and destination must be of the same size.

use crate::{Cmov, CmovEq, Condition};
use core::arch::asm;

macro_rules! cmov {
    ($instruction:expr, $dst:expr, $src:expr, $condition:expr) => {
        unsafe {
            asm! {
                "test {0}, {0}",
                $instruction,
                in(reg_byte) $condition,
                inlateout(reg) *$dst,
                in(reg) *$src,
                options(pure, nomem, nostack),
            };
        }
    };
}

macro_rules! cmov_eq {
    ($xor:expr, $instruction:expr, $lhs:expr, $rhs:expr, $condition:expr, $dst:expr) => {
        let mut tmp = u16::from(*$dst);
        let condition = u16::from($condition);
        unsafe {
            asm! {
                $xor,
                $instruction,
                inout(reg) *$lhs => _,
                in(reg) *$rhs,
                inlateout(reg) tmp,
                in(reg) condition,
                options(pure, nomem, nostack),
            };
        }

        *$dst = (tmp & 0xFF) as u8;
    };
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for u16 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        cmov!("cmovnz {1:e}, {2:e}", self, value, condition);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        cmov!("cmovz {1:e}, {2:e}", self, value, condition);
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for u32 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        cmov!("cmovnz {1:e}, {2:e}", self, value, condition);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        cmov!("cmovz {1:e}, {2:e}", self, value, condition);
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        cmov!("cmovnz {1:r}, {2:r}", self, value, condition);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        cmov!("cmovz {1:r}, {2:r}", self, value, condition);
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl CmovEq for u16 {
    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        cmov_eq!(
            "xor {0:x}, {1:x}",
            "cmovz {2:e}, {3:e}",
            self,
            rhs,
            input,
            output
        );
    }

    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        cmov_eq!(
            "xor {0:x}, {1:x}",
            "cmovnz {2:e}, {3:e}",
            self,
            rhs,
            input,
            output
        );
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl CmovEq for u32 {
    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        cmov_eq!(
            "xor {0:e}, {1:e}",
            "cmovz {2:e}, {3:e}",
            self,
            rhs,
            input,
            output
        );
    }

    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        cmov_eq!(
            "xor {0:e}, {1:e}",
            "cmovnz {2:e}, {3:e}",
            self,
            rhs,
            input,
            output
        );
    }
}

#[cfg_attr(docsrs, doc(cfg(true)))]
impl CmovEq for u64 {
    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        cmov_eq!(
            "xor {0:r}, {1:r}",
            "cmovz {2:r}, {3:r}",
            self,
            rhs,
            input,
            output
        );
    }

    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        cmov_eq!(
            "xor {0:r}, {1:r}",
            "cmovnz {2:r}, {3:r}",
            self,
            rhs,
            input,
            output
        );
    }
}
