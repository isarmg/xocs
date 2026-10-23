//! Conversions between Rust and SQL types.
//!
//! The concrete type mapping is documented in [`crate::sqlite::types`].
//!
//! Any external types that have had [`Type`] implemented for, are re-exported in this module
//! for convenience as downstream users need to use a compatible version of the external crate
//! to take advantage of the implementation.
//!
//! # Nullable
//!
//! To represent nullable SQL types, `Option<T>` is supported where `T` implements `Type`.
//! An `Option<T>` represents a potentially `NULL` value from SQL.

use std::{borrow::Cow, rc::Rc, sync::Arc};

use crate::query_engine::database::Database;
use crate::query_engine::type_info::TypeInfo;

mod non_zero;

#[cfg(feature = "json")]
mod json;
mod text;
#[cfg(feature = "json")]
pub use json::{Json, JsonRawValue, JsonValue};
pub use text::Text;

#[cfg(feature = "chrono")]
pub mod chrono {
    pub use chrono::{
        DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
    };
}

/// Indicates that a SQL type is supported for a database.
///
/// Type metadata is checked at runtime when decoding a value. To define a Rust
/// wrapper type, implement this trait together with
/// [`Encode`](crate::Encode) and [`Decode`](crate::Decode), delegating to the
/// underlying SQLite representation where appropriate.
///
/// The current derive support maps rows through [`crate::FromRow`]. It does not
/// provide compile-time SQL analysis or a derive for this type trait.
pub trait Type<DB: Database> {
    /// Returns the canonical SQL type for this Rust type.
    ///
    /// When binding arguments, this is used to tell the database what is about to be sent; which,
    /// the database then uses to guide query plans. This can be overridden by `Encode::produces`.
    ///
    fn type_info() -> DB::TypeInfo;

    /// Determines if this Rust type is compatible with the given SQL type.
    ///
    /// When decoding values from a row, this method is checked to determine if we should continue
    /// or raise a runtime type mismatch error.
    ///
    /// Defaults to checking [`TypeInfo::type_compatible()`].
    fn compatible(ty: &DB::TypeInfo) -> bool {
        Self::type_info().type_compatible(ty)
    }
}

// for references, the underlying SQL type is identical
impl<T: ?Sized + Type<DB>, DB: Database> Type<DB> for &'_ T {
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <T as Type<DB>>::compatible(ty)
    }
}

// for optionals, the underlying SQL type is identical
impl<T: Type<DB>, DB: Database> Type<DB> for Option<T> {
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        ty.is_null() || <T as Type<DB>>::compatible(ty)
    }
}

macro_rules! impl_type_for_smartpointer {
    ($smart_pointer:ty) => {
        impl<T, DB: Database> Type<DB> for $smart_pointer
        where
            T: Type<DB> + ?Sized,
        {
            fn type_info() -> DB::TypeInfo {
                <T as Type<DB>>::type_info()
            }

            fn compatible(ty: &DB::TypeInfo) -> bool {
                <T as Type<DB>>::compatible(ty)
            }
        }
    };
}

impl_type_for_smartpointer!(Arc<T>);
impl_type_for_smartpointer!(Box<T>);
impl_type_for_smartpointer!(Rc<T>);

impl<T, DB: Database> Type<DB> for Cow<'_, T>
where
    // `ToOwned` is required here to satisfy `Cow`
    T: Type<DB> + ToOwned + ?Sized,
{
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <T as Type<DB>>::compatible(ty)
    }
}
