use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};

use crate::query_engine::arguments::IntoArguments;
use crate::query_engine::database::{Database, HasStatementCache};
use crate::query_engine::encode::Encode;
use crate::query_engine::error::{BoxDynError, Error};
use crate::query_engine::executor::{Execute, Executor};
use crate::query_engine::from_row::FromRow;
use crate::query_engine::query::{
    query, query_statement, query_statement_with, query_with_result, Query,
};
use crate::query_engine::sql_str::{SqlSafeStr, SqlStr};
use crate::query_engine::types::Type;

/// A single SQL query as a prepared statement, mapping results using [`FromRow`].
/// Returned by [`query_as()`].
#[must_use = "query must be executed to affect database"]
pub struct QueryAs<'q, DB: Database, O, A> {
    pub(crate) inner: Query<'q, DB, A>,
    pub(crate) output: PhantomData<O>,
}

impl<'q, DB, O: Send, A: Send> Execute<'q, DB> for QueryAs<'q, DB, O, A>
where
    DB: Database,
    A: 'q + IntoArguments<DB>,
{
    #[inline]
    fn sql(self) -> SqlStr {
        self.inner.sql()
    }

    #[inline]
    fn statement(&self) -> Option<&DB::Statement> {
        self.inner.statement()
    }

    #[inline]
    fn take_arguments(&mut self) -> Result<Option<<DB as Database>::Arguments>, BoxDynError> {
        self.inner.take_arguments()
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.inner.persistent()
    }
}

impl<'q, DB: Database, O> QueryAs<'q, DB, O, <DB as Database>::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`](Query::bind).
    pub fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        self.inner = self.inner.bind(value);
        self
    }
}

impl<DB, O, A> QueryAs<'_, DB, O, A>
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
impl<'q, DB, O, A> QueryAs<'q, DB, O, A>
where
    DB: Database,
    A: 'q + IntoArguments<DB>,
    O: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
{
    /// Execute the query and return the generated results as a stream.
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<O, Error>>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        executor
            .fetch(self.inner)
            .map(|row| O::from_row(&row?))
            .boxed()
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
        O: 'e,
        A: 'e,
    {
        self.fetch(executor).try_collect().await
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
    pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        self.fetch_optional(executor)
            .await
            .and_then(|row| row.ok_or(Error::RowNotFound))
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
    pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        let row = executor.fetch_optional(self.inner).await?;
        if let Some(row) = row {
            O::from_row(&row).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Execute a cached SQLite query and map each row through [`FromRow`].
///
/// Tuple mappings decode columns by position. Struct mappings generated by the
/// `derive` feature match fields to column names. Select columns explicitly,
/// particularly in joins where names can repeat. Missing columns and incompatible
/// Rust/SQLite types return decode errors.
///
/// ```
/// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) -> blog_storage_sqlite_access::Result<()> {
/// let row: (i64, String) = blog_storage_sqlite_access::query_as("SELECT ?1 AS id, ?2 AS name")
///     .bind(1_i64)
///     .bind("Blog")
///     .fetch_one(pool)
///     .await?;
/// assert_eq!(row, (1, "Blog".to_owned()));
/// # Ok(())
/// # }
/// ```
///
/// With the `derive` feature, the same two columns can be mapped by name:
///
/// ```
/// # #[cfg(feature = "derive")]
/// #[derive(blog_storage_sqlite_access::FromRow)]
/// struct Entry {
///     id: i64,
///     name: String,
/// }
/// ```
///
/// [`QueryAs::fetch_one`] reports [`Error::RowNotFound`] if no row is returned;
/// [`QueryAs::fetch_optional`] returns `None`. Use a known result bound when
/// collecting rows with [`QueryAs::fetch_all`]. SQLite type mappings are described
/// in [`crate::sqlite::types`], and parameter binding in [`crate::query()`].
#[inline]
pub fn query_as<'q, DB, O>(sql: impl SqlSafeStr) -> QueryAs<'q, DB, O, <DB as Database>::Arguments>
where
    DB: Database,
    O: for<'r> FromRow<'r, DB::Row>,
{
    QueryAs {
        inner: query(sql),
        output: PhantomData,
    }
}

/// Execute a single SQL query, with the given arguments as a prepared statement (transparently cached).
/// Maps rows to Rust types using [`FromRow`].
///
/// For details about prepared statements and allowed SQL syntax, see [`query()`][crate::query_engine::query::query].
///
/// For details about type mapping from [`FromRow`], see [`query_as()`].
#[inline]
pub fn query_as_with<'q, DB, O, A>(sql: impl SqlSafeStr, arguments: A) -> QueryAs<'q, DB, O, A>
where
    DB: Database,
    A: IntoArguments<DB>,
    O: for<'r> FromRow<'r, DB::Row>,
{
    query_as_with_result(sql, Ok(arguments))
}

/// Same as [`query_as_with`] but takes arguments as a Result
#[inline]
pub fn query_as_with_result<'q, DB, O, A>(
    sql: impl SqlSafeStr,
    arguments: Result<A, BoxDynError>,
) -> QueryAs<'q, DB, O, A>
where
    DB: Database,
    A: IntoArguments<DB>,
    O: for<'r> FromRow<'r, DB::Row>,
{
    QueryAs {
        inner: query_with_result(sql, arguments),
        output: PhantomData,
    }
}

// Make a SQL query from a statement, that is mapped to a concrete type.
pub fn query_statement_as<DB, O>(
    statement: &DB::Statement,
) -> QueryAs<'_, DB, O, <DB as Database>::Arguments>
where
    DB: Database,
    O: for<'r> FromRow<'r, DB::Row>,
{
    QueryAs {
        inner: query_statement(statement),
        output: PhantomData,
    }
}

// Make a SQL query from a statement, with the given arguments, that is mapped to a concrete type.
pub fn query_statement_as_with<'q, DB, O, A>(
    statement: &'q DB::Statement,
    arguments: A,
) -> QueryAs<'q, DB, O, A>
where
    DB: Database,
    A: IntoArguments<DB>,
    O: for<'r> FromRow<'r, DB::Row>,
{
    QueryAs {
        inner: query_statement_with(statement, arguments),
        output: PhantomData,
    }
}
