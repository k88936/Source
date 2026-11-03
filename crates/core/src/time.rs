//! Capture-time parsing built on the `time` crate.
//!
//! Cloudreve returns RFC3339 strings; filenames carry a variety of local
//! timestamps. Calendar validation, offsets and the EXIF date syntax come from
//! `time`; `regex` finds the timestamp hidden in a filename.

use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use time::format_description::BorrowedFormatItem;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

/// RFC3339 without an offset, read as UTC.
const LOCAL_DATE_TIME: &[BorrowedFormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");

/// Some servers write the date and time separated by a space.
const SPACED_DATE_TIME: &[BorrowedFormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

/// A date, optionally followed by a time, anywhere in a filename.
///
/// Every separator is optional so `20240102_030405`, `2024-01-02 03:04:05` and
/// `IMG-20240102-WA0001` all match; the year alternation lets the scan skip a
/// digit run that starts with something else.
static FILENAME_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?P<year>19[7-9]\d|2\d{3})",
        r"[-_.]?(?P<month>0[1-9]|1[0-2])",
        r"[-_.]?(?P<day>0[1-9]|[12]\d|3[01])",
        r"(?:[T _-]?(?P<hour>[01]\d|2[0-3])",
        r"[:._-]?(?P<minute>[0-5]\d)",
        r"(?:[:._-]?(?P<second>[0-5]\d))?)?",
    ))
    .expect("filename timestamp pattern is valid")
});

/// Parse an RFC3339 timestamp into Unix milliseconds.
///
/// Zone-less values (which Cloudreve also emits) are read as UTC.
pub fn parse_rfc3339(input: &str) -> Option<i64> {
    OffsetDateTime::parse(input, &Rfc3339)
        .map(unix_millis)
        .ok()
        .or_else(|| zone_less(input))
}

fn zone_less(input: &str) -> Option<i64> {
    [LOCAL_DATE_TIME, SPACED_DATE_TIME]
        .into_iter()
        .find_map(|format| PrimitiveDateTime::parse(input, format).ok())
        .map(|value| unix_millis(value.assume_utc()))
}

/// Infer a capture time from a filename, as Unix milliseconds in UTC.
///
/// EXIF timestamps carry no timezone, so the UI only needs a stable UTC instant
/// to bucket by local day.
pub fn parse_filename_date(name: &str) -> Option<i64> {
    let parts = FILENAME_DATE.captures(name)?;
    let year: i32 = parts.name("year")?.as_str().parse().ok()?;
    let month: u8 = parts.name("month")?.as_str().parse().ok()?;
    let day: u8 = parts.name("day")?.as_str().parse().ok()?;
    let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;
    let clock = match parts.name("hour") {
        Some(hour) => {
            let minute: u8 = parts.name("minute")?.as_str().parse().ok()?;
            let second: u8 = parts
                .name("second")
                .map_or(Some(0), |second| second.as_str().parse().ok())?;
            Time::from_hms(hour.as_str().parse().ok()?, minute, second).ok()?
        }
        None => Time::MIDNIGHT,
    };
    Some(unix_millis(
        PrimitiveDateTime::new(date, clock).assume_utc(),
    ))
}

/// Unix milliseconds for the current instant.
pub fn now_ms() -> i64 {
    system_millis(SystemTime::now())
}

/// Unix seconds for the current instant.
pub fn now_secs() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// Unix milliseconds of a timestamp, truncating the sub-millisecond part.
pub fn unix_millis(value: OffsetDateTime) -> i64 {
    (value.unix_timestamp_nanos() / 1_000_000) as i64
}

/// Unix milliseconds of a [`SystemTime`], `0` when it predates the epoch.
pub fn system_millis(value: SystemTime) -> i64 {
    value
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}
