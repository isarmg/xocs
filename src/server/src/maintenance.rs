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
    let state = PrivateDirectory::create(&settings.data_dir)?;
    let lock = crate::config::runtime_lock(&settings)?;
    let database = settings.database_path();
    let media = PrivateDirectory::create(settings.media_path())?;
    let run = PrivateDirectory::create(root.join("run"))?;
    let logs = PrivateDirectory::create(root.join("logs"))?;
    match crate::database::initialize_new_database(&database, &username, &password).await {
        Ok(()) => {}
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
            return Err(Error::PublishDatabase { staging, source });
        }
        Err(error) => return Err(error),
    }
    crate::database::validate_existing_database(&database, &root.join("run")).await?;
    for directory in [&state, &media, &run, &logs] {
        directory.verify_identity()?;
    }
    drop(lock);
    Ok(())
}

/// Validate a stopped instance using a bounded temporary database snapshot.
pub async fn check(root: &Path) -> Result<(), Error> {
    crate::verify_compiled_identity()?;
    let settings = crate::config::load_offline(root, &root.join("config/config.json"))?.value;
    let lock = crate::config::runtime_lock(&settings)?;
    let media = PrivateDirectory::open_existing(settings.media_path())?;
    let run = PrivateDirectory::open_existing(root.join("run"))?;
    let logs = PrivateDirectory::open_existing(root.join("logs"))?;
    crate::database::validate_existing_database(&settings.database_path(), &root.join("run"))
        .await?;
    for directory in [&media, &run, &logs] {
        directory.verify_identity()?;
    }
    drop(lock);
    Ok(())
}

/// Reset the sole administrator's password and revoke every prior session in
/// one database transaction, with the same audit records as current auth code.
pub async fn reset_password(root: &Path, username: &str, password: &str) -> Result<(), Error> {
    crate::verify_compiled_identity()?;
    let username = admin_auth::normalize_administrator_username(username)?;
    let password = admin_auth::Password::new(password)?;
    let settings = crate::config::load_offline(root, &root.join("config/config.json"))?.value;
    let lock = crate::config::runtime_lock(&settings)?;
    PrivateDirectory::open_existing(settings.media_path())?;
    PrivateDirectory::open_existing(root.join("logs"))?;
    let database = settings.database_path();
    crate::database::validate_existing_database(&database, &root.join("run")).await?;
    let hash = password.hash()?;
    let pool = crate::database::open_database(&database, false).await?;
    let result = async {
        let store = SqliteAdministratorStore::new(pool.clone());
        let administrator = store
            .administrator_by_username(&username)
            .await?
            .ok_or(Error::AdministratorNotFound)?;
        let now = crate::app::now_micros()?;
        let audit = |action| -> Result<_, Error> {
            Ok(SecurityAuditEvent {
                event_id: Identifier::new(admin_auth::random_token()?)?,
                action,
                outcome: AuditOutcome::Success,
                actor_administrator_id: None,
                subject_digest: Some(admin_auth::token_hash(&username)),
                request_id: None,
                detail_json: "{}".into(),
                occurred_at_micros: now,
            })
        };
        store
            .change_password(
                &administrator.administrator_id,
                &hash,
                now,
                [
                    audit(SecurityAction::AdministratorPasswordChanged)?,
                    audit(SecurityAction::AdministratorSessionsRevoked)?,
                ],
            )
            .await?;
        store.validate_all_administrators().await?;
        Ok::<(), Error>(())
    }
    .await;
    pool.close().await;
    result?;
    drop(lock);
    Ok(())
}
