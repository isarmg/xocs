# SQLite test coverage

The remaining `sqlite/` files are reference scenarios for the maintained SQLite
connection, value, error and record decoding APIs. They are not Cargo test targets
(`autotests = false`) and do not count as executed coverage. Their former upstream
test harness is not part of this application.

Current executable coverage uses the product's actual dependency configuration:

```sh
cargo test --locked --offline -p blog --test sqlite_bindings --test record_derives
cargo test --locked --offline -p blog-application --test session_cleanup
cargo test --locked --offline -p blog-application --lib sqlite::blocking::tests
cargo test --locked --offline -p blog-storage-sqlite-access --lib
```

Run these commands from the repository root to use its single lockfile. The
maintained dependency test script also runs the worker handoff regression.

Tests and harnesses for removed database drivers, generic driver selection,
compile-time SQL query macros, migration management and SQLCipher were removed
with those APIs. Original source archives and provenance remain in `compliance/`.
