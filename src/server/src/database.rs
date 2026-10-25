use crate::Error;
#[cfg(any(test, feature = "maintenance"))]
use crate::now_micros;
use crate::schema;
#[cfg(any(test, feature = "maintenance"))]
use blog_application::admin_core::AdministratorService;
use blog_application::admin_sqlite::SqliteAdministratorStore;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    time::Duration,
};

// Articles permit 2,000,000 bytes of content. Leave room for the rest of the
// encoded row while bounding each native SQLite value and statement.
const CONNECTION_LIMITS: blog_application::sqlite::ConnectionLimits =
    blog_application::sqlite::ConnectionLimits::new(4 * 1024 * 1024)
        .with_max_sql_bytes(256 * 1024)
        .with_max_vm_operations(100_000);

pub(crate) async fn open_database(path: &Path, create: bool) -> Result<SqlitePool, Error> {
    open_database_with_limit(path, create, 8).await
}

pub(crate) async fn open_database_with_limit(
    path: &Path,
    create: bool,
    max_connections: u32,
) -> Result<SqlitePool, Error> {
    let parent = path
        .parent()
        .expect("invariant: database has a parent directory");
    let directory = blog_application::state_file::PrivateStateDirectory::open(parent)?;
    if create {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
    }
    let descriptor = directory.open_existing(
        path.file_name()
            .expect("invariant: database has a filename"),
    )?;
    descriptor.verify_identity()?;
    let before = descriptor.file().metadata()?;
    // Closing a raw descriptor after SQLite opens the inode releases POSIX locks.
    drop(descriptor);
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
                blog_application::sqlite::apply_connection_limits(connection, CONNECTION_LIMITS)
                    .await
                    .map(|_| ())
                    .map_err(|error| sqlx::Error::Configuration(Box::new(error)))
            })
        })
        .connect_with(options)
        .await?;
    use std::os::unix::fs::MetadataExt;
    let verified = (|| {
        let after = fs::symlink_metadata(path)?;
        if (before.dev(), before.ino()) != (after.dev(), after.ino()) {
            return Err(Error::DatabaseIdentityChanged);
        }
        directory.verify_identity()?;
        Ok::<(), Error>(())
    })();
    if let Err(error) = verified {
        pool.close().await;
        return Err(error);
    }
    Ok(pool)
}

#[cfg(any(test, feature = "maintenance"))]
pub(crate) async fn initialize_new_database(
    database: &Path,
    username: &str,
    password: &blog_application::admin_auth::Password,
) -> Result<(), Error> {
    let root = database
        .parent()
        .expect("invariant: database has a parent directory");
    let directory = blog_application::fs_safety::PrivateDirectory::open_existing(root)?;
    let temporary = tempfile::Builder::new()
        .prefix(".blog-init-")
        .suffix(".sqlite")
        .tempfile_in(root)?;
    let staging = temporary.path().to_path_buf();
    let pool = open_database_with_limit(&staging, false, 1).await?;
    let result = async {
        initialize(&pool, username, password).await?;
        let (busy, _, _): (i64, i64, i64) = sqlx::query_as("PRAGMA wal_checkpoint(TRUNCATE)")
            .fetch_one(&pool)
            .await?;
        if busy != 0 {
            return Err(Error::InitialCheckpointBusy);
        }
        let mode: String = sqlx::query_scalar("PRAGMA journal_mode=DELETE")
            .fetch_one(&pool)
            .await?;
        if !mode.eq_ignore_ascii_case("delete") {
            return Err(Error::InitialWalMode);
        }
        Ok::<(), Error>(())
    }
    .await;
    pool.close().await;
    result?;
    temporary.as_file().sync_all()?;
    let (file, staging) = temporary.keep()?;
    let source = blog_application::fs_safety::RelativePath::new(
        staging
            .file_name()
            .expect("invariant: tempfile has a filename"),
    )?;
    let destination = blog_application::fs_safety::RelativePath::new(
        database
            .file_name()
            .expect("invariant: database has a filename"),
    )?;
    blog_application::fs_safety::NoClobberPublish::publish(&directory, &source, &destination)
        .map_err(|source| Error::PublishDatabase { staging, source })?;
    file.sync_all()?;
    Ok(())
}

#[cfg(any(test, feature = "maintenance"))]
pub(crate) async fn initialize(
    pool: &SqlitePool,
    username: &str,
    password: &blog_application::admin_auth::Password,
) -> Result<(), Error> {
    sqlx::raw_sql(include_str!(concat!(
        env!("OUT_DIR"),
        "/blog-current-schema.sql"
    )))
    .execute(pool)
    .await?;
    schema::initialize_identity(pool).await?;
    sqlx::query(include_str!("../schema/initialize-site-author.sql"))
        .execute(pool)
        .await?;
    let service = AdministratorService::new(SqliteAdministratorStore::new(pool.clone()));
    crate::administrator::seed_administrator(&service, username, password, now_micros()?).await?;
    validate_database(pool).await
}

pub(crate) async fn validate_database(pool: &SqlitePool) -> Result<(), Error> {
    schema::validate(pool).await?;
    let violations: Vec<(String, Option<i64>, String, i64)> =
        sqlx::query_as("PRAGMA foreign_key_check")
            .fetch_all(pool)
            .await?;
    if !violations.is_empty() {
        return Err(Error::ForeignKeyViolations);
    }
    let duplicate_family: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM family WHERE user_id IS NOT NULL GROUP BY user_id HAVING COUNT(*) > 1)",
    )
    .fetch_one(pool)
    .await?;
    if duplicate_family {
        return Err(Error::DuplicateFamily);
    }
    let authors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user WHERE user_type=0 AND deleted=0 AND user_status=1",
    )
    .fetch_one(pool)
    .await?;
    if authors == 0 {
        return Err(Error::MissingAuthor);
    }
    Ok(())
}

pub(crate) async fn validate_existing_database(
    path: &Path,
    temporary_root: &Path,
) -> Result<(), Error> {
    let snapshot = blog_application::sqlite::ValidationSnapshot::capture_in(path, temporary_root)?
        .into_pool_with_connection_limits(CONNECTION_LIMITS)
        .await?;
    let result = async {
        validate_database(snapshot.pool()).await?;
        let store = SqliteAdministratorStore::new(snapshot.pool().clone());
        use blog_application::admin_core::AdministratorStore as _;
        if store.administrator_count().await? == 0 {
            return Err(Error::NotInitialized);
        }
        store.validate_all_administrators().await?;
        let integrity: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_all(snapshot.pool())
            .await?;
        if integrity != ["ok"] {
            return Err(Error::IntegrityCheckFailed);
        }
        Ok::<(), Error>(())
    }
    .await;
    snapshot.close().await;
    result
}
