use super::*;
use anyhow::Result;

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
async fn edits_during_a_blocked_attempt_schedule_another_build_without_a_third_edit() -> Result<()>
{
    let repository = tempfile::tempdir()?;
    let source = repository.path().join("src/web/custom/entry.js");
    std::fs::create_dir_all(source.parent().unwrap())?;
    std::fs::write(&source, "first")?;
    let root = repository.path().to_owned();
    let mut watcher = WatchState::new(source_signature(root.clone()).await);
    let now = std::time::Instant::now();
    std::fs::write(&source, "second")?;
    let second = source_signature(root.clone()).await;
    assert!(!watcher.observe(second.clone(), now));
    assert!(watcher.observe(second, now + Duration::from_millis(400)));
    let (started, entered) = tokio::sync::oneshot::channel();
    let (resume, resumed) = tokio::sync::oneshot::channel();
    let attempt = tokio::spawn(async move {
        started.send(()).unwrap();
        resumed.await.unwrap();
        // Source-drift failure is settled; the changed signature itself must
        // cause the next attempt, independently of the error's wording.
        BuildOutcome::Settled
    });
    entered.await?;
    std::fs::write(&source, "third")?;
    let third = source_signature(root).await;
    resume.send(()).unwrap();
    watcher.finished(
        third.clone(),
        attempt.await?,
        now + Duration::from_millis(500),
    );
    assert!(watcher.observe(third.clone(), now + Duration::from_millis(900)));
    watcher.finished(
        third.clone(),
        BuildOutcome::Settled,
        now + Duration::from_millis(1000),
    );
    assert!(!watcher.observe(third, now + Duration::from_secs(2)));
    Ok(())
}

#[tokio::test]
async fn lock_contention_is_retryable_even_when_the_source_is_unchanged() -> Result<()> {
    let repository = tempfile::tempdir()?;
    let owner = crate::BuildLock::acquire(repository.path())?;
    let options = Options {
        root: repository.path().into(),
        offline: true,
        debug: true,
        dist: None,
    };
    let status = Arc::new(RwLock::new(BuildStatus::default()));
    let outcome = rebuild(options, status.clone()).await;
    assert!(outcome == BuildOutcome::Retry);
    assert_eq!(status.read().await.generation, 0);
    assert!(
        status
            .read()
            .await
            .last_error
            .as_deref()
            .unwrap()
            .contains("Another asset build owns")
    );
    let now = std::time::Instant::now();
    let mut watcher = WatchState::new(Signature::Ready("unchanged".into()));
    watcher.finished(Signature::Ready("unchanged".into()), outcome, now);
    drop(owner);
    assert!(watcher.observe(
        Signature::Ready("unchanged".into()),
        now + Duration::from_millis(400)
    ));
    Ok(())
}

#[test]
fn startup_edits_are_not_adopted_as_already_built() {
    let now = std::time::Instant::now();
    let mut watcher = WatchState::new(Signature::Ready("before initial build".into()));
    watcher.finished(
        Signature::Ready("edited during initial build".into()),
        BuildOutcome::Settled,
        now,
    );
    assert!(watcher.observe(
        Signature::Ready("edited during initial build".into()),
        now + Duration::from_millis(400)
    ));
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
        status: Arc::new(RwLock::new(BuildStatus::default())),
    };
    let router = Router::new().fallback(handler).with_state(Arc::new(app));
    let status = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/__rust_web/status")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(status.status(), StatusCode::OK);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &axum::body::to_bytes(status.into_body(), 1024).await?
        )?,
        json!({"building":false,"generation":0,"last_error":null})
    );
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
        status: Arc::new(RwLock::new(BuildStatus::default())),
    };
    let router = Router::new().fallback(handler).with_state(Arc::new(app));
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

#[test]
fn unavailable_sources_wait_for_recovery_without_scheduling_a_failed_build() {
    let missing = std::path::PathBuf::from("missing-source");
    let error = SignatureError::Inventory(inventory::InventoryError::Read {
        path: missing,
        source: std::io::Error::from(std::io::ErrorKind::NotFound),
    });
    let unavailable = Signature::Unavailable(Arc::new(error));
    let now = std::time::Instant::now();
    let mut watcher = WatchState::new(Signature::Ready("previous".into()));
    assert!(!watcher.observe(unavailable.clone(), now));
    assert!(!watcher.observe(unavailable, now + Duration::from_secs(1)));
    assert!(!watcher.observe(
        Signature::Ready("recovered".into()),
        now + Duration::from_secs(2)
    ));
    assert!(watcher.observe(
        Signature::Ready("recovered".into()),
        now + Duration::from_millis(2400)
    ));
}
