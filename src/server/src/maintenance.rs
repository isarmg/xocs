//! Explicit offline operations for the separate `blog-setup` executable.
//! Every operation takes the same instance lock as the server. Runtime startup
//! never calls these functions and never initializes or resets stored data.
use crate::Error;
use blog_application::{
    admin_auth,
    admin_core::{
        AdministratorStore, AuditOutcome, Identifier, SecurityAction, SecurityAuditEvent,
    },
    admin_sqlite::SqliteAdministratorStore,
    fs_safety::PrivateDirectory,
};
use std::path::Path;

/// Initialize a fresh current database without replacing any existing file.
/// The administrator controls ROOT/config/config.json. Mutable directories may
/// already exist, but must be private and owned by the invoking service user.
pub async fn initialize(root: &Path, username: &str, password: &str) -> Result<(), Error> {
    crate::verify_compiled_identity()?;
    let username = admin_auth::normalize_administrator_username(username)?;
    let password = admin_auth::Password::new(password)?;
    let settings = crate::config::load_offline(root, &root.join("config/config.json"))?.value;
    PrivateDirectory::create(&settings.data_dir)?;
    let lock = crate::config::runtime_lock(&settings)?;
    let result = async {
        let database = settings.database_path();
        PrivateDirectory::create(settings.media_path())?;
        PrivateDirectory::create(root.join("run"))?;
        PrivateDirectory::create(root.join("logs"))?;
        match crate::database::initialize_new_database(&database, &username, &password).await {
            Ok(()) => Ok(()),
            Err(Error::PublishDatabase {
                staging,
                mut source,
            }) => {
                // EEXIST from RENAME_NOREPLACE guarantees that this staging was not published.
                if matches!(
                    source,
                    blog_application::fs_safety::Error::DestinationExists(_)
                ) && let Err(cleanup) = std::fs::remove_file(&staging)
                {
                    source = blog_application::fs_safety::Error::Cleanup {
                        operation: Box::new(source),
                        cleanup,
                    };
                }
                Err(Error::PublishDatabase { staging, source })
            }
            Err(error) => Err(error),
        }
    }
    .await;
    finish_operation(result, lock.release())
}

/// Validate a stopped instance using a bounded temporary database snapshot.
pub async fn check(root: &Path) -> Result<(), Error> {
    crate::verify_compiled_identity()?;
    let settings = crate::config::load_offline(root, &root.join("config/config.json"))?.value;
    let lock = crate::config::runtime_lock(&settings)?;
    let result = async {
        PrivateDirectory::open_existing(settings.media_path())?;
        PrivateDirectory::open_existing(root.join("run"))?;
        PrivateDirectory::open_existing(root.join("logs"))?;
        crate::database::validate_existing_database(&settings.database_path(), &root.join("run"))
            .await
    }
    .await;
    finish_operation(result, lock.release())
}

/// Reset the sole administrator's password and revoke every prior session in
/// one database transaction, with the same audit records as current auth code.
pub async fn reset_password(root: &Path, username: &str, password: &str) -> Result<(), Error> {
    crate::verify_compiled_identity()?;
    let username = admin_auth::normalize_administrator_username(username)?;
    let password = admin_auth::Password::new(password)?;
    let settings = crate::config::load_offline(root, &root.join("config/config.json"))?.value;
    let lock = crate::config::runtime_lock(&settings)?;
    let result = async {
        PrivateDirectory::open_existing(settings.media_path())?;
        PrivateDirectory::open_existing(root.join("logs"))?;
        let database = settings.database_path();
        crate::database::validate_existing_database(&database, &root.join("run")).await?;
        let hash = password.hash()?;
        let pool = crate::database::open_database(&database).await?;
        let result = reset_administrator_password(&pool, &username, &hash).await;
        pool.close().await;
        result
    }
    .await;
    finish_operation(result, lock.release())
}

async fn reset_administrator_password(
    pool: &sqlx::SqlitePool,
    username: &str,
    hash: &str,
) -> Result<(), Error> {
    let store = SqliteAdministratorStore::new(pool.clone());
    let administrator = store
        .administrator_by_username(username)
        .await?
        .ok_or(Error::AdministratorNotFound)?;
    let now = crate::app::now_micros()?;
    let audit = |action| -> Result<_, Error> {
        Ok(SecurityAuditEvent {
            event_id: Identifier::generate()?,
            action,
            outcome: AuditOutcome::Success,
            actor_administrator_id: None,
            subject_digest: Some(admin_auth::token_hash(username)),
            request_id: None,
            detail_json: "{}".into(),
            occurred_at_micros: now,
        })
    };
    store
        .change_password(
            &administrator.administrator_id,
            hash,
            now,
            [
                audit(SecurityAction::AdministratorPasswordChanged)?,
                audit(SecurityAction::AdministratorSessionsRevoked)?,
            ],
        )
        .await?;
    Ok(())
}

fn finish_operation(
    operation: Result<(), Error>,
    release: Result<(), blog_application::state_file::Error>,
) -> Result<(), Error> {
    match (operation, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(operation), Ok(())) => Err(operation),
        (Ok(()), Err(release)) => Err(release.into()),
        (Err(operation), Err(release)) => Err(Error::StateRelease {
            operation: Box::new(operation),
            release,
        }),
    }
}

#[cfg(test)]
#[path = "../../../test/server/unit/maintenance.rs"]
mod tests;
