//! Rounding that matches the Python reference implementation.
//!
//! The golden files were produced by CPython, whose `round()` is *banker's*
//! rounding: it rounds the exact binary value of the double to the nearest
//! multiple of `10^-digits`, breaking exact ties towards the even digit.
//! Rust's `f64::round()` breaks ties away from zero, so it is **not** a
//! drop-in replacement.
//!
//! `format!("{:.n}", x)` uses an exact decimal conversion and rounds the
//! last kept digit to even, which is the same rule CPython applies; parsing
//! the result back gives the same double CPython returns.

/// Round `value` to `digits` decimal places, exactly like Python's `round()`.
///
/// Non-finite values are returned unchanged.
pub(crate) fn round_to(value: f64, digits: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }
    // `format!` never fails and the result of a fixed-precision float format
    // always parses back into a finite `f64`.
    format!("{value:.digits$}").parse::<f64>().unwrap_or(value)
}

/// Truncate towards zero, like Python's `int()` on a float.
pub(crate) fn trunc_to_int(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    value.trunc() as i64
}

/// The mean of a slice, or `None` for an empty slice.
///
/// CPython's `statistics.mean` computes the mean exactly (over rationals) and
/// rounds once at the end; a plain `sum / len` in `f64` agrees with it for
/// every value this importer produces, because the sums here are small
/// integers or short decimals.
pub(crate) fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    Some(sum / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::round_to;

    #[test]
    fn rounds_like_python() {
        // Ties go to the even digit, exactly like CPython's round().
        assert_eq!(round_to(0.5, 0), 0.0);
        assert_eq!(round_to(1.5, 0), 2.0);
        assert_eq!(round_to(2.5, 0), 2.0);
        assert_eq!(round_to(-0.5, 0), -0.0);
        assert_eq!(round_to(1.25, 1), 1.2);
        assert_eq!(round_to(1.35, 1), 1.4);
        assert_eq!(round_to(12.9716145, 7), 12.9716145);
        assert_eq!(round_to(333.3333333, 2), 333.33);
    }
}
