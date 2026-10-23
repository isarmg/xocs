# Error handling

Handlers return values implementing [`IntoResponse`]. A handler result such as
`Result<String, StatusCode>` produces an HTTP response on both branches; an
`Err(StatusCode::NOT_FOUND)` is a normal response, not a transport failure.
Extractor rejections also become responses without calling the handler.

The final routed service must use [`Infallible`] as its error type. A service
error that reaches the transport cannot become a normal response by itself and
may terminate the connection. Convert it before routing or before it leaves a
middleware stack.

## Routing to fallible services

[`HandleError`] wraps a service and asynchronously maps its error to a response:

```rust
use http_routing::{
    body::Body,
    error_handling::HandleError,
    extract::Request,
    http::{Response, StatusCode},
    Router,
};

let service = service_stack::service_fn(|_: Request| async {
    Err::<Response<Body>, _>(std::io::Error::other("unavailable"))
});

let app: Router = Router::new().route_service(
    "/",
    HandleError::new(service, |_error: std::io::Error| async {
        StatusCode::SERVICE_UNAVAILABLE
    }),
);
```

[`HandleErrorLayer`] provides the same conversion as a layer. Place it outside
the layer that can fail so it receives that layer's errors. Error handlers may
also extract request parts, such as the HTTP method and URI; their final
argument is the service error.

Body-stream errors are separate: after response headers are sent, an error in
a streaming response cannot replace those headers with a new status code.

[`Infallible`]: std::convert::Infallible
[`IntoResponse`]: crate::response::IntoResponse
