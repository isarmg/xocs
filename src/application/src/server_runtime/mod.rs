//! Shared server lifecycle and health primitives. Product routers remain outside this crate.

mod http1;
mod listeners;
mod shutdown;
mod tasks;
mod transport;
pub use listeners::BoundListeners;
pub use shutdown::{
    LifecycleParticipant, ProcessSignals, ShutdownFailure, ShutdownLimits, ShutdownPhase,
    ShutdownReport,
};
pub use tasks::{TrackedTasks, WorkClosed, WorkScope};
pub use transport::{Http1Limits, HttpServer};

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::{Request, State as AxumState},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use futures::FutureExt;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::sync::{RwLock, watch};

pub const LIVENESS_PATH: &str = "/healthz";
pub const READINESS_PATH: &str = "/readyz";
/// Exact platform roots/subtrees that filesystem consumers must reserve.
/// This does not reserve unrelated files beneath `/api` or `/api/v1`.
pub const PLATFORM_RESERVED_PATHS: &[&str] = &[
    LIVENESS_PATH,
    READINESS_PATH,
    "/api/v1/auth",
    crate::contracts::ADMINISTRATORS_PATH,
];
pub const DEFAULT_REQUEST_BODY_BYTES: usize = 1024 * 1024;
pub const HEALTH_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);
pub const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProductDescriptor {
    pub id: String,
    pub version: String,
    pub core_source_sha256: String,
    pub profile: String,
    pub capabilities: Vec<String>,
}

impl ProductDescriptor {
    pub fn validate(&self) -> Result<(), Error> {
        for (label, value) in [
            ("id", &self.id),
            ("version", &self.version),
            ("profile", &self.profile),
        ] {
            if value.is_empty()
                || value.len() > crate::log::MAX_IDENTIFIER_BYTES
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
            {
                return Err(Error::InvalidDescriptor(label));
            }
        }
        let bound = self.core_source_sha256.len() == 64
            && self
                .core_source_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'));
        if !bound {
            return Err(Error::InvalidDescriptor("core_source_sha256"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCriticality {
    Critical,
    Degrading,
    BestEffort,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Running,
    Stopped,
    Completed,
    Failed,
    Panicked,
    Aborted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TaskSnapshot {
    pub criticality: TaskCriticality,
    pub state: TaskState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HealthSnapshot {
    pub live: bool,
    pub ready: bool,
    pub degraded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostics {
    pub request_id: Option<String>,
    pub schema_identity: Option<crate::schema_identity::SchemaIdentity>,
    pub metrics: BTreeMap<DiagnosticMetric, Option<u64>>,
    pub product: ProductDescriptor,
    pub health: HealthSnapshot,
    pub checks: BTreeMap<String, bool>,
    pub tasks: BTreeMap<String, TaskSnapshot>,
}

/// Closed, numeric diagnostic fields: no arbitrary path, secret or error text
/// can be supplied by a product probe. Null means unsupported or unavailable.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticMetric {
    AuditBacklog,
    OperationBacklog,
    SpoolPendingBytes,
    SpoolPendingRecords,
}

fn empty_metrics() -> BTreeMap<DiagnosticMetric, Option<u64>> {
    [
        DiagnosticMetric::AuditBacklog,
        DiagnosticMetric::OperationBacklog,
        DiagnosticMetric::SpoolPendingBytes,
        DiagnosticMetric::SpoolPendingRecords,
    ]
    .into_iter()
    .map(|metric| (metric, None))
    .collect()
}

#[async_trait]
trait MetricProbe: Send + Sync {
    async fn read(&self) -> Option<u64>;
}

struct ClosureMetricProbe<F>(F);

#[async_trait]
impl<F, Fut> MetricProbe for ClosureMetricProbe<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Option<u64>> + Send,
{
    async fn read(&self) -> Option<u64> {
        (self.0)().await
    }
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> bool;
}

struct ClosureHealthCheck<F>(F);

#[async_trait]
impl<F, Fut> HealthCheck for ClosureHealthCheck<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = bool> + Send,
{
    async fn check(&self) -> bool {
        (self.0)().await
    }
}

pub fn health_check<F, Fut>(check: F) -> Arc<dyn HealthCheck>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = bool> + Send + 'static,
{
    Arc::new(ClosureHealthCheck(check))
}

#[derive(Clone)]
pub struct RuntimeHandle {
    state: Arc<RwLock<RuntimeState>>,
    shutdown: watch::Sender<bool>,
}

impl RuntimeHandle {
    pub fn shutdown(&self) {
        // Retain the request even before the runtime or HTTP listener subscribes.
        self.shutdown.send_replace(true);
    }
    pub fn shutdown_signal(&self) -> watch::Receiver<bool> {
        self.shutdown.subscribe()
    }
    pub async fn health(&self) -> HealthSnapshot {
        let mut health = self.state.read().await.health();
        health.ready &= !*self.shutdown.borrow();
        health
    }
    pub async fn diagnostics(&self) -> Diagnostics {
        self.state.read().await.diagnostics()
    }
}

struct RuntimeState {
    product: ProductDescriptor,
    schema_identity: Option<crate::schema_identity::SchemaIdentity>,
    metrics: BTreeMap<DiagnosticMetric, Option<u64>>,
    checks: BTreeMap<String, bool>,
    tasks: BTreeMap<String, TaskSnapshot>,
    shutting_down: bool,
}

impl RuntimeState {
    fn health(&self) -> HealthSnapshot {
        let critical_failed = self.tasks.values().any(|t| {
            t.criticality == TaskCriticality::Critical
                && !matches!(t.state, TaskState::Running | TaskState::Stopped)
        });
        let degraded = self.tasks.values().any(|t| {
            t.criticality == TaskCriticality::Degrading
                && !matches!(t.state, TaskState::Running | TaskState::Stopped)
        });
        HealthSnapshot {
            live: !critical_failed,
            ready: !self.shutting_down
                && !critical_failed
                && self.checks.values().all(|ready| *ready),
            degraded,
        }
    }
    fn diagnostics(&self) -> Diagnostics {
        Diagnostics {
            request_id: None,
            schema_identity: self.schema_identity.clone(),
            metrics: self.metrics.clone(),
            product: self.product.clone(),
            health: self.health(),
            checks: self.checks.clone(),
            tasks: self.tasks.clone(),
        }
    }
}

pub struct ServerRuntime {
    handle: RuntimeHandle,
    checks: Vec<(String, Arc<dyn HealthCheck>)>,
    metrics: Vec<(DiagnosticMetric, Arc<dyn MetricProbe>)>,
    tasks: Vec<TaskRegistration>,
}
struct TaskRegistration {
    name: String,
    criticality: TaskCriticality,
    task: Box<SupervisedTask>,
}
type SupervisedTask = dyn FnOnce(watch::Receiver<bool>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
    + Send;

pub struct ServerRuntimeBuilder {
    product: ProductDescriptor,
    schema_identity: Option<crate::schema_identity::SchemaIdentity>,
    metrics: Vec<(DiagnosticMetric, Arc<dyn MetricProbe>)>,
    checks: Vec<(String, Arc<dyn HealthCheck>)>,
    tasks: Vec<TaskRegistration>,
}

impl ServerRuntime {
    pub fn builder(product: ProductDescriptor) -> ServerRuntimeBuilder {
        ServerRuntimeBuilder {
            product,
            schema_identity: None,
            metrics: vec![],
            checks: vec![],
            tasks: vec![],
        }
    }
    pub fn handle(&self) -> RuntimeHandle {
        self.handle.clone()
    }

    pub async fn run_until_shutdown(mut self) {
        let mut runtime_shutdown = self.handle.shutdown_signal();
        let joins = tokio_util::task::TaskTracker::new();
        for registration in self.tasks.drain(..) {
            let state = self.handle.state.clone();
            let shutdown = self.handle.shutdown_signal();
            let shutdown_requested = shutdown.clone();
            let task_handle = self.handle.clone();
            joins.spawn(async move {
                let outcome =
                    std::panic::AssertUnwindSafe(
                        async move { (registration.task)(shutdown).await },
                    )
                    .catch_unwind()
                    .await;
                let task_state = match outcome {
                    Ok(Ok(())) if *shutdown_requested.borrow() => TaskState::Stopped,
                    Ok(Ok(())) => TaskState::Completed,
                    Ok(Err(_)) => TaskState::Failed,
                    Err(_) => TaskState::Panicked,
                };
                let mut guard = state.write().await;
                if let Some(snapshot) = guard.tasks.get_mut(&registration.name) {
                    snapshot.state = task_state;
                }
                if !guard.health().live {
                    task_handle.shutdown();
                }
            });
        }
        self.refresh_health().await;
        let mut health_interval = tokio::time::interval(HEALTH_REFRESH_INTERVAL);
        health_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            if *runtime_shutdown.borrow() {
                break;
            }
            tokio::select! {
                _ = wait_for_shutdown(&mut runtime_shutdown) => break,
                _ = health_interval.tick() => self.refresh_health().await,
            }
        }
        self.handle.state.write().await.shutting_down = true;
        self.handle.shutdown();
        // The process HTTP owner enforces the hard deadline. Dropping its
        // waiter must not abort tasks that still hold persistent obligations.
        joins.close();
        joins.wait().await;
    }

    pub fn refresh_health(&self) -> impl Future<Output = ()> + Send + use<> {
        let checks = self.checks.clone();
        let metrics = self.metrics.clone();
        let handle = self.handle.clone();
        async move {
            let checks = checks.into_iter().map(|(name, check)| async move {
                let ready = tokio::time::timeout(
                    HEALTH_CHECK_TIMEOUT,
                    std::panic::AssertUnwindSafe(async { check.check().await }).catch_unwind(),
                )
                .await
                .is_ok_and(|result| matches!(result, Ok(true)));
                (name, ready)
            });
            let metrics = metrics.into_iter().map(|(name, probe)| async move {
                let value = tokio::time::timeout(
                    HEALTH_CHECK_TIMEOUT,
                    std::panic::AssertUnwindSafe(async { probe.read().await }).catch_unwind(),
                )
                .await
                .ok()
                .and_then(Result::ok)
                .flatten();
                (name, value)
            });
            let (checks, metrics) = tokio::join!(
                futures::future::join_all(checks),
                futures::future::join_all(metrics),
            );
            let mut state = handle.state.write().await;
            for (name, ready) in checks {
                state.checks.insert(name, ready);
            }
            for (name, value) in metrics {
                state.metrics.insert(name, value);
            }
        }
    }
}

/// Observe both a retained shutdown request and a later notification. A closed
/// channel also stops the consumer; no task may outlive its runtime owner.
pub async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    loop {
        if *shutdown.borrow_and_update() || shutdown.changed().await.is_err() {
            return;
        }
    }
}

/// Create a route-only runtime handle. Servers that already own their socket
/// lifecycle can adopt the xcss HTTP surface before moving workers into
/// [`ServerRuntime`]. No background task is left unsupervised by this helper.
pub fn platform_handle(product: ProductDescriptor) -> Result<RuntimeHandle, Error> {
    product.validate()?;
    let (shutdown, _) = watch::channel(false);
    Ok(RuntimeHandle {
        state: Arc::new(RwLock::new(RuntimeState {
            product,
            schema_identity: None,
            metrics: empty_metrics(),
            checks: BTreeMap::new(),
            tasks: BTreeMap::new(),
            shutting_down: false,
        })),
        shutdown,
    })
}

impl ServerRuntimeBuilder {
    pub fn with_schema_identity(
        mut self,
        identity: crate::schema_identity::SchemaIdentity,
    ) -> Self {
        self.schema_identity = Some(identity);
        self
    }

    pub fn register_metric<F, Fut>(mut self, metric: DiagnosticMetric, read: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<u64>> + Send + 'static,
    {
        self.metrics
            .push((metric, Arc::new(ClosureMetricProbe(read))));
        self
    }

    pub fn register_health_check(
        mut self,
        name: impl Into<String>,
        check: Arc<dyn HealthCheck>,
    ) -> Self {
        self.checks.push((name.into(), check));
        self
    }
    pub fn register_background_task<F, Fut>(
        mut self,
        name: impl Into<String>,
        criticality: TaskCriticality,
        task: F,
    ) -> Self
    where
        F: FnOnce(watch::Receiver<bool>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        self.tasks.push(TaskRegistration {
            name: name.into(),
            criticality,
            task: Box::new(move |shutdown| Box::pin(task(shutdown))),
        });
        self
    }
    pub async fn build(self) -> Result<ServerRuntime, Error> {
        self.product.validate()?;
        if let Some(schema) = &self.schema_identity
            && (schema.validate().is_err() || schema.application != self.product.id)
        {
            return Err(Error::InvalidDescriptor("schema_identity"));
        }
        let mut seen_metrics = std::collections::BTreeSet::new();
        for (metric, _) in &self.metrics {
            if !seen_metrics.insert(*metric) {
                return Err(Error::DuplicateName(format!("{metric:?}")));
            }
        }
        let mut task_states = BTreeMap::new();
        for task in &self.tasks {
            if task_states
                .insert(
                    task.name.clone(),
                    TaskSnapshot {
                        criticality: task.criticality,
                        state: TaskState::Running,
                    },
                )
                .is_some()
            {
                return Err(Error::DuplicateName(task.name.clone()));
            }
        }
        let mut check_states = BTreeMap::new();
        for (name, _) in &self.checks {
            if check_states.insert(name.clone(), false).is_some() {
                return Err(Error::DuplicateName(name.clone()));
            }
        }
        let (shutdown, _) = watch::channel(false);
        Ok(ServerRuntime {
            handle: RuntimeHandle {
                state: Arc::new(RwLock::new(RuntimeState {
                    product: self.product,
                    schema_identity: self.schema_identity,
                    metrics: empty_metrics(),
                    checks: check_states,
                    tasks: task_states,
                    shutting_down: false,
                })),
                shutdown,
            },
            checks: self.checks,
            metrics: self.metrics,
            tasks: self.tasks,
        })
    }
}

pub fn new_request_id() -> String {
    static FALLBACK_COUNTER: AtomicU64 = AtomicU64::new(1);
    let mut bytes = [0_u8; 16];
    if getrandom::fill(&mut bytes).is_err() {
        let counter = FALLBACK_COUNTER.fetch_add(1, Ordering::Relaxed);
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        bytes[..8].copy_from_slice(&counter.to_be_bytes());
        bytes[8..].copy_from_slice(&(elapsed as u64).to_be_bytes());
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Clone)]
struct PlatformState {
    handle: RuntimeHandle,
}

/// Compose the xcss-owned auth, health and readiness routes.
/// Request IDs are installed once by `serve`, outside product URI validation.
/// No administrator diagnostics endpoint is provided. Product routes must be merged separately.
pub fn platform_router<Store>(
    handle: RuntimeHandle,
    product_id: impl Into<String>,
    mode: crate::admin_auth::AdministratorOriginMode,
    administrator: Arc<crate::admin_core::AdministratorService<Store>>,
) -> Result<Router, crate::admin_core::Error>
where
    Store: crate::admin_core::AdministratorStore + 'static,
{
    let product_id = product_id.into();
    let auth = crate::admin_axum::administrator_router(
        product_id.clone(),
        mode,
        Arc::clone(&administrator),
    )?;
    let state = PlatformState { handle };
    let runtime = Router::new()
        .route(LIVENESS_PATH, get(liveness))
        .route(READINESS_PATH, get(readiness))
        .with_state(state);
    Ok(Router::new().merge(auth).merge(runtime))
}

async fn liveness(AxumState(state): AxumState<PlatformState>) -> Response {
    let status = if state.handle.health().await.live {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, [(axum::http::header::CACHE_CONTROL, "no-store")]).into_response()
}

async fn readiness(AxumState(state): AxumState<PlatformState>) -> Response {
    let ready = state.handle.health().await.ready;
    (
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"ready": ready})),
    )
        .into_response()
}

fn request_error(status: StatusCode, code: &'static str, request_id: &str) -> Response {
    let mut response = (
        status,
        Json(serde_json::json!({
            "code": code,
            "message": code.replace('.', " "),
            "request_id": request_id,
            "retryable": false,
            "details": {},
        })),
    )
        .into_response();
    if let Ok(value) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, private, max-age=0"),
    );
    response
        .extensions_mut()
        .insert(crate::admin_axum::BlogErrorResponse);
    response
}

pub fn request_service<S>(
    inner: S,
) -> tower::util::BoxCloneSyncService<Request, Response, std::convert::Infallible>
where
    S: tower::Service<Request, Response = Response, Error = std::convert::Infallible>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send,
{
    tower::util::BoxCloneSyncService::new(tower::service_fn(move |request| {
        let inner = inner.clone();
        async move { Ok::<_, std::convert::Infallible>(request_id_boundary(request, inner).await) }
    }))
}

async fn request_id_boundary<S>(mut request: Request, inner: S) -> Response
where
    S: tower::Service<Request, Response = Response, Error = std::convert::Infallible>,
{
    use tower::ServiceExt;
    static HEADER: axum::http::HeaderName = axum::http::HeaderName::from_static("x-request-id");
    let request_id = match request
        .headers()
        .get_all(&HEADER)
        .iter()
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => new_request_id(),
        [value] => match value.to_str() {
            Ok(value) if crate::contracts::RequestId::new(value).is_ok() => value.to_owned(),
            _ => {
                return request_error(
                    StatusCode::BAD_REQUEST,
                    "request.invalid_id",
                    &new_request_id(),
                );
            }
        },
        _ => {
            return request_error(
                StatusCode::BAD_REQUEST,
                "request.duplicate_id",
                &new_request_id(),
            );
        }
    };
    request.extensions_mut().insert(request_id.clone());
    request.extensions_mut().insert(
        crate::contracts::RequestId::new(request_id.clone()).expect("validated request ID"),
    );
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        request.headers_mut().insert(HEADER.clone(), value);
    }
    let mut response = match std::panic::AssertUnwindSafe(inner.oneshot(request))
        .catch_unwind()
        .await
    {
        Ok(Ok(response)) => response,
        Ok(Err(never)) => match never {},
        Err(_) => request_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "platform.internal",
            &request_id,
        ),
    };
    response = correlate_error_response(response, &request_id).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(HEADER.clone(), value);
    }
    response
}

/// Correlation of flat JSON errors is bounded even when a product accidentally
/// returns a streaming error. Success/download bodies never enter this path.
async fn correlate_error_response(response: Response, request_id: &str) -> Response {
    use axum::http::header;
    const MAX_ERROR_BYTES: usize = 16 * 1024;
    if !(response.status().is_client_error() || response.status().is_server_error())
        || response.headers().contains_key(header::CONTENT_ENCODING)
        || !response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|value| {
                value.to_str().is_ok_and(|value| {
                    value
                        .split(';')
                        .next()
                        .is_some_and(|mime| mime.trim() == "application/json")
                })
            })
    {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    let bytes = match tokio::time::timeout(
        Duration::from_secs(1),
        axum::body::to_bytes(body, MAX_ERROR_BYTES),
    )
    .await
    {
        Ok(Ok(bytes)) => bytes,
        _ => {
            return request_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "platform.internal",
                request_id,
            );
        }
    };
    let Ok(mut error) = serde_json::from_slice::<crate::contracts::ErrorEnvelope>(&bytes) else {
        return Response::from_parts(parts, axum::body::Body::from(bytes));
    };
    error.request_id =
        Some(crate::contracts::RequestId::new(request_id).expect("validated request ID"));
    let Ok(bytes) = serde_json::to_vec(&error) else {
        return request_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "platform.internal",
            request_id,
        );
    };
    if bytes.len() > MAX_ERROR_BYTES {
        return request_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "platform.internal",
            request_id,
        );
    }
    parts.headers.remove(header::CONTENT_LENGTH);
    parts
        .headers
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Response::from_parts(parts, axum::body::Body::from(bytes))
}

pub fn install_panic_hook() {
    // Panic payloads may include credentials, file names, or request contents.
    std::panic::set_hook(Box::new(|_| eprintln!("runtime panic; task terminated")));
}

pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid product descriptor field: {0}")]
    InvalidDescriptor(&'static str),
    #[error("duplicate runtime registration: {0}")]
    DuplicateName(String),
    #[error("HTTP server failed")]
    Http(#[from] std::io::Error),
    #[error("critical background task stopped unexpectedly")]
    CriticalTaskStopped,
    #[error("all HTTP listeners stopped unexpectedly")]
    ListenersStopped,
    #[error("product state could not close cleanly")]
    StateCloseFailed,
    #[error(transparent)]
    ShutdownIncomplete(#[from] ShutdownFailure),
}

#[path = "../../../../test/application/unit/server_runtime/tests.rs"]
#[cfg(test)]
mod tests;
