use either::Either;
use futures::future::BoxFuture;
use futures::stream::BoxStream;

use crate::query_engine::database::Database;
use crate::query_engine::error::BoxDynError;
use crate::query_engine::executor::{Execute, Executor};
use crate::query_engine::sql_str::{SqlSafeStr, SqlStr};
use crate::query_engine::Error;

// AUTHOR'S NOTE: I was just going to call this API `sql()` and `Sql`, respectively,
// but realized that would be extremely annoying to deal with as a SQLite user
// because IDE smart completion would always recommend the `Sql` type first.
//
// It doesn't really need a super convenient name anyway as it's not meant to be used very often.

/// One or more raw SQL statements, separated by semicolons (`;`).
///
/// See [`raw_sql()`] for details.
pub struct RawSql(SqlStr);

/// Execute SQLite statements separated by semicolons.
///
/// This interface is suitable for trusted schema or administration statements.
/// It does not retain statements in the connection cache and exposes no bound
/// argument input. Use [`crate::query()`] with `.bind()` for dynamic values; this
/// API does not escape SQL fragments or identifiers.
///
/// Statements execute in order. Separate statements do not become atomic merely
/// because they share one SQL string. Run them through a [`crate::Transaction`]
/// when they must commit or roll back together.
///
/// ```
/// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) -> blog_storage_sqlite_access::Result<()> {
/// let mut tx = pool.begin().await?;
/// blog_storage_sqlite_access::raw_sql(
///     "CREATE TABLE entries(id INTEGER PRIMARY KEY, title TEXT NOT NULL);      INSERT INTO entries(title) VALUES ('Blog');"
/// )
/// .execute(&mut *tx)
/// .await?;
/// tx.commit().await?;
/// # Ok(())
/// # }
/// ```
pub fn raw_sql(sql: impl SqlSafeStr) -> RawSql {
    RawSql(sql.into_sql_str())
}

impl<DB: Database> Execute<'_, DB> for RawSql {
    fn sql(self) -> SqlStr {
        self.0
    }

    fn statement(&self) -> Option<&<DB as Database>::Statement> {
        None
    }

    fn take_arguments(&mut self) -> Result<Option<<DB as Database>::Arguments>, BoxDynError> {
        Ok(None)
    }

    fn persistent(&self) -> bool {
        false
    }
}

impl RawSql {
    /// Execute the SQL string and return the total number of rows affected.
    #[inline]
    pub async fn execute<'e, E, DB>(self, executor: E) -> crate::query_engine::Result<DB::QueryResult>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.execute(self).await
    }

    /// Execute the SQL string. Returns a stream which gives the number of rows affected for each statement in the string.
    #[inline]
    pub fn execute_many<'e, E, DB>(
        self,
        executor: E,
    ) -> BoxStream<'e, crate::query_engine::Result<DB::QueryResult>>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.execute_many(self)
    }

    /// Execute the SQL string and return the generated results as a stream.
    ///
    /// If the string contains multiple statements, their results will be concatenated together.
    #[inline]
    pub fn fetch<'e, E, DB>(self, executor: E) -> BoxStream<'e, Result<DB::Row, Error>>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.fetch(self)
    }

    /// Execute the SQL string and return the generated results as a stream.
    ///
    /// For each query in the stream, any generated rows are returned first,
    /// then the `QueryResult` with the number of rows affected.
    #[inline]
    pub fn fetch_many<'e, E, DB>(
        self,
        executor: E,
    ) -> BoxStream<'e, Result<Either<DB::QueryResult, DB::Row>, Error>>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.fetch_many(self)
    }

    /// Execute the SQL string and return all the resulting rows collected into a [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known upper bound,
    /// e.g. using `LIMIT`.
    #[inline]
    pub fn fetch_all<'e, E, DB>(self, executor: E) -> BoxFuture<'e, crate::query_engine::Result<Vec<DB::Row>>>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.fetch_all(self)
    }

    /// Execute the SQL string, returning the first row or [`Error::RowNotFound`] otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more than one row,
    /// it may lead to wasted CPU time and bandwidth on the database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring the query returns
    /// at most one row can result in a more optimal query plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    #[inline]
    pub fn fetch_one<'e, E, DB>(self, executor: E) -> BoxFuture<'e, crate::query_engine::Result<DB::Row>>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.fetch_one(self)
    }

    /// Execute the SQL string, returning the first row or [`None`] otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more than one row,
    /// it may lead to wasted CPU time and bandwidth on the database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring the query returns
    /// at most one row can result in a more optimal query plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    #[inline]
    pub async fn fetch_optional<'e, E, DB>(self, executor: E) -> crate::query_engine::Result<Option<DB::Row>>
    where
        DB: Database,
        E: Executor<'e, Database = DB>,
    {
        executor.fetch_optional(self).await
    }
}
