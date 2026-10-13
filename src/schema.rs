use sqlx::SqlitePool;
use xcss::schema_identity::SchemaIdentity;

pub fn current_identity() -> anyhow::Result<SchemaIdentity> {
    Ok(SchemaIdentity::new(
        "xocs",
        "xocs-db-v5",
        5,
        env!("XOCS_SCHEMA_SHA256"),
    )?)
}

pub async fn initialize_identity(pool: &SqlitePool) -> anyhow::Result<()> {
    let identity = current_identity()?;
    sqlx::query("INSERT INTO product_metadata(singleton,application,application_version,schema_revision,schema_sha256) VALUES(1,?,?,?,?)")
        .bind(&identity.application).bind(&identity.application_version)
        .bind(i64::try_from(identity.schema_revision)?).bind(&identity.schema_sha256)
        .execute(pool).await?;
    Ok(())
}

/// Check the declared identity and every current SQL object in one read transaction.
/// The release version is deliberately excluded from the data identity.
pub async fn validate(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut transaction = pool.begin().await?;
    let result = xcss::sqlite::require_current_schema(&mut transaction, &current_identity()?).await;
    let rolled_back = transaction.rollback().await;
    match result {
        Ok(_) => {}
        // Keep the CLI's existing public contract rejection for exact schema
        // identity failures instead of hiding it behind the SQLx adapter.
        Err(xcss::sqlite::Error::SchemaIdentity(error)) => return Err(error.into()),
        Err(error) => return Err(error.into()),
    }
    rolled_back?;
    Ok(())
}
