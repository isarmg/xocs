use crate::query_engine::database::Database;
use crate::query_engine::error::Error;
use crate::query_engine::pool::{MaybePoolConnection, Pool, PoolConnection};

use crate::query_engine::transaction::Transaction;
use futures::future::BoxFuture;
use std::ops::{Deref, DerefMut};

/// Acquire a connection or begin a transaction through one interface.
///
/// Pool references and mutable SQLite connection references implement this trait.
/// An acquired connection dereferences to the concrete connection; use
/// `&mut *connection` when passing it to a query executor.
///
/// ```
/// use blog_storage_sqlite_access::{Acquire, Sqlite};
///
/// async fn read_value<'a, A>(source: A) -> blog_storage_sqlite_access::Result<i64>
/// where
///     A: Acquire<'a, Database = Sqlite>,
/// {
///     let mut connection = source.acquire().await?;
///     blog_storage_sqlite_access::query_scalar("SELECT 1")
///         .fetch_one(&mut *connection)
///         .await
/// }
/// ```
///
/// A function that only needs an already-acquired connection can instead accept
/// `&mut SqliteConnection`. Keep the connection borrow short, and begin a
/// transaction explicitly when several writes must be atomic.
pub trait Acquire<'c> {
    type Database: Database;

    type Connection: Deref<Target = <Self::Database as Database>::Connection> + DerefMut + Send;

    fn acquire(self) -> BoxFuture<'c, Result<Self::Connection, Error>>;

    fn begin(self) -> BoxFuture<'c, Result<Transaction<'c, Self::Database>, Error>>;
}

impl<'a, DB: Database> Acquire<'a> for &'_ Pool<DB> {
    type Database = DB;

    type Connection = PoolConnection<DB>;

    fn acquire(self) -> BoxFuture<'static, Result<Self::Connection, Error>> {
        Box::pin(self.acquire())
    }

    fn begin(self) -> BoxFuture<'static, Result<Transaction<'a, DB>, Error>> {
        let conn = self.acquire();

        Box::pin(async move {
            Transaction::begin(MaybePoolConnection::PoolConnection(conn.await?), None).await
        })
    }
}

#[macro_export]
macro_rules! impl_acquire {
    ($DB:ident, $C:ident) => {
        impl<'c> $crate::acquire::Acquire<'c> for &'c mut $C {
            type Database = $DB;

            type Connection = &'c mut <$DB as $crate::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) -> futures::future::BoxFuture<'c, Result<Self::Connection, $crate::error::Error>>
            {
                Box::pin(std::future::ready(Ok(self)))
            }

            #[inline]
            fn begin(
                self,
            ) -> futures::future::BoxFuture<
                'c,
                Result<$crate::transaction::Transaction<'c, $DB>, $crate::error::Error>,
            > {
                $crate::transaction::Transaction::begin(self, None)
            }
        }
    };
}
