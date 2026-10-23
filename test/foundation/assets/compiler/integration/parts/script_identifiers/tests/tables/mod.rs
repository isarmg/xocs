#![allow(clippy::module_inception)]

#[path = "tables.rs"]
#[allow(clippy::redundant_static_lifetimes)]
#[rustfmt::skip]
mod tables;

pub(crate) use self::tables::*;
