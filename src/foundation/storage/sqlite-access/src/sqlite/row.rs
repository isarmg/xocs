#![allow(clippy::rc_buffer)]

use std::sync::Arc;

use crate::query_engine::column::ColumnIndex;
use crate::query_engine::error::Error;
use crate::query_engine::ext::ustr::UStr;
use crate::query_engine::row::Row;
use crate::query_engine::HashMap;
use crate::query_engine::{column::Column, value::Value};

use crate::sqlite::statement::StatementHandle;
use crate::sqlite::{Sqlite, SqliteColumn, SqliteValue, SqliteValueRef};

/// Implementation of [`Row`] for SQLite.
pub struct SqliteRow {
    pub(crate) values: Box<[SqliteValue]>,
    pub(crate) columns: Arc<Vec<SqliteColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

// A row can move to another thread, but cannot be shared concurrently: decoding
// may mutate SQLite value storage and the Rust borrow cache in SqliteValue.
// sqlite3_value_dup() detaches the value from the connection (db = NULL in the
// bundled SQLite implementation); sqlite3_value_blob() can then write its flags
// without a connection mutex. "Protected" does not make this Rust cache Sync.

impl SqliteRow {
    pub(crate) fn current(
        statement: &StatementHandle,
        columns: &Arc<Vec<SqliteColumn>>,
        column_names: &Arc<HashMap<UStr, usize>>,
    ) -> Self {
        let size = statement.column_count();
        let mut values = Vec::with_capacity(size);

        for i in 0..size {
            values.push(unsafe {
                let raw = statement.column_value(i);

                SqliteValue::dup(raw, Some(columns[i].type_info.clone()))
            });
        }

        Self {
            values: values.into_boxed_slice(),
            columns: Arc::clone(columns),
            column_names: Arc::clone(column_names),
        }
    }
}

impl Row for SqliteRow {
    type Database = Sqlite;

    fn columns(&self) -> &[SqliteColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<SqliteValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        Ok(SqliteValueRef::value(&self.values[index]))
    }
}

impl ColumnIndex<SqliteRow> for &'_ str {
    fn index(&self, row: &SqliteRow) -> Result<usize, Error> {
        row.column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .copied()
    }
}

impl std::fmt::Debug for SqliteRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqliteRow ")?;
        let mut map = f.debug_map();
        for (column, value) in self.columns.iter().zip(&self.values) {
            map.entry(&column.name(), &DebugValue(value));
        }
        map.finish()
    }
}

struct DebugValue<'a>(&'a SqliteValue);

impl std::fmt::Debug for DebugValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::sqlite::type_info::DataType;

        // Formatting must not invalidate a previously decoded &str or &[u8].
        let value = match self.0.try_clone() {
            Ok(value) => value,
            Err(error) => return write!(f, "<decode error: {error}>"),
        };
        if value.is_null() {
            return f.write_str("NULL");
        }
        match value.type_info().0 {
            DataType::Bool => fmt_decoded(value.try_decode::<bool>(), f),
            DataType::Int4 | DataType::Integer => fmt_decoded(value.try_decode::<i64>(), f),
            DataType::Float => fmt_decoded(value.try_decode::<f64>(), f),
            DataType::Blob => fmt_decoded(value.try_decode::<&[u8]>(), f),
            _ => fmt_decoded(value.try_decode::<&str>(), f),
        }
    }
}

fn fmt_decoded<T: std::fmt::Debug>(
    value: Result<T, Error>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match value {
        Ok(value) => value.fmt(f),
        Err(error) => write!(f, "<decode error: {error}>"),
    }
}
