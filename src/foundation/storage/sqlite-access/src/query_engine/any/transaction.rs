use std::future::Future;

use crate::query_engine::any::{Any, AnyConnection};
use crate::query_engine::database::Database;
use crate::query_engine::error::Error;
use crate::query_engine::sql_str::SqlStr;
use crate::query_engine::transaction::TransactionManager;

pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;

    fn begin(
        conn: &mut AnyConnection,
        statement: Option<SqlStr>,
    ) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.backend.begin(statement)
    }

    fn commit(conn: &mut AnyConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.backend.commit()
    }

    fn rollback(conn: &mut AnyConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.backend.rollback()
    }

    fn start_rollback(conn: &mut AnyConnection) {
        conn.backend.start_rollback()
    }

    fn get_transaction_depth(conn: &<Self::Database as Database>::Connection) -> usize {
        conn.backend.get_transaction_depth()
    }
}
