//! The numeric helpers the reference implementation uses, reproduced exactly.
//!
//! The Python services lean on `statistics` (exact rational arithmetic) and on
//! `round()` (correct decimal rounding, ties to even). The port reproduces the
//! *results*, not the algorithms: a `f64` sum with a compensated accumulator is
//! accurate to a few ULP, which is far inside the contract's `1e-9` tolerance,
//! and [`round_to`] is bit-identical to `round()` for the values the domain
//! produces.

/// The reference's `number(value)`: a finite number that is not negative.
///
/// Provider sentinels and non-finite values are rejected before they reach a
/// JSON payload.
pub fn is_number(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

/// The reference's `number(value, positive=True)`: a finite number above zero.
pub fn is_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// Python's `round(value, digits)`.
///
/// Python rounds the *decimal* value of the double to the requested number of
/// digits, with exact ties going to the even digit. Rust's `{:.*}` formatting
/// does the same thing over the exact binary value, so formatting and parsing
/// back is a faithful `round()`.
pub fn round_to(value: f64, digits: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let text = format!("{:.*}", digits, value);
    text.parse::<f64>().unwrap_or(value)
}

/// Python's `round(value)` with no digit argument: an integer, ties to even.
pub fn round_int(value: f64) -> i64 {
    let rounded = round_to(value, 0);
    if rounded >= 0.0 {
        rounded as i64
    } else {
        // `as i64` truncates towards zero; Python's round() moves away from
        // zero only for halves it has already rounded to an even integer.
        -((-rounded) as i64)
    }
}

/// The sum of a sequence of floats.
///
/// CPython 3.12's `sum()` uses Neumaier compensated summation for floats, and
/// so does this. Plain left-to-right addition loses the last bit often enough
/// to move a two-decimal rounding at an exact tie, which is exactly the kind
/// of "slightly wrong" the golden files exist to catch.
pub fn sum(values: &[f64]) -> f64 {
    let mut total = 0.0f64;
    let mut compensation = 0.0f64;
    for value in values {
        let next = total + value;
        if total.abs() >= value.abs() {
            compensation += (total - next) + value;
        } else {
            compensation += (value - next) + total;
        }
        total = next;
    }
    total + compensation
}

/// The arithmetic mean, or `None` for an empty sample.
///
/// The reference's `statistics.mean` computes in exact rational arithmetic;
/// the compensated sum here lands on the same double to within a few ULP.
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(sum(values) / values.len() as f64)
}

/// The sample standard deviation (n − 1), or `None` for fewer than two values.
pub fn stdev(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let average = mean(values)?;
    let mut sum = 0.0f64;
    for value in values {
        let delta = value - average;
        sum += delta * delta;
    }
    Some((sum / (values.len() - 1) as f64).sqrt())
}

/// The coefficient of variation as a percentage, or `None`.
///
/// The reference requires at least three values and a positive mean.
pub fn cv(values: &[f64]) -> Option<f64> {
    if values.len() < 3 {
        return None;
    }
    let average = mean(values)?;
    if !(average.is_finite() && average > 0.0) {
        // A non-positive (or non-comparable) mean is "no variability".
        return None;
    }
    Some(stdev(values)? / average * 100.0)
}

/// The median, or `None` for an empty sample.
///
/// An even-sized sample averages the two middle values, like
/// `statistics.median`.
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

/// The relative change from `first` to `last`, as a percentage.
///
/// `None` when either end is missing or when the baseline is zero — a
/// percentage change from zero is not a number.
pub fn change(first: Option<f64>, last: Option<f64>) -> Option<f64> {
    match (first, last) {
        (Some(first), Some(last)) if first != 0.0 => Some((last - first) / first * 100.0),
        _ => None,
    }
}

/// Clamp a quality into [0, 1].
pub fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

/// Clamp a score into [0, 100].
pub fn clamp100(value: f64) -> f64 {
    value.clamp(0.0, 100.0)
}

/// Format a float the way Python's `str(float)` does.
///
/// Python always keeps a decimal point (`58.0`, not `58`); Rust's `Display`
/// does not. The readiness detail strings quote these values verbatim, so the
/// difference is visible in the output.
pub fn format_float(value: f64) -> String {
    let text = format!("{value}");
    if text.contains('.') || text.contains('e') || text.contains("inf") || text.contains("NaN") {
        text
    } else {
        format!("{text}.0")
    }
}
