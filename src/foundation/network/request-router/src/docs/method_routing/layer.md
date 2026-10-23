Apply a [`service_stack::Layer`] to all routes in the router.

This can be used to add additional processing to a request for a group
of routes.

Note that the middleware is only applied to existing routes. So you have to
first add your routes (and / or fallback) and then call `layer` afterwards. Additional
routes added after `layer` is called will not have the middleware added.

Works similarly to [`Router::layer`](super::Router::layer). See that method for
more details.

# Example

```rust
use http_routing::{extract::Extension, routing::get, Router};

let app: Router = Router::new().route(
    "/",
    get(|| async { "Blog" }).layer(Extension("label")),
);
```
