//! Runtime query-builder API.

use std::fmt::Display;
use std::fmt::Write;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::query_engine::arguments::{Arguments, IntoArguments};
use crate::query_engine::database::Database;
use crate::query_engine::encode::Encode;
use crate::query_engine::from_row::FromRow;
use crate::query_engine::query::Query;
use crate::query_engine::query_as::QueryAs;
use crate::query_engine::query_scalar::QueryScalar;
use crate::query_engine::sql_str::AssertSqlSafe;
use crate::query_engine::sql_str::SqlSafeStr;
use crate::query_engine::sql_str::SqlStr;
use crate::query_engine::types::Type;
use crate::query_engine::Either;

/// Construct a SQLite query from trusted SQL fragments and bound values.
///
/// Use [`QueryBuilder::push`] for validated SQL syntax and
/// [`QueryBuilder::push_bind`] for input values. [`QueryBuilder::push_values`]
/// constructs multi-row `INSERT` value lists. Keep the total number of parameters
/// within the configured SQLite limit.
pub struct QueryBuilder<DB>
where
    DB: Database,
{
    query: Arc<String>,
    init_len: usize,
    arguments: Option<<DB as Database>::Arguments>,
}

impl<DB: Database> Default for QueryBuilder<DB> {
    fn default() -> Self {
        QueryBuilder {
            init_len: 0,
            query: Default::default(),
            arguments: Some(Default::default()),
        }
    }
}

const ERROR: &str = "BUG: query must not be shared at this point in time";

impl<DB: Database> QueryBuilder<DB>
where
    DB: Database,
{
    // `init` is provided because a query will almost always start with a constant fragment
    // such as `INSERT INTO ...` or `SELECT ...`, etc.
    /// Start building a query with an initial SQL fragment, which may be an empty string.
    pub fn new(init: impl Into<String>) -> Self
    where
        <DB as Database>::Arguments: Default,
    {
        let init = init.into();

        QueryBuilder {
            init_len: init.len(),
            query: init.into(),
            arguments: Some(Default::default()),
        }
    }

    /// Construct a `QueryBuilder` with existing SQL and arguments.
    ///
    /// ### Note
    /// This does *not* check if `arguments` is valid for the given SQL.
    pub fn with_arguments<A>(init: impl Into<String>, arguments: A) -> Self
    where
        DB: Database,
        A: IntoArguments<DB>,
    {
        let init = init.into();

        QueryBuilder {
            init_len: init.len(),
            query: init.into(),
            arguments: Some(arguments.into_arguments()),
        }
    }

    #[inline]
    fn sanity_check(&self) {
        assert!(
            self.arguments.is_some(),
            "QueryBuilder must be reset before reuse after `.build()`"
        );
    }

    /// Append a SQL fragment to the query.
    ///
    /// May be a string or anything that implements `Display`.
    /// You can also use `format_args!()` here to push a formatted string without an intermediate
    /// allocation.
    ///
    /// ### Warning: Beware SQL Injection Vulnerabilities and Untrusted Input!
    /// You should *not* use this to insert input directly into the query from an untrusted user as
    /// this can be used by an attacker to extract sensitive data or take over your database.
    ///
    /// Security breaches due to SQL injection can cost your organization a lot of money from
    /// damage control and lost clients, betray the trust of your users in your system, and are just
    /// plain embarrassing. If you are unfamiliar with the threat that SQL injection imposes, you
    /// should take some time to learn more about it before proceeding:
    ///
    /// * [SQL Injection on OWASP.org](https://owasp.org/www-community/attacks/SQL_Injection)
    /// * [SQL Injection on Wikipedia](https://en.wikipedia.org/wiki/SQL_injection)
    ///     * See "Examples" for notable instances of security breaches due to SQL injection.
    ///
    /// This method does *not* perform sanitization. Instead, you should use
    /// [`.push_bind()`][Self::push_bind] which inserts a placeholder into the query and then
    /// sends the possibly untrustworthy value separately (called a "bind argument") so that it
    /// cannot be misinterpreted by the database server.
    ///
    /// Note that you should still at least have some sort of sanity checks on the values you're
    /// sending as that's just good practice and prevent other types of attacks against your system,
    /// e.g. check that strings aren't too long, numbers are within expected ranges, etc.
    pub fn push(&mut self, sql: impl Display) -> &mut Self {
        self.sanity_check();
        let query: &mut String = Arc::get_mut(&mut self.query).expect(ERROR);

        write!(query, "{sql}").expect("error formatting `sql`");

        self
    }

    /// Append a SQLite parameter placeholder and bind a value.
    ///
    /// The builder uses the SQLite argument formatter. A statement's parameter count
    /// must fit the configured `SQLITE_LIMIT_VARIABLE_NUMBER`; the repository's
    /// SQLite build can set `SQLITE_MAX_VARIABLE_NUMBER`. Exceeding the active limit
    /// returns an execution error or may trigger a builder sanity check.
    ///
    /// This separates input values from SQL syntax; it cannot bind a table name,
    /// column name or SQL fragment.
    pub fn push_bind<'t, T>(&mut self, value: T) -> &mut Self
    where
        T: Encode<'t, DB> + Type<DB>,
    {
        self.sanity_check();

        let arguments = self
            .arguments
            .as_mut()
            .expect("BUG: Arguments taken already");
        arguments.add(value).expect("Failed to add argument");

        let query: &mut String = Arc::get_mut(&mut self.query).expect(ERROR);
        arguments
            .format_placeholder(query)
            .expect("error in format_placeholder");

        self
    }

    /// Start a list that inserts `separator` between successive items.
    ///
    /// Use the returned helper's `push_bind` method for values, and `push` only for
    /// trusted SQL fragments. Empty input yields an empty list, which the caller must
    /// handle when the surrounding syntax requires at least one item.
    ///
    /// ```
    /// use blog_storage_sqlite_access::{QueryBuilder, Sqlite};
    ///
    /// let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT 1 WHERE 1 IN (");
    /// let mut values = builder.separated(", ");
    /// values.push_bind(1_i64).push_bind(2_i64);
    /// values.push_unseparated(")");
    /// ```
    pub fn separated<Sep>(&mut self, separator: Sep) -> Separated<'_, DB, Sep>
    where
        Sep: Display,
    {
        self.sanity_check();

        Separated {
            query_builder: self,
            separator,
            push_separator: false,
        }
    }

    // Most of the `QueryBuilder` API is purposefully very low-level but this was a commonly
    // requested use-case so it made sense to support.
    /// Append a `VALUES` list for a multi-row SQLite insert.
    ///
    /// The callback receives a separated tuple builder and one input item. Bind each
    /// field through that tuple builder. Empty input produces invalid `VALUES` syntax,
    /// so handle it before building the query. Chunk large batches so all bound fields
    /// fit the active SQLite parameter limit.
    ///
    /// ```
    /// use blog_storage_sqlite_access::{QueryBuilder, Sqlite};
    ///
    /// let entries = [(1_i64, "first"), (2, "second")];
    /// let mut builder: QueryBuilder<Sqlite> =
    ///     QueryBuilder::new("INSERT INTO entries(id, title) ");
    /// builder.push_values(entries, |mut row, (id, title)| {
    ///     row.push_bind(id).push_bind(title);
    /// });
    /// let query = builder.build();
    /// ```
    ///
    /// Each distinct number of tuples produces a different statement shape. Disable
    /// statement persistence when a highly variable batch size would fill the cache
    /// with shapes that will not be reused.
    pub fn push_values<I, F>(&mut self, tuples: I, mut push_tuple: F) -> &mut Self
    where
        I: IntoIterator,
        F: FnMut(Separated<'_, DB, &'static str>, I::Item),
    {
        self.sanity_check();

        self.push("VALUES ");

        let mut separated = self.separated(", ");

        for tuple in tuples {
            separated.push("(");

            // use a `Separated` with a separate (hah) internal state
            push_tuple(separated.query_builder.separated(", "), tuple);

            separated.push_unseparated(")");
        }

        debug_assert!(
            separated.push_separator,
            "No value being pushed. QueryBuilder may not build correct sql query!"
        );

        separated.query_builder
    }

    /// Append a parenthesized list of row-value tuples.
    ///
    /// The callback fills each tuple using bound values. This can express SQLite
    /// row-value lists where accepted by the surrounding query. The caller supplies
    /// the surrounding SQL and must handle empty input explicitly. Chunk the input to
    /// respect the active SQLite parameter limit.
    pub fn push_tuples<I, F>(&mut self, tuples: I, mut push_tuple: F) -> &mut Self
    where
        I: IntoIterator,
        F: FnMut(Separated<'_, DB, &'static str>, I::Item),
    {
        self.sanity_check();

        self.push(" (");

        let mut separated = self.separated(", ");

        for tuple in tuples {
            separated.push("(");

            push_tuple(separated.query_builder.separated(", "), tuple);

            separated.push_unseparated(")");
        }
        separated.push_unseparated(") ");

        separated.query_builder
    }

    /// Produce an executable query from this builder.
    ///
    /// ### Note: Query is not Checked
    /// It is your responsibility to ensure that you produce a syntactically correct query here,
    /// this API has no way to check it for you.
    ///
    /// ### Note: Reuse
    /// You can reuse this builder afterwards to amortize the allocation overhead of the query
    /// string, however you must call [`.reset()`][Self::reset] first, which returns `Self`
    /// to the state it was in immediately after [`new()`][Self::new].
    ///
    /// Calling any other method but `.reset()` after `.build()` will panic for sanity reasons.
    pub fn build(&mut self) -> Query<'_, DB, <DB as Database>::Arguments> {
        self.sanity_check();

        Query {
            statement: Either::Left(self.sql()),
            arguments: self.arguments.take().map(Ok),
            database: PhantomData,
            persistent: true,
        }
    }

    /// Produce an executable query from this builder.
    ///
    /// ### Note: Query is not Checked
    /// It is your responsibility to ensure that you produce a syntactically correct query here,
    /// this API has no way to check it for you.
    ///
    /// ### Note: Reuse
    /// You can reuse this builder afterwards to amortize the allocation overhead of the query
    /// string, however you must call [`.reset()`][Self::reset] first, which returns `Self`
    /// to the state it was in immediately after [`new()`][Self::new].
    ///
    /// Calling any other method but `.reset()` after `.build()` will panic for sanity reasons.
    pub fn build_query_as<'q, T: FromRow<'q, DB::Row>>(
        &'q mut self,
    ) -> QueryAs<'q, DB, T, <DB as Database>::Arguments> {
        QueryAs {
            inner: self.build(),
            output: PhantomData,
        }
    }

    /// Produce an executable query from this builder.
    ///
    /// ### Note: Query is not Checked
    /// It is your responsibility to ensure that you produce a syntactically correct query here,
    /// this API has no way to check it for you.
    ///
    /// ### Note: Reuse
    /// You can reuse this builder afterwards to amortize the allocation overhead of the query
    /// string, however you must call [`.reset()`][Self::reset] first, which returns `Self`
    /// to the state it was in immediately after [`new()`][Self::new].
    ///
    /// Calling any other method but `.reset()` after `.build()` will panic for sanity reasons.
    pub fn build_query_scalar<'q, T>(
        &'q mut self,
    ) -> QueryScalar<'q, DB, T, <DB as Database>::Arguments>
    where
        DB: Database,
        (T,): for<'r> FromRow<'r, DB::Row>,
    {
        QueryScalar {
            inner: self.build_query_as(),
        }
    }

    /// Reset this `QueryBuilder` back to its initial state.
    ///
    /// The query is truncated to the initial fragment provided to [`new()`][Self::new] and
    /// the bind arguments are reset.
    pub fn reset(&mut self) -> &mut Self {
        // Someone can hold onto a clone of `self.query`, to avoid panicking here we should just
        // allocate a new `String`.
        let query: &mut String = Arc::make_mut(&mut self.query);
        query.truncate(self.init_len);
        self.arguments = Some(Default::default());

        self
    }

    /// Get the current build SQL; **note**: may not be syntactically correct.
    pub fn sql(&self) -> SqlStr {
        AssertSqlSafe(self.query.clone()).into_sql_str()
    }

    /// Deconstruct this `QueryBuilder`, returning the built SQL. May not be syntactically correct.
    pub fn into_string(self) -> String {
        Arc::unwrap_or_clone(self.query)
    }

    /// Deconstruct this `QueryBuilder`, returning the built SQL. May not be syntactically correct.
    pub fn into_sql(self) -> SqlStr {
        AssertSqlSafe(self.query).into_sql_str()
    }
}

/// A wrapper around `QueryBuilder` for creating comma(or other token)-separated lists.
///
/// See [`QueryBuilder::separated()`] for details.
#[allow(explicit_outlives_requirements)]
pub struct Separated<'qb, DB, Sep>
where
    DB: Database,
{
    query_builder: &'qb mut QueryBuilder<DB>,
    separator: Sep,
    push_separator: bool,
}

impl<DB, Sep> Separated<'_, DB, Sep>
where
    DB: Database,
    Sep: Display,
{
    /// Push the separator if applicable, and then the given SQL fragment.
    ///
    /// See [`QueryBuilder::push()`] for details.
    pub fn push(&mut self, sql: impl Display) -> &mut Self {
        if self.push_separator {
            self.query_builder
                .push(format_args!("{}{}", self.separator, sql));
        } else {
            self.query_builder.push(sql);
            self.push_separator = true;
        }

        self
    }

    /// Push a SQL fragment without a separator.
    ///
    /// Simply calls [`QueryBuilder::push()`] directly.
    pub fn push_unseparated(&mut self, sql: impl Display) -> &mut Self {
        self.query_builder.push(sql);
        self
    }

    /// Push the separator if applicable, then append a bind argument.
    ///
    /// See [`QueryBuilder::push_bind()`] for details.
    pub fn push_bind<'t, T>(&mut self, value: T) -> &mut Self
    where
        T: Encode<'t, DB> + Type<DB>,
    {
        if self.push_separator {
            self.query_builder.push(&self.separator);
        }

        self.query_builder.push_bind(value);
        self.push_separator = true;

        self
    }

    /// Append a bound SQLite parameter without inserting this list's separator.
    pub fn push_bind_unseparated<'t, T>(&mut self, value: T) -> &mut Self
    where
        T: Encode<'t, DB> + Type<DB>,
    {
        self.query_builder.push_bind(value);
        self
    }
}
