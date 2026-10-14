//! Product-owned current schema identity used by runtime, build and test fixtures.
pub fn identity(
    sha256: &str,
) -> Result<xcss::schema_identity::SchemaIdentity, xcss::schema_identity::Error> {
    xcss::schema_identity::SchemaIdentity::new("xocs", "xocs-db-v1", 1, sha256)
}
