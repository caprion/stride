//! The rolling view: what a window of history answers, and how it moved.
//!
//! These are the analyses a running log can answer from its own history
//! without a single new input: the runs that are stored, and the zone
//! model that is stored. Nothing here is stored — every number is
//! recomputed on read, exactly as the rest of the derived values are.
//!
//! ## What is in here
//!
//! * [`compare_windows`] — the last *N* days against the same *N* days
//!   ending a day earlier. The comparison is a comparison: two numbers
//!   side by side and the difference between them, never a verdict.
//! * [`polarised_shares`] — how the training was distributed across the
//!   athlete's own zones: low (Z1–Z2), moderate (Z3), high (Z4–Z5), as
//!   shares of the time in the zones.
//! * [`weekday_split`] — what usually happened on which day of the week.
//! * [`daytime_split`] — what usually happened when in the day, in the
//!   athlete's own local time.
//! * [`monthly_totals`] and [`yearly_totals`] — what a month or a year
//!   looks like.
//!
//! ## The window
//!
//! The window is **the last *N* days ending on the day the answer is
//! about**, and the comparison period is **the same *N* days ending a day
//! earlier**. That is the reference implementation's own comparison — its
//! rolling window is a window that moves one day at a time, and its
//! zone split is compared against "the same window as of the previous
//! day" — and it is stated here rather than left to be guessed at.
//!
//! ## No value
//!
//! Every one of these is **"no value" when it cannot be computed**, never
//! zero. A window with no runs has no totals to compare, a window with no
//! time in the zones has no shares, and a month with no runs is not a row
//! in the monthly table at all.

use serde::{Deserialize, Serialize};

use crate::date::{Date, Weekday};
use crate::strings;
use crate::units;

/// The hour the morning starts, in the athlete's own local time.
pub const MORNING_START_HOUR: u32 = 6;

/// The hour the afternoon starts.
pub const AFTERNOON_START_HOUR: u32 = 12;

/// The hour the evening starts.
pub const EVENING_START_HOUR: u32 = 17;

/// The hour the night starts.
pub const NIGHT_START_HOUR: u32 = 23;

/// One run, as the rolling view reads it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RollingActivity {
    /// The athlete's local day the run belongs to.
    pub date: Date,
    /// The run's distance, in metres.
    pub distance_m: f64,
    /// How long the run moved, in seconds.
    pub moving_time_s: f64,
    /// The run's elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// The hour the run started, in the athlete's own local time (0–23).
    pub start_hour: u32,
    /// The run's average heart rate, when it carries one.
    pub average_hr: Option<f64>,
    /// The sport the run was.
    pub sport: String,
}

/// A window of days: how long it is, and the day it ends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollingWindow {
    /// How many days the window covers.
    pub days: i64,
    /// The last day of the window.
    pub to: Date,
}

impl RollingWindow {
    /// A window of `days` days ending on `to`.
    pub fn new(days: i64, to: Date) -> Self {
        Self { days, to }
    }

    /// The first day of the window.
    pub fn from(&self) -> Date {
        self.to.add_days(-(self.days - 1))
    }

    /// The window **before** this one: the same number of days, ending the
    /// day before this window begins.
    ///
    /// This is the period the answer compares itself against, and it must
    /// not overlap the window it is compared with: the two are meant to be
    /// two halves of a longer story, not the same days twice.
    pub fn previous(&self) -> RollingWindow {
        RollingWindow {
            days: self.days,
            to: self.from().add_days(-1),
        }
    }

    /// Whether a day falls inside the window.
    pub fn contains(&self, date: Date) -> bool {
        date >= self.from() && date <= self.to
    }
}

/// What a window of days holds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowTotals {
    /// The distance of the window, in metres.
    pub distance_m: f64,
    /// The moving time of the window, in seconds.
    pub moving_time_s: f64,
    /// How many runs the window holds.
    pub sessions: usize,
}

impl WindowTotals {
    /// A window with nothing in it.
    pub fn empty() -> Self {
        Self {
            distance_m: 0.0,
            moving_time_s: 0.0,
            sessions: 0,
        }
    }
}

/// Two windows, side by side.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowComparison {
    /// The window the answer is about.
    pub current: WindowTotals,
    /// The same window, a day earlier.
    pub previous: WindowTotals,
}

/// The totals of a window of days.
pub fn totals_in(activities: &[RollingActivity], window: RollingWindow) -> WindowTotals {
    let mut totals = WindowTotals::empty();
    for activity in activities {
        if !window.contains(activity.date) {
            continue;
        }
        totals.distance_m += activity.distance_m;
        totals.moving_time_s += activity.moving_time_s;
        totals.sessions += 1;
    }
    totals
}

/// The window and the period it is compared against.
///
/// The comparison is a comparison: the two sets of numbers and the
/// difference between them. Nothing here says whether the difference is
/// good, and nothing here tells the athlete what to do about it.
pub fn compare_windows(activities: &[RollingActivity], window: RollingWindow) -> WindowComparison {
    WindowComparison {
        current: totals_in(activities, window),
        previous: totals_in(activities, window.previous()),
    }
}

/// The three bands the training is distributed across.
///
/// They are the athlete's own zones: Z1–Z2 are the low band, Z3 the
/// moderate one and Z4–Z5 the high one. The names describe what the
/// bands are; they are not a prescription, and there is no target range
/// anywhere near them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolarisedZone {
    /// Zones 1 and 2: the low band.
    Low,
    /// Zone 3: the moderate band.
    Moderate,
    /// Zones 4 and 5: the high band.
    High,
}

impl PolarisedZone {
    /// The bands' order, as the answer lists them.
    pub fn all() -> [PolarisedZone; 3] {
        [
            PolarisedZone::Low,
            PolarisedZone::Moderate,
            PolarisedZone::High,
        ]
    }

    /// The key the app reads the band by.
    pub fn key(&self) -> &'static str {
        match self {
            PolarisedZone::Low => "low",
            PolarisedZone::Moderate => "moderate",
            PolarisedZone::High => "high",
        }
    }

    /// The band's name, in the core's own words: "Z1–Z2 low".
    pub fn label(&self) -> &'static str {
        match self {
            PolarisedZone::Low => strings::text("zones.band.low").unwrap_or("Z1–Z2 low"),
            PolarisedZone::Moderate => {
                strings::text("zones.band.moderate").unwrap_or("Z3 moderate")
            }
            PolarisedZone::High => strings::text("zones.band.high").unwrap_or("Z4–Z5 high"),
        }
    }
}

/// The time spent in each of the five zones, in seconds.
///
/// The time outside the zones — no reading, or a reading at or below the
/// first floor — is not part of the total: the shares are shares of the
/// time the athlete was *in a zone*.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ZoneTimes {
    /// The time in zone 1, in seconds.
    pub zone1_s: f64,
    /// The time in zone 2, in seconds.
    pub zone2_s: f64,
    /// The time in zone 3, in seconds.
    pub zone3_s: f64,
    /// The time in zone 4, in seconds.
    pub zone4_s: f64,
    /// The time in zone 5, in seconds.
    pub zone5_s: f64,
}

impl ZoneTimes {
    /// The time in the five zones.
    pub fn per_zone(&self) -> [f64; 5] {
        [
            self.zone1_s,
            self.zone2_s,
            self.zone3_s,
            self.zone4_s,
            self.zone5_s,
        ]
    }

    /// The total time in the zones, in seconds.
    pub fn total_s(&self) -> f64 {
        self.per_zone().iter().sum()
    }

    /// The time of one band, in seconds.
    pub fn band_s(&self, band: PolarisedZone) -> f64 {
        match band {
            PolarisedZone::Low => self.zone1_s + self.zone2_s,
            PolarisedZone::Moderate => self.zone3_s,
            PolarisedZone::High => self.zone4_s + self.zone5_s,
        }
    }

    /// Whether there is any time in the zones at all.
    pub fn is_empty(&self) -> bool {
        self.total_s() <= 0.0
    }

    /// Each zone's share of the time in the zones, in per cent.
    ///
    /// The five shares are the **distribution** of the training across the
    /// athlete's own zones: how much of the time in the zones was spent
    /// in each of them. The time outside the zones is not part of the
    /// total, exactly as it is not part of the bands' shares.
    ///
    /// `None` when there is no time in the zones at all: that is no
    /// value, never a row of zeros.
    pub fn shares(&self) -> Option<[f64; 5]> {
        let total = self.total_s();
        if total <= 0.0 {
            return None;
        }
        let times = self.per_zone();
        let mut shares = [0.0_f64; 5];
        for index in 0..5 {
            shares[index] = round_share(times[index] / total * 100.0);
        }
        Some(shares)
    }
}

/// Which way a value moved against the window before it.
///
/// The word is the movement and nothing else: a share that grew and a
/// facet that fell are both "up" and "down". **Which way is the better
/// way is not decided here** — the reader reads the movement and decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Movement {
    /// The value is larger than it was.
    Up,
    /// The value is smaller than it was.
    Down,
    /// The value is the same.
    Steady,
}

impl Movement {
    /// The key the app reads the movement by.
    pub fn key(&self) -> &'static str {
        match self {
            Movement::Up => "up",
            Movement::Down => "down",
            Movement::Steady => "steady",
        }
    }

    /// The movement of two numbers.
    ///
    /// The comparison is the numbers' own: a value that is larger is up,
    /// a value that is smaller is down, and two equal values have not
    /// moved.
    pub fn of(current: f64, previous: f64) -> Movement {
        match current.total_cmp(&previous) {
            std::cmp::Ordering::Greater => Movement::Up,
            std::cmp::Ordering::Less => Movement::Down,
            std::cmp::Ordering::Equal => Movement::Steady,
        }
    }
}

/// One band's share of the time in the zones.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneShare {
    /// The band.
    pub band: PolarisedZone,
    /// The band's share of the time in the zones, in per cent. `None` when
    /// the window holds no time in the zones: that is no value, never a
    /// zero.
    pub share: Option<f64>,
    /// The same share in the window before it, when there is one.
    pub previous_share: Option<f64>,
    /// Which way it moved, when both windows have a share to compare.
    pub trend: Option<Movement>,
}

/// How the training was distributed across the athlete's zones.
///
/// The shares are **per cent of the time in the zones**, and they are
/// rounded to two decimals — the precision the reference implementation
/// used, and the precision the comparison is made at, so that two windows
/// that are a hundredth apart do not read as a move.
///
/// A window with no time in the zones has **no shares at all**: the whole
/// result is absent, which is not the same as a share of zero.
pub fn polarised_shares(current: &ZoneTimes, previous: &ZoneTimes) -> Option<Vec<ZoneShare>> {
    if current.is_empty() {
        return None;
    }
    let total = current.total_s();
    let previous_total = previous.total_s();
    let comparable = previous_total > 0.0;

    let shares = PolarisedZone::all()
        .iter()
        .map(|band| {
            let share = round_share(current.band_s(*band) / total * 100.0);
            let previous_share = if comparable {
                Some(round_share(previous.band_s(*band) / previous_total * 100.0))
            } else {
                None
            };
            let trend = previous_share.map(|previous_share| Movement::of(share, previous_share));
            ZoneShare {
                band: *band,
                share: Some(share),
                previous_share,
                trend,
            }
        })
        .collect();
    Some(shares)
}

/// A share, to the precision the comparison is made at.
fn round_share(share: f64) -> f64 {
    units::round_half_even(share, 2)
}

/// What usually happened on one day of the week.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekdayTotals {
    /// The day of the week.
    pub weekday: Weekday,
    /// How many runs fell on it.
    pub sessions: usize,
    /// The distance of those runs, in metres.
    pub distance_m: f64,
    /// The moving time of those runs, in seconds.
    pub moving_time_s: f64,
    /// The day's share of the moving time, in per cent. `None` when there
    /// is no moving time at all to take a share of.
    pub share: Option<f64>,
}

/// What usually happened on each day of the week.
///
/// The share is the day's part of the moving time — the reference
/// implementation's own measure, and the one that says which day the
/// athlete trains on rather than how far they went on it.
pub fn weekday_split(activities: &[RollingActivity]) -> Vec<WeekdayTotals> {
    let mut totals: Vec<WeekdayTotals> = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ]
    .iter()
    .map(|weekday| WeekdayTotals {
        weekday: *weekday,
        sessions: 0,
        distance_m: 0.0,
        moving_time_s: 0.0,
        share: None,
    })
    .collect();

    for activity in activities {
        let index = weekday_index(activity.date.weekday());
        let total = &mut totals[index];
        total.sessions += 1;
        total.distance_m += activity.distance_m;
        total.moving_time_s += activity.moving_time_s;
    }

    let moving_time: f64 = totals.iter().map(|total| total.moving_time_s).sum();
    for total in totals.iter_mut() {
        total.share = if moving_time > 0.0 {
            Some(round_share(total.moving_time_s / moving_time * 100.0))
        } else {
            None
        };
    }
    totals
}

/// The index of a day of the week, Monday first.
fn weekday_index(weekday: Weekday) -> usize {
    match weekday {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    }
}

/// The four times of day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Daytime {
    /// From the morning's cut point to the afternoon's.
    Morning,
    /// From the afternoon's cut point to the evening's.
    Afternoon,
    /// From the evening's cut point to the night's.
    Evening,
    /// From the night's cut point to the morning's.
    Night,
}

impl Daytime {
    /// The buckets, in the order of the day.
    pub fn all() -> [Daytime; 4] {
        [
            Daytime::Morning,
            Daytime::Afternoon,
            Daytime::Evening,
            Daytime::Night,
        ]
    }

    /// The key the app reads the bucket by.
    pub fn key(&self) -> &'static str {
        match self {
            Daytime::Morning => "morning",
            Daytime::Afternoon => "afternoon",
            Daytime::Evening => "evening",
            Daytime::Night => "night",
        }
    }

    /// The bucket's name, in the core's own words.
    pub fn label(&self) -> &'static str {
        match self {
            Daytime::Morning => strings::text("daytime.morning").unwrap_or("Morning"),
            Daytime::Afternoon => strings::text("daytime.afternoon").unwrap_or("Afternoon"),
            Daytime::Evening => strings::text("daytime.evening").unwrap_or("Evening"),
            Daytime::Night => strings::text("daytime.night").unwrap_or("Night"),
        }
    }
}

/// The hours the times of day start at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaytimeCuts {
    /// The hour the morning starts.
    pub morning: u32,
    /// The hour the afternoon starts.
    pub afternoon: u32,
    /// The hour the evening starts.
    pub evening: u32,
    /// The hour the night starts.
    pub night: u32,
}

impl Default for DaytimeCuts {
    fn default() -> Self {
        Self {
            morning: MORNING_START_HOUR,
            afternoon: AFTERNOON_START_HOUR,
            evening: EVENING_START_HOUR,
            night: NIGHT_START_HOUR,
        }
    }
}

impl DaytimeCuts {
    /// The hour the given bucket starts at.
    pub fn start_hour(&self, daytime: Daytime) -> u32 {
        match daytime {
            Daytime::Morning => self.morning,
            Daytime::Afternoon => self.afternoon,
            Daytime::Evening => self.evening,
            Daytime::Night => self.night,
        }
    }

    /// The hour the given bucket ends at: the next bucket's start.
    pub fn end_hour(&self, daytime: Daytime) -> u32 {
        let next = match daytime {
            Daytime::Morning => Daytime::Afternoon,
            Daytime::Afternoon => Daytime::Evening,
            Daytime::Evening => Daytime::Night,
            Daytime::Night => Daytime::Morning,
        };
        self.start_hour(next)
    }

    /// The bucket an hour of the athlete's own local time falls into.
    ///
    /// The cut points are the hours the buckets start at, and the hour
    /// belongs to the bucket whose cut point it is at or after — the
    /// reference implementation's own rule, measured from the morning so
    /// that the night may wrap around midnight.
    pub fn bucket_of(&self, hour: u32) -> Daytime {
        let hour = hour % 24;
        let mut found = Daytime::Morning;
        for daytime in Daytime::all() {
            if hours_since(self.morning, self.start_hour(daytime))
                <= hours_since(self.morning, hour)
            {
                found = daytime;
            }
        }
        found
    }
}

/// The hours between the morning's cut point and an hour.
fn hours_since(morning: u32, hour: u32) -> u32 {
    (hour + 24 - morning % 24) % 24
}

/// What usually happened in one time of day.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DaytimeTotals {
    /// The bucket.
    pub daytime: Daytime,
    /// The hour the bucket starts at.
    pub from_hour: u32,
    /// The hour the bucket ends at.
    pub to_hour: u32,
    /// How many runs fell into it.
    pub sessions: usize,
    /// The distance of those runs, in metres.
    pub distance_m: f64,
    /// The moving time of those runs, in seconds.
    pub moving_time_s: f64,
}

/// What usually happened in each time of day.
///
/// The runs are placed by the hour they **started**, in the athlete's own
/// local time: a run at 07:10 is a morning run wherever the athlete is.
pub fn daytime_split(activities: &[RollingActivity], cuts: DaytimeCuts) -> Vec<DaytimeTotals> {
    let mut totals: Vec<DaytimeTotals> = Daytime::all()
        .iter()
        .map(|daytime| DaytimeTotals {
            daytime: *daytime,
            from_hour: cuts.start_hour(*daytime),
            to_hour: cuts.end_hour(*daytime),
            sessions: 0,
            distance_m: 0.0,
            moving_time_s: 0.0,
        })
        .collect();

    for activity in activities {
        let bucket = cuts.bucket_of(activity.start_hour);
        let index = Daytime::all()
            .iter()
            .position(|daytime| *daytime == bucket)
            .unwrap_or(0);
        let total = &mut totals[index];
        total.sessions += 1;
        total.distance_m += activity.distance_m;
        total.moving_time_s += activity.moving_time_s;
    }
    totals
}

/// What a month or a year holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeriodTotals {
    /// The period's key: "2026-04" for a month, "2026" for a year.
    pub key: String,
    /// How many runs fell into it.
    pub sessions: usize,
    /// The distance of those runs, in metres.
    pub distance_m: f64,
    /// The moving time of those runs, in seconds.
    pub moving_time_s: f64,
    /// The elevation gain of those runs, in metres.
    pub elevation_gain_m: f64,
}

/// What each month of the history holds, newest first.
///
/// **A month with no runs is not a row.** The table holds the months the
/// athlete ran in, and a month without a run is absent from it — not a
/// row of zeros.
pub fn monthly_totals(activities: &[RollingActivity]) -> Vec<PeriodTotals> {
    period_totals(activities, |date| {
        format!("{:04}-{:02}", date.year(), date.month())
    })
}

/// What each year of the history holds, newest first.
///
/// A year with a single run in it is a year with one run in it: the row
/// says so, and it does not pretend to be a year of statistics.
pub fn yearly_totals(activities: &[RollingActivity]) -> Vec<PeriodTotals> {
    period_totals(activities, |date| format!("{:04}", date.year()))
}

/// One bucket of the distance breakdown.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DistanceBucket {
    /// The bucket's lower edge, in kilometres. The bucket holds the
    /// distances above it.
    pub from_km: f64,
    /// The bucket's upper edge, in kilometres. The bucket holds the
    /// distances up to and including it.
    pub to_km: f64,
    /// How many runs fell into it.
    pub sessions: usize,
    /// The distance of those runs, in metres.
    pub distance_m: f64,
    /// The elevation gain of those runs, in metres.
    pub elevation_gain_m: f64,
    /// The moving time of those runs, in seconds.
    pub moving_time_s: f64,
}

/// How many buckets the breakdown holds.
///
/// The reference implementation's own number: the bucket size is chosen so
/// that there are about eleven of them and the longest run lands in the
/// last one.
const BREAKDOWN_BUCKETS: f64 = 11.0;

/// The distance breakdown of a window of runs.
///
/// The bucket size is the reference implementation's own: it is taken from
/// the longest run in the window, so a longest run of 42 km gives 5 km
/// buckets and a longest run of 16 km gives 4 km buckets. A run falls in
/// the bucket it reaches: a run of exactly 5 km is in the first bucket,
/// and a run of 42 km is in the last.
///
/// A window with no run has **no breakdown**: there is nothing to put in
/// the buckets, and the answer is empty rather than a row of zeros.
pub fn distance_breakdown(activities: &[RollingActivity]) -> Vec<DistanceBucket> {
    let longest_km = activities
        .iter()
        .map(|activity| activity.distance_m / 1000.0)
        .fold(0.0_f64, f64::max);
    if longest_km <= 0.0 {
        return Vec::new();
    }
    // The scale of the buckets, from the longest run: a short history is
    // broken down in finer steps than a long one.
    let scale = if longest_km <= 5.0 {
        1.0
    } else if longest_km <= 10.0 {
        2.0
    } else if longest_km <= 15.0 {
        3.0
    } else if longest_km <= 20.0 {
        4.0
    } else {
        5.0
    };
    let bucket_km = ((longest_km / BREAKDOWN_BUCKETS) / scale).ceil() * scale;
    if bucket_km <= 0.0 {
        return Vec::new();
    }
    let count = (longest_km / bucket_km).ceil().max(1.0) as usize;
    let mut buckets: Vec<DistanceBucket> = Vec::with_capacity(count);
    for index in 1..=count {
        buckets.push(DistanceBucket {
            from_km: bucket_km * (index as f64 - 1.0),
            to_km: bucket_km * index as f64,
            sessions: 0,
            distance_m: 0.0,
            elevation_gain_m: 0.0,
            moving_time_s: 0.0,
        });
    }
    for activity in activities {
        let distance_km = activity.distance_m / 1000.0;
        if distance_km <= 0.0 {
            continue;
        }
        let index = (distance_km / bucket_km).ceil() as usize;
        let index = index.saturating_sub(1);
        let Some(bucket) = buckets.get_mut(index) else {
            continue;
        };
        bucket.sessions += 1;
        bucket.distance_m += activity.distance_m;
        bucket.elevation_gain_m += activity.elevation_gain_m;
        bucket.moving_time_s += activity.moving_time_s;
    }
    buckets
}

/// The totals of each period the runs fall into, newest first.
fn period_totals(
    activities: &[RollingActivity],
    key_of: impl Fn(Date) -> String,
) -> Vec<PeriodTotals> {
    let mut periods: Vec<PeriodTotals> = Vec::new();
    for activity in activities {
        let key = key_of(activity.date);
        match periods.iter_mut().find(|period| period.key == key) {
            Some(period) => {
                period.sessions += 1;
                period.distance_m += activity.distance_m;
                period.moving_time_s += activity.moving_time_s;
                period.elevation_gain_m += activity.elevation_gain_m;
            }
            None => periods.push(PeriodTotals {
                key,
                sessions: 1,
                distance_m: activity.distance_m,
                moving_time_s: activity.moving_time_s,
                elevation_gain_m: activity.elevation_gain_m,
            }),
        }
    }
    // The newest first, the way the feed is ordered.
    periods.sort_by(|left, right| right.key.cmp(&left.key));
    periods
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> Option<Date> {
        Date::parse(text)
    }

    fn activity(
        day: &str,
        distance_m: f64,
        moving_time_s: f64,
        start_hour: u32,
    ) -> Option<RollingActivity> {
        Some(RollingActivity {
            date: date(day)?,
            distance_m,
            moving_time_s,
            elevation_gain_m: 0.0,
            start_hour,
            average_hr: None,
            sport: "running".to_string(),
        })
    }

    /// A list of runs, written out.
    fn runs(activities: &[Option<RollingActivity>]) -> Option<Vec<RollingActivity>> {
        let mut out: Vec<RollingActivity> = Vec::new();
        for activity in activities {
            out.push(activity.clone()?);
        }
        Some(out)
    }

    #[test]
    fn a_window_holds_the_runs_inside_it() {
        let runs = match runs(&[
            activity("2026-05-10", 10_000.0, 3_000.0, 7),
            activity("2026-05-12", 8_000.0, 2_400.0, 18),
            activity("2026-05-13", 12_000.0, 3_600.0, 7),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let window = match date("2026-05-13") {
            Some(day) => RollingWindow::new(3, day),
            None => return,
        };
        // The last three days ending on the 13th: the 11th, the 12th and
        // the 13th. The run of the 10th is outside it.
        let totals = totals_in(&runs, window);
        assert_eq!(totals.sessions, 2);
        assert_eq!(totals.distance_m, 20_000.0);
        assert_eq!(totals.moving_time_s, 6_000.0);
    }

    #[test]
    fn the_comparison_is_the_window_before_the_window() {
        let runs = match runs(&[
            activity("2026-05-08", 5_000.0, 1_500.0, 7),
            activity("2026-05-09", 6_000.0, 1_800.0, 7),
            activity("2026-05-11", 10_000.0, 3_000.0, 7),
            activity("2026-05-12", 8_000.0, 2_400.0, 18),
            activity("2026-05-13", 12_000.0, 3_600.0, 7),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let window = match date("2026-05-13") {
            Some(day) => RollingWindow::new(3, day),
            None => return,
        };
        let comparison = compare_windows(&runs, window);
        // The window holds the 11th, the 12th and the 13th.
        assert_eq!(comparison.current.sessions, 3);
        assert_eq!(comparison.current.distance_m, 30_000.0);
        // The window **before** it holds the 8th, the 9th and the 10th: the
        // two runs of that period, and nothing of the window it is compared
        // with. The two windows never share a day.
        assert_eq!(comparison.previous.sessions, 2);
        assert_eq!(comparison.previous.distance_m, 11_000.0);
        assert_eq!(comparison.previous.moving_time_s, 3_300.0);
    }

    #[test]
    fn a_window_with_no_runs_has_no_totals_to_speak_of() {
        let window = match date("2026-05-13") {
            Some(day) => RollingWindow::new(7, day),
            None => return,
        };
        let comparison = compare_windows(&[], window);
        assert_eq!(comparison.current.sessions, 0);
        assert_eq!(comparison.previous.sessions, 0);
        assert_eq!(comparison.current.distance_m, 0.0);
    }

    #[test]
    fn the_shares_are_of_the_time_in_the_zones() {
        let current = ZoneTimes {
            zone1_s: 600.0,
            zone2_s: 600.0,
            zone3_s: 300.0,
            zone4_s: 100.0,
            zone5_s: 100.0,
        };
        let shares = match polarised_shares(&current, &current) {
            Some(shares) => shares,
            None => return,
        };
        // 1200 of 1700 is low, 300 of 1700 is moderate, 200 of 1700 high.
        assert_eq!(shares[0].share, Some(70.59));
        assert_eq!(shares[1].share, Some(17.65));
        assert_eq!(shares[2].share, Some(11.76));
        // The same window compared with itself moved nowhere.
        assert_eq!(shares[0].trend, Some(Movement::Steady));
    }

    #[test]
    fn a_window_with_no_time_in_the_zones_has_no_shares() {
        let empty = ZoneTimes::default();
        assert!(polarised_shares(&empty, &empty).is_none());
    }

    #[test]
    fn a_share_that_grew_says_so() {
        let current = ZoneTimes {
            zone1_s: 800.0,
            zone2_s: 0.0,
            zone3_s: 200.0,
            zone4_s: 0.0,
            zone5_s: 0.0,
        };
        let previous = ZoneTimes {
            zone1_s: 500.0,
            zone2_s: 0.0,
            zone3_s: 500.0,
            zone4_s: 0.0,
            zone5_s: 0.0,
        };
        let shares = match polarised_shares(&current, &previous) {
            Some(shares) => shares,
            None => return,
        };
        assert_eq!(shares[0].trend, Some(Movement::Up));
        assert_eq!(shares[1].trend, Some(Movement::Down));
        assert_eq!(shares[0].previous_share, Some(50.0));
        assert_eq!(shares[1].previous_share, Some(50.0));
    }

    #[test]
    fn a_window_before_with_no_time_has_no_trend_to_report() {
        let current = ZoneTimes {
            zone1_s: 600.0,
            zone2_s: 0.0,
            zone3_s: 0.0,
            zone4_s: 0.0,
            zone5_s: 0.0,
        };
        let shares = match polarised_shares(&current, &ZoneTimes::default()) {
            Some(shares) => shares,
            None => return,
        };
        assert_eq!(shares[0].share, Some(100.0));
        assert_eq!(shares[0].previous_share, None);
        assert_eq!(shares[0].trend, None);
    }

    #[test]
    fn the_weekday_split_places_the_runs_on_their_day() {
        // 11 May 2026 is a Monday, 13 May a Wednesday.
        let runs = match runs(&[
            activity("2026-05-11", 10_000.0, 3_000.0, 7),
            activity("2026-05-12", 5_000.0, 1_500.0, 7),
            activity("2026-05-13", 5_000.0, 1_500.0, 7),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let split = weekday_split(&runs);
        assert_eq!(split.len(), 7);
        // Monday: one run, 10 km.
        assert_eq!(split[0].weekday, Weekday::Monday);
        assert_eq!(split[0].sessions, 1);
        assert_eq!(split[0].distance_m, 10_000.0);
        // Tuesday: one run, 5 km.
        assert_eq!(split[1].sessions, 1);
        assert_eq!(split[1].distance_m, 5_000.0);
        // Wednesday: one run, 5 km.
        assert_eq!(split[2].sessions, 1);
        // The rest of the week: nothing at all.
        assert_eq!(split[3].sessions, 0);
        // The shares are of the moving time: 6000 s in total, 3000 on the
        // Monday.
        assert_eq!(split[0].share, Some(50.0));
        assert_eq!(split[3].share, Some(0.0));
    }

    #[test]
    fn the_weekday_split_has_no_shares_without_moving_time() {
        let split = weekday_split(&[]);
        assert!(split.iter().all(|day| day.share.is_none()));
        assert!(split.iter().all(|day| day.sessions == 0));
    }

    #[test]
    fn the_daytime_split_uses_the_cut_points() {
        let runs = match runs(&[
            activity("2026-05-11", 10_000.0, 3_000.0, 6),
            activity("2026-05-11", 1_000.0, 300.0, 11),
            activity("2026-05-12", 5_000.0, 1_500.0, 12),
            activity("2026-05-12", 1_000.0, 300.0, 17),
            activity("2026-05-13", 1_000.0, 300.0, 23),
            activity("2026-05-13", 1_000.0, 300.0, 5),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let split = daytime_split(&runs, DaytimeCuts::default());
        assert_eq!(split.len(), 4);
        // Morning: 06:00 to 11:59.
        assert_eq!(split[0].from_hour, 6);
        assert_eq!(split[0].to_hour, 12);
        assert_eq!(split[0].sessions, 2);
        assert_eq!(split[0].distance_m, 11_000.0);
        // Afternoon: 12:00 to 16:59.
        assert_eq!(split[1].sessions, 1);
        assert_eq!(split[1].distance_m, 5_000.0);
        // Evening: 17:00 to 22:59.
        assert_eq!(split[2].sessions, 1);
        // Night: 23:00 to 05:59 — the two runs in it.
        assert_eq!(split[3].sessions, 2);
        assert_eq!(split[3].from_hour, 23);
        assert_eq!(split[3].to_hour, 6);
    }

    #[test]
    fn the_night_wraps_around_midnight() {
        let cuts = DaytimeCuts::default();
        assert_eq!(cuts.bucket_of(0), Daytime::Night);
        assert_eq!(cuts.bucket_of(5), Daytime::Night);
        assert_eq!(cuts.bucket_of(6), Daytime::Morning);
        assert_eq!(cuts.bucket_of(11), Daytime::Morning);
        assert_eq!(cuts.bucket_of(12), Daytime::Afternoon);
        assert_eq!(cuts.bucket_of(16), Daytime::Afternoon);
        assert_eq!(cuts.bucket_of(17), Daytime::Evening);
        assert_eq!(cuts.bucket_of(22), Daytime::Evening);
        assert_eq!(cuts.bucket_of(23), Daytime::Night);
    }

    #[test]
    fn the_monthly_table_holds_the_months_the_athlete_ran_in() {
        let runs = match runs(&[
            activity("2026-03-02", 10_000.0, 3_000.0, 7),
            activity("2026-04-02", 12_000.0, 3_600.0, 7),
            activity("2026-04-04", 8_000.0, 2_400.0, 7),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let months = monthly_totals(&runs);
        // Two months: April and March. February is not there.
        assert_eq!(months.len(), 2);
        assert_eq!(months[0].key, "2026-04");
        assert_eq!(months[0].sessions, 2);
        assert_eq!(months[0].distance_m, 20_000.0);
        assert_eq!(months[1].key, "2026-03");
        assert_eq!(months[1].sessions, 1);
    }

    #[test]
    fn a_year_with_one_run_is_a_year_with_one_run() {
        let runs = match runs(&[activity("2026-01-05", 10_000.0, 3_000.0, 7)]) {
            Some(runs) => runs,
            None => return,
        };
        let years = yearly_totals(&runs);
        assert_eq!(years.len(), 1);
        assert_eq!(years[0].key, "2026");
        assert_eq!(years[0].sessions, 1);
        assert_eq!(years[0].distance_m, 10_000.0);
    }

    #[test]
    fn the_distance_breakdown_buckets_the_runs() {
        // The longest run is 42 km, so the buckets are 5 km wide: the
        // reference implementation's own rule.
        let runs = match runs(&[
            activity("2026-05-10", 3_000.0, 1_000.0, 7),
            activity("2026-05-11", 7_000.0, 2_000.0, 7),
            activity("2026-05-12", 12_000.0, 3_000.0, 7),
            activity("2026-05-13", 42_000.0, 10_000.0, 7),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let buckets = distance_breakdown(&runs);
        // Nine buckets: 0-5 up to 40-45.
        assert_eq!(buckets.len(), 9);
        assert_eq!(buckets[0].from_km, 0.0);
        assert_eq!(buckets[0].to_km, 5.0);
        assert_eq!(buckets[8].to_km, 45.0);
        // The run of 3 km is in the first bucket, the run of 7 km in the
        // second, the run of 12 km in the third.
        assert_eq!(buckets[0].sessions, 1);
        assert_eq!(buckets[0].distance_m, 3_000.0);
        assert_eq!(buckets[1].sessions, 1);
        assert_eq!(buckets[1].distance_m, 7_000.0);
        assert_eq!(buckets[2].sessions, 1);
        // And the long one in the last bucket.
        assert_eq!(buckets[8].sessions, 1);
        assert_eq!(buckets[8].distance_m, 42_000.0);
        // The buckets in between hold nothing.
        assert_eq!(buckets[3].sessions, 0);
    }

    #[test]
    fn a_window_with_no_run_has_no_breakdown() {
        let buckets = distance_breakdown(&[]);
        assert!(buckets.is_empty());
    }

    #[test]
    fn the_yearly_table_holds_every_year_that_has_a_run() {
        let runs = match runs(&[
            activity("2024-05-05", 10_000.0, 3_000.0, 7),
            activity("2025-05-05", 10_000.0, 3_000.0, 7),
            activity("2026-05-05", 10_000.0, 3_000.0, 7),
        ]) {
            Some(runs) => runs,
            None => return,
        };
        let years = yearly_totals(&runs);
        assert_eq!(years.len(), 3);
        // The newest first.
        assert_eq!(years[0].key, "2026");
        assert_eq!(years[2].key, "2024");
    }
}
