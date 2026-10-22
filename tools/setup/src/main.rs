//! Blog-owned offline administration. Passwords are read only from standard input.
#![deny(unsafe_code)]
use anyhow::{Context, ensure};
use clap::{Parser, Subcommand};
use std::{io::Read, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "Initialize and maintain a stopped Blog instance")]
struct Cli {
    #[arg(long)]
    root: PathBuf,
    #[arg(long)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize the current schema and the only administrator; password on stdin.
    Initialize {
        #[arg(long, default_value = "admin")]
        username: String,
    },
    /// Check current schema, database contents, private files and administrator.
    Check,
    /// Change the administrator password and revoke sessions; password on stdin.
    AdminReset {
        #[arg(long)]
        username: String,
    },
}

fn password() -> anyhow::Result<String> {
    let mut bytes = Vec::new();
    std::io::stdin().take(4097).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 4096, "password input exceeds its bound");
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    ensure!(
        !bytes.iter().any(|byte| matches!(byte, b'\r' | b'\n')),
        "password must be one line"
    );
    String::from_utf8(bytes).context("password input must be UTF-8")
}

async fn execute(cli: Cli) -> anyhow::Result<()> {
    blog::verify_compiled_identity()?;
    if let Some(config) = &cli.config {
        ensure!(
            *config == cli.root.join("config/config.json"),
            "configuration must identify ROOT/config/config.json"
        );
    }
    match cli.command {
        Command::Initialize { username } => {
            blog::maintenance::initialize(&cli.root, &username, &password()?).await?;
        }
        Command::Check => blog::maintenance::check(&cli.root).await?,
        Command::AdminReset { username } => {
            blog::maintenance::reset_password(&cli.root, &username, &password()?).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match execute(Cli::parse()).await {
        Ok(()) => {
            println!("Blog offline operation completed.");
            std::process::ExitCode::SUCCESS
        }
        Err(_) => {
            // Errors may carry input values. Never print a chained diagnostic
            // from a password-consuming operation.
            eprintln!(
                "Blog offline operation failed. Verify configuration, permissions, current state, stopped writers and credential policy."
            );
            std::process::ExitCode::FAILURE
        }
    }
}
