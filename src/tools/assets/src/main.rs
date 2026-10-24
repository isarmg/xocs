use anyhow::{Context, Result, bail};
use std::{collections::BTreeMap, env, path::PathBuf};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let status = match run_cli().await {
        Ok(status) => status,
        Err(error) => {
            eprintln!("blog-assets: {error:#}");
            1
        }
    };
    if status != 0 {
        std::process::exit(status);
    }
}

async fn run_cli() -> Result<i32> {
    use tokio::signal::unix::{SignalKind, signal};

    // Register before starting synchronous work. Owned child process groups do
    // not receive terminal signals sent to this executable's foreground group.
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    let mut operation = tokio::task::spawn_blocking(run);
    let status = tokio::select! {
        _ = interrupt.recv() => 130,
        _ = terminate.recv() => 143,
        result = &mut operation => {
            result.context("Asset command task failed")??;
            return Ok(0);
        }
    };
    blog_assets::request_cancellation();
    // Await Drop/cleanup in the blocking command, including all pipe readers,
    // before exiting rather than abandoning its separately owned child group.
    let _ = operation.await;
    Ok(status)
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    let mut values = BTreeMap::<String, Vec<String>>::new();
    while let Some(key) = args.next() {
        if matches!(
            key.as_str(),
            "--offline"
                | "--debug"
                | "--no-watch"
                | "--skip-build"
                | "--web-only"
                | "--rust-only"
                | "--no-install"
        ) {
            values.entry(key).or_default();
        } else if key.starts_with("--") {
            values.entry(key.clone()).or_default().push(
                args.next()
                    .with_context(|| format!("{key} requires a value"))?,
            );
        } else {
            bail!("Unexpected argument {key:?}");
        }
    }
    let allowed = [
        "--product-root",
        "--offline",
        "--debug",
        "--dist",
        "--bind",
        "--proxy",
        "--no-watch",
        "--skip-build",
        "--config",
        "--mode",
        "--source-revision",
        "--web-only",
        "--rust-only",
        "--no-install",
        "--binary",
        "--receipt",
    ];
    for key in values.keys() {
        if !allowed.contains(&key.as_str()) {
            bail!("Unknown option {key}");
        }
        if key != "--proxy" && values[key].len() > 1 {
            bail!("Duplicate option {key}");
        }
    }
    let value = |name: &str| values.get(name).and_then(|v| v.first()).cloned();
    let root = value("--product-root")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?)
        .canonicalize()?;
    let options = blog_assets::Options {
        root,
        offline: values.contains_key("--offline"),
        debug: values.contains_key("--debug"),
        dist: value("--dist")
            .map(PathBuf::from)
            .or_else(|| env::var_os("BLOG_WEB_DIST").map(PathBuf::from)),
    };
    match command.as_str() {
        "build" => {
            blog_assets::build(&options)?;
        }
        "server" => {
            let mode = value("--mode").unwrap_or_else(|| "development".into());
            let config = value("--config");
            let revision = value("--source-revision");
            let output = blog_assets::server_build::build(
                &options,
                config.as_deref(),
                &mode,
                values.contains_key("--web-only"),
                values.contains_key("--rust-only"),
                revision.as_deref(),
            )?;
            println!("{}", output.display());
        }
        "verify" => {
            let binary = PathBuf::from(value("--binary").context("verify requires --binary")?);
            let receipt = PathBuf::from(value("--receipt").context("verify requires --receipt")?);
            let dist = blog_assets::distribution(&options)?;
            blog_assets::server_build::verify(&binary, &dist, &receipt)?;
            println!("Verified {}", binary.display());
        }
        "check" | "test" => blog_assets::cargo_action(&options, &command)?,
        "source-stamp" => println!(
            "{}",
            serde_json::to_string(&blog_assets::server_build::source_stamp(&options.root)?)?
        ),
        "graph" => println!("{}", blog_assets::graph::resolve(&options)?),
        "check-sources" => {
            blog_assets::inventory::snapshot(&options.root)?;
        }
        "serve" | "preview" => {
            let bind = value("--bind")
                .unwrap_or_else(|| "127.0.0.1:5173".into())
                .parse()?;
            let proxies = values.get("--proxy").cloned().unwrap_or_default();
            tokio::runtime::Runtime::new()?.block_on(blog_assets::dev::serve(
                options,
                bind,
                proxies,
                command == "serve" && !values.contains_key("--no-watch"),
                command == "preview" || values.contains_key("--skip-build"),
            ))?;
        }
        "help" | "--help" | "-h" => println!(
            "blog-assets build|check|test|serve|preview|server|check-sources|source-stamp\n  --product-root DIR --offline --debug --dist DIR\n  serve: --bind 127.0.0.1:5173 --proxy /api=http://127.0.0.1:8080 [--no-watch] [--skip-build]\n  server: --mode development|candidate|release --config blog-assets.json [--web-only|--rust-only] [--source-revision SHA]\nConfiguration: src/web/rust-web.toml; see src/tools/assets/README.md"
        ),
        _ => bail!("Unknown command {command:?}; use help"),
    }
    Ok(())
}
