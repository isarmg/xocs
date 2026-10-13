//! Current offline adapters used by the independent signed installation helper.
use crate::{CliFailure, config, database, schema};
use std::path::Path;

pub async fn validate_config(
    root: &Path,
    path: &Path,
) -> anyhow::Result<xcss::offline::ConfigReport> {
    let settings = config::load(root, path)?.value;
    let data = settings.state_directory()?;
    let media = settings.media_path()?;
    let mut state_paths = vec![root.join("config"), root.join("logs"), data.clone()];
    if !state_paths.iter().any(|parent| media.starts_with(parent)) {
        state_paths.push(media.clone());
    }
    Ok(xcss::offline::ConfigReport {
        product: "xocs".into(),
        root: root.to_path_buf(),
        data_dir: data.clone(),
        state_paths,
        shared_paths: vec![],
        schema_identity: schema::current_identity()?,
        bind: settings.bind,
        writable_paths: vec![data, media, root.join("logs"), root.join("run")],
    })
}

pub async fn initialize(
    root: &Path,
    path: &Path,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let value = config::load(root, path)?.value;
    let username = xcss::admin_auth::normalize_administrator_username(username)?;
    xcss::admin_auth::validate_password(password)?;
    xcss::server_cli::create_empty_private_directory(&value.data_dir).map_err(CliFailure)?;
    let state = xcss::state_file::PrivateStateDirectory::open(&value.data_dir)?;
    state.verify_no_pending_maintenance()?;
    let _maintenance = state.try_maintenance_lock()?;
    state.verify_no_pending_maintenance()?;
    for name in ["logs", "run"] {
        xcss::state_file::PrivateStateDirectory::create(root.join(name))?;
    }
    xcss::state_file::PrivateStateDirectory::create(value.media_path()?)?;
    database::initialize_new_database(&value.database_path()?, &username, password).await?;
    Ok(())
}

pub async fn check(root: &Path, path: &Path) -> anyhow::Result<xcss::offline::CheckReport> {
    let report = validate_config(root, path).await?;
    let value = config::load(root, path)?.value;
    xcss::state_file::PrivateStateDirectory::open(&value.data_dir)?;
    xcss::state_file::PrivateStateDirectory::open(value.media_path()?)?;
    xcss::fs_safety::PrivateDirectory::open_existing(root.join("logs"))?;
    xcss::fs_safety::PrivateDirectory::open_existing(root.join("run"))?;
    database::validate_existing_database(&value.database_path()?, &root.join("run")).await?;
    Ok(xcss::offline::CheckReport {
        config: report,
        initialized: true,
    })
}

pub async fn reset_password(
    root: &Path,
    path: &Path,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let value = config::load(root, path)?.value;
    let username = xcss::admin_auth::normalize_administrator_username(username)?;
    xcss::admin_auth::validate_password(password)?;
    let state = xcss::state_file::PrivateStateDirectory::open(&value.data_dir)?;
    state.verify_no_pending_maintenance()?;
    let _maintenance = state.try_maintenance_lock()?;
    state.verify_no_pending_maintenance()?;
    check(root, path).await?;
    let pool = database::open_database(&value.database_path()?, false).await?;
    let service = xcss::admin_core::AdministratorService::new(
        xcss::admin_sqlite::SqliteAdministratorStore::new(pool.clone()),
    );
    let result = service
        .change_administrator_password(&username, password, crate::now_micros()?)
        .await;
    pool.close().await;
    result?;
    Ok(())
}

pub async fn migrate(
    root: &Path,
    path: &Path,
    data_dir: &Path,
    source: &xcss::contracts::SchemaIdentity,
    target: &xcss::contracts::SchemaIdentity,
) -> anyhow::Result<()> {
    let report = validate_config(root, path).await?;
    anyhow::ensure!(
        source == target && target == &report.schema_identity,
        "XOCS does not support the requested schema migration"
    );
    anyhow::ensure!(
        data_dir == report.data_dir,
        "migration data_dir differs from the locked instance"
    );
    check(root, path).await?;
    Ok(())
}
