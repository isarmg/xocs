use crate::{
    AppState, CliFailure, PRODUCT_ID,
    app::router,
    config,
    database::{
        initialize_new_database, open_database, validate_database, validate_existing_database,
    },
    lifecycle, media, release, schema, static_assets,
};
use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use std::{
    fs,
    io::{self, Read},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::Arc,
};
use xcss::admin_auth::AdministratorOriginMode;
use xcss::admin_core::AdministratorService;
use xcss::admin_sqlite::SqliteAdministratorStore;

#[derive(Parser)]
#[command(about = "XOCS Rust service", version, long_version = concat!(env!("CARGO_PKG_VERSION"), " target=", env!("XOCS_BUILD_TARGET"), " source=", env!("XOCS_SOURCE_REVISION")))]
struct Cli {
    /// Read current JSON settings; explicit CLI > mapped environment > file > defaults.
    #[arg(long, global = true, env = "XOCS_CONFIG")]
    config: Option<PathBuf>,
    /// Private state root containing site.sqlite and media/ by default.
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new empty database. Read the administrator password from stdin.
    Init {
        #[arg(long)]
        database: Option<PathBuf>,
        #[arg(long, default_value = "admin")]
        username: String,
    },
    /// Run an initialized database with the Web resources embedded in this binary.
    Run {
        #[command(flatten)]
        settings: config::Overrides,
    },
    /// Read-only checks of configuration, current data and required locations.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Probe actual application readiness instead of checking for a process.
    Status {
        #[command(flatten)]
        settings: config::Overrides,
    },
    Doctor {
        #[arg(long)]
        database: PathBuf,
    },
    /// Print the exact Web inventory compiled into this executable.
    WebAssets,
    /// Print the software, target, source and Web resource identities.
    ReleaseIdentity,
    /// Print the current data and backup contract compiled into this executable.
    StateContract,
}

#[derive(Subcommand)]
enum ConfigCommand {
    Validate {
        #[command(flatten)]
        settings: config::Overrides,
    },
}

pub(crate) async fn run() -> std::process::ExitCode {
    let arguments = std::env::args_os().collect::<Vec<_>>();
    let machine_output = arguments.iter().any(|argument| argument == "--json");
    let mut cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            let _ = error.print();
            return std::process::ExitCode::SUCCESS;
        }
        Err(_) => {
            return xcss::server_cli::report_error(
                &xcss::server_cli::ErrorEnvelope::new(
                    xcss::server_cli::HttpStatus::BadRequest,
                    "Command arguments do not satisfy the current contract.",
                ),
                machine_output,
                2,
            );
        }
    };
    if let Some(path) = cli.config.take() {
        let current = if path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "unsafe configuration path",
            ))
        } else {
            std::path::absolute(path)
        };
        match current {
            Ok(path) => cli.config = Some(path),
            Err(_) => {
                return xcss::server_cli::report_error(
                    &xcss::config::ConfigError::new(
                        xcss::config::Reason::InvalidValue,
                        "/config",
                        xcss::config::ConfigSource::CommandLine,
                    )
                    .envelope(),
                    machine_output,
                    2,
                );
            }
        }
    }
    use tracing_subscriber::prelude::*;
    let log_layer = xcss::log::XcssStructuredLayer::new("xocs").expect("static service identity");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(log_layer.clone())
        .init();
    match execute(cli, &log_layer).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let envelope = if let Some(error) = error.downcast_ref::<xcss::config::ConfigError>() {
                error.envelope()
            } else if let Some(error) = error.downcast_ref::<xcss::state_file::Error>() {
                xcss::server_cli::state_error(error)
            } else if let Some(error) = error.downcast_ref::<xcss::server_cli::SnapshotError>() {
                xcss::server_cli::snapshot_error(error)
            } else if let Some(auth_error) = error.downcast_ref::<xcss::admin_auth::Error>() {
                let path = match auth_error {
                    xcss::admin_auth::Error::InvalidAdministratorUsername => Some("/username"),
                    xcss::admin_auth::Error::InvalidPassword => Some("/password"),
                    xcss::admin_auth::Error::InvalidPasswordHash => Some("/password_hash"),
                    _ => None,
                };
                if let Some(path) = path {
                    xcss::server_cli::ErrorEnvelope::with_code(
                        xcss::server_cli::ErrorCode::new("auth.invalid_request")
                            .expect("static code"),
                        "Administrator credentials do not satisfy the current contract.",
                    )
                    .with_detail("reason", "INVALID_VALUE")
                    .with_detail("path", path)
                } else {
                    xcss::server_cli::ErrorEnvelope::with_code(
                        xcss::server_cli::ErrorCode::new("auth.operation_failed")
                            .expect("static code"),
                        "The administrator operation could not be completed.",
                    )
                }
            } else if let Some(error) = error.downcast_ref::<CliFailure>() {
                error.0.clone()
            } else if error
                .downcast_ref::<xcss::schema_identity::Error>()
                .is_some()
                || matches!(
                    error.downcast_ref::<xcss::sqlite::Error>(),
                    Some(
                        xcss::sqlite::Error::SchemaBudgetExceeded
                            | xcss::sqlite::Error::ProductMetadataTableMissing
                            | xcss::sqlite::Error::ProductMetadataStorageClass { .. }
                            | xcss::sqlite::Error::SchemaIdentity(_)
                    )
                )
            {
                xcss::server_cli::ErrorEnvelope::with_code(
                    xcss::server_cli::ErrorCode::new("contract_violation")
                        .expect("static error code"),
                    "The actual database structure does not satisfy the current schema contract.",
                )
            } else {
                tracing::error!(%error, "Xocs command failed");
                xcss::server_cli::ErrorEnvelope::with_code(
                    xcss::server_cli::ErrorCode::new("xocs.command_failed")
                        .expect("static error code"),
                    "The command failed. Check the service diagnostic log.",
                )
            };
            xcss::server_cli::report_error(&envelope, machine_output, 1)
        }
    }
}

async fn execute(cli: Cli, log_layer: &xcss::log::XcssStructuredLayer) -> anyhow::Result<()> {
    let output_json = cli.json;
    match cli.command {
        Command::Init { database, username } => {
            let settings = config::load(
                cli.config.as_deref(),
                cli.data_dir.as_deref(),
                &config::Overrides {
                    database,
                    ..Default::default()
                },
            )?
            .value;
            let database = settings.database_path()?;
            if database.exists() {
                bail!(
                    "refusing to overwrite existing database: {}",
                    database.display()
                );
            }
            let username = xcss::admin_auth::normalize_administrator_username(&username)?;
            let mut password = String::new();
            io::stdin().take(4097).read_to_string(&mut password)?;

            let password = password.trim_end_matches(['\r', '\n']);
            xcss::admin_auth::validate_password(password)?;
            xcss::server_cli::create_empty_private_directory(&settings.state_directory()?)
                .map_err(CliFailure)?;
            let state_directory =
                xcss::state_file::PrivateStateDirectory::open(settings.state_directory()?)?;
            state_directory.verify_no_pending_maintenance()?;
            let _maintenance = state_directory.try_maintenance_lock()?;
            state_directory.verify_no_pending_maintenance()?;
            xcss::state_file::PrivateStateDirectory::create(settings.media_path()?)?;
            xcss::server_cli::create_runtime_log_directory(&settings.state_directory()?)
                .map_err(CliFailure)?;
            initialize_new_database(&database, &username, password).await?;
            tracing::info!(event = "common.initialization.completed");
            if output_json {
                println!(
                    "{}",
                    serde_json::json!({"status":"initialized","schema_identity":schema::current_identity()?})
                );
            } else {
                println!("created {}", database.display());
            }
        }
        Command::Run {
            settings: overrides,
        } => {
            let settings =
                config::load(cli.config.as_deref(), cli.data_dir.as_deref(), &overrides)?.value;
            let database = settings.database_path()?;
            let media = settings.media_path()?;
            let instance_lock = Arc::new(config::runtime_lock(&settings)?);
            let bind = settings.bind;
            let web = settings.web.clone();
            let development_http = settings.development_http;
            static_assets::verify()?;
            if let Some(web) = &web
                && (!development_http || !web.is_absolute() || !web.join("index.html").is_file())
            {
                bail!("--web requires development HTTP and an absolute built Web directory");
            }
            if development_http && !bind.ip().is_loopback() {
                bail!("development HTTP may only bind loopback");
            }
            if !media.is_absolute() {
                bail!("media directory path must be absolute");
            }
            let media_metadata =
                fs::symlink_metadata(&media).context("media directory is missing")?;
            if !media_metadata.is_dir()
                || media_metadata.file_type().is_symlink()
                || media_metadata.permissions().mode() & 0o077 != 0
            {
                bail!("media directory must be a private 0700 directory");
            }
            validate_existing_database(&database).await?;
            let pool = open_database(&database, false).await?;
            let result = async {
                validate_database(&pool).await?;
                let origin = if development_http {
                    AdministratorOriginMode::LoopbackDevelopmentHttp
                } else {
                    AdministratorOriginMode::ProductionHttps
                };
                let admin = Arc::new(AdministratorService::new(SqliteAdministratorStore::new(
                    pool.clone(),
                )));
                admin.store().validate_all_administrators().await?;
                xcss::server_cli::validate_runtime_log_directory(&settings.state_directory()?)
                    .map_err(CliFailure)?;
                let logs = settings.state_directory()?.join("logs");
                log_layer.set_rotating_file(xcss::log::RotatingLogFile::open(
                    logs,
                    "server",
                    xcss::log::LogRetention::default(),
                )?)?;
                tracing::info!(event = "common.config.loaded");
                let scope = xcss::server_runtime::WorkScope::new();
                let participant = Arc::new(lifecycle::Lifecycle {
                    scope: scope.clone(),
                    pool: pool.clone(),
                    lock: std::sync::Mutex::new(Some(instance_lock.clone())),
                });
                let health_pool = pool.clone();
                let runtime = xcss::server_runtime::ServerRuntime::builder(
                    xcss::server_runtime::ProductDescriptor {
                        id: PRODUCT_ID.into(),
                        version: env!("CARGO_PKG_VERSION").into(),
                        common_revision: env!("XCSS_REVISION").into(),
                        profile: "public-content".into(),
                        capabilities: vec!["admin-persistent".into()],
                    },
                )
                .with_schema_identity(schema::current_identity()?)
                .register_health_check(
                    "database",
                    xcss::server_runtime::health_check(move || {
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
                    trusted_proxies: settings.trusted_proxies.clone(),
                };
                let app = router(state, web)?;
                let signals = xcss::server_runtime::ProcessSignals::install()?;
                let listeners = xcss::server_runtime::BoundListeners::bind([bind])?;
                tracing::info!(event = "common.runtime.started");
                let mut transport = xcss::server_runtime::HttpServer::new(listeners, signals);
                transport.participant = Some(participant);
                tracing::info!(%bind, "XOCS listening");
                if let Err(error) = runtime.serve(transport, app).await {
                    if matches!(error, xcss::server_runtime::Error::ShutdownIncomplete(_)) {
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
        }
        Command::Doctor { database } => {
            validate_existing_database(&database).await?;
            if output_json {
                println!(
                    "{}",
                    serde_json::json!({"status":"valid","schema_identity":schema::current_identity()?})
                );
            } else {
                println!("ok");
            }
        }
        Command::Config {
            command: ConfigCommand::Validate {
                settings: overrides,
            },
        } => {
            let loaded = config::load(cli.config.as_deref(), cli.data_dir.as_deref(), &overrides)?;
            let database = loaded.value.database_path()?;
            let media = loaded.value.media_path()?;
            xcss::state_file::PrivateStateDirectory::open(loaded.value.state_directory()?)?;
            xcss::state_file::PrivateStateDirectory::open(media)?;
            validate_existing_database(&database).await?;
            xcss::server_cli::validate_runtime_log_directory(&loaded.value.state_directory()?)
                .map_err(CliFailure)?;
            let mut state_paths = vec![loaded.value.state_directory()?, loaded.value.media_path()?];
            if let Some(config) = &cli.config {
                state_paths.push(config.clone());
            }
            let report = serde_json::json!({"status":"valid","schema_identity":schema::current_identity()?,"state_paths":state_paths,"sources":loaded.sources});
            if output_json {
                println!("{report}");
            } else {
                println!("configuration and current database are valid");
            }
        }
        Command::Status {
            settings: overrides,
        } => {
            let settings =
                config::load(cli.config.as_deref(), cli.data_dir.as_deref(), &overrides)?.value;
            let report = xcss::server_cli::query_status(settings.bind, PRODUCT_ID)
                .await
                .map_err(CliFailure)?;
            xcss::server_cli::print_report(&report, output_json)?;
            if !report.ready {
                std::process::exit(1);
            }
        }
        Command::WebAssets => {
            static_assets::verify()?;
            println!("{}", static_assets::MANIFEST);
        }
        Command::ReleaseIdentity => {
            static_assets::verify()?;
            println!("{}", serde_json::to_string(&release::identity()?)?);
        }
        Command::StateContract => {
            println!("{}", String::from_utf8(release::state_contract_bytes()?)?);
        }
    }
    Ok(())
}
