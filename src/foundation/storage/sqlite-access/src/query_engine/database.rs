//! Associated types shared by SQLite query, row and connection abstractions.

use std::fmt::Debug;

use crate::query_engine::arguments::Arguments;
use crate::query_engine::column::Column;
use crate::query_engine::connection::Connection;
use crate::query_engine::row::Row;

use crate::query_engine::statement::Statement;
use crate::query_engine::transaction::TransactionManager;
use crate::query_engine::type_info::TypeInfo;
use crate::query_engine::value::{Value, ValueRef};

/// The associated types used by the maintained SQLite query interfaces.
pub trait Database: 'static + Sized + Send + Debug {
    /// The concrete `Connection` implementation for this database.
    type Connection: Connection<Database = Self>;

    /// The concrete `TransactionManager` implementation for this database.
    type TransactionManager: TransactionManager<Database = Self>;

    /// The concrete `Row` implementation for this database.
    type Row: Row<Database = Self>;

    /// The concrete `QueryResult` implementation for this database.
    type QueryResult: 'static + Sized + Send + Sync + Default + Extend<Self::QueryResult>;

    /// The concrete `Column` implementation for this database.
    type Column: Column<Database = Self>;

    /// The concrete `TypeInfo` implementation for this database.
    type TypeInfo: TypeInfo;

    /// The concrete type used to hold an owned copy of the not-yet-decoded value that was
    /// received from the database.
    type Value: Value<Database = Self> + 'static;
    /// The concrete type used to hold a reference to the not-yet-decoded value that has just been
    /// received from the database.
    type ValueRef<'r>: ValueRef<'r, Database = Self>;

    /// The concrete `Arguments` implementation for this database.
    type Arguments: Arguments<Database = Self>;
    /// The concrete type used as a buffer for arguments while encoding.
    type ArgumentBuffer;

    /// The concrete `Statement` implementation for this database.
    type Statement: Statement<Database = Self>;

    /// The display name for this database driver.
    const NAME: &'static str;
}

/// A [`Database`] that maintains a client-side cache of prepared statements.
pub trait HasStatementCache {}
