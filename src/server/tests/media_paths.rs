//! Linux filesystem consumers retain strict URL/path boundaries after pruning.
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use tower_http::services::ServeDir;

#[tokio::test]
async fn cloned_media_service_preserves_unicode_and_rejects_path_escapes() {
    let directory = tempfile::tempdir().unwrap();
    let media = directory.path().join("media");
    tokio::fs::create_dir(&media).await.unwrap();
    tokio::fs::write(directory.path().join("outside.txt"), b"private")
        .await
        .unwrap();
    tokio::fs::write(media.join("你好.txt"), b"public")
        .await
        .unwrap();
    // Linux permits this filename, but URL delivery deliberately rejects
    // backslashes rather than giving clients/platforms different path meanings.
    tokio::fs::write(media.join("a\\b.txt"), b"ambiguous")
        .await
        .unwrap();
    let service = ServeDir::new(&media);
    for path in [
        "/../outside.txt",
        "/%2e%2e/outside.txt",
        "/a%5Cb.txt",
        "/%00",
    ] {
        let response = service
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
    let response = service
        .clone()
        .oneshot(
            Request::builder()
                .uri("/%E4%BD%A0%E5%A5%BD.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(Body::new(response.into_body()), 1024)
            .await
            .unwrap(),
        "public"
    );
}
