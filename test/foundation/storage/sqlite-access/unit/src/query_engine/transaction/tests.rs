
    use crate::query_engine::sql_str::SqlSafeStr;
    use crate::{Connection, Error, SqliteConnection};
    use std::future::Future;
    use std::task::{Context, Waker};
    use std::time::{Duration, Instant};

    #[test]
    fn rejected_nested_begin_keeps_parent_transaction_open() {
        futures::executor::block_on(async {
            let mut connection = SqliteConnection::connect("sqlite::memory:").await.unwrap();
            crate::query("CREATE TABLE entries (id INTEGER)")
                .execute(&mut connection)
                .await
                .unwrap();
            let mut outer = connection.begin().await.unwrap();
            crate::query("INSERT INTO entries VALUES (1)")
                .execute(&mut *outer)
                .await
                .unwrap();

            assert!(matches!(
                outer.begin_with("BEGIN IMMEDIATE").await,
                Err(Error::InvalidSavePointStatement)
            ));

            let count: i64 = crate::query_scalar("SELECT COUNT(*) FROM entries")
                .fetch_one(&mut *outer)
                .await
                .unwrap();
            assert_eq!(
                count, 1,
                "a rejected child transaction must not roll back its parent"
            );
            assert!(outer.is_in_transaction());
            outer.commit().await.unwrap();
            connection.close().await.unwrap();
        });
    }

    #[test]
    fn cancelled_nested_begin_rolls_back_only_its_own_savepoint() {
        for cancel_after_begin in [false, true] {
            cancelled_nested_begin_preserves_parent(cancel_after_begin, false);
        }
        cancelled_nested_begin_preserves_parent(false, true);
    }

    fn cancelled_nested_begin_preserves_parent(cancel_after_begin: bool, custom: bool) {
        futures::executor::block_on(async {
            let mut connection = SqliteConnection::connect("sqlite::memory:").await.unwrap();
            crate::query("CREATE TABLE entries (id INTEGER)")
                .execute(&mut connection)
                .await
                .unwrap();
            let mut outer = connection.begin().await.unwrap();
            crate::query("INSERT INTO entries VALUES (1)")
                .execute(&mut *outer)
                .await
                .unwrap();

            let (started_tx, started_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut first_call = true;
            outer
                .lock_handle()
                .await
                .unwrap()
                .set_progress_handler(1, move || {
                    if first_call {
                        first_call = false;
                        started_tx.send(()).unwrap();
                        release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                    }
                    true
                });

            // Hold the worker in SQLite while enqueuing and cancelling BEGIN,
            // so the child future cannot receive the worker's acknowledgement.
            let mut context = Context::from_waker(Waker::noop());
            {
                let mut query = std::pin::pin!(crate::query("SELECT 1").fetch_one(&mut *outer));
                assert!(query.as_mut().poll(&mut context).is_pending());
            }
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let shared = std::sync::Arc::clone(&outer.worker.shared);
            {
                let statement = custom.then(|| "BEGIN IMMEDIATE".into_sql_str());
                let mut begin = std::pin::pin!(super::Transaction::<crate::Sqlite>::begin(
                    &mut *outer,
                    statement
                ));
                assert!(begin.as_mut().poll(&mut context).is_pending());
                if cancel_after_begin {
                    release_tx.send(()).unwrap();
                    // With no receiver polls, the worker cannot acknowledge BEGIN.
                    // Wait for the savepoint to exist before cancelling its future.
                    let deadline = Instant::now() + Duration::from_secs(5);
                    while shared.get_transaction_depth() != 2 {
                        assert!(Instant::now() < deadline, "worker did not create savepoint");
                        std::thread::yield_now();
                    }
                }
            }
            if !cancel_after_begin {
                release_tx.send(()).unwrap();
            }

            let count: i64 = crate::query_scalar("SELECT COUNT(*) FROM entries")
                .fetch_one(&mut *outer)
                .await
                .unwrap();
            assert_eq!(count, 1, "cancelling a child must not roll back its parent");
            assert!(outer.is_in_transaction());
            outer.commit().await.unwrap();
            connection.close().await.unwrap();
        });
    }
