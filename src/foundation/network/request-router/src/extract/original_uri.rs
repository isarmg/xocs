use super::{Extension, FromRequestParts};
use http::{request::Parts, Uri};
use std::convert::Infallible;

/// Extractor that gets the original request URI regardless of nesting.
///
/// This is necessary since [`Uri`](http::Uri), when used as an extractor, will
/// have the prefix stripped if used in a nested service.
///
/// # Example
///
/// ```
/// use http_routing::{
///     routing::get,
///     Router,
///     extract::OriginalUri,
///     http::Uri
/// };
///
/// let api_routes = Router::new()
///     .route(
///         "/users",
///         get(|uri: Uri, OriginalUri(original_uri): OriginalUri| async {
///             // `uri` is `/users`
///             // `original_uri` is `/api/users`
///         }),
///     );
///
/// let app = Router::new().nest("/api", api_routes);
/// # let _: Router = app;
/// ```
///
/// # Extracting via request extensions
///
/// `OriginalUri` can also be accessed from middleware via request extensions.
/// Middleware can inspect `request.extensions().get::<OriginalUri>()`
/// to record this routing metadata. The extension may be absent when a request
/// did not pass through the router or another middleware removed it.
#[derive(Debug, Clone)]
pub struct OriginalUri(pub Uri);

impl<S> FromRequestParts<S> for OriginalUri
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uri = Extension::<Self>::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|_| Extension(OriginalUri(parts.uri.clone())))
            .0;
        Ok(uri)
    }
}

crate::__impl_deref!(OriginalUri: Uri);
