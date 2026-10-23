//! Exercise maintained SQLite binding and callback ownership through the product dependency.
use sqlx::{ConnectOptions, Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::sync::Arc;

// This fails to compile if SqliteRow ever regains an unsafe Sync promise. SQLite
// values and their Rust OnceCell borrow cache are mutable during decoding.
const _: fn() = || {
    trait AmbiguousIfSync<A> {
        fn check() {}
    }
    impl<T: ?Sized> AmbiguousIfSync<()> for T {}
    impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}
    let _ = <sqlx::sqlite::SqliteRow as AmbiguousIfSync<_>>::check;
};

#[tokio::test]
async fn query_mappers_preserve_statement_order_and_propagate_decode_errors() -> sqlx::Result<()> {
    use sqlx::Row;

    let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
    let values = sqlx::query("SELECT ?; SELECT ?")
        .bind(7_i64)
        .bind(11_i64)
        .try_map(|row: sqlx::sqlite::SqliteRow| row.try_get::<i64, _>(0).map(|value| value * 2))
        .fetch_all(&mut connection)
        .await?;
    assert_eq!(values, [14, 22]);

    let error = sqlx::query("SELECT 1 AS present; SELECT 2 AS missing")
        .try_map(|row: sqlx::sqlite::SqliteRow| row.try_get::<i64, _>("present"))
        .fetch_all(&mut connection)
        .await
        .unwrap_err();
    assert!(matches!(error, sqlx::Error::ColumnNotFound(name) if name == "present"));
    connection.close().await
}

#[tokio::test]
async fn cancelled_handle_waiter_does_not_block_the_worker_or_next_waiter() -> sqlx::Result<()> {
    use std::{
        future::Future,
        sync::atomic::{AtomicBool, Ordering},
        task::{Context, Wake, Waker},
        time::Duration,
    };

    struct WakeFlag(AtomicBool);
    impl Wake for WakeFlag {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    // A spawned task has a finite Tokio cooperative scheduling budget.
    tokio::spawn(async {
        let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let mut entered_tx = Some(entered_tx);
        connection
            .lock_handle()
            .await?
            .set_progress_handler(1, move || {
                if let Some(entered_tx) = entered_tx.take() {
                    let _ = entered_tx.send(());
                    // Bound a failed test's native worker lifetime as well.
                    return release_rx.recv_timeout(Duration::from_secs(10)).is_ok();
                }
                true
            });

        let mut query = Box::pin(sqlx::query("SELECT 1").execute(&mut connection));
        tokio::time::timeout(Duration::from_secs(5), async {
        tokio::select! {
            result = &mut query => panic!("query finished before releasing its worker: {result:?}"),
            entered = entered_rx => entered.unwrap(),
        }
    })
    .await
    .expect("query never entered its SQLite progress callback");
        // The worker still holds the mutex inside the callback, so the following
        // request deterministically queues a waiter and sends UnlockDb, then cancels.
        drop(query);
        let mut cancelled = Box::pin(connection.lock_handle());
        while tokio::task::coop::has_budget_remaining() {
            tokio::task::coop::consume_budget().await;
        }
        let wake = Arc::new(WakeFlag(AtomicBool::new(false)));
        let waker = Waker::from(Arc::clone(&wake));
        assert!(
            cancelled
                .as_mut()
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        tokio::task::yield_now().await;
        // The SQLite worker is still blocked, so a correctly queued mutex waiter
        // cannot wake yet. A cooperative-budget yield would instead wake before
        // entering the mutex queue, breaking the driver's unlock/relock protocol.
        assert!(
            !wake.0.load(Ordering::SeqCst),
            "lock yielded before joining the mutex queue"
        );
        drop(cancelled);
        release_tx.send(()).unwrap();

        tokio::time::timeout(Duration::from_secs(5), async {
            connection.lock_handle().await?.remove_progress_handler();
            // Repeated handoffs exercise the FIFO ordering between the caller and
            // the worker's immediate attempt to acquire its mutex again.
            for _ in 0..8 {
                let value: i64 = sqlx::query_scalar("SELECT 42")
                    .fetch_one(&mut connection)
                    .await?;
                assert_eq!(value, 42);
                drop(connection.lock_handle().await?);
            }
            connection.close().await
        })
        .await
        .expect("cancelled handle waiter prevented worker progress")
    })
    .await
    .expect("SQLite worker handoff task panicked")
}

#[tokio::test]
async fn sqlite_rows_move_between_threads_and_debug_preserves_existing_borrows() -> sqlx::Result<()>
{
    use sqlx::{Row, Value, ValueRef};
    let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
    let row =
        sqlx::query("SELECT 'shared text' AS text,x'0102ff' AS bytes,42 AS number,NULL AS empty")
            .fetch_one(&mut connection)
            .await?;
    std::thread::spawn(move || {
        let text: &str = row.try_get("text").unwrap();
        let bytes: &[u8] = row.try_get("bytes").unwrap();
        let debug = format!("{row:?}");
        assert!(debug.contains("shared text"));
        assert!(debug.contains("42"));
        assert!(debug.contains("NULL"));
        assert_eq!(text, "shared text");
        assert_eq!(bytes, [1, 2, 255]);
        assert_eq!(row.try_get::<i64, _>("number").unwrap(), 42);
        let owned = row.try_get_raw("text").unwrap().to_owned();
        drop(row);
        assert_eq!(owned.try_decode::<&str>().unwrap(), "shared text");
    })
    .join()
    .unwrap();
    Ok(())
}

#[tokio::test]
async fn invalid_numbered_parameters_return_errors_without_losing_the_connection()
-> sqlx::Result<()> {
    let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
    for query in [
        "SELECT $0",
        "SELECT $00",
        "SELECT $1suffix",
        "SELECT $1::suffix",
        "SELECT $1(suffix)",
        "SELECT $999999999999999999999999999999999999999999",
    ] {
        let error = sqlx::query(query)
            .bind(7_i64)
            .fetch_one(&mut connection)
            .await
            .unwrap_err();
        assert!(
            matches!(error, sqlx::Error::Protocol(_)),
            "{query}: {error}"
        );
        let value: i64 = sqlx::query_scalar("SELECT ?")
            .bind(11_i64)
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(value, 11);
    }
    Ok(())
}

#[tokio::test]
async fn missing_arguments_do_not_skip_later_numbered_bindings_or_reuse_cached_values()
-> sqlx::Result<()> {
    let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
    let query = "SELECT $2, $1, $2";
    let row: (Option<i64>, i64, Option<i64>) = sqlx::query_as(query)
        .bind(10_i64)
        .bind(20_i64)
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(row, (Some(20), 10, Some(20)));
    let row: (Option<i64>, i64, Option<i64>) = sqlx::query_as(query)
        .bind(30_i64)
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(row, (None, 30, None));

    let row: (Option<i64>, Option<i64>) = sqlx::query_as("SELECT ?, ?")
        .bind(40_i64)
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(row, (Some(40), None));
    Ok(())
}

#[tokio::test]
async fn anonymous_and_explicit_parameters_preserve_multistatement_binding_contracts()
-> sqlx::Result<()> {
    let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
    let rows: Vec<i64> = sqlx::query_scalar("SELECT ?; SELECT ?")
        .bind(10_i64)
        .bind(20_i64)
        .fetch_all(&mut connection)
        .await?;
    assert_eq!(rows, [10, 20]);

    let rows: Vec<i64> = sqlx::query_scalar("SELECT ?1; SELECT ?1; SELECT $1")
        .bind(30_i64)
        .fetch_all(&mut connection)
        .await?;
    assert_eq!(rows, [30, 30, 30]);

    // Explicit indices address the shared argument list; anonymous parameters
    // consume that list independently, including across statement boundaries.
    let rows: Vec<(i64, i64)> = sqlx::query_as("SELECT ?1, ?; SELECT $1, ?")
        .bind(40_i64)
        .bind(50_i64)
        .fetch_all(&mut connection)
        .await?;
    assert_eq!(rows, [(40, 40), (40, 50)]);

    let row: (i64, i64, i64) = sqlx::query_as("SELECT ?2, ?1, $01")
        .bind(60_i64)
        .bind(70_i64)
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(row, (70, 60, 60));
    Ok(())
}

#[tokio::test]
async fn invalid_collation_names_release_both_kinds_of_callback_owner() -> sqlx::Result<()> {
    let marker = Arc::new(String::from("callback"));
    let captured = Arc::clone(&marker);
    let options = SqliteConnectOptions::new().in_memory(true).collation(
        "invalid\0name",
        move |left, right| {
            assert!(!captured.is_empty());
            left.cmp(right)
        },
    );
    assert!(matches!(
        options.connect().await,
        Err(sqlx::Error::Protocol(_))
    ));
    drop(options);
    assert_eq!(Arc::strong_count(&marker), 1);

    let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
    let captured = Arc::clone(&marker);
    let error =
        connection
            .lock_handle()
            .await?
            .create_collation("invalid\0name", move |left, right| {
                assert!(!captured.is_empty());
                left.cmp(right)
            });
    assert!(matches!(error, Err(sqlx::Error::Protocol(_))));
    assert_eq!(Arc::strong_count(&marker), 1);
    connection.close().await
}

#[tokio::test]
async fn registered_collations_keep_callbacks_until_replacement_or_connection_close()
-> sqlx::Result<()> {
    let marker = Arc::new(String::from("callback"));
    let captured = Arc::clone(&marker);
    let options = SqliteConnectOptions::new().in_memory(true).collation(
        "custom_order",
        move |left, right| {
            assert!(!captured.is_empty());
            right.cmp(left)
        },
    );
    let mut connection = options.connect().await?;
    drop(options);
    assert_eq!(Arc::strong_count(&marker), 2);
    let reverse: bool = sqlx::query_scalar("SELECT 'a' < 'b' COLLATE custom_order")
        .fetch_one(&mut connection)
        .await?;
    assert!(!reverse);

    let next = Arc::new(String::from("replacement"));
    let captured = Arc::clone(&next);
    connection
        .lock_handle()
        .await?
        .create_collation("custom_order", move |left, right| {
            assert!(!captured.is_empty());
            left.cmp(right)
        })?;
    assert_eq!(Arc::strong_count(&marker), 1);
    assert_eq!(Arc::strong_count(&next), 2);
    let forward: bool = sqlx::query_scalar("SELECT 'a' < 'b' COLLATE custom_order")
        .fetch_one(&mut connection)
        .await?;
    assert!(forward);
    connection.close().await?;
    assert_eq!(Arc::strong_count(&next), 1);
    Ok(())
}
