//! Current test data fixtures, excluded from the Server and release package.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[path = "administrator.rs"]
mod administrator;
#[path = "../../src/schema_definition.rs"]
mod schema_definition;

pub async fn initialize(root: &Path, username: &str, password: &str) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(root.is_absolute(), "fixture ROOT must be absolute");
    for directory in ["state", "media", "run", "logs"] {
        fs::create_dir(root.join(directory))?;
        fs::set_permissions(root.join(directory), fs::Permissions::from_mode(0o700))?;
    }
    let database = root.join("state/site.sqlite");
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&database)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Delete);
    let pool = sqlx::SqlitePool::connect_with(options).await?;
    fs::set_permissions(&database, fs::Permissions::from_mode(0o600))?;
    sqlx::raw_sql(include_str!(concat!(
        env!("OUT_DIR"),
        "/blog-current-schema.sql"
    )))
    .execute(&pool)
    .await?;
    let identity = schema_definition::identity(env!("BLOG_SCHEMA_SHA256"))?;
    sqlx::query(blog_application::schema_identity::PRODUCT_METADATA_INSERT_SQL)
        .bind(identity.application)
        .bind(identity.application_version)
        .bind(i64::try_from(identity.schema_revision)?)
        .bind(identity.schema_sha256)
        .execute(&pool)
        .await?;
    sqlx::query(include_str!("../../schema/initialize-site-author.sql"))
        .execute(&pool)
        .await?;
    let service = blog_application::admin_core::AdministratorService::new(
        blog_application::admin_sqlite::SqliteAdministratorStore::new(pool.clone()),
    );
    let now = u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_micros(),
    )?;
    administrator::seed_administrator(&service, username, password, now).await?;
    pool.close().await;
    check(root).await?;
    Ok(database)
}

pub async fn check(root: &Path) -> anyhow::Result<()> {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(root.join("state/site.sqlite"))
        .read_only(true)
        .create_if_missing(false);
    let pool = sqlx::SqlitePool::connect_with(options).await?;
    let mut transaction = pool.begin().await?;
    let result = blog_application::sqlite::require_current_schema(
        &mut transaction,
        &schema_definition::identity(env!("BLOG_SCHEMA_SHA256"))?,
    )
    .await;
    transaction.rollback().await?;
    pool.close().await;
    result.map(|_| ()).map_err(anyhow::Error::from)
}
