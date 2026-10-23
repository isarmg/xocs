Apply a [`service_stack::Layer`] to all routes in the router.

This can be used to add additional processing to a request for a group
of routes.

Note that the middleware is only applied to existing routes. So you have to
first add your routes (and / or fallback) and then call `layer` afterwards. Additional
routes added after `layer` is called will not have the middleware added.

If you want to add middleware to a single handler you can either use
[`MethodRouter::layer`] or [`Handler::layer`].

# Example

Attach an extension to a group of routes:

```rust
use http_routing::{extract::Extension, routing::get, Router};

let app: Router = Router::new()
    .route("/foo", get(|| async {}))
    .route("/bar", get(|| async {}))
    .layer(Extension("Blog"));
```

Use [`Router::merge`] to combine routers with different layer stacks.
See [writing middleware](crate::middleware#writing-middleware) to implement
request and response wrappers.

# Multiple middleware

It's recommended to use [`service_stack::ServiceBuilder`] when applying multiple
middleware. See [`middleware`](crate::middleware) for more details.

# Runs after routing

Middleware added with this method will run _after_ routing and thus cannot be
used to rewrite the request URI. See ["Rewriting request URI in
middleware"](crate::middleware#rewriting-request-uri-in-middleware) for more
details and a workaround.

# Error handling

See [`middleware`](crate::middleware) for details on how error handling impacts
middleware.
