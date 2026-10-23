
    use super::*;

    // Does a `LocalSet` running on a current-thread runtime...basically work?
    //
    // This duplicates a test in `tests/task_local_set.rs`, but because this is
    // a lib test, it will run under Miri, so this is necessary to catch stacked
    // borrows violations in the `LocalSet` implementation.
    #[test]
    fn local_current_thread_scheduler() {
        let f = async {
            LocalSet::new()
                .run_until(async {
                    spawn_local(async {}).await.unwrap();
                })
                .await;
        };
        crate::runtime::Builder::new_current_thread()
            .build()
            .expect("rt")
            .block_on(f)
    }

    // Tests that when a task on a `LocalSet` is woken by an io driver on the
    // same thread, the task is woken to the localset's local queue rather than
    // its remote queue.
    //
    // This test has to be defined in the `local.rs` file as a lib test, rather
    // than in `tests/`, because it makes assertions about the local set's
    // internal state.
    #[test]
    fn wakes_to_local_queue() {
        use super::*;
        use crate::sync::Notify;
        let rt = crate::runtime::Builder::new_current_thread()
            .build()
            .expect("rt");
        rt.block_on(async {
            let local = LocalSet::new();
            let notify = Arc::new(Notify::new());
            let task = local.spawn_local({
                let notify = notify.clone();
                async move {
                    notify.notified().await;
                }
            });
            let mut run_until = Box::pin(local.run_until(async move {
                task.await.unwrap();
            }));

            // poll the run until future once
            std::future::poll_fn(|cx| {
                let _ = run_until.as_mut().poll(cx);
                Poll::Ready(())
            })
            .await;

            notify.notify_one();
            let task = unsafe { local.context.shared.local_state.task_pop_front() };
            // TODO(eliza): it would be nice to be able to assert that this is
            // the local task.
            assert!(
                task.is_some(),
                "task should have been notified to the LocalSet's local queue"
            );
        })
    }
