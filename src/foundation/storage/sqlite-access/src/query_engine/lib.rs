//! Query execution, transactions and pooling for the maintained SQLite driver.

#![allow(clippy::needless_doctest_main, clippy::type_complexity)]
// Native SQLite FFI stays in the sqlite module; the query engine is safe Rust.
#![forbid(unsafe_code)]

#[macro_use]
pub mod ext;

#[macro_use]
pub mod error;

#[macro_use]
pub mod arguments;

#[macro_use]
pub mod pool;

pub mod connection;

#[macro_use]
pub mod transaction;

#[macro_use]
pub mod encode;

#[macro_use]
pub mod decode;

#[macro_use]
pub mod types;

#[macro_use]
pub mod query;

#[macro_use]
pub mod acquire;

#[macro_use]
pub mod column;

#[macro_use]
pub mod statement;

pub mod common;
pub mod database;
pub mod executor;
pub mod from_row;
pub mod logger;
pub mod query_as;
pub mod query_builder;
pub mod query_scalar;
pub mod sql_str;

pub mod raw_sql;
pub mod row;
pub mod rt;
pub mod sync;
pub mod type_info;
pub mod value;

pub use error::{Error, Result};

pub use bytes;
pub use either::Either;
pub use hashbrown::HashMap;
pub use indexmap::IndexMap;
pub use smallvec::SmallVec;
