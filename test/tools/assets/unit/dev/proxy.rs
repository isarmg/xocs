use super::*;
use anyhow::Result;
use axum::{Router, body::Bytes, http::StatusCode};
use std::io;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn preserves_streamed_request_identity_and_does_not_follow_redirects() -> Result<()> {
    let seen = Arc::new(Mutex::new(None));
    let capture = seen.clone();
    let app = Router::new().fallback(move |request: Request| {
        let capture = capture.clone();
        async move {
            let (parts, body) = request.into_parts();
            let bytes = axum::body::to_bytes(body, 1024).await.unwrap();
            *capture.lock().unwrap() = Some((parts, bytes));
            Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(header::LOCATION, "http://127.0.0.1:1/do-not-follow")
                .header(header::SET_COOKIE, "first=1; HttpOnly")
                .header(header::SET_COOKIE, "second=2; HttpOnly")
                .header(header::CONNECTION, "x-upstream-only")
                .header("x-upstream-only", "private connection state")
                .body(Body::from("upstream response"))
                .unwrap()
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let client = Client::new(Duration::from_secs(2));
    let request = Request::builder()
        .method("POST")
        .uri("/api/write?q=a%2Fb&n=2")
        .header(header::HOST, "localhost:5173")
        .header(header::ORIGIN, "http://localhost:5173")
        .header(header::COOKIE, "session=secret")
        .header("x-csrf-token", "csrf-token")
        .header(header::CONNECTION, "x-client-only")
        .header("x-client-only", "private connection state")
        .body(Body::from("streamed upload"))?;
    let response = client
        .forward(
            request,
            format!("http://{address}/api/write?q=a%2Fb&n=2").parse()?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .count(),
        2
    );
    assert!(!response.headers().contains_key(header::CONNECTION));
    assert!(!response.headers().contains_key("x-upstream-only"));
    let data = axum::body::to_bytes(response.into_body(), 1024).await?;
    assert_eq!(data, "upstream response");
    let (parts, body) = seen.lock().unwrap().take().unwrap();
    assert_eq!(parts.method, "POST");
    assert_eq!(
        parts.uri.path_and_query().unwrap().as_str(),
        "/api/write?q=a%2Fb&n=2"
    );
    assert_eq!(parts.headers[header::HOST], "localhost:5173");
    assert_eq!(parts.headers[header::ORIGIN], "http://localhost:5173");
    assert_eq!(parts.headers[header::COOKIE], "session=secret");
    assert_eq!(parts.headers["x-csrf-token"], "csrf-token");
    assert!(!parts.headers.contains_key("x-client-only"));
    assert_eq!(body, "streamed upload");
    server.abort();
    Ok(())
}

struct PendingBody;

impl HttpBody for PendingBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Pending
    }
}

#[tokio::test]
async fn shared_deadline_limits_response_body_after_headers_arrive() -> Result<()> {
    let app = Router::new().fallback(|| async { Body::new(PendingBody) });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let client = Client::new(Duration::from_millis(100));
    let response = client
        .forward(
            Request::new(Body::empty()),
            format!("http://{address}/").parse()?,
        )
        .await?;
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        axum::body::to_bytes(response.into_body(), 1024),
    )
    .await?;
    assert!(
        result.is_err(),
        "a streamed response must retain the request deadline"
    );
    server.abort();
    Ok(())
}

#[tokio::test]
async fn deadline_body_stops_polling_after_timeout() {
    let mut body = DeadlineBody::new(PendingBody, Instant::now());
    let waker = std::task::Waker::noop();
    let mut context = Context::from_waker(waker);
    // Let Tokio observe the already elapsed deadline.
    tokio::time::sleep(Duration::from_millis(1)).await;
    assert!(matches!(
        Pin::new(&mut body).poll_frame(&mut context),
        Poll::Ready(Some(Err(_)))
    ));
    assert!(body.is_end_stream());
    assert_eq!(body.size_hint().exact(), Some(0));
    assert!(matches!(
        Pin::new(&mut body).poll_frame(&mut context),
        Poll::Ready(None)
    ));
}

#[tokio::test]
async fn client_clones_reuse_the_http_connection() -> Result<()> {
    use axum::extract::ConnectInfo;
    use std::net::SocketAddr;
    let app = Router::new().fallback(|ConnectInfo(address): ConnectInfo<SocketAddr>| async move {
        address.to_string()
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
    });
    let client = Client::new(Duration::from_secs(2));
    let mut peers = Vec::new();
    for _ in 0..2 {
        let response = client
            .clone()
            .forward(
                Request::new(Body::empty()),
                format!("http://{address}/").parse()?,
            )
            .await?;
        peers.push(axum::body::to_bytes(response.into_body(), 1024).await?);
    }
    assert_eq!(
        peers[0], peers[1],
        "cloning the proxy must share its idle connection pool"
    );
    server.abort();
    Ok(())
}

#[tokio::test]
async fn deadline_cancels_a_pending_upload_before_response_headers() -> Result<()> {
    let app = Router::new().fallback(|request: Request| async move {
        match axum::body::to_bytes(request.into_body(), 1024).await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::BAD_REQUEST,
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let client = Client::new(Duration::from_millis(50));
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        client.forward(
            Request::builder()
                .method("POST")
                .body(Body::new(PendingBody))?,
            format!("http://{address}/").parse()?,
        ),
    )
    .await?;
    assert!(
        result.is_err(),
        "upload must be cancelled at the same request deadline"
    );
    server.abort();
    Ok(())
}

#[test]
fn connection_lists_accept_empty_elements_but_reject_invalid_tokens() {
    use std::error::Error;
    for hop in [ProxyHop::Client, ProxyHop::Upstream] {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONNECTION,
            header::HeaderValue::from_static(", x-private, ,"),
        );
        headers.insert("x-private", header::HeaderValue::from_static("hop value"));
        strip_connection_headers(&mut headers, hop).unwrap();
        assert!(!headers.contains_key("x-private"));
        assert!(!headers.contains_key(header::CONNECTION));
        headers.insert(
            header::CONNECTION,
            header::HeaderValue::from_static("bad name"),
        );
        let error = strip_connection_headers(&mut headers, hop).unwrap_err();
        assert!(matches!(error, ProxyError::ConnectionName { .. }));
        assert!(error.source().is_some());
    }
}
