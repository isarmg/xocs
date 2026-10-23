//! SQLite runtime for the blog: bundled native SQLite, async worker connections,
//! statement caching, transactions, pools and runtime row decoding.

// SQLite is a C library. All interactions require FFI which is unsafe.
// All unsafe blocks should have comments pointing to SQLite docs and ensuring that we maintain
// invariants.
#![allow(unsafe_code)]

pub use arguments::{SqliteArgumentValue, SqliteArguments, SqliteArgumentsBuffer};
pub use column::SqliteColumn;
pub use connection::{LockedSqliteHandle, SqliteConnection, SqliteOperation, UpdateHookResult};
pub use database::Sqlite;
pub use error::SqliteError;
pub use options::{
    SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode, SqliteSynchronous,
};
pub use query_result::SqliteQueryResult;
pub use row::SqliteRow;
pub use statement::SqliteStatement;
pub use transaction::SqliteTransactionManager;
pub use type_info::SqliteTypeInfo;
pub use value::{SqliteValue, SqliteValueRef};

pub(crate) use crate::query_engine::{common, decode, encode, ext, pool};

use crate::query_engine::executor::Executor;

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod logger;
mod options;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

/// An alias for [`Pool`][crate::sqlite::pool::Pool], specialized for SQLite.
pub type SqlitePool = crate::sqlite::pool::Pool<Sqlite>;

/// An alias for [`PoolOptions`][crate::sqlite::pool::PoolOptions], specialized for SQLite.
pub type SqlitePoolOptions = crate::sqlite::pool::PoolOptions<Sqlite>;

/// An alias for [`Executor<'_, Database = Sqlite>`][Executor].
pub trait SqliteExecutor<'c>: Executor<'c, Database = Sqlite> {}
impl<'c, T: Executor<'c, Database = Sqlite>> SqliteExecutor<'c> for T {}

/// An alias for [`Transaction`][crate::query_engine::transaction::Transaction], specialized for SQLite.
pub type SqliteTransaction<'c> = crate::query_engine::transaction::Transaction<'c, Sqlite>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(SqliteArguments);
impl_column_index_for_row!(SqliteRow);
impl_column_index_for_statement!(SqliteStatement);
impl_acquire!(Sqlite, SqliteConnection);

// required because some databases have a different handling of NULL
impl_encode_for_option!(Sqlite);
