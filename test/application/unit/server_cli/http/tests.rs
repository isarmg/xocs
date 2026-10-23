use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    number: u32,
}

async fn json(ContractJson(input): ContractJson<Input>) -> String {
    input.number.to_string()
}
async fn query(ContractQuery(input): ContractQuery<Input>) -> String {
    input.number.to_string()
}
async fn path(ContractPath(number): ContractPath<u32>) -> String {
    number.to_string()
}

#[tokio::test]
async fn extraction_errors_are_flat_safe_bounded_and_uncacheable() {
    let router = Router::new()
        .route("/json", post(json))
        .route("/query", get(query))
        .route("/path/{number}", get(path))
        .layer(DefaultBodyLimit::max(64));
    for (method, uri, content_type, body, status, code, reason) in [
        (
            "POST",
            "/json",
            Some("application/json"),
            r#"{"number":"SECRET"}"#,
            400,
            "contract_violation",
            "TYPE_MISMATCH",
        ),
        (
            "POST",
            "/json",
            Some("application/json"),
            r#"{"number":2,"SECRET":2}"#,
            400,
            "contract_violation",
            "UNKNOWN_FIELD",
        ),
        (
            "POST",
            "/json",
            Some("application/json"),
            "{}",
            400,
            "contract_violation",
            "MISSING_FIELD",
        ),
        (
            "POST",
            "/json",
            Some("application/json"),
            "[]",
            400,
            "contract_violation",
            "JSON_STRUCTURE",
        ),
        (
            "POST",
            "/json",
            Some("application/json"),
            r#"{"number":SECRET}"#,
            400,
            "contract_violation",
            "INVALID_JSON",
        ),
        (
            "POST",
            "/json",
            None,
            "SECRET",
            415,
            "contract_violation",
            "CONTENT_TYPE_REQUIRED",
        ),
        (
            "POST",
            "/json",
            Some("application/json"),
            "SECRETSECRETSECRETSECRETSECRETSECRETSECRETSECRETSECRETSECRETSECRET",
            413,
            "payload_too_large",
            "BODY_LIMIT",
        ),
        (
            "GET",
            "/query?number=SECRET",
            None,
            "",
            400,
            "contract_violation",
            "INVALID_QUERY",
        ),
        (
            "GET",
            "/path/SECRET",
            None,
            "",
            400,
            "contract_violation",
            "INVALID_PATH",
        ),
    ] {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(content_type) = content_type {
            builder = builder.header(header::CONTENT_TYPE, content_type);
        }
        let mut request = builder.body(Body::from(body)).unwrap();
        request.extensions_mut().insert("request-1".to_string());
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status().as_u16(), status);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        assert!(!std::str::from_utf8(&bytes).unwrap().contains("SECRET"));
        let error: ErrorEnvelope = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(error.code.as_str(), code);
        assert_eq!(error.details["reason"], reason);
        assert_eq!(error.request_id.unwrap().as_str(), "request-1");
    }
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/json")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"number":7}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn router_middleware_correlates_extractor_and_product_errors() {
    let router = Router::new()
        .route("/json", post(json))
        .layer(axum::middleware::from_fn(request_context_middleware));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/json")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-request-id", "trace-123")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.headers()["x-request-id"], "trace-123");
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    let error: ErrorEnvelope = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(error.request_id.unwrap().as_str(), "trace-123");
    assert_eq!(error.details["reason"], "MISSING_FIELD");
}

#[tokio::test(start_paused = true)]
async fn body_deadline_does_not_cancel_a_handler_after_input_was_received() {
    let router = Router::new().route(
        "/json",
        post(|ContractJson(input): ContractJson<Input>| async move {
            tokio::time::sleep(JSON_BODY_TIMEOUT * 2).await;
            input.number.to_string()
        }),
    );
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/json")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"number":7}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(to_bytes(response.into_body(), 64).await.unwrap(), "7");
}
