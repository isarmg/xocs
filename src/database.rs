#[cfg(test)]
use crate::now_micros;
use crate::schema;
use anyhow::{Context, bail};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::Duration,
};
#[cfg(test)]
use xcss::admin_core::AdministratorService;
use xcss::admin_sqlite::SqliteAdministratorStore;

// Articles permit 2,000,000 bytes of content. Leave room for the rest of the
// encoded row while bounding each native SQLite value and statement.
const CONNECTION_LIMITS: xcss::sqlite::ConnectionLimits =
    xcss::sqlite::ConnectionLimits::new(4 * 1024 * 1024)
        .with_max_sql_bytes(256 * 1024)
        .with_max_vm_operations(100_000);

pub(crate) async fn open_database(path: &Path, create: bool) -> anyhow::Result<SqlitePool> {
    open_database_with_limit(path, create, 8).await
}

pub(crate) async fn open_database_with_limit(
    path: &Path,
    create: bool,
    max_connections: u32,
) -> anyhow::Result<SqlitePool> {
    if !path.is_absolute() {
        bail!("database path must be absolute");
    }
    let parent = path
        .parent()
        .context("database path has no parent directory")?;
    let parent_metadata =
        fs::symlink_metadata(parent).context("database parent directory is missing")?;
    if !parent_metadata.is_dir()
        || parent_metadata.file_type().is_symlink()
        || parent_metadata.permissions().mode() & 0o077 != 0
    {
        bail!("database parent directory must be a private 0700 directory");
    }
    if create {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .context("create private SQLite database")?;
    } else {
        let metadata = fs::symlink_metadata(path).context("database does not exist")?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.permissions().mode() & 0o077 != 0
        {
            bail!("database must be a regular private 0600 file");
        }
    }
    let directory = xcss::state_file::PrivateStateDirectory::open(parent)?;
    let descriptor =
        directory.open_existing(path.file_name().context("database name is required")?)?;
    descriptor.verify_identity()?;
    // Closing any raw descriptor for a SQLite inode after SQLite opens it can
    // release that process's POSIX locks. Complete descriptor checks first.
    drop(descriptor);
    let before = fs::symlink_metadata(path)?;
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|connection, _| {
            Box::pin(async move {
                xcss::sqlite::apply_connection_limits(connection, CONNECTION_LIMITS)
                    .await
                    .map(|_| ())
                    .map_err(|error| sqlx::Error::Configuration(Box::new(error)))
            })
        })
        .connect_with(options)
        .await
        .context("open SQLite database")?;
    use std::os::unix::fs::MetadataExt;
    let verified = (|| {
        let after = fs::symlink_metadata(path)?;
        anyhow::ensure!(
            (before.dev(), before.ino()) == (after.dev(), after.ino()),
            "database identity changed while opening"
        );
        directory.verify_identity()?;
        Ok::<(), anyhow::Error>(())
    })();
    if let Err(error) = verified {
        pool.close().await;
        return Err(error);
    }
    Ok(pool)
}

#[cfg(test)]
pub(crate) async fn initialize_new_database(
    database: &Path,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let root = database.parent().context("database parent is required")?;
    let directory = xcss::fs_safety::PrivateDirectory::open_existing(root)?;
    let temporary = tempfile::Builder::new()
        .prefix(".xocs-init-")
        .suffix(".sqlite")
        .tempfile_in(root)?;
    let staging = temporary.path().to_path_buf();
    let pool = open_database_with_limit(&staging, false, 1).await?;
    let result = async {
        initialize(&pool, username, password).await?;
        let (busy, _, _): (i64, i64, i64) = sqlx::query_as("PRAGMA wal_checkpoint(TRUNCATE)")
            .fetch_one(&pool)
            .await?;
        anyhow::ensure!(busy == 0, "initial database checkpoint did not complete");
        let mode: String = sqlx::query_scalar("PRAGMA journal_mode=DELETE")
            .fetch_one(&pool)
            .await?;
        anyhow::ensure!(
            mode.eq_ignore_ascii_case("delete"),
            "initial database could not leave staging WAL mode"
        );
        Ok::<(), anyhow::Error>(())
    }
    .await;
    pool.close().await;
    result?;
    temporary.as_file().sync_all()?;
    let (file, staging) = temporary
        .keep()
        .context("retain completed initialization staging file")?;
    let source =
        xcss::fs_safety::RelativePath::new(staging.file_name().context("staging filename")?)?;
    let destination =
        xcss::fs_safety::RelativePath::new(database.file_name().context("database filename")?)?;
    xcss::fs_safety::NoClobberPublish::publish(&directory, &source, &destination)
        .context("publish initialized database without replacing an existing file; completed staging is retained if publication fails")?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
pub(crate) async fn initialize(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    sqlx::raw_sql(include_str!(concat!(
        env!("OUT_DIR"),
        "/xocs-current-schema.sql"
    )))
    .execute(pool)
    .await?;
    schema::initialize_identity(pool).await?;
    sqlx::query(include_str!("../schema/initialize-site-author.sql"))
        .execute(pool)
        .await?;
    let service = AdministratorService::new(SqliteAdministratorStore::new(pool.clone()));
    crate::test_support::seed_administrator(&service, username, password, now_micros()?).await?;
    validate_database(pool).await
}

pub(crate) async fn validate_database(pool: &SqlitePool) -> anyhow::Result<()> {
    schema::validate(pool).await?;
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_schema WHERE type='table'")
            .fetch_all(pool)
            .await?;
    for table in [
        "user",
        "article",
        "comment",
        "comment_submission",
        "sort",
        "label",
        "resource",
        "_common_administrators",
        "_common_admin_sessions",
    ] {
        if !names.iter().any(|name| name == table) {
            bail!("missing required table: {table}");
        }
    }
    let violations: Vec<(String, Option<i64>, String, i64)> =
        sqlx::query_as("PRAGMA foreign_key_check")
            .fetch_all(pool)
            .await?;
    if !violations.is_empty() {
        bail!("database contains foreign-key violations");
    }
    let duplicate_family: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM family WHERE user_id IS NOT NULL GROUP BY user_id HAVING COUNT(*) > 1)",
    )
    .fetch_one(pool)
    .await?;
    anyhow::ensure!(
        !duplicate_family,
        "an author must have at most one family record"
    );
    let authors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user WHERE user_type=0 AND deleted=0 AND user_status=1",
    )
    .fetch_one(pool)
    .await?;
    if authors == 0 {
        bail!("database has no active site author");
    }
    Ok(())
}

pub(crate) async fn validate_existing_database(
    path: &PathBuf,
    temporary_root: &Path,
) -> anyhow::Result<()> {
    let snapshot = xcss::sqlite::ValidationSnapshot::capture_in(path, temporary_root)?
        .into_pool_with_connection_limits(CONNECTION_LIMITS)
        .await?;
    let result = async {
        validate_database(snapshot.pool()).await?;
        let store = SqliteAdministratorStore::new(snapshot.pool().clone());
        use xcss::admin_core::AdministratorStore as _;
        anyhow::ensure!(
            store.administrator_count().await? > 0,
            "explicit initialization is required"
        );
        store.validate_all_administrators().await?;
        let integrity: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_all(snapshot.pool())
            .await?;
        anyhow::ensure!(integrity == ["ok"], "database integrity check failed");
        Ok::<(), anyhow::Error>(())
    }
    .await;
    snapshot.close().await;
    result
}
