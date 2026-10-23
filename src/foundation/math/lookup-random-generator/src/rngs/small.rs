// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A small fast RNG

use rand_core::{Error, RngCore, SeedableRng};

#[cfg(target_pointer_width = "64")]
type Rng = super::xoshiro256plusplus::Xoshiro256PlusPlus;
#[cfg(not(target_pointer_width = "64"))]
type Rng = super::xoshiro128plusplus::Xoshiro128PlusPlus;

/// A small-state, fast non-cryptographic PRNG.
///
/// Seed the generator explicitly. It is suitable for deterministic lookup-table
/// generation, not for passwords, keys or any application requiring resistance
/// to prediction. [`SeedableRng::seed_from_u64`] expands an integer seed;
/// [`SeedableRng::from_rng`] accepts an independently supplied source.
///
/// The implementation uses Xoshiro256PlusPlus on 64-bit targets and
/// Xoshiro128PlusPlus on 32-bit targets. Do not use its output as a persistent
/// cross-target sequence format.
///
/// ```
/// use blog_math_lookup_random_generator::{Rng, SeedableRng, rngs::SmallRng};
///
/// let mut rng = SmallRng::seed_from_u64(42);
/// let value: u32 = rng.gen();
/// ```

#[cfg_attr(docsrs, doc(cfg(feature = "small_rng")))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmallRng(Rng);

impl RngCore for SmallRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    #[inline(always)]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0.try_fill_bytes(dest)
    }
}

impl SeedableRng for SmallRng {
    type Seed = <Rng as SeedableRng>::Seed;

    #[inline(always)]
    fn from_seed(seed: Self::Seed) -> Self {
        SmallRng(Rng::from_seed(seed))
    }

    #[inline(always)]
    fn from_rng<R: RngCore>(rng: R) -> Result<Self, Error> {
        Rng::from_rng(rng).map(SmallRng)
    }
}
