//! This crate makes it easier to write asynchronous code that is executor-agnostic, by providing a
//! utility that can be used to spawn tasks in a variety of executors.
//!
//! Browser dispatch uses one global executor configured at runtime. Native adapters may install
//! an override for their current thread; threads without an override use the global executor.
//!
//! This can be extended to support any executor or runtime that supports spawning [`Future`]s.
//!
//! This is a least common denominator implementation in many ways. Limitations include:
//! - installing each global or native thread-local executor is a one-time action
//! - no "join handle" or other result is returned from the spawn
//! - the `Future` must output `()`
//!
//! ```no_run
//! use blog_dom::tasks::Executor;
//!
//! // make sure an Executor has been initialized with one of the init_ functions
//!
//! // spawn a thread-safe Future
//! Executor::spawn(async { /* ... */ });
//!
//! // spawn a Future that is !Send
//! Executor::spawn_local(async { /* ... */ });
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{future::Future, pin::Pin, sync::OnceLock};
use thiserror::Error;

/// A future that has been pinned.
pub type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
/// A future that has been pinned.
pub type PinnedLocalFuture<T> = Pin<Box<dyn Future<Output = T>>>;

// Type alias for the spawn function pointer.
type SpawnFn = fn(PinnedFuture<()>);
// Type alias for the spawn_local function pointer.
type SpawnLocalFn = fn(PinnedLocalFuture<()>);
// Type alias for the poll_local function pointer.
type PollLocalFn = fn();

/// Holds the function pointers for the current global executor.
#[derive(Clone, Copy)]
struct ExecutorFns {
    spawn: SpawnFn,
    spawn_local: SpawnLocalFn,
    poll_local: PollLocalFn,
}

// Use a single OnceLock to ensure atomic initialization of all functions.
static EXECUTOR_FNS: OnceLock<ExecutorFns> = OnceLock::new();

// Browser task dispatch stays on its fixed global vtable. Native adapters may
// override the executor per thread without changing another thread's backend.
#[cfg(not(target_family = "wasm"))]
thread_local! {
    static LOCAL_CUSTOM_EXECUTOR: std::cell::OnceCell<Box<dyn CustomExecutor>> = const {
        std::cell::OnceCell::new()
    };
}

// No-op functions to use when an executor doesn't support a specific operation.

#[cold]
#[inline(never)]
fn no_op_poll() {}

// Wasm panics if you spawn without an executor

#[cold]
#[inline(never)]
fn no_op_spawn(_: PinnedFuture<()>) {
    panic!("Executor::spawn called, but no global 'spawn' function is configured.");
}

#[cfg(not(debug_assertions))]
#[cold]
#[inline(never)]
fn no_op_spawn_local(_: PinnedLocalFuture<()>) {
    panic!(
        "Executor::spawn_local called, but no global 'spawn_local' function \
         is configured."
    );
}

/// Errors that can occur when using the executor.
#[derive(Error, Debug)]
pub enum ExecutorError {
    /// The selected global or current-thread executor has already been set.
    #[error("Executor has already been set for this scope.")]
    AlreadySet,
}

/// A global async executor that can spawn tasks.
pub struct Executor;

impl Executor {
    /// Spawns a thread-safe [`Future`].
    ///
    /// Native callers use this thread's override when installed, otherwise the
    /// global executor. Browser callers use the global executor directly.
    /// Panics if no applicable executor has been initialized.
    #[inline(always)]
    #[track_caller]
    pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
        let pinned_fut = Box::pin(fut);
        #[cfg(not(target_family = "wasm"))]
        let Some(pinned_fut) = LOCAL_CUSTOM_EXECUTOR.with(|local| {
            if let Some(executor) = local.get() {
                executor.spawn(pinned_fut);
                None
            } else {
                Some(pinned_fut)
            }
        }) else {
            return;
        };

        if let Some(fns) = EXECUTOR_FNS.get() {
            (fns.spawn)(pinned_fut)
        } else {
            // No global executor set.
            handle_uninitialized_spawn(pinned_fut);
        }
    }

    /// Spawns a [`Future`] that cannot be sent across threads.
    ///
    /// Native callers use this thread's override when installed, otherwise the
    /// global executor. Browser callers use the global executor directly.
    /// Panics if no applicable executor has been initialized.
    #[inline(always)]
    #[track_caller]
    pub fn spawn_local(fut: impl Future<Output = ()> + 'static) {
        let pinned_fut = Box::pin(fut);
        #[cfg(not(target_family = "wasm"))]
        let Some(pinned_fut) = LOCAL_CUSTOM_EXECUTOR.with(|local| {
            if let Some(executor) = local.get() {
                executor.spawn_local(pinned_fut);
                None
            } else {
                Some(pinned_fut)
            }
        }) else {
            return;
        };

        if let Some(fns) = EXECUTOR_FNS.get() {
            (fns.spawn_local)(pinned_fut)
        } else {
            // No global executor set.
            handle_uninitialized_spawn_local(pinned_fut);
        }
    }

    /// Waits until the next "tick" of the current async executor.
    /// Uses this thread's native override when installed, otherwise the global executor.
    #[inline(always)]
    pub async fn tick() {
        let (tx, rx) = futures::channel::oneshot::channel();
        #[cfg(not(target_family = "wasm"))]
        Executor::spawn(async move {
            _ = tx.send(());
        });
        #[cfg(target_family = "wasm")]
        Executor::spawn_local(async move {
            _ = tx.send(());
        });

        _ = rx.await;
    }

    /// Polls the global async executor.
    ///
    /// Native callers use this thread's override when installed, otherwise the
    /// global executor. Browser callers use the global executor directly.
    /// Does nothing when no executor is installed or it does not support polling.
    #[inline(always)]
    pub fn poll_local() {
        #[cfg(not(target_family = "wasm"))]
        if LOCAL_CUSTOM_EXECUTOR.with(|local| {
            if let Some(executor) = local.get() {
                executor.poll_local();
                true
            } else {
                false
            }
        }) {
            return;
        }
        if let Some(fns) = EXECUTOR_FNS.get() {
            (fns.poll_local)()
        }
        // If not initialized or doesn't support polling, do nothing gracefully.
    }
}

impl Executor {
    /// Globally sets the [`wasm-bindgen-futures`] runtime as the executor used to spawn tasks.
    ///
    /// Returns `Err(_)` if a global executor has already been set.
    ///
    /// Requires the `wasm-bindgen` feature to be activated on this crate.
    #[cfg_attr(docsrs, doc(cfg(all())))]
    pub fn init_wasm_bindgen() -> Result<(), ExecutorError> {
        let executor_impl = ExecutorFns {
            // wasm-bindgen-futures only supports spawn_local
            spawn: no_op_spawn,
            spawn_local: |fut| {
                wasm_bindgen_futures::spawn_local(fut);
            },
            poll_local: no_op_poll,
        };
        EXECUTOR_FNS
            .set(executor_impl)
            .map_err(|_| ExecutorError::AlreadySet)
    }

    /// Globally sets a custom executor as the executor used to spawn tasks.
    ///
    /// Requires the custom executor to be `Send + Sync` as it will be stored statically.
    ///
    /// Returns `Err(_)` if a global executor has already been set.
    pub fn init_custom_executor(
        custom_executor: impl CustomExecutor + Send + Sync + 'static,
    ) -> Result<(), ExecutorError> {
        // Store the custom executor instance itself to call its methods.
        // Use Box for dynamic dispatch.
        static CUSTOM_EXECUTOR_INSTANCE: OnceLock<Box<dyn CustomExecutor + Send + Sync>> =
            OnceLock::new();

        CUSTOM_EXECUTOR_INSTANCE
            .set(Box::new(custom_executor))
            .map_err(|_| ExecutorError::AlreadySet)?;

        // Now set the ExecutorFns using the stored instance
        let executor_impl = ExecutorFns {
            spawn: |fut| {
                // Unwrap is safe because we just set it successfully or returned Err.
                CUSTOM_EXECUTOR_INSTANCE.get().unwrap().spawn(fut);
            },
            spawn_local: |fut| {
                CUSTOM_EXECUTOR_INSTANCE.get().unwrap().spawn_local(fut);
            },
            poll_local: || {
                CUSTOM_EXECUTOR_INSTANCE.get().unwrap().poll_local();
            },
        };

        EXECUTOR_FNS
            .set(executor_impl)
            .map_err(|_| ExecutorError::AlreadySet)
        // If setting EXECUTOR_FNS fails (extremely unlikely race if called *concurrently*
        // with another init_* after CUSTOM_EXECUTOR_INSTANCE was set), we technically
        // leave CUSTOM_EXECUTOR_INSTANCE set but EXECUTOR_FNS not. This is an edge case,
        // but the primary race condition is solved.
    }

    /// Sets a native custom executor for the current thread only.
    ///
    /// Calls made from this thread prefer this executor. Other threads continue
    /// using their own override or the global executor. This entry point is for
    /// native contract adapters; browser dispatch uses its global vtable directly.
    ///
    /// The executor does not need to be `Send` or `Sync`. No mutable TLS borrow is
    /// held while invoking it, so callbacks may reenter the dispatch functions.
    /// Returns [`ExecutorError::AlreadySet`] only if this thread already has an override.
    #[cfg(not(target_family = "wasm"))]
    pub fn init_local_custom_executor(
        custom_executor: impl CustomExecutor + 'static,
    ) -> Result<(), ExecutorError> {
        LOCAL_CUSTOM_EXECUTOR.with(|local| {
            local
                .set(Box::new(custom_executor))
                .map_err(|_| ExecutorError::AlreadySet)
        })
    }
}

/// A trait for custom executors.
/// Custom executors can be used to integrate with any executor that supports spawning futures.
///
/// If used with `init_custom_executor`, the implementation must be `Send + Sync + 'static`.
///
/// All methods can be called recursively. Implementors should be mindful of potential
/// deadlocks or excessive resource consumption if recursive calls are not handled carefully
/// (e.g., using `try_borrow_mut` or non-blocking polls within implementations).
pub trait CustomExecutor {
    /// Spawns a future, usually on a thread pool.
    fn spawn(&self, fut: PinnedFuture<()>);
    /// Spawns a local future. May require calling `poll_local` to make progress.
    fn spawn_local(&self, fut: PinnedLocalFuture<()>);
    /// Polls the executor, if it supports polling. Implementations should ideally be
    /// non-blocking or use mechanisms like `try_tick` or `try_borrow_mut` to handle
    /// re-entrant calls safely.
    fn poll_local(&self);
}

// Ensure CustomExecutor is object-safe
#[allow(dead_code)]
fn test_object_safety(_: Box<dyn CustomExecutor + Send + Sync>) {} // Added Send + Sync constraint here for global usage

/// Handles the case where `Executor::spawn` is called without an initialized executor.
#[cold] // Less likely path
#[inline(never)]
#[track_caller]
fn handle_uninitialized_spawn(_fut: PinnedFuture<()>) {
    #[cfg(debug_assertions)]
    {
        let caller = std::panic::Location::caller();
        panic!(
            "At {caller}, tried to spawn a Future with Executor::spawn() \
             before a global executor was initialized."
        );
    }
    // In release builds (without tracing), call the specific no-op function.
    #[cfg(not(debug_assertions))]
    {
        no_op_spawn(_fut);
    }
}

/// Handles the case where `Executor::spawn_local` is called without an initialized executor.
#[cold] // Less likely path
#[inline(never)]
#[track_caller]
fn handle_uninitialized_spawn_local(_fut: PinnedLocalFuture<()>) {
    #[cfg(debug_assertions)]
    {
        let caller = std::panic::Location::caller();
        panic!(
            "At {caller}, tried to spawn a Future with \
             Executor::spawn_local() before a global executor was initialized."
        );
    }
    // In release builds (without tracing), call the specific no-op function (which usually panics).
    #[cfg(not(debug_assertions))]
    {
        no_op_spawn_local(_fut);
    }
}

#[cfg(not(target_family = "wasm"))]
impl Executor {
    /// Installs the local Futures executor for native examples and CSR contract tests.
    ///
    /// Call [`Executor::poll_local`] to drive pending tasks. Each thread owns its pool;
    /// browser builds continue to use [`Executor::init_wasm_bindgen`].
    pub fn init_native() -> Result<(), ExecutorError> {
        use futures::{
            executor::{LocalPool, LocalSpawner},
            task::LocalSpawnExt,
        };
        use std::cell::RefCell;
        thread_local! {
            static POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
            static SPAWNER: LocalSpawner = POOL.with(|pool| pool.borrow().spawner());
        }
        let spawn: SpawnLocalFn = |future| {
            SPAWNER.with(|spawner| {
                spawner
                    .spawn_local(future)
                    .expect("native local executor rejected task");
            })
        };
        EXECUTOR_FNS
            .set(ExecutorFns {
                spawn: |future| {
                    SPAWNER.with(|spawner| {
                        spawner
                            .spawn_local(future)
                            .expect("native local executor rejected task");
                    })
                },
                spawn_local: spawn,
                poll_local: || {
                    POOL.with(|pool| {
                        if let Ok(mut pool) = pool.try_borrow_mut() {
                            pool.run_until_stalled();
                        }
                    })
                },
            })
            .map_err(|_| ExecutorError::AlreadySet)
    }
}
