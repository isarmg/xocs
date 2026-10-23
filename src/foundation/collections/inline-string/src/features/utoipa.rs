use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "utoipa")))]
impl utoipa::PartialSchema for CompactString {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::new(
                utoipa::openapi::schema::Type::String,
            ))
            .into()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "utoipa")))]
impl utoipa::ToSchema for CompactString {}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/utoipa/tests.rs"]
#[cfg(test)]
mod tests;
