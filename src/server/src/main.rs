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
                _ => blog_application::server_cli::ErrorEnvelope::with_code(
                    blog_application::server_cli::ErrorCode::new("blog.startup_failed")
                        .expect("static error code"),
                    "The service could not start. Check configuration and service diagnostics.",
                ),
            };
            blog_application::server_cli::report_error(&envelope, false, 1)
        }
    }
}
