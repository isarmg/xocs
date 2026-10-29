use super::*;
use tower::ServiceExt;

#[tokio::test]
#[ignore = "manual request-boundary throughput measurement"]
async fn measure_nested_request_boundaries() {
    let router = Router::new()
        .route(
            "/",
            get(|| async {
                (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "code": "conflict", "message": "The operation conflicts.",
                        "retryable": false, "request_id": null, "details": {}
                    })),
                )
            }),
        )
        .layer(axum::middleware::from_fn(
            crate::server_cli::request_context_middleware,
        ));
    for (name, app) in [
        (
            "router",
            tower::util::BoxCloneSyncService::new(router.clone()),
        ),
        ("transport_and_router", request_service(router)),
    ] {
        let started = std::time::Instant::now();
        for _ in 0..5_000 {
            let response = app
                .clone()
                .oneshot(
                    Request::get("/")
                        .header("x-request-id", "measure-123")
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::CONFLICT);
            let bytes = axum::body::to_bytes(response.into_body(), 16 * 1024)
                .await
                .unwrap();
            std::hint::black_box(bytes);
        }
        eprintln!("{name}: 5000 requests in {:?}", started.elapsed());
    }
}
struct Ready;
#[async_trait]
impl HealthCheck for Ready {
    async fn check(&self) -> bool {
        true
    }
}
fn descriptor() -> ProductDescriptor {
    ProductDescriptor {
        id: "example".into(),
        version: "1.0.0".into(),
        core_source_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .into(),
        profile: "server-control-plane".into(),
        capabilities: vec!["server-runtime".into()],
    }
}

#[tokio::test]
async fn readiness_requires_checks_and_critical_exit_stops_runtime() {
    let runtime = ServerRuntime::builder(descriptor())
        .register_health_check("database", Arc::new(Ready))
        .register_background_task("worker", TaskCriticality::Critical, |_shutdown| async {
            Err(std::io::Error::other("stopped"))
        })
        .build()
        .await
        .unwrap();
    runtime.refresh_health().await;
    assert!(runtime.handle().health().await.ready);
    let handle = runtime.handle();
    runtime.run_until_shutdown().await;
    let health = handle.health().await;
    assert!(!health.live && !health.ready);
}

#[test]
fn request_ids_are_bounded_random_hex() {
    let id = new_request_id();
    assert_eq!(id.as_str().len(), 32);
    assert!(id.as_str().bytes().all(|b| b.is_ascii_hexdigit()));
    let encoded = serde_json::to_string(&id).unwrap();
    let decoded: RequestId = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, id);
}

#[tokio::test]
async fn request_id_boundary_preserves_the_maximum_valid_identifier() {
    let request_id = format!("{}._:-", "A".repeat(crate::error::MAX_REQUEST_ID_BYTES - 4));
    let expected = request_id.clone();
    let app = request_service(Router::new().route(
        "/",
        get(move |request: Request| async move {
            let id = request.extensions().get::<RequestId>().unwrap();
            assert_eq!(id.as_str(), expected);
            assert_eq!(request.headers()["x-request-id"], expected);
            StatusCode::NO_CONTENT
        }),
    ));
    let response = app
        .oneshot(
            Request::get("/")
                .header("x-request-id", &request_id)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.headers()["x-request-id"], request_id);
}

#[tokio::test]
async fn request_policy_injects_one_id_and_redacts_handler_panics() {
    let app = Router::new()
        .route(
            "/echo",
            get(|headers: axum::http::HeaderMap| async move {
                headers["x-request-id"].to_str().unwrap().to_owned()
            }),
        )
        .route(
            "/panic",
            get(|| async {
                panic!("password=secret /private/database.sqlite");
                #[allow(unreachable_code)]
                StatusCode::OK
            }),
        )
        .layer(axum::middleware::from_fn(
            crate::server_cli::request_context_middleware,
        ));
    let app = request_service(app);
    let response = app
        .clone()
        .oneshot(
            Request::get("/echo")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let id = response.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), id.as_bytes());
    let response = app
        .oneshot(
            Request::get("/panic")
                .header("x-request-id", "trace-123")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.headers()["x-request-id"], "trace-123");
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let error: crate::contracts::ErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.code.as_str(), "platform.internal");
    assert_eq!(error.request_id.unwrap().as_str(), "trace-123");
    assert!(!String::from_utf8_lossy(&body).contains("secret"));
}

#[tokio::test]
async fn invalid_and_duplicate_ids_use_strict_error_envelopes() {
    let app = Router::new().route("/", get(|| async { StatusCode::OK }));
    let app = request_service(app);
    for duplicate in [false, true] {
        let mut request = Request::get("/").body(axum::body::Body::empty()).unwrap();
        // A public ID extension is not proof that the wire headers passed the
        // outer boundary; standalone router callers cannot bypass that policy.
        request
            .extensions_mut()
            .insert(RequestId::new("untrusted-extension").unwrap());
        request.headers_mut().insert(
            "x-request-id",
            HeaderValue::from_static(if duplicate {
                "valid-123"
            } else {
                "contains spaces"
            }),
        );
        if duplicate {
            request
                .headers_mut()
                .append("x-request-id", HeaderValue::from_static("valid-123"));
        }
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let id = response.headers()["x-request-id"]
            .to_str()
            .unwrap()
            .to_owned();
        let body = axum::body::to_bytes(response.into_body(), 8192)
            .await
            .unwrap();
        let error: crate::contracts::ErrorEnvelope = serde_json::from_slice(&body).unwrap();
        assert_eq!(error.request_id.unwrap().as_str(), id);
        assert_eq!(
            error.code.as_str(),
            if duplicate {
                "request.duplicate_id"
            } else {
                "request.invalid_id"
            }
        );
    }
}

#[tokio::test(start_paused = true)]
async fn product_error_correlation_is_consistent_and_bounded() {
    let app = request_service(
        Router::new()
            .route(
                "/error",
                get(|request: Request| async move {
                    assert_eq!(
                        request
                            .extensions()
                            .get::<crate::contracts::RequestId>()
                            .unwrap()
                            .as_str(),
                        "trace-123"
                    );
                    (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({
                            "code": "conflict", "message": "The operation conflicts.",
                            "retryable": false, "request_id": "stale-id", "details": {}
                        })),
                    )
                        .into_response()
                }),
            )
            .route(
                "/large",
                get(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        [("content-type", "application/json")],
                        "x".repeat(16 * 1024 + 1),
                    )
                        .into_response()
                }),
            )
            .route(
                "/stalled",
                get(|| async {
                    let body = axum::body::Body::from_stream(futures::stream::pending::<
                        Result<Vec<u8>, std::convert::Infallible>,
                    >());
                    (
                        StatusCode::BAD_REQUEST,
                        [("content-type", "application/json")],
                        body,
                    )
                        .into_response()
                }),
            )
            .route(
                "/other",
                get(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"business": "value"})),
                    )
                        .into_response()
                }),
            )
            .layer(axum::middleware::from_fn(
                crate::server_cli::request_context_middleware,
            )),
    );
    for path in ["/error", "/large", "/stalled", "/other"] {
        let response = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("x-request-id", "trace-123")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()["x-request-id"], "trace-123");
        let status = response.status();
        if path != "/other" {
            assert!(
                response.headers()["cache-control"]
                    .to_str()
                    .unwrap()
                    .split(',')
                    .any(|directive| directive.trim() == "no-store")
            );
        }
        let bytes = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        if path == "/other" {
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
                serde_json::json!({"business": "value"})
            );
        } else {
            let error: crate::contracts::ErrorEnvelope = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(error.request_id.unwrap().as_str(), "trace-123");
            assert_eq!(
                status,
                if path == "/error" {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            );
        }
    }
}

#[test]
fn core_identity_requires_a_full_source_sha256() {
    let mut product = descriptor();
    assert!(product.validate().is_ok());
    product.core_source_sha256.truncate(7);
    assert!(product.validate().is_err());
    product.core_source_sha256 = "A".repeat(64);
    assert!(product.validate().is_err());
    product = descriptor();
    product.id = "p".repeat(crate::log::MAX_IDENTIFIER_BYTES + 1);
    assert!(product.validate().is_err());
}

#[tokio::test]
async fn shutdown_before_subscribing_is_retained() {
    let runtime = ServerRuntime::builder(descriptor()).build().await.unwrap();
    let handle = runtime.handle();
    handle.shutdown();
    let mut signal = handle.shutdown_signal();
    tokio::time::timeout(Duration::from_secs(1), wait_for_shutdown(&mut signal))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(1), runtime.run_until_shutdown())
        .await
        .unwrap();
    assert!(!handle.health().await.ready);
}

#[tokio::test(start_paused = true)]
async fn uncooperative_worker_is_reported_without_aborting_at_the_deadline() {
    let runtime = ServerRuntime::builder(descriptor())
        .register_background_task("stalled", TaskCriticality::Critical, |_| {
            std::future::pending()
        })
        .build()
        .await
        .unwrap();
    let handle = runtime.handle();
    handle.shutdown();
    let listeners = BoundListeners::bind(["127.0.0.1:0".parse().unwrap()]).unwrap();
    let transport = HttpServer::new(listeners, ProcessSignals::install().unwrap());
    let result = runtime.serve(transport, Router::new()).await;
    let Err(Error::ShutdownIncomplete(failure)) = result else {
        panic!("hard deadline must fail")
    };
    assert!(!failure.report.clean);
    assert_eq!(failure.report.unfinished_ordinary_work, 1);
    assert_eq!(
        handle.diagnostics().await.tasks["stalled"].state,
        TaskState::Running
    );
    assert!(!handle.health().await.ready);
}

#[tokio::test]
async fn http_lifecycle_observes_shutdown_requested_before_start() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let runtime = ServerRuntime::builder(descriptor()).build().await.unwrap();
    runtime.handle().shutdown();
    tokio::time::timeout(
        Duration::from_secs(2),
        runtime.serve(
            HttpServer::new(
                BoundListeners::from_listener(listener).unwrap(),
                ProcessSignals::install().unwrap(),
            ),
            Router::new(),
        ),
    )
    .await
    .unwrap()
    .unwrap();
}

#[tokio::test]
async fn http_lifecycle_propagates_critical_worker_failure() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let runtime = ServerRuntime::builder(descriptor())
        .register_background_task("failed", TaskCriticality::Critical, |_| async {
            Err(std::io::Error::other("failure"))
        })
        .build()
        .await
        .unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        runtime.serve(
            HttpServer::new(
                BoundListeners::from_listener(listener).unwrap(),
                ProcessSignals::install().unwrap(),
            ),
            Router::new(),
        ),
    )
    .await
    .unwrap();
    assert!(matches!(result, Err(Error::CriticalTaskStopped)));
}

#[tokio::test]
async fn panicking_task_factory_is_recorded_and_stops_critical_runtime() {
    let runtime = ServerRuntime::builder(descriptor())
        .register_background_task("factory", TaskCriticality::Critical, |_| {
            panic!("factory panic");
            #[allow(unreachable_code)]
            std::future::ready(Ok(()))
        })
        .build()
        .await
        .unwrap();
    let handle = runtime.handle();
    tokio::time::timeout(Duration::from_secs(1), runtime.run_until_shutdown())
        .await
        .unwrap();
    assert_eq!(
        handle.diagnostics().await.tasks["factory"].state,
        TaskState::Panicked
    );
    assert!(!handle.health().await.live);
}

#[tokio::test]
async fn degrading_and_best_effort_exits_do_not_stop_the_service() {
    let runtime = ServerRuntime::builder(descriptor())
        .register_background_task("maintenance", TaskCriticality::Degrading, |_| async {
            Err(std::io::Error::other("failure"))
        })
        .register_background_task("optional", TaskCriticality::BestEffort, |_| async {
            Err(std::io::Error::other("failure"))
        })
        .build()
        .await
        .unwrap();
    let handle = runtime.handle();
    let task = tokio::spawn(runtime.run_until_shutdown());
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let diagnostic = handle.diagnostics().await;
            if diagnostic
                .tasks
                .values()
                .all(|task| task.state == TaskState::Failed)
            {
                assert!(
                    diagnostic.health.ready && diagnostic.health.live && diagnostic.health.degraded
                );
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(!task.is_finished());
    handle.shutdown();
    task.await.unwrap();
}

#[tokio::test]
async fn slow_or_panicking_checks_fail_closed_without_locking_diagnostics() {
    let runtime = ServerRuntime::builder(descriptor())
        .register_health_check("stalled", health_check(std::future::pending))
        .register_health_check("panic", health_check(|| async { panic!("check panic") }))
        .build()
        .await
        .unwrap();
    let handle = runtime.handle();
    let refresh = runtime.refresh_health();
    tokio::pin!(refresh);
    tokio::select! {
        () = &mut refresh => panic!("stalled check completed without its deadline"),
        () = tokio::time::sleep(Duration::from_millis(10)) => {}
    }
    let health = tokio::time::timeout(Duration::from_millis(100), handle.health())
        .await
        .unwrap();
    assert!(!health.ready);
    tokio::time::timeout(HEALTH_CHECK_TIMEOUT + Duration::from_secs(1), refresh)
        .await
        .unwrap();
    assert!(
        handle
            .diagnostics()
            .await
            .checks
            .values()
            .all(|ready| !ready)
    );
}

#[tokio::test]
async fn patch_release_reports_its_version_while_retaining_the_data_format_identity() {
    let mut product = descriptor();
    product.version = "1.0.1".into();
    let schema =
        crate::schema_identity::SchemaIdentity::new("example", "1.0.0", 1, "a".repeat(64)).unwrap();
    let runtime = ServerRuntime::builder(product)
        .with_schema_identity(schema.clone())
        .build()
        .await
        .unwrap();
    let diagnostics = runtime.handle().diagnostics().await;
    assert_eq!(diagnostics.product.version, "1.0.1");
    assert_eq!(diagnostics.schema_identity, Some(schema));
}

#[tokio::test]
async fn schema_must_match_product_and_health_checks_are_unique() {
    let invalid = ServerRuntime::builder(descriptor())
        .with_schema_identity(
            crate::schema_identity::SchemaIdentity::new("different", "1.0.0", 1, "a".repeat(64))
                .unwrap(),
        )
        .build()
        .await;
    assert!(matches!(
        invalid,
        Err(Error::InvalidDescriptor("schema_identity"))
    ));
    let duplicate = ServerRuntime::builder(descriptor())
        .register_health_check("database", Arc::new(Ready))
        .register_health_check("database", Arc::new(Ready))
        .build()
        .await;
    assert!(matches!(duplicate, Err(Error::DuplicateName(_))));
}

#[tokio::test]
async fn diagnostics_are_absent_for_both_anonymous_and_authenticated_requests() {
    use crate::admin_core::AdministratorService;
    use axum::{body::Body, http::header};
    let store = crate::testkit::administrator_fixture("admin", "correct horse battery").await;
    let runtime = ServerRuntime::builder(descriptor())
        .with_schema_identity(
            crate::schema_identity::SchemaIdentity::new("example", "1.0.0", 3, "a".repeat(64))
                .unwrap(),
        )
        .register_health_check("database", health_check(|| async { true }))
        .build()
        .await
        .unwrap();
    runtime.refresh_health().await;
    let app = platform_router(
        runtime.handle(),
        "example",
        crate::admin_auth::AdministratorOriginMode::LoopbackDevelopmentHttp,
        Arc::new(AdministratorService::with_isolated_test_budget(store)),
    )
    .unwrap();
    let live = app
        .clone()
        .oneshot(Request::get(LIVENESS_PATH).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(live.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        live.headers()[axum::http::header::CACHE_CONTROL],
        "no-store"
    );
    assert!(
        axum::body::to_bytes(live.into_body(), 8192)
            .await
            .unwrap()
            .is_empty()
    );
    let ready = app
        .clone()
        .oneshot(Request::get(READINESS_PATH).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        ready.headers()[axum::http::header::CACHE_CONTROL],
        "no-store"
    );
    let bytes = axum::body::to_bytes(ready.into_body(), 8192).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        serde_json::json!({"ready": true})
    );
    let anonymous = app
        .clone()
        .oneshot(
            Request::get("/api/v1/platform/diagnostics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::NOT_FOUND);
    let login = Request::post(crate::contracts::ADMIN_LOGIN_PATH)
        .header(header::HOST, "127.0.0.1")
        .header(header::ORIGIN, "http://127.0.0.1")
        .header("sec-fetch-site", "same-origin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"username":"admin","password":"correct horse battery"}"#,
        ))
        .unwrap();
    let response = app.clone().oneshot(login).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let diagnostic = app
        .oneshot(
            Request::get("/api/v1/platform/diagnostics")
                .header(header::COOKIE, cookie)
                .header("x-request-id", "diag-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(diagnostic.status(), StatusCode::NOT_FOUND);
}
