use crate::query_engine::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::query_engine::database::Database;
use crate::query_engine::decode::Decode;
use crate::query_engine::encode::{Encode, IsNull};
use crate::query_engine::error::BoxDynError;
use crate::query_engine::types::Type;

impl Type<Any> for bool {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Bool,
        }
    }
}

impl Encode<'_, Any> for bool {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::Bool(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for bool {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Bool(b) => Ok(*b),
            other => other.unexpected(),
        }
    }
}
