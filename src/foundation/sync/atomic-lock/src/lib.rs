#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]
#![cfg_attr(not(test), no_std)]

//! A light-weight lock guarded by an atomic boolean.
//!
//! Most efficient when contention is low, acquiring the lock is a single
//! atomic swap, and releasing it just 1 more atomic swap.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use blog_sync_atomic_lock::TryLock;
//!
//! // a thing we want to share
//! struct Widget {
//!     name: String,
//! }
//!
//! // lock it up!
//! let widget1 = Arc::new(TryLock::new(Widget {
//!     name: "Spanner".into(),
//! }));
//!
//! let widget2 = widget1.clone();
//!
//!
//! // mutate the widget
//! let mut locked = widget1.try_lock().expect("example isn't locked yet");
//! locked.name.push_str(" Bundle");
//!
//! // hands off, buddy
//! let not_locked = widget2.try_lock();
//! assert!(not_locked.is_none(), "widget1 has the lock");
//!
//! // ok, you can have it
//! drop(locked);
//!
//! let locked2 = widget2.try_lock().expect("widget1 lock is released");
//!
//! assert_eq!(locked2.name, "Spanner Bundle");
//! ```

#[cfg(test)]
extern crate core;

use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A light-weight lock guarded by an atomic boolean.
///
/// Most efficient when contention is low, acquiring the lock is a single
/// atomic swap, and releasing it just 1 more atomic swap.
///
/// It is only possible to try to acquire the lock, it is not possible to
/// wait for the lock to become ready, like with a `Mutex`.
#[derive(Default)]
pub struct TryLock<T> {
    is_locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> TryLock<T> {
    /// Create a `TryLock` around the value.
    #[inline]
    pub const fn new(val: T) -> TryLock<T> {
        TryLock {
            is_locked: AtomicBool::new(false),
            value: UnsafeCell::new(val),
        }
    }

    /// Try to acquire the lock of this value.
    ///
    /// If the lock is already acquired by someone else, this returns
    /// `None`. You can try to acquire again whenever you want, perhaps
    /// by spinning a few times, or by using some other means of
    /// notification.
    ///
    /// # Note
    ///
    /// The default memory ordering is to use `Acquire` to lock, and `Release`
    /// to unlock. If different ordering is required, use
    /// [`try_lock_explicit`](TryLock::try_lock_explicit) or
    /// [`try_lock_explicit_unchecked`](TryLock::try_lock_explicit_unchecked).
    #[inline]
    pub fn try_lock(&self) -> Option<Locked<'_, T>> {
        unsafe { self.try_lock_explicit_unchecked(Ordering::Acquire, Ordering::Release) }
    }

    /// Try to acquire the lock of this value using the specified lock and
    /// unlock orderings.
    ///
    /// If the lock is already acquired by someone else, this returns
    /// `None`. You can try to acquire again whenever you want, perhaps
    /// by spinning a few times, or by using some other means of
    /// notification.
    ///
    /// # Panic
    ///
    /// This method panics if `lock_order` is not any of `Acquire`, `AcqRel`,
    /// and `SeqCst`, or `unlock_order` is not any of `Release` and `SeqCst`.
    #[inline]
    pub fn try_lock_explicit(
        &self,
        lock_order: Ordering,
        unlock_order: Ordering,
    ) -> Option<Locked<'_, T>> {
        match lock_order {
            Ordering::Acquire | Ordering::AcqRel | Ordering::SeqCst => {}
            _ => panic!("lock ordering must be `Acquire`, `AcqRel`, or `SeqCst`"),
        }

        match unlock_order {
            Ordering::Release | Ordering::SeqCst => {}
            _ => panic!("unlock ordering must be `Release` or `SeqCst`"),
        }

        unsafe { self.try_lock_explicit_unchecked(lock_order, unlock_order) }
    }

    /// Try to acquire the lock of this value using the specified lock and
    /// unlock orderings without checking that the specified orderings are
    /// strong enough to be safe.
    ///
    /// If the lock is already acquired by someone else, this returns
    /// `None`. You can try to acquire again whenever you want, perhaps
    /// by spinning a few times, or by using some other means of
    /// notification.
    ///
    /// # Safety
    ///
    /// Unlike [`try_lock_explicit`], this method is unsafe because it does not
    /// check that the given memory orderings are strong enough to prevent data
    /// race.
    ///
    /// [`try_lock_explicit`]: Self::try_lock_explicit
    #[inline]
    pub unsafe fn try_lock_explicit_unchecked(
        &self,
        lock_order: Ordering,
        unlock_order: Ordering,
    ) -> Option<Locked<'_, T>> {
        if !self.is_locked.swap(true, lock_order) {
            Some(Locked {
                lock: self,
                order: unlock_order,
                _p: PhantomData,
            })
        } else {
            None
        }
    }

    /// Take the value back out of the lock when this is the sole owner.
    #[inline]
    pub fn into_inner(self) -> T {
        debug_assert!(
            !self.is_locked.load(Ordering::Relaxed),
            "TryLock was mem::forgotten"
        );
        self.value.into_inner()
    }
}

unsafe impl<T: Send> Send for TryLock<T> {}
unsafe impl<T: Send> Sync for TryLock<T> {}

impl<T: fmt::Debug> fmt::Debug for TryLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Used if the TryLock cannot acquire the lock.
        struct LockedPlaceholder;

        impl fmt::Debug for LockedPlaceholder {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("<locked>")
            }
        }

        let mut builder = f.debug_struct("TryLock");
        if let Some(locked) = self.try_lock() {
            builder.field("value", &*locked);
        } else {
            builder.field("value", &LockedPlaceholder);
        }
        builder.finish()
    }
}

/// A locked value acquired from a `TryLock`.
///
/// The type represents an exclusive view at the underlying value. The lock is
/// released when this type is dropped.
///
/// This type derefs to the underlying value.
#[must_use = "TryLock will immediately unlock if not used"]
pub struct Locked<'a, T: 'a> {
    lock: &'a TryLock<T>,
    order: Ordering,
    /// Suppresses Send and Sync autotraits for `struct Locked`.
    _p: PhantomData<*mut T>,
}

impl<'a, T> Deref for Locked<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for Locked<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for Locked<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.is_locked.store(false, self.order);
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Locked<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::TryLock;
    use core::sync::atomic::Ordering;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Arc;

    #[test]
    fn explicit_orderings_reject_every_weak_or_invalid_combination() {
        let orders = [
            Ordering::Relaxed,
            Ordering::Release,
            Ordering::Acquire,
            Ordering::AcqRel,
            Ordering::SeqCst,
        ];
        for lock_order in orders {
            for unlock_order in orders {
                let lock = TryLock::new(0);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let mut guard = lock.try_lock_explicit(lock_order, unlock_order).unwrap();
                    *guard += 1;
                }));
                let valid = matches!(
                    lock_order,
                    Ordering::Acquire | Ordering::AcqRel | Ordering::SeqCst
                ) && matches!(unlock_order, Ordering::Release | Ordering::SeqCst);
                assert_eq!(result.is_ok(), valid, "{lock_order:?}/{unlock_order:?}");
                // Validation must happen before touching the lock or its value.
                assert_eq!(*lock.try_lock().unwrap(), i32::from(valid));
            }
        }
    }

    #[test]
    fn default_lock_serializes_writes_across_threads() {
        let lock = Arc::new(TryLock::new(0));
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let lock = Arc::clone(&lock);
                std::thread::spawn(move || {
                    for _ in 0..2000 {
                        loop {
                            if let Some(mut guard) = lock.try_lock() {
                                *guard += 1;
                                break;
                            }
                            std::thread::yield_now();
                        }
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(*lock.try_lock().unwrap(), 8000);
    }

    #[test]
    fn fmt_debug() {
        let lock = TryLock::new(5);
        assert_eq!(format!("{:?}", lock), "TryLock { value: 5 }");

        let locked = lock.try_lock().unwrap();
        assert_eq!(format!("{:?}", locked), "5");

        assert_eq!(format!("{:?}", lock), "TryLock { value: <locked> }");
    }
}
