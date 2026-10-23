# SQLite access

`blog-storage-sqlite-access` provides asynchronous SQLite queries, pools,
transactions, prepared statements, value conversion and record mapping.
The supported native target is AMD64 Linux GNU/glibc. The SQLite implementation
and C source are local dependencies in this repository.

## Runtime and connections

The connection pool uses the repository's Tokio runtime for timeouts and
background work. SQLite calls run through the driver's worker connection.
Configure `SqliteConnectOptions` and `SqlitePoolOptions` explicitly for the
application's database path, pool limits and busy timeout.

Acquire a pool connection when several operations must share a connection.
Use `Transaction` when they must commit atomically. Closing a pool and dropping
in-flight work have separate effects; follow the API's cancellation and
transaction rules rather than assuming that dropping a future rolls back an
already committed operation.

## Queries and values

- `query` executes parameterized SQL and returns SQLite rows.
- `query_as` maps rows through `FromRow`; the `derive` feature exports its derive.
- `query_scalar` returns a single selected column per row.
- `QueryBuilder` assembles SQL with bound values.
- `Row`, `Column`, `Type`, `Encode` and `Decode` describe the conversion boundary.

Bind untrusted values as parameters. SQL identifiers and fragments assembled by
the caller require their own validation. Statements execute against SQLite;
there are no network database or TLS backends in this package.

A row can be transferred between threads. Its internal decode state is mutable
and is not `Sync`; do not share the same row concurrently. Supported Rust/SQLite
type mappings are documented in the local `sqlite::types` module.

Schema identity, administrator data, snapshots and installation checks belong
to the application's persistence layer. Opening a connection alone does not
initialize or validate a product database.
