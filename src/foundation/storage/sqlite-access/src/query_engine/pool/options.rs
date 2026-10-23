use crate::query_engine::connection::Connection;
use crate::query_engine::database::Database;
use crate::query_engine::error::Error;
use crate::query_engine::pool::inner::PoolInner;
use crate::query_engine::pool::Pool;
use futures::future::BoxFuture;
use log::LevelFilter;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration options for [`Pool`][super::Pool].
///
/// ### Callback Functions: Why Do I Need `Box::pin()`?
/// Essentially, because it's impossible to write generic bounds that describe a closure
/// with a higher-ranked lifetime parameter, returning a future with that same lifetime.
///
/// Ideally, you could define it like this:
/// ```rust,ignore
/// async fn takes_foo_callback(f: impl for<'a> Fn(&'a mut Foo) -> impl Future<'a, Output = ()>)
/// ```
///
/// However, the compiler does not allow using `impl Trait` in the return type of an `impl Fn`.
///
/// And if you try to do it like this:
/// ```rust,ignore
/// async fn takes_foo_callback<F, Fut>(f: F)
/// where
///     F: for<'a> Fn(&'a mut Foo) -> Fut,
///     Fut: for<'a> Future<Output = ()> + 'a
/// ```
///
/// There's no way to tell the compiler that those two `'a`s should be the same lifetime.
///
/// It's possible to make this work with a custom trait, but it's fiddly and requires naming
///  the type of the closure parameter.
///
/// Having the closure return `BoxFuture` allows us to work around this, as all the type information
/// fits into a single generic parameter.
///
/// We still need to `Box` the future internally to give it a concrete type to avoid leaking a type
/// parameter everywhere, and `Box` is in the prelude so it doesn't need to be manually imported,
/// so having the closure return `Pin<Box<dyn Future>` directly is the path of least resistance from
/// the perspectives of both API designer and consumer.
pub struct PoolOptions<DB: Database> {
    pub(crate) test_before_acquire: bool,
    pub(crate) after_connect: Option<
        Arc<
            dyn Fn(&mut DB::Connection, PoolConnectionMetadata) -> BoxFuture<'_, Result<(), Error>>
                + 'static
                + Send
                + Sync,
        >,
    >,
    pub(crate) before_acquire: Option<
        Arc<
            dyn Fn(
                    &mut DB::Connection,
                    PoolConnectionMetadata,
                ) -> BoxFuture<'_, Result<bool, Error>>
                + 'static
                + Send
                + Sync,
        >,
    >,
    pub(crate) after_release: Option<
        Arc<
            dyn Fn(
                    &mut DB::Connection,
                    PoolConnectionMetadata,
                ) -> BoxFuture<'_, Result<bool, Error>>
                + 'static
                + Send
                + Sync,
        >,
    >,
    pub(crate) max_connections: u32,
    pub(crate) acquire_time_level: LevelFilter,
    pub(crate) acquire_slow_level: LevelFilter,
    pub(crate) acquire_slow_threshold: Duration,
    pub(crate) acquire_timeout: Duration,
    pub(crate) min_connections: u32,
    pub(crate) max_lifetime: Option<Duration>,
    pub(crate) idle_timeout: Option<Duration>,
    pub(crate) fair: bool,

    pub(crate) parent_pool: Option<Pool<DB>>,
}

// Manually implement `Clone` to avoid a trait bound issue.
//
// See: https://github.com/launchbadge/sqlx/issues/2548
impl<DB: Database> Clone for PoolOptions<DB> {
    fn clone(&self) -> Self {
        PoolOptions {
            test_before_acquire: self.test_before_acquire,
            after_connect: self.after_connect.clone(),
            before_acquire: self.before_acquire.clone(),
            after_release: self.after_release.clone(),
            max_connections: self.max_connections,
            acquire_time_level: self.acquire_time_level,
            acquire_slow_threshold: self.acquire_slow_threshold,
            acquire_slow_level: self.acquire_slow_level,
            acquire_timeout: self.acquire_timeout,
            min_connections: self.min_connections,
            max_lifetime: self.max_lifetime,
            idle_timeout: self.idle_timeout,
            fair: self.fair,
            parent_pool: self.parent_pool.clone(),
        }
    }
}

/// Metadata for the connection being processed by a [`PoolOptions`] callback.
#[derive(Debug)] // Don't want to commit to any other trait impls yet.
#[non_exhaustive] // So we can safely add fields in the future.
pub struct PoolConnectionMetadata {
    /// The duration since the connection was first opened.
    ///
    /// For [`after_connect`][PoolOptions::after_connect], this is [`Duration::ZERO`].
    pub age: Duration,

    /// The duration that the connection spent in the idle queue.
    ///
    /// Only relevant for [`before_acquire`][PoolOptions::before_acquire].
    /// For other callbacks, this is [`Duration::ZERO`].
    pub idle_for: Duration,
}

impl<DB: Database> Default for PoolOptions<DB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database> PoolOptions<DB> {
    /// Returns a default "sane" configuration, suitable for testing or light-duty applications.
    ///
    /// Production applications will likely want to at least modify
    /// [`max_connections`][Self::max_connections].
    ///
    /// See the source of this method for the current default values.
    pub fn new() -> Self {
        Self {
            // User-specifiable routines
            after_connect: None,
            before_acquire: None,
            after_release: None,
            test_before_acquire: true,
            // A production application will want to set a higher limit than this.
            max_connections: 10,
            min_connections: 0,
            // Logging all acquires is opt-in
            acquire_time_level: LevelFilter::Off,
            // Default to warning, because an acquire timeout will be an error
            acquire_slow_level: LevelFilter::Warn,
            // Fast enough to catch problems (e.g. a full pool); slow enough
            // to not flag typical time to add a new connection to a pool.
            acquire_slow_threshold: Duration::from_secs(2),
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(10 * 60)),
            max_lifetime: Some(Duration::from_secs(30 * 60)),
            fair: true,
            parent_pool: None,
        }
    }

    /// Set the maximum number of connections that this pool should maintain.
    ///
    /// Be mindful of the connection limits for your database as well as other applications
    /// which may want to connect to the same database (or even multiple instances of the same
    /// application in high-availability deployments).
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Get the maximum number of connections that this pool should maintain
    pub fn get_max_connections(&self) -> u32 {
        self.max_connections
    }

    /// Set the minimum number of connections to maintain at all times.
    ///
    /// When the pool is built, this many connections will be automatically spun up.
    ///
    /// If any connection is reaped by [`max_lifetime`] or [`idle_timeout`], or explicitly closed,
    /// and it brings the connection count below this amount, a new connection will be opened to
    /// replace it.
    ///
    /// This is only done on a best-effort basis, however. The routine that maintains this value
    /// has a deadline so it doesn't wait forever if the database is being slow or returning errors.
    ///
    /// This value is clamped internally to not exceed [`max_connections`].
    ///
    /// We've chosen not to assert `min_connections <= max_connections` anywhere
    /// because it shouldn't break anything internally if the condition doesn't hold,
    /// and if the application allows either value to be dynamically set
    /// then it should be checking this condition itself and returning
    /// a nicer error than a panic anyway.
    ///
    /// [`max_lifetime`]: Self::max_lifetime
    /// [`idle_timeout`]: Self::idle_timeout
    /// [`max_connections`]: Self::max_connections
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Get the minimum number of connections to maintain at all times.
    pub fn get_min_connections(&self) -> u32 {
        self.min_connections
    }

    /// Enable logging of time taken to acquire a connection from the connection pool via
    /// [`Pool::acquire()`].
    ///
    /// If slow acquire logging is also enabled, this level is used for acquires that are not
    /// considered slow.
    pub fn acquire_time_level(mut self, level: LevelFilter) -> Self {
        self.acquire_time_level = level;
        self
    }

    /// Log excessive time taken to acquire a connection at a different log level than time taken
    /// for faster connection acquires via [`Pool::acquire()`].
    pub fn acquire_slow_level(mut self, level: LevelFilter) -> Self {
        self.acquire_slow_level = level;
        self
    }

    /// Set a threshold for reporting excessive time taken to acquire a connection from
    /// the connection pool via [`Pool::acquire()`]. When the threshold is exceeded, a warning is logged.
    ///
    /// Defaults to a value that should not typically be exceeded by the pool enlarging
    /// itself with an additional new connection.
    pub fn acquire_slow_threshold(mut self, threshold: Duration) -> Self {
        self.acquire_slow_threshold = threshold;
        self
    }

    /// Get the threshold for reporting excessive time taken to acquire a connection via
    /// [`Pool::acquire()`].
    pub fn get_acquire_slow_threshold(&self) -> Duration {
        self.acquire_slow_threshold
    }

    /// Set the maximum amount of time to spend waiting for a connection in [`Pool::acquire()`].
    ///
    /// Caps the total amount of time `Pool::acquire()` can spend waiting across multiple phases:
    ///
    /// * First, it may need to wait for a permit from the semaphore, which grants it the privilege
    ///   of opening a connection or popping one from the idle queue.
    /// * If an existing idle connection is acquired, by default it will be checked for liveness
    ///   and integrity before being returned, which may require executing a command on the
    ///   connection. This can be disabled with [`test_before_acquire(false)`][Self::test_before_acquire].
    ///     * If [`before_acquire`][Self::before_acquire] is set, that will also be executed.
    /// * If a new connection needs to be opened, that will obviously require I/O, handshaking,
    ///   and initialization commands.
    ///     * If [`after_connect`][Self::after_connect] is set, that will also be executed.
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// Get the maximum amount of time to spend waiting for a connection in [`Pool::acquire()`].
    pub fn get_acquire_timeout(&self) -> Duration {
        self.acquire_timeout
    }

    /// Set the maximum lifetime of individual connections.
    ///
    /// Any connection with a lifetime greater than this will be closed.
    ///
    /// When set to `None`, all connections live until either reaped by [`idle_timeout`]
    /// or explicitly disconnected.
    ///
    /// Infinite connections are not recommended due to the unfortunate reality of memory/resource
    /// leaks on the database-side. It is better to retire connections periodically
    /// (even if only once daily) to allow the database the opportunity to clean up data structures
    /// (parse trees, query metadata caches, thread-local storage, etc.) that are associated with a
    /// session.
    ///
    /// [`idle_timeout`]: Self::idle_timeout
    pub fn max_lifetime(mut self, lifetime: impl Into<Option<Duration>>) -> Self {
        self.max_lifetime = lifetime.into();
        self
    }

    /// Get the maximum lifetime of individual connections.
    pub fn get_max_lifetime(&self) -> Option<Duration> {
        self.max_lifetime
    }

    /// Set a maximum idle duration for individual connections.
    ///
    /// Any connection that remains in the idle queue longer than this will be closed.
    ///
    /// For usage-based database server billing, this can be a cost saver.
    pub fn idle_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.idle_timeout = timeout.into();
        self
    }

    /// Get the maximum idle duration for individual connections.
    pub fn get_idle_timeout(&self) -> Option<Duration> {
        self.idle_timeout
    }

    /// If true, the health of a connection will be verified by a call to [`Connection::ping`]
    /// before returning the connection.
    ///
    /// Defaults to `true`.
    pub fn test_before_acquire(mut self, test: bool) -> Self {
        self.test_before_acquire = test;
        self
    }

    /// Get whether `test_before_acquire` is currently set.
    pub fn get_test_before_acquire(&self) -> bool {
        self.test_before_acquire
    }

    /// If set to `true`, calls to `acquire()` are fair and connections  are issued
    /// in first-come-first-serve order. If `false`, "drive-by" tasks may steal idle connections
    /// ahead of tasks that have been waiting.
    ///
    /// According to `sqlx-bench/benches/pg_pool` this may slightly increase time
    /// to `acquire()` at low pool contention but at very high contention it helps
    /// avoid tasks at the head of the waiter queue getting repeatedly preempted by
    /// these "drive-by" tasks and tasks further back in the queue timing out because
    /// the queue isn't moving.
    ///
    /// Currently only exposed for benchmarking; `fair = true` seems to be the superior option
    /// in most cases.
    #[doc(hidden)]
    pub fn __fair(mut self, fair: bool) -> Self {
        self.fair = fair;
        self
    }

    /// Run an async callback after a SQLite connection is opened.
    ///
    /// If the callback fails, its error is logged and the connection is closed.
    /// Opening and callback execution are retried with backoff. This also applies to
    /// connections opened internally to maintain [`PoolOptions::min_connections`].
    /// Use `SqliteConnectOptions` for supported connection settings; reserve this
    /// callback for additional application initialization.
    ///
    /// ```
    /// # async fn example() -> blog_storage_sqlite_access::Result<()> {
    /// use blog_storage_sqlite_access::sqlite::SqlitePoolOptions;
    ///
    /// let pool = SqlitePoolOptions::new()
    ///     .after_connect(|connection, _metadata| Box::pin(async move {
    ///         blog_storage_sqlite_access::query("PRAGMA optimize")
    ///             .execute(connection)
    ///             .await?;
    ///         Ok(())
    ///     }))
    ///     .connect("sqlite::memory:")
    ///     .await?;
    /// pool.close().await;
    /// # Ok(())
    /// # }
    /// ```
    pub fn after_connect<F>(mut self, callback: F) -> Self
    where
        // We're passing the `PoolConnectionMetadata` here mostly for future-proofing.
        // `age` and `idle_for` are obviously not useful for fresh connections.
        for<'c> F: Fn(&'c mut DB::Connection, PoolConnectionMetadata) -> BoxFuture<'c, Result<(), Error>>
            + 'static
            + Send
            + Sync,
    {
        self.after_connect = Some(Arc::new(callback));
        self
    }

    /// Inspect a previously idle connection before lending it to the caller.
    ///
    /// The callback receives its age and idle duration in [`PoolConnectionMetadata`].
    /// Return `Ok(true)` to lend it, or `Ok(false)` to close it and try another.
    /// Errors are logged and also close the connection. New connections use
    /// [`PoolOptions::after_connect`] instead.
    ///
    /// For a custom idle check, disable [`PoolOptions::test_before_acquire`] and call
    /// [`crate::Connection::ping`] from this callback when the idle duration exceeds
    /// the application's threshold. The callback returns a boxed future borrowing
    /// the connection.
    pub fn before_acquire<F>(mut self, callback: F) -> Self
    where
        for<'c> F: Fn(&'c mut DB::Connection, PoolConnectionMetadata) -> BoxFuture<'c, Result<bool, Error>>
            + 'static
            + Send
            + Sync,
    {
        self.before_acquire = Some(Arc::new(callback));
        self
    }

    /// Inspect a connection before returning it to the idle pool.
    ///
    /// The callback receives [`PoolConnectionMetadata`]. Return `Ok(true)` to retain
    /// the connection, or `Ok(false)` to close it; an error is logged and also closes
    /// it. A later acquisition can open a replacement subject to pool limits.
    ///
    /// Use this for application-specific connection reuse policy. Prefer
    /// [`PoolOptions::max_lifetime`] when an age limit alone is sufficient.
    pub fn after_release<F>(mut self, callback: F) -> Self
    where
        for<'c> F: Fn(&'c mut DB::Connection, PoolConnectionMetadata) -> BoxFuture<'c, Result<bool, Error>>
            + 'static
            + Send
            + Sync,
    {
        self.after_release = Some(Arc::new(callback));
        self
    }

    /// Set the parent `Pool` from which the new pool will inherit its semaphore.
    ///
    /// This is currently an internal-only API.
    ///
    /// ### Panics
    /// If `self.max_connections` is greater than the setting the given pool was created with,
    /// or `self.fair` differs from the setting the given pool was created with.
    #[doc(hidden)]
    pub fn parent(mut self, pool: Pool<DB>) -> Self {
        self.parent_pool = Some(pool);
        self
    }

    /// Create a new pool from this `PoolOptions` and immediately open at least one connection.
    ///
    /// This ensures the configuration is correct.
    ///
    /// The total number of connections opened is <code>max(1, [min_connections][Self::min_connections])</code>.
    ///
    /// Refer to the relevant `ConnectOptions` impl for your database for the expected URL format:
    ///
    /// * SQLite: [`SqliteConnectOptions`][crate::sqlite::SqliteConnectOptions]
    pub async fn connect(self, url: &str) -> Result<Pool<DB>, Error> {
        self.connect_with(url.parse()?).await
    }

    /// Create a new pool from this `PoolOptions` and immediately open at least one connection.
    ///
    /// This ensures the configuration is correct.
    ///
    /// The total number of connections opened is <code>max(1, [min_connections][Self::min_connections])</code>.
    pub async fn connect_with(
        self,
        options: <DB::Connection as Connection>::Options,
    ) -> Result<Pool<DB>, Error> {
        // Don't take longer than `acquire_timeout` starting from when this is called.
        let deadline = Instant::now() + self.acquire_timeout;

        let inner = PoolInner::new_arc(self, options);

        if inner.options.min_connections > 0 {
            // If the idle reaper is spawned then this will race with the call from that task
            // and may not report any connection errors.
            inner.try_min_connections(deadline).await?;
        }

        // If `min_connections` is nonzero then we'll likely just pull a connection
        // from the idle queue here, but it should at least get tested first.
        let conn = inner.acquire().await?;
        inner.release(conn);

        Ok(Pool(inner))
    }

    /// Create a new pool from this `PoolOptions`, but don't open any connections right now.
    ///
    /// If [`min_connections`][Self::min_connections] is set, a background task will be spawned to
    /// optimistically establish that many connections for the pool.
    ///
    /// Refer to the relevant `ConnectOptions` impl for your database for the expected URL format:
    ///
    /// * SQLite: [`SqliteConnectOptions`][crate::sqlite::SqliteConnectOptions]
    pub fn connect_lazy(self, url: &str) -> Result<Pool<DB>, Error> {
        Ok(self.connect_lazy_with(url.parse()?))
    }

    /// Create a new pool from this `PoolOptions`, but don't open any connections right now.
    ///
    /// If [`min_connections`][Self::min_connections] is set, a background task will be spawned to
    /// optimistically establish that many connections for the pool.
    pub fn connect_lazy_with(self, options: <DB::Connection as Connection>::Options) -> Pool<DB> {
        // `min_connections` is guaranteed by the idle reaper now.
        Pool(PoolInner::new_arc(self, options))
    }
}

impl<DB: Database> Debug for PoolOptions<DB> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolOptions")
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("connect_timeout", &self.acquire_timeout)
            .field("max_lifetime", &self.max_lifetime)
            .field("idle_timeout", &self.idle_timeout)
            .field("test_before_acquire", &self.test_before_acquire)
            .finish()
    }
}
