use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blog", version, about = "BLOG content server")]
struct Cli {
    #[arg(long)]
    root: PathBuf,
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match blog::run(&cli.root).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "BLOG startup failed");
            let envelope = match &error {
                blog::Error::Config(error) => error.envelope(),
                blog::Error::State(error) => blog_application::server_cli::state_error(error),
                blog::Error::Snapshot(error) => blog_application::server_cli::snapshot_error(error),
                blog::Error::Assets(_)
                | blog::Error::Filesystem(_)
                | blog::Error::Sqlite(_)
                | blog::Error::SchemaValidationRollback { .. }
                | blog::Error::StateRelease { .. }
                | blog::Error::Database(_)
                | blog::Error::Administrator(_)
                | blog::Error::AdministratorStore(_)
                | blog::Error::Authentication(_)
                | blog::Error::Runtime(_)
                | blog::Error::Log(_)
                | blog::Error::Tracing(_)
                | blog::Error::CreateDatabase(_)
                | blog::Error::ReadDatabaseDescriptor(_)
                | blog::Error::ReadDatabasePath(_)
                | blog::Error::CreateInitializationStaging(_)
                | blog::Error::ReadWebEntry(_)
                | blog::Error::InstallProcessSignals(_)
                | blog::Error::BindListeners(_)
                | blog::Error::TimestampRange(_)
                | blog::Error::Clock(_)
                | blog::Error::Environment { .. }
                | blog::Error::InvalidRoot
                | blog::Error::InvalidWebEntry
                | blog::Error::DatabaseIdentityChanged
                | blog::Error::InitialCheckpointBusy
                | blog::Error::InitialWalMode
                | blog::Error::RetainStaging(_)
                | blog::Error::PublishDatabase { .. }
                | blog::Error::ForeignKeyViolations
                | blog::Error::DuplicateFamily
                | blog::Error::MissingAuthor
                | blog::Error::NotInitialized
                | blog::Error::IntegrityCheckFailed
                | blog::Error::AdministratorNotFound
                | blog::Error::MediaRecovery(_) => {
                    blog_application::server_cli::ErrorEnvelope::with_code(
                        blog_application::server_cli::ErrorCode::new("blog.startup_failed")
                            .expect("static error code"),
                        "The service could not start. Check configuration and service diagnostics.",
                    )
                }
            };
            blog_application::server_cli::report_error(&envelope, false, 1)
        }
    }
}
