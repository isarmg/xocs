//! Format tracing spans and events as line-oriented logs.
//!
//! Initialize the formatter once, in the executable:
//!
//! ```rust
//! tracing::collector::fmt().with_max_level(tracing::Level::INFO).init();
//! ```
//!
//! Libraries emit spans and events without installing a subscriber. Use
//! [`SubscriberBuilder`] to configure fields, output writers and timestamps.
//! [`format::Full`] and [`format::Compact`] produce single-line output; the
//! `ansi` feature adds the `Pretty` formatter, and `json` adds newline-delimited
//! JSON output. Custom formatters implement [`FormatEvent`] and [`FormatFields`].
//!
//! Target-specific filtering uses [`Targets`](super::filter::Targets). For example:
//!
//! ```rust
//! use tracing::collector::{filter::Targets, prelude::*};
//! use tracing::Level;
//!
//! tracing::collector::registry()
//!     .with(tracing::collector::fmt::layer())
//!     .with(Targets::new().with_default(Level::INFO).with_target("my_app", Level::DEBUG))
//!     .init();
//! ```
//!
//! [`try_init`] and [`init`] read `RUST_LOG` as comma-separated target/level
//! directives. The target matcher has no span-field expression syntax.

use alloc::boxed::Box;
use core::any::TypeId;
use std::{error::Error, io};
use tracing::primitives::{span, subscriber::Interest, Event, Metadata};

mod fmt_layer;
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
pub mod format;
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
pub mod time;
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
pub mod writer;

pub use fmt_layer::{FmtContext, FormattedFields, Layer};

use crate::collector::layer::Layer as _;
use crate::collector::util::SubscriberInitExt;
use crate::collector::{
    filter::LevelFilter,
    layer,
    registry::{LookupSpan, Registry},
};

#[doc(inline)]
pub use self::{
    format::{format, FormatEvent, FormatFields},
    time::time,
    writer::{MakeWriter, TestWriter},
};

/// A `Subscriber` that logs formatted representations of `tracing` events.
///
/// This consists of an inner `Formatter` wrapped in a layer that performs filtering.
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
#[derive(Debug)]
pub struct Subscriber<
    N = format::DefaultFields,
    E = format::Format<format::Full>,
    F = LevelFilter,
    W = fn() -> io::Stdout,
> {
    inner: layer::Layered<F, Formatter<N, E, W>>,
}

/// A `Subscriber` that logs formatted representations of `tracing` events.
/// This type only logs formatted events; it does not perform any filtering.
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
pub type Formatter<
    N = format::DefaultFields,
    E = format::Format<format::Full>,
    W = fn() -> io::Stdout,
> = layer::Layered<fmt_layer::Layer<Registry, N, E, W>, Registry>;

/// Configures and constructs `Subscriber`s.
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
#[derive(Debug)]
#[must_use]
pub struct SubscriberBuilder<
    N = format::DefaultFields,
    E = format::Format<format::Full>,
    F = LevelFilter,
    W = fn() -> io::Stdout,
> {
    filter: F,
    inner: Layer<Registry, N, E, W>,
}

/// Returns a new [`SubscriberBuilder`] for configuring a [formatting subscriber].
///
/// This is essentially shorthand for [`SubscriberBuilder::default()]`.
///
/// # Examples
///
/// Using [`init`] to set the default subscriber:
///
/// ```rust
/// tracing::collector::fmt().init();
/// ```
///
/// Configuring the output format:
///
/// ```rust
///
/// tracing::collector::fmt()
///     // Configure formatting settings.
///     .with_target(false)
///     .with_timer(tracing::collector::fmt::time::uptime())
///     .with_level(true)
///     // Set the subscriber as the default.
///     .init();
/// ```
///
/// [`try_init`] returns an error if the default subscriber could not be set:
///
/// ```rust
/// use std::error::Error;
///
/// fn init_subscriber() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
///     tracing::collector::fmt()
///         // Configure the subscriber to emit logs in JSON format.
///         .json()
///         // Configure the subscriber to flatten event fields in the output JSON objects.
///         .flatten_event(true)
///         // Set the subscriber as the default, returning an error if this fails.
///         .try_init()?;
///
///     Ok(())
/// }
/// ```
///
/// Rather than setting the subscriber as the default, [`finish`] _returns_ the
/// constructed subscriber, which may then be passed to other functions:
///
/// ```rust
/// let subscriber = tracing::collector::fmt()
///     .with_max_level(tracing::Level::DEBUG)
///     .compact()
///     .finish();
///
/// tracing::subscriber::with_default(subscriber, || {
///     // the subscriber will only be set as the default
///     // inside this closure...
/// })
/// ```
///
/// [formatting subscriber]: Subscriber
/// [`SubscriberBuilder::default()`]: SubscriberBuilder::default
/// [`init`]: SubscriberBuilder::init()
/// [`try_init`]: SubscriberBuilder::try_init()
/// [`finish`]: SubscriberBuilder::finish()
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
pub fn fmt() -> SubscriberBuilder {
    SubscriberBuilder::default()
}

/// Returns a new [formatting layer] that can be [composed] with other layers to
/// construct a [`Subscriber`].
///
/// This is a shorthand for the equivalent [`Layer::default()`] function.
///
/// [formatting layer]: Layer
/// [composed]: crate::collector::layer
/// [`Layer::default()`]: Layer::default
#[cfg_attr(docsrs, doc(cfg(all(feature = "fmt", feature = "std"))))]
pub fn layer<S>() -> Layer<S> {
    Layer::default()
}

impl Subscriber {
    /// The maximum [verbosity level] that is enabled by a `Subscriber` by
    /// default.
    ///
    /// This can be overridden with the [`SubscriberBuilder::with_max_level`] method.
    ///
    /// [verbosity level]: tracing::primitives::Level
    /// [`SubscriberBuilder::with_max_level`]: SubscriberBuilder::with_max_level
    pub const DEFAULT_MAX_LEVEL: LevelFilter = LevelFilter::INFO;

    /// Returns a new `SubscriberBuilder` for configuring a format subscriber.
    pub fn builder() -> SubscriberBuilder {
        SubscriberBuilder::default()
    }

    /// Returns a new format subscriber with the default configuration.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Subscriber {
    fn default() -> Self {
        SubscriberBuilder::default().finish()
    }
}

// === impl Subscriber ===

impl<N, E, F, W> tracing::primitives::Subscriber for Subscriber<N, E, F, W>
where
    N: for<'writer> FormatFields<'writer> + 'static,
    E: FormatEvent<Registry, N> + 'static,
    F: layer::Layer<Formatter<N, E, W>> + 'static,
    W: for<'writer> MakeWriter<'writer> + 'static,
    layer::Layered<F, Formatter<N, E, W>>: tracing::primitives::Subscriber,
    fmt_layer::Layer<Registry, N, E, W>: layer::Layer<Registry>,
{
    #[inline]
    fn register_callsite(&self, meta: &'static Metadata<'static>) -> Interest {
        self.inner.register_callsite(meta)
    }

    #[inline]
    fn enabled(&self, meta: &Metadata<'_>) -> bool {
        self.inner.enabled(meta)
    }

    #[inline]
    fn new_span(&self, attrs: &span::Attributes<'_>) -> span::Id {
        self.inner.new_span(attrs)
    }

    #[inline]
    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        self.inner.record(span, values)
    }

    #[inline]
    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        self.inner.record_follows_from(span, follows)
    }

    #[inline]
    fn event_enabled(&self, event: &Event<'_>) -> bool {
        self.inner.event_enabled(event)
    }

    #[inline]
    fn event(&self, event: &Event<'_>) {
        self.inner.event(event);
    }

    #[inline]
    fn enter(&self, id: &span::Id) {
        // TODO: add on_enter hook
        self.inner.enter(id);
    }

    #[inline]
    fn exit(&self, id: &span::Id) {
        self.inner.exit(id);
    }

    #[inline]
    fn current_span(&self) -> span::Current {
        self.inner.current_span()
    }

    #[inline]
    fn clone_span(&self, id: &span::Id) -> span::Id {
        self.inner.clone_span(id)
    }

    #[inline]
    fn try_close(&self, id: span::Id) -> bool {
        self.inner.try_close(id)
    }

    #[inline]
    fn max_level_hint(&self) -> Option<tracing::primitives::LevelFilter> {
        self.inner.max_level_hint()
    }

    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        if id == TypeId::of::<Self>() {
            Some(self as *const Self as *const ())
        } else {
            unsafe { self.inner.downcast_raw(id) }
        }
    }
}

impl<'a, N, E, F, W> LookupSpan<'a> for Subscriber<N, E, F, W>
where
    layer::Layered<F, Formatter<N, E, W>>: LookupSpan<'a>,
{
    type Data = <layer::Layered<F, Formatter<N, E, W>> as LookupSpan<'a>>::Data;

    fn span_data(&'a self, id: &span::Id) -> Option<Self::Data> {
        self.inner.span_data(id)
    }
}

// ===== impl SubscriberBuilder =====

impl Default for SubscriberBuilder {
    fn default() -> Self {
        SubscriberBuilder {
            filter: Subscriber::DEFAULT_MAX_LEVEL,
            inner: Default::default(),
        }
        .log_internal_errors(true)
    }
}

impl<N, E, F, W> SubscriberBuilder<N, E, F, W>
where
    N: for<'writer> FormatFields<'writer> + 'static,
    E: FormatEvent<Registry, N> + 'static,
    W: for<'writer> MakeWriter<'writer> + 'static,
    F: layer::Layer<Formatter<N, E, W>> + Send + Sync + 'static,
    fmt_layer::Layer<Registry, N, E, W>: layer::Layer<Registry> + Send + Sync + 'static,
{
    /// Finish the builder, returning a new `FmtSubscriber`.
    pub fn finish(self) -> Subscriber<N, E, F, W> {
        let subscriber = self.inner.with_subscriber(Registry::default());
        Subscriber {
            inner: self.filter.with_subscriber(subscriber),
        }
    }

    /// Install this Subscriber as the global default if one is
    /// not already set.
    ///
    /// If the `log-tracer` feature is enabled, this will also install
    /// the LogTracer to convert `Log` records into `tracing` `Event`s.
    ///
    /// # Errors
    /// Returns an Error if the initialization was unsuccessful, likely
    /// because a global subscriber was already installed by another
    /// call to `try_init`.
    pub fn try_init(self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        use crate::collector::util::SubscriberInitExt;
        self.finish().try_init()?;

        Ok(())
    }

    /// Install this Subscriber as the global default.
    ///
    /// If the `log-tracer` feature is enabled, this will also install
    /// the LogTracer to convert `Log` records into `tracing` `Event`s.
    ///
    /// # Panics
    /// Panics if the initialization was unsuccessful, likely because a
    /// global subscriber was already installed by another call to `try_init`.
    pub fn init(self) {
        self.try_init()
            .expect("Unable to install global subscriber")
    }
}

impl<N, E, F, W> From<SubscriberBuilder<N, E, F, W>> for tracing::primitives::Dispatch
where
    N: for<'writer> FormatFields<'writer> + 'static,
    E: FormatEvent<Registry, N> + 'static,
    W: for<'writer> MakeWriter<'writer> + 'static,
    F: layer::Layer<Formatter<N, E, W>> + Send + Sync + 'static,
    fmt_layer::Layer<Registry, N, E, W>: layer::Layer<Registry> + Send + Sync + 'static,
{
    fn from(builder: SubscriberBuilder<N, E, F, W>) -> tracing::primitives::Dispatch {
        tracing::primitives::Dispatch::new(builder.finish())
    }
}

impl<N, L, T, F, W> SubscriberBuilder<N, format::Format<L, T>, F, W>
where
    N: for<'writer> FormatFields<'writer> + 'static,
{
    /// Use the given [`timer`] for log message timestamps.
    ///
    /// See the [`time` module] for the provided timer implementations.
    ///
    /// The provided timers report wall-clock time or elapsed time. Implement
    /// the timer trait to supply a different timestamp format.
    ///
    /// [`timer`]: time::FormatTime
    /// [`time` module]: mod@time
    pub fn with_timer<T2>(self, timer: T2) -> SubscriberBuilder<N, format::Format<L, T2>, F, W> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.with_timer(timer),
        }
    }

    /// Do not emit timestamps with log messages.
    pub fn without_time(self) -> SubscriberBuilder<N, format::Format<L, ()>, F, W> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.without_time(),
        }
    }

    /// Configures how synthesized events are emitted at points in the [span
    /// lifecycle][lifecycle].
    ///
    /// The following options are available:
    ///
    /// - `FmtSpan::NONE`: No events will be synthesized when spans are
    ///   created, entered, exited, or closed. Data from spans will still be
    ///   included as the context for formatted events. This is the default.
    /// - `FmtSpan::NEW`: An event will be synthesized when spans are created.
    /// - `FmtSpan::ENTER`: An event will be synthesized when spans are entered.
    /// - `FmtSpan::EXIT`: An event will be synthesized when spans are exited.
    /// - `FmtSpan::CLOSE`: An event will be synthesized when a span closes. If
    ///   [timestamps are enabled][time] for this formatter, the generated
    ///   event will contain fields with the span's _busy time_ (the total
    ///   time for which it was entered) and _idle time_ (the total time that
    ///   the span existed but was not entered).
    /// - `FmtSpan::ACTIVE`: An event will be synthesized when spans are entered
    ///   or exited.
    /// - `FmtSpan::FULL`: Events will be synthesized whenever a span is
    ///   created, entered, exited, or closed. If timestamps are enabled, the
    ///   close event will contain the span's busy and idle time, as
    ///   described above.
    ///
    /// The options can be enabled in any combination. For instance, the following
    /// will synthesize events whenever spans are created and closed:
    ///
    /// ```rust
    /// use tracing::collector::fmt::format::FmtSpan;
    /// use tracing::collector::fmt;
    ///
    /// let subscriber = fmt()
    ///     .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    ///     .finish();
    /// ```
    ///
    /// Note that the generated events will only be part of the log output by
    /// this formatter; they will not be recorded by other `Subscriber`s or by
    /// `Layer`s added to this subscriber.
    ///
    /// [lifecycle]: https://docs.rs/tracing/latest/foundation/diagnostics/structured-events/legacy/tracing/span/index.html#the-span-lifecycle
    /// [time]: SubscriberBuilder::without_time()
    pub fn with_span_events(self, kind: format::FmtSpan) -> Self {
        SubscriberBuilder {
            inner: self.inner.with_span_events(kind),
            ..self
        }
    }

    /// Sets whether or not the formatter emits ANSI terminal escape codes
    /// for colors and other text formatting.
    ///
    /// Enabling ANSI escapes (calling `with_ansi(true)`) requires the "ansi"
    /// crate feature flag. Calling `with_ansi(true)` without the "ansi"
    /// feature flag enabled will panic if debug assertions are enabled, or
    /// print a warning otherwise.
    ///
    /// This method itself is still available without the feature flag. This
    /// is to allow ANSI escape codes to be explicitly *disabled* without
    /// having to opt-in to the dependencies required to emit ANSI formatting.
    /// This way, code which constructs a formatter that should never emit
    /// ANSI escape codes can ensure that they are not used, regardless of
    /// whether or not other crates in the dependency graph enable the "ansi"
    /// feature flag.
    pub fn with_ansi(self, ansi: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_ansi(ansi),
            ..self
        }
    }

    /// Sets whether ANSI control character sanitization is enabled.
    ///
    /// This defaults to `true` as a protective measure against terminal
    /// injection attacks. If this is set to `false`, ANSI sanitization is
    /// disabled and trusted ANSI control sequences in logged values are passed
    /// through unchanged.
    pub fn with_ansi_sanitization(
        self,
        ansi_sanitization: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_ansi_sanitization(ansi_sanitization),
            ..self
        }
    }

    /// Sets whether to write errors from [`FormatEvent`] to the writer.
    /// Defaults to true.
    ///
    /// By default, `fmt::Layer` will write any `FormatEvent`-internal errors to
    /// the writer. These errors are unlikely and will only occur if there is a
    /// bug in the `FormatEvent` implementation or its dependencies.
    ///
    /// If writing to the writer fails, the error message is printed to stderr
    /// as a fallback.
    ///
    /// [`FormatEvent`]: crate::collector::fmt::FormatEvent
    pub fn log_internal_errors(
        self,
        log_internal_errors: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.log_internal_errors(log_internal_errors),
            ..self
        }
    }

    /// Sets whether or not an event's target is displayed.
    pub fn with_target(
        self,
        display_target: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_target(display_target),
            ..self
        }
    }

    /// Sets whether or not an event's [source code file path][file] is
    /// displayed.
    ///
    /// [file]: tracing::primitives::Metadata::file
    pub fn with_file(
        self,
        display_filename: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_file(display_filename),
            ..self
        }
    }

    /// Sets whether or not an event's [source code line number][line] is
    /// displayed.
    ///
    /// [line]: tracing::primitives::Metadata::line
    pub fn with_line_number(
        self,
        display_line_number: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_line_number(display_line_number),
            ..self
        }
    }

    /// Sets whether or not an event's level is displayed.
    pub fn with_level(
        self,
        display_level: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_level(display_level),
            ..self
        }
    }

    /// Sets whether or not the [name] of the current thread is displayed
    /// when formatting events.
    ///
    /// [name]: std::thread#naming-threads
    pub fn with_thread_names(
        self,
        display_thread_names: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_thread_names(display_thread_names),
            ..self
        }
    }

    /// Sets whether or not the [thread ID] of the current thread is displayed
    /// when formatting events.
    ///
    /// [thread ID]: std::thread::ThreadId
    pub fn with_thread_ids(
        self,
        display_thread_ids: bool,
    ) -> SubscriberBuilder<N, format::Format<L, T>, F, W> {
        SubscriberBuilder {
            inner: self.inner.with_thread_ids(display_thread_ids),
            ..self
        }
    }

    /// Sets the subscriber being built to use a less verbose formatter.
    ///
    /// See [`format::Compact`].
    pub fn compact(self) -> SubscriberBuilder<N, format::Format<format::Compact, T>, F, W>
    where
        N: for<'writer> FormatFields<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.compact(),
        }
    }

    /// Sets the subscriber being built to use an [excessively pretty, human-readable formatter](crate::collector::fmt::format::Pretty).
    #[cfg(feature = "ansi")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ansi")))]
    pub fn pretty(
        self,
    ) -> SubscriberBuilder<format::Pretty, format::Format<format::Pretty, T>, F, W> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.pretty(),
        }
    }

    /// Sets the subscriber being built to use a JSON formatter.
    ///
    /// See [`format::Json`] for details.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn json(
        self,
    ) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W>
    where
        N: for<'writer> FormatFields<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.json(),
        }
    }
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
impl<T, F, W> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W> {
    /// Sets the json subscriber being built to flatten event metadata.
    ///
    /// See [`format::Json`] for details.
    pub fn flatten_event(
        self,
        flatten_event: bool,
    ) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.flatten_event(flatten_event),
        }
    }

    /// Sets whether or not the JSON subscriber being built will include the current span
    /// in formatted events.
    ///
    /// See [`format::Json`] for details.
    pub fn with_current_span(
        self,
        display_current_span: bool,
    ) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.with_current_span(display_current_span),
        }
    }

    /// Sets whether or not the JSON subscriber being built will include a list (from
    /// root to leaf) of all currently entered spans in formatted events.
    ///
    /// See [`format::Json`] for details.
    pub fn with_span_list(
        self,
        display_span_list: bool,
    ) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.with_span_list(display_span_list),
        }
    }
}

impl<N, E, F, W> SubscriberBuilder<N, E, F, W> {
    /// Sets the field formatter that the subscriber being built will use to record
    /// fields.
    ///
    /// For example:
    /// ```rust
    /// use tracing::collector::fmt::format;
    /// use tracing::collector::prelude::*;
    ///
    /// let formatter =
    ///     // Construct a custom formatter for `Debug` fields
    ///     format::debug_fn(|writer, field, value| write!(writer, "{}: {:?}", field, value))
    ///         // Use the `tracing::collector::MakeFmtExt` trait to wrap the
    ///         // formatter so that a delimiter is added between fields.
    ///         .delimited(", ");
    ///
    /// let subscriber = tracing::collector::fmt()
    ///     .fmt_fields(formatter)
    ///     .finish();
    /// # drop(subscriber)
    /// ```
    pub fn fmt_fields<N2>(self, fmt_fields: N2) -> SubscriberBuilder<N2, E, F, W>
    where
        N2: for<'writer> FormatFields<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.fmt_fields(fmt_fields),
        }
    }

    /// Sets the maximum [verbosity level] that will be enabled by the
    /// subscriber.
    ///
    /// This replaces the builder's existing filter with the given maximum
    /// level.
    ///
    /// # Examples
    ///
    /// Enable up to the `DEBUG` verbosity level:
    /// ```rust
    /// use tracing::collector::fmt;
    /// use tracing::Level;
    ///
    /// fmt()
    ///     .with_max_level(Level::DEBUG)
    ///     .init();
    /// ```
    /// This subscriber won't record any spans or events!
    /// ```rust
    /// use tracing::collector::{fmt, filter::LevelFilter};
    ///
    /// let subscriber = fmt()
    ///     .with_max_level(LevelFilter::OFF)
    ///     .finish();
    /// ```
    /// [verbosity level]: tracing::primitives::Level
    pub fn with_max_level(
        self,
        filter: impl Into<LevelFilter>,
    ) -> SubscriberBuilder<N, E, LevelFilter, W> {
        let filter = filter.into();
        SubscriberBuilder {
            filter,
            inner: self.inner,
        }
    }

    /// Sets the [event formatter][`FormatEvent`] that the subscriber being built
    /// will use to format events that occur.
    ///
    /// The event formatter may be any type implementing the [`FormatEvent`]
    /// trait, which is implemented for all functions taking a [`FmtContext`], a
    /// [`Writer`], and an [`Event`].
    ///
    /// # Examples
    ///
    /// Setting a type implementing [`FormatEvent`] as the formatter:
    ///
    /// ```rust
    /// use tracing::collector::fmt::format;
    ///
    /// let subscriber = tracing::collector::fmt()
    ///     .event_format(format().compact())
    ///     .finish();
    /// ```
    ///
    /// [`Writer`]: struct@self::format::Writer
    pub fn event_format<E2>(self, fmt_event: E2) -> SubscriberBuilder<N, E2, F, W>
    where
        E2: FormatEvent<Registry, N> + 'static,
        N: for<'writer> FormatFields<'writer> + 'static,
        W: for<'writer> MakeWriter<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.event_format(fmt_event),
        }
    }

    /// Sets the [`MakeWriter`] that the subscriber being built will use to write events.
    ///
    /// # Examples
    ///
    /// Using `stderr` rather than `stdout`:
    ///
    /// ```rust
    /// use tracing::collector::fmt;
    /// use std::io;
    ///
    /// fmt()
    ///     .with_writer(io::stderr)
    ///     .init();
    /// ```
    pub fn with_writer<W2>(self, make_writer: W2) -> SubscriberBuilder<N, E, F, W2>
    where
        W2: for<'writer> MakeWriter<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.with_writer(make_writer),
        }
    }

    /// Configures the subscriber to support [`libtest`'s output capturing][capturing] when used in
    /// unit tests.
    ///
    /// See [`TestWriter`] for additional details.
    ///
    /// # Examples
    ///
    /// Using [`TestWriter`] to let `cargo test` capture test output. Note that we do not install it
    /// globally as it may cause conflicts.
    ///
    /// ```rust
    /// use tracing::collector::fmt;
    /// use tracing::subscriber;
    ///
    /// subscriber::set_default(
    ///     fmt()
    ///         .with_test_writer()
    ///         .finish()
    /// );
    /// ```
    ///
    /// [capturing]:
    /// https://doc.rust-lang.org/book/ch11-02-running-tests.html#showing-function-output
    /// [`TestWriter`]: writer::TestWriter
    pub fn with_test_writer(self) -> SubscriberBuilder<N, E, F, TestWriter> {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.with_writer(TestWriter::default()),
        }
    }

    /// Updates the event formatter by applying a function to the existing event formatter.
    ///
    /// This sets the event formatter that the subscriber being built will use to record fields.
    ///
    /// # Examples
    ///
    /// Updating an event formatter:
    ///
    /// ```rust
    /// let subscriber = tracing::collector::fmt()
    ///     .map_event_format(|e| e.compact())
    ///     .finish();
    /// ```
    pub fn map_event_format<E2>(self, f: impl FnOnce(E) -> E2) -> SubscriberBuilder<N, E2, F, W>
    where
        E2: FormatEvent<Registry, N> + 'static,
        N: for<'writer> FormatFields<'writer> + 'static,
        W: for<'writer> MakeWriter<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.map_event_format(f),
        }
    }

    /// Updates the field formatter by applying a function to the existing field formatter.
    ///
    /// This sets the field formatter that the subscriber being built will use to record fields.
    ///
    /// # Examples
    ///
    /// Updating a field formatter:
    ///
    /// ```rust
    /// use tracing::collector::field::MakeExt;
    /// let subscriber = tracing::collector::fmt()
    ///     .map_fmt_fields(|f| f.debug_alt())
    ///     .finish();
    /// ```
    pub fn map_fmt_fields<N2>(self, f: impl FnOnce(N) -> N2) -> SubscriberBuilder<N2, E, F, W>
    where
        N2: for<'writer> FormatFields<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.map_fmt_fields(f),
        }
    }

    /// Updates the [`MakeWriter`] by applying a function to the existing [`MakeWriter`].
    ///
    /// This sets the [`MakeWriter`] that the subscriber being built will use to write events.
    ///
    /// # Examples
    ///
    /// Redirect output to stderr if level is <= WARN:
    ///
    /// ```rust
    /// use tracing::Level;
    /// use tracing::collector::fmt::{self, writer::MakeWriterExt};
    ///
    /// let stderr = std::io::stderr.with_max_level(Level::WARN);
    /// let layer = tracing::collector::fmt()
    ///     .map_writer(move |w| stderr.or_else(w))
    ///     .finish();
    /// ```
    pub fn map_writer<W2>(self, f: impl FnOnce(W) -> W2) -> SubscriberBuilder<N, E, F, W2>
    where
        W2: for<'writer> MakeWriter<'writer> + 'static,
    {
        SubscriberBuilder {
            filter: self.filter,
            inner: self.inner.map_writer(f),
        }
    }
}

/// Install a global tracing subscriber that listens for events and
/// filters using target/level directives from `RUST_LOG`,
/// if one is not already set.
///
/// If the `log-tracer` feature is enabled, this will also install
/// the [`LogTracer`] to convert `log` records into `tracing` `Event`s.
///
/// `RUST_LOG` is parsed by [`Targets`](super::filter::Targets). Without it,
/// the default level is `INFO`; invalid directives are reported on stderr.
///
///
/// # Errors
///
/// Returns an Error if the initialization was unsuccessful,
/// likely because a global subscriber was already installed by another
/// call to `try_init`.
///
/// [`LogTracer`]:
///     https://docs.rs/tracing-log/0.1.0/tracing_log/struct.LogTracer.html
pub fn try_init() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let builder = Subscriber::builder();

    // If `env-filter` is disabled, remove the default max level filter from the
    // subscriber; it will be added to the `Targets` filter instead if no filter
    // is set in `RUST_LOG`.
    // Replacing the default `LevelFilter` with an `EnvFilter` would imply this,
    // but we can't replace the builder's filter with a `Targets` filter yet.
    #[cfg(not(any()))]
    let builder = builder.with_max_level(LevelFilter::TRACE);

    let subscriber = builder.finish();
    #[cfg(not(any()))]
    let subscriber = {
        use crate::collector::{filter::Targets, layer::SubscriberExt};
        use std::{env, eprintln, str::FromStr};
        let targets = match env::var("RUST_LOG") {
            Ok(var) => Targets::from_str(&var)
                .map_err(|e| {
                    eprintln!("Ignoring `RUST_LOG={:?}`: {}", var, e);
                })
                .unwrap_or_default(),
            Err(env::VarError::NotPresent) => {
                Targets::new().with_default(Subscriber::DEFAULT_MAX_LEVEL)
            }
            Err(e) => {
                eprintln!("Ignoring `RUST_LOG`: {}", e);
                Targets::new().with_default(Subscriber::DEFAULT_MAX_LEVEL)
            }
        };
        subscriber.with(targets)
    };

    subscriber.try_init().map_err(Into::into)
}

/// Install a global tracing subscriber that listens for events and
/// filters using target/level directives from `RUST_LOG`.
///
/// The configuration of the subscriber initialized by this function
/// depends on what [feature flags](crate#feature-flags) are enabled.
///
/// If the `log-tracer` feature is enabled, this will also install
/// the LogTracer to convert `Log` records into `tracing` `Event`s.
///
/// The variable is parsed as a comma-separated list of target prefixes and
/// maximum levels, such as `my_app=debug,info`. Without it, the default level
/// is `INFO`. Invalid directives are reported on stderr.
///
/// # Panics
/// Panics if the initialization was unsuccessful, likely because a
/// global subscriber was already installed by another call to `try_init`.
///
pub fn init() {
    try_init().expect("Unable to install global subscriber")
}
