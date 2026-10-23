
    use super::*;
    use crate::sqlite::SqliteConnectOptions;
    use std::task::{Context, Poll, Waker};

    #[test]
    fn unlock_db_never_holds_the_guard_while_waiting_for_a_command_queue_slot() {
        for capacity in [0, 1] {
            for initially_locked in [false, true] {
                let params =
                    EstablishParams::from_options(&SqliteConnectOptions::new().in_memory(true))
                        .unwrap();
                let shared = Arc::new(WorkerSharedState {
                    transaction_depth: AtomicUsize::new(0),
                    cached_statements_size: AtomicUsize::new(0),
                    conn: Mutex::new(params.establish().unwrap()),
                });
                let (command_tx, command_rx) = flume::bounded(capacity);
                if capacity == 1 {
                    assert!(command_tx
                        .try_send((Command::UnlockDb, Span::none()))
                        .is_ok());
                }
                let mut worker = ConnectionWorker {
                    command_tx,
                    shared: Arc::clone(&shared),
                };
                let held = initially_locked.then(|| shared.conn.try_lock().unwrap());
                let mut unlock = std::pin::pin!(worker.unlock_db());
                let mut context = Context::from_waker(Waker::noop());
                if initially_locked {
                    assert!(unlock.as_mut().poll(&mut context).is_pending());
                }
                // Model the worker yielding its database after an earlier
                // UnlockDb. No receiver can free a command queue slot until
                // the worker gets the database guard back.
                drop(held);
                assert!(matches!(
                    unlock.as_mut().poll(&mut context),
                    Poll::Ready(Ok(_))
                ));
                assert_eq!(command_rx.len(), capacity);
            }
        }
    }
