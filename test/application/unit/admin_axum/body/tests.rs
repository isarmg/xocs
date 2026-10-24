use super::*;
use http_body::Frame;
use std::{
    convert::Infallible,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

struct PendingBody(Arc<AtomicBool>);
impl http_body::Body for PendingBody {
    type Data = Bytes;
    type Error = Infallible;
    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
        Poll::Pending
    }
}
impl Drop for PendingBody {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn request(body: Body) -> Request {
    Request::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap()
}

#[test]
fn global_budget_releases_permits_without_source_identity() {
    let admission = BodyAdmission::isolated();
    let mut permits = Vec::new();
    for _ in 0..ADMIN_BODY_READERS_GLOBAL {
        permits.push(admission.acquire(None).unwrap());
    }
    assert_eq!(
        admission.acquire(None).err().unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    permits.pop();
    permits.push(admission.acquire(None).unwrap());
    drop(permits);
    assert_eq!(admission.0.lock().unwrap().total, 0);
}

#[tokio::test]
async fn independently_created_routers_share_the_process_body_budget() {
    use crate::admin_auth::AdministratorOriginMode;
    use crate::admin_core::{ADMIN_BODY_READERS_GLOBAL, AdministratorService};
    use tower::ServiceExt;

    // Saturating the actual process budget must not compete with parallel
    // tests. The child runs only this test and uses the ordinary router factory.
    let name = std::thread::current().name().unwrap().to_owned();
    if std::env::var("APPLICATION_BODY_BUDGET_TEST").as_deref() != Ok(&name) {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env("APPLICATION_BODY_BUDGET_TEST", &name)
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }

    let store = crate::testkit::administrator_fixture("admin", "correct horse battery").await;
    let service = Arc::new(AdministratorService::with_isolated_test_budget(store));
    let routers = ["first-product", "second-product"].map(|product| {
        crate::admin_axum::administrator_router(
            product,
            AdministratorOriginMode::LoopbackDevelopmentHttp,
            service.clone(),
        )
        .unwrap()
    });
    let login_request = |body: Body| {
        Request::builder()
            .method("POST")
            .uri(crate::contracts::ADMIN_LOGIN_PATH)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ORIGIN, "http://127.0.0.1")
            .header(header::HOST, "127.0.0.1")
            .header("sec-fetch-site", "same-origin")
            .body(body)
            .unwrap()
    };
    let mut readers = Vec::new();
    let mut dropped = Vec::new();
    for index in 0..ADMIN_BODY_READERS_GLOBAL {
        let flag = Arc::new(AtomicBool::new(false));
        let mut read = Box::pin(
            routers[index % routers.len()]
                .clone()
                .oneshot(login_request(Body::new(PendingBody(flag.clone())))),
        );
        std::future::poll_fn(|cx| {
            assert!(read.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        readers.push(read);
        dropped.push(flag);
    }
    for router in &routers {
        let response = router
            .clone()
            .oneshot(login_request(Body::from("{}")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "1");
    }
    readers.pop();
    let response = routers[0]
        .clone()
        .oneshot(login_request(Body::from("{}")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    drop(readers);
    assert!(dropped.iter().all(|flag| flag.load(Ordering::SeqCst)));
    assert_eq!(BodyAdmission::default().0.lock().unwrap().total, 0);
}

#[tokio::test(start_paused = true)]
async fn slow_body_deadline_and_cancellation_drop_body_and_permit() {
    let admission = BodyAdmission::isolated();
    let dropped = Arc::new(AtomicBool::new(false));
    let id = RequestId::new("slow-body-request-1").unwrap();
    let response = admission
        .read(
            request(Body::new(PendingBody(dropped.clone()))),
            Scope::Login,
            true,
            Some(&id),
        )
        .await
        .unwrap_err();
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
    let envelope: crate::contracts::ErrorEnvelope = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope.request_id.as_ref(), Some(&id));
    assert!(envelope.retryable);
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(admission.0.lock().unwrap().total, 0);
    let dropped = Arc::new(AtomicBool::new(false));
    let mut read = Box::pin(admission.read(
        request(Body::new(PendingBody(dropped.clone()))),
        Scope::Login,
        true,
        None,
    ));
    std::future::poll_fn(|cx| {
        assert!(read.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert_eq!(admission.0.lock().unwrap().total, 1);
    drop(read);
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(admission.0.lock().unwrap().total, 0);
}

#[tokio::test]
async fn exact_size_boundary_and_malformed_transport_are_distinct_safe_errors() {
    assert_eq!(
        read_bounded(
            Body::from(vec![b' '; ADMIN_BODY_MAX_BYTES]),
            Scope::Login,
            None
        )
        .await
        .unwrap()
        .len(),
        ADMIN_BODY_MAX_BYTES
    );
    assert_eq!(
        read_bounded(
            Body::from(vec![b' '; ADMIN_BODY_MAX_BYTES + 1]),
            Scope::Login,
            None
        )
        .await
        .unwrap_err()
        .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    struct BrokenBody;
    impl http_body::Body for BrokenBody {
        type Data = Bytes;
        type Error = std::io::Error;
        fn poll_frame(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
            Poll::Ready(Some(Err(std::io::Error::other("SECRET transport details"))))
        }
    }
    let response = read_bounded(Body::new(BrokenBody), Scope::Login, None)
        .await
        .unwrap_err();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
    assert!(!std::str::from_utf8(&bytes).unwrap().contains("SECRET"));
}
