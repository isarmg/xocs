# Blog offline setup

This crate is maintained with Blog and calls the product's `maintenance` API. It does not download installation code or execute an external product server to initialize state.

Build with `cargo build --locked --offline --release -p blog-setup`. Prepare a private `ROOT/config/config.json` first, then run as the instance's service account:

```bash
systemd-ask-password 'Initial administrator password:' | blog-setup --root /absolute/instance initialize --username admin
blog-setup --root /absolute/instance check
systemd-ask-password 'New administrator password:' | blog-setup --root /absolute/instance admin-reset --username admin
```

Before reading credentials or touching state, the tool verifies its compiled Blog identity against the current product, schema and local core source constants. Candidate and release packaging verifies the same canonical identity bytes and digest are present in both the server and setup executables.

Offline operations use only the explicitly named `ROOT/config/config.json`; runtime `BLOG_*` overrides cannot redirect them to another instance. Installed wrappers also clear the six declared runtime overrides while retaining build identity variables.

Stop the server before maintenance. The product API obtains the same instance lock as the server, rejects an existing database during initialization, validates the current SQLite schema and contents, and invalidates sessions when credentials change. Password input is bounded, single-line UTF-8 on standard input; it is never a command argument or diagnostic.
