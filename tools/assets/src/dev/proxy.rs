//! The development proxy only needs pooled, streaming HTTP/1 on loopback.
//! It does not redirect, replay requests or consult environment proxy settings.

use anyhow::Result;
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, Uri, header},
    response::Response,
};
use hyper::{
    adapters::{
        client::legacy::{Client as HttpClient, connect::HttpConnector},
        rt::{TokioExecutor, TokioTimer},
    },
    body::{Body as HttpBody, Frame, SizeHint},
};
use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{Instant, Sleep};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub(super) struct Client {
    inner: HttpClient<HttpConnector, DeadlineBody<Body>>,
    timeout: Duration,
}

impl Client {
    pub(super) fn new(timeout: Duration) -> Self {
        let mut connector = HttpConnector::new();
        connector.set_nodelay(true);
        connector.set_connect_timeout(Some(timeout));
        let inner = HttpClient::builder(TokioExecutor::new())
            .pool_timer(TokioTimer::new())
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .retry_canceled_requests(false)
            .build(connector);
        Self { inner, timeout }
    }

    pub(super) async fn forward(&self, request: Request, target: Uri) -> Result<Response> {
        let deadline = Instant::now() + self.timeout;
        let (mut parts, body) = request.into_parts();
        parts.uri = target;
        parts.extensions.clear();
        strip_connection_headers(&mut parts.headers);
        let request = Request::from_parts(parts, DeadlineBody::new(body, deadline));
        let response = tokio::time::timeout_at(deadline, self.inner.request(request)).await??;
        let (mut parts, body) = response.into_parts();
        parts.extensions.clear();
        strip_connection_headers(&mut parts.headers);
        Ok(Response::from_parts(
            parts,
            Body::new(DeadlineBody::new(body, deadline)),
        ))
    }
}

fn strip_connection_headers(headers: &mut HeaderMap) {
    // Parse the connection-specific field names before removing Connection.
    // Moving the remaining map preserves repeated headers (notably Set-Cookie)
    // without cloning every header name and value for each proxy direction.
    let named: Vec<header::HeaderName> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .filter_map(|name| header::HeaderName::from_bytes(name.trim().as_bytes()).ok())
        .collect();
    for name in named {
        headers.remove(name);
    }
    for name in [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    ] {
        headers.remove(name);
    }
}

struct DeadlineBody<B> {
    inner: B,
    deadline: Pin<Box<Sleep>>,
    done: bool,
}

impl<B> DeadlineBody<B> {
    fn new(inner: B, deadline: Instant) -> Self {
        Self {
            inner,
            deadline: Box::pin(tokio::time::sleep_until(deadline)),
            done: false,
        }
    }
}

impl<B: HttpBody + Unpin> HttpBody for DeadlineBody<B>
where
    B::Error: Into<BoxError>,
{
    type Data = B::Data;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        if this.done {
            return Poll::Ready(None);
        }
        if this.deadline.as_mut().poll(cx).is_ready() {
            this.done = true;
            return Poll::Ready(Some(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "development proxy request deadline elapsed",
            )
            .into())));
        }
        match Pin::new(&mut this.inner).poll_frame(cx) {
            Poll::Ready(None) => {
                this.done = true;
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(error))) => {
                this.done = true;
                Poll::Ready(Some(Err(error.into())))
            }
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.done || self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        if self.done {
            SizeHint::with_exact(0)
        } else {
            self.inner.size_hint()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Bytes, http::StatusCode};
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
        let app =
            Router::new().fallback(|ConnectInfo(address): ConnectInfo<SocketAddr>| async move {
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
            let _ = axum::body::to_bytes(request.into_body(), 1024).await;
            "finished"
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
}
