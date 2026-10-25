use crate::Error;
use blog_application::schema_identity::SchemaIdentity;
use sqlx::SqlitePool;

pub fn current_identity() -> SchemaIdentity {
    crate::schema_definition::identity(env!("BLOG_SCHEMA_SHA256"))
        .expect("invariant: compiled schema identity is valid")
}

#[cfg(any(test, feature = "maintenance"))]
pub async fn initialize_identity(pool: &SqlitePool) -> Result<(), Error> {
    let identity = current_identity();
    sqlx::query("INSERT INTO product_metadata(singleton,application,application_version,schema_revision,schema_sha256) VALUES(1,?,?,?,?)")
        .bind(&identity.application).bind(&identity.application_version)
        .bind(identity.schema_revision as i64).bind(&identity.schema_sha256)
        .execute(pool).await?;
    Ok(())
}

/// Check the declared identity and every current SQL object in one read transaction.
/// The release version is deliberately excluded from the data identity.
pub async fn validate(pool: &SqlitePool) -> Result<(), Error> {
    let mut transaction = pool.begin().await?;
    let result =
        blog_application::sqlite::require_current_schema(&mut transaction, &current_identity())
            .await;
    let rolled_back = transaction.rollback().await;
    match (result, rolled_back) {
        (Ok(_), Ok(())) => Ok(()),
        (Err(validation), Ok(())) => Err(validation.into()),
        (Ok(_), Err(rollback)) => Err(rollback.into()),
        (Err(validation), Err(rollback)) => Err(Error::SchemaValidationRollback {
            validation: Box::new(validation),
            rollback,
        }),
    }
}
