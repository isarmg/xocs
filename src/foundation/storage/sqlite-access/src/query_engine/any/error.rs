use std::any::type_name;

use crate::query_engine::any::type_info::AnyTypeInfo;
use crate::query_engine::any::Any;
use crate::query_engine::error::BoxDynError;
use crate::query_engine::type_info::TypeInfo;
use crate::query_engine::types::Type;

pub(super) fn mismatched_types<T: Type<Any>>(ty: &AnyTypeInfo) -> BoxDynError {
    format!(
        "mismatched types; Rust type `{}` is not compatible with SQL type `{}`",
        type_name::<T>(),
        ty.name()
    )
    .into()
}
