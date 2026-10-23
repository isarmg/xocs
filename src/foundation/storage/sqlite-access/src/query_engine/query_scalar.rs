use futures::stream::BoxStream;
use futures::{StreamExt, TryFutureExt, TryStreamExt};

use crate::query_engine::arguments::IntoArguments;
use crate::query_engine::database::{Database, HasStatementCache};
use crate::query_engine::encode::Encode;
use crate::query_engine::error::{BoxDynError, Error};
use crate::query_engine::executor::{Execute, Executor};
use crate::query_engine::from_row::FromRow;
use crate::query_engine::query_as::{
    query_as, query_as_with_result, query_statement_as, query_statement_as_with, QueryAs,
};
use crate::query_engine::sql_str::{SqlSafeStr, SqlStr};
use crate::query_engine::types::Type;

/// A single SQL query as a prepared statement which extracts only the first column of each row.
/// Returned by [`query_scalar()`].
#[must_use = "query must be executed to affect database"]
pub struct QueryScalar<'q, DB: Database, O, A> {
    pub(crate) inner: QueryAs<'q, DB, (O,), A>,
}

impl<'q, DB: Database, O: Send, A: Send> Execute<'q, DB> for QueryScalar<'q, DB, O, A>
where
    A: 'q + IntoArguments<DB>,
{
    #[inline]
    fn sql(self) -> SqlStr {
        self.inner.sql()
    }

    fn statement(&self) -> Option<&DB::Statement> {
        self.inner.statement()
    }

    #[inline]
    fn take_arguments(&mut self) -> Result<Option<<DB as Database>::Arguments>, BoxDynError> {
        self.inner.take_arguments()
    }

    #[inline]
    fn persistent(&self) -> bool {
        Execute::persistent(&self.inner)
    }
}

impl<'q, DB: Database, O> QueryScalar<'q, DB, O, <DB as Database>::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`](crate::query_engine::query::Query::bind).
    pub fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        self.inner = self.inner.bind(value);
        self
    }
}

impl<DB, O, A> QueryScalar<'_, DB, O, A>
where
    DB: Database + HasStatementCache,
{
    /// If `true`, the statement will get prepared once and cached to the
    /// connection's statement cache.
    ///
    /// If queried once with the flag set to `true`, all subsequent queries
    /// matching the one with the flag will use the cached statement until the
    /// cache is cleared.
    ///
    /// If `false`, the prepared statement will be closed after execution.
    ///
    /// Default: `true`.
    pub fn persistent(mut self, value: bool) -> Self {
        self.inner = self.inner.persistent(value);
        self
    }
}

// FIXME: This is very close, nearly 1:1 with `Map`
// noinspection DuplicatedCode
impl<'q, DB, O, A> QueryScalar<'q, DB, O, A>
where
    DB: Database,
    O: Send + Unpin,
    A: 'q + IntoArguments<DB>,
    (O,): Send + Unpin + for<'r> FromRow<'r, DB::Row>,
{
    /// Execute the query and return the generated results as a stream.
    #[inline]
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<O, Error>>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        A: 'e,
        O: 'e,
    {
        self.inner.fetch(executor).map_ok(|it| it.0).boxed()
    }

    /// Execute the query and return all the resulting rows collected into a [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known upper bound,
    /// e.g. using `LIMIT`.
    #[inline]
    pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        (O,): 'e,
        A: 'e,
    {
        self.inner
            .fetch(executor)
            .map_ok(|it| it.0)
            .try_collect()
            .await
    }

    /// Execute the query, returning the first row or [`Error::RowNotFound`] otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more than one row,
    /// it may lead to wasted CPU time and bandwidth on the database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring the query returns at most one row
    /// can result in a more optimal query plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    #[inline]
    pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        self.inner.fetch_one(executor).map_ok(|it| it.0).await
    }

    /// Execute the query, returning the first row or `None` otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more than one row,
    /// it may lead to wasted CPU time and bandwidth on the database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring the query returns at most one row
    /// can result in a more optimal query plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    #[inline]
    pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        Ok(self.inner.fetch_optional(executor).await?.map(|it| it.0))
    }
}

/// Execute a cached SQLite query and decode the first column of each row.
///
/// Additional columns are ignored. The output must have a compatible SQLite type
/// and implement the corresponding decoding traits. Use `i64` for `COUNT`, `bool`
/// for `EXISTS`, and `Option<T>` for a selected value that may be SQL `NULL`.
///
/// ```
/// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) -> blog_storage_sqlite_access::Result<()> {
/// let value: i64 = blog_storage_sqlite_access::query_scalar("SELECT ?1")
///     .bind(42_i64)
///     .fetch_one(pool)
///     .await?;
/// assert_eq!(value, 42);
///
/// let nullable: Option<i64> = blog_storage_sqlite_access::query_scalar("SELECT NULL")
///     .fetch_one(pool)
///     .await?;
/// assert_eq!(nullable, None);
/// # Ok(())
/// # }
/// ```
///
/// `fetch_optional` distinguishes an absent row from a returned row: combined with
/// an `Option<T>` output, it yields `Option<Option<T>>`. `fetch_one` returns
/// [`Error::RowNotFound`] for no row; later rows are ignored. `fetch_all` collects
/// the first column of every row, so bound the query result when memory is limited.
/// See [`crate::query()`] for parameter binding.
#[inline]
pub fn query_scalar<'q, DB, O>(
    sql: impl SqlSafeStr,
) -> QueryScalar<'q, DB, O, <DB as Database>::Arguments>
where
    DB: Database,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_as(sql),
    }
}

/// Execute a SQL query as a prepared statement (transparently cached), with the given arguments,
/// and extract the first column of each row.
///
/// See [`query_scalar()`] for details.
///
/// For details about prepared statements and allowed SQL syntax, see [`query()`][crate::query_engine::query::query].
#[inline]
pub fn query_scalar_with<'q, DB, O, A>(
    sql: impl SqlSafeStr,
    arguments: A,
) -> QueryScalar<'q, DB, O, A>
where
    DB: Database,
    A: IntoArguments<DB>,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    query_scalar_with_result(sql, Ok(arguments))
}

/// Same as [`query_scalar_with`] but takes arguments as Result
#[inline]
pub fn query_scalar_with_result<'q, DB, O, A>(
    sql: impl SqlSafeStr,
    arguments: Result<A, BoxDynError>,
) -> QueryScalar<'q, DB, O, A>
where
    DB: Database,
    A: IntoArguments<DB>,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_as_with_result(sql, arguments),
    }
}

// Make a SQL query from a statement, that is mapped to a concrete value.
pub fn query_statement_scalar<DB, O>(
    statement: &DB::Statement,
) -> QueryScalar<'_, DB, O, <DB as Database>::Arguments>
where
    DB: Database,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_statement_as(statement),
    }
}

// Make a SQL query from a statement, with the given arguments, that is mapped to a concrete value.
pub fn query_statement_scalar_with<'q, DB, O, A>(
    statement: &'q DB::Statement,
    arguments: A,
) -> QueryScalar<'q, DB, O, A>
where
    DB: Database,
    A: IntoArguments<DB>,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_statement_as_with(statement, arguments),
    }
}
