# Middleware

Middleware wraps a service through [`service_stack::Layer`]. Routers, method
routers and individual handlers expose layer methods. Their final service
errors must be converted into responses; see [`crate::error_handling`].

## Applying middleware

- [`Router::layer`] wraps all routes already present in the router.
- [`Router::route_layer`] runs only for matching routes.
- [`MethodRouter::layer`] and [`MethodRouter::route_layer`] scope middleware to
  selected HTTP methods.
- [`Handler::layer`] wraps one handler.

Add routes before calling the layer method. Routes added afterwards do not
inherit previously applied layers. Middleware attached to a router runs after
routing; to change the URI before route selection, wrap the whole router as a
service.

## Writing middleware

[`from_fn`] runs an async function before and after the next service. A request
extractor is followed by [`Next`], which continues processing the request:

```rust
use http_routing::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};

async fn add_header(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert("x-blog", "1".parse().unwrap());
    response
}

let app: Router = Router::new()
    .route("/", get(|| async { "Blog" }))
    .layer(middleware::from_fn(add_header));
```

Use [`from_fn_with_state`] for middleware that extracts application state.
[`from_extractor`] runs an extractor before the handler, [`map_request`] maps
requests, and [`map_response`] maps responses. A custom `Layer`/`Service`
implementation is appropriate when the wrapper needs explicit service state or
readiness behavior.

## Accessing state in middleware

Use [`from_fn_with_state`] to provide a cloned state value and extract it with
[`crate::extract::State`]. Request extensions and application state are
separate: extensions are attached to a particular request, while state is
configured for the service.

## Rewriting request URI in middleware

Router layers run after route selection. To rewrite a URI before selecting a
route, wrap the whole router as a service, then pass that service to the
server. [`crate::ServiceExt::into_make_service`] provides the connection-level
service factory for a wrapped router. Preserve the original URI explicitly if
later middleware must inspect it.

## Layer ordering

Successive `Router::layer` calls wrap the existing router, so the last layer
receives the request first. [`service_stack::ServiceBuilder`] composes layers
in request order, from the first added layer to the last. Response processing
runs in the reverse order.

A layer that converts service errors into responses must wrap the fallible
layer. Handler rejections are already responses, while an unhandled service
error prevents a valid HTTP response. Body-stream errors can also occur after
response headers have been sent and require separate handling.

## Routing to services/middleware and backpressure

Routing must inspect the request before selecting a destination. The router
reports readiness and drives the selected service's readiness within the call
future. A readiness error is returned from that future; the service is retained.

Apply application-wide admission control outside the router when it must bound
all requests. Use explicit queue bounds, cancellation and deadlines. The local
service stack provides composition and readiness adapters; it does not provide
a background queue, retry policy or load balancer.

[`Router::layer`]: crate::Router::layer
[`Router::route_layer`]: crate::Router::route_layer
[`MethodRouter::layer`]: crate::routing::MethodRouter::layer
[`MethodRouter::route_layer`]: crate::routing::MethodRouter::route_layer
[`Handler::layer`]: crate::handler::Handler::layer
