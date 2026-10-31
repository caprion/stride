//! The units, the rounding and the human rendering.
//!
//! `docs/contract/core-contract.md` §1 (units), §3 (rounding) and §5 (human
//! rendering) are the source of truth. Every rounding helper in the crate
//! lives here and nowhere else: a second copy of "round half to even" in
//! another module is a second answer to the same question, and the contract's
//! table is the contract.
//!
//! The human rendering is the one the athlete reads: "6 h 10 m", not "6.166 h";
//! "14 below your usual", not "−13.7%".

/// Round half to even at `decimals` places.
///
/// This is Python's `round(value, digits)`, which is what the reference
/// implementation used: the *decimal* value of the double is rounded, and an
/// exact tie goes to the even digit. Rust's `{:.N}` formatting does the same
/// over the exact binary value, so formatting and parsing back is a faithful
/// `round()`.
///
/// A port that rounds halves away from zero differs from the reference on
/// exact ties, and a tie is exactly where a golden file notices.
pub fn round_half_even(value: f64, decimals: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let text = format!("{value:.decimals$}");
    text.parse::<f64>().unwrap_or(value)
}

/// Contract §3: the times (`running_time_s`, `moving_time_s`, …) to one decimal.
pub fn round_time_s(value: f64) -> f64 {
    round_half_even(value, 1)
}

/// Contract §3: `running_distance_m` to one decimal.
pub fn round_distance_m(value: f64) -> f64 {
    round_half_even(value, 1)
}

/// Contract §3: a lap's `running_distance_m` to two decimals.
pub fn round_lap_distance_m(value: f64) -> f64 {
    round_half_even(value, 2)
}

/// Contract §3: a lap's `avg_cadence` / `max_cadence` to one decimal.
pub fn round_cadence(value: f64) -> f64 {
    round_half_even(value, 1)
}

/// Contract §3: `pace_s_per_km` (and any derived pace) to two decimals.
pub fn round_pace_s_per_km(value: f64) -> f64 {
    round_half_even(value, 2)
}

/// Contract §3: GPS degrees to seven decimals.
pub fn round_gps_degrees(value: f64) -> f64 {
    round_half_even(value, 7)
}

/// Contract §3: heart rate is truncated to an integer — not rounded.
pub fn truncate_heart_rate(value: f64) -> i64 {
    value.trunc() as i64
}

/// Contract §3: the distance sums Progress reports (`km` totals, `delta_km`)
/// to two decimals.
pub fn round_km(value: f64) -> f64 {
    round_half_even(value, 2)
}

/// The human rendering of a duration (contract §5).
///
/// "6 h 10 m", not "6.166 h". Under an hour: "42 m". Minutes are rounded to
/// the nearest minute, and an exact zero minutes is left out — a person says
/// "1 h", not "1 h 00 m".
pub fn format_duration(seconds: f64) -> String {
    if !seconds.is_finite() {
        return String::new();
    }
    let minutes = round_half_even(seconds / 60.0, 0) as i64;
    let sign = if minutes < 0 { "-" } else { "" };
    let (hours, rest) = (minutes.abs() / 60, minutes.abs() % 60);
    match (hours, rest) {
        (0, rest) => format!("{sign}{rest} m"),
        (hours, 0) => format!("{sign}{hours} h"),
        (hours, rest) => format!("{sign}{hours} h {rest} m"),
    }
}

/// The human rendering of a distance, in kilometres.
pub fn format_distance_km(metres: f64, decimals: usize) -> String {
    if !metres.is_finite() {
        return String::new();
    }
    format!("{:.decimals$}", round_half_even(metres / 1000.0, decimals))
}

/// The human rendering of a distance, in kilometres, without the decimals a
/// person would leave out: "6 km", "8.4 km".
pub fn format_distance_km_short(metres: f64) -> String {
    if !metres.is_finite() {
        return String::new();
    }
    let text = format!("{:.1}", round_half_even(metres / 1000.0, 1));
    let trimmed = text.strip_suffix(".0").unwrap_or(&text);
    trimmed.to_string()
}

/// A number in its own scale, with no decimals: "48", "63".
pub fn format_whole(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    let rounded = round_half_even(value, 0);
    if rounded == 0.0 {
        // `-0.0` and `0.0` are the same number; only one of them may be shown.
        return "0".to_string();
    }
    format!("{rounded}")
}

/// The ordinal of a count: 1st, 2nd, 3rd, 4th, 11th, 21st.
pub fn ordinal(count: usize) -> String {
    let suffix = match (count % 100, count % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{count}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_is_half_to_even_like_the_reference() {
        // Python: round(2.5) == 2, round(3.5) == 4, round(0.125, 2) == 0.12.
        assert_eq!(round_half_even(2.5, 0), 2.0);
        assert_eq!(round_half_even(3.5, 0), 4.0);
        assert_eq!(round_half_even(-2.5, 0), -2.0);
        assert_eq!(round_half_even(0.125, 2), 0.12);
        assert_eq!(round_half_even(0.135, 2), 0.14);
        assert_eq!(round_half_even(1.005, 2), 1.0);
    }

    #[test]
    fn the_contract_table_is_reproduced_exactly() {
        // §3, in the order the table gives it.
        assert_eq!(round_time_s(1234.5678), 1234.6);
        assert_eq!(round_distance_m(2490.6666), 2490.7);
        assert_eq!(round_lap_distance_m(2490.6666), 2490.67);
        assert_eq!(round_cadence(90.0), 90.0);
        assert_eq!(round_cadence(89.949), 89.9);
        assert_eq!(round_pace_s_per_km(305.5555), 305.56);
        assert_eq!(round_gps_degrees(-35.123456789), -35.1234568);
        assert_eq!(truncate_heart_rate(144.9), 144);
        assert_eq!(truncate_heart_rate(63.9), 63);
        assert_eq!(round_km(12.3456), 12.35);
    }

    #[test]
    fn a_duration_is_read_as_a_person_reads_it() {
        // §5: "6 h 10 m", not "6.166 h"; "42 m" under an hour.
        assert_eq!(format_duration(6.0 * 3600.0 + 10.0 * 60.0), "6 h 10 m");
        assert_eq!(format_duration(42.0 * 60.0), "42 m");
        assert_eq!(format_duration(60.0 * 60.0), "1 h");
        assert_eq!(format_duration(0.0), "0 m");
        // Minutes are rounded to the nearest minute.
        assert_eq!(format_duration(90.0), "2 m");
        assert_eq!(
            format_duration(6.0 * 3600.0 + 10.0 * 60.0 + 20.0),
            "6 h 10 m"
        );
    }

    #[test]
    fn a_whole_number_carries_no_decimal_point() {
        assert_eq!(format_whole(48.0), "48");
        assert_eq!(format_whole(48.4), "48");
        assert_eq!(format_whole(48.6), "49");
        assert_eq!(format_whole(-0.2), "0");
    }

    #[test]
    fn a_distance_is_read_without_the_decimals_a_person_would_leave_out() {
        assert_eq!(format_distance_km_short(6000.0), "6");
        assert_eq!(format_distance_km_short(8400.0), "8.4");
        assert_eq!(format_distance_km_short(0.0), "0");
    }

    #[test]
    fn ordinals_read_as_ordinals() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(4), "4th");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(21), "21st");
    }
}
