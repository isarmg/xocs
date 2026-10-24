use crate::{PRODUCT_ID, admin, content, home, media, schema, static_assets};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use blog_application::admin_auth::AdministratorOriginMode;
use blog_application::admin_core::AdministratorService;
use blog_application::admin_sqlite::SqliteAdministratorStore;
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
        return AppError(
            StatusCode::SERVICE_UNAVAILABLE,
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
        return AppError(
            StatusCode::SERVICE_UNAVAILABLE,
            "服务写入暂时繁忙，请稍后重试",
        )
        .into_response();
    };
    next.run(request).await
}

#[derive(Debug)]
pub(crate) struct AppError(pub(crate) StatusCode, pub(crate) &'static str);
impl AppError {
    pub(crate) fn envelope(&self) -> blog_application::error::ErrorEnvelope {
        let code = match self.0 {
            StatusCode::BAD_REQUEST => "bad_request",
            StatusCode::UNAUTHORIZED => "unauthorized",
            StatusCode::FORBIDDEN => "forbidden",
            StatusCode::NOT_FOUND => "not_found",
            StatusCode::CONFLICT => "conflict",
            StatusCode::PAYLOAD_TOO_LARGE => "payload_too_large",
            StatusCode::TOO_MANY_REQUESTS => "too_many_requests",
            StatusCode::SERVICE_UNAVAILABLE => "service_unavailable",
            _ => "internal_error",
        };
        blog_application::error::ErrorEnvelope::with_code(
            blog_application::error::ErrorCode::new(code).expect("static error code"),
            self.1,
        )
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let envelope = self.envelope();
        let mut response = (self.0, Json(envelope)).into_response();
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store, private, max-age=0"),
        );
        response
    }
}
pub(crate) type ApiResult<T> = Result<Json<T>, AppError>;

pub(crate) fn db_error(error: sqlx::Error) -> AppError {
    tracing::error!(%error, "database operation failed");
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败")
}
pub(crate) fn invalid(message: &'static str) -> AppError {
    AppError(StatusCode::BAD_REQUEST, message)
}
pub(crate) fn absent() -> AppError {
    AppError(StatusCode::NOT_FOUND, "内容不存在")
}
pub(crate) async fn site_author_id(pool: &SqlitePool) -> Result<i64, AppError> {
    sqlx::query_scalar(
        "SELECT id FROM user WHERE user_type=0 AND deleted=0 AND user_status=1 ORDER BY id LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::CONFLICT, "缺少站点作者账号"))
}
#[cfg(any(test, feature = "maintenance"))]
pub(crate) fn now_micros() -> anyhow::Result<u64> {
    use std::time::{SystemTime, UNIX_EPOCH};
    Ok(u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros(),
    )?)
}

pub(crate) fn router(state: AppState, web: Option<PathBuf>) -> anyhow::Result<Router> {
    let admin_routes = content::admin_routes()
        .merge(admin::routes())
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
        return AppError(StatusCode::SERVICE_UNAVAILABLE, "服务正在停止").into_response();
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
    match sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(1) => Json(serde_json::json!({"status":"ready","application":PRODUCT_ID,"schema_identity":schema::current_identity().expect("compiled schema identity")})).into_response(),
        _ => AppError(StatusCode::SERVICE_UNAVAILABLE, "数据库暂时不可用").into_response(),
    }
}

async fn readiness(State(pool): State<SqlitePool>) -> Response {
    let ready = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&pool)
        .await
        .is_ok_and(|value| value == 1);
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
