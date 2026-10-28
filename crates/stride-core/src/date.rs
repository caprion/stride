//! Calendar arithmetic without a date library.
//!
//! The reference works with `datetime.date` and ISO week numbers. The domain
//! needs three things: a difference in days between two ISO dates, the ISO
//! week a date falls in, and the Monday that starts that week. The formulas
//! are Howard Hinnant's `days_from_civil` / `civil_from_days`, which are exact
//! for the whole proleptic Gregorian calendar.

use serde::{Deserialize, Serialize};

/// A calendar date, without a time or a timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// Parse an ISO-8601 date, `YYYY-MM-DD`.
    ///
    /// The reference uses `date.fromisoformat(value[:10])`, so anything longer
    /// is truncated to its date part first.
    pub fn parse(value: &str) -> Option<Self> {
        let head: String = value.chars().take(10).collect();
        let mut parts = head.split('-');
        let year = parts.next()?.parse::<i32>().ok()?;
        let month = parts.next()?.parse::<u32>().ok()?;
        let day = parts.next()?.parse::<u32>().ok()?;
        if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// The number of days since 1970-01-01.
    pub fn to_days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// The date that many days after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);
        Self { year, month, day }
    }

    /// The date `days` days after this one.
    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// The number of days from `self` to `other`.
    pub fn days_until(self, other: Self) -> i64 {
        other.to_days() - self.to_days()
    }

    /// The ISO-8601 week: the year and the week number.
    ///
    /// The ISO year of a date is the calendar year of the Thursday of its
    /// week, which is what `date.isocalendar()` returns.
    pub fn iso_week(self) -> (i32, u32) {
        let days = self.to_days();
        // 1970-01-01 was a Thursday, so (days + 3) % 7 counts Monday as zero.
        let weekday = (days + 3).rem_euclid(7);
        let thursday = days - weekday + 3;
        let year = Self::from_days(thursday).year;
        let january_first = Self {
            year,
            month: 1,
            day: 1,
        }
        .to_days();
        let week = (thursday - january_first) / 7 + 1;
        (year, week as u32)
    }

    /// The Monday that starts this date's ISO week.
    pub fn iso_week_start(self) -> Self {
        let days = self.to_days();
        let weekday = (days + 3).rem_euclid(7);
        Self::from_days(days - weekday)
    }

    /// The ISO-8601 week key, `YYYY-Www`.
    pub fn iso_week_key(self) -> String {
        let (year, week) = self.iso_week();
        format!("{year:04}-W{week:02}")
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

/// Days since 1970-01-01 for a proleptic Gregorian date.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year } as i64;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_position = (month as i64 + 9) % 12;
    let day_of_year = (153 * month_position + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The proleptic Gregorian date that many days after 1970-01-01.
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
    let year = if month <= 2 { year + 1 } else { year };
    (year as i32, month, day)
}

#[cfg(test)]
mod tests {
    use super::Date;

    #[test]
    fn parses_and_formats() {
        let date = Date::parse("2026-03-02").expect("a date");
        assert_eq!((date.year, date.month, date.day), (2026, 3, 2));
        assert_eq!(date.to_string(), "2026-03-02");
        assert!(Date::parse("not-a-date").is_none());
    }

    #[test]
    fn truncates_a_timestamp_to_its_date() {
        let date = Date::parse("2026-03-02T05:30:00+00:00").expect("a date");
        assert_eq!(date.to_string(), "2026-03-02");
    }

    #[test]
    fn computes_iso_weeks() {
        // 2026-03-02 is a Monday and starts ISO week 10.
        let date = Date::parse("2026-03-02").expect("a date");
        assert_eq!(date.iso_week_key(), "2026-W10");
        assert_eq!(date.iso_week_start().to_string(), "2026-03-02");
        // 2026-03-09 starts week 11.
        let date = Date::parse("2026-03-09").expect("a date");
        assert_eq!(date.iso_week_key(), "2026-W11");
        assert_eq!(date.iso_week_start().to_string(), "2026-03-09");
    }

    #[test]
    fn counts_days_between_dates() {
        let start = Date::parse("2026-02-10").expect("a date");
        let end = Date::parse("2026-03-12").expect("a date");
        assert_eq!(end.days_until(start), -30);
        assert_eq!(start.add_days(30).to_string(), "2026-03-12");
    }
}
