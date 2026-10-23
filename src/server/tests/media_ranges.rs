//! Exercise byte-range parsing through the same file service used for /media.

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;

#[tokio::test]
async fn empty_media_never_advertises_a_nonexistent_byte() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("empty.bin");
    tokio::fs::write(&path, []).await.unwrap();

    for method in [Method::GET, Method::HEAD] {
        for range in ["bytes=0-0", "bytes=0-", "bytes=-1"] {
            let response = ServeFile::new(&path)
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .header(header::RANGE, range)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if method == Method::GET {
                assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
                assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes */0");
            } else {
                assert_eq!(response.status(), StatusCode::OK);
                assert!(!response.headers().contains_key(header::CONTENT_RANGE));
                assert_eq!(response.headers()[header::CONTENT_LENGTH], "0");
            }
        }
    }

    let response = ServeFile::new(&path)
        .oneshot(Request::new(Body::empty()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_LENGTH], "0");
}

#[tokio::test]
async fn oversized_suffix_serves_the_entire_media_representation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("media.bin");
    let data = b"small media fixture";
    tokio::fs::write(&path, data).await.unwrap();

    for method in [Method::GET, Method::HEAD] {
        let response = ServeFile::new(&path)
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .header(header::RANGE, "bytes=-18446744073709551615")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if method == Method::GET {
            assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
            assert_eq!(
                response.headers()[header::CONTENT_RANGE],
                format!("bytes 0-{}/{}", data.len() - 1, data.len())
            );
        } else {
            assert_eq!(response.status(), StatusCode::OK);
            assert!(!response.headers().contains_key(header::CONTENT_RANGE));
        }
        assert_eq!(
            response.headers()[header::CONTENT_LENGTH],
            data.len().to_string()
        );
        let body = to_bytes(Body::new(response.into_body()), 1024)
            .await
            .unwrap();
        if method == Method::GET {
            assert_eq!(body.as_ref(), data);
        } else {
            assert!(body.is_empty());
        }
    }
}

#[tokio::test]
async fn head_ignores_invalid_and_partial_ranges() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("media.bin");
    tokio::fs::write(&path, b"fixture").await.unwrap();

    for range in ["bytes=0-1", "bytes=999-", "invalid", "bytes=0-1,3-4"] {
        let response = ServeFile::new(&path)
            .oneshot(
                Request::builder()
                    .method(Method::HEAD)
                    .header(header::RANGE, range)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "7");
        assert!(!response.headers().contains_key(header::CONTENT_RANGE));
        let body = to_bytes(Body::new(response.into_body()), 1024)
            .await
            .unwrap();
        assert!(body.is_empty());
    }
}
