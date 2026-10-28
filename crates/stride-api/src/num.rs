//! Numbers, rounding and labels — the small arithmetic the Python app does in
//! its API layer.
//!
//! The reference is `backend/app/services/analytics.py`.
//!
//! **Rounding.** Python's `round()` is *banker's* rounding: it rounds a tie to
//! the nearest even value. Rust's `f64::round()` rounds a tie away from zero,
//! so a port that uses it differs from the reference on exact ties. The
//! golden files contain no exact ties, but the contract
//! (`docs/core-contract.md` §8) says a port that wants bit-identical doubles
//! must implement round-half-to-even — so [`round_to`] does, by going through
//! the float formatter, which rounds the *exact* binary value half-to-even
//! exactly like CPython does.

use chrono::Datelike;

/// Round to `digits` decimal places, the way Python's `round(x, digits)` does.
///
/// Ties go to the even neighbour, and the rounding is applied to the exact
/// binary value of the double — which is what CPython does, and what makes
/// `round(2.675, 2) == 2.67` on both sides.
pub fn round_to(value: f64, digits: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }
    // `format!` rounds half-to-even on the exact value; parsing the text back
    // gives the same double CPython returns.
    format!("{value:.digits$}").parse::<f64>().unwrap_or(value)
}

/// Python's `int(round(x))` — the nearest integer, ties to even.
pub fn round_int(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    format!("{value:.0}")
        .parse::<f64>()
        .unwrap_or(value)
        .trunc() as i64
}

/// Python's `divmod` for integers: the quotient floors toward negative
/// infinity and the remainder takes the sign of the divisor.
fn python_divmod(value: i64, divisor: i64) -> (i64, i64) {
    let quotient = (value as f64 / divisor as f64).floor() as i64;
    (quotient, value - quotient * divisor)
}

/// Seconds per kilometre, or `None` when the run has no distance or no time.
///
/// The reference: `analytics._pace_s_per_km`.
pub fn pace_s_per_km(distance_m: f64, moving_time_s: f64) -> Option<f64> {
    if distance_m <= 0.0 || moving_time_s <= 0.0 {
        return None;
    }
    Some(moving_time_s / (distance_m / 1000.0))
}

/// `m:ss` — the pace as the runner reads it.
///
/// The reference: `analytics._pace_label`.
pub fn pace_label(seconds: Option<f64>) -> Option<String> {
    let seconds = seconds?;
    if seconds <= 0.0 {
        return None;
    }
    let total = round_int(seconds);
    if total < 0 {
        return None;
    }
    Some(format!("{}:{:02}", total / 60, total % 60))
}

/// `h:mm:ss` or `m:ss` — a duration as the runner reads it.
///
/// The reference: `analytics._duration_label`.
pub fn duration_label(seconds: Option<f64>) -> Option<String> {
    let seconds = seconds?;
    if seconds == 0.0 {
        return None;
    }
    let total = round_int(seconds);
    let (hours, remainder) = python_divmod(total, 3600);
    let (minutes, secs) = python_divmod(remainder, 60);
    if hours != 0 {
        Some(format!("{hours}:{minutes:02}:{secs:02}"))
    } else {
        Some(format!("{minutes}:{secs:02}"))
    }
}

/// `1.23 km` or `840 m` — a distance as the runner reads it.
///
/// The reference: `analytics._display_for`.
pub fn distance_display(value: f64, unit: &str) -> String {
    match unit {
        "s/km" => pace_label(Some(value)).unwrap_or_else(|| "-".to_string()),
        "s" => duration_label(Some(value)).unwrap_or_else(|| "-".to_string()),
        "m" => {
            if value >= 1000.0 {
                format!("{:.2} km", value / 1000.0)
            } else {
                format!("{value:.0} m")
            }
        }
        _ => format!("{value:.0} {unit}"),
    }
}

/// The ISO week key of a `YYYY-MM-DD` date, e.g. `2026-W03`.
///
/// The reference: `ingest/normalize.week_key`. An unparseable date gives an
/// empty string, exactly as the Python does.
pub fn week_key(date: &str) -> String {
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(day) => {
            let iso = day.iso_week();
            format!("{}-W{:02}", iso.year(), iso.week())
        }
        Err(_) => String::new(),
    }
}

/// The month key of a `YYYY-MM-DD` date, e.g. `2026-03`.
pub fn month_key(date: &str) -> String {
    date.chars().take(7).collect()
}

/// The Monday of the ISO week a `YYYY-Www` key names.
pub fn week_start(key: &str) -> Option<String> {
    let (year, week) = key.split_once("-W")?;
    let year: i32 = year.parse().ok()?;
    let week: u32 = week.parse().ok()?;
    // Walk from the Monday of week 1 of the ISO year that owns the key: the
    // ISO year is the one that owns 4 January.
    let jan4 = chrono::NaiveDate::from_ymd_opt(year, 1, 4)?;
    let week1_monday = jan4
        - chrono::Duration::days(chrono::Datelike::weekday(&jan4).num_days_from_monday() as i64);
    let monday = week1_monday + chrono::Duration::days(7 * (week as i64 - 1));
    Some(monday.format("%Y-%m-%d").to_string())
}

/// The median of a slice of numbers — `statistics.median`.
///
/// For an even count Python averages the two middle values.
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[middle])
    } else {
        Some((sorted[middle - 1] + sorted[middle]) / 2.0)
    }
}

/// The arithmetic mean — `statistics.mean`.
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// Python's `round()` of a value that may be absent.
pub fn round_opt(value: Option<f64>, digits: usize) -> Option<f64> {
    value.map(|v| round_to(v, digits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_to_is_bankers_rounding() {
        assert_eq!(round_to(2.5, 0), 2.0);
        assert_eq!(round_to(3.5, 0), 4.0);
        assert_eq!(round_to(-2.5, 0), -2.0);
        assert_eq!(round_to(2.675, 2), 2.67);
        assert_eq!(round_to(1.005, 2), 1.0);
        assert_eq!(round_to(1.115, 2), 1.11);
    }

    #[test]
    fn duration_label_matches_python() {
        assert_eq!(duration_label(Some(0.0)), None);
        assert_eq!(duration_label(Some(65.0)).as_deref(), Some("1:05"));
        assert_eq!(duration_label(Some(3725.0)).as_deref(), Some("1:02:05"));
        assert_eq!(duration_label(Some(59.6)).as_deref(), Some("1:00"));
    }

    #[test]
    fn pace_label_matches_python() {
        assert_eq!(pace_label(Some(0.0)), None);
        assert_eq!(pace_label(Some(299.0)).as_deref(), Some("4:59"));
        assert_eq!(pace_label(Some(60.4)).as_deref(), Some("1:00"));
    }

    #[test]
    fn week_key_matches_python() {
        assert_eq!(week_key("2026-03-02"), "2026-W10");
        assert_eq!(week_key("nonsense"), "");
    }

    #[test]
    fn week_start_matches_python() {
        assert_eq!(week_start("2026-W10").as_deref(), Some("2026-03-02"));
        assert_eq!(week_start("2026-W01").as_deref(), Some("2025-12-29"));
        assert_eq!(week_start("rubbish"), None);
    }
}
