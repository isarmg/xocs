
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
