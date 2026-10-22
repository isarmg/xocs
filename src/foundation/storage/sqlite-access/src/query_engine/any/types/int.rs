use crate::query_engine::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::query_engine::database::Database;
use crate::query_engine::decode::Decode;
use crate::query_engine::encode::{Encode, IsNull};
use crate::query_engine::error::BoxDynError;
use crate::query_engine::types::Type;

impl Type<Any> for i16 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::SmallInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for i16 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::SmallInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for i16 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for i32 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Integer,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for i32 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::Integer(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for i32 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for i64 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::BigInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for i64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::BigInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for i64 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}
