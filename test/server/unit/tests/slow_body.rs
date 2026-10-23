use super::*;
use axum::body::Bytes;
use http_body::{Body as HttpBody, Frame};
use std::{
    convert::Infallible,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
    time::Duration,
};

struct StalledBody {
    prefix: Option<Bytes>,
    polled: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
    counted: bool,
}

impl HttpBody for StalledBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
        if let Some(prefix) = self.prefix.take() {
            return Poll::Ready(Some(Ok(Frame::data(prefix))));
        }
        if !self.counted {
            self.counted = true;
            self.polled.fetch_add(1, Ordering::SeqCst);
        }
        Poll::Pending
    }
}

impl Drop for StalledBody {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

fn stalled_request(
    polled: &Arc<AtomicUsize>,
    dropped: &Arc<AtomicUsize>,
    id: usize,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/tree-hole/guest")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-request-id", format!("slow-write-{id}"))
        .body(Body::new(StalledBody {
            prefix: Some(Bytes::from_static(b"{\"message\":\"SECRET")),
            polled: polled.clone(),
            dropped: dropped.clone(),
            counted: false,
        }))
        .unwrap()
}

async fn wait_for_reads(polled: &AtomicUsize, expected: usize) {
    for _ in 0..10_000 {
        if polled.load(Ordering::SeqCst) == expected {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("requests did not enter body extraction");
}

#[tokio::test]
async fn stalled_json_bodies_expire_without_writes_and_release_all_public_execution_slots() {
    let (_directory, app, pool) = setup().await;
    let polled = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));
    tokio::time::pause();
    let requests: Vec<_> = (0..4)
        .map(|id| tokio::spawn(app.clone().oneshot(stalled_request(&polled, &dropped, id))))
        .collect();
    wait_for_reads(&polled, 4).await;
    let deadline = blog_application::server_cli::http::JSON_BODY_TIMEOUT;
    tokio::time::advance(deadline - Duration::from_millis(1)).await;
    assert!(requests.iter().all(|request| !request.is_finished()));
    tokio::time::advance(Duration::from_millis(1)).await;
    for (id, request) in requests.into_iter().enumerate() {
        let response = tokio::time::timeout(Duration::from_secs(1), request)
            .await
            .expect("body deadline did not release the request")
            .unwrap()
            .unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(response.headers()[header::CONNECTION], "close");
        assert_eq!(
            response.headers()["x-request-id"],
            format!("slow-write-{id}")
        );
        let error = body_json(response).await;
        assert_eq!(error["code"], "request_timeout");
        assert_eq!(error["details"]["reason"], "BODY_TIMEOUT");
        assert_eq!(error["retryable"], true);
        assert!(!error.to_string().contains("SECRET"));
    }
    assert_eq!(dropped.load(Ordering::SeqCst), 4);
    tokio::time::resume();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tree_hole")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        anonymous_post(
            &app,
            "/api/v1/tree-hole/guest",
            serde_json::json!({"message":"Capacity recovered"}),
        )
        .await
        .status(),
        StatusCode::OK
    );

    // Client cancellation drops the reader and its capacity before the deadline.
    let request = tokio::spawn(app.clone().oneshot(stalled_request(&polled, &dropped, 4)));
    wait_for_reads(&polled, 5).await;
    request.abort();
    assert!(request.await.unwrap_err().is_cancelled());
    assert_eq!(dropped.load(Ordering::SeqCst), 5);
    assert_eq!(
        anonymous_post(
            &app,
            "/api/v1/tree-hole/guest",
            serde_json::json!({"message":"Cancellation recovered"}),
        )
        .await
        .status(),
        StatusCode::OK
    );
}
