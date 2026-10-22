use either::Either;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::TryStreamExt;

use crate::query_engine::database::Database;
use crate::query_engine::error::Error;
use crate::query_engine::executor::{Execute, Executor};
use crate::query_engine::pool::Pool;
use crate::query_engine::sql_str::SqlStr;

impl<'p, DB: Database> Executor<'p> for &'_ Pool<DB>
where
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
{
    type Database = DB;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<DB::QueryResult, DB::Row>, Error>>
    where
        E: 'q + Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        Box::pin(try_stream! {
            let mut conn = pool.acquire().await?;
            let mut s = conn.fetch_many(query);

            while let Some(v) = s.try_next().await? {
                r#yield!(v);
            }

            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<DB::Row>, Error>>
    where
        E: 'q + Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        Box::pin(async move { pool.acquire().await?.fetch_optional(query).await })
    }

    fn prepare_with<'e>(
        self,
        sql: SqlStr,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<'e, Result<<Self::Database as Database>::Statement, Error>>
    where
        'p: 'e,
    {
        let pool = self.clone();

        Box::pin(async move { pool.acquire().await?.prepare_with(sql, parameters).await })
    }

    #[doc(hidden)]
    #[cfg(feature = "offline")]
    fn describe<'e>(
        self,
        sql: SqlStr,
    ) -> BoxFuture<'e, Result<crate::query_engine::describe::Describe<Self::Database>, Error>> {
        let pool = self.clone();

        Box::pin(async move { pool.acquire().await?.describe(sql).await })
    }
}

// Causes an overflow when evaluating `&mut DB::Connection: Executor`.
//
//
// impl<'c, DB: Database> crate::query_engine::executor::Executor<'c> for &'c mut crate::query_engine::pool::PoolConnection<DB>
// where
//     &'c mut DB::Connection: Executor<'c, Database = DB>,
// {
//     type Database = DB;
//
//
//
//     #[inline]
//     fn fetch_many<'e, 'q: 'e, E: 'q>(
//         self,
//         query: E,
//     ) -> futures::stream::BoxStream<
//         'e,
//         Result<
//             either::Either<<DB as crate::query_engine::database::Database>::QueryResult, DB::Row>,
//             crate::query_engine::error::Error,
//         >,
//     >
//     where
//         'c: 'e,
//         E: crate::query_engine::executor::Execute<'q, DB>,
//     {
//         (**self).fetch_many(query)
//     }
//
//     #[inline]
//     fn fetch_optional<'e, 'q: 'e, E: 'q>(
//         self,
//         query: E,
//     ) -> futures::future::BoxFuture<'e, Result<Option<DB::Row>, crate::query_engine::error::Error>>
//     where
//         'c: 'e,
//         E: crate::query_engine::executor::Execute<'q, DB>,
//     {
//         (**self).fetch_optional(query)
//     }
//
//     #[inline]
//     fn prepare_with<'e, 'q: 'e>(
//         self,
//         sql: &'q str,
//         parameters: &'e [<DB as crate::query_engine::database::Database>::TypeInfo],
//     ) -> futures::future::BoxFuture<
//         'e,
//         Result<<DB as crate::query_engine::database::Database>::Statement<'q>, crate::query_engine::error::Error>,
//     >
//     where
//         'c: 'e,
//     {
//         (**self).prepare_with(sql, parameters)
//     }
//
//     #[doc(hidden)]
//     #[cfg(feature = "offline")]
//     #[inline]
//     fn describe<'e, 'q: 'e>(
//         self,
//         sql: &'q str,
//     ) -> futures::future::BoxFuture<
//         'e,
//         Result<crate::query_engine::describe::Describe<DB>, crate::query_engine::error::Error>,
//     >
//     where
//         'c: 'e,
//     {
//         (**self).describe(sql)
//     }
// }
