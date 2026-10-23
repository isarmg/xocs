use self::private::DefaultBodyLimitService;
use http::Request;
use service_stack::Layer;

/// Layer for configuring the default request body limit.
///
/// For security reasons, [`Bytes`] will, by default, not accept bodies larger than 2MB. This also
/// applies to extractors that uses [`Bytes`] internally such as `String`, [`Json`], and [`Form`].
///
/// This middleware provides ways to configure that.
///
/// Note that if an extractor consumes the body directly with [`Body::poll_frame`], or similar, the
/// default limit is _not_ applied.
///
/// The limit applies only to extractors that explicitly use the limited-body
/// API, such as [`Bytes`], `String`, [`Json`] and [`Form`]. Custom streaming
/// extractors must bound their own reads.
///
/// # Example
///
/// ```
/// use http_routing::{
///     Router,
///     routing::post,
///     body::Body,
///     extract::{Request, DefaultBodyLimit},
/// };
///
/// let app = Router::new()
///     .route("/", post(|request: Request| async {}))
///     // change the default limit
///     .layer(DefaultBodyLimit::max(1024));
/// # let _: Router = app;
/// ```
///
/// # Different limits for different routes
///
/// `DefaultBodyLimit` can also be selectively applied to have different limits for different
/// routes:
///
/// ```
/// use http_routing::{
///     Router,
///     routing::post,
///     body::Body,
///     extract::{Request, DefaultBodyLimit},
/// };
///
/// let app = Router::new()
///     // this route has a different limit
///     .route("/", post(|request: Request| async {}).layer(DefaultBodyLimit::max(1024)))
///     // this route still has the default limit
///     .route("/foo", post(|request: Request| async {}));
/// # let _: Router = app;
/// ```
///
/// [`Body::poll_frame`]: http_body::Body::poll_frame
/// [`Bytes`]: bytes::Bytes
/// [`Json`]: https://docs.rs/axum/0.8/axum/struct.Json.html
/// [`Form`]: https://docs.rs/axum/0.8/axum/struct.Form.html
/// [`FromRequest`]: crate::kernel::extract::FromRequest
/// [`RequestExt::with_limited_body`]: crate::kernel::RequestExt::with_limited_body
/// [`RequestExt::into_limited_body`]: crate::kernel::RequestExt::into_limited_body
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct DefaultBodyLimit {
    kind: DefaultBodyLimitKind,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DefaultBodyLimitKind {
    Disable,
    Limit(usize),
}

impl DefaultBodyLimit {
    /// Disable the default request body limit.
    ///
    /// This must be used to receive bodies larger than the default limit of 2MB using [`Bytes`] or
    /// an extractor built on it such as `String`, [`Json`], [`Form`].
    ///
    /// Untrusted streaming bodies still require a bound enforced by the caller.
    /// To increase the extractor limit while keeping a finite bound, use
    /// [`DefaultBodyLimit::max`] instead of disabling it.
    ///
    /// [`Bytes`]: bytes::Bytes
    /// [`Json`]: crate::Json
    /// [`Form`]: crate::Form
    pub const fn disable() -> Self {
        Self {
            kind: DefaultBodyLimitKind::Disable,
        }
    }

    /// Set the default request body limit.
    ///
    /// By default the limit of request body sizes that [`Bytes::from_request`] (and other
    /// extractors built on top of it such as `String`, [`Json`], and [`Form`]) is 2MB. This method
    /// can be used to change that limit.
    ///
    /// # Example
    ///
    /// ```
    /// use http_routing::{
    ///     Router,
    ///     routing::get,
    ///     body::{Bytes, Body},
    ///     extract::DefaultBodyLimit,
    /// };
    ///
    /// let app: Router<()> = Router::new()
    ///     .route("/", get(|body: Bytes| async {}))
    ///     // Replace the default of 2MB with 1024 bytes.
    ///     .layer(DefaultBodyLimit::max(1024));
    /// ```
    ///
    /// [`Bytes::from_request`]: bytes::Bytes
    /// [`Json`]: https://docs.rs/axum/0.8/axum/struct.Json.html
    /// [`Form`]: https://docs.rs/axum/0.8/axum/struct.Form.html
    pub const fn max(limit: usize) -> Self {
        Self {
            kind: DefaultBodyLimitKind::Limit(limit),
        }
    }

    /// Apply a request body limit to the given request.
    ///
    /// This can be used, for example, to modify the default body limit inside a specific
    /// extractor.
    ///
    /// # Example
    ///
    /// An extractor similar to [`Bytes`](bytes::Bytes), but limiting the body to 1 KB.
    ///
    /// ```
    /// use http_routing::{
    ///     extract::{DefaultBodyLimit, FromRequest, rejection::BytesRejection, Request},
    ///     body::Bytes,
    /// };
    ///
    /// struct Bytes1KB(Bytes);
    ///
    /// impl<S: Sync> FromRequest<S> for Bytes1KB {
    ///     type Rejection = BytesRejection;
    ///
    ///     async fn from_request(mut req: Request, _: &S) -> Result<Self, Self::Rejection> {
    ///         DefaultBodyLimit::max(1024).apply(&mut req);
    ///         Ok(Self(Bytes::from_request(req, &()).await?))
    ///     }
    /// }
    /// ```
    pub fn apply<B>(self, req: &mut Request<B>) {
        req.extensions_mut().insert(self.kind);
    }
}

impl<S> Layer<S> for DefaultBodyLimit {
    type Service = DefaultBodyLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DefaultBodyLimitService {
            inner,
            kind: self.kind,
        }
    }
}

mod private {
    use super::DefaultBodyLimitKind;
    use http::Request;
    use std::task::Context;
    use service_stack::Service;

    #[derive(Debug, Clone, Copy)]
    pub struct DefaultBodyLimitService<S> {
        pub(super) inner: S,
        pub(super) kind: DefaultBodyLimitKind,
    }

    impl<B, S> Service<Request<B>> for DefaultBodyLimitService<S>
    where
        S: Service<Request<B>>,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = S::Future;

        #[inline]
        fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        #[inline]
        fn call(&mut self, mut req: Request<B>) -> Self::Future {
            req.extensions_mut().insert(self.kind);
            self.inner.call(req)
        }
    }
}
