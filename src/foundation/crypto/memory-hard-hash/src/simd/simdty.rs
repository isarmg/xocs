// Copyright 2016 blake2-rfc Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(non_camel_case_types)]

use crate::as_bytes::Safe;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Simd4<T>(pub T, pub T, pub T, pub T);

pub type u32x4 = Simd4<u32>;
pub type u64x4 = Simd4<u64>;

impl<T> Simd4<T> {
    #[inline(always)]
    pub fn new(e0: T, e1: T, e2: T, e3: T) -> Simd4<T> {
        Simd4(e0, e1, e2, e3)
    }
}

unsafe impl<T: Safe> Safe for Simd4<T> {}
