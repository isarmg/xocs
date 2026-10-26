use crate::Error;
use crate::{
    AppState, PRODUCT_ID,
    app::router,
    config,
    database::{open_database, validate_existing_database},
    lifecycle, media, schema, static_assets,
};
use blog_application::admin_auth::AdministratorOriginMode;
use blog_application::admin_core::AdministratorService;
use blog_application::admin_sqlite::SqliteAdministratorStore;
use std::{fs, path::Path, sync::Arc};

pub(crate) async fn run(root: &Path) -> Result<(), Error> {
    static_assets::verify()?;
    let loaded = config::load(root, &root.join("config/config.json"))?;
    let settings = &loaded.value;
    let log_layer = blog_application::log::BlogStructuredLayer::new("blog")?;
    use tracing::collector::prelude::*;
    tracing::collector::registry()
        .with(
            settings
                .log_level
                .parse::<tracing::collector::filter::LevelFilter>()
                .expect("invariant: configured log level was validated"),
        )
        .with(log_layer)
        .try_init()?;
    let database = settings.database_path();
    let media = settings.media_path();
    let bind = settings.bind;
    let web = settings.web.clone();
    let development = settings.development;
    if let Some(web) = &web
        && !fs::metadata(web.join("index.html"))
            .map_err(Error::ReadWebEntry)?
            .is_file()
    {
        return Err(Error::InvalidWebEntry);
    }
    blog_application::fs_safety::PrivateDirectory::open_existing(&media)?;
    let instance_lock = config::runtime_lock(settings)?;
    let pool = match async {
        validate_existing_database(&database, &settings.root.join("run")).await?;
        open_database(&database).await
    }
    .await
    {
        Ok(pool) => pool,
        Err(operation) => {
            return match instance_lock.release() {
                Ok(()) => Err(operation),
                Err(release) => Err(Error::StateRelease {
                    operation: Box::new(operation),
                    release,
                }),
            };
        }
    };
    let scope = blog_application::server_runtime::WorkScope::new();
    let participant = Arc::new(lifecycle::Lifecycle {
        scope: scope.clone(),
        pool: pool.clone(),
        lock: std::sync::Mutex::new(Some(instance_lock)),
    });
    let result = async {
        let origin = if development {
            AdministratorOriginMode::LoopbackDevelopmentHttp
        } else {
            AdministratorOriginMode::ProductionHttps
        };
        let admin = Arc::new(AdministratorService::new(SqliteAdministratorStore::new(
            pool.clone(),
        )));
        attach_log_file(settings)?;
        tracing::info!(event = "common.config.loaded");
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
        .with_schema_identity(schema::current_identity())
        .register_health_check(
            "database",
            blog_application::server_runtime::health_check(move || {
                let pool = health_pool.clone();
                async move { crate::app::database_ready(&pool).await }
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
        let signals = blog_application::server_runtime::ProcessSignals::install()
            .map_err(Error::InstallProcessSignals)?;
        let listeners = blog_application::server_runtime::BoundListeners::bind([bind])
            .map_err(Error::BindListeners)?;
        tracing::info!(event = "common.runtime.started");
        let mut transport = blog_application::server_runtime::HttpServer::new(listeners, signals);
        transport.participant = Some(Arc::clone(&participant)
            as Arc<dyn blog_application::server_runtime::LifecycleParticipant>);
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
        Ok::<_, Error>(())
    }
    .await;
    pool.close().await;
    match (result, participant.release_lock()) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(operation), Ok(())) => Err(operation),
        (Ok(()), Err(release)) => Err(release.into()),
        (Err(operation), Err(release)) => Err(Error::StateRelease {
            operation: Box::new(operation),
            release,
        }),
    }
}

fn attach_log_file(settings: &config::Settings) -> Result<(), Error> {
    blog_application::log::install_rotating_file(blog_application::log::RotatingLogFile::open(
        settings.root.join("logs"),
        "server",
        blog_application::log::LogRetention::default(),
    )?)?;
    Ok(())
}
