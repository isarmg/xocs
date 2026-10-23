use super::*;
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions, sqlite::SqliteJournalMode};
use std::{fs, path::Path, time::Duration};

fn exercise_direct_connection(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    block_on_sqlite_connection(async {
        let mut connection = SqliteConnection::connect_with(
            &SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal),
        )
        .await?;
        // The worker must yield its database mutex to a caller that is
        // polled by futures::executor, without depending on a Tokio task.
        connection
            .lock_handle()
            .await?
            .create_collation("direct_order", str::cmp)?;
        let ordered: bool = sqlx::query_scalar("SELECT 'a' < 'b' COLLATE direct_order")
            .fetch_one(&mut connection)
            .await?;
        assert!(ordered);
        sqlx::raw_sql("CREATE TABLE items(value INTEGER NOT NULL);INSERT INTO items VALUES(7)")
            .execute(&mut connection)
            .await?;
        let value: i64 = sqlx::query_scalar("SELECT value FROM items")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(value, 7);
        // Closing the native worker, rather than dropping a pool, is the
        // required boundary before byte snapshots and permission changes.
        connection.close().await?;
        Ok::<_, sqlx::Error>(())
    })?;
    let bytes = fs::read(path)?;
    let wal = path.with_extension("sqlite3-wal");
    let shm = path.with_extension("sqlite3-shm");
    assert!(!wal.exists());
    assert!(!shm.exists());
    std::thread::sleep(Duration::from_millis(25));
    assert_eq!(fs::read(path)?, bytes);
    assert!(!wal.exists());
    assert!(!shm.exists());
    Ok(())
}

#[test]
fn direct_connection_without_an_async_runtime_closes_before_returning()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    exercise_direct_connection(&directory.path().join("outside.sqlite3"))
}

#[test]
fn direct_connection_inside_current_thread_runtime_closes_before_returning()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    tokio::runtime::Builder::new_current_thread()
        .build()?
        .block_on(async { exercise_direct_connection(&directory.path().join("current.sqlite3")) })
}

#[test]
fn direct_connection_inside_multi_thread_runtime_closes_before_returning()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()?
        .block_on(async { exercise_direct_connection(&directory.path().join("multi.sqlite3")) })
}
