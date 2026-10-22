use crate::query_engine::any::{Any, AnyConnection, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo};
use crate::query_engine::error::Error;
use crate::query_engine::executor::{Execute, Executor};
use crate::query_engine::sql_str::SqlStr;
use either::Either;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{stream, FutureExt, StreamExt};
use std::future;

impl<'c> Executor<'c> for &'c mut AnyConnection {
    type Database = Any;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<AnyQueryResult, AnyRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Any>,
    {
        let arguments = match query.take_arguments().map_err(Error::Encode) {
            Ok(arguments) => arguments,
            Err(error) => return stream::once(future::ready(Err(error))).boxed(),
        };
        let persistent = query.persistent();
        self.backend.fetch_many(query.sql(), persistent, arguments)
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<AnyRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        let arguments = match query.take_arguments().map_err(Error::Encode) {
            Ok(arguments) => arguments,
            Err(error) => return future::ready(Err(error)).boxed(),
        };
        let persistent = query.persistent();
        self.backend
            .fetch_optional(query.sql(), persistent, arguments)
    }

    fn prepare_with<'e>(
        self,
        sql: SqlStr,
        parameters: &[AnyTypeInfo],
    ) -> BoxFuture<'e, Result<AnyStatement, Error>>
    where
        'c: 'e,
    {
        self.backend.prepare_with(sql, parameters)
    }

    #[cfg(feature = "offline")]
    fn describe<'e>(
        self,
        sql: SqlStr,
    ) -> BoxFuture<'e, Result<crate::query_engine::describe::Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        self.backend.describe(sql)
    }
}
