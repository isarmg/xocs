use super::*;

#[test]
fn proxy_accepts_both_loopback_address_families() {
    for target in [
        "http://localhost:8081",
        "http://127.0.0.1:8081",
        "http://[::1]:8081",
    ] {
        assert!(validate_proxy_target(target).is_ok(), "{target}");
    }
    for target in [
        "http://192.0.2.1:8081",
        "http://[2001:db8::1]:8081",
        "http://[::]:8081",
        "http://0.0.0.0:8081",
        "http://example.com",
        "https://localhost:8081",
        "http://user:secret@[::1]:8081",
        "http://[::1]:8081/?query=1",
        "http://[::1]:8081/#fragment",
    ] {
        assert!(validate_proxy_target(target).is_err(), "{target}");
    }
}

#[tokio::test]
async fn static_head_etag_and_traversal() -> Result<()> {
    use tower::ServiceExt;
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("a.js"), "export const a=1")?;
    let video = b"\0\0\0\x18ftypisom\0\0\0\0isommp42";
    std::fs::write(dir.path().join("clip.mp4"), video)?;
    let app = App {
        dist: dir.path().into(),
        base: "/".into(),
        proxies: Vec::new(),
        client: proxy::Client::new(Duration::from_secs(300)),
        status: Arc::new(RwLock::new(json!({}))),
    };
    let router = Router::new().fallback(handler).with_state(app);
    let first = router
        .clone()
        .oneshot(Request::builder().uri("/a.js").body(Body::empty())?)
        .await?;
    assert_eq!(first.status(), StatusCode::OK);
    let etag = first.headers()[header::ETAG].clone();
    let head = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri("/a.js")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(head.headers()[header::CONTENT_LENGTH], "16");
    let cached = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/a.js")
                .header(header::IF_NONE_MATCH, etag)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(cached.status(), StatusCode::NOT_MODIFIED);
    let media = router
        .clone()
        .oneshot(Request::builder().uri("/clip.mp4").body(Body::empty())?)
        .await?;
    assert_eq!(media.status(), StatusCode::OK);
    assert_eq!(media.headers()[header::CONTENT_TYPE], "video/mp4");
    assert_eq!(media.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(
        media.headers()[header::CONTENT_LENGTH],
        video.len().to_string()
    );
    assert_eq!(
        axum::body::to_bytes(media.into_body(), 1024)
            .await?
            .as_ref(),
        video
    );
    let escaped = router
        .oneshot(
            Request::builder()
                .uri("/%2e%2e/private")
                .body(Body::empty())?,
        )
        .await?;
    assert_ne!(escaped.status(), StatusCode::OK);
    Ok(())
}
#[tokio::test]
async fn wasm_encoding_refusal_never_falls_back_to_forbidden_identity() -> Result<()> {
    use tower::ServiceExt;
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("app.wasm"), b"raw")?;
    std::fs::write(dir.path().join("app.wasm.gz"), b"compressed")?;
    let app = App {
        dist: dir.path().into(),
        base: "/".into(),
        proxies: Vec::new(),
        client: proxy::Client::new(Duration::from_secs(300)),
        status: Arc::new(RwLock::new(json!({}))),
    };
    let router = Router::new().fallback(handler).with_state(app);
    for method in [Method::GET, Method::HEAD] {
        for (encoding, status, compressed) in [
            ("gzip;q=0, identity;q=0", StatusCode::NOT_ACCEPTABLE, false),
            ("*;q=0, identity;q=1", StatusCode::OK, false),
            ("gzip;q=0, gzip;q=1", StatusCode::OK, false),
            ("gzip, identity;q=0", StatusCode::OK, true),
        ] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri("/app.wasm")
                        .header(header::ACCEPT_ENCODING, encoding)
                        .body(Body::empty())?,
                )
                .await?;
            assert_eq!(response.status(), status, "{encoding}");
            assert_eq!(response.headers()[header::VARY], "Accept-Encoding");
            assert_eq!(
                response.headers().contains_key(header::CONTENT_ENCODING),
                compressed
            );
            let body = axum::body::to_bytes(response.into_body(), 1024).await?;
            if method == Method::HEAD || status == StatusCode::NOT_ACCEPTABLE {
                assert!(body.is_empty());
            } else {
                assert_eq!(
                    body.as_ref(),
                    if compressed {
                        b"compressed".as_slice()
                    } else {
                        b"raw".as_slice()
                    }
                );
            }
        }
    }
    std::fs::remove_file(dir.path().join("app.wasm.gz"))?;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/app.wasm")
                .header(header::ACCEPT_ENCODING, "gzip, identity;q=0")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
    let missing = router
        .oneshot(
            Request::builder()
                .uri("/missing.wasm")
                .header(header::ACCEPT_ENCODING, "*;q=0")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    Ok(())
}
