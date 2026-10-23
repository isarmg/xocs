// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Random number sources and adapters for explicit sampling.
//!
//! All generators implement [`RngCore`](crate::RngCore), which receives the
//! [`Rng`](crate::Rng) sampling extension automatically. Seedable generators
//! implement [`SeedableRng`](crate::SeedableRng).
//!
//! [`mock::StepRng`] generates an arithmetic sequence for tests. With the
//! `small_rng` feature, `SmallRng` supplies a small-state, non-cryptographic
//! generator seeded by the caller. Its algorithm depends on pointer width;
//! do not use it as a persistent sequence format or for security-sensitive data.
//!
//! This package does not provide a thread-local RNG or `StdRng`. The optional
//! `getrandom` feature exposes the random-trait dependency's `OsRng`; product
//! entropy and password generation use the dedicated system-entropy layer.

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")] pub mod adapter;

pub mod mock; // Public so we don't export `StepRng` directly, making it a bit
              // more clear it is intended for testing.

#[cfg(all(feature = "small_rng", target_pointer_width = "64"))]
mod xoshiro256plusplus;
#[cfg(all(feature = "small_rng", not(target_pointer_width = "64")))]
mod xoshiro128plusplus;
#[cfg(feature = "small_rng")] mod small;

#[cfg(feature = "std_rng")] mod std;
#[cfg(all(feature = "std", feature = "std_rng"))] pub(crate) mod thread;

#[cfg(feature = "small_rng")] pub use self::small::SmallRng;
#[cfg(feature = "std_rng")] pub use self::std::StdRng;
#[cfg(all(feature = "std", feature = "std_rng"))] pub use self::thread::ThreadRng;

#[cfg_attr(docsrs, doc(cfg(feature = "getrandom")))]
#[cfg(feature = "getrandom")] pub use rand_core::OsRng;
