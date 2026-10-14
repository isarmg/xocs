use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xocs", version, about = "XOCS content server")]
struct Cli {
    #[arg(long)]
    root: PathBuf,
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match xocs::run(&cli.root).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "XOCS startup failed");
            let envelope = if let Some(error) = error.downcast_ref::<xcss::config::ConfigError>() {
                error.envelope()
            } else if let Some(error) = error.downcast_ref::<xcss::state_file::Error>() {
                xcss::server_cli::state_error(error)
            } else if let Some(error) = error.downcast_ref::<xcss::sqlite::SnapshotError>() {
                xcss::server_cli::snapshot_error(error)
            } else {
                xcss::server_cli::ErrorEnvelope::with_code(
                    xcss::server_cli::ErrorCode::new("xocs.startup_failed")
                        .expect("static error code"),
                    "The service could not start. Check configuration and service diagnostics.",
                )
            };
            xcss::server_cli::report_error(&envelope, false, 1)
        }
    }
}
