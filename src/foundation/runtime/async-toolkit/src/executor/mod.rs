//! Built-in executors and related tools.
//!
//! All asynchronous computation occurs within an executor, which is
//! capable of spawning futures as tasks. This module provides several
//! built-in executors, as well as tools for building your own.
//!
//! All items are only available when the `std` feature of this
//! library is activated, and it is activated by default.
//!
//! # Threaded execution
//!
//! The optional `thread-pool` feature provides `ThreadPool` and its `spawn_ok`
//! method for running tasks on worker threads. The default executor supports
//! single-threaded execution through [`LocalPool`] and [`block_on`].
//!
//! # Spawning additional tasks
//!
//! Tasks can be spawned onto a spawner by calling its [`spawn_obj`] method
//! directly. In the case of `!Send` futures, [`spawn_local_obj`] can be used
//! instead.
//!
//! # Single-threaded execution
//!
//! A task and the tasks it spawns can run entirely within a single thread via
//! the [`LocalPool`] executor.
//! Aside from cutting down on synchronization costs, this executor also makes
//! it possible to spawn non-`Send` tasks, via [`spawn_local_obj`]. The
//! [`LocalPool`] is best suited for running I/O-bound tasks that do relatively
//! little work between I/O operations.
//!
//! There is also a convenience function [`block_on`] for simply running a
//! future to completion on the current thread.
//!
//! [`spawn_obj`]: crate::task::Spawn::spawn_obj
//! [`spawn_local_obj`]: crate::task::LocalSpawn::spawn_local_obj

#![warn(missing_docs, unsafe_op_in_unsafe_fn)]

#[cfg(feature = "std")]
mod local_pool;
#[cfg(feature = "std")]
pub use crate::executor::local_pool::{
    block_on, block_on_stream, BlockingStream, LocalPool, LocalSpawner,
};

#[cfg(feature = "thread-pool")]
#[cfg_attr(docsrs, doc(cfg(feature = "thread-pool")))]
#[cfg(feature = "std")]
mod thread_pool;
#[cfg(feature = "thread-pool")]
#[cfg(feature = "std")]
mod unpark_mutex;
#[cfg(feature = "thread-pool")]
#[cfg_attr(docsrs, doc(cfg(feature = "thread-pool")))]
#[cfg(feature = "std")]
pub use crate::executor::thread_pool::{ThreadPool, ThreadPoolBuilder};

#[cfg(feature = "std")]
mod enter;
#[cfg(feature = "std")]
pub use crate::executor::enter::{enter, Enter, EnterError};
