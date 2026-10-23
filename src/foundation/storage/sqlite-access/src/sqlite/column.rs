use crate::sqlite::ext::ustr::UStr;
use crate::sqlite::{Sqlite, SqliteTypeInfo};

pub(crate) use crate::query_engine::column::*;

#[derive(Debug, Clone)]

pub struct SqliteColumn {
    pub(crate) name: UStr,
    pub(crate) ordinal: usize,
    pub(crate) type_info: SqliteTypeInfo,

    pub(crate) origin: ColumnOrigin,
}

impl Column for SqliteColumn {
    type Database = Sqlite;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_info(&self) -> &SqliteTypeInfo {
        &self.type_info
    }

    fn origin(&self) -> ColumnOrigin {
        self.origin.clone()
    }
}
