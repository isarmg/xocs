//! Local development uses Axum's HTTP implementation and a bounded, streaming
//! HTTP/1 proxy. Credentials, Host and Origin are forwarded without rewriting.
use crate::layout;
use crate::{Options, assets::Config, inventory};
pub use proxy::{ProxyError, ProxyHop};

#[derive(Debug, thiserror::Error)]
pub enum DevError {
    #[error(transparent)]
    Configuration(#[from] crate::assets::AssetError),
    #[error(transparent)]
    Distribution(#[from] crate::DistributionError),
    #[error(transparent)]
    Source(#[from] inventory::InventoryError),
    #[error(transparent)]
    Proxy(#[from] ProxyError),
    #[error("build preview HTTP response")]
    Response(#[from] axum::http::Error),
    #[error("parse proxy request URI")]
    RequestUri(#[source] axum::http::uri::InvalidUri),
    #[error("decode static resource path")]
    PathEncoding(#[source] std::str::Utf8Error),
    #[error("invalid static resource path")]
    InvalidPath,
    #[error("WebSocket proxying is not configured; use the backend WebSocket origin directly")]
    WebSocket,
    #[error("parse development proxy origin")]
    ProxyOrigin(#[source] url_encoding::address::ParseError),
    #[error("development proxy target must be an unauthenticated loopback HTTP origin")]
    UnsafeProxyOrigin,
    #[error("development credentials/API proxy bind must remain on loopback")]
    NonLoopbackBind,
    #[error("proxy must be /prefix=http://127.0.0.1:PORT")]
    InvalidProxyOverride,
    #[error("proxy prefix must be an absolute path")]
    InvalidProxyPrefix,
    #[error("bind development listener at {address}")]
    Bind {
        address: SocketAddr,
        #[source]
        source: std::io::Error,
    },
    #[error("inspect development listener")]
    ListenerAddress(#[source] std::io::Error),
    #[error("serve development HTTP requests")]
    Serve(#[source] std::io::Error),
    #[error("development asset reader worker failed")]
    ReadWorker(#[source] tokio::task::JoinError),
    #[error("development source watcher failed")]
    Watcher(#[source] tokio::task::JoinError),
    #[error("development watcher shutdown failed after {operation}")]
    Shutdown {
        operation: Box<Self>,
        #[source]
        watcher: tokio::task::JoinError,
    },
}

type Result<T> = std::result::Result<T, DevError>;
use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode, header},
    response::Response,
};
use serde_json::json;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;

mod proxy;

const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

struct App {
    dist: PathBuf,
    base: String,
    proxies: Vec<(String, String)>,
    client: proxy::Client,
    status: Arc<RwLock<BuildStatus>>,
}

#[derive(Default, serde::Serialize)]
struct BuildStatus {
    building: bool,
    generation: u64,
    last_error: Option<String>,
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

async fn handler(State(app): State<Arc<App>>, request: Request) -> Response {
    match respond(app, request).await {
        Ok(response) => response,
        Err(error) => {
            eprintln!("development request failed: {}", crate::error_chain(&error));
            plain(
                StatusCode::BAD_GATEWAY,
                "Development request failed; see the local build log\n",
            )
        }
    }
}

async fn respond(app: Arc<App>, request: Request) -> Result<Response> {
    let path = request.uri().path();
    if path == "/__rust_web/status" {
        return Ok(Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CACHE_CONTROL, "no-store")
            .body(Body::from(
                serde_json::to_vec(&*app.status.read().await)
                    .expect("invariant: typed development status serializes to JSON"),
            ))?);
    }
    if let Some((prefix, target)) = app.proxies.iter().find(|(prefix, _)| {
        path == prefix
            || path
                .strip_prefix(prefix.trim_end_matches('/'))
                .is_some_and(|suffix| suffix.starts_with('/'))
    }) {
        if request.headers().contains_key(header::UPGRADE) {
            return Err(DevError::WebSocket);
        }
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
        return Ok(app
            .client
            .forward(request, url.parse().map_err(DevError::RequestUri)?)
            .await?);
    }
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(header::ALLOW, "GET, HEAD")
            .body(Body::empty())?);
    }
    let decoded = url_encoding::percent::percent_decode_str(path)
        .decode_utf8()
        .map_err(DevError::PathEncoding)?;
    if decoded.contains(['\\', '\0']) {
        return Err(DevError::InvalidPath);
    }
    let relative = decoded
        .strip_prefix(&app.base)
        .unwrap_or(decoded.trim_start_matches('/'));
    let mut source = app.dist.join(if relative.is_empty() {
        "index.html"
    } else {
        relative
    });
    inventory::relative(
        source
            .strip_prefix(&app.dist)
            .expect("invariant: source was joined below the distribution"),
    )?;
    let wasm = source.extension().is_some_and(|value| value == "wasm");
    let accepted = header::accepted_encodings(request.headers());
    let compressed_path = PathBuf::from(format!("{}.gz", source.display()));
    let compressed_body = if wasm && accepted.gzip {
        read_resource(&compressed_path).await?
    } else {
        None
    };
    let (body, compressed) = match compressed_body {
        Some(body) => (body, true),
        None => {
            let body = match read_resource(&source).await? {
                Some(body) => body,
                None if Path::new(relative).extension().is_some()
                    || relative.starts_with("assets/") =>
                {
                    return Ok(plain(StatusCode::NOT_FOUND, "Not found\n"));
                }
                None => {
                    source = app.dist.join("index.html");
                    let Some(body) = read_resource(&source).await? else {
                        return Ok(plain(
                            StatusCode::SERVICE_UNAVAILABLE,
                            "Frontend is compiling; inspect /__rust_web/status\n",
                        ));
                    };
                    body
                }
            };
            (body, false)
        }
    };
    if wasm && !compressed && !accepted.identity {
        return Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .header(header::CACHE_CONTROL, "no-store")
            .header(header::CONTENT_LENGTH, "0")
            .header(header::VARY, "Accept-Encoding")
            .body(Body::empty())?);
    }
    let html = source.extension().is_some_and(|value| value == "html");
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

async fn read_resource(path: &Path) -> Result<Option<Vec<u8>>> {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || match inventory::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(inventory::InventoryError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(None)
        }
        Err(error) => Err(DevError::Source(error)),
    })
    .await
    .map_err(DevError::ReadWorker)?
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuildOutcome {
    Settled,
    Retry,
}

#[derive(Debug, thiserror::Error)]
enum SignatureError {
    #[error("read development source snapshot")]
    Inventory(#[source] inventory::InventoryError),
    #[error("source snapshot worker failed")]
    Worker(#[source] tokio::task::JoinError),
}

#[derive(Clone, Debug)]
enum Signature {
    Ready(String),
    Unavailable(Arc<SignatureError>),
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ready(left), Self::Ready(right)) => left == right,
            // A readable snapshot resumes change detection after unavailable input.
            (Self::Unavailable(_), Self::Unavailable(_)) => true,
            (Self::Ready(_), Self::Unavailable(_)) | (Self::Unavailable(_), Self::Ready(_)) => {
                false
            }
        }
    }
}

struct WatchState {
    previous: Signature,
    pending: Option<std::time::Instant>,
}

impl WatchState {
    fn new(previous: Signature) -> Self {
        Self {
            previous,
            pending: None,
        }
    }

    fn observe(&mut self, current: Signature, now: std::time::Instant) -> bool {
        if matches!(&current, Signature::Unavailable(_)) {
            self.previous = current;
            self.pending = None;
            return false;
        }
        if current != self.previous {
            self.previous = current;
            self.pending = Some(now);
            false
        } else if self
            .pending
            .is_some_and(|time| now.duration_since(time) >= Duration::from_millis(300))
        {
            self.pending = None;
            true
        } else {
            false
        }
    }

    fn finished(&mut self, current: Signature, outcome: BuildOutcome, now: std::time::Instant) {
        if current != self.previous || outcome == BuildOutcome::Retry {
            self.previous = current;
            self.pending = Some(now);
        }
    }
}

async fn source_signature(root: PathBuf) -> Signature {
    match tokio::task::spawn_blocking(move || inventory::signature(&root)).await {
        Ok(Ok(signature)) => Signature::Ready(signature),
        Ok(Err(source)) => Signature::Unavailable(Arc::new(SignatureError::Inventory(source))),
        Err(source) => Signature::Unavailable(Arc::new(SignatureError::Worker(source))),
    }
}

#[derive(Debug, thiserror::Error)]
enum RebuildError {
    #[error(transparent)]
    Build(#[from] crate::BuildError),
    #[error("development build worker failed")]
    Worker(#[source] tokio::task::JoinError),
}

async fn rebuild(options: Options, status: Arc<RwLock<BuildStatus>>) -> BuildOutcome {
    status.write().await.building = true;
    let result = tokio::task::spawn_blocking(move || crate::build(&options))
        .await
        .map_err(RebuildError::Worker)
        .and_then(|result| result.map_err(RebuildError::Build));
    let outcome = if matches!(
        &result,
        Err(RebuildError::Build(crate::BuildError::Lock(
            crate::LockError::Busy { .. }
        )))
    ) {
        BuildOutcome::Retry
    } else {
        BuildOutcome::Settled
    };
    let mut status = status.write().await;
    status.building = false;
    match result {
        Ok(_) => {
            status.last_error = None;
            status.generation += 1;
        }
        Err(error) => {
            let message = crate::error_chain(&error);
            eprintln!("Build failed; preserving last successful assets: {message}");
            status.last_error = Some(message);
        }
    }
    outcome
}

fn validate_proxy_target(target: &str) -> Result<()> {
    use url_encoding::address::{Host, Url};

    let url = Url::parse(target).map_err(DevError::ProxyOrigin)?;
    let loopback = match url.host() {
        Some(Host::Domain(host)) => host == "localhost",
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    if url.scheme() != "http"
        || !loopback
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(DevError::UnsafeProxyOrigin);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchMode {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitialBuild {
    Build,
    ExistingAssets,
}

pub async fn serve(
    options: Options,
    bind: SocketAddr,
    overrides: Vec<String>,
    watch: WatchMode,
    initial: InitialBuild,
) -> Result<()> {
    if !bind.ip().is_loopback() {
        return Err(DevError::NonLoopbackBind);
    }
    let config = Config::read(&options.root.join(layout::WEB))?;
    let dist = crate::resolve_distribution(
        &options.root,
        options
            .dist
            .clone()
            .unwrap_or_else(|| config.web.dist.clone().into()),
    )?;
    let mut proxies = config.web.proxies;
    for value in overrides {
        let (prefix, target) = value
            .split_once('=')
            .ok_or(DevError::InvalidProxyOverride)?;
        proxies.insert(prefix.into(), target.into());
    }
    for (prefix, target) in &proxies {
        if !prefix.starts_with('/') || prefix.contains(['?', '#']) {
            return Err(DevError::InvalidProxyPrefix);
        }
        validate_proxy_target(target)?;
    }
    let mut proxies = proxies.into_iter().collect::<Vec<_>>();
    proxies.sort_by_key(|(prefix, _)| std::cmp::Reverse(prefix.len()));
    let status = Arc::new(RwLock::new(BuildStatus::default()));
    let mut watcher = if watch == WatchMode::Enabled {
        Some(WatchState::new(
            source_signature(options.root.clone()).await,
        ))
    } else {
        None
    };
    if initial == InitialBuild::ExistingAssets
        && let Some(WatchState {
            previous: Signature::Unavailable(error),
            ..
        }) = &watcher
    {
        let message = crate::error_chain(error.as_ref());
        eprintln!("Source scan failed; preserving existing assets: {message}");
        status.write().await.last_error = Some(message);
    }
    if initial == InitialBuild::Build {
        let outcome = rebuild(options.clone(), status.clone()).await;
        if let Some(watcher) = &mut watcher {
            watcher.finished(
                source_signature(options.root.clone()).await,
                outcome,
                std::time::Instant::now(),
            );
        }
    }
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .map_err(|source| DevError::Bind {
            address: bind,
            source,
        })?;
    let watcher_task = if let Some(mut watcher) = watcher {
        let status = status.clone();
        let options = options.clone();
        Some(tokio::spawn(async move {
            loop {
                if crate::process::cancelled() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(400)).await;
                if crate::process::cancelled() {
                    break;
                }
                let current = source_signature(options.root.clone()).await;
                if let Signature::Unavailable(error) = &current
                    && !matches!(watcher.previous, Signature::Unavailable(_))
                {
                    let message = crate::error_chain(error.as_ref());
                    eprintln!("Source scan failed; preserving last successful assets: {message}");
                    status.write().await.last_error = Some(message);
                }
                if watcher.observe(current, std::time::Instant::now()) {
                    let outcome = rebuild(options.clone(), status.clone()).await;
                    watcher.finished(
                        source_signature(options.root.clone()).await,
                        outcome,
                        std::time::Instant::now(),
                    );
                }
            }
        }))
    } else {
        None
    };
    let app = Arc::new(App {
        dist,
        base: config.web.base,
        proxies,
        status,
        client: proxy::Client::new(Duration::from_secs(300)),
    });
    println!(
        "{}",
        json!({"event":"listening","address":format!("http://{}",listener.local_addr().map_err(DevError::ListenerAddress)?),"status":"/__rust_web/status"})
    );
    let serving = async {
        axum::serve(listener, Router::new().fallback(handler).with_state(app))
            .with_graceful_shutdown(crate::process::cancellation())
            .await
    };
    tokio::pin!(serving);
    let result = tokio::select! {
        result = &mut serving => result.map_err(DevError::Serve),
        _ = crate::process::cancellation() => {
            match tokio::time::timeout(SHUTDOWN_GRACE, &mut serving).await {
                Ok(result) => result.map_err(DevError::Serve),
                // Closing the listener after this deadline terminates the remaining preview connections.
                Err(_elapsed) => Ok(()),
            }
        }
    };
    crate::process::cancel();
    if let Some(watcher) = watcher_task
        && let Err(watcher) = watcher.await
    {
        return Err(match result {
            Ok(()) => DevError::Watcher(watcher),
            Err(operation) => DevError::Shutdown {
                operation: Box::new(operation),
                watcher,
            },
        });
    }
    result
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/dev.rs"]
mod tests;
