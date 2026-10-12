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
    /// Read private JSON configuration (default: /etc/isarmg/xocs/config.json).
    #[arg(long, global = true, env = "XOCS_CONFIG")]
    config: Option<PathBuf>,
    /// Private state root containing site.sqlite, logs/ and media/ by default.
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    /// Structured log threshold: trace, debug, info, warn, error or off.
    #[arg(long, global = true)]
    log_level: Option<String>,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new empty database. Read the administrator password from stdin.
    Init {
        #[arg(long, default_value = "admin")]
        username: String,
    },
    /// Run an initialized database with the Web resources embedded in this binary.
    Run {
        #[command(flatten)]
        settings: config::Overrides,
    },
    /// Validate configuration without requiring initialized state.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Probe actual application readiness instead of checking for a process.
    Status {
        #[command(flatten)]
        settings: config::Overrides,
    },
    /// Read-only, offline checks of initialized state and required directories.
    Check {
        #[command(flatten)]
        settings: config::Overrides,
    },
    /// Offline administrator maintenance. Stop the service before changing credentials.
    Admin {
        #[command(subcommand)]
        command: AdminCommand,
    },
    /// Print the exact Web inventory compiled into this executable.
    WebAssets,
    /// Print the software, target, source and Web resource identities.
    ReleaseIdentity,
    /// Print the current data and backup contract compiled into this executable.
    StateContract,
}

#[derive(Subcommand)]
enum AdminCommand {
    /// Read a new password from stdin and revoke all sessions for this administrator.
    ResetPassword {
        #[arg(long, default_value = "admin")]
        username: String,
    },
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
    match execute(cli).await {
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

async fn execute(cli: Cli) -> anyhow::Result<()> {
    let output_json = cli.json;
    let mut overrides = match &cli.command {
        Command::Run { settings }
        | Command::Status { settings }
        | Command::Check { settings }
        | Command::Config {
            command: ConfigCommand::Validate { settings },
        } => settings.clone(),
        _ => config::Overrides::default(),
    };
    overrides.log_level = cli.log_level;
    let loaded = match &cli.command {
        Command::WebAssets | Command::ReleaseIdentity | Command::StateContract => None,
        _ => Some(config::load(
            cli.config.as_deref(),
            cli.data_dir.as_deref(),
            &overrides,
        )?),
    };
    let log_layer = xcss::log::XcssStructuredLayer::new("xocs")?;
    use tracing_subscriber::prelude::*;
    let level = loaded
        .as_ref()
        .map_or("info", |loaded| loaded.value.log_level.as_str());
    tracing_subscriber::registry()
        .with(level.parse::<tracing_subscriber::filter::LevelFilter>()?)
        .with(log_layer.clone())
        .try_init()?;

    match cli.command {
        Command::Init { username } => {
            let settings = &loaded.as_ref().expect("configuration loaded").value;
            let database = settings.database_path()?;
            if database.exists() {
                bail!(
                    "refusing to overwrite existing database: {}",
                    database.display()
                );
            }
            let username = xcss::admin_auth::normalize_administrator_username(&username)?;
            let password = read_password()?;
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
            attach_log_file(settings, &log_layer)?;
            initialize_new_database(&database, &username, &password).await?;
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
        Command::Run { .. } => {
            let settings = &loaded.as_ref().expect("configuration loaded").value;
            let database = settings.database_path()?;
            let media = settings.media_path()?;
            let instance_lock = Arc::new(config::runtime_lock(settings)?);
            let bind = settings.bind;
            let web = settings.web.clone();
            let development = settings.development;
            static_assets::verify()?;
            if let Some(web) = &web
                && (!development || !web.is_absolute() || !web.join("index.html").is_file())
            {
                bail!("--web requires development HTTP and an absolute built Web directory");
            }
            if development && !bind.ip().is_loopback() {
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
                let origin = if development {
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
                attach_log_file(settings, &log_layer)?;
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
        Command::Admin {
            command: AdminCommand::ResetPassword { username },
        } => {
            let settings = &loaded.as_ref().expect("configuration loaded").value;
            let username = xcss::admin_auth::normalize_administrator_username(&username)?;
            let password = read_password()?;
            let state = xcss::state_file::PrivateStateDirectory::open(settings.state_directory()?)?;
            state.verify_no_pending_maintenance()?;
            let _maintenance = state.try_maintenance_lock()?;
            state.verify_no_pending_maintenance()?;
            check_state(settings).await?;
            attach_log_file(settings, &log_layer)?;
            let pool = open_database(&settings.database_path()?, false).await?;
            let service = AdministratorService::new(SqliteAdministratorStore::new(pool.clone()));
            let result = service
                .change_administrator_password(&username, &password, crate::now_micros()?)
                .await;
            pool.close().await;
            result?;
            if output_json {
                println!(
                    "{}",
                    serde_json::json!({"status":"password_reset","username":username,"sessions_revoked":true})
                );
            } else {
                println!("administrator password reset; existing sessions revoked");
            }
        }
        Command::Check { .. } => {
            let settings = &loaded.as_ref().expect("configuration loaded").value;
            check_state(settings).await?;
            if output_json {
                println!(
                    "{}",
                    serde_json::json!({"status":"valid","schema_identity":schema::current_identity()?,"state_paths":[settings.data_dir, settings.media_path()?]})
                );
            } else {
                println!("initialized state is valid");
            }
        }
        Command::Config { .. } => {
            let loaded = loaded.as_ref().expect("configuration loaded");
            if output_json {
                println!(
                    "{}",
                    serde_json::json!({"status":"valid","sources":loaded.sources})
                );
            } else {
                println!("configuration is valid");
            }
        }
        Command::Status { .. } => {
            let settings = &loaded.as_ref().expect("configuration loaded").value;
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

fn read_password() -> anyhow::Result<String> {
    let mut password = String::new();
    io::stdin().take(4097).read_to_string(&mut password)?;
    anyhow::ensure!(password.len() <= 4096, "password input is too long");
    if password.ends_with('\n') {
        password.pop();
        if password.ends_with('\r') {
            password.pop();
        }
    }
    anyhow::ensure!(
        !password.contains(['\r', '\n']),
        "password input must contain exactly one line"
    );
    xcss::admin_auth::validate_password(&password)?;
    Ok(password)
}

fn attach_log_file(
    settings: &config::Settings,
    log_layer: &xcss::log::XcssStructuredLayer,
) -> anyhow::Result<()> {
    xcss::server_cli::validate_runtime_log_directory(&settings.data_dir).map_err(CliFailure)?;
    log_layer.set_rotating_file(xcss::log::RotatingLogFile::open(
        settings.data_dir.join("logs"),
        "server",
        xcss::log::LogRetention::default(),
    )?)?;
    Ok(())
}

async fn check_state(settings: &config::Settings) -> anyhow::Result<()> {
    let state = xcss::state_file::PrivateStateDirectory::open(settings.state_directory()?)?;
    state.verify_no_pending_maintenance()?;
    xcss::state_file::PrivateStateDirectory::open(settings.media_path()?)?;
    xcss::server_cli::validate_runtime_log_directory(&settings.data_dir).map_err(CliFailure)?;
    validate_existing_database(&settings.database_path()?).await?;
    Ok(())
}
