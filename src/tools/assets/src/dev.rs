//! Local development uses Axum's HTTP implementation and a bounded, streaming
//! HTTP/1 proxy. Credentials, Host and Origin are forwarded without rewriting.
use crate::layout;
use crate::{Options, assets::Config, inventory};
use anyhow::{Context, Result, ensure};
use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode, header},
    response::Response,
};
use serde_json::{Value, json};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;

mod proxy;

#[derive(Clone)]
struct App {
    dist: PathBuf,
    base: String,
    proxies: Vec<(String, String)>,
    client: proxy::Client,
    status: Arc<RwLock<Value>>,
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "wasm" => "application/wasm",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ico" => "image/x-icon",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "txt" => "text/plain; charset=utf-8",
        _ => mime_guess::from_path(path)
            .first_raw()
            .unwrap_or("application/octet-stream"),
    }
}

fn plain(status: StatusCode, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(message.to_owned()))
        .expect("Constant headers")
}

async fn handler(State(app): State<App>, request: Request) -> Response {
    match respond(app, request).await {
        Ok(response) => response,
        Err(error) => {
            eprintln!("development request failed: {error:#}");
            plain(
                StatusCode::BAD_GATEWAY,
                "Development request failed; see the local build log\n",
            )
        }
    }
}

async fn respond(app: App, request: Request) -> Result<Response> {
    let path = request.uri().path();
    if path == "/__rust_web/status" {
        return Ok(Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CACHE_CONTROL, "no-store")
            .body(Body::from(serde_json::to_vec(&*app.status.read().await)?))?);
    }
    if let Some((prefix, target)) = app.proxies.iter().find(|(prefix, _)| {
        path == prefix || path.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
    }) {
        ensure!(
            !request.headers().contains_key(header::UPGRADE),
            "WebSocket proxying is not configured; use the backend WebSocket origin directly"
        );
        let suffix = request
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or("/");
        let url = if target.ends_with('/') {
            format!(
                "{}{}",
                target,
                suffix
                    .strip_prefix(prefix)
                    .unwrap_or(suffix)
                    .trim_start_matches('/')
            )
        } else {
            format!("{target}{suffix}")
        };
        return app.client.forward(request, url.parse()?).await;
    }
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(header::ALLOW, "GET, HEAD")
            .body(Body::empty())?);
    }
    let decoded = url_encoding::percent::percent_decode_str(path).decode_utf8()?;
    ensure!(
        !decoded.contains(['\\', '\0']),
        "Invalid static resource path"
    );
    let relative = decoded
        .strip_prefix(&app.base)
        .unwrap_or(decoded.trim_start_matches('/'));
    let mut source = app.dist.join(if relative.is_empty() {
        "index.html"
    } else {
        relative
    });
    inventory::relative(source.strip_prefix(&app.dist)?)?;
    if !source.is_file() {
        if Path::new(relative).extension().is_some() || relative.starts_with("assets/") {
            return Ok(plain(StatusCode::NOT_FOUND, "Not found\n"));
        }
        source = app.dist.join("index.html");
    }
    if !source.is_file() {
        return Ok(plain(
            StatusCode::SERVICE_UNAVAILABLE,
            "Frontend is compiling; inspect /__rust_web/status\n",
        ));
    }
    let html = source.extension().is_some_and(|v| v == "html");
    let compressed_path = PathBuf::from(format!("{}.gz", source.display()));
    let wasm = source.extension().is_some_and(|v| v == "wasm");
    let accepted = header::accepted_encodings(request.headers());
    let compressed = wasm && compressed_path.is_file() && accepted.gzip;
    if wasm && !compressed && !accepted.identity {
        return Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .header(header::CACHE_CONTROL, "no-store")
            .header(header::CONTENT_LENGTH, "0")
            .header(header::VARY, "Accept-Encoding")
            .body(Body::empty())?);
    }
    let body = inventory::read(if compressed {
        &compressed_path
    } else {
        &source
    })?;
    let etag = format!("\"{}\"", inventory::sha(&body));
    let unchanged = !html
        && request
            .headers()
            .get_all(header::IF_NONE_MATCH)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(','))
            .any(|v| v.trim() == "*" || v.trim().trim_start_matches("W/") == etag);
    let mut response = Response::builder()
        .status(if unchanged {
            StatusCode::NOT_MODIFIED
        } else {
            StatusCode::OK
        })
        .header(header::CONTENT_TYPE, content_type(&source))
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::REFERRER_POLICY, "same-origin")
        .header(
            header::CACHE_CONTROL,
            if html {
                "no-store, max-age=0"
            } else {
                "public, no-cache"
            },
        );
    if !html {
        response = response.header(header::ETAG, etag);
    }
    if source.extension().is_some_and(|v| v == "wasm") {
        response = response.header(header::VARY, "Accept-Encoding");
    }
    if compressed {
        response = response.header(header::CONTENT_ENCODING, "gzip");
    }
    if !unchanged {
        response = response.header(header::CONTENT_LENGTH, body.len());
    }
    Ok(
        response.body(if unchanged || request.method() == Method::HEAD {
            Body::empty()
        } else {
            Body::from(body)
        })?,
    )
}

async fn rebuild(options: Options, status: Arc<RwLock<Value>>) {
    {
        status.write().await["building"] = json!(true);
    }
    let result = tokio::task::spawn_blocking(move || crate::build(&options)).await;
    let mut status = status.write().await;
    status["building"] = json!(false);
    match result {
        Ok(Ok(_)) => {
            status["last_error"] = Value::Null;
            status["generation"] = json!(status["generation"].as_u64().unwrap_or(0) + 1);
        }
        error => {
            let message = format!("{error:?}");
            eprintln!("Build failed; preserving last successful assets: {message}");
            status["last_error"] = json!(message);
        }
    }
}

fn validate_proxy_target(target: &str) -> Result<()> {
    use url_encoding::address::{Host, Url};

    let url = Url::parse(target)?;
    let loopback = match url.host() {
        Some(Host::Domain(host)) => host == "localhost",
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    ensure!(
        url.scheme() == "http"
            && loopback
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "Development proxy target must be an unauthenticated loopback HTTP origin"
    );
    Ok(())
}

pub async fn serve(
    options: Options,
    bind: SocketAddr,
    overrides: Vec<String>,
    watch: bool,
    skip_build: bool,
) -> Result<()> {
    ensure!(
        bind.ip().is_loopback(),
        "Development credentials/API proxy bind must remain on loopback"
    );
    let config = Config::read(&options.root.join(layout::WEB))?;
    let mut proxies = config.web.proxies;
    for value in overrides {
        let (prefix, target) = value
            .split_once('=')
            .context("Proxy must be /prefix=http://127.0.0.1:PORT")?;
        proxies.insert(prefix.into(), target.into());
    }
    for (prefix, target) in &proxies {
        ensure!(
            prefix.starts_with('/') && !prefix.contains(['?', '#']),
            "Proxy prefix must be an absolute path"
        );
        validate_proxy_target(target)?;
    }
    let mut proxies = proxies.into_iter().collect::<Vec<_>>();
    proxies.sort_by_key(|(prefix, _)| std::cmp::Reverse(prefix.len()));
    let status = Arc::new(RwLock::new(
        json!({"building":false,"generation":0,"last_error":null}),
    ));
    let dist = crate::distribution(&options)?;
    if !skip_build {
        rebuild(options.clone(), status.clone()).await;
    }
    if watch {
        let status = status.clone();
        let options = options.clone();
        tokio::spawn(async move {
            let mut previous = inventory::signature(&options.root).map_err(|e| format!("{e:#}"));
            let mut pending = None;
            loop {
                tokio::time::sleep(Duration::from_millis(400)).await;
                let root = options.root.clone();
                let current = tokio::task::spawn_blocking(move || {
                    inventory::signature(&root).map_err(|e| format!("{e:#}"))
                })
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
                if current != previous {
                    previous = current;
                    pending = Some(std::time::Instant::now());
                } else if pending.is_some_and(|time| time.elapsed() >= Duration::from_millis(300)) {
                    pending = None;
                    rebuild(options.clone(), status.clone()).await;
                    previous = inventory::signature(&options.root).map_err(|e| format!("{e:#}"));
                }
            }
        });
    }
    let app = App {
        dist,
        base: config.web.base,
        proxies,
        status,
        client: proxy::Client::new(Duration::from_secs(300)),
    };
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!(
        "{}",
        json!({"event":"listening","address":format!("http://{}",listener.local_addr()?),"status":"/__rust_web/status"})
    );
    axum::serve(listener, Router::new().fallback(handler).with_state(app))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/dev.rs"]
mod tests;
