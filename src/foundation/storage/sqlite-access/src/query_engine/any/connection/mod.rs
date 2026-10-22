use futures::future::BoxFuture;
use std::future::Future;

use crate::query_engine::any::{Any, AnyConnectOptions};
use crate::query_engine::connection::{ConnectOptions, Connection};
use crate::query_engine::error::Error;

use crate::query_engine::config;
use crate::query_engine::database::Database;
use crate::query_engine::sql_str::SqlSafeStr;
use crate::query_engine::transaction::Transaction;
pub use backend::AnyConnectionBackend;

mod backend;
mod executor;

/// A connection to _any_ SQLx database.
///
/// The database driver used is determined by the scheme
/// of the connection url.
///
/// ```text
/// postgres://postgres@localhost/test
/// sqlite://a.sqlite
/// ```
#[derive(Debug)]
pub struct AnyConnection {
    pub(crate) backend: Box<dyn AnyConnectionBackend>,
}

impl AnyConnection {
    /// Returns the name of the database backend in use (e.g. PostgreSQL, MySQL, SQLite, etc.)
    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }

    pub(crate) fn connect(options: &AnyConnectOptions) -> BoxFuture<'_, crate::query_engine::Result<Self>> {
        Box::pin(async {
            let driver = crate::query_engine::any::driver::from_url(&options.database_url)?;
            (*driver.connect)(options, None).await
        })
    }

    /// UNSTABLE: for use with `sqlx-cli`
    ///
    /// Connect to the database, and instruct the nested driver to
    /// read options from the sqlx.toml file as appropriate.
    #[doc(hidden)]
    pub async fn connect_with_driver_config(
        url: &str,
        driver_config: &config::drivers::Config,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let options: AnyConnectOptions = url.parse()?;

        let driver = crate::query_engine::any::driver::from_url(&options.database_url)?;
        (*driver.connect)(&options, Some(driver_config)).await
    }

    pub(crate) fn connect_with_db<'a, DB: Database>(
        options: &'a AnyConnectOptions,
        driver_config: Option<&'a config::drivers::Config>,
    ) -> BoxFuture<'a, crate::query_engine::Result<Self>>
    where
        DB::Connection: AnyConnectionBackend,
        <DB::Connection as Connection>::Options:
            for<'b> TryFrom<&'b AnyConnectOptions, Error = Error>,
    {
        let res = TryFrom::try_from(options);

        Box::pin(async move {
            let mut options: <DB::Connection as Connection>::Options = res?;

            if let Some(config) = driver_config {
                options = options.__unstable_apply_driver_config(config)?;
            }

            Ok(AnyConnection {
                backend: Box::new(options.connect().await?),
            })
        })
    }
}

impl Connection for AnyConnection {
    type Database = Any;

    type Options = AnyConnectOptions;

    fn close(self) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        self.backend.close()
    }

    fn close_hard(self) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        self.backend.close()
    }

    fn ping(&mut self) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        self.backend.ping()
    }

    fn begin(
        &mut self,
    ) -> impl Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_
    where
        Self: Sized,
    {
        Transaction::begin(self, None)
    }

    fn begin_with(
        &mut self,
        statement: impl SqlSafeStr,
    ) -> impl Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_
    where
        Self: Sized,
    {
        Transaction::begin(self, Some(statement.into_sql_str()))
    }

    fn cached_statements_size(&self) -> usize {
        self.backend.cached_statements_size()
    }

    fn clear_cached_statements(&mut self) -> impl Future<Output = crate::query_engine::Result<()>> + Send + '_ {
        self.backend.clear_cached_statements()
    }

    fn shrink_buffers(&mut self) {
        self.backend.shrink_buffers()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        self.backend.flush()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        self.backend.should_flush()
    }
}
