//! The Eddington number: consistency over peaks.
//!
//! The Eddington number is the largest *E* such that the athlete covered at
//! least *E* units of distance on at least *E* days inside a period. It is a
//! count, so its unit is part of the definition: an Eddington number of 15
//! means "15 kilometres on 15 days" (or "15 miles on 15 days"), and the same
//! history has a different number in each unit. That is the point of a
//! running log: the number rewards doing it again, not doing it once.
//!
//! Two companion values travel with it:
//!
//! * the **days still needed for the next number** — how many more days at or
//!   above *E+1* units would move the number from *E* to *E+1*;
//! * the **longest day** in the period — the single-day reference point, in
//!   metres.
//!
//! The input is the athlete's daily distance, already stored per activity:
//! the store holds `local_date` and `distance_m`, and the caller sums a day's
//! activities into one [`DailyDistance`] before handing the list here. A day
//! with no distance is simply absent from the list — absence is not zero, and
//! neither one contributes to any threshold above zero.
//!
//! Everything time-dependent is an argument: the period is a [`Period`] of
//! [`Date`]s, and there is no clock here.

use serde::{Deserialize, Serialize};

use crate::date::Date;

/// The distance unit an Eddington number is counted in.
///
/// The unit is a parameter, not a second function: the arithmetic is one
/// thing, and a second function per unit would be the same disease as a
/// second rounding table — a rule that can drift. The unit changes only the
/// conversion from metres into the count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EddingtonUnit {
    /// Kilometres.
    Kilometres,
    /// Miles.
    Miles,
}

impl EddingtonUnit {
    /// How many metres make one unit of this kind.
    pub fn metres_per_unit(self) -> f64 {
        match self {
            EddingtonUnit::Kilometres => 1000.0,
            EddingtonUnit::Miles => 1609.344,
        }
    }
}

/// One day's distance, as the Eddington engine reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DailyDistance {
    /// The athlete-local day.
    pub date: Date,
    /// The day's total distance, in metres.
    pub distance_m: f64,
}

impl DailyDistance {
    /// One day's distance.
    pub fn new(date: Date, distance_m: f64) -> Self {
        Self { date, distance_m }
    }
}

/// A date range, both ends inclusive.
///
/// The period is a calendar window, the same inclusive range the store's
/// activity reads use: the first and last day both count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Period {
    /// The first day of the period.
    pub from: Date,
    /// The last day of the period.
    pub to: Date,
}

impl Period {
    /// A period, both ends inclusive.
    pub fn new(from: Date, to: Date) -> Self {
        Self { from, to }
    }

    /// Whether a day falls inside the period.
    pub fn contains(self, date: Date) -> bool {
        date >= self.from && date <= self.to
    }
}

/// The Eddington number and its two companions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eddington {
    /// The Eddington number: the largest *E* with at least *E* days at or
    /// above *E* units of the chosen [`EddingtonUnit`].
    pub number: u32,
    /// The days still needed to move from `number` to `number + 1`.
    pub days_needed_for_next: u32,
    /// The longest distance in a day, in metres. `None` when the period has
    /// no distance.
    pub longest_day_m: Option<f64>,
}

/// The Eddington number and its companions for a period.
///
/// The daily distances are converted to the chosen unit once, and every count
/// is made in that unit. The longest day stays in metres: it is a stored
/// distance, and the contract converts distance at the display boundary, not
/// inside the arithmetic.
pub fn eddington(days: &[DailyDistance], period: Period, unit: EddingtonUnit) -> Eddington {
    let distances = distances_in_unit(days, period, unit);
    let number = eddington_number_of(&distances);
    let days_needed_for_next = days_needed_for_next_in_unit(&distances, number);
    let longest_day_m = longest_day_metres(days, period);
    Eddington {
        number,
        days_needed_for_next,
        longest_day_m,
    }
}

/// The days still needed to move from `current` to `current + 1`.
///
/// The reference behaviour, written down so it is not lost: count the days
/// already at or above `current + 1` units and subtract that count from
/// `current + 1`.
///
/// ```text
/// days_needed_for_next(current) = (current + 1) − count(days ≥ current + 1)
/// ```
///
/// The subtraction is saturating, so the answer is `0` when the next number
/// is already reached — which happens whenever `current` is below the true
/// Eddington number. For the true number itself the answer is at least one:
/// if `current + 1` days had already reached `current + 1` units, the
/// Eddington number would be `current + 1`, not `current`.
pub fn days_needed_for_next(
    days: &[DailyDistance],
    period: Period,
    unit: EddingtonUnit,
    current: u32,
) -> u32 {
    days_needed_for_next_in_unit(&distances_in_unit(days, period, unit), current)
}

/// The days that are inside the period and carry a usable distance.
///
/// A usable distance is a finite, non-negative number of metres: the
/// validity rule lives here, once, so the counts and the longest day cannot
/// disagree about which days exist.
fn usable_days<'a>(
    days: &'a [DailyDistance],
    period: Period,
) -> impl Iterator<Item = &'a DailyDistance> + 'a {
    days.iter()
        .filter(move |day| period.contains(day.date))
        .filter(|day| day.distance_m.is_finite() && day.distance_m >= 0.0)
}

/// The daily distances inside the period, in the chosen unit.
fn distances_in_unit(days: &[DailyDistance], period: Period, unit: EddingtonUnit) -> Vec<f64> {
    let metres_per_unit = unit.metres_per_unit();
    usable_days(days, period)
        .map(|day| day.distance_m / metres_per_unit)
        .collect()
}

/// The longest daily distance inside the period, in metres.
fn longest_day_metres(days: &[DailyDistance], period: Period) -> Option<f64> {
    usable_days(days, period)
        .map(|day| day.distance_m)
        .reduce(f64::max)
}

/// The largest *E* with at least *E* values at or above *E*.
///
/// The values are sorted largest first. The *k*-th largest value being at
/// least *k* means the first *k* values are all at least *k*, so the number
/// is the longest prefix that keeps that property.
fn eddington_number_of(distances: &[f64]) -> u32 {
    let mut sorted = distances.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    let mut number = 0;
    for (index, distance) in sorted.iter().enumerate() {
        let threshold = (index + 1) as f64;
        if *distance >= threshold {
            number = index as u32 + 1;
        } else {
            break;
        }
    }
    number
}

/// The days still needed to reach `current + 1`, over unit distances.
fn days_needed_for_next_in_unit(distances: &[f64], current: u32) -> u32 {
    let next = current.saturating_add(1);
    let already = distances
        .iter()
        .filter(|distance| **distance >= next as f64)
        .count() as u32;
    next.saturating_sub(already)
}
