use std::fmt::{self, Debug, Formatter};
use std::future::{self, Future};
use std::ops::{Deref, DerefMut};

use futures::future::BoxFuture;

use crate::query_engine::database::Database;
use crate::query_engine::error::Error;
use crate::query_engine::pool::MaybePoolConnection;
use crate::query_engine::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};

/// Generic management of database transactions.
///
/// This trait should not be used, except when implementing [`Connection`](crate::Connection).
pub trait TransactionManager {
    type Database: Database;

    /// Begin a new transaction or establish a savepoint within the active transaction.
    ///
    /// If this is a new transaction, `statement` may be used instead of the
    /// default "BEGIN" statement.
    ///
    /// If we are already inside a transaction and `statement.is_some()`, then
    /// `Error::InvalidSavePoint` is returned without running any statements.
    fn begin(
        conn: &mut <Self::Database as Database>::Connection,
        statement: Option<SqlStr>,
    ) -> impl Future<Output = Result<(), Error>> + Send + '_;

    /// Commit the active transaction or release the most recent savepoint.
    fn commit(
        conn: &mut <Self::Database as Database>::Connection,
    ) -> impl Future<Output = Result<(), Error>> + Send + '_;

    /// Abort the active transaction or restore from the most recent savepoint.
    fn rollback(
        conn: &mut <Self::Database as Database>::Connection,
    ) -> impl Future<Output = Result<(), Error>> + Send + '_;

    /// Starts to abort the active transaction or restore from the most recent snapshot.
    fn start_rollback(conn: &mut <Self::Database as Database>::Connection);

    /// Returns the current transaction depth.
    ///
    /// Transaction depth indicates the level of nested transactions:
    /// - Level 0: No active transaction.
    /// - Level 1: A transaction is active.
    /// - Level 2 or higher: A transaction is active and one or more SAVEPOINTs have been created within it.
    fn get_transaction_depth(conn: &<Self::Database as Database>::Connection) -> usize;
}

/// An in-progress SQLite transaction or savepoint.
///
/// Start a transaction with [`crate::Pool::begin`] or [`crate::Connection::begin`].
/// Commit it with [`Transaction::commit`] or roll it back with
/// [`Transaction::rollback`]. Dropping an open transaction queues a rollback on
/// the connection worker; use the explicit async operation when its completion
/// must be observed.
///
/// A nested transaction uses a savepoint, allowing changes after that savepoint
/// to be undone without rolling back its parent transaction. Pass `&mut *tx` to
/// query methods to access the transaction's connection.
///
/// ```
/// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) -> blog_storage_sqlite_access::Result<()> {
/// let mut tx = pool.begin().await?;
/// let value: i64 = blog_storage_sqlite_access::query_scalar("SELECT ?1")
///     .bind(1_i64)
///     .fetch_one(&mut *tx)
///     .await?;
/// assert_eq!(value, 1);
/// tx.commit().await?;
/// # Ok(())
/// # }
/// ```
pub struct Transaction<'c, DB>
where
    DB: Database,
{
    connection: MaybePoolConnection<'c, DB>,
    open: bool,
}

impl<'c, DB> Transaction<'c, DB>
where
    DB: Database,
{
    #[doc(hidden)]
    pub fn begin(
        conn: impl Into<MaybePoolConnection<'c, DB>>,
        statement: Option<SqlStr>,
    ) -> BoxFuture<'c, Result<Self, Error>> {
        let conn = conn.into();

        Box::pin(async move {
            let mut tx = Self {
                connection: conn,

                // The SQLite worker owns cancellation until BEGIN is acknowledged:
                // it rolls back any savepoint created for a cancelled request.
                // A failed or unacknowledged begin must not drop another rollback
                // onto the connection and accidentally abort the parent transaction.
                open: false,
            };

            DB::TransactionManager::begin(&mut tx.connection, statement).await?;
            tx.open = true;

            Ok(tx)
        })
    }

    /// Commits this transaction or savepoint.
    pub async fn commit(mut self) -> Result<(), Error> {
        DB::TransactionManager::commit(&mut self.connection).await?;
        self.open = false;

        Ok(())
    }

    /// Aborts this transaction or savepoint.
    pub async fn rollback(mut self) -> Result<(), Error> {
        DB::TransactionManager::rollback(&mut self.connection).await?;
        self.open = false;

        Ok(())
    }
}

impl<DB> Debug for Transaction<'_, DB>
where
    DB: Database,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Show the full type <..<..<..
        f.debug_struct("Transaction").finish()
    }
}

impl<DB> Deref for Transaction<'_, DB>
where
    DB: Database,
{
    type Target = DB::Connection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<DB> DerefMut for Transaction<'_, DB>
where
    DB: Database,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

// Implement `AsMut<DB::Connection>` so `Transaction` can be given to a
// `PgAdvisoryLockGuard`.
//
// See: https://github.com/launchbadge/sqlx/issues/2520
impl<DB: Database> AsMut<DB::Connection> for Transaction<'_, DB> {
    fn as_mut(&mut self) -> &mut DB::Connection {
        &mut self.connection
    }
}

impl<'t, DB: Database> crate::query_engine::acquire::Acquire<'t> for &'t mut Transaction<'_, DB> {
    type Database = DB;

    type Connection = &'t mut <DB as Database>::Connection;

    #[inline]
    fn acquire(self) -> BoxFuture<'t, Result<Self::Connection, Error>> {
        Box::pin(future::ready(Ok(&mut **self)))
    }

    #[inline]
    fn begin(self) -> BoxFuture<'t, Result<Transaction<'t, DB>, Error>> {
        Transaction::begin(&mut **self, None)
    }
}

impl<DB> Drop for Transaction<'_, DB>
where
    DB: Database,
{
    fn drop(&mut self) {
        if self.open {
            // starts a rollback operation

            // what this does depends on the database but generally this means we queue a rollback
            // operation that will happen on the next asynchronous invocation of the underlying
            // connection (including if the connection is returned to a pool)

            DB::TransactionManager::start_rollback(&mut self.connection);
        }
    }
}

pub fn begin_ansi_transaction_sql(depth: usize) -> SqlStr {
    if depth == 0 {
        "BEGIN".into_sql_str()
    } else {
        AssertSqlSafe(format!("SAVEPOINT _sqlx_savepoint_{depth}")).into_sql_str()
    }
}

pub fn commit_ansi_transaction_sql(depth: usize) -> SqlStr {
    if depth == 1 {
        "COMMIT".into_sql_str()
    } else {
        AssertSqlSafe(format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", depth - 1)).into_sql_str()
    }
}

pub fn rollback_ansi_transaction_sql(depth: usize) -> SqlStr {
    if depth == 1 {
        "ROLLBACK".into_sql_str()
    } else {
        AssertSqlSafe(format!(
            "ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}",
            depth - 1
        ))
        .into_sql_str()
    }
}

#[cfg(test)]
mod tests {
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
}
