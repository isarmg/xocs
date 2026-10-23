//! Exercise maintained SQLite binding and callback ownership through the product dependency.
use sqlx::{ConnectOptions, Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::sync::Arc;

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
