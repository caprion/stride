//! The athlete's shape over a window: seven facets, as values with a
//! direction of travel.
//!
//! The reference implementation drew these as a **radar**: a polygon over
//! six axes, each axis a percentage of a fixed anchor — "10 hours a week
//! is a very active amateur", "5 days a week is excellent consistency",
//! "90 minutes is endurance-leaning". A polygon is a claim about a shape,
//! and the anchors are the reference's coaching assumptions: they are not
//! the athlete's own history, and a shape drawn over them says something
//! about the athlete that nothing measured.
//!
//! So the facets here are **values**, and each one says **which way it
//! moved** against the window before it. The reader draws the shape.
//!
//! ## The seven facets
//!
//! | facet | the value |
//! |---|---|
//! | volume | the distance of the window |
//! | consistency | the days of the window that carry a run |
//! | intensity | the median heart rate of the runs |
//! | duration | the median moving time of a run |
//! | density | the moving time of an average day with a run |
//! | sport variety | the sports the window holds |
//! | week days | the days of the week the window carries a run on |
//!
//! ## A facet is a total **and** a comparable number
//!
//! A facet that grows with the window cannot be compared with a facet of
//! another length: a ninety-day window holds about three times the
//! distance of a thirty-day one, and drawn as totals the longer window is
//! always outside the shorter one. The chart says then only that the
//! window is longer.
//!
//! So every facet carries **two** numbers, and the core decides which is
//! which:
//!
//! * the **total** — what the window holds: the distance of the window,
//!   the days it holds. This is the number a reader asks for when they
//!   ask "how far did I run in three months?";
//! * the **comparable value** — the same thing as a **rate**, so that a
//!   thirty-day window and a ninety-day window can be read against each
//!   other. The volume is a distance **a week**; the consistency is the
//!   **share of the window's days** that carry a run.
//!
//! The facets that are already averages of the runs inside the window —
//! the intensity (a median), the duration (a median), the density (an
//! average per active day) — do not grow with the window, so their
//! comparable value **is** their own number. The variety is a count of
//! the sports the athlete does: it is bounded by the athlete's repertoire
//! and not by the length of the window, so it is a count in every window.
//!
//! The division lives **here**, and not in the app: every consumer gets a
//! comparable number, and nobody has to remember to divide.
//!
//! ## No value
//!
//! A facet that cannot be computed is **no value**: a window with no run
//! has no median, no variety and no intensity. And the movement needs two
//! windows: a facet with a value but no value in the window before it
//! has **no movement** to report, and says so rather than guessing one.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::band;
use crate::rolling::{Movement, RollingActivity};
use crate::strings;

/// The days a week holds: the volume's rate is a distance a week.
const DAYS_PER_WEEK: f64 = 7.0;

/// The seven facets of the athlete's shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacetKey {
    /// The distance of the window.
    Volume,
    /// The days of the window that carry a run.
    Consistency,
    /// The median heart rate of the runs.
    Intensity,
    /// The median moving time of a run.
    Duration,
    /// The moving time of an average day with a run.
    Density,
    /// The sports the window holds.
    Variety,
    /// The days of the week the window carries a run on.
    WeekDays,
}

impl FacetKey {
    /// The facets, in the order the answer lists them.
    pub fn all() -> [FacetKey; 7] {
        [
            FacetKey::Volume,
            FacetKey::Consistency,
            FacetKey::Intensity,
            FacetKey::Duration,
            FacetKey::Density,
            FacetKey::Variety,
            FacetKey::WeekDays,
        ]
    }

    /// The key the app reads the facet by.
    pub fn key(&self) -> &'static str {
        match self {
            FacetKey::Volume => "volume",
            FacetKey::Consistency => "consistency",
            FacetKey::Intensity => "intensity",
            FacetKey::Duration => "duration",
            FacetKey::Density => "density",
            FacetKey::Variety => "variety",
            FacetKey::WeekDays => "week_days",
        }
    }

    /// The facet's name, in the core's own words: "Sport variety".
    pub fn label(&self) -> &'static str {
        match self {
            FacetKey::Volume => strings::text("profile.volume").unwrap_or("Volume"),
            FacetKey::Consistency => strings::text("profile.consistency").unwrap_or("Consistency"),
            FacetKey::Intensity => strings::text("profile.intensity").unwrap_or("Intensity"),
            FacetKey::Duration => strings::text("profile.duration").unwrap_or("Duration"),
            FacetKey::Density => strings::text("profile.density").unwrap_or("Density"),
            FacetKey::Variety => strings::text("profile.variety").unwrap_or("Sport variety"),
            FacetKey::WeekDays => strings::text("profile.week_days").unwrap_or("Week days"),
        }
    }

    /// The unit the facet's **comparable value** is in.
    ///
    /// This is the unit of the rate for the facets that grow with the
    /// window, and the facet's own unit for the ones that do not.
    pub fn unit(&self) -> FacetUnit {
        match self {
            // The distance of the window, a week at a time.
            FacetKey::Volume => FacetUnit::DistancePerWeek,
            // The share of the window's days that carry a run.
            FacetKey::Consistency => FacetUnit::Share,
            // The rest are averages of the runs inside the window: the
            // same number in a short window and a long one.
            FacetKey::Intensity => FacetUnit::HeartRate,
            FacetKey::Duration => FacetUnit::Duration,
            FacetKey::Density => FacetUnit::Duration,
            FacetKey::Variety => FacetUnit::Count,
            FacetKey::WeekDays => FacetUnit::Count,
        }
    }

    /// The unit the facet's **total** is in: what the window holds.
    pub fn total_unit(&self) -> FacetUnit {
        match self {
            FacetKey::Volume => FacetUnit::Distance,
            FacetKey::Consistency => FacetUnit::Count,
            FacetKey::Intensity => FacetUnit::HeartRate,
            FacetKey::Duration => FacetUnit::Duration,
            FacetKey::Density => FacetUnit::Duration,
            FacetKey::Variety => FacetUnit::Count,
            FacetKey::WeekDays => FacetUnit::Count,
        }
    }
}

/// The unit a facet's value is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacetUnit {
    /// A distance, in metres.
    Distance,
    /// A distance in a week, in metres.
    DistancePerWeek,
    /// A count, with no unit.
    Count,
    /// A share of the window's days, in per cent.
    Share,
    /// A duration, in seconds.
    Duration,
    /// A heart rate, in beats per minute.
    HeartRate,
}

/// One facet of the athlete's shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Facet {
    /// The facet.
    pub key: FacetKey,
    /// The facet's name, in the core's own words.
    pub label: &'static str,
    /// The comparable value: the rate, for the facets that grow with the
    /// window, and the average of the runs for the ones that do not.
    ///
    /// `None` when it cannot be computed.
    pub value: Option<f64>,
    /// The comparable value in the window before it, of the same length.
    pub previous: Option<f64>,
    /// The unit the comparable values are in.
    pub unit: FacetUnit,
    /// The window's own total: what the window holds.
    ///
    /// `None` when it cannot be computed.
    pub total: Option<f64>,
    /// The unit the total is in.
    pub total_unit: FacetUnit,
    /// Which way the comparable value moved.
    ///
    /// `None` when there is no value to compare: a facet that cannot be
    /// computed has no movement, and one that has no value in the window
    /// before it has nothing to move against.
    ///
    /// The movement is the movement of the **comparable value**, and it is
    /// the movement of the total as well: the two windows are the same
    /// length, so the rate and the total move the same way.
    pub movement: Option<Movement>,
}

/// The seven facets of a window, and how they moved.
///
/// The window before the one in question is the comparison period, the
/// same way it is for every other comparison the product makes.
///
/// The **window's own length** is what the rates are measured against: the
/// volume a week, and the share of the window's days that carry a run. The
/// window knows its own length — the last twelve calendar weeks are
/// seventy-eight days on a Monday and eighty-four on a Sunday — so the
/// denominator is never a number somebody remembered.
pub fn facets(
    current: &[RollingActivity],
    previous: &[RollingActivity],
    window: crate::rolling::Window,
) -> Vec<Facet> {
    let days = window.days();
    let now = Values::of(current, days);
    let before = Values::of(previous, days);
    FacetKey::all()
        .iter()
        .map(|key| {
            let value = now.as_ref().and_then(|values| values.get(*key));
            let previous = before.as_ref().and_then(|values| values.get(*key));
            let movement = match (value, previous) {
                (Some(value), Some(previous)) => Some(Movement::of(value, previous)),
                _ => None,
            };
            Facet {
                key: *key,
                label: key.label(),
                value,
                previous,
                unit: key.unit(),
                total: now.as_ref().and_then(|values| values.total(*key)),
                total_unit: key.total_unit(),
                movement,
            }
        })
        .collect()
}

/// The seven values of one window.
struct Values {
    volume_m: f64,
    volume_per_week_m: f64,
    active_days: u32,
    active_share: f64,
    intensity_bpm: Option<f64>,
    duration_s: Option<f64>,
    density_s: Option<f64>,
    variety: u32,
    week_days: u32,
}

impl Values {
    /// The values of a window of runs.
    ///
    /// `None` for a window that holds no run at all: such a window has
    /// no shape, and every one of its values is **no value** — never a
    /// zero. The athlete's profile is about what they did, and a window
    /// in which they did nothing says nothing about their shape.
    ///
    /// `days` is the length of the window: the volume is a distance a
    /// week, and the consistency is the share of the window's days that
    /// carry a run.
    fn of(activities: &[RollingActivity], days: i64) -> Option<Values> {
        if activities.is_empty() {
            return None;
        }
        let days = days.max(1) as f64;
        let mut days_with_a_run: BTreeSet<crate::date::Date> = BTreeSet::new();
        let mut sports: BTreeSet<&str> = BTreeSet::new();
        let mut weekdays: BTreeSet<crate::date::Weekday> = BTreeSet::new();
        let mut volume_m = 0.0_f64;
        let mut moving_time_s = 0.0_f64;
        let mut hearts: Vec<f64> = Vec::new();
        let mut durations: Vec<f64> = Vec::new();
        for activity in activities {
            days_with_a_run.insert(activity.date);
            sports.insert(activity.sport.as_str());
            weekdays.insert(activity.date.weekday());
            volume_m += activity.distance_m;
            moving_time_s += activity.moving_time_s;
            durations.push(activity.moving_time_s);
            if let Some(heart) = activity.average_hr {
                hearts.push(heart);
            }
        }
        let active_days = days_with_a_run.len() as u32;
        Some(Values {
            volume_m,
            // The distance of the window, a week at a time: the same
            // number in a short window and a long one, so that the two
            // can be read against each other.
            volume_per_week_m: volume_m * DAYS_PER_WEEK / days,
            active_days,
            // The share of the window's days that carry a run.
            active_share: f64::from(active_days) / days * 100.0,
            intensity_bpm: band::median(&hearts),
            duration_s: band::median(&durations),
            density_s: if active_days > 0 {
                Some(moving_time_s / f64::from(active_days))
            } else {
                None
            },
            variety: sports.len() as u32,
            week_days: weekdays.len() as u32,
        })
    }

    /// The comparable value of one facet.
    fn get(&self, key: FacetKey) -> Option<f64> {
        match key {
            FacetKey::Volume => Some(self.volume_per_week_m),
            FacetKey::Consistency => Some(self.active_share),
            FacetKey::Intensity => self.intensity_bpm,
            FacetKey::Duration => self.duration_s,
            FacetKey::Density => self.density_s,
            FacetKey::Variety => Some(f64::from(self.variety)),
            FacetKey::WeekDays => Some(f64::from(self.week_days)),
        }
    }

    /// The window's own total of one facet.
    fn total(&self, key: FacetKey) -> Option<f64> {
        match key {
            FacetKey::Volume => Some(self.volume_m),
            FacetKey::Consistency => Some(f64::from(self.active_days)),
            FacetKey::Intensity => self.intensity_bpm,
            FacetKey::Duration => self.duration_s,
            FacetKey::Density => self.density_s,
            FacetKey::Variety => Some(f64::from(self.variety)),
            FacetKey::WeekDays => Some(f64::from(self.week_days)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> Option<crate::date::Date> {
        crate::date::Date::parse(text)
    }

    fn activity(
        day: &str,
        distance_m: f64,
        moving_time_s: f64,
        heart_rate: Option<f64>,
        sport: &str,
    ) -> Option<RollingActivity> {
        Some(RollingActivity {
            date: date(day)?,
            distance_m,
            moving_time_s,
            elevation_gain_m: 0.0,
            start_hour: 7,
            average_hr: heart_rate,
            sport: sport.to_string(),
        })
    }

    fn runs(activities: &[Option<RollingActivity>]) -> Option<Vec<RollingActivity>> {
        let mut out: Vec<RollingActivity> = Vec::new();
        for activity in activities {
            out.push(activity.clone()?);
        }
        Some(out)
    }

    fn facet_of(facets: &[Facet], key: FacetKey) -> Option<&Facet> {
        facets.iter().find(|facet| facet.key == key)
    }

    /// A window of `days` days ending on the 13th of May 2026 — the day
    /// the fixtures of these tests are on.
    fn window(days: i64) -> Option<crate::rolling::Window> {
        let today = date("2026-05-13")?;
        Some(crate::rolling::Window::of_days(days, today))
    }

    #[test]
    fn the_facets_are_the_athlete_s_own_numbers() {
        let activities = match runs(&[
            activity("2026-05-11", 10_000.0, 3_000.0, Some(150.0), "running"),
            activity("2026-05-12", 20_000.0, 6_000.0, Some(160.0), "running"),
            activity("2026-05-13", 5_000.0, 1_500.0, Some(140.0), "cycling"),
        ]) {
            Some(activities) => activities,
            None => return,
        };
        let window = match window(30) {
            Some(window) => window,
            None => return,
        };
        let facets = facets(&activities, &[], window);

        // Volume: the distance of the window, 35 km, and 35 km a week
        // over a thirty-day window.
        let volume = match facet_of(&facets, FacetKey::Volume) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(volume.total, Some(35_000.0));
        assert_eq!(volume.total_unit, FacetUnit::Distance);
        assert_eq!(volume.value, Some(35_000.0 * 7.0 / 30.0));
        assert_eq!(volume.unit, FacetUnit::DistancePerWeek);
        // Consistency: three days carry a run, and three days of thirty
        // is a tenth of the window.
        let consistency = match facet_of(&facets, FacetKey::Consistency) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(consistency.total, Some(3.0));
        assert_eq!(consistency.value, Some(10.0));
        assert_eq!(consistency.unit, FacetUnit::Share);
        // Intensity: the median of 140, 150 and 160 is 150.
        let intensity = match facet_of(&facets, FacetKey::Intensity) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(intensity.value, Some(150.0));
        assert_eq!(intensity.total, Some(150.0));
        // Duration: the median of 1 500, 3 000 and 6 000 is 3 000.
        let duration = match facet_of(&facets, FacetKey::Duration) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(duration.value, Some(3_000.0));
        // Density: 10 500 s over three days.
        let density = match facet_of(&facets, FacetKey::Density) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(density.value, Some(3_500.0));
        // Sport variety: two sports.
        let variety = match facet_of(&facets, FacetKey::Variety) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(variety.value, Some(2.0));
        // Week days: the 11th is a Monday, the 12th a Tuesday, the 13th a
        // Wednesday — three days of the week.
        let week_days = match facet_of(&facets, FacetKey::WeekDays) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(week_days.value, Some(3.0));
    }

    #[test]
    fn the_same_training_reads_the_same_way_in_a_short_window_and_a_long_one() {
        // Six runs in the last thirty days, and six more in the thirty
        // days before them: the same training, twice as long a window.
        let mut activities = match runs(&[
            activity("2026-05-08", 10_000.0, 3_000.0, Some(150.0), "running"),
            activity("2026-05-09", 10_000.0, 3_000.0, Some(150.0), "running"),
            activity("2026-05-10", 10_000.0, 3_000.0, Some(150.0), "running"),
            activity("2026-05-11", 10_000.0, 3_000.0, Some(150.0), "running"),
            activity("2026-05-12", 10_000.0, 3_000.0, Some(150.0), "running"),
            activity("2026-05-13", 10_000.0, 3_000.0, Some(150.0), "running"),
        ]) {
            Some(activities) => activities,
            None => return,
        };
        activities.extend(
            match runs(&[
                activity("2026-04-08", 10_000.0, 3_000.0, Some(150.0), "running"),
                activity("2026-04-09", 10_000.0, 3_000.0, Some(150.0), "running"),
                activity("2026-04-10", 10_000.0, 3_000.0, Some(150.0), "running"),
                activity("2026-04-11", 10_000.0, 3_000.0, Some(150.0), "running"),
                activity("2026-04-12", 10_000.0, 3_000.0, Some(150.0), "running"),
                activity("2026-04-13", 10_000.0, 3_000.0, Some(150.0), "running"),
            ]) {
                Some(activities) => activities,
                None => return,
            },
        );

        let short_window = match window(30) {
            Some(window) => window,
            None => return,
        };
        let long_window = match window(60) {
            Some(window) => window,
            None => return,
        };
        let short = facets(&activities[..6], &[], short_window);
        let long = facets(&activities, &[], long_window);

        let volume_short = match facet_of(&short, FacetKey::Volume) {
            Some(facet) => facet,
            None => return,
        };
        let volume_long = match facet_of(&long, FacetKey::Volume) {
            Some(facet) => facet,
            None => return,
        };
        // The totals are not the same: the window is twice as long.
        assert_eq!(volume_short.total, Some(60_000.0));
        assert_eq!(volume_long.total, Some(120_000.0));
        // And the rate is the same: the same training, a week at a time.
        assert_eq!(volume_short.value, Some(14_000.0));
        assert_eq!(volume_long.value, Some(14_000.0));
        // The consistency is a share of the window's days, and it is the
        // same share: a fifth of the days in both.
        let consistency_short = match facet_of(&short, FacetKey::Consistency) {
            Some(facet) => facet,
            None => return,
        };
        let consistency_long = match facet_of(&long, FacetKey::Consistency) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(consistency_short.value, Some(20.0));
        assert_eq!(consistency_long.value, Some(20.0));
        // The intensity, the duration and the density are the averages of
        // the runs inside the window: the same number in both.
        let intensity_short = match facet_of(&short, FacetKey::Intensity) {
            Some(facet) => facet,
            None => return,
        };
        let intensity_long = match facet_of(&long, FacetKey::Intensity) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(intensity_short.value, intensity_long.value);
        assert_eq!(intensity_short.value, Some(150.0));
    }

    #[test]
    fn a_facet_moves_against_the_window_before_it() {
        let now = match runs(&[activity(
            "2026-05-13",
            20_000.0,
            3_000.0,
            Some(150.0),
            "running",
        )]) {
            Some(runs) => runs,
            None => return,
        };
        let before = match runs(&[activity(
            "2026-05-12",
            10_000.0,
            3_000.0,
            Some(150.0),
            "running",
        )]) {
            Some(runs) => runs,
            None => return,
        };
        let window = match window(7) {
            Some(window) => window,
            None => return,
        };
        let facets = facets(&now, &before, window);
        // The distance grew: up.
        let volume = match facet_of(&facets, FacetKey::Volume) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(volume.movement, Some(Movement::Up));
        assert_eq!(volume.previous, Some(10_000.0));
        // The heart rate did not move.
        let intensity = match facet_of(&facets, FacetKey::Intensity) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(intensity.movement, Some(Movement::Steady));
    }

    #[test]
    fn a_facet_with_nothing_to_compare_has_no_movement() {
        let activities = match runs(&[activity(
            "2026-05-13",
            20_000.0,
            3_000.0,
            Some(150.0),
            "running",
        )]) {
            Some(activities) => activities,
            None => return,
        };
        // The window before it holds nothing, so there is nothing to move
        // against.
        let window = match window(7) {
            Some(window) => window,
            None => return,
        };
        let facets = facets(&activities, &[], window);
        let volume = match facet_of(&facets, FacetKey::Volume) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(volume.total, Some(20_000.0));
        assert_eq!(volume.movement, None);
    }

    #[test]
    fn a_window_with_no_run_has_no_facets_at_all() {
        let window = match window(7) {
            Some(window) => window,
            None => return,
        };
        let facets = facets(&[], &[], window);
        let intensity = match facet_of(&facets, FacetKey::Intensity) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(intensity.value, None);
        let variety = match facet_of(&facets, FacetKey::Variety) {
            Some(facet) => facet,
            None => return,
        };
        // A window with nothing in it has no shape: the count is no
        // value, never a zero.
        assert_eq!(variety.value, None);
        let volume = match facet_of(&facets, FacetKey::Volume) {
            Some(facet) => facet,
            None => return,
        };
        assert_eq!(volume.value, None);
        assert_eq!(volume.total, None);
        // And no facet has a movement.
        assert!(facets.iter().all(|facet| facet.movement.is_none()));
    }
}
