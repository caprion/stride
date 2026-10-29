//! The readiness score: seven weighted drivers and what they say about today.
//!
//! The reference is `backend/app/services/readiness.py`. Two distinctions
//! matter:
//!
//! * **Scale-based drivers are anchored to a fixed scale, not to the
//!   athlete's baseline.** The sleep score is scored out of 100, stress out of
//!   100, the body battery out of 100 and the acute/chronic load ratio against
//!   1.3. Only sleep, HRV and resting heart rate are measured against the
//!   athlete's own trailing baseline, and those three are the ones that carry
//!   a `baseline` and a `delta` in the output.
//! * **No baseline means no driver.** A missing personal baseline does not
//!   become a score against a population default; the driver is unavailable
//!   and the weights are renormalised over the drivers that remain.

use serde::{Deserialize, Serialize};

use crate::model::DailyPhysiology;
use crate::numeric::{is_number, is_positive, mean, median, round_int, round_to, sum};

/// The number of calendar days the baseline window covers.
pub const BASELINE_WINDOW_DAYS: i64 = 30;

/// The athlete's trailing baselines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Baselines {
    /// The median sleep duration, in seconds.
    pub sleep_s: Option<f64>,
    /// The mean heart-rate variability, in milliseconds.
    pub hrv_ms: Option<f64>,
    /// The mean resting heart rate, in beats per minute.
    pub resting_hr: Option<f64>,
}

/// One readiness driver.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessDriver {
    /// The driver's key: `sleep`, `sleep_score`, `hrv`, `resting_hr`,
    /// `stress`, `body_battery`, `load`.
    pub key: String,
    /// The driver's label, as the UI shows it.
    pub label: String,
    /// The driver's measured value.
    pub value: f64,
    /// The driver's unit.
    pub unit: String,
    /// The athlete's baseline for this driver, or `None` for the scale-based
    /// drivers.
    pub baseline: Option<f64>,
    /// The difference from the baseline, or `None` for the scale-based
    /// drivers.
    pub delta: Option<f64>,
    /// The driver's contribution to the score, as a percentage of the score.
    pub contribution: f64,
    /// A human-readable explanation.
    pub detail: String,
}

/// A day's readiness.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Readiness {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The score, out of 100. `None` when there is no usable data.
    pub score: Option<f64>,
    /// The grade: `peak`, `good`, `fair`, `low` or `unknown`.
    pub grade: String,
    /// The drivers, in the reference's order.
    pub drivers: Vec<ReadinessDriver>,
    /// What to do today.
    pub recommendation: String,
    /// Why the score is what it is.
    pub rationale: String,
}

/// The athlete's trailing baselines for one day.
///
/// The window is the thirty calendar days before the day itself: today's own
/// measurements are excluded, and so is anything older than the window.
pub fn baselines(rows: &[DailyPhysiology], day: &str) -> Baselines {
    let start = crate::date::Date::parse(day)
        .map(|date| date.add_days(-BASELINE_WINDOW_DAYS).to_string())
        .unwrap_or_else(|| day.to_string());
    let day = day.get(..10).unwrap_or(day);

    let history: Vec<&DailyPhysiology> = rows
        .iter()
        .filter(|row| {
            let date = row.date.get(..10).unwrap_or(row.date.as_str());
            date >= start.as_str() && date < day
        })
        .collect();

    let collect = |pick: fn(&DailyPhysiology) -> Option<f64>| -> Vec<f64> {
        history
            .iter()
            .filter_map(|row| pick(row))
            .filter(|value| is_positive(*value))
            .collect()
    };

    let sleep = collect(|row| row.sleep_s);
    let hrv = collect(|row| row.hrv_ms);
    let resting_hr = collect(|row| row.resting_hr);

    Baselines {
        sleep_s: median(&sleep),
        hrv_ms: mean(&hrv),
        resting_hr: mean(&resting_hr),
    }
}

/// The error the domain returns when a range does not make sense.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    /// The range ends before it starts.
    InvalidRange {
        /// The first day of the range.
        start: String,
        /// The last day of the range.
        end: String,
    },
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::InvalidRange { start, end } => {
                write!(formatter, "start must not be after end: {start} > {end}")
            }
        }
    }
}

impl std::error::Error for CoreError {}

/// The readiness score for every day of a range.
///
/// The missing days are included explicitly, so a gap never looks like poor
/// recovery.
pub fn readiness_range(
    rows: &[DailyPhysiology],
    start: &str,
    end: &str,
) -> Result<Vec<Readiness>, CoreError> {
    let first = crate::date::Date::parse(start).ok_or(CoreError::InvalidRange {
        start: start.to_string(),
        end: end.to_string(),
    })?;
    let last = crate::date::Date::parse(end).ok_or(CoreError::InvalidRange {
        start: start.to_string(),
        end: end.to_string(),
    })?;
    if first > last {
        return Err(CoreError::InvalidRange {
            start: start.to_string(),
            end: end.to_string(),
        });
    }

    // The reference reads thirty days of history before the range and keeps
    // it in the baseline window, so the history is the rows from the day
    // before the range's first day.
    let history_start = first.add_days(-BASELINE_WINDOW_DAYS);
    let history: Vec<DailyPhysiology> = rows
        .iter()
        .filter(|row| {
            let date = row.date.get(..10).unwrap_or(row.date.as_str());
            let start = history_start.to_string();
            date >= start.as_str() && date <= last.to_string().as_str()
        })
        .cloned()
        .collect();

    let mut items = Vec::new();
    let mut day = first;
    loop {
        let key = day.to_string();
        let row = history.iter().find(|row| row.date == key);
        let baseline = baselines(&history, &key);
        items.push(readiness(&key, row, &baseline));
        if day == last {
            break;
        }
        day = day.add_days(1);
    }
    Ok(items)
}

/// The readiness score for one day.
pub fn readiness(day: &str, row: Option<&DailyPhysiology>, baseline: &Baselines) -> Readiness {
    derive(day, row, baseline)
}

/// Format a duration in hours as `7h45m`.
fn hours(value: f64) -> String {
    let minutes = round_int(value * 60.0);
    format!("{}h{:02}m", minutes.div_euclid(60), minutes.rem_euclid(60))
}

/// The reference's `_round`: a presentation rounding to one decimal.
fn one_decimal(value: Option<f64>) -> Option<f64> {
    value.map(|value| round_to(value, 1))
}

fn derive(day: &str, row: Option<&DailyPhysiology>, baseline: &Baselines) -> Readiness {
    let mut drivers: Vec<(ReadinessDriver, f64)> = Vec::new();

    /// Add one readiness driver.
    ///
    /// `reference` is what the driver's quality is measured against: the
    /// athlete's own trailing baseline for the personal drivers, a fixed
    /// anchor for the scale-based ones. The scale-based drivers pass
    /// `personal = false`, so no "difference from the anchor" is shown.
    #[allow(clippy::too_many_arguments)]
    fn add(
        drivers: &mut Vec<(ReadinessDriver, f64)>,
        key: &str,
        label: &str,
        value: Option<f64>,
        reference: Option<f64>,
        weight: f64,
        quality: Option<f64>,
        detail: String,
        personal: bool,
        unit: &str,
    ) {
        let (value, quality) = match (value, quality) {
            (Some(value), Some(quality)) => (value, quality),
            _ => return,
        };
        if !is_number(value) {
            return;
        }
        let contribution = weight * quality.clamp(0.0, 1.0);
        drivers.push((
            ReadinessDriver {
                key: key.to_string(),
                label: label.to_string(),
                value,
                unit: unit.to_string(),
                baseline: if personal { reference } else { None },
                delta: match (personal, reference) {
                    (true, Some(reference)) => Some(value - reference),
                    _ => None,
                },
                contribution,
                detail,
            },
            weight,
        ));
    }

    if let Some(row) = row {
        let sleep = row
            .sleep_s
            .filter(|value| is_number(*value))
            .map(|v| v / 3600.0);
        let sleep_base = baseline
            .sleep_s
            .filter(|value| *value != 0.0)
            .map(|value| value / 3600.0);
        let sleep_quality = match (sleep, sleep_base) {
            // The reference asks whether the sleep is *present*, not whether
            // it is positive: a night of zero sleep is a driver with a
            // quality of zero, not a missing driver.
            (Some(sleep), Some(base)) if base != 0.0 => Some(sleep / base),
            _ => None,
        };
        let sleep_detail = match (sleep, sleep_base) {
            (Some(sleep), Some(base)) => format!("{} vs {} baseline", hours(sleep), hours(base)),
            _ => "Personal sleep baseline unavailable".to_string(),
        };
        add(
            &mut drivers,
            "sleep",
            "Sleep",
            sleep,
            sleep_base,
            25.0,
            sleep_quality,
            sleep_detail,
            true,
            "h",
        );

        let sleep_score_quality = row.sleep_score.map(|value| value / 100.0);
        add(
            &mut drivers,
            "sleep_score",
            "Sleep score",
            row.sleep_score,
            Some(100.0),
            10.0,
            sleep_score_quality,
            "Garmin sleep score out of 100".to_string(),
            false,
            "",
        );

        let hrv_base = baseline.hrv_ms;
        let hrv_quality = match (row.hrv_ms, hrv_base) {
            (Some(hrv), Some(base)) if base != 0.0 => Some(hrv / base),
            _ => None,
        };
        let hrv_detail = format!(
            "{} ms vs {} ms baseline",
            one_decimal(row.hrv_ms)
                .map(crate::numeric::format_float)
                .unwrap_or_default(),
            one_decimal(hrv_base)
                .map(crate::numeric::format_float)
                .unwrap_or_default()
        );
        add(
            &mut drivers,
            "hrv",
            "HRV",
            row.hrv_ms,
            hrv_base,
            25.0,
            hrv_quality,
            hrv_detail,
            true,
            "ms",
        );

        let rhr_base = baseline.resting_hr;
        let rhr_quality = match (row.resting_hr, rhr_base) {
            (Some(rhr), Some(base)) if base != 0.0 => Some(1.0 - (rhr - base) / (0.2 * base)),
            _ => None,
        };
        let rhr_detail = format!(
            "{} bpm vs {} bpm baseline",
            one_decimal(row.resting_hr)
                .map(crate::numeric::format_float)
                .unwrap_or_default(),
            one_decimal(rhr_base)
                .map(crate::numeric::format_float)
                .unwrap_or_default()
        );
        add(
            &mut drivers,
            "resting_hr",
            "Resting HR",
            row.resting_hr,
            rhr_base,
            15.0,
            rhr_quality,
            rhr_detail,
            true,
            "bpm",
        );

        let stress_quality = row.stress_avg.map(|value| 1.0 - value / 100.0);
        add(
            &mut drivers,
            "stress",
            "Stress",
            row.stress_avg,
            Some(0.0),
            10.0,
            stress_quality,
            "Average daily stress; lower is better".to_string(),
            false,
            "",
        );

        let battery_quality = row.body_battery_avg.map(|value| value / 100.0);
        add(
            &mut drivers,
            "body_battery",
            "Body battery",
            row.body_battery_avg,
            Some(100.0),
            10.0,
            battery_quality,
            "Average daily body battery out of 100".to_string(),
            false,
            "",
        );

        let ratio = match (row.acute_load, row.chronic_load) {
            (Some(acute), Some(chronic)) if is_number(acute) && is_positive(chronic) => {
                Some(acute / chronic)
            }
            _ => None,
        };
        let load_quality = ratio.map(|ratio| 1.0 - (ratio - 1.3).max(0.0) / 1.3);
        add(
            &mut drivers,
            "load",
            "Load",
            ratio,
            Some(1.3),
            5.0,
            load_quality,
            "Acute/chronic load; ratios above 1.3 reduce readiness".to_string(),
            false,
            "",
        );
    }

    let total_weight: f64 = drivers.iter().map(|(_, weight)| weight).sum();
    let score = if total_weight == 0.0 {
        None
    } else {
        Some(
            sum(&drivers
                .iter()
                .map(|(driver, _)| driver.contribution)
                .collect::<Vec<_>>())
                / total_weight
                * 100.0,
        )
    };

    // The two drivers with the largest deduction, in the reference's order.
    let mut order: Vec<usize> = (0..drivers.len()).collect();
    order.sort_by(|left, right| {
        let left_key = drivers[*left].1 - drivers[*left].0.contribution;
        let right_key = drivers[*right].1 - drivers[*right].0.contribution;
        right_key
            .partial_cmp(&left_key)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let weak: Vec<String> = order
        .iter()
        .filter(|index| {
            let (driver, weight) = &drivers[**index];
            weight - driver.contribution > 0.01
        })
        .take(2)
        .map(|index| drivers[*index].0.label.clone())
        .collect();

    let drivers: Vec<ReadinessDriver> = drivers
        .into_iter()
        .map(|(mut driver, _)| {
            driver.contribution = round_to(driver.contribution / total_weight * 100.0, 2);
            driver
        })
        .collect();

    let grade = match score {
        None => "unknown",
        Some(score) if score >= 85.0 => "peak",
        Some(score) if score >= 70.0 => "good",
        Some(score) if score >= 50.0 => "fair",
        Some(_) => "low",
    }
    .to_string();

    let recommendation = match grade.as_str() {
        "peak" => "Full session as planned — you're primed.",
        "good" => "Train as planned.",
        "fair" => "Consider trimming volume by ~20 %; keep the session easy.",
        "low" => "Downgrade today's session — swap quality work for easy running, or rest.",
        _ => "Readiness unavailable; use perceived recovery to guide training.",
    }
    .to_string();

    let rationale = if !weak.is_empty() {
        format!(
            "{} account for the largest readiness deductions.",
            weak.join(" and ")
        )
    } else if !drivers.is_empty() {
        "Available drivers are at their reference levels.".to_string()
    } else {
        "No usable physiology data or personal baselines are available for this day.".to_string()
    };

    Readiness {
        date: day.to_string(),
        score: score.map(|value| round_to(value, 2)),
        grade,
        drivers,
        recommendation,
        rationale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DailyPhysiology;

    fn row(day: &str, sleep_s: Option<f64>) -> DailyPhysiology {
        DailyPhysiology {
            date: day.to_string(),
            sleep_s,
            ..DailyPhysiology::default()
        }
    }

    #[test]
    fn a_zero_sleep_is_a_driver_with_no_quality_not_a_missing_driver() {
        // The reference asks whether the sleep is *present*, not whether it is
        // positive: a night of zero sleep scores zero, it does not vanish.
        let baseline = Baselines {
            sleep_s: Some(25200.0),
            hrv_ms: None,
            resting_hr: None,
        };
        let score = readiness("2026-03-01", Some(&row("2026-03-01", Some(0.0))), &baseline);
        let sleep = score
            .drivers
            .iter()
            .find(|driver| driver.key == "sleep")
            .expect("the sleep driver");
        assert_eq!(sleep.value, 0.0);
        assert_eq!(sleep.contribution, 0.0);
        assert_eq!(sleep.baseline, Some(7.0));
        assert_eq!(sleep.detail, "0h00m vs 7h00m baseline");
    }

    #[test]
    fn a_missing_baseline_drops_the_personal_driver() {
        let baseline = Baselines::default();
        let score = readiness(
            "2026-03-01",
            Some(&row("2026-03-01", Some(27000.0))),
            &baseline,
        );
        assert!(score.drivers.is_empty());
        assert_eq!(score.score, None);
        assert_eq!(score.grade, "unknown");
    }

    #[test]
    fn the_baseline_window_excludes_today_and_older_rows() {
        let rows = vec![
            row("2026-01-01", Some(1000.0)),
            row("2026-02-02", Some(27000.0)),
            row("2026-02-27", Some(25200.0)),
            row("2026-03-01", Some(99999.0)),
        ];
        let baseline = baselines(&rows, "2026-03-01");
        // 2026-01-01 is outside the thirty-day window (which starts on
        // 2026-01-30) and 2026-03-01 is today: neither counts.
        assert_eq!(baseline.sleep_s, Some(26100.0));
    }

    #[test]
    fn the_sleep_baseline_is_a_median_and_the_others_are_means() {
        let rows = vec![
            row("2026-02-27", Some(24000.0)),
            row("2026-02-28", Some(27000.0)),
            row("2026-03-01", Some(25200.0)),
        ];
        let baseline = baselines(&rows, "2026-03-02");
        assert_eq!(baseline.sleep_s, Some(25200.0));
    }
}

#[cfg(test)]
mod range_tests {
    use super::*;

    #[test]
    fn a_range_includes_the_missing_days() {
        let rows = vec![DailyPhysiology {
            date: "2026-03-02".to_string(),
            sleep_s: Some(27000.0),
            ..DailyPhysiology::default()
        }];
        let scores = readiness_range(&rows, "2026-03-01", "2026-03-03").expect("a range");
        assert_eq!(scores.len(), 3);
        assert_eq!(scores[0].date, "2026-03-01");
        assert_eq!(scores[1].date, "2026-03-02");
        assert_eq!(scores[2].date, "2026-03-03");
        // The day with no row scores nothing, and says so.
        assert_eq!(scores[0].score, None);
        assert_eq!(scores[0].grade, "unknown");
    }

    #[test]
    fn a_range_that_ends_before_it_starts_is_an_error() {
        let error = readiness_range(&[], "2026-03-05", "2026-03-01").expect_err("an error");
        assert_eq!(
            error,
            CoreError::InvalidRange {
                start: "2026-03-05".to_string(),
                end: "2026-03-01".to_string()
            }
        );
        assert_eq!(
            error.to_string(),
            "start must not be after end: 2026-03-05 > 2026-03-01"
        );
    }
}
