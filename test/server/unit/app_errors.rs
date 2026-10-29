use super::*;
use std::error::Error as _;

#[tokio::test]
async fn media_parse_failures_retain_sources_without_echoing_values() {
    let json_error = serde_json::from_str::<Vec<String>>(r#"["private-cover",false]"#).unwrap_err();
    let url_error = url_encoding::address::Url::parse("http://[private-host]").unwrap_err();
    let errors = [
        AppError::MediaCovers(json_error),
        AppError::MediaUrl(url_error),
    ];
    for error in errors {
        assert!(error.source().is_some());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["code"], "bad_request");
        assert!(
            !String::from_utf8(bytes.to_vec())
                .unwrap()
                .contains("private")
        );
    }
}

#[tokio::test]
async fn database_failures_retain_the_source_and_return_a_safe_response() {
    let error = AppError::from(sqlx::Error::Io(std::io::Error::other(
        "private-database-detail",
    )));
    assert!(matches!(
        error.source().unwrap().downcast_ref::<sqlx::Error>(),
        Some(sqlx::Error::Io(_))
    ));
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response.headers()[axum::http::header::CACHE_CONTROL],
        "no-store, private, max-age=0"
    );
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope["code"], "internal_error");
    assert_eq!(envelope["message"], "数据库操作失败");
    assert!(
        !String::from_utf8(bytes.to_vec())
            .unwrap()
            .contains("private-database-detail")
    );
}

#[tokio::test]
async fn failed_workers_keep_the_join_error_without_exposing_panic_details() {
    let source = tokio::spawn(async { panic!("private-worker-detail") })
        .await
        .unwrap_err();
    let error = AppError::Task {
        message: "媒体保存失败",
        source,
    };
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<tokio::task::JoinError>()
            .unwrap()
            .is_panic()
    );
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope["message"], "媒体保存失败");
    assert!(
        !String::from_utf8(bytes.to_vec())
            .unwrap()
            .contains("private-worker-detail")
    );
}

#[tokio::test]
async fn database_readiness_reports_closed_pool_without_exposing_a_diagnostic() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    assert!(database_ready(&pool).await);
    pool.close().await;
    assert!(!database_ready(&pool).await);
    let response = readiness(State(pool)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        serde_json::json!({"ready": false})
    );
}
