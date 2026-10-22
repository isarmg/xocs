#![cfg_attr(docsrs, feature(doc_cfg))]
//! Blog SQLite access: queries, pools, transactions, time codecs and record mapping.
#![recursion_limit = "512"]

#[macro_use]
#[path = "query_engine/lib.rs"]
mod query_engine;
#[doc(hidden)]
pub use query_engine::*;
#[path = "sqlite/lib.rs"]
pub mod sqlite;

pub use query_engine::Either;
pub use query_engine::acquire::Acquire;
pub use query_engine::arguments::{Arguments, IntoArguments};
pub use query_engine::column::Column;
pub use query_engine::column::ColumnIndex;
pub use query_engine::column::ColumnOrigin;
pub use query_engine::connection::{ConnectOptions, Connection};
pub use query_engine::database::{self, Database};
pub use query_engine::describe::Describe;
pub use query_engine::executor::{Execute, Executor};
pub use query_engine::from_row::FromRow;
pub use query_engine::pool::{self, Pool};
#[doc(hidden)]
pub use crate::query_engine::query::query_with_result as __query_with_result;
pub use crate::query_engine::query::{query, query_with};
pub use crate::query_engine::query_as::{query_as, query_as_with};
pub use query_engine::query_builder::{self, QueryBuilder};
#[doc(hidden)]
pub use crate::query_engine::query_scalar::query_scalar_with_result as __query_scalar_with_result;
pub use crate::query_engine::query_scalar::{query_scalar, query_scalar_with};
pub use query_engine::raw_sql::{RawSql, raw_sql};
pub use query_engine::row::Row;
pub use query_engine::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};
pub use query_engine::statement::Statement;
pub use query_engine::transaction::Transaction;
pub use query_engine::type_info::TypeInfo;
pub use query_engine::types::Type;
pub use query_engine::value::{Value, ValueRef};

#[doc(inline)]
pub use query_engine::error::{self, Error, Result};

#[cfg(feature = "_sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "_sqlite")))]
#[doc(inline)]
pub use sqlite::{ Sqlite, SqliteConnection, SqliteExecutor, SqlitePool, SqliteTransaction,
};

#[cfg(feature = "derive")]
#[doc(hidden)]
pub extern crate sqlx_macros;

// derives
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use sqlx_macros::FromRow;

#[doc(hidden)]
pub use query_engine::rt::test_block_on;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub mod spec_error;

#[doc(hidden)]
pub use query_engine::rt as __rt;

/// Conversions between Rust and SQL types.
///
/// To see how each SQL type maps to a Rust type, see the corresponding `types` module for each
/// database:
///
///  * SQLite: [sqlite::types]
///
/// Any external types that have had [`Type`] implemented for, are re-exported in this module
/// for convenience as downstream users need to use a compatible version of the external crate
/// to take advantage of the implementation.
///
/// [`Type`]: types::Type
pub mod types {
    pub use crate::query_engine::types::*;
}

/// Provides [`Encode`] for encoding values for the database.
pub mod encode {
    pub use crate::query_engine::encode::{Encode, IsNull};
}

pub use self::encode::Encode;

/// Provides [`Decode`] for decoding values from the database.
pub mod decode {
    pub use crate::query_engine::decode::Decode;
}

pub use self::decode::Decode;

/// Types and traits for the `query` family of functions and macros.
pub mod query {
    pub use crate::query_engine::query::*;
    pub use crate::query_engine::query_as::QueryAs;
    pub use crate::query_engine::query_scalar::QueryScalar;
}

/// Convenience re-export of common traits.
pub mod prelude {
    pub use super::Acquire;
    pub use super::ConnectOptions;
    pub use super::Connection;
    pub use super::Decode;
    pub use super::Encode;
    pub use super::Executor;
    pub use super::FromRow;
    pub use super::IntoArguments;
    pub use super::Row;
    pub use super::Statement;
    pub use super::Type;
}
