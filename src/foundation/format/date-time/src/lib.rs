//! # Date and time
//!
//! This package derives from Chrono and serves this repository's Linux AMD64 GNU/glibc
//! and browser WASM consumers. It uses the workspace Rust toolchain and is maintained
//! for the current product APIs.
//!
//! Chrono aims to provide all functionality needed to do correct operations on dates and times in
//! the [proleptic Gregorian calendar]:
//!
//! * The [`DateTime`] type is timezone-aware by default, with separate timezone-naive types.
//! * Operations that may produce an invalid or ambiguous date and time return `Option` or
//!   [`MappedLocalTime`].
//! * Configurable parsing and formatting with a `strftime` inspired date and time formatting
//!   syntax.
//! * With `clock`, the `Local` timezone uses the host or browser local timezone.
//! * Types and operations are implemented to be reasonably efficient.
//!
//! Timezone data is read from the Linux host or the browser; no timezone database is bundled.
//! Linux local-time conversion parses timezone transitions in Rust. Browser local-time conversion
//! uses JavaScript `Date`, with the ambiguity limitations described by [`MappedLocalTime`].
//!
//! [proleptic Gregorian calendar]: https://en.wikipedia.org/wiki/Proleptic_Gregorian_calendar
//!
//! ### Features
//!
//! The default features are `clock`, `std`, and `wasmbind`:
//!
//! - `alloc`: Enables allocation-dependent formatting; included by `std`.
//! - `std`: Enables standard-library integration.
//! - `now`: Enables reading the system time; included by `clock` and enables `std`.
//! - `clock`: Enables the `Local` timezone and current-time access.
//! - `wasmbind`: Uses the JavaScript Date API on browser WASM.
//! - `serde`: Optional serialization through the repository's data-model package.
//! - `core-error`: Enables error trait implementations when `std` is disabled.
//!
//! The manifest also retains `__internal_bench` for internal benchmarks. These switches do not
//! expand the product's supported target set. Formatting uses built-in English month and weekday
//! names; localized formatting and the upstream rkyv, arbitrary, and oldtime features are absent.
//!
//! ## Overview
//!
//! ### Time delta / Duration
//!
//! Chrono has a [`TimeDelta`] type to represent the magnitude of a time span. This is an "accurate"
//! duration represented as seconds and nanoseconds, and does not represent "nominal" components
//! such as days or months.
//!
//! Unlike [`core::time::Duration`], [`TimeDelta`] represents signed durations.
//!
//! Chrono currently only supports a small number of operations with [`core::time::Duration`].
//! You can convert between both types with the [`TimeDelta::from_std`] and [`TimeDelta::to_std`]
//! methods.
//!
//! ### Date and Time
//!
//! Chrono provides a [`DateTime`] type to represent a date and a time in a timezone.
//!
//! For more abstract moment-in-time tracking such as internal timekeeping that is unconcerned with
//! timezones, consider [`std::time::SystemTime`], which tracks your system clock, or
//! [`std::time::Instant`], which is an opaque but monotonically-increasing representation of a
//! moment in time.
//!
//! [`DateTime`] is timezone-aware and must be constructed from a [`TimeZone`] object, which defines
//! how the local date is converted to and back from the UTC date.
//! There are three well-known [`TimeZone`] implementations:
//!
//! * [`Utc`] specifies the UTC time zone. It is most efficient.
//!
//! * With `clock`, `Local` specifies the system local time zone.
//!
//! * [`FixedOffset`] specifies an arbitrary, fixed time zone such as UTC+09:00 or UTC-10:30.
//!   This often results from the parsed textual date and time. Since it stores the most information
//!   and does not depend on the system environment, you would want to normalize other `TimeZone`s
//!   into this type.
//!
//! [`DateTime`]s with different [`TimeZone`] types are distinct and do not mix, but can be
//! converted to each other using the [`DateTime::with_timezone`] method.
//!
//! The `now` feature enables `Utc::now()` for reading the current instant in UTC.
//! The `clock` feature additionally enables `Local::now()` for local-time access.
//! Without these features, construct instants from explicit date or timestamp values.
//!
//! ```
//! # #[cfg(feature = "now")] {
//! use blog_format_date_time::prelude::*;
//!
//! let utc: DateTime<Utc> = Utc::now(); // e.g. `2014-11-28T12:45:59.324310806Z`
//! # let _ = utc;
//! # }
//! ```
//!
//! ```
//! # #[cfg(feature = "clock")] {
//! use blog_format_date_time::prelude::*;
//!
//! let local: DateTime<Local> = Local::now(); // e.g. `2014-11-28T21:45:59.324310806+09:00`
//! # let _ = local;
//! # }
//! ```
//!
//! Alternatively, you can create your own date and time. This is a bit verbose due to Rust's lack
//! of function and method overloading, but in turn we get a rich combination of initialization
//! methods.
//!
//! ```
//! use blog_format_date_time::offset::MappedLocalTime;
//! use blog_format_date_time::prelude::*;
//!
//! # fn doctest() -> Option<()> {
//!
//! let dt = Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`
//! assert_eq!(
//!     dt,
//!     NaiveDate::from_ymd_opt(2014, 7, 8)?
//!         .and_hms_opt(9, 10, 11)?
//!         .and_utc()
//! );
//!
//! // July 8 is 188th day of the year 2014 (`o` for "ordinal")
//! assert_eq!(dt, NaiveDate::from_yo_opt(2014, 189)?.and_hms_opt(9, 10, 11)?.and_utc());
//! // July 8 is Tuesday in ISO week 28 of the year 2014.
//! assert_eq!(
//!     dt,
//!     NaiveDate::from_isoywd_opt(2014, 28, Weekday::Tue)?.and_hms_opt(9, 10, 11)?.and_utc()
//! );
//!
//! let dt = NaiveDate::from_ymd_opt(2014, 7, 8)?
//!     .and_hms_milli_opt(9, 10, 11, 12)?
//!     .and_utc(); // `2014-07-08T09:10:11.012Z`
//! assert_eq!(
//!     dt,
//!     NaiveDate::from_ymd_opt(2014, 7, 8)?
//!         .and_hms_micro_opt(9, 10, 11, 12_000)?
//!         .and_utc()
//! );
//! assert_eq!(
//!     dt,
//!     NaiveDate::from_ymd_opt(2014, 7, 8)?
//!         .and_hms_nano_opt(9, 10, 11, 12_000_000)?
//!         .and_utc()
//! );
//!
//! // dynamic verification
//! assert_eq!(
//!     Utc.with_ymd_and_hms(2014, 7, 8, 21, 15, 33),
//!     MappedLocalTime::Single(
//!         NaiveDate::from_ymd_opt(2014, 7, 8)?.and_hms_opt(21, 15, 33)?.and_utc()
//!     )
//! );
//! assert_eq!(Utc.with_ymd_and_hms(2014, 7, 8, 80, 15, 33), MappedLocalTime::None);
//! assert_eq!(Utc.with_ymd_and_hms(2014, 7, 38, 21, 15, 33), MappedLocalTime::None);
//!
//! # #[cfg(feature = "clock")] {
//! // other time zone objects can be used to construct a local datetime.
//! // obviously, `local_dt` is normally different from `dt`, but `fixed_dt` should be identical.
//! let local_dt = Local
//!     .from_local_datetime(
//!         &NaiveDate::from_ymd_opt(2014, 7, 8).unwrap().and_hms_milli_opt(9, 10, 11, 12).unwrap(),
//!     )
//!     .unwrap();
//! let fixed_dt = FixedOffset::east_opt(9 * 3600)
//!     .unwrap()
//!     .from_local_datetime(
//!         &NaiveDate::from_ymd_opt(2014, 7, 8)
//!             .unwrap()
//!             .and_hms_milli_opt(18, 10, 11, 12)
//!             .unwrap(),
//!     )
//!     .unwrap();
//! assert_eq!(dt, fixed_dt);
//! # let _ = local_dt;
//! # }
//! # Some(())
//! # }
//! # doctest().unwrap();
//! ```
//!
//! Various properties are available to the date and time, and can be altered individually. Most of
//! them are defined in the traits [`Datelike`] and [`Timelike`] which you should `use` before.
//! Addition and subtraction is also supported.
//! The following illustrates most supported operations to the date and time:
//!
//! ```rust
//! use blog_format_date_time::prelude::*;
//! use blog_format_date_time::TimeDelta;
//!
//! // assume this returned `2014-11-28T21:45:59.324310806+09:00`:
//! let dt = FixedOffset::east_opt(9 * 3600)
//!     .unwrap()
//!     .from_local_datetime(
//!         &NaiveDate::from_ymd_opt(2014, 11, 28)
//!             .unwrap()
//!             .and_hms_nano_opt(21, 45, 59, 324310806)
//!             .unwrap(),
//!     )
//!     .unwrap();
//!
//! // property accessors
//! assert_eq!((dt.year(), dt.month(), dt.day()), (2014, 11, 28));
//! assert_eq!((dt.month0(), dt.day0()), (10, 27)); // for unfortunate souls
//! assert_eq!((dt.hour(), dt.minute(), dt.second()), (21, 45, 59));
//! assert_eq!(dt.weekday(), Weekday::Fri);
//! assert_eq!(dt.weekday().number_from_monday(), 5); // Mon=1, ..., Sun=7
//! assert_eq!(dt.ordinal(), 332); // the day of year
//! assert_eq!(dt.num_days_from_ce(), 735565); // the number of days from and including Jan 1, 1
//!
//! // time zone accessor and manipulation
//! assert_eq!(dt.offset().fix().local_minus_utc(), 9 * 3600);
//! assert_eq!(dt.timezone(), FixedOffset::east_opt(9 * 3600).unwrap());
//! assert_eq!(
//!     dt.with_timezone(&Utc),
//!     NaiveDate::from_ymd_opt(2014, 11, 28)
//!         .unwrap()
//!         .and_hms_nano_opt(12, 45, 59, 324310806)
//!         .unwrap()
//!         .and_utc()
//! );
//!
//! // a sample of property manipulations (validates dynamically)
//! assert_eq!(dt.with_day(29).unwrap().weekday(), Weekday::Sat); // 2014-11-29 is Saturday
//! assert_eq!(dt.with_day(32), None);
//! assert_eq!(dt.with_year(-300).unwrap().num_days_from_ce(), -109606); // November 29, 301 BCE
//!
//! // arithmetic operations
//! let dt1 = Utc.with_ymd_and_hms(2014, 11, 14, 8, 9, 10).unwrap();
//! let dt2 = Utc.with_ymd_and_hms(2014, 11, 14, 10, 9, 8).unwrap();
//! assert_eq!(dt1.signed_duration_since(dt2), TimeDelta::try_seconds(-2 * 3600 + 2).unwrap());
//! assert_eq!(dt2.signed_duration_since(dt1), TimeDelta::try_seconds(2 * 3600 - 2).unwrap());
//! assert_eq!(
//!     Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
//!         + TimeDelta::try_seconds(1_000_000_000).unwrap(),
//!     Utc.with_ymd_and_hms(2001, 9, 9, 1, 46, 40).unwrap()
//! );
//! assert_eq!(
//!     Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
//!         - TimeDelta::try_seconds(1_000_000_000).unwrap(),
//!     Utc.with_ymd_and_hms(1938, 4, 24, 22, 13, 20).unwrap()
//! );
//! ```
//!
//! ### Formatting and Parsing
//!
//! Formatting is done via the [`format`](DateTime::format()) method, which format is equivalent to
//! the familiar `strftime` format.
//!
//! See [`format::strftime`](format::strftime#specifiers) documentation for full syntax and list of
//! specifiers.
//!
//! The default `to_string` method and `{:?}` specifier also give a reasonable representation.
//! Chrono also provides [`to_rfc2822`](DateTime::to_rfc2822) and
//! [`to_rfc3339`](DateTime::to_rfc3339) methods for well-known formats.
//!
//! Month and weekday names use the built-in English tables.
//!
//! ```rust
//! # #[cfg(feature = "alloc")] {
//! use blog_format_date_time::prelude::*;
//! let dt = Utc.with_ymd_and_hms(2014, 11, 28, 12, 0, 9).unwrap();
//! assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2014-11-28 12:00:09");
//! assert_eq!(dt.format("%a %b %e %T %Y").to_string(), "Fri Nov 28 12:00:09 2014");
//! assert_eq!(dt.format("%a %b %e %T %Y").to_string(), dt.format("%c").to_string());
//! assert_eq!(dt.to_string(), "2014-11-28 12:00:09 UTC");
//! assert_eq!(dt.to_rfc2822(), "Fri, 28 Nov 2014 12:00:09 +0000");
//! assert_eq!(dt.to_rfc3339(), "2014-11-28T12:00:09+00:00");
//! assert_eq!(format!("{:?}", dt), "2014-11-28T12:00:09Z");
//!
//! // Note that milli/nanoseconds are only printed if they are non-zero
//! let dt_nano = NaiveDate::from_ymd_opt(2014, 11, 28)
//!     .unwrap()
//!     .and_hms_nano_opt(12, 0, 9, 1)
//!     .unwrap()
//!     .and_utc();
//! assert_eq!(format!("{:?}", dt_nano), "2014-11-28T12:00:09.000000001Z");
//! # }
//! ```
//!
//! Parsing can be done with two methods:
//!
//! 1. The standard [`FromStr`](std::str::FromStr) trait (and [`parse`](str::parse) method on a
//!    string) can be used for parsing `DateTime<FixedOffset>`, `DateTime<Utc>` and
//!    `DateTime<Local>` values. This parses what the `{:?}` ([`std::fmt::Debug`] format specifier
//!    prints, and requires the offset to be present.
//!
//! 2. [`DateTime::parse_from_str`] parses a date and time with offsets and returns
//!    `DateTime<FixedOffset>`. This should be used when the offset is a part of input and the
//!    caller cannot guess that. It *cannot* be used when the offset can be missing.
//!    [`DateTime::parse_from_rfc2822`] and [`DateTime::parse_from_rfc3339`] are similar but for
//!    well-known formats.
//!
//! More detailed control over the parsing process is available via [`format`](mod@format) module.
//!
//! ```rust
//! use blog_format_date_time::prelude::*;
//!
//! let dt = Utc.with_ymd_and_hms(2014, 11, 28, 12, 0, 9).unwrap();
//! let fixed_dt = dt.with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
//!
//! // method 1
//! assert_eq!("2014-11-28T12:00:09Z".parse::<DateTime<Utc>>(), Ok(dt.clone()));
//! assert_eq!("2014-11-28T21:00:09+09:00".parse::<DateTime<Utc>>(), Ok(dt.clone()));
//! assert_eq!("2014-11-28T21:00:09+09:00".parse::<DateTime<FixedOffset>>(), Ok(fixed_dt.clone()));
//!
//! // method 2
//! assert_eq!(
//!     DateTime::parse_from_str("2014-11-28 21:00:09 +09:00", "%Y-%m-%d %H:%M:%S %z"),
//!     Ok(fixed_dt.clone())
//! );
//! assert_eq!(
//!     DateTime::parse_from_rfc2822("Fri, 28 Nov 2014 21:00:09 +0900"),
//!     Ok(fixed_dt.clone())
//! );
//! assert_eq!(DateTime::parse_from_rfc3339("2014-11-28T21:00:09+09:00"), Ok(fixed_dt.clone()));
//!
//! // oops, the year is missing!
//! assert!(DateTime::parse_from_str("Fri Nov 28 12:00:09", "%a %b %e %T %Y").is_err());
//! // oops, the format string does not include the year at all!
//! assert!(DateTime::parse_from_str("Fri Nov 28 12:00:09", "%a %b %e %T").is_err());
//! // oops, the weekday is incorrect!
//! assert!(DateTime::parse_from_str("Sat Nov 28 12:00:09 2014", "%a %b %e %T %Y").is_err());
//! ```
//!
//! Again: See [`format::strftime`](format::strftime#specifiers) documentation for full syntax and
//! list of specifiers.
//!
//! ### Conversion from and to EPOCH timestamps
//!
//! Use [`DateTime::from_timestamp(seconds, nanoseconds)`](DateTime::from_timestamp)
//! to construct a [`DateTime<Utc>`] from a UNIX timestamp
//! (seconds, nanoseconds that passed since January 1st 1970).
//!
//! Use [`DateTime.timestamp`](DateTime::timestamp) to get the timestamp (in seconds)
//! from a [`DateTime`]. Additionally, you can use
//! [`DateTime.timestamp_subsec_nanos`](DateTime::timestamp_subsec_nanos)
//! to get the number of additional number of nanoseconds.
//!
//! ```
//! # #[cfg(feature = "alloc")] {
//! // We need the trait in scope to use Utc::timestamp().
//! use blog_format_date_time::{DateTime, Utc};
//!
//! // Construct a datetime from epoch:
//! let dt: DateTime<Utc> = DateTime::from_timestamp_secs(1_500_000_000).unwrap();
//! assert_eq!(dt.to_rfc2822(), "Fri, 14 Jul 2017 02:40:00 +0000");
//!
//! // Get epoch value from a datetime:
//! let dt = DateTime::parse_from_rfc2822("Fri, 14 Jul 2017 02:40:00 +0000").unwrap();
//! assert_eq!(dt.timestamp(), 1_500_000_000);
//! # }
//! ```
//!
//! ### Naive date and time
//!
//! Chrono provides naive counterparts to `Date`, (non-existent) `Time` and `DateTime` as
//! [`NaiveDate`], [`NaiveTime`] and [`NaiveDateTime`] respectively.
//!
//! They have almost equivalent interfaces as their timezone-aware twins, but are not associated to
//! time zones obviously and can be quite low-level. They are mostly useful for building blocks for
//! higher-level types.
//!
//! Timezone-aware `DateTime` and `Date` types have two methods returning naive versions:
//! [`naive_local`](DateTime::naive_local) returns a view to the naive local time,
//! and [`naive_utc`](DateTime::naive_utc) returns a view to the naive UTC time.
//!
//! ## Limitations
//!
//! * Only the proleptic Gregorian calendar (i.e. extended to support older dates) is supported.
//! * Date types are limited to about +/- 262,000 years from the common epoch.
//! * Time types are limited to nanosecond accuracy.
//! * Leap seconds can be represented, but Chrono does not fully support them.
//!   See [Leap Second Handling](NaiveTime#leap-second-handling).
//!
//! ## Platform and feature scope
//!
//! Build with the toolchain pinned in the workspace (`rust-toolchain.toml`). The supported
//! consumers are AMD64 Linux GNU/glibc and browser WASM. Select the features needed by each
//! consumer; arithmetic, checked constructors and explicit timestamps do not need a live clock.
//!
//! Prefer [`TimeDelta`], [`DateTime`] and checked constructors. Original source material is
//! available in the provenance archives and `docs/dependencies/origins/chrono-0.4.45`.
//!
//! With `clock` on Linux, `Local` reads `TZ` and system timezone data through Rust
//! implementations. If local timezone discovery fails, it falls back to UTC.

#![doc(test(attr(deny(warnings))))]
#![warn(unreachable_pub)]
#![warn(clippy::tests_outside_test_module)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod time_delta;
#[doc(no_inline)]
#[cfg(any(feature = "std", feature = "core-error"))]
pub use time_delta::OutOfRangeError;
pub use time_delta::TimeDelta;

/// Alias of [`TimeDelta`].
pub type Duration = TimeDelta;

use core::fmt;

/// A convenience module appropriate for glob imports (`use blog_format_date_time::prelude::*;`).
pub mod prelude {
    #[allow(deprecated)]
    pub use crate::Date;
    #[cfg(feature = "clock")]
    pub use crate::Local;
    #[cfg(all(feature = "unstable-locales", feature = "alloc"))]
    pub use crate::Locale;
    pub use crate::SubsecRound;
    pub use crate::{DateTime, SecondsFormat};
    pub use crate::{Datelike, Month, Timelike, Weekday};
    pub use crate::{FixedOffset, Utc};
    pub use crate::{NaiveDate, NaiveDateTime, NaiveTime};
    pub use crate::{Offset, TimeZone};
}

mod date;
#[allow(deprecated)]
pub use date::Date;
#[doc(no_inline)]
#[allow(deprecated)]
pub use date::{MAX_DATE, MIN_DATE};

mod datetime;
pub use datetime::DateTime;
#[allow(deprecated)]
#[doc(no_inline)]
pub use datetime::{MAX_DATETIME, MIN_DATETIME};

pub mod format;
/// L10n locales.
#[cfg(feature = "unstable-locales")]
pub use format::Locale;
pub use format::{ParseError, ParseResult, SecondsFormat};

pub mod naive;
#[doc(inline)]
pub use naive::{Days, NaiveDate, NaiveDateTime, NaiveTime};
pub use naive::{IsoWeek, NaiveWeek};

pub mod offset;
#[cfg(feature = "clock")]
#[doc(inline)]
pub use offset::Local;
#[doc(hidden)]
pub use offset::LocalResult;
pub use offset::MappedLocalTime;
#[doc(inline)]
pub use offset::{FixedOffset, Offset, TimeZone, Utc};

pub mod round;
pub use round::{DurationRound, RoundingError, SubsecRound};

mod weekday;
#[doc(no_inline)]
pub use weekday::ParseWeekdayError;
pub use weekday::Weekday;

mod weekday_set;
pub use weekday_set::WeekdaySet;

mod month;
#[doc(no_inline)]
pub use month::ParseMonthError;
pub use month::{Month, Months};

mod traits;
pub use traits::{Datelike, Timelike};

#[cfg(feature = "__internal_bench")]
#[doc(hidden)]
pub use naive::__BenchYearFlags;

/// Serialization/Deserialization with serde
///
/// The [`DateTime`] type has default implementations for (de)serializing to/from the [RFC 3339]
/// format. This module provides alternatives for serializing to timestamps.
///
/// The alternatives are for use with serde's [`with` annotation] combined with the module name.
/// Alternatively the individual `serialize` and `deserialize` functions in each module can be used
/// with serde's [`serialize_with`] and [`deserialize_with`] annotations.
///
/// *Available on crate feature 'serde' only.*
///
/// [RFC 3339]: https://tools.ietf.org/html/rfc3339
/// [`with` annotation]: https://serde.rs/field-attrs.html#with
/// [`serialize_with`]: https://serde.rs/field-attrs.html#serialize_with
/// [`deserialize_with`]: https://serde.rs/field-attrs.html#deserialize_with
#[cfg(feature = "serde")]
pub mod serde {
    use core::fmt;
    use serde::de;

    pub use super::datetime::serde::*;

    /// Create a custom `de::Error` with `SerdeError::InvalidTimestamp`.
    pub(crate) fn invalid_ts<E, T>(value: T) -> E
    where
        E: de::Error,
        T: fmt::Display,
    {
        E::custom(SerdeError::InvalidTimestamp(value))
    }

    enum SerdeError<T: fmt::Display> {
        InvalidTimestamp(T),
    }

    impl<T: fmt::Display> fmt::Display for SerdeError<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                SerdeError::InvalidTimestamp(ts) => {
                    write!(f, "value is not a legal timestamp: {ts}")
                }
            }
        }
    }
}

/// Out of range error type used in various converting APIs
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct OutOfRange {
    _private: (),
}

impl OutOfRange {
    const fn new() -> OutOfRange {
        OutOfRange { _private: () }
    }
}

impl fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "out of range")
    }
}

impl fmt::Debug for OutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "out of range")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutOfRange {}

#[cfg(all(not(feature = "std"), feature = "core-error"))]
impl core::error::Error for OutOfRange {}

/// Workaround because `?` is not (yet) available in const context.
#[macro_export]
#[doc(hidden)]
macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None,
        }
    };
}

/// Workaround because `.expect()` is not (yet) available in const context.
pub(crate) const fn expect<T: Copy>(opt: Option<T>, msg: &str) -> T {
    match opt {
        Some(val) => val,
        None => panic!("{}", msg),
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "clock")]
    use crate::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    #[test]
    #[allow(deprecated)]
    #[cfg(feature = "clock")]
    fn test_type_sizes() {
        use core::mem::size_of;
        assert_eq!(size_of::<NaiveDate>(), 4);
        assert_eq!(size_of::<Option<NaiveDate>>(), 4);
        assert_eq!(size_of::<NaiveTime>(), 8);
        assert_eq!(size_of::<Option<NaiveTime>>(), 12);
        assert_eq!(size_of::<NaiveDateTime>(), 12);
        assert_eq!(size_of::<Option<NaiveDateTime>>(), 12);

        assert_eq!(size_of::<DateTime<Utc>>(), 12);
        assert_eq!(size_of::<DateTime<FixedOffset>>(), 16);
        assert_eq!(size_of::<DateTime<Local>>(), 16);
        assert_eq!(size_of::<Option<DateTime<FixedOffset>>>(), 16);
    }
}
