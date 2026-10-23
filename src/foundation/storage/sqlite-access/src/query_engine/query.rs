use std::{future, marker::PhantomData};

use either::Either;
use futures::stream::BoxStream;
use futures::{StreamExt, TryFutureExt, TryStreamExt};

use crate::query_engine::arguments::{Arguments, IntoArguments};
use crate::query_engine::database::{Database, HasStatementCache};
use crate::query_engine::encode::Encode;
use crate::query_engine::error::{BoxDynError, Error};
use crate::query_engine::executor::{Execute, Executor};
use crate::query_engine::sql_str::{SqlSafeStr, SqlStr};
use crate::query_engine::statement::Statement;
use crate::query_engine::types::Type;

/// A single SQL query as a prepared statement. Returned by [`query()`].
#[must_use = "query must be executed to affect database"]
pub struct Query<'q, DB: Database, A> {
    pub(crate) statement: Either<SqlStr, &'q DB::Statement>,
    pub(crate) arguments: Option<Result<A, BoxDynError>>,
    pub(crate) database: PhantomData<DB>,
    pub(crate) persistent: bool,
}

/// A single SQL query that will map its results to an owned Rust type.
///
/// Executes as a prepared statement.
///
/// Returned by [`Query::try_map`], `query!()`, etc. Has most of the same methods as [`Query`] but
/// the return types are changed to reflect the mapping. However, there is no equivalent of
/// [`Query::execute`] as it doesn't make sense to map the result type and then ignore it.
///
/// [`Query::bind`] is also omitted; stylistically we recommend placing your `.bind()` calls
/// before `.try_map()`. This is also to prevent adding superfluous binds to the result of
/// `query!()` et al.
#[must_use = "query must be executed to affect database"]
pub struct Map<'q, DB: Database, F, A> {
    inner: Query<'q, DB, A>,
    mapper: F,
}

impl<'q, DB, A> Execute<'q, DB> for Query<'q, DB, A>
where
    DB: Database,
    A: Send + IntoArguments<DB>,
{
    #[inline]
    fn sql(self) -> SqlStr {
        match self.statement {
            Either::Right(statement) => statement.sql().clone(),
            Either::Left(sql) => sql,
        }
    }

    fn statement(&self) -> Option<&DB::Statement> {
        match self.statement {
            Either::Right(statement) => Some(statement),
            Either::Left(_) => None,
        }
    }

    #[inline]
    fn take_arguments(&mut self) -> Result<Option<<DB as Database>::Arguments>, BoxDynError> {
        self.arguments
            .take()
            .transpose()
            .map(|option| option.map(IntoArguments::into_arguments))
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.persistent
    }
}

impl<DB: Database> Query<'_, DB, <DB as Database>::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// If the number of times this is called does not match the number of bind parameters that
    /// appear in the SQLite query, then an error
    /// will be returned when this query is executed.
    ///
    /// There is no validation that the value is of the type expected by the query. Most SQL
    /// values may be coerced by SQLite; decoded output types are checked separately.
    ///
    /// If encoding the value fails, the error is stored and later surfaced when executing the query.
    pub fn bind<'t, T: Encode<'t, DB> + Type<DB>>(mut self, value: T) -> Self {
        let Ok(arguments) = self.get_arguments() else {
            return self;
        };

        let argument_number = arguments.len() + 1;
        if let Err(error) = arguments.add(value) {
            self.arguments = Some(Err(format!(
                "Encoding argument ${argument_number} failed: {error}"
            )
            .into()));
        }

        self
    }

    /// Like [`Query::bind`] but immediately returns an error if encoding a value failed.
    pub fn try_bind<'t, T: Encode<'t, DB> + Type<DB>>(
        &mut self,
        value: T,
    ) -> Result<(), BoxDynError> {
        let arguments = self.get_arguments()?;

        arguments.add(value)
    }

    fn get_arguments(&mut self) -> Result<&mut DB::Arguments, BoxDynError> {
        let Some(Ok(arguments)) = self.arguments.as_mut().map(Result::as_mut) else {
            return Err("A previous call to Query::bind produced an error"
                .to_owned()
                .into());
        };

        Ok(arguments)
    }
}

impl<DB, A> Query<'_, DB, A>
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
        self.persistent = value;
        self
    }
}

impl<'q, DB, A: Send> Query<'q, DB, A>
where
    DB: Database,
    A: 'q + IntoArguments<DB>,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`](Query::try_map) for a fallible version of this method.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn map<F, O>(
        self,
        mut f: F,
    ) -> Map<'q, DB, impl FnMut(DB::Row) -> Result<O, Error> + Send, A>
    where
        F: FnMut(DB::Row) -> O + Send,
        O: Unpin,
    {
        self.try_map(move |row| Ok(f(row)))
    }

    /// Map each row in the result to another type.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn try_map<F, O>(self, f: F) -> Map<'q, DB, F, A>
    where
        F: FnMut(DB::Row) -> Result<O, Error> + Send,
        O: Unpin,
    {
        Map {
            inner: self,
            mapper: f,
        }
    }

    /// Execute the query and return the total number of rows affected.
    #[inline]
    pub async fn execute<'e, 'c: 'e, E>(self, executor: E) -> Result<DB::QueryResult, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.execute(self).await
    }

    /// Execute the query and return the generated results as a stream.
    #[inline]
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<DB::Row, Error>>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch(self)
    }

    /// Execute the query and return all the resulting rows collected into a [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known upper bound,
    /// e.g. using `LIMIT`.
    #[inline]
    pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<DB::Row>, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_all(self).await
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
    pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<DB::Row, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_one(self).await
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
    pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<DB::Row>, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_optional(self).await
    }
}

impl<'q, DB, F: Send, A: Send> Execute<'q, DB> for Map<'q, DB, F, A>
where
    DB: Database,
    A: IntoArguments<DB>,
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
        self.inner.arguments.is_some()
    }
}

impl<'q, DB, F, O, A> Map<'q, DB, F, A>
where
    DB: Database,
    F: FnMut(DB::Row) -> Result<O, Error> + Send,
    O: Send + Unpin,
    A: 'q + Send + IntoArguments<DB>,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`](Map::try_map) for a fallible version of this method.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn map<G, P>(
        self,
        mut g: G,
    ) -> Map<'q, DB, impl FnMut(DB::Row) -> Result<P, Error> + Send, A>
    where
        G: FnMut(O) -> P + Send,
        P: Unpin,
    {
        self.try_map(move |data| Ok(g(data)))
    }

    /// Map each row in the result to another type.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn try_map<G, P>(
        self,
        mut g: G,
    ) -> Map<'q, DB, impl FnMut(DB::Row) -> Result<P, Error> + Send, A>
    where
        G: FnMut(O) -> Result<P, Error> + Send,
        P: Unpin,
    {
        let mut f = self.mapper;
        Map {
            inner: self.inner,
            mapper: move |row| f(row).and_then(&mut g),
        }
    }

    /// Execute the query and return the generated results as a stream.
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<O, Error>>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        let mut mapper = self.mapper;
        executor
            .fetch(self.inner)
            .map(move |row| mapper(row?))
            .boxed()
    }

    /// Execute the query and return all the resulting rows collected into a [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known upper bound,
    /// e.g. using `LIMIT`.
    pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
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
        F: 'e,
        O: 'e,
    {
        self.fetch_optional(executor)
            .and_then(|row| {
                future::ready(match row {
                    Some(row) => Ok(row),
                    None => Err(Error::RowNotFound),
                })
            })
            .await
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
    pub async fn fetch_optional<'e, 'c: 'e, E>(mut self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        let row = executor.fetch_optional(self.inner).await?;

        if let Some(row) = row {
            (self.mapper)(row).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Execute a single SQL query as a prepared statement (explicitly created).
pub fn query_statement<DB>(statement: &DB::Statement) -> Query<'_, DB, <DB as Database>::Arguments>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Ok(Default::default())),
        statement: Either::Right(statement),
        persistent: true,
    }
}

/// Execute a single SQL query as a prepared statement (explicitly created), with the given arguments.
pub fn query_statement_with<DB, A>(statement: &DB::Statement, arguments: A) -> Query<'_, DB, A>
where
    DB: Database,
    A: IntoArguments<DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(Ok(arguments)),
        statement: Either::Right(statement),
        persistent: true,
    }
}

/// Execute a parameterized SQLite statement using a connection's statement cache.
///
/// Build the statement from trusted SQL, then bind untrusted values with
/// [`Query::bind`]. SQLite placeholders such as `?1` may occur more than once;
/// the bound value is separate from SQL syntax. Identifiers and SQL fragments
/// cannot be supplied as value parameters and need separate validation.
///
/// ```
/// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) -> Result<(), blog_storage_sqlite_access::Error> {
/// let title = "Blog";
/// let rows = blog_storage_sqlite_access::query("SELECT ?1 AS title")
///     .bind(title)
///     .fetch_all(pool)
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// The query supports fetching rows, fetching an optional row, executing for a
/// result count and mapping rows through the [`Query`] methods. Values must
/// implement [`Encode`] and [`Type<DB>`]; see [`crate::sqlite::types`] for the
/// SQLite conversions. SQL and returned column types are checked at execution
/// and decode boundaries, not by compile-time query macros.
pub fn query<'a, DB>(sql: impl SqlSafeStr) -> Query<'a, DB, <DB as Database>::Arguments>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Ok(Default::default())),
        statement: Either::Left(sql.into_sql_str()),
        persistent: true,
    }
}

/// Execute a SQL query as a prepared statement (transparently cached), with the given arguments.
///
/// See [`query()`][query] for details, such as supported syntax.
pub fn query_with<'q, DB, A>(sql: impl SqlSafeStr, arguments: A) -> Query<'q, DB, A>
where
    DB: Database,
    A: IntoArguments<DB>,
{
    query_with_result(sql, Ok(arguments))
}

/// Same as [`query_with`] but is initialized with a Result of arguments instead
pub fn query_with_result<'q, DB, A>(
    sql: impl SqlSafeStr,
    arguments: Result<A, BoxDynError>,
) -> Query<'q, DB, A>
where
    DB: Database,
    A: IntoArguments<DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(arguments),
        statement: Either::Left(sql.into_sql_str()),
        persistent: true,
    }
}
