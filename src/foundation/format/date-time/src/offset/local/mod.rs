// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! The local (system) time zone.

use super::fixed::FixedOffset;
use super::{MappedLocalTime, TimeZone};
use crate::naive::{NaiveDate, NaiveDateTime, NaiveTime};
#[allow(deprecated)]
use crate::Date;
use crate::{DateTime, Utc};

#[cfg(unix)]
#[path = "unix.rs"]
mod inner;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasmbind",
    not(any(target_os = "emscripten", target_os = "wasi", target_os = "linux"))
))]
mod inner {
    use crate::{Datelike, FixedOffset, MappedLocalTime, NaiveDateTime, Timelike};

    pub(super) fn offset_from_utc_datetime(utc: &NaiveDateTime) -> MappedLocalTime<FixedOffset> {
        let offset = js_sys::Date::from(utc.and_utc()).get_timezone_offset();
        MappedLocalTime::Single(FixedOffset::west_opt((offset as i32) * 60).unwrap())
    }

    pub(super) fn offset_from_local_datetime(
        local: &NaiveDateTime,
    ) -> MappedLocalTime<FixedOffset> {
        let mut year = local.year();
        if year < 100 {
            // The API in `js_sys` does not let us create a `Date` with negative years.
            // And values for years from `0` to `99` map to the years `1900` to `1999`.
            // Shift the value by a multiple of 400 years until it is `>= 100`.
            let shift_cycles = (year - 100).div_euclid(400);
            year -= shift_cycles * 400;
        }
        let js_date = js_sys::Date::new_with_year_month_day_hr_min_sec(
            year as u32,
            local.month0() as i32,
            local.day() as i32,
            local.hour() as i32,
            local.minute() as i32,
            local.second() as i32,
            // ignore milliseconds, our representation of leap seconds may be problematic
        );
        let offset = js_date.get_timezone_offset();
        // We always get a result, even if this time does not exist or is ambiguous.
        MappedLocalTime::Single(FixedOffset::west_opt((offset as i32) * 60).unwrap())
    }
}

#[cfg(unix)]
mod tz_info;

/// The local timescale.
///
/// Using the [`TimeZone`](./trait.TimeZone.html) methods
/// on the Local struct is the preferred way to construct `DateTime<Local>`
/// instances.
///
/// # Example
///
/// ```
/// use blog_format_date_time::{DateTime, Local, TimeZone};
///
/// let dt1: DateTime<Local> = Local::now();
/// let dt2: DateTime<Local> = Local.timestamp_opt(0, 0).unwrap();
/// assert!(dt1 >= dt2);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Local;

impl Local {
    /// Returns a `Date` which corresponds to the current date.
    #[deprecated(since = "0.4.23", note = "use `Local::now()` instead")]
    #[allow(deprecated)]
    #[must_use]
    pub fn today() -> Date<Local> {
        Local::now().date()
    }

    /// Returns a `DateTime<Local>` which corresponds to the current date, time and offset from
    /// UTC.
    ///
    /// See also the similar [`Utc::now()`] which returns `DateTime<Utc>`, i.e. without the local
    /// offset.
    ///
    /// # Example
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// # use blog_format_date_time::{DateTime, FixedOffset, Local};
    /// // Current local time
    /// let now = Local::now();
    ///
    /// // Current local date
    /// let today = now.date_naive();
    ///
    /// // Current local time, converted to `DateTime<FixedOffset>`
    /// let now_fixed_offset = Local::now().fixed_offset();
    /// // or
    /// let now_fixed_offset: DateTime<FixedOffset> = Local::now().into();
    ///
    /// // Current time in some timezone (let's use +05:00)
    /// // Note that it is usually more efficient to use `Utc::now` for this use case.
    /// let offset = FixedOffset::east_opt(5 * 60 * 60).unwrap();
    /// let now_with_offset = Local::now().with_timezone(&offset);
    /// ```
    pub fn now() -> DateTime<Local> {
        Utc::now().with_timezone(&Local)
    }
}

impl TimeZone for Local {
    type Offset = FixedOffset;

    fn from_offset(_offset: &FixedOffset) -> Local {
        Local
    }

    #[allow(deprecated)]
    fn offset_from_local_date(&self, local: &NaiveDate) -> MappedLocalTime<FixedOffset> {
        // Get the offset at local midnight.
        self.offset_from_local_datetime(&local.and_time(NaiveTime::MIN))
    }

    fn offset_from_local_datetime(&self, local: &NaiveDateTime) -> MappedLocalTime<FixedOffset> {
        inner::offset_from_local_datetime(local)
    }

    #[allow(deprecated)]
    fn offset_from_utc_date(&self, utc: &NaiveDate) -> FixedOffset {
        // Get the offset at midnight.
        self.offset_from_utc_datetime(&utc.and_time(NaiveTime::MIN))
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> FixedOffset {
        inner::offset_from_utc_datetime(utc).unwrap()
    }
}

// Calculate the time in UTC given a local time and transitions.
// `transitions` must be sorted.
#[path = "../../../../../../../test/foundation/format/date-time/unit/src/offset/local/mod/tests.rs"]
#[cfg(test)]
mod tests;
