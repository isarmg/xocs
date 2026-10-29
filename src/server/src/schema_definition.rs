//! Product-owned current schema identity used by runtime, build and test fixtures.
pub fn identity(
    sha256: &str,
) -> Result<
    blog_application::schema_identity::SchemaIdentity,
    blog_application::schema_identity::Error,
> {
    blog_application::schema_identity::SchemaIdentity::new("blog", "blog-db-v5", 5, sha256)
}
