use crate::{
    AppState, PRODUCT_ID,
    app::router,
    config,
    database::{open_database, validate_existing_database},
    lifecycle, media, schema, static_assets,
};
use anyhow::{Context, bail};
use blog_application::admin_auth::AdministratorOriginMode;
use blog_application::admin_core::{AdministratorService, AdministratorStore as _};
use blog_application::admin_sqlite::SqliteAdministratorStore;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, sync::Arc};

pub(crate) async fn run(root: &Path) -> anyhow::Result<()> {
    static_assets::verify()?;
    let loaded = config::load(root, &root.join("config/config.json"))?;
    let settings = &loaded.value;
    let log_layer = blog_application::log::BlogStructuredLayer::new("blog")?;
    use tracing::collector::prelude::*;
    tracing::collector::registry()
        .with(
            settings
                .log_level
                .parse::<tracing::collector::filter::LevelFilter>()?,
        )
        .with(log_layer)
        .try_init()?;
    let database = settings.database_path()?;
    let media = settings.media_path()?;
    let instance_lock = Arc::new(config::runtime_lock(settings)?);
    let bind = settings.bind;
    let web = settings.web.clone();
    let development = settings.development;
    if let Some(web) = &web
        && (!development || !web.is_absolute() || !web.join("index.html").is_file())
    {
        bail!(
            "Web directory configuration requires development HTTP and an absolute built directory"
        );
    }
    if development && !bind.ip().is_loopback() {
        bail!("development HTTP may only bind loopback");
    }
    if !media.is_absolute() {
        bail!("media directory path must be absolute");
    }
    let media_metadata = fs::symlink_metadata(&media).context("media directory is missing")?;
    if !media_metadata.is_dir()
        || media_metadata.file_type().is_symlink()
        || media_metadata.permissions().mode() & 0o077 != 0
    {
        bail!("media directory must be a private 0700 directory");
    }
    validate_existing_database(&database, &settings.root.join("run")).await?;
    let pool = open_database(&database, false).await?;
    let result = async {
        let origin = if development {
            AdministratorOriginMode::LoopbackDevelopmentHttp
        } else {
            AdministratorOriginMode::ProductionHttps
        };
        let admin = Arc::new(AdministratorService::new(SqliteAdministratorStore::new(
            pool.clone(),
        )));
        anyhow::ensure!(
            admin.store().administrator_count().await? > 0,
            "explicit initialization is required"
        );
        blog_application::fs_safety::PrivateDirectory::open_existing(settings.root.join("logs"))?;
        attach_log_file(settings)?;
        tracing::info!(event = "common.config.loaded");
        let scope = blog_application::server_runtime::WorkScope::new();
        let participant = Arc::new(lifecycle::Lifecycle {
            scope: scope.clone(),
            pool: pool.clone(),
            lock: std::sync::Mutex::new(Some(instance_lock.clone())),
        });
        let health_pool = pool.clone();
        let runtime = blog_application::server_runtime::ServerRuntime::builder(
            blog_application::server_runtime::ProductDescriptor {
                id: PRODUCT_ID.into(),
                version: env!("CARGO_PKG_VERSION").into(),
                core_source_sha256: env!("BLOG_CORE_SOURCE_SHA256").into(),
                profile: "public-content".into(),
                capabilities: vec!["admin-persistent".into()],
            },
        )
        .with_schema_identity(schema::current_identity()?)
        .register_health_check(
            "database",
            blog_application::server_runtime::health_check(move || {
                let pool = health_pool.clone();
                async move {
                    sqlx::query_scalar::<_, i64>("SELECT 1")
                        .fetch_one(&pool)
                        .await
                        .is_ok_and(|value| value == 1)
                }
            }),
        )
        .build()
        .await?;
        media::recover_deletions(&pool, &media).await?;
        let state = AppState {
            scope,
            pool: pool.clone(),
            admin,
            origin,
            media,
            media_operations: std::sync::Arc::new(tokio::sync::Mutex::new(())),
            public_mutations: Arc::new(crate::app::PublicMutationCapacity::default()),
            password_slots: Arc::new(tokio::sync::Semaphore::new(2)),
        };
        let app = router(state, web)?;
        let signals = blog_application::server_runtime::ProcessSignals::install()?;
        let listeners = blog_application::server_runtime::BoundListeners::bind([bind])?;
        tracing::info!(event = "common.runtime.started");
        let mut transport = blog_application::server_runtime::HttpServer::new(listeners, signals);
        transport.participant = Some(participant);
        tracing::info!(%bind, "BLOG listening");
        if let Err(error) = runtime.serve(transport, app).await {
            if matches!(
                error,
                blog_application::server_runtime::Error::ShutdownIncomplete(_)
            ) {
                // An incomplete commit drain must retain the state owner and
                // its locks until the operating system ends the process.
                eprintln!("{error}");
                std::process::exit(1);
            }
            return Err(error.into());
        }
        tracing::info!(event = "common.runtime.stopped");
        Ok::<_, anyhow::Error>(())
    }
    .await;
    pool.close().await;
    result?;
    Ok(())
}

fn attach_log_file(settings: &config::Settings) -> anyhow::Result<()> {
    blog_application::fs_safety::PrivateDirectory::open_existing(settings.root.join("logs"))?;
    blog_application::log::install_rotating_file(blog_application::log::RotatingLogFile::open(
        settings.root.join("logs"),
        "server",
        blog_application::log::LogRetention::default(),
    )?)?;
    Ok(())
}
