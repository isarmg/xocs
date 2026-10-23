//! Intrusive linked list-based implementation of `event-listener`.
//!
//! This implementation crates an intrusive linked list of listeners. This list
//! is secured using a libstd mutex, or a spinlock without `std`.

use crate::notify::{GenericNotify, Internal, Notification};
use crate::sync::atomic::Ordering;
use crate::sync::cell::{Cell, UnsafeCell};
use crate::{RegisterResult, State, TaskRef};

#[cfg(feature = "std")]
use core::ops::{Deref, DerefMut};

use core::marker::PhantomPinned;
use core::mem;
use core::pin::Pin;
use core::ptr::NonNull;

pub(super) struct List<T>(
    /// libstd-based implementation uses a normal Mutex to secure the data.
    #[cfg(feature = "std")]
    crate::sync::Mutex<Inner<T>>,
    /// Spinlock implementation uses a spinlock.
    #[cfg(not(feature = "std"))]
    spinlock::Spinlock<Inner<T>>,
);

struct Inner<T> {
    /// The head of the linked list.
    head: Option<NonNull<Link<T>>>,

    /// The tail of the linked list.
    tail: Option<NonNull<Link<T>>>,

    /// The first unnotified listener.
    next: Option<NonNull<Link<T>>>,

    /// Total number of listeners.
    len: usize,

    /// The number of notified listeners.
    notified: usize,
}

impl<T> List<T> {
    /// Create a new, empty event listener list.
    pub(super) fn new() -> Self {
        let inner = Inner {
            head: None,
            tail: None,
            next: None,
            len: 0,
            notified: 0,
        };

        #[cfg(feature = "std")]
        {
            Self(crate::sync::Mutex::new(inner))
        }

        #[cfg(not(feature = "std"))]
        {
            Self(spinlock::Spinlock::new(inner))
        }
    }

    /// Get the total number of listeners without blocking.
    #[cfg(feature = "std")]
    pub(crate) fn try_total_listeners(&self) -> Option<usize> {
        match self.0.try_lock() {
            Ok(list) => Some(list.len),
            Err(std::sync::TryLockError::Poisoned(error)) => Some(error.into_inner().len),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }

    /// Get the total number of listeners without blocking.
    #[cfg(not(feature = "std"))]
    pub(crate) fn try_total_listeners(&self) -> Option<usize> {
        self.0.try_with(|list| list.len)
    }

    /// Get the total number of listeners with blocking.
    #[cfg(feature = "std")]
    pub(crate) fn total_listeners(&self) -> usize {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).len
    }
}

impl<T> crate::Inner<T> {
    #[cfg(feature = "std")]
    fn with_inner<R>(&self, f: impl FnOnce(&mut Inner<T>) -> R) -> R {
        struct ListLock<'a, 'b, T> {
            lock: crate::sync::MutexGuard<'a, Inner<T>>,
            inner: &'b crate::Inner<T>,
        }

        impl<T> Deref for ListLock<'_, '_, T> {
            type Target = Inner<T>;

            fn deref(&self) -> &Self::Target {
                &self.lock
            }
        }

        impl<T> DerefMut for ListLock<'_, '_, T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.lock
            }
        }

        impl<T> Drop for ListLock<'_, '_, T> {
            fn drop(&mut self) {
                update_notified(&self.inner.notified, &self.lock);
            }
        }

        let mut list = ListLock {
            inner: self,
            lock: self.list.0.lock().unwrap_or_else(|e| e.into_inner()),
        };
        f(&mut list)
    }

    #[cfg(not(feature = "std"))]
    fn with_inner<R>(&self, f: impl FnOnce(&mut Inner<T>) -> R) -> R {
        struct ListWrapper<'a, T> {
            inner: &'a crate::Inner<T>,
            list: &'a mut Inner<T>,
        }

        impl<T> Drop for ListWrapper<'_, T> {
            fn drop(&mut self) {
                update_notified(&self.inner.notified, self.list);
            }
        }

        self.list.0.with(move |list| {
            let wrapper = ListWrapper {
                inner: self,
                list: &mut *list,
            };

            f(wrapper.list)
        })
    }

    /// Add a new listener to the list.
    pub(crate) fn insert(&self, mut listener: Pin<&mut Option<Listener<T>>>) {
        self.with_inner(|inner| {
            listener.as_mut().set(Some(Listener {
                link: UnsafeCell::new(Link {
                    state: Cell::new(State::Created),
                    prev: Cell::new(inner.tail),
                    next: Cell::new(None),
                }),
                _pin: PhantomPinned,
            }));
            let listener = listener.as_pin_mut().unwrap();

            {
                let entry_guard = listener.link.get();
                // SAFETY: We are locked, so we can access the inner `link`.
                let entry = unsafe { &*entry_guard };

                // Replace the tail with the new entry.
                match inner.tail.replace(entry.into()) {
                    None => inner.head = Some(entry.into()),
                    Some(t) => unsafe { t.as_ref().next.set(Some(entry.into())) },
                };
            }

            // If there are no unnotified entries, this is the first one.
            if inner.next.is_none() {
                inner.next = inner.tail;
            }

            // Bump the entry count.
            inner.len += 1;
        });
    }

    /// Remove a listener from the list.
    pub(crate) fn remove(
        &self,
        listener: Pin<&mut Option<Listener<T>>>,
        propagate: bool,
    ) -> Option<State<T>> {
        self.with_inner(|inner| inner.remove(listener, propagate))
    }

    /// Notifies a number of entries.
    #[cold]
    pub(crate) fn notify(&self, notify: impl Notification<Tag = T>) -> usize {
        self.with_inner(|inner| inner.notify(notify))
    }

    /// Register a task to be notified when the event is triggered.
    ///
    /// Returns `true` if the listener was already notified, and `false` otherwise. If the listener
    /// isn't inserted, returns `None`.
    pub(crate) fn register(
        &self,
        mut listener: Pin<&mut Option<Listener<T>>>,
        task: TaskRef<'_>,
    ) -> RegisterResult<T> {
        self.with_inner(|inner| {
            let entry_guard = match listener.as_mut().as_pin_mut() {
                Some(listener) => listener.link.get(),
                None => return RegisterResult::NeverInserted,
            };
            // SAFETY: We are locked, so we can access the inner `link`.
            let entry = unsafe { &*entry_guard };

            // SAFETY: The list lock exclusively protects this state. Inspect it before
            // cloning a user-provided waker, which can panic; no placeholder state may
            // escape that callback. The borrow ends before any state replacement.
            match unsafe { &*entry.state.as_ptr() } {
                State::Notified { .. } => {
                    let State::Notified { tag, .. } = entry.state.replace(State::NotifiedTaken)
                    else {
                        unreachable!()
                    };
                    inner.remove(listener, false);
                    RegisterResult::Notified(tag)
                }
                State::Task(other_task) if task.will_wake(other_task.as_task_ref()) => {
                    RegisterResult::Registered
                }
                State::Created | State::Task(_) => {
                    let new_task = task.into_task();
                    // Commit before dropping the old task: its waker can also panic.
                    drop(entry.state.replace(State::Task(new_task)));
                    RegisterResult::Registered
                }
                State::NotifiedTaken => unreachable!("notified listener was already removed"),
            }
        })
    }
}

impl<T> Inner<T> {
    fn remove(
        &mut self,
        mut listener: Pin<&mut Option<Listener<T>>>,
        propagate: bool,
    ) -> Option<State<T>> {
        let entry_guard = listener.as_mut().as_pin_mut()?.link.get();
        let entry = unsafe { &*entry_guard };

        let prev = entry.prev.get();
        let next = entry.next.get();

        // Unlink from the previous entry.
        match prev {
            None => self.head = next,
            Some(p) => unsafe {
                p.as_ref().next.set(next);
            },
        }

        // Unlink from the next entry.
        match next {
            None => self.tail = prev,
            Some(n) => unsafe {
                n.as_ref().prev.set(prev);
            },
        }

        // If this was the first unnotified entry, update the next pointer.
        if self.next == Some(entry.into()) {
            self.next = next;
        }

        // The entry is now fully unlinked, so we can now take it out safely.
        let entry = unsafe {
            listener
                .get_unchecked_mut()
                .take()
                .unwrap()
                .link
                .into_inner()
        };

        let mut state = entry.state.into_inner();

        // Commit removal before forwarding a notification, which may invoke a
        // user-provided waker and unwind.
        self.len -= 1;
        if state.is_notified() {
            self.notified -= 1;

            if propagate {
                let state = mem::replace(&mut state, State::NotifiedTaken);
                if let State::Notified { additional, tag } = state {
                    let tags = {
                        let mut tag = Some(tag);
                        move || tag.take().expect("tag already taken")
                    };
                    self.notify(GenericNotify::new(1, additional, tags));
                }
            }
        }

        Some(state)
    }

    #[cold]
    fn notify(&mut self, mut notify: impl Notification<Tag = T>) -> usize {
        let mut n = notify.count(Internal::new());
        let is_additional = notify.is_additional(Internal::new());

        if !is_additional {
            if n < self.notified {
                return 0;
            }
            n -= self.notified;
        }

        let original_count = n;
        while n > 0 {
            n -= 1;

            // Notify the next entry.
            match self.next {
                None => return original_count - n - 1,

                Some(e) => {
                    // Generate the tag before changing the list: user code can panic.
                    let tag = notify.next_tag(Internal::new());
                    let entry = unsafe { e.as_ref() };
                    self.next = entry.next.get();
                    let previous = entry.state.replace(State::Notified {
                        additional: is_additional,
                        tag,
                    });
                    // Commit every invariant before invoking a user-provided waker.
                    self.notified += 1;
                    if let State::Task(task) = previous {
                        task.wake();
                    }
                }
            }
        }

        original_count - n
    }
}

fn update_notified<T>(slot: &crate::sync::atomic::AtomicUsize, list: &Inner<T>) {
    // Update the notified count.
    let notified = if list.notified < list.len {
        list.notified
    } else {
        usize::MAX
    };

    slot.store(notified, Ordering::Release);
}

pub(crate) struct Listener<T> {
    /// The inner link in the linked list.
    ///
    /// # Safety
    ///
    /// This can only be accessed while the central mutex is locked.
    link: UnsafeCell<Link<T>>,

    /// This listener cannot be moved after being pinned.
    _pin: PhantomPinned,
}

struct Link<T> {
    /// The current state of the listener.
    state: Cell<State<T>>,

    /// The previous link in the linked list.
    prev: Cell<Option<NonNull<Link<T>>>>,

    /// The next link in the linked list.
    next: Cell<Option<NonNull<Link<T>>>>,
}

/// Spinlock implementation for builds without `std`.
#[cfg(not(feature = "std"))]
mod spinlock {
    use crate::sync::atomic::{AtomicBool, Ordering};
    use crate::sync::cell::UnsafeCell;

    /// A spinlock.
    pub(super) struct Spinlock<T> {
        /// Is this spinlock locked?
        locked: AtomicBool,

        /// The data contained within the lock.
        data: UnsafeCell<T>,
    }

    impl<T> Spinlock<T> {
        /// Create a new [`Spinlock`].
        #[inline]
        pub(super) fn new(data: T) -> Self {
            Self {
                locked: AtomicBool::new(false),
                data: UnsafeCell::new(data),
            }
        }

        /// Try to acquire a spinlock, run a closure, then release.
        #[inline]
        pub(super) fn try_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
            if self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                let _guard = CallOnDrop(|| self.locked.store(false, Ordering::Release));
                // SAFETY: We have the lock held.
                Some(f(unsafe { &mut *self.data.get() }))
            } else {
                None
            }
        }

        /// Run an operation with the spinlock held.
        #[inline]
        pub(super) fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            loop {
                if self
                    .locked
                    .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    let _guard = CallOnDrop(|| self.locked.store(false, Ordering::Release));
                    // SAFETY: We have the lock held.
                    return f(unsafe { &mut *self.data.get() });
                }

                while self.locked.load(Ordering::Relaxed) {
                    core::hint::spin_loop();
                }
            }
        }
    }

    struct CallOnDrop<F: FnMut()>(F);
    impl<F: FnMut()> Drop for CallOnDrop<F> {
        #[inline]
        fn drop(&mut self) {
            (self.0)();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! make_listeners {
        ($($id:ident),*) => {
            $(
                let mut $id = core::pin::pin!(Option::<Listener<()>>::None);
            )*
        };
    }

    #[test]
    fn insert() {
        let inner = crate::Inner::new();
        make_listeners!(listen1, listen2, listen3);

        // Register the listeners.
        inner.insert(listen1.as_mut());
        inner.insert(listen2.as_mut());
        inner.insert(listen3.as_mut());

        assert_eq!(inner.list.try_total_listeners(), Some(3));

        // Remove one.
        assert_eq!(inner.remove(listen2, false), Some(State::Created));
        assert_eq!(inner.list.try_total_listeners(), Some(2));

        // Remove another.
        assert_eq!(inner.remove(listen1, false), Some(State::Created));
        assert_eq!(inner.list.try_total_listeners(), Some(1));
    }

    #[test]
    fn drop_non_notified() {
        let inner = crate::Inner::new();
        make_listeners!(listen1, listen2, listen3);

        // Register the listeners.
        inner.insert(listen1.as_mut());
        inner.insert(listen2.as_mut());
        inner.insert(listen3.as_mut());

        // Notify one.
        inner.notify(GenericNotify::new(1, false, || ()));

        // Remove one.
        inner.remove(listen3, true);

        // Remove the rest.
        inner.remove(listen1, true);
        inner.remove(listen2, true);
    }
}

#[cfg(all(test, feature = "std"))]
mod panic_tests {
    use crate::{Event, EventListener, IntoNotification};
    use std::future::Future;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Wake, Waker};

    fn poll<T>(listener: &mut EventListener<T>, waker: &Waker) -> Poll<T> {
        Pin::new(listener).poll(&mut Context::from_waker(waker))
    }

    struct PanicWake;
    impl Wake for PanicWake {
        fn wake(self: Arc<Self>) {
            panic!("waker failed");
        }
    }

    #[test]
    fn panicking_tag_does_not_skip_the_oldest_listener() {
        let event = Event::<u32>::with_tag();
        let mut first = event.listen();
        let mut second = event.listen();
        assert!(catch_unwind(|| event.notify(1.tag_with(|| panic!("tag failed")))).is_err());
        let mut tag = 20;
        assert_eq!(
            event.notify(2.tag_with(|| {
                tag += 1;
                tag
            })),
            2
        );
        assert_eq!(poll(&mut first, Waker::noop()), Poll::Ready(21));
        assert_eq!(poll(&mut second, Waker::noop()), Poll::Ready(22));
        assert_eq!(event.total_listeners(), 0);
    }

    #[test]
    fn panicking_waker_keeps_notification_accounting_consistent() {
        let event = Event::new();
        let mut first = event.listen();
        let mut second = event.listen();
        let waker = Waker::from(Arc::new(PanicWake));
        assert!(poll(&mut first, &waker).is_pending());
        assert!(catch_unwind(|| event.notify(1)).is_err());
        assert_eq!(event.notify(1), 0);
        assert_eq!(poll(&mut first, Waker::noop()), Poll::Ready(()));
        assert_eq!(event.notify(1), 1);
        assert_eq!(poll(&mut second, Waker::noop()), Poll::Ready(()));
        assert_eq!(event.total_listeners(), 0);
    }

    #[test]
    fn panicking_forwarded_waker_keeps_listener_count_consistent() {
        let event = Event::new();
        let first = event.listen();
        let mut second = event.listen();
        let waker = Waker::from(Arc::new(PanicWake));
        assert!(poll(&mut second, &waker).is_pending());
        assert_eq!(event.notify(1), 1);
        assert!(catch_unwind(|| drop(first)).is_err());
        assert_eq!(event.total_listeners(), 1);
        assert_eq!(poll(&mut second, Waker::noop()), Poll::Ready(()));
        assert_eq!(event.total_listeners(), 0);
    }

    #[test]
    fn panicking_waker_clone_leaves_listener_removable() {
        unsafe fn clone(_: *const ()) -> RawWaker {
            panic!("waker clone failed")
        }
        unsafe fn no_op(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
        // SAFETY: All callbacks ignore the data pointer; cloning unwinds without
        // creating or transferring resources, and the other callbacks are no-ops.
        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let event = Event::new();
        let mut listener = event.listen();
        assert!(catch_unwind(AssertUnwindSafe(|| poll(&mut listener, &waker))).is_err());
        assert!(catch_unwind(|| drop(listener)).is_ok());
        assert_eq!(event.total_listeners(), 0);
    }

    #[test]
    fn panicking_replaced_waker_drop_keeps_new_registration() {
        struct PanicDrop;
        impl Wake for PanicDrop {
            fn wake(self: Arc<Self>) {}
        }
        impl Drop for PanicDrop {
            fn drop(&mut self) {
                panic!("old waker drop failed");
            }
        }

        let event = Event::new();
        let mut listener = event.listen();
        let old_waker = Waker::from(Arc::new(PanicDrop));
        assert!(poll(&mut listener, &old_waker).is_pending());
        // Only the registered waker owns the old task from this point onward.
        drop(old_waker);

        assert!(catch_unwind(AssertUnwindSafe(|| poll(&mut listener, Waker::noop()))).is_err());
        assert_eq!(event.total_listeners(), 1);
        // Poison records the callback panic, not a lock that is still held.
        assert!(format!("{event:?}").contains("listeners_total: 1"));
        assert_eq!(event.notify(1), 1);
        assert_eq!(poll(&mut listener, Waker::noop()), Poll::Ready(()));
        assert_eq!(event.total_listeners(), 0);
    }
}
