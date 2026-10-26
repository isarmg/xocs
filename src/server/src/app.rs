use crate::{PRODUCT_ID, content, home, media, schema, static_assets};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use blog_application::admin_core::AdministratorService;
use blog_application::admin_sqlite::SqliteAdministratorStore;
use blog_application::{admin_auth::AdministratorOriginMode, error::HttpStatus};
use sqlx::SqlitePool;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) scope: blog_application::server_runtime::WorkScope,
    pub(crate) pool: SqlitePool,
    pub(crate) admin: Arc<AdministratorService<SqliteAdministratorStore>>,
    pub(crate) origin: AdministratorOriginMode,
    pub(crate) media: PathBuf,
    pub(crate) media_operations: Arc<tokio::sync::Mutex<()>>,
    pub(crate) public_mutations: Arc<PublicMutationCapacity>,
    pub(crate) password_slots: Arc<tokio::sync::Semaphore>,
}

/// Concurrent write and bounded-wait capacity, independent of request identity.
pub(crate) struct PublicMutationCapacity {
    active: Arc<tokio::sync::Semaphore>,
    admitted: Arc<tokio::sync::Semaphore>,
}
impl Default for PublicMutationCapacity {
    fn default() -> Self {
        Self {
            active: Arc::new(tokio::sync::Semaphore::new(4)),
            admitted: Arc::new(tokio::sync::Semaphore::new(36)),
        }
    }
}
async fn admit_public_mutation(
    State(capacity): State<Arc<PublicMutationCapacity>>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() != axum::http::Method::POST {
        return next.run(request).await;
    }
    let Ok(_admission) = capacity.admitted.clone().try_acquire_owned() else {
        return AppError::Public(
            HttpStatus::ServiceUnavailable,
            "服务写入容量已满，请稍后重试",
        )
        .into_response();
    };
    let Ok(Ok(_execution)) = tokio::time::timeout(
        Duration::from_secs(10),
        capacity.active.clone().acquire_owned(),
    )
    .await
    else {
        return AppError::Public(
            HttpStatus::ServiceUnavailable,
            "服务写入暂时繁忙，请稍后重试",
        )
        .into_response();
    };
    next.run(request).await
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{1}")]
    Public(HttpStatus, &'static str),
    #[error("数据库操作失败")]
    Database(#[from] sqlx::Error),
    #[error("数据库暂时不可用")]
    DatabaseUnavailable(#[source] sqlx::Error),
    #[error("媒体文件操作失败")]
    Filesystem(#[from] blog_application::fs_safety::Error),
    #[error("服务正在停止")]
    Stopping(#[from] blog_application::server_runtime::WorkClosed),
    #[error("{message}")]
    Task {
        message: &'static str,
        #[source]
        source: tokio::task::JoinError,
    },
    #[error("{message}")]
    Busy {
        message: &'static str,
        #[source]
        source: tokio::time::error::Elapsed,
    },
    #[error("文章密码应为 12 至 1024 字节且不含控制字符")]
    InvalidPassword(#[source] blog_application::admin_auth::Error),
    #[error("密码计算失败")]
    Password(#[from] blog_application::admin_auth::Error),
    #[error("文件名编码无效")]
    FileNameHeader(#[source] axum::http::header::ToStrError),
    #[error("文件名编码无效")]
    FileNameEncoding(#[source] std::str::Utf8Error),
    #[error("信笺存储内容无效")]
    StoredLetter(#[source] serde_json::Error),
    #[error("媒体删除日志名称无效")]
    InvalidDeletionJournal,
}
impl AppError {
    fn status(&self) -> HttpStatus {
        match self {
            Self::Public(status, _) => *status,
            Self::Stopping(_) | Self::Busy { .. } | Self::DatabaseUnavailable(_) => {
                HttpStatus::ServiceUnavailable
            }
            Self::StoredLetter(_) => HttpStatus::Conflict,
            Self::FileNameHeader(_) | Self::FileNameEncoding(_) | Self::InvalidPassword(_) => {
                HttpStatus::BadRequest
            }
            Self::Database(_)
            | Self::Filesystem(_)
            | Self::Task { .. }
            | Self::Password(_)
            | Self::InvalidDeletionJournal => HttpStatus::Internal,
        }
    }

    pub(crate) fn envelope(&self) -> blog_application::error::ErrorEnvelope {
        blog_application::error::ErrorEnvelope::with_code(
            self.status().error_code(),
            self.to_string(),
        )
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        use std::error::Error as _;
        if self.source().is_some() {
            tracing::error!(error = ?self, "request operation failed");
        }
        let envelope = self.envelope();
        let status = StatusCode::from_u16(self.status().status())
            .expect("invariant: HTTP error statuses are valid");
        let mut response = (status, Json(envelope)).into_response();
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store, private, max-age=0"),
        );
        response
    }
}
pub(crate) type ApiResult<T> = Result<Json<T>, AppError>;

pub(crate) fn db_error(error: sqlx::Error) -> AppError {
    AppError::Database(error)
}
pub(crate) fn invalid(message: &'static str) -> AppError {
    AppError::Public(HttpStatus::BadRequest, message)
}
pub(crate) fn absent() -> AppError {
    AppError::Public(HttpStatus::NotFound, "内容不存在")
}
pub(crate) async fn site_author_id<'executor, E>(executor: E) -> Result<i64, AppError>
where
    E: sqlx::Executor<'executor, Database = sqlx::Sqlite>,
{
    sqlx::query_scalar(
        "SELECT id FROM user WHERE user_type=0 AND deleted=0 AND user_status=1 ORDER BY id LIMIT 1",
    )
    .fetch_optional(executor)
    .await
    .map_err(db_error)?
    .ok_or(AppError::Public(HttpStatus::Conflict, "缺少站点作者账号"))
}
#[cfg(any(test, feature = "maintenance"))]
pub(crate) fn now_micros() -> Result<u64, crate::Error> {
    use std::time::{SystemTime, UNIX_EPOCH};
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_micros()
        .try_into()?)
}

pub(crate) fn router(state: AppState, web: Option<PathBuf>) -> Result<Router, crate::Error> {
    let admin_routes = content::admin_routes()
        .merge(media::admin_routes())
        .merge(home::admin_routes())
        .route_layer(from_fn_with_state(state.clone(), admin_gate))
        .with_state(state.clone());
    let public_routes = content::public_routes()
        .merge(home::public_routes())
        .route_layer(from_fn_with_state(
            state.public_mutations.clone(),
            admit_public_mutation,
        ))
        .with_state(state.clone());
    let web_routes = if let Some(web) = web {
        let entry = web.join("index.html");
        let mut routes = Router::new().fallback_service(ServeDir::new(web));
        for path in PAGE_ROUTES {
            routes = routes.route_service(path, ServeFile::new(&entry));
        }
        routes
    } else {
        let mut routes = Router::new().fallback(static_assets::serve);
        for path in PAGE_ROUTES {
            routes = routes.route(path, get(static_assets::entry));
        }
        routes
    };
    Ok(web_routes
        .route("/readyz", get(readiness).with_state(state.pool.clone()))
        .route("/api/v1/health", get(health).with_state(state.pool.clone()))
        .merge(public_routes)
        .nest_service("/media", ServeDir::new(&state.media))
        .merge(admin_routes)
        .merge(blog_application::admin_axum::administrator_router(
            PRODUCT_ID,
            state.origin,
            Arc::clone(&state.admin),
        )?)
        .route(
            "/api/{*path}",
            get(|| async { absent() }).fallback(|| async { absent() }),
        )
        .method_not_allowed_fallback(|| async {
            (
                StatusCode::METHOD_NOT_ALLOWED,
                Json(blog_application::error::ErrorEnvelope::with_code(
                    blog_application::error::ErrorCode::new("method_not_allowed")
                        .expect("static code"),
                    "The request method is not supported by this route.",
                )),
            )
        })
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    let request_id = request
                        .extensions()
                        .get::<blog_application::contracts::RequestId>()
                        .map(|value| value.as_str())
                        .unwrap_or("");
                    tracing::info_span!("http.request", request_id)
                })
                .on_request(())
                .on_response(
                    |response: &Response, duration: Duration, _: &tracing::Span| {
                        tracing::info!(
                            event = "blog.http.completed",
                            component = "http",
                            status = u64::from(response.status().as_u16()),
                            duration_ms = duration.as_millis().min(u128::from(u64::MAX)) as u64
                        );
                    },
                )
                .on_body_chunk(())
                .on_eos(())
                .on_failure(()),
        )
        .layer(from_fn_with_state(state.scope.clone(), admit_request))
        .layer(from_fn_with_state(
            PRODUCT_ID.to_string(),
            blog_application::server_cli::service_identity_middleware,
        ))
        .layer(axum::middleware::from_fn(
            blog_application::server_cli::request_context_middleware,
        )))
}

async fn admit_request(
    State(scope): State<blog_application::server_runtime::WorkScope>,
    request: Request,
    next: Next,
) -> Response {
    if request
        .uri()
        .path()
        .strip_prefix("/media/")
        .is_some_and(|name| !media::is_media_filename(name))
    {
        return absent().into_response();
    }
    let Some(_admission) = scope.enter_request().await else {
        return AppError::Public(HttpStatus::ServiceUnavailable, "服务正在停止").into_response();
    };
    next.run(request).await
}

const PAGE_ROUTES: &[&str] = &[
    "/admin",
    "/sort",
    "/search",
    "/jotting",
    "/menory",
    "/favorite",
    "/travel",
    "/love",
    "/message",
    "/letter",
    "/article/{id}",
];

async fn health(State(pool): State<SqlitePool>) -> Response {
    match sqlx::query("SELECT 1")
        .execute(&pool)
        .await
    {
        Ok(_) => {
            Json(serde_json::json!({"status":"ready","application":PRODUCT_ID,"schema_identity":schema::current_identity()})).into_response()
        }
        Err(error) => AppError::DatabaseUnavailable(error).into_response(),
    }
}

async fn readiness(State(pool): State<SqlitePool>) -> Response {
    let ready = database_ready(&pool).await;
    (
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(serde_json::json!({"ready":ready})),
    )
        .into_response()
}

#[derive(Debug)]
enum DatabaseProbeFailure {
    Database,
    Io,
    PoolClosed,
    PoolTimedOut,
    WorkerCrashed,
    Driver,
}

pub(crate) async fn database_ready(pool: &SqlitePool) -> bool {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => true,
        Err(error) => {
            let (failure, io_kind, database_kind) = match &error {
                sqlx::Error::Database(source) => {
                    (DatabaseProbeFailure::Database, None, Some(source.kind()))
                }
                sqlx::Error::Io(source) => (DatabaseProbeFailure::Io, Some(source.kind()), None),
                sqlx::Error::PoolClosed => (DatabaseProbeFailure::PoolClosed, None, None),
                sqlx::Error::PoolTimedOut => (DatabaseProbeFailure::PoolTimedOut, None, None),
                sqlx::Error::WorkerCrashed => (DatabaseProbeFailure::WorkerCrashed, None, None),
                _ => (DatabaseProbeFailure::Driver, None, None),
            };
            tracing::warn!(
                ?failure,
                ?io_kind,
                ?database_kind,
                "database health probe failed"
            );
            false
        }
    }
}

async fn admin_gate(State(state): State<AppState>, request: Request, next: Next) -> Response {
    match blog_application::admin_axum::authenticate_request(
        &state.admin,
        request.headers(),
        request.uri(),
        request.method(),
        PRODUCT_ID,
        state.origin,
    )
    .await
    {
        Ok(_) => next.run(request).await,
        Err(response) => *response,
    }
}

#[cfg(test)]
#[path = "../../../test/server/unit/app_errors.rs"]
mod error_tests;
