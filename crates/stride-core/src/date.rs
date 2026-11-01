//! A civil date, with no clock and no timezone.
//!
//! The core never asks what time it is: the day a reading belongs to and the
//! day the athlete is looking at are both injected. What the core *does* need
//! is calendar arithmetic — the band is "the last 30 nights", and a pattern's
//! streak is a run of consecutive days — so the date lives here as a plain
//! count of days, with no library and no timezone.
//!
//! The athlete's day is a local day (`local_date` in the schema), so a date is
//! a local calendar date and nothing else. There is no instant behind it.

use serde::{Deserialize, Serialize};

/// A calendar date, `YYYY-MM-DD`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Date {
    /// Days since 1970-01-01.
    days: i64,
}

impl Date {
    /// Build a date from its parts. `None` when the date does not exist.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self {
            days: days_from_civil(year, month, day),
        })
    }

    /// Parse `YYYY-MM-DD`. `None` for anything that is not a real date.
    pub fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year: i32 = text[0..4].parse().ok()?;
        let month: u32 = text[5..7].parse().ok()?;
        let day: u32 = text[8..10].parse().ok()?;
        Self::from_ymd(year, month, day)
    }

    /// The year.
    pub fn year(&self) -> i32 {
        civil_from_days(self.days).0
    }

    /// The month, 1 to 12.
    pub fn month(&self) -> u32 {
        civil_from_days(self.days).1
    }

    /// The day of the month.
    pub fn day(&self) -> u32 {
        civil_from_days(self.days).2
    }

    /// The date as `YYYY-MM-DD`.
    pub fn to_iso(&self) -> String {
        let (year, month, day) = civil_from_days(self.days);
        format!("{year:04}-{month:02}-{day:02}")
    }

    /// The date `days` days later (or earlier, for a negative count).
    pub fn add_days(&self, days: i64) -> Date {
        Date {
            days: self.days.saturating_add(days),
        }
    }

    /// The number of days from `other` to `self`: negative when `self` is
    /// earlier.
    pub fn days_since(&self, other: Date) -> i64 {
        self.days.saturating_sub(other.days)
    }

    /// The number of days since 1970-01-01.
    pub fn epoch_day(&self) -> i64 {
        self.days
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.to_iso())
    }
}

impl std::str::FromStr for Date {
    type Err = DateError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Date::parse(text).ok_or(DateError)
    }
}

/// The error the core returns when a date cannot be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateError;

impl std::fmt::Display for DateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "not a date of the form YYYY-MM-DD")
    }
}

impl std::error::Error for DateError {}

impl TryFrom<String> for Date {
    type Error = DateError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Date::parse(&text).ok_or(DateError)
    }
}

impl From<Date> for String {
    fn from(date: Date) -> Self {
        date.to_iso()
    }
}

/// The number of days in a month.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days from 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year } as i64;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_position = ((month + 9) % 12) as i64;
    let day_of_year = (153 * month_position + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The civil date for a count of days since 1970-01-01.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_position + 2) / 5 + 1) as u32;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    } as u32;
    let year = if month <= 2 { year + 1 } else { year } as i32;
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_round_trips_through_its_text() {
        for text in [
            "1970-01-01",
            "2026-03-02",
            "2026-12-31",
            "2000-02-29",
            "1999-12-31",
        ] {
            let date = Date::parse(text);
            assert!(date.is_some(), "{text} should parse");
            assert_eq!(date.map(|date| date.to_iso()), Some(text.to_string()));
        }
    }

    #[test]
    fn a_day_that_does_not_exist_is_not_a_date() {
        assert!(Date::parse("2026-02-30").is_none());
        assert!(Date::parse("2026-13-01").is_none());
        assert!(Date::parse("2026-00-10").is_none());
        assert!(Date::parse("2025-02-29").is_none());
        assert!(Date::parse("2026-3-2").is_none());
        assert!(Date::parse("").is_none());
        assert!(Date::parse("2026-03-02T00:00:00").is_none());
        // A leap year is a leap year.
        assert!(Date::parse("2024-02-29").is_some());
    }

    #[test]
    fn days_are_counted_from_the_epoch() {
        let epoch = Date::parse("1970-01-01");
        assert_eq!(epoch.map(|date| date.epoch_day()), Some(0));
        let next = Date::parse("1970-01-02");
        assert_eq!(next.map(|date| date.epoch_day()), Some(1));
        let before = Date::parse("1969-12-31");
        assert_eq!(before.map(|date| date.epoch_day()), Some(-1));
    }

    #[test]
    fn a_date_can_be_moved_by_days() {
        let date = Date::parse("2026-03-02");
        assert_eq!(
            date.map(|date| date.add_days(1).to_iso()),
            Some("2026-03-03".to_string())
        );
        assert_eq!(
            date.map(|date| date.add_days(-1).to_iso()),
            Some("2026-03-01".to_string())
        );
        assert_eq!(
            date.map(|date| date.add_days(30).to_iso()),
            Some("2026-04-01".to_string())
        );
        assert_eq!(
            date.map(|date| date.add_days(-31).to_iso()),
            Some("2026-01-30".to_string())
        );
    }

    #[test]
    fn the_distance_between_two_dates_is_signed() {
        let first = Date::parse("2026-03-02");
        let second = Date::parse("2026-03-12");
        assert_eq!(
            second
                .zip(first)
                .map(|(second, first)| second.days_since(first)),
            Some(10)
        );
        assert_eq!(
            first
                .zip(second)
                .map(|(first, second)| first.days_since(second)),
            Some(-10)
        );
    }

    #[test]
    fn a_leap_day_is_in_the_calendar_it_belongs_to() {
        let march = Date::parse("2024-02-28");
        assert_eq!(
            march.map(|date| date.add_days(1).to_iso()),
            Some("2024-02-29".to_string())
        );
        assert_eq!(
            march.map(|date| date.add_days(2).to_iso()),
            Some("2024-03-01".to_string())
        );
        let february = Date::parse("2025-02-28");
        assert_eq!(
            february.map(|date| date.add_days(1).to_iso()),
            Some("2025-03-01".to_string())
        );
    }
}
