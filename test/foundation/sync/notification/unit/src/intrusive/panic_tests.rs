
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
