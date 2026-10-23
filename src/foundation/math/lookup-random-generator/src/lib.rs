// Copyright 2018 Developers of the Rand project.
// Copyright 2013-2017 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Random sampling utilities for deterministic lookup-table construction.
//!
//! [`Rng`] extends an explicit [`RngCore`] source with distribution sampling,
//! while [`distributions`] and [`seq`] provide numeric ranges and sequence
//! operations. Supply and seed the generator explicitly; this package does not
//! expose a thread-local generator or a global `random()` function.
//!
//! With `small_rng`, `rngs::SmallRng` supplies a fast non-cryptographic PRNG.
//! It must not be used for passwords, keys or other security-sensitive input.
//! The `alloc` feature enables allocation-backed sampling helpers.
//!
//! ```
//! use blog_math_lookup_random_generator::{Rng, rngs::mock::StepRng};
//!
//! // StepRng is deterministic test data, not a source of secure randomness.
//! let mut rng = StepRng::new(2, 1);
//! assert_eq!(rng.gen::<[u64; 3]>(), [2, 3, 4]);
//! ```

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://rust-random.github.io/rand/"
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![doc(test(attr(allow(unused_variables), deny(warnings))))]
#![no_std]
#![cfg_attr(feature = "nightly", feature(trusted_len))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::float_cmp,
    clippy::neg_cmp_op_on_partial_ord,
)]

#[cfg(feature = "std")] extern crate std;
#[cfg(feature = "alloc")] extern crate alloc;

// Re-exports from rand_core
pub use rand_core::{CryptoRng, Error, RngCore, SeedableRng};

// Public modules
pub mod distributions;
pub mod prelude;
mod rng;
pub mod rngs;
pub mod seq;

// Public exports
#[cfg(all(feature = "std", feature = "std_rng"))]
pub use crate::rngs::thread::thread_rng;
pub use rng::{Fill, Rng};

#[cfg(all(feature = "std", feature = "std_rng"))]
use crate::distributions::{Distribution, Standard};

/// Generates a random value using the thread-local random number generator.
///
/// This is simply a shortcut for `thread_rng().gen()`. See [`thread_rng`] for
/// documentation of the entropy source and [`Standard`] for documentation of
/// distributions and type-specific generation.
///
/// # Provided implementations
///
/// The following types have provided implementations that
/// generate values with the following ranges and distributions:
///
/// * Integers (`i32`, `u32`, `isize`, `usize`, etc.): Uniformly distributed
///   over all values of the type.
/// * `char`: Uniformly distributed over all Unicode scalar values, i.e. all
///   code points in the range `0...0x10_FFFF`, except for the range
///   `0xD800...0xDFFF` (the surrogate code points). This includes
///   unassigned/reserved code points.
/// * `bool`: Generates `false` or `true`, each with probability 0.5.
/// * Floating point types (`f32` and `f64`): Uniformly distributed in the
///   half-open range `[0, 1)`. See notes below.
/// * Wrapping integers (`Wrapping<T>`), besides the type identical to their
///   normal integer variants.
///
/// Also supported is the generation of the following
/// compound types where all component types are supported:
///
/// *   Tuples (up to 12 elements): each element is generated sequentially.
/// *   Arrays (up to 32 elements): each element is generated sequentially;
///     see also [`Rng::fill`] which supports arbitrary array length for integer
///     types and tends to be faster for `u32` and smaller types.
/// *   `Option<T>` first generates a `bool`, and if true generates and returns
///     `Some(value)` where `value: T`, otherwise returning `None`.
///
/// # Examples
///
/// ```
/// let x = rand::random::<u8>();
/// println!("{}", x);
///
/// let y = rand::random::<f64>();
/// println!("{}", y);
///
/// if rand::random() { // generates a boolean
///     println!("Better lucky than good!");
/// }
/// ```
///
/// If you're calling `random()` in a loop, caching the generator as in the
/// following example can increase performance.
///
/// ```
/// use rand::Rng;
///
/// let mut v = vec![1, 2, 3];
///
/// for x in v.iter_mut() {
///     *x = rand::random()
/// }
///
/// // can be made faster by caching thread_rng
///
/// let mut rng = rand::thread_rng();
///
/// for x in v.iter_mut() {
///     *x = rng.gen();
/// }
/// ```
///
/// [`Standard`]: distributions::Standard
#[cfg(all(feature = "std", feature = "std_rng"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "std_rng"))))]
#[inline]
pub fn random<T>() -> T
where Standard: Distribution<T> {
    thread_rng().gen()
}

#[path = "../../../../../test/foundation/math/lookup-random-generator/unit/src/lib/test.rs"]
#[cfg(test)]
mod test;
