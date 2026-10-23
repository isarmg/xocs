//! Standard-library primitives used by the concurrent implementation.

pub(crate) use lazy_static::lazy_static;
pub(crate) use std::{
    hint,
    sync::{atomic, Mutex},
    thread::yield_now,
    thread_local,
};

/// Access to a slot remains guarded by its atomic lifecycle protocol.
#[derive(Debug)]
pub(crate) struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

impl<T> UnsafeCell<T> {
    pub fn new(data: T) -> Self {
        Self(std::cell::UnsafeCell::new(data))
    }

    #[inline(always)]
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*const T) -> R,
    {
        f(self.0.get())
    }

    #[inline(always)]
    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut T) -> R,
    {
        f(self.0.get())
    }
}
