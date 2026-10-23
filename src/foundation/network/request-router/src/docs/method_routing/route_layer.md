Apply a [`service_stack::Layer`] to the router that will only run if the request matches
a route.

Note that the middleware is only applied to existing routes. First add your routes and then call `route_layer`
afterwards. Additional routes added after `route_layer` is called will not have
the middleware added.

This works similarly to [`MethodRouter::layer`] except the middleware will only run if
the request matches a route. This is useful for middleware that return early
(such as authorization) which might otherwise convert a `405 Method Not Allowed` into a
`401 Unauthorized`.

# Example

```rust
use http_routing::{extract::Extension, routing::get, Router};

let app: Router = Router::new()
    .route("/foo", get(|| async { "Blog" }))
    .route_layer(Extension("label"));
```

Middleware that rejects a request early should return its own response.
Implement it with [`crate::middleware::from_fn`] or a local service layer.
