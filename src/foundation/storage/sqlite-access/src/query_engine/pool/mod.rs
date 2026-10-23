//! Bounded connection pooling for the local SQLite driver.
//!
//! [`Pool`] reuses worker connections and waits fairly when the configured limit
//! is reached. [`crate::SqlitePool`] is the concrete SQLite alias. Configure pool
//! size, acquisition timeout, idle lifetime and busy behavior through
//! [`PoolOptions`] and [`crate::sqlite::SqliteConnectOptions`].
//!
//! ```
//! # async fn example() -> Result<(), blog_storage_sqlite_access::Error> {
//! let pool = blog_storage_sqlite_access::SqlitePool::connect("sqlite::memory:").await?;
//! let value: i64 = blog_storage_sqlite_access::query_scalar("SELECT 1").fetch_one(&pool).await?;
//! assert_eq!(value, 1);
//! pool.close().await;
//! # Ok(())
//! # }
//! ```
//!
//! A pool reference implements [`crate::Executor`]. Acquire a connection for
//! multiple operations on one connection, or begin a transaction when their
//! writes must commit atomically.
use std::fmt;
use std::future::Future;
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::task::{ready, Context, Poll};
use std::time::{Duration, Instant};

use event_listener::EventListener;
use futures::FusedFuture;
use futures::FutureExt;

use crate::query_engine::connection::Connection;
use crate::query_engine::database::Database;
use crate::query_engine::error::Error;
use crate::query_engine::sql_str::SqlSafeStr;
use crate::query_engine::transaction::Transaction;

pub use self::connection::PoolConnection;
use self::inner::PoolInner;
#[doc(hidden)]
pub use self::maybe::MaybePoolConnection;
pub use self::options::{PoolConnectionMetadata, PoolOptions};

#[macro_use]
mod executor;

#[macro_use]
pub mod maybe;

mod connection;
mod inner;
mod options;

/// An asynchronous pool of SQLite worker connections.
///
/// The pool enforces a maximum number of open connections. When all are checked
/// out, [`Pool::acquire`] waits in a fair queue subject to
/// [`PoolOptions::acquire_timeout`]. Each checked-out connection and transaction
/// holds a pool reference.
///
/// Cloning a pool is cheap: handles share reference-counted state. Share one pool
/// across the tasks using a database instead of opening a pool for each request.
/// [`crate::SqlitePool`] is the concrete alias used by this project.
///
/// Call [`Pool::close`] during shutdown to reject waiting and subsequent
/// acquisitions and wait for checked-out connections to return. Dropping the last
/// handle releases local state; it does not provide an async shutdown barrier.
///
/// Connections cache prepared statements. Reusing a connection reuses that cache;
/// the cache is not shared across connections. Parameterized queries are created
/// with [`crate::query()`] and values are supplied through
/// [`crate::query::Query::bind`].
pub struct Pool<DB: Database>(pub(crate) Arc<PoolInner<DB>>);

/// A future that resolves when the pool is closed.
///
/// See [`Pool::close_event()`] for details.
pub struct CloseEvent {
    listener: Option<EventListener>,
}

impl<DB: Database> Pool<DB> {
    /// Create a new connection pool with a default pool configuration and
    /// the given connection URL, and immediately establish one connection.
    ///
    /// Refer to the relevant `ConnectOptions` impl for your database for the expected URL format:
    ///
    /// * SQLite: [`SqliteConnectOptions`][crate::sqlite::SqliteConnectOptions]
    ///
    /// The default configuration is mainly suited for testing and light-duty applications.
    /// For production applications, you'll likely want to make at least few tweaks.
    ///
    /// See [`PoolOptions::new()`] for details.
    pub async fn connect(url: &str) -> Result<Self, Error> {
        PoolOptions::<DB>::new().connect(url).await
    }

    /// Create a new connection pool with a default pool configuration and
    /// the given `ConnectOptions`, and immediately establish one connection.
    ///
    /// The default configuration is mainly suited for testing and light-duty applications.
    /// For production applications, you'll likely want to make at least few tweaks.
    ///
    /// See [`PoolOptions::new()`] for details.
    pub async fn connect_with(
        options: <DB::Connection as Connection>::Options,
    ) -> Result<Self, Error> {
        PoolOptions::<DB>::new().connect_with(options).await
    }

    /// Create a new connection pool with a default pool configuration and
    /// the given connection URL.
    ///
    /// The pool will establish connections only as needed.
    ///
    /// Refer to the relevant [`ConnectOptions`][crate::query_engine::connection::ConnectOptions] impl for your database for the expected URL format:
    ///
    /// * SQLite: [`SqliteConnectOptions`][crate::sqlite::SqliteConnectOptions]
    ///
    /// The default configuration is mainly suited for testing and light-duty applications.
    /// For production applications, you'll likely want to make at least few tweaks.
    ///
    /// See [`PoolOptions::new()`] for details.
    pub fn connect_lazy(url: &str) -> Result<Self, Error> {
        PoolOptions::<DB>::new().connect_lazy(url)
    }

    /// Create a new connection pool with a default pool configuration and
    /// the given `ConnectOptions`.
    ///
    /// The pool will establish connections only as needed.
    ///
    /// The default configuration is mainly suited for testing and light-duty applications.
    /// For production applications, you'll likely want to make at least few tweaks.
    ///
    /// See [`PoolOptions::new()`] for details.
    pub fn connect_lazy_with(options: <DB::Connection as Connection>::Options) -> Self {
        PoolOptions::<DB>::new().connect_lazy_with(options)
    }

    /// Retrieves a connection from the pool.
    ///
    /// The total time this method is allowed to execute is capped by
    /// [`PoolOptions::acquire_timeout`].
    /// If that timeout elapses, this will return [`Error::PoolClosed`].
    ///
    /// ### Note: Cancellation/Timeout May Drop Connections
    /// If `acquire` is cancelled or times out after it acquires a connection from the idle queue or
    /// opens a new one, it will drop that connection because we don't want to assume it
    /// is safe to return to the pool, and testing it to see if it's safe to release could introduce
    /// subtle bugs if not implemented correctly. To avoid that entirely, we've decided to not
    /// gracefully handle cancellation here.
    ///
    /// However, if your workload is sensitive to dropped connections such as using an in-memory
    /// SQLite database with a pool size of 1, you can pretty easily ensure that a cancelled
    /// `acquire()` call will never drop connections by tweaking your [`PoolOptions`]:
    ///
    /// * Set [`test_before_acquire(false)`][PoolOptions::test_before_acquire]
    /// * Never set [`before_acquire`][PoolOptions::before_acquire] or
    ///   [`after_connect`][PoolOptions::after_connect].
    ///
    /// This should eliminate any potential `.await` points between acquiring a connection and
    /// returning it.
    pub fn acquire(&self) -> impl Future<Output = Result<PoolConnection<DB>, Error>> + 'static {
        let shared = self.0.clone();
        async move { shared.acquire().await.map(|conn| conn.reattach()) }
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` immediately if there are no idle connections available in the pool
    /// or there are tasks waiting for a connection which have yet to wake.
    pub fn try_acquire(&self) -> Option<PoolConnection<DB>> {
        self.0.try_acquire().map(|conn| conn.into_live().reattach())
    }

    /// Retrieves a connection and immediately begins a new transaction.
    pub async fn begin(&self) -> Result<Transaction<'static, DB>, Error> {
        Transaction::begin(
            MaybePoolConnection::PoolConnection(self.acquire().await?),
            None,
        )
        .await
    }

    /// Attempts to retrieve a connection and immediately begins a new transaction if successful.
    pub async fn try_begin(&self) -> Result<Option<Transaction<'static, DB>>, Error> {
        match self.try_acquire() {
            Some(conn) => Transaction::begin(MaybePoolConnection::PoolConnection(conn), None)
                .await
                .map(Some),

            None => Ok(None),
        }
    }

    /// Retrieves a connection and immediately begins a new transaction using `statement`.
    pub async fn begin_with(
        &self,
        statement: impl SqlSafeStr,
    ) -> Result<Transaction<'static, DB>, Error> {
        Transaction::begin(
            MaybePoolConnection::PoolConnection(self.acquire().await?),
            Some(statement.into_sql_str()),
        )
        .await
    }

    /// Attempts to retrieve a connection and, if successful, immediately begins a new
    /// transaction using `statement`.
    pub async fn try_begin_with(
        &self,
        statement: impl SqlSafeStr,
    ) -> Result<Option<Transaction<'static, DB>>, Error> {
        match self.try_acquire() {
            Some(conn) => Transaction::begin(
                MaybePoolConnection::PoolConnection(conn),
                Some(statement.into_sql_str()),
            )
            .await
            .map(Some),

            None => Ok(None),
        }
    }

    /// Shut down the connection pool, immediately waking all tasks waiting for a connection.
    ///
    /// Upon calling this method, any currently waiting or subsequent calls to [`Pool::acquire`] and
    /// the like will immediately return [`Error::PoolClosed`] and no new connections will be opened.
    /// Checked-out connections are unaffected, but will be gracefully closed on-drop
    /// rather than being returned to the pool.
    ///
    /// Returns a `Future` which can be `.await`ed to ensure all connections are
    /// gracefully closed. It will first close any idle connections currently waiting in the pool,
    /// then wait for all checked-out connections to be returned or closed.
    ///
    /// Waiting for connections to be gracefully closed is optional, but will allow the database
    /// server to clean up the resources sooner rather than later. This is especially important
    /// for tests that create a new pool every time, otherwise you may see errors about connection
    /// limits being exhausted even when running tests in a single thread.
    ///
    /// If the returned `Future` is not run to completion, any remaining connections will be dropped
    /// when the last handle for the given pool instance is dropped, which could happen in a task
    /// spawned by `Pool` internally and so may be unpredictable otherwise.
    ///
    /// `.close()` may be safely called and `.await`ed on multiple handles concurrently.
    pub fn close(&self) -> impl Future<Output = ()> + '_ {
        self.0.close()
    }

    /// Returns `true` if [`.close()`][Pool::close] has been called on the pool, `false` otherwise.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    /// Return a future that resolves when [`Pool::close`] is called.
    ///
    /// An already-closed pool returns an immediately ready future. A task holding a
    /// connection can observe this event and stop long-running work, returning its
    /// connection so shutdown can complete. Dropping a query future does not undo an
    /// already committed statement.
    ///
    /// ```
    /// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) {
    /// let observer = pool.clone();
    /// let task = tokio::spawn(async move { observer.close_event().await });
    /// pool.close().await;
    /// task.await.unwrap();
    /// # }
    /// ```
    pub fn close_event(&self) -> CloseEvent {
        self.0.close_event()
    }

    /// Returns the number of connections currently active. This includes idle connections.
    pub fn size(&self) -> u32 {
        self.0.size()
    }

    /// Returns the number of connections active and idle (not in use).
    pub fn num_idle(&self) -> usize {
        self.0.num_idle()
    }

    /// Gets a clone of the connection options for this pool
    pub fn connect_options(&self) -> Arc<<DB::Connection as Connection>::Options> {
        self.0
            .connect_options
            .read()
            .expect("write-lock holder panicked")
            .clone()
    }

    /// Updates the connection options this pool will use when opening any future connections.  Any
    /// existing open connection in the pool will be left as-is.
    pub fn set_connect_options(&self, connect_options: <DB::Connection as Connection>::Options) {
        // technically write() could also panic if the current thread already holds the lock,
        // but because this method can't be re-entered by the same thread that shouldn't be a problem
        let mut guard = self
            .0
            .connect_options
            .write()
            .expect("write-lock holder panicked");
        *guard = Arc::new(connect_options);
    }

    /// Get the options for this pool
    pub fn options(&self) -> &PoolOptions<DB> {
        &self.0.options
    }
}

/// Returns a new [Pool] tied to the same shared connection pool.
impl<DB: Database> Clone for Pool<DB> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<DB: Database> fmt::Debug for Pool<DB> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Pool")
            .field("size", &self.0.size())
            .field("num_idle", &self.0.num_idle())
            .field("is_closed", &self.0.is_closed())
            .field("options", &self.0.options)
            .finish()
    }
}

impl CloseEvent {
    /// Execute the given future until it returns or the pool is closed.
    ///
    /// Cancels the future and returns `Err(PoolClosed)` if/when the pool is closed.
    /// If the pool was already closed, the future is never run.
    pub async fn do_until<Fut: Future>(&mut self, fut: Fut) -> Result<Fut::Output, Error> {
        // Check that the pool wasn't closed already.
        //
        // We use `poll_immediate()` as it will use the correct waker instead of
        // a no-op one like `.now_or_never()`, but it won't actually suspend execution here.
        futures::future::poll_immediate(&mut *self)
            .await
            .map_or(Ok(()), |_| Err(Error::PoolClosed))?;

        let mut fut = pin!(fut);

        // I find that this is clearer in intent than `futures::future::select()`
        // or `futures::select_biased!{}` (which isn't enabled anyway).
        std::future::poll_fn(|cx| {
            // Poll `fut` first as the wakeup event is more likely for it than `self`.
            if let Poll::Ready(ret) = fut.as_mut().poll(cx) {
                return Poll::Ready(Ok(ret));
            }

            // Can't really factor out mapping to `Err(Error::PoolClosed)` though it seems like
            // we should because that results in a different `Ok` type each time.
            //
            // Ideally we'd map to something like `Result<!, Error>` but using `!` as a type
            // is not allowed on stable Rust yet.
            self.poll_unpin(cx).map(|_| Err(Error::PoolClosed))
        })
        .await
    }
}

impl Future for CloseEvent {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(listener) = &mut self.listener {
            ready!(listener.poll_unpin(cx));
        }

        // `EventListener` doesn't like being polled after it yields, and even if it did it
        // would probably just wait for the next event, neither of which we want.
        //
        // So this way, once we get our close event, we fuse this future to immediately return.
        self.listener = None;

        Poll::Ready(())
    }
}

impl FusedFuture for CloseEvent {
    fn is_terminated(&self) -> bool {
        self.listener.is_none()
    }
}

/// get the time between the deadline and now and use that as our timeout
///
/// returns `Error::PoolTimedOut` if the deadline is in the past
fn deadline_as_timeout(deadline: Instant) -> Result<Duration, Error> {
    deadline
        .checked_duration_since(Instant::now())
        .ok_or(Error::PoolTimedOut)
}

#[test]
fn assert_pool_traits() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_clone<T: Clone>() {}

    assert_send_sync::<Pool<crate::sqlite::Sqlite>>();
    assert_clone::<Pool<crate::sqlite::Sqlite>>();
}
