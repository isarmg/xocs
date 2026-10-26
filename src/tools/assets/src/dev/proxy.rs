//! The development proxy only needs pooled, streaming HTTP/1 on loopback.
//! It does not redirect, replay requests or consult environment proxy settings.

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
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{Instant, Sleep};

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("development proxy request deadline elapsed")]
    Deadline(#[source] tokio::time::error::Elapsed),
    #[error("forward development proxy request")]
    Request(#[source] hyper::adapters::client::legacy::Error),
    #[error("decode {hop} connection header")]
    ConnectionEncoding {
        hop: ProxyHop,
        #[source]
        source: header::ToStrError,
    },
    #[error("parse {hop} connection-specific header name")]
    ConnectionName {
        hop: ProxyHop,
        #[source]
        source: header::InvalidHeaderName,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ProxyHop {
    Client,
    Upstream,
}

impl std::fmt::Display for ProxyHop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Client => "client",
            Self::Upstream => "upstream",
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum DeadlineBodyError<E: std::error::Error + 'static> {
    #[error("development proxy stream deadline elapsed")]
    Deadline,
    #[error("read development proxy stream")]
    Stream(#[source] E),
}

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

    pub(super) async fn forward(
        &self,
        request: Request,
        target: Uri,
    ) -> Result<Response, ProxyError> {
        let deadline = Instant::now() + self.timeout;
        let (mut parts, body) = request.into_parts();
        parts.uri = target;
        parts.extensions.clear();
        strip_connection_headers(&mut parts.headers, ProxyHop::Client)?;
        let request = Request::from_parts(parts, DeadlineBody::new(body, deadline));
        let response = tokio::time::timeout_at(deadline, self.inner.request(request))
            .await
            .map_err(ProxyError::Deadline)?
            .map_err(ProxyError::Request)?;
        let (mut parts, body) = response.into_parts();
        parts.extensions.clear();
        strip_connection_headers(&mut parts.headers, ProxyHop::Upstream)?;
        Ok(Response::from_parts(
            parts,
            Body::new(DeadlineBody::new(body, deadline)),
        ))
    }
}

fn strip_connection_headers(headers: &mut HeaderMap, hop: ProxyHop) -> Result<(), ProxyError> {
    let mut named = Vec::new();
    for value in headers.get_all(header::CONNECTION) {
        let value = value
            .to_str()
            .map_err(|source| ProxyError::ConnectionEncoding { hop, source })?;
        for name in value
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            named.push(
                header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|source| ProxyError::ConnectionName { hop, source })?,
            );
        }
    }
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
    Ok(())
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
    B::Error: std::error::Error + Send + Sync + 'static,
{
    type Data = B::Data;
    type Error = DeadlineBodyError<B::Error>;

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
            return Poll::Ready(Some(Err(DeadlineBodyError::Deadline)));
        }
        match Pin::new(&mut this.inner).poll_frame(cx) {
            Poll::Ready(None) => {
                this.done = true;
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(error))) => {
                this.done = true;
                Poll::Ready(Some(Err(DeadlineBodyError::Stream(error))))
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
