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
#[path = "../../../../../test/tools/assets/unit/dev/proxy.rs"]
mod tests;
