#![cfg(test)]

use crate::{ThreadPoolBuildError, ThreadPoolBuilder};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

#[cfg(test)]
include!("test/149.rs");

#[cfg(test)]
include!("test/531.rs");

#[cfg(test)]
include!("test/1381.rs");

#[cfg(test)]
include!("test/2301.rs");

#[cfg(test)]
include!("test/3962.rs");

/// Helper used by check_error_send_sync to ensure ThreadPoolBuildError is Send + Sync
fn _send_sync<T: Send + Sync>() {}

#[cfg(test)]
include!("test/4319.rs");

#[cfg(test)]
include!("test/4401.rs");

#[cfg(test)]
include!("test/5058.rs");

#[cfg(test)]
include!("test/5215.rs");
