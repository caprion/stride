//! The response shapes — one type per route.
//!
//! The shapes are the contract: the app consumes them, so they are written
//! down here and nowhere else. Four rules shape every one of them.
//!
//! * **The core's sentences travel as text.** A direction word, the
//!   arithmetic, the direction of travel, the "Rest" line, a pattern
//!   sentence and the three statements are rendered by `stride-core` and
//!   arrive here already written. There is no template in this crate and no
//!   hole in a string: a sentence with a `{placeholder}` in it is the
//!   core's business, not this crate's.
//! * **The core's values travel as text and as a number.** A domain value
//!   crosses the wire as a [`RenderedValue`]: the core's words *and* the
//!   raw number a chart draws. There is no `format:` hint — a hint tells
//!   the app how to render, which is the hole the contract closes.
//! * **"No value" is `null`.** A missing measurement is `None`, never `0`.
//!   The fields that may be absent are `Option`, and the fields that are
//!   always there are not.
//! * **Every response carries `generated_at`.** The app shows staleness
//!   rather than silently showing old numbers.
//!
//! The names are the app's own (`frontend/src/lib/types.ts`): where the app
//! types a route, this crate answers with exactly that shape — the same
//! keys, the same nullability — so the two cannot drift.

use serde::{Deserialize, Serialize};

/// A domain value, as the core renders it.
///
/// `text` is the core's words ("8.4 km", "42 m", "148 bpm"); `raw` is the
/// number, for a chart. `raw` is `None` when there is no value, and then
/// `text` is empty — never "0", never "0 km".
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RenderedValue {
    /// The core's words.
    pub text: String,
    /// The number, for a chart. `None` when there is no value.
    #[serde(serialize_with = "serialize_raw")]
    pub raw: Option<f64>,
}

/// A number the way the contract writes it.
///
/// The contract's §2: *JSON output carries no `NaN`, no `Infinity`, and no
/// trailing `.0` on an integer-valued float.* A distance of 6 439 metres
/// is `6439`, not `6439.0`.
///
/// It is not only the contract. **The two hosts hand the app the same
/// bytes**, and a number written two ways is two answers: the rig's
/// serializer would write `162.0` where the phone's writes `162`, and the
/// app could not tell which host it was talking to. One number, one way of
/// writing it, on both sides.
fn serialize_raw<S>(raw: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match raw {
        None => serializer.serialize_none(),
        Some(value) if !value.is_finite() => serializer.serialize_none(),
        Some(value) if value.fract() == 0.0 && value.abs() < 9.0e15 => {
            serializer.serialize_i64(*value as i64)
        }
        Some(value) => serializer.serialize_f64(*value),
    }
}

// ---------------------------------------------------------------- activities

/// One run, as the feed shows it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActivitySummary {
    /// The run's id.
    pub id: i64,
    /// The run's title.
    pub name: String,
    /// The normalised sport.
    pub sport_type: String,
    /// When the run started, in UTC.
    pub start_time: String,
    /// The athlete's local date.
    pub local_date: String,
    /// The distance, as the core renders it.
    pub distance: RenderedValue,
    /// The moving time, as the core renders it.
    pub duration: RenderedValue,
    /// The average heart rate, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
    /// The elevation gain, as the core renders it.
    pub elevation_gain: Option<RenderedValue>,
    /// The pace, as the core renders it.
    pub pace: Option<RenderedValue>,
    /// The group the run is in, and that group's totals.
    pub group: ActivityGroupTotals,
    /// The one sentence about what the run was.
    ///
    /// The core's story engine writes it — descriptive, never a verdict —
    /// and it is never empty: a row without a sentence is a spreadsheet.
    pub story: String,
    /// The route, as `[latitude, longitude]` pairs.
    pub geo_points: Vec<[f64; 2]>,
}

/// The totals of a group of runs.
///
/// The feed is grouped by the API — the API is the one that knows the
/// totals — and the app labels the groups.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActivityGroupTotals {
    /// The group's key: the day the group starts.
    pub key: String,
    /// The group's distance, as the core renders it.
    pub distance: RenderedValue,
    /// The group's time, as the core renders it.
    pub duration: RenderedValue,
}

/// The feed, windowed by date.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActivityListResponse {
    /// The runs in the page, newest first.
    pub activities: Vec<ActivitySummary>,
    /// The cursor of the next page. `null` when the feed has ended.
    pub next_cursor: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One split of a run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Split {
    /// The lap's number, from one.
    ///
    /// It is a **lap**, not a kilometre: the number counts the laps the
    /// watch recorded, and a lap is not a distance. The field says which,
    /// because the reader of the answer should not have to know.
    pub lap_index: i64,
    /// The split's distance, as the core renders it.
    pub distance: RenderedValue,
    /// The split's duration, as the core renders it.
    pub duration: RenderedValue,
    /// The average heart rate over the split, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
    /// The lap's pace, when it has a positive distance and duration.
    pub pace: Option<RenderedValue>,
}

/// One point of the elevation profile.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ElevationPoint {
    /// The distance from the start, as the core renders it.
    pub distance: RenderedValue,
    /// The altitude, as the core renders it.
    pub elevation: RenderedValue,
}

/// One point of the run's series.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SamplePoint {
    /// The seconds from the start of the run.
    pub offset_s: f64,
    /// The pace over the sample, as the core renders it.
    pub pace: Option<RenderedValue>,
    /// The heart rate over the sample, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
}

/// The plan's line under a run: what was planned and what was run.
///
/// The line is **the core's statement**: "8.4 km · 1.6 km less than the 10
/// km target". It is absent when the gap from the target is within ten per
/// cent — 10.2 km against a 10 km target is not news, and neither here nor
/// on Today.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanLine {
    /// The line, already written.
    pub text: String,
}

/// One run, in full.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActivityDetail {
    /// The run, as the feed shows it.
    #[serde(flatten)]
    pub summary: ActivitySummary,
    /// The run's splits.
    pub splits: Vec<Split>,
    /// The run's elevation profile.
    pub elevation: Vec<ElevationPoint>,
    /// The run's series.
    pub samples: Vec<SamplePoint>,
    /// The plan's line under the run, when the plan had one.
    pub plan_line: Option<PlanLine>,
    /// The athlete's own note about the run.
    pub note: Option<String>,
    /// The one extra thing the detail knows: the run's own shape, in the
    /// core's words. `null` when there is nothing more to say than the
    /// sentence.
    pub why: Option<String>,
}

// --------------------------------------------------------------- physiology

/// One day of wellness.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PhysiologyDay {
    /// The athlete's local date.
    pub date: String,
    /// The sleep, as the core renders it.
    pub sleep: Option<RenderedValue>,
    /// The deep sleep, as the core renders it.
    pub deep_sleep: Option<RenderedValue>,
    /// The REM sleep, as the core renders it.
    pub rem_sleep: Option<RenderedValue>,
    /// The light sleep, as the core renders it.
    pub light_sleep: Option<RenderedValue>,
    /// The time awake, as the core renders it.
    pub awake: Option<RenderedValue>,
    /// The heart-rate variability, as the core renders it.
    pub hrv: Option<RenderedValue>,
    /// The resting heart rate, as the core renders it.
    pub resting_hr: Option<RenderedValue>,
    /// The average stress, as the core renders it.
    pub stress: Option<RenderedValue>,
    /// Where the row came from.
    pub source: String,
}

/// The days of wellness.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PhysiologyDays {
    /// The days, oldest first.
    pub days: Vec<PhysiologyDay>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One night of sleep.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SleepDay {
    /// The athlete's local date.
    pub date: String,
    /// The sleep, as the core renders it.
    pub sleep: Option<RenderedValue>,
    /// The deep sleep, as the core renders it.
    pub deep_sleep: Option<RenderedValue>,
    /// The REM sleep, as the core renders it.
    pub rem_sleep: Option<RenderedValue>,
    /// The light sleep, as the core renders it.
    pub light_sleep: Option<RenderedValue>,
    /// The time awake, as the core renders it.
    pub awake: Option<RenderedValue>,
}

/// The nights of sleep.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SleepDays {
    /// The nights, oldest first.
    pub days: Vec<SleepDay>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One morning's heart-rate variability.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HrvDay {
    /// The athlete's local date.
    pub date: String,
    /// The heart-rate variability, as the core renders it.
    pub hrv: Option<RenderedValue>,
    /// The resting heart rate, as the core renders it.
    pub resting_hr: Option<RenderedValue>,
}

/// The mornings.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HrvDays {
    /// The mornings, oldest first.
    pub days: Vec<HrvDay>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The most recent day of wellness.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LatestPhysiology {
    /// The day itself. `null` when nothing has been synced.
    pub day: Option<PhysiologyDay>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// Where one kind of measurement comes from.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SourceInfo {
    /// The measurement's name: `sleep`, `hrv`, `resting_hr`.
    pub name: String,
    /// How many days of it there are.
    pub days: i64,
    /// The most recent day of it.
    pub latest: Option<String>,
}

/// The sources of the wellness measurements.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Sources {
    /// The sources.
    pub sources: Vec<SourceInfo>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The band a direction was measured against.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VitalBand {
    /// The lower edge: the usual, less one and a half deviations.
    pub lower: f64,
    /// The upper edge: the usual, plus one and a half deviations.
    pub upper: f64,
    /// How many nights the band was measured from.
    pub nights: usize,
}

/// One vital, as the core renders it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VitalReading {
    /// Which vital this is: `sleep`, `hrv`, `resting_hr`.
    pub key: String,
    /// The label the app shows.
    pub label: String,
    /// The value, rendered: "6 h 10 m", "48 ms", "63 bpm".
    pub value_text: Option<String>,
    /// The direction word: "could be better".
    pub direction_text: Option<String>,
    /// The arithmetic: "1 h 10 m less than your usual".
    pub deviation_text: Option<String>,
    /// The direction of travel: "3rd night down".
    pub travel_text: Option<String>,
    /// What the row says when there is no comparison to make.
    pub status_text: Option<String>,
    /// The band, for the chart.
    pub band: Option<VitalBand>,
}

/// The "Rest" row, as the core renders it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RestReading {
    /// The label of the row.
    pub label: String,
    /// The line, already written.
    pub line: Option<String>,
}

/// One night of history, for the charts.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HistoryPoint {
    /// The athlete's local date.
    pub date: String,
    /// The value that night. `null` when there was none.
    pub value: Option<f64>,
    /// The core-rendered value for axes and tooltips, with its unit.
    pub text: Option<String>,
}

/// The history of the three vitals.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct VitalsHistory {
    /// The nights of sleep.
    pub sleep: Vec<HistoryPoint>,
    /// The mornings of HRV.
    pub hrv: Vec<HistoryPoint>,
    /// The mornings of resting heart rate.
    pub resting_hr: Vec<HistoryPoint>,
}

/// A pattern: a fact about the past, with its sample size.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PatternReading {
    /// The pattern's id.
    pub id: String,
    /// The sentence, already written.
    pub text: String,
    /// How many comparable days it was read from.
    pub sample_size: usize,
}

/// The vitals and their directions — the same snapshot Today and Body read.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VitalsSnapshot {
    /// The day the snapshot is of.
    pub date: String,
    /// When this answer was produced.
    pub generated_at: String,
    /// The "Rest" row.
    pub rest: RestReading,
    /// The three vitals, in the order the app shows them.
    pub vitals: Vec<VitalReading>,
    /// The history, for the charts.
    pub history: VitalsHistory,
    /// The patterns, with their sample sizes.
    pub patterns: Vec<PatternReading>,
}

// ------------------------------------------------------------------ fitness

/// One run's fingerprint: the run's own mechanics, as **observations**.
///
/// The fingerprint is what the run's laps say about the run — the cadence,
/// the pace and the heart rate as they moved across it, and the shares of
/// the moving time in the athlete's own zones. Every observation is either
/// a value or the reason it has none; **there is no score**, because a
/// weighted mean of a run's numbers is a grade and a grade is a verdict.
///
/// The numbers are the core's: the computation lives in `stride-core`, and
/// this payload carries what the core rendered.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Fingerprint {
    /// The run's id.
    pub activity_id: i64,
    /// The athlete's local date.
    pub local_date: String,
    /// The basis the numbers were computed on: the run's own laps and its
    /// moving samples. `None` when there is nothing to compute them from.
    pub basis: Option<String>,
    /// How many laps the analysis used: the running ones, and the ones long
    /// enough to measure. A fragment — a lap too short for its pace to mean
    /// anything — is left out, and is not counted.
    pub laps: usize,
    /// The observations, in the order the app shows them. Empty when the
    /// run has no file to read.
    pub observations: Vec<FingerprintObservation>,
    /// The zone distribution: the shares of the moving time in the
    /// athlete's own zones.
    pub zones: FingerprintZones,
    /// Why there is no value at all, in the core's words.
    pub unavailable: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One observation of a run's mechanics.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FingerprintObservation {
    /// The observation's key: `cadence`, `cadence_variability`, and so on.
    pub key: String,
    /// The label, in the core's own words.
    pub label: String,
    /// The value, as the core renders it. `None` when there is no value.
    pub value: Option<RenderedValue>,
    /// Why there is no value, in the core's words. `Some` exactly when
    /// `value` is `None`.
    pub reason: Option<String>,
}

/// The zone distribution of a run, against the athlete's **stored** model.
///
/// The shares are of the time **in the zones**: the time outside them — no
/// heart-rate reading, or a reading at or below the first floor — is
/// reported on its own, because it is not part of any zone.
///
/// **No value is the absence of the whole result**: a run with no stored
/// model has no shares, and says why in the core's words.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FingerprintZones {
    /// The model the shares were computed against, named and dated.
    /// `None` when no model has been read.
    pub model: Option<ZoneModel>,
    /// The shares, one per zone. Empty when there is no value.
    pub shares: Vec<ZoneShare>,
    /// The time outside the zones. Absent when there is no value.
    pub outside: Option<RenderedValue>,
    /// Why there is no value, in the core's words.
    pub unavailable: Option<String>,
    /// Where the model came from, in the core's words.
    pub source: Option<String>,
}

/// One zone's share of the moving time.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneShare {
    /// The zone's name, as the core names it: "Z1".
    pub label: String,
    /// The zone's share of the time in the zones, as the core renders it.
    pub share: RenderedValue,
    /// The time spent in the zone, as the core renders it.
    pub time: Option<RenderedValue>,
}

/// One point of a trend.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TrendPoint {
    /// The athlete's local date.
    pub date: String,
    /// The value on that day, as the core renders it.
    pub value: Option<RenderedValue>,
}

/// One measured trend.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Trend {
    /// The metric's name.
    pub metric: String,
    /// The metric's label.
    pub label: String,
    /// The unit the values are in.
    pub unit: String,
    /// The points, oldest first.
    pub points: Vec<TrendPoint>,
    /// The median of the points, when there are any.
    pub baseline: Option<RenderedValue>,
}

/// The measured trends.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Trends {
    /// The trends.
    pub trends: Vec<Trend>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One heart-rate band of a run.
///
/// The bands are the athlete's own zones: their edges are the floors of
/// the **stored zone model**, not tenths of this run's maximum heart rate.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneBand {
    /// The band's name, as the core names it: "Z2".
    pub label: String,
    /// The band's range, in the core's words: "106-121 bpm".
    pub range: Option<String>,
    /// The band's lower edge, in beats per minute — the number the core
    /// rendered the range from, for a chart to draw.
    pub from_bpm: Option<i64>,
    /// The band's upper edge, in beats per minute. `None` for the top band
    /// when the model carries no maximum heart rate: that band has no
    /// upper edge.
    pub to_bpm: Option<i64>,
    /// The time spent in the band, as the core renders it: "12 m". The
    /// number beside the words is the time in **seconds**.
    pub time: Option<RenderedValue>,
}

/// The heart-rate zones of a run, computed from the athlete's **stored**
/// zone model.
///
/// The model is the athlete's own configuration, read from Garmin and
/// stored with the day it was read. The payload names it — its sport, its
/// method, its maximum heart rate, its five floors and the day it was read
/// — so a number can say where it came from and when.
///
/// **No value is the absence of the whole result**: a run with no stored
/// model has no bands and no time outside them, and says why in the core's
/// words. It is never band 1 and never a zero.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FitnessZones {
    /// The basis the times were computed on: "moving".
    pub basis: Option<String>,
    /// The model the times were computed against. `None` when no model has
    /// been read; present even when the model cannot be used, so the reader
    /// can see which model was rejected.
    pub model: Option<ZoneModel>,
    /// The five bands, each with the time spent in it. Empty when there is
    /// no value.
    pub zones: Vec<ZoneBand>,
    /// The time outside the zones: no heart-rate reading, or a reading at
    /// or below the first floor. Absent when there is no value.
    pub outside: Option<RenderedValue>,
    /// Why there is no value, in the core's words.
    pub unavailable: Option<String>,
}

/// The athlete's zone model: a dated fact about their configuration.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneModel {
    /// The sport Garmin reported the model for: "DEFAULT".
    pub sport: String,
    /// The training method the model was set with: "HR_MAX".
    pub training_method: String,
    /// The maximum heart rate the model was built on, when Garmin reported
    /// one: "178 bpm".
    pub max_heart_rate: Option<RenderedValue>,
    /// The five floors, in beats per minute.
    pub floors: ZoneModelFloors,
    /// When the model was read from Garmin: "2026-05-10T06:30:00Z".
    pub read_at: String,
    /// The day the model was read, in the athlete's own calendar, in the
    /// api's own date rendering: "10 May 2026".
    ///
    /// The api renders the day; the core's canonical form stays inside the
    /// core, and the surface shows the api's.
    pub read_on: String,
    /// The model's own source line, in the core's words: the method it was
    /// set with, the maximum heart rate it was built on and the day it was
    /// read — "your zone model · HR_MAX · max heart rate 178 bpm · read on
    /// 10 May 2026".
    pub source: Option<String>,
}

/// The five floors of a zone model.
///
/// The floors are the **exclusive lower bounds** of the zones: the interval
/// of zone *n* is `(floor_n, floor_{n+1}]`, and a reading at or below the
/// first floor is outside the zones.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneModelFloors {
    /// The floor of zone 1: "106 bpm".
    pub zone1: RenderedValue,
    /// The floor of zone 2: "121 bpm".
    pub zone2: RenderedValue,
    /// The floor of zone 3: "135 bpm".
    pub zone3: RenderedValue,
    /// The floor of zone 4: "149 bpm".
    pub zone4: RenderedValue,
    /// The floor of zone 5: "163 bpm".
    pub zone5: RenderedValue,
}

// ---------------------------------------------------------------- the film

/// One sample of a run's film.
///
/// The sample is the run's own: decoded from its FIT file on read, and
/// never stored. It carries the zone it sits in, measured against the
/// athlete's **stored** zone model.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FilmSample {
    /// The time from the start of the run, as the core renders it: "42 m".
    ///
    /// The number beside the words is the time in **seconds**, and it is
    /// the number the film's timeline draws on.
    pub elapsed: RenderedValue,
    /// The distance from the start, as the core renders it: "3.2 km".
    ///
    /// The number beside the words is the distance in **metres**. It is
    /// `null` when the file carries no distance for that sample.
    pub distance: Option<RenderedValue>,
    /// The pace over the sample, as the core renders it: "5:24 /km".
    pub pace: Option<RenderedValue>,
    /// The heart rate over the sample, as the core renders it: "148 bpm".
    pub heart_rate: Option<RenderedValue>,
    /// The zone the sample sits in: the index of the band in the film's
    /// `zones`, counted from zero.
    ///
    /// `null` when the reading is **outside the zones** — at or below the
    /// first floor, or not there at all — and when the athlete has no
    /// usable zone model, in which case the whole ribbon is absent.
    pub zone: Option<i64>,
}

/// One point of the run's ground.
///
/// The ground is on the film's own time axis: the point says how far into
/// the run it is, so the pace, the heart rate and the ground share one
/// cursor.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FilmGround {
    /// The seconds from the start of the run.
    pub offset_s: f64,
    /// The altitude, as the core renders it: "926 m".
    pub elevation: RenderedValue,
}

/// One split of a run, as the film draws it.
///
/// The split carries the time it **began**: a split is a place on the
/// timeline, and the film can be played from it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FilmSplit {
    /// The lap's number, from one — a lap, not a kilometre marker.
    pub lap_index: i64,
    /// The seconds from the start of the run to the start of the split.
    pub offset_s: f64,
    /// The split's distance, as the core renders it.
    pub distance: RenderedValue,
    /// The split's duration, as the core renders it.
    pub duration: RenderedValue,
    /// The average heart rate over the split, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
    /// The split's pace, as the core renders it.
    pub pace: Option<RenderedValue>,
}

/// One band of the athlete's zones, as the film draws it.
///
/// The band is the model's own: its name and its range are the core's, and
/// its edges are the numbers the core rendered the range from.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FilmZone {
    /// The band's name, as the core names it: "Z2".
    pub label: String,
    /// The band's range, in the core's words: "106-121 bpm".
    pub range: Option<String>,
    /// The band's lower edge, in beats per minute — for drawing.
    pub from_bpm: Option<i64>,
    /// The band's upper edge, in beats per minute. `None` for the top band
    /// when the model carries no maximum heart rate.
    pub to_bpm: Option<i64>,
}

/// A run's film: the run, playable.
///
/// The series is the run's own, decoded from its FIT file on read: the
/// samples with their pace, their heart rate and **the zone each one sits
/// in**, the ground, and the splits. The zone model the zones were
/// measured against is named and dated beside them.
///
/// **No value is the absence of the whole result.** A run with no file has
/// no series at all, and says why in the core's words. It is never a zero
/// and never a band.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunFilm {
    /// The run's id.
    pub activity_id: i64,
    /// The athlete's local date of the run.
    pub local_date: String,
    /// The basis the series was read on: "samples".
    pub basis: Option<String>,
    /// The zone model the zones were measured against, named and dated.
    /// `None` when no model has been read.
    pub model: Option<ZoneModel>,
    /// The five bands of the athlete's zones. Empty when there is no usable
    /// model, and then the ribbon is absent.
    pub zones: Vec<FilmZone>,
    /// The name of the space **outside** the zones, in the core's words:
    /// "outside your zones". `None` when there is no usable model.
    pub outside_label: Option<String>,
    /// The run's samples.
    pub samples: Vec<FilmSample>,
    /// The run's ground.
    pub ground: Vec<FilmGround>,
    /// The run's splits.
    pub splits: Vec<FilmSplit>,
    /// Why there is no film at all, in the core's words.
    pub unavailable: Option<String>,
    /// Why the zones have no value, in the core's words. The film itself
    /// may still be there: the pace, the heart rate and the ground do not
    /// need a zone model.
    pub zones_unavailable: Option<String>,
    /// When the answer was produced.
    pub generated_at: String,
}

// -------------------------------------------------------------------- stats

/// One set of totals.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Totals {
    /// How many runs there are.
    pub activity_count: i64,
    /// The distance, as the core renders it.
    pub distance: Option<RenderedValue>,
    /// The elevation gain, as the core renders it.
    pub elevation_gain: Option<RenderedValue>,
    /// The moving time, as the core renders it.
    pub duration: Option<RenderedValue>,
    /// The average pace, as the core renders it.
    pub avg_pace: Option<RenderedValue>,
    /// The average heart rate, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
}

/// The totals, over four windows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StatsSummary {
    /// Every run there is.
    pub all_time: Totals,
    /// The runs of this year.
    pub this_year: Totals,
    /// The runs of this month.
    pub this_month: Totals,
    /// The runs of this week.
    pub this_week: Totals,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One week of running.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WeekTotals {
    /// The first day of the week.
    pub week_start: String,
    /// How many runs there are.
    pub activity_count: i64,
    /// The distance, as the core renders it.
    pub distance: Option<RenderedValue>,
    /// The moving time, as the core renders it.
    pub duration: Option<RenderedValue>,
    /// The elevation gain, as the core renders it.
    pub elevation_gain: Option<RenderedValue>,
    /// The average pace, as the core renders it.
    pub avg_pace: Option<RenderedValue>,
}

/// The weeks of running.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WeeklyStats {
    /// The weeks, oldest first.
    pub weeks: Vec<WeekTotals>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One week of pace.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceWeek {
    /// The first day of the week.
    pub week_start: String,
    /// How many runs there are.
    pub run_count: i64,
    /// The median pace, as the core renders it.
    pub median_pace: Option<RenderedValue>,
    /// The fastest pace, as the core renders it.
    pub best_pace: Option<RenderedValue>,
}

/// The pace, week by week.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceTrends {
    /// The weeks, oldest first.
    pub weeks: Vec<PaceWeek>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One week of climbing.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ElevationWeek {
    /// The first day of the week.
    pub week_start: String,
    /// The elevation gain, as the core renders it.
    pub elevation_gain: Option<RenderedValue>,
}

/// The climbing, week by week.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ElevationStats {
    /// The weeks, oldest first.
    pub weeks: Vec<ElevationWeek>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One personal record.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PersonalRecord {
    /// The record's key: `progress.best.5k` and the like.
    pub key: String,
    /// The record's label.
    pub label: String,
    /// The value, as the core renders it.
    pub value: Option<RenderedValue>,
    /// The run the record was set in.
    pub activity_id: i64,
    /// The day the record was set on.
    pub activity_date: String,
}

/// The personal records.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PersonalRecords {
    /// The records.
    pub records: Vec<PersonalRecord>,
    /// When this answer was produced.
    pub generated_at: String,
}

// --------------------------------------------------------------------- plan

/// One planned session.
///
/// The app's shape, field for field: the session's own title, the session
/// already rendered by the core, the target and the actual as rendered
/// values, and the plain status — *done*, *planned* or *not done*.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanSession {
    /// The day of the session.
    pub date: String,
    /// The plan's own vocabulary: `easy`, `long`, `tempo`, `rest`.
    pub session_type: String,
    /// The session's own title, in the athlete's words.
    pub title: Option<String>,
    /// The session, already rendered: "Today — 10 km · easy".
    pub sentence: String,
    /// The target, as the core renders it.
    pub target: Option<RenderedValue>,
    /// What was actually run, as the core renders it.
    pub actual: Option<RenderedValue>,
    /// `done`, `planned` or `not_done`.
    pub status: String,
    /// Where the plan came from: `baseline`, `prescribed` or `typed`.
    pub source: String,
}

/// One plan block.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanBlock {
    /// The block's id. `null` for the baseline, which is not stored.
    pub id: Option<i64>,
    /// The block's name.
    pub name: String,
    /// Where the block came from.
    pub source: String,
    /// The first day of the block.
    pub start_date: String,
    /// The last day of the block. `null` when the block is open-ended.
    pub end_date: Option<String>,
    /// The block's goal.
    pub goal: Option<String>,
    /// `active` or `archived`.
    pub status: String,
    /// How many sessions the block holds.
    pub session_count: usize,
    /// Whether the block is a **baseline**: inferred from the athlete's own
    /// history rather than written down.
    ///
    /// It changes what the app may say about the plan: a baseline is a
    /// source, not a prescription.
    pub is_baseline: bool,
}

/// The plan's week.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanWeek {
    /// The name of the block the week is in.
    pub block_name: String,
    /// Where the block came from.
    pub source: String,
    /// The first day of the week.
    pub week_start: String,
    /// The last day of the week.
    pub week_end: String,
    /// The athlete's day, as the api counts it.
    ///
    /// The app highlights it inside the week, and the app does not keep a
    /// clock of its own for it: the api's day is the athlete's day.
    pub today: String,
    /// The week's target, as the core renders it.
    pub target: Option<RenderedValue>,
    /// The week's sessions, in the order they fall.
    pub sessions: Vec<PlanSession>,
    /// How many of the week's sessions were run.
    pub adherence: Option<Adherence>,
    /// The week's counts, by status.
    ///
    /// Three numbers and never a percentage: the api counts them, so the
    /// app never counts the plan's sessions itself.
    pub counts: Counts,
}

/// How many of the planned sessions were run.
///
/// A count, never a score: there is no percentage and no grade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Adherence {
    /// How many were run.
    pub done: usize,
    /// How many were planned.
    pub planned: usize,
}

/// The count by status: planned, done and not done.
///
/// Three numbers and never a percentage: a percentage hides the count, and
/// the count is the fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Counts {
    /// How many were run.
    pub done: usize,
    /// How many were planned.
    pub planned: usize,
    /// How many were planned and did not happen.
    ///
    /// A **baseline** never counts here: an inferred week is not a
    /// prescription.
    pub not_done: usize,
}

/// The plan's blocks.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanBlocks {
    /// The blocks.
    pub blocks: Vec<PlanBlock>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The plan in force.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CurrentPlan {
    /// The block in force. `null` when there is none.
    pub block: Option<PlanBlock>,
    /// The source of the plan in force, always named.
    ///
    /// `baseline` when the athlete has no plan of their own: the baseline is
    /// what the app infers from the athlete's own history, and it is a
    /// source rather than a prescription.
    pub source: String,
    /// Whether the plan in force is a baseline.
    pub is_baseline: bool,
    /// How many weeks of history the baseline was read from.
    ///
    /// `null` when the plan is not a baseline.
    pub baseline_weeks: Option<usize>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The plan's day.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanToday {
    /// The day.
    pub date: String,
    /// The session the plan has for the day. `null` when it has none.
    pub session: Option<PlanSession>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One plan source.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanSource {
    /// The source's name.
    pub name: String,
    /// How many blocks the source holds.
    pub blocks: usize,
    /// Whether the source has a block in force.
    pub active: bool,
}

/// The plan's sources.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanSources {
    /// The sources.
    pub sources: Vec<PlanSource>,
    /// The source in force, always named.
    pub active_source: String,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The adherence, over a window.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanAdherence {
    /// How many of the planned sessions were run.
    pub adherence: Adherence,
    /// The count by status: planned, done and not done.
    pub counts: Counts,
    /// The weeks of the window.
    pub weeks: Vec<AdherenceWeek>,
    /// Whether the adherence is suspended.
    ///
    /// A pause suspends the count: the sessions of a paused week were not
    /// missed, and the answer says so rather than counting them.
    pub suspended: bool,
    /// Why the adherence is suspended, in the athlete's own word: `travel`,
    /// `sick` or `injured`.
    pub suspension_reason: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One week of adherence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdherenceWeek {
    /// The first day of the week.
    pub week_start: String,
    /// How many of the week's sessions were run.
    pub done: usize,
    /// How many the week planned.
    pub planned: usize,
    /// How many the week planned and did not happen.
    pub not_done: usize,
}

/// One planned session and what happened.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanResult {
    /// The session's id.
    ///
    /// `null` for a session of the baseline: the baseline is inferred, and
    /// it is not stored.
    pub session_id: Option<i64>,
    /// The day of the session.
    pub date: String,
    /// The plan's own vocabulary.
    pub session_type: String,
    /// The session's label, as the core renders it.
    pub label: String,
    /// `done`, `planned` or `not_done`.
    pub status: String,
    /// The distance that was planned, as the core renders it.
    pub target: Option<RenderedValue>,
    /// The distance that was run, as the core renders it.
    pub actual: Option<RenderedValue>,
    /// The difference between the two, as the core states it.
    ///
    /// "2.4 km more than the 10 km target" — and nothing at all when the
    /// gap is within ten per cent: 10.2 km against a 10 km target is not
    /// news, and neither more nor less is good or bad.
    pub difference: Option<String>,
}

/// The plan's results.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanResults {
    /// The results, oldest first.
    pub results: Vec<PlanResult>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to an import.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanImport {
    /// Whether the plan was imported.
    pub imported: bool,
    /// The block that was written.
    ///
    /// `null` when nothing was written.
    pub block_id: Option<i64>,
    /// How many sessions were written.
    pub sessions: usize,
    /// When this answer was produced.
    pub generated_at: String,
}

// ----------------------------------------------------------------- settings

/// The weekly goal.
///
/// **Blank means no goal.** There is no default: the app never invents a
/// number the athlete did not write, and a goal of zero would read as a
/// failure rather than as the absence of one.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsGoal {
    /// The goal, as the core renders it: "40 km".
    ///
    /// `null` when there is no goal.
    pub value: Option<RenderedValue>,
    /// The goal in kilometres, for the field: 40.
    ///
    /// `null` when there is no goal.
    pub km: Option<f64>,
}

/// A pause in the plan: a start, an end and a reason.
///
/// The reason is one of three words, not free text: a run's note is the
/// only free text the product stores.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsPause {
    /// The first day of the pause.
    pub start_date: String,
    /// The last day of the pause.
    pub end_date: String,
    /// Why: `travel`, `sick` or `injured`. `null` when there is no reason.
    pub reason: Option<String>,
}

/// The storage: what is on the device, and how big it is.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsStorage {
    /// The size of the database, as the host renders it: "2.4 MB".
    pub size: Option<RenderedValue>,
    /// How many runs are in it.
    pub activity_count: i64,
    /// How many wellness days are in it.
    pub wellness_day_count: i64,
    /// How many planned sessions are in it.
    pub plan_session_count: i64,
    /// How many notes are in it — the only free text the app stores.
    pub note_count: i64,
}

/// The settings the athlete has set.
///
/// The goal, the pause and the storage. The connection is not here: it lives
/// on `/auth/status`, which owns it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsState {
    /// The weekly goal.
    pub goal: SettingsGoal,
    /// The pause, when there is one.
    pub pause: Option<SettingsPause>,
    /// The storage.
    pub storage: SettingsStorage,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to exporting a backup.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsExportResult {
    /// Whether the backup was written.
    pub exported: bool,
    /// How big it is, as the host renders it: "2.4 MB".
    pub size: Option<RenderedValue>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to deleting everything.
///
/// The typed confirmation is the caller's: the word has to arrive with the
/// request, so the answer can only say what happened.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingsDeleteResult {
    /// Whether the data was deleted.
    pub deleted: bool,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to writing a note on a run.
///
/// A note is the only free text the product stores, and it belongs to the
/// run it is about.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NoteResult {
    /// The note as it stands now: the athlete's own words.
    pub note: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

// ------------------------------------------------------------------- export

/// The export that was written.
///
/// The router's own shape: the rig is the one that reads an export back,
/// and it wants to know what was written and where.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportWritten {
    /// The name of the directory the export was written into.
    ///
    /// The name, not the path: the path is the rig's own, and the athlete
    /// moves the file, not the router.
    pub directory: String,
    /// How many rows the export holds.
    pub rows: i64,
    /// How many rows each table holds.
    pub tables: std::collections::BTreeMap<String, i64>,
    /// The schema version of the export.
    pub schema_version: i64,
    /// The size of the exported database, in bytes.
    pub database_bytes: u64,
    /// The SHA-256 of the exported database.
    pub database_sha256: String,
    /// When the export was made.
    pub created_at: String,
    /// When this answer was produced.
    pub generated_at: String,
}

/// What an import read.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImportRead {
    /// How many runs were written.
    pub activities: usize,
    /// How many days of wellness were written.
    pub wellness: usize,
    /// How many plan blocks were written.
    pub plan_blocks: usize,
    /// How many planned sessions were written.
    pub plan_sessions: usize,
    /// How many notes were written.
    pub notes: usize,
    /// How many rows there were in all.
    pub rows: usize,
    /// When this answer was produced.
    pub generated_at: String,
}

// ------------------------------------------------------------------ insights

/// The plan's session for today.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TodayInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The day.
    pub date: String,
    /// The plan's own session for the day.
    pub planned: Option<PlanSession>,
    /// The plan's other sessions — unranked, unlinked.
    pub options: Vec<PlanSession>,
    /// The last run.
    pub last_run: Option<ActivitySummary>,
    /// The distance comparison, when there is one.
    ///
    /// The core states it, and states it only when the gap from the target
    /// is beyond 10%: "2.4 km more than the 10 km target". Within 10% it is
    /// absent — a 2% difference is not news.
    pub distance_comparison: Option<String>,
    /// This week's running.
    pub week: WeekRunning,
}

/// This week's running.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WeekRunning {
    /// The distance, as the core renders it.
    pub distance: RenderedValue,
    /// How many runs there were.
    pub sessions: usize,
    /// Recorded activity counts for each calendar day, including future days.
    pub days: Vec<RunningDay>,
    /// The same elapsed days of the previous calendar week.
    ///
    /// The comparison is **like for like**: the week to date against the
    /// same days of the week before, so the two windows are the same
    /// length. A week to date against a whole week would compare a part
    /// with the whole.
    pub previous: PreviousWeek,
    /// This week's distance against the previous week's, as the core
    /// renders the difference: the sign is the number's own.
    pub difference: Option<RenderedValue>,
}

/// The totals of a stretch of days.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PreviousWeek {
    /// The first day of the stretch, in the athlete's own calendar.
    pub from: String,
    /// The last day of the stretch, in the athlete's own calendar.
    pub to: String,
    /// The distance, as the core renders it.
    pub distance: RenderedValue,
    /// How many runs there were.
    pub sessions: usize,
}

/// One marker in the current calendar week.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunningDay {
    /// The local date.
    pub date: String,
    /// Runs already recorded on this day, not a coverage or rest-day claim.
    pub sessions: usize,
    /// The recorded distance, not a coverage or rest-day claim.
    pub distance: RenderedValue,
}

/// One week of volume.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WeeklyVolume {
    /// The first day of the week.
    pub week_start: String,
    /// The distance, as the core renders it.
    pub distance: RenderedValue,
}

/// One week of pace at a heart rate.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceAtHr {
    /// The first day of the week.
    pub week_start: String,
    /// The median pace of the week, as the core renders it.
    pub pace: Option<RenderedValue>,
    /// The median heart rate of the week, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
}

/// One best.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Best {
    /// The catalogue key that names the row.
    pub key: String,
    /// The value, as the core renders it.
    pub value: String,
    /// The day it was set on.
    pub date: String,
}

/// One week of consistency.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Consistency {
    /// The first day of the week.
    pub week_start: String,
    /// How many runs there were.
    pub sessions: usize,
}

/// The Progress screen's data.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProgressInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The local calendar date used by the core's weekly grouping.
    pub date: String,
    /// Totals for the selected calendar-week window.
    pub period: ProgressPeriod,
    /// The headline, already written.
    pub headline: String,
    /// The evidence under it.
    pub headline_note: Option<String>,
    /// The volume, week by week.
    pub weekly_volume: Vec<WeeklyVolume>,
    /// The typical week, as the core renders it.
    pub typical_week: Option<RenderedValue>,
    /// The pace at a heart rate, week by week.
    pub pace_at_hr: Vec<PaceAtHr>,
    /// The bests.
    pub bests: Vec<Best>,
    /// The consistency, week by week.
    pub consistency: Vec<Consistency>,
    /// The sessions a week, as the core renders them.
    pub sessions_per_week: Option<RenderedValue>,
    /// The plan's adherence.
    pub adherence: Option<Adherence>,
    /// How many more weeks the charts need before they can draw a trend.
    ///
    /// `null` when they have what they need — and `null` until the core
    /// has a rule for how long a trend is: the number is the core's rule,
    /// and this crate does not keep a second one.
    pub weeks_needed: Option<i64>,
    /// The Eddington number, over the window the screen is showing.
    ///
    /// `None` when the window holds no distance: a window with no
    /// distance has no number, and absence is not zero.
    pub eddington: Option<ProgressEddington>,
}

/// The Eddington number and its two companions, over the Progress window.
///
/// The number is the largest *E* such that at least *E* days in the window
/// covered at least *E* kilometres — consistency over peaks. It is counted
/// in kilometres, the contract's unit, and the count travels with the
/// days still needed for the next number and the longest distance in a
/// day.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProgressEddington {
    /// The number: the largest *E* with at least *E* days at or above *E*
    /// kilometres in the window.
    pub number: u32,
    /// The days still needed to reach the next number.
    pub days_needed_for_next: u32,
    /// The longest distance in a day in the window, as the core renders it.
    pub longest_day: Option<RenderedValue>,
}

/// Recorded activity within a selected progress window.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProgressPeriod {
    /// First included local date.
    pub from: String,
    /// Last included local date.
    pub to: String,
    /// Total recorded distance.
    pub distance: RenderedValue,
    /// Total recorded moving time.
    pub duration: RenderedValue,
    /// Number of recorded sessions.
    pub sessions: usize,
    /// Calendar weeks containing recorded sessions.
    pub recorded_weeks: usize,
}

/// One chip about a run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Chip {
    /// The chip's key.
    pub key: String,
    /// The chip's label.
    pub label: String,
    /// The chip's value, as the core renders it.
    pub value: Option<RenderedValue>,
}

/// The chips of one run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Chips {
    /// The run the chips are about.
    pub activity_id: i64,
    /// The chips.
    pub chips: Vec<Chip>,
    /// When this answer was produced.
    pub generated_at: String,
}

// ------------------------------------------------------------------- compare

/// One row of the metric diff.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompareMetric {
    /// The metric's key.
    pub key: String,
    /// The catalogue key that names the row.
    pub label_key: String,
    /// The first run's value, as the core renders it.
    pub a: Option<RenderedValue>,
    /// The second run's value, as the core renders it.
    pub b: Option<RenderedValue>,
    /// B − A, in the metric's own unit.
    pub delta: Option<RenderedValue>,
}

/// The identity of a run, in a comparison.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompareRun {
    /// The run's id.
    pub id: i64,
    /// The run's label.
    pub label: String,
    /// The day of the run.
    pub date: String,
    /// The distance, as the core renders it.
    pub distance: RenderedValue,
    /// The one sentence about what the run meant.
    pub story: Option<String>,
}

/// One run's series, in a comparison.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompareSeries {
    /// The first run's series.
    pub a: Vec<SamplePoint>,
    /// The second run's series.
    pub b: Vec<SamplePoint>,
}

/// The comparison itself.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompareResponse {
    /// The basis, already written: "Same distance, 12 days apart".
    pub basis_text: String,
    /// The rows of the diff.
    pub metrics: Vec<CompareMetric>,
    /// The first run.
    pub a: CompareRun,
    /// The second run.
    pub b: CompareRun,
    /// The two series.
    pub series: CompareSeries,
}

/// A run that can be compared against.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompareCandidate {
    /// The run's id.
    pub id: i64,
    /// The run's label.
    pub label: String,
    /// The day of the run.
    pub date: String,
    /// The distance, as the core renders it.
    pub distance: RenderedValue,
}

// ---------------------------------------------------------------------- app

/// The health of the sync window.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HealthDaily {
    /// The day.
    pub date: String,
    /// When this answer was produced.
    pub generated_at: String,
    /// Whether the data is older than the window.
    pub stale: bool,
}

/// The liveness of the app.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Health {
    /// Whether the router is answering.
    pub ok: bool,
    /// The service's name.
    pub service: String,
    /// The service's version.
    pub version: String,
    /// Whether a Garmin key is held.
    pub garmin_connected: bool,
    /// How many runs are stored.
    pub activities: i64,
    /// How many days of wellness are stored.
    pub wellness_days: i64,
    /// How many planned sessions are stored.
    pub plan_sessions: i64,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The state of the last sync.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncStatus {
    /// When the last sync succeeded.
    pub last_sync_at: Option<String>,
    /// The last failure, when there was one.
    pub last_error: Option<String>,
    /// How many runs are stored.
    pub activity_count: i64,
    /// The newest Garmin activity the app has already pulled.
    ///
    /// The pull is a delta: the next one asks Garmin for what is after
    /// this, so a known run is never downloaded twice.
    pub cursor: Option<String>,
    /// The runs the last pull downloaded.
    pub pulled: Option<i64>,
    /// The runs the last pull recognised and did not download again.
    pub skipped: Option<i64>,
    /// The runs Garmin no longer lists, marked and not resurrected.
    pub deleted: Option<i64>,
    /// Checked date ranges, including successful no-data days.
    pub history: Option<SyncHistoryCoverage>,
    /// Whether a bounded history job still has dated work to do.
    pub more_pending: bool,
}

/// Independently checkpointed import resources.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncHistoryCoverage {
    /// Activity listing coverage, not an upstream retention guarantee.
    pub activities: Option<SyncResourceCoverage>,
    /// Wellness endpoint coverage, not measurement availability.
    pub wellness: Option<SyncResourceCoverage>,
}

/// One contiguous checked range and its next older date.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncResourceCoverage {
    /// The fixed oldest requested date.
    pub target_from: String,
    /// The oldest date checked without a gap.
    pub checked_from: Option<String>,
    /// The newest date checked.
    pub checked_to: Option<String>,
    /// The next date to check, or null when the target is exhausted.
    pub next_date: Option<String>,
}

/// A surfaced failure after partial data has landed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyncFailure {
    /// The router error code.
    pub code: String,
    /// A credential-free explanation.
    pub detail: String,
    /// The status used by the native failure classifier.
    pub status: u16,
    /// A provider-supplied retry delay, when present.
    pub retry_after_seconds: Option<u64>,
}

/// What a pull brought in.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncRun {
    /// How many runs the pull brought in.
    pub activities: usize,
    /// The runs the pull recognised and did not download again.
    pub skipped: usize,
    /// The runs Garmin no longer lists, marked and not resurrected.
    pub deleted: usize,
    /// How many days of wellness the pull brought in.
    pub wellness_days: usize,
    /// Whether all requested dated history is checked without a failure.
    pub complete: bool,
    /// Whether another bounded history job is needed.
    pub more_pending: bool,
    /// A partial failure; normal budget pauses are not failures.
    pub failure: Option<SyncFailure>,
    /// When the pull finished.
    pub finished_at: String,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One run that came from Garmin.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncedRun {
    /// The run's id.
    pub id: i64,
    /// The run's title.
    pub name: String,
    /// The day of the run.
    pub local_date: String,
    /// The distance, as the core renders it.
    pub distance: Option<RenderedValue>,
    /// When the run was synced.
    pub updated_at: String,
}

/// The runs that came from Garmin.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncedRuns {
    /// The runs, newest first.
    pub runs: Vec<SyncedRun>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The state of one kind of sync.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncHistoryEntry {
    /// Which kind of data.
    pub kind: String,
    /// When the last sync of this kind succeeded.
    pub last_success_at: Option<String>,
    /// When the disposable access value ends.
    pub token_expires_at: Option<String>,
}

/// The sync history.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncHistory {
    /// The kinds of data that are synced.
    pub entries: Vec<SyncHistoryEntry>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The sync settings.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncSettings {
    /// How many days a pull reaches back.
    pub sync_days: i64,
    /// How many months of history the app keeps.
    pub history_months: i64,
    /// How many days of health data the app keeps.
    pub health_history_days: i64,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The state of the connection.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AuthStatus {
    /// Whether a Garmin key is held.
    pub connected: bool,
    /// The method the key is used with.
    pub method: Option<String>,
    /// Where the key comes from.
    pub key_source: String,
    /// The last four characters of the key — and never the key.
    ///
    /// `null` when no key is held.
    pub key_last4: Option<String>,
    /// When the last sync succeeded.
    pub last_sync_at: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to a connection.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AuthConnect {
    /// Whether the key is held.
    pub connected: bool,
    /// The method the key is used with.
    pub method: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to a key test.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AuthTest {
    /// Whether Garmin accepted the key.
    pub ok: bool,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The answer to importing a FIT file.
///
/// A file that is already in the log is a duplicate, and a duplicate adds
/// no row — it is reported, once, in one line.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FitImport {
    /// Whether the file became a run.
    pub imported: bool,
    /// Whether the file was a run that is already in the log.
    pub duplicate: bool,
    /// The run the file is: the one it became, or the one already there.
    pub activity_id: Option<i64>,
    /// The run's day, when the file was a duplicate.
    pub date: Option<String>,
    /// The run's distance, as the core renders it: "10.2 km".
    pub distance: Option<RenderedValue>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// One day of the calendar.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CalendarDay {
    /// The athlete's local date.
    pub date: String,
    /// The day of the month.
    pub day: u32,
    /// The runs of the day.
    pub activities: Vec<ActivitySummary>,
    /// The distance of the day, as the core renders it.
    pub distance: Option<RenderedValue>,
    /// The moving time of the day, as the core renders it.
    pub duration: Option<RenderedValue>,
}

/// The local month calendar.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Calendar {
    /// The year.
    pub year: i32,
    /// The month, from one.
    pub month: u32,
    /// The days of the month.
    pub days: Vec<CalendarDay>,
    /// When this answer was produced.
    pub generated_at: String,
}

// ------------------------------------------------------------------- load

/// The training load over the selected window.
///
/// The load is a **series** and a **scenario**, never a verdict: the
/// numbers, the assumption they were made under, and what a run of rest
/// days is expected to do with them. There is no band name, no risk and
/// nothing that tells the athlete what to do.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The athlete's local date.
    pub date: String,
    /// The window the numbers describe.
    pub window: LoadWindow,
    /// The load itself.
    ///
    /// `None` when the load has no value — and then `unavailable` says
    /// why. A missing zone model is never a zero and never a guessed
    /// maximum heart rate.
    pub load: Option<LoadSeries>,
    /// Why there is no load. `None` when there is one.
    pub unavailable: Option<String>,
    /// The recovery scenario.
    ///
    /// `None` when there is no load: there is nothing to project.
    pub scenario: Option<LoadScenario>,
}

/// The window the load's numbers describe.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadWindow {
    /// The first day of the window, in the athlete's own calendar.
    pub from: String,
    /// The first day of the window, as the api writes a date.
    pub from_label: String,
    /// The last day of the window, in the athlete's own calendar.
    pub to: String,
    /// The last day of the window, as the api writes a date.
    pub to_label: String,
    /// How many calendar weeks the window covers.
    pub weeks: i64,
    /// How many days of history the series was warmed with, before the
    /// window.
    ///
    /// The chronic series is weighted over forty-two days, so the series
    /// needs a warm-up: without it the numbers at the start of the window
    /// would be the numbers of a series that began there. The days are
    /// read and weighed, and they are not part of the window's series.
    pub warmup_days: i64,
}

/// The load of the days in the window.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadSeries {
    /// The days of the window the record covers, oldest first.
    pub days: Vec<LoadDay>,
    /// The state of the two series at each of those days, oldest first.
    pub states: Vec<LoadState>,
    /// The seven-day values at the last day of the series.
    ///
    /// `None` when the last seven days are not all there with a load: a
    /// week with a missing day has no impulse, no monotony and no strain,
    /// because absence is not zero.
    pub week: Option<LoadWeek>,
    /// The assumptions the numbers were made under.
    pub assumptions: LoadAssumption,
}

/// One day of the load.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadDay {
    /// The day, in the athlete's own calendar.
    pub date: String,
    /// The day, as the api writes a date.
    pub label: String,
    /// The day's load, as the core renders it.
    ///
    /// `None` when the day has no load: it is not covered by the record,
    /// or nothing on it could be weighed.
    pub load: Option<RenderedValue>,
    /// Why the day has no load. `Some` exactly when `load` is `None`.
    pub reason: Option<String>,
}

/// The state of the two series at a day.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadState {
    /// The day, in the athlete's own calendar.
    pub date: String,
    /// The day, as the api writes a date.
    pub label: String,
    /// The acute load: the seven-day weighted average.
    pub acute: RenderedValue,
    /// The chronic load: the forty-two-day weighted average.
    pub chronic: RenderedValue,
    /// The form: the chronic load minus the acute load.
    pub form: RenderedValue,
    /// The acute-to-chronic ratio.
    pub ratio: RenderedValue,
}

/// The seven-day values.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadWeek {
    /// The sum of the seven days' loads.
    pub impulse: RenderedValue,
    /// The monotony: the mean of the seven days over their standard
    /// deviation.
    pub monotony: RenderedValue,
    /// The strain: the seven-day sum times the monotony.
    pub strain: RenderedValue,
}

/// The assumptions the load's numbers rest on.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadAssumption {
    /// The Banister coefficient the model used.
    pub banister_k: f64,
    /// The maximum heart rate the zone model was built on.
    pub max_heart_rate_bpm: f64,
    /// The threshold heart rate: the zone-5 floor of the stored model.
    pub threshold_hr_bpm: f64,
    /// The day the zone model was read, in the athlete's own calendar.
    pub zone_model_read_on: String,
    /// The day the zone model was read, as the api writes a date.
    pub zone_model_read_on_label: String,
    /// The acute window, in days.
    pub acute_window_days: i64,
    /// The chronic window, in days.
    pub chronic_window_days: i64,
    /// How many days the scenario projects forward.
    pub forecast_days: i64,
    /// The assumptions as a sentence, in the core's own words.
    pub statement: String,
}

/// The recovery scenario.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadScenario {
    /// How many complete rest days the ratio is expected to need to be back
    /// inside its band. `None` when seven rest days do not bring it back.
    pub days_to_ratio_band: Option<u32>,
    /// How many days the form is expected to need to turn positive.
    /// `None` when seven rest days do not turn it positive.
    pub days_to_positive_form: Option<u32>,
    /// The band of the ratio, as a range: "0.8 to 1.3".
    pub band: String,
    /// The scenario as a sentence, in the core's own words.
    pub statement: String,
    /// The assumption the scenario rests on, in the core's own words.
    pub assumption: String,
    /// The projection itself, one entry per projected day.
    pub projection: Vec<LoadProjection>,
}

/// One projected day of the scenario.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoadProjection {
    /// The day of the projection: one is the day after the series ends.
    pub day: u32,
    /// The projected form.
    pub form: RenderedValue,
    /// The projected acute-to-chronic ratio.
    pub ratio: RenderedValue,
}

// ---------------------------------------------------------------- rolling

/// The rolling view: a window of history and what it holds.
///
/// The window is **the last *N* days ending on the day the answer is
/// about**, and the comparison period is **the same *N* days ending a day
/// earlier**. The comparison is a comparison: two numbers side by side and
/// the difference between them, never a verdict.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RollingInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The athlete's local calendar date the answer is about.
    pub date: String,
    /// The window the answer is about.
    pub window: RollingWindow,
    /// The period the window is compared against.
    pub comparison: RollingWindow,
    /// The distance of the window and of the period before it.
    pub distance: WindowComparison,
    /// The moving time of the window and of the period before it.
    pub duration: WindowComparison,
    /// The runs of the window and of the period before it.
    pub sessions: SessionComparison,
    /// The split of the training across the athlete's own zones.
    pub zones: ZoneSplit,
    /// The buckets of the distance breakdown: the runs, by how far they
    /// went.
    pub distances: Vec<DistanceBucketRow>,
    /// The athlete's shape over the window: seven facets, each with the
    /// way it moved against the window before it.
    pub profile: Vec<ProfileFacet>,
    /// What usually happened on each day of the week.
    pub weekday: Vec<WeekdayRow>,
    /// What usually happened in each time of day.
    pub daytime: Vec<DaytimeRow>,
    /// What each month of the history holds, newest first.
    pub months: Vec<PeriodRow>,
    /// What each year of the history holds, newest first.
    pub years: Vec<PeriodRow>,
}

/// A window of days.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RollingWindow {
    /// The first day of the window, in the athlete's own calendar.
    pub from: String,
    /// The same day, as the api writes a date: "18 Feb 2026".
    pub from_label: String,
    /// The last day of the window.
    pub to: String,
    /// The same day, as the api writes a date: "13 May 2026".
    pub to_label: String,
    /// How many days the window covers.
    pub days: i64,
}

/// A value in a window, in the window before it, and the difference.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WindowComparison {
    /// The value in the window.
    pub current: Option<RenderedValue>,
    /// The value in the window before it.
    pub previous: Option<RenderedValue>,
    /// The difference between the two, sign and all.
    pub difference: Option<RenderedValue>,
}

/// A count in a window, in the window before it, and the difference.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SessionComparison {
    /// The runs in the window.
    pub current: usize,
    /// The runs in the window before it.
    pub previous: usize,
    /// The difference between the two, as the core renders it: "+3", "−3".
    pub difference: Option<RenderedValue>,
}

/// The split of the training across the athlete's own zones.
///
/// The shares are facts: what share of the time in the zones was spent in
/// the low band, the moderate band and the high band. There is no target
/// range beside them and nothing that says whether the split is any good.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneSplit {
    /// The model the split was measured against, when there is one.
    pub model: Option<ZoneModelSource>,
    /// The shares, when there is a value.
    pub shares: Option<Vec<ZoneShareRow>>,
    /// The distribution: the time in each of the five zones and the share
    /// of the time in the zones it holds.
    pub distribution: Option<Vec<ZoneDistributionRow>>,
    /// Why there is no split, in the core's own words.
    pub unavailable: Option<String>,
}

/// One zone of the distribution.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneDistributionRow {
    /// The zone's label: "Z1".
    pub zone: String,
    /// The range the zone covers, as the core renders it: "106-121 bpm".
    pub range: Option<String>,
    /// The time in the zone, as the core renders it.
    pub time: RenderedValue,
    /// The zone's share of the time in the zones.
    pub share: Option<RenderedValue>,
}

/// One bucket of the distance breakdown.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DistanceBucketRow {
    /// The bucket's key: "0-5".
    pub key: String,
    /// The bucket's range, as the core renders it: "0-5 km".
    pub label: String,
    /// The runs that fell into the bucket.
    pub sessions: usize,
    /// The distance of those runs.
    pub distance: RenderedValue,
    /// The elevation gain of those runs.
    pub elevation: RenderedValue,
    /// The moving time of those runs.
    pub duration: RenderedValue,
}

/// One facet of the athlete's shape.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProfileFacet {
    /// The facet's key: "volume", "consistency" and so on.
    pub key: String,
    /// The facet's name, in the core's words: "Sport variety".
    pub label: String,
    /// The value in the window. Null when it cannot be computed.
    pub value: Option<RenderedValue>,
    /// The value in the window before it.
    pub previous: Option<RenderedValue>,
    /// Which way it moved: "up", "down" or "steady".
    pub movement: Option<String>,
}

/// The zone model a split was measured against.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneModelSource {
    /// The day the model was read, in the athlete's own calendar.
    pub read_on: String,
    /// The same day, as the api writes a date: "10 May 2026".
    pub read_on_label: String,
    /// The model's own source line, in the core's words.
    pub source: Option<String>,
}

/// One band's share of the time in the zones.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZoneShareRow {
    /// The band's key: "low", "moderate" or "high".
    pub key: String,
    /// The band's name, in the core's words: "Z1–Z2 low".
    pub label: String,
    /// The share of the time in the zones, as the core renders it.
    pub share: Option<RenderedValue>,
    /// The share in the window before it.
    pub previous_share: Option<RenderedValue>,
    /// Which way it moved: "up", "down" or "steady".
    pub trend: Option<String>,
}

/// What usually happened on one day of the week.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WeekdayRow {
    /// The day's name, in the core's words: "Monday".
    pub weekday: String,
    /// How many runs fell on it.
    pub sessions: usize,
    /// The distance of those runs.
    pub distance: RenderedValue,
    /// The moving time of those runs.
    pub duration: RenderedValue,
    /// The day's share of the moving time.
    pub share: Option<RenderedValue>,
}

/// What usually happened in one time of day.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DaytimeRow {
    /// The bucket's key: "morning", "afternoon", "evening" or "night".
    pub key: String,
    /// The bucket's name, in the core's words.
    pub label: String,
    /// The hour the bucket starts at, in the athlete's own local time.
    pub from_hour: u32,
    /// The hour the bucket ends at.
    pub to_hour: u32,
    /// How many runs fell into it.
    pub sessions: usize,
    /// The distance of those runs.
    pub distance: RenderedValue,
    /// The moving time of those runs.
    pub duration: RenderedValue,
}

/// What a month or a year of the history holds.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PeriodRow {
    /// The period's key: "2026-04" for a month, "2026" for a year.
    pub key: String,
    /// The period's name, as the api writes it: "April 2026" or "2026".
    pub label: String,
    /// How many runs fell into it.
    pub sessions: usize,
    /// The distance of those runs.
    pub distance: RenderedValue,
    /// The moving time of those runs.
    pub duration: RenderedValue,
    /// The elevation gain of those runs.
    pub elevation: RenderedValue,
}

// ---------------------------------------------------------------- streaks

/// The streaks of the athlete's history.
///
/// A streak is a count and the days it covers: nothing more. There is no
/// "you are consistent" here — the number is the number.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StreakInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The athlete's local calendar date the answer is about.
    pub date: String,
    /// The streaks of days.
    pub days: StreakDetail,
    /// The streaks of weeks.
    pub weeks: StreakDetail,
    /// The streaks of months.
    pub months: MonthStreakDetail,
}

/// The streaks of months: the current one and the longest one.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MonthStreakDetail {
    /// The streak that is still going. `None` when there is none: a streak
    /// of zero months is no streak.
    pub current: Option<MonthStreakValue>,
    /// The longest one in the history.
    pub longest: Option<MonthStreakValue>,
}

/// One streak of months: how long it is, and the months it covers.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MonthStreakValue {
    /// How many months the streak runs.
    pub count: u32,
    /// The month it started, as `YYYY-MM`.
    pub start: String,
    /// The same month, as the api writes a month: "November 2025".
    pub start_label: String,
    /// The month it ended, when it ended in the past.
    pub end: Option<String>,
    /// The same month, as the api writes a month.
    pub end_label: Option<String>,
}

/// The streaks of one kind: the current one and the longest one.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StreakDetail {
    /// The streak that is still going. `None` when there is none: a streak
    /// of zero days is no streak.
    pub current: Option<StreakValue>,
    /// The longest one in the history.
    pub longest: Option<StreakValue>,
}

/// One streak: how long it is, and the days it covers.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StreakValue {
    /// How many days (or weeks) the streak runs.
    pub count: u32,
    /// The day it started, in the athlete's own calendar.
    pub start: String,
    /// The same day, as the api writes a date: "2 May 2026".
    pub start_label: String,
    /// The day it ended, when it ended in the past.
    pub end: Option<String>,
    /// The same day, as the api writes a date.
    pub end_label: Option<String>,
}

// ------------------------------------------------------------------ heatmap

/// The heatmap: a grid over the stored positions.
///
/// **Computed on the device.** Nothing is fetched for it and nothing is
/// sent with it: the cells are made of the positions the athlete's own
/// runs carry, decoded from their stored FIT files on read.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HeatmapInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The athlete's local date.
    pub date: String,
    /// How many runs carried a position and were counted.
    pub runs: u32,
    /// Whether the store holds runs this answer did not read.
    ///
    /// The answer is bounded: it reads the newest runs only, and this says
    /// so rather than letting a partial map read as the whole history.
    pub more_runs: bool,
    /// The size of one cell on the ground, as the core renders it.
    ///
    /// `None` when there is no cell: there is nothing to size.
    pub cell_size: Option<RenderedValue>,
    /// The cells, north to south and west to east.
    pub cells: Vec<HeatmapCell>,
    /// The box the cells cover.
    pub extent: Option<HeatmapExtent>,
    /// Why there is no heatmap. `None` when there is one.
    pub unavailable: Option<String>,
}

/// One cell of the heatmap.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HeatmapCell {
    /// The cell's column in the grid.
    pub x: u32,
    /// The cell's row in the grid.
    pub y: u32,
    /// The cell's southern edge.
    pub south: f64,
    /// The cell's western edge.
    pub west: f64,
    /// The cell's northern edge.
    pub north: f64,
    /// The cell's eastern edge.
    pub east: f64,
    /// How many runs went through this cell.
    ///
    /// A count of runs, never a path: the cell does not know the order,
    /// the time or the way the run went through it.
    pub runs: u32,
}

/// The box the heatmap's cells cover.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct HeatmapExtent {
    /// The southern edge.
    pub south: f64,
    /// The western edge.
    pub west: f64,
    /// The northern edge.
    pub north: f64,
    /// The eastern edge.
    pub east: f64,
}

/// The pace page's answer.
///
/// The page answers one question — how the pace has behaved — in three
/// panels: the training week by week, the pace at a heart rate, and where
/// the time went by pace. The last one is the pace bands, and they are a
/// property of the window: the answer names the window they cover.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The athlete's local calendar date the answer is about.
    pub date: String,
    /// The window the answer describes.
    pub window: PaceWindow,
    /// The weeks of the window, oldest first.
    pub weeks: Vec<TrainingWeek>,
    /// The bands the window's time falls into, or `None` when the window
    /// holds no moving samples.
    pub bands: Option<PaceBands>,
    /// Why there are no bands, in the core's own words. `None` when there
    /// are bands.
    pub unavailable: Option<String>,
}

/// The window the pace answer describes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceWindow {
    /// The first day of the window, in the athlete's own calendar.
    pub from: String,
    /// The same day, as the api writes a date: "31 Aug 2026".
    pub from_label: String,
    /// The last day of the window.
    pub to: String,
    /// The same day, as the api writes a date: "20 Nov 2026".
    pub to_label: String,
    /// How many calendar weeks the window covers.
    pub weeks: i64,
}

/// One week of the window: what it covered, and the long run inside it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TrainingWeek {
    /// The Monday the week starts on.
    pub week_start: String,
    /// The week's distance.
    pub distance: RenderedValue,
    /// How many runs the week holds.
    pub sessions: usize,
    /// The week's longest run, or `None` when the week holds no run.
    pub longest: Option<PaceLongest>,
    /// The week's median pace — the pace of the week's runs, not the pace
    /// of one of them.
    pub pace: Option<RenderedValue>,
    /// The week's median heart rate, on the same week.
    ///
    /// The pace and the heart rate are **separate medians**: they are
    /// never a matched-effort comparison, and the app does not draw one.
    pub heart_rate: Option<RenderedValue>,
}

/// The longest run of a week.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceLongest {
    /// The run's distance.
    pub distance: RenderedValue,
    /// The run's pace.
    pub pace: Option<RenderedValue>,
}

/// The bands the window's moving time falls into.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceBands {
    /// How wide a band is, in seconds per kilometre.
    pub width_s_per_km: f64,
    /// The window's moving time, in seconds: the sum of the bands' times.
    pub total: RenderedValue,
    /// The distance covered in the window.
    pub distance: RenderedValue,
    /// The bands, fastest first.
    pub rows: Vec<PaceBandRow>,
}

/// One band of the distribution.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaceBandRow {
    /// The band's lower edge, in seconds per kilometre.
    pub from: f64,
    /// The band's upper edge, in seconds per kilometre.
    pub to: f64,
    /// The band's name, in the core's own words: "4:30–5:00 /km".
    pub label: String,
    /// The time held at the band's paces.
    pub time: RenderedValue,
    /// The distance covered at the band's paces.
    pub distance: RenderedValue,
    /// The band's share of the window's time.
    pub share: Option<RenderedValue>,
}

/// The year: the months, the calendar of the days, and what changed.
///
/// **The calendar colours days, never places.** The days carry the zone
/// mix of that day's runs and nothing else: no position, no place name and
/// nothing that says where the athlete ran. Where they ran is the
/// heatmap's surface, and it has its own screen and its own disclosure.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YearInsight {
    /// When this answer was produced.
    pub generated_at: String,
    /// The athlete's local calendar date the answer is about.
    pub date: String,
    /// The window the answer is about: the last twelve calendar months.
    pub window: YearWindow,
    /// The period the window is compared against: the same number of days,
    /// ending a day earlier.
    pub comparison: YearWindow,
    /// The months of the window, oldest first.
    pub months: Vec<YearMonth>,
    /// Every day of the window, oldest first.
    pub days: Vec<YearDay>,
    /// What changed, in the core's own sentences.
    ///
    /// The sentences are written by the core's catalogue and travel to the
    /// app already rendered: the app writes no sentence of its own.
    pub changed: Vec<String>,
}

/// A window of days, as the year names it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YearWindow {
    /// The first day of the window, in the athlete's own calendar.
    pub from: String,
    /// The same day, as the api writes a date: "1 Jun 2025".
    pub from_label: String,
    /// The last day of the window.
    pub to: String,
    /// The same day, as the api writes a date: "13 May 2026".
    pub to_label: String,
    /// How many days the window covers.
    pub days: i64,
}

/// One month of the year.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YearMonth {
    /// The month's key: "2026-05".
    pub key: String,
    /// The month's label, as the api writes it: "May 2026".
    pub label: String,
    /// How many runs fell in the month.
    pub sessions: usize,
    /// The month's distance. `None` when the month holds no run.
    pub distance: Option<RenderedValue>,
    /// The month's moving time. `None` when it holds no run.
    pub duration: Option<RenderedValue>,
    /// The month's elevation gain. `None` when it holds no run.
    pub elevation: Option<RenderedValue>,
    /// The longest run of the month. `None` when it holds no run.
    pub longest: Option<RenderedValue>,
    /// The training load's form at the month's end.
    pub form: Option<RenderedValue>,
    /// The weeks the month touches, oldest first.
    pub weeks: Vec<YearWeek>,
    /// The mix of the zones the month was trained in: each zone's share of
    /// the time in the zones.
    ///
    /// Empty when the month holds no time in the zones: that is no value,
    /// never a row of zeros.
    pub mix: Vec<YearZoneShare>,
    /// The share of the time in the zones that was in the high zones
    /// (Z4–Z5). `None` when the month holds no time in the zones.
    pub high: Option<RenderedValue>,
}

/// One calendar week inside a month.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YearWeek {
    /// The first day of the week — a Monday.
    pub start: String,
    /// The same day, as the api writes a date: "4 May 2026".
    pub start_label: String,
    /// The distance of the week's days that fall in the month. `None` when
    /// the week holds no run in the month.
    pub distance: Option<RenderedValue>,
    /// How many runs fell in the week's days that are in the month.
    pub sessions: usize,
}

/// One zone's share of the time in the zones.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YearZoneShare {
    /// The zone, as the core writes it: "Z1".
    pub zone: String,
    /// The zone's share of the time in the zones. `None` when there is no
    /// time in the zones at all.
    pub share: Option<RenderedValue>,
}

/// One day of the calendar.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YearDay {
    /// The day, in the athlete's own calendar.
    pub date: String,
    /// How many runs fell on the day.
    pub sessions: usize,
    /// The day's distance. `None` on a day with no run.
    pub distance: Option<RenderedValue>,
    /// The share of the day's time in the zones that was in the high zones
    /// (Z4–Z5) — what the calendar colours the day by.
    ///
    /// `None` when the day's runs carry no time in the zones: no value,
    /// never a share of zero.
    pub high: Option<RenderedValue>,
}

/// One run's update.
///
/// The update is a model's own reading of the run: it is **attributed**,
/// it is not the product's voice, and it is kept with the checks it passed
/// and the hash of the payload it was written from.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunUpdate {
    /// The writer's model, by name.
    pub writer: String,
    /// The reviewer's model, by name.
    pub reviewer: String,
    /// The SHA-256 of the payload the model was given.
    pub payload_hash: String,
    /// The model's own words.
    pub draft: String,
    /// The checks it passed, and what they found.
    pub checks: Vec<UpdateCheck>,
    /// The verdict: `ship`.
    pub verdict: String,
    /// When the update was made.
    pub created_at: String,
}

/// One check, and what it found.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateCheck {
    /// The check's name: `groundedness`, `direction` or `register`.
    pub check: String,
    /// What the check found, in the core's own words. Empty when it found
    /// nothing.
    pub findings: Vec<String>,
}

/// The stored update of a run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunUpdateResponse {
    /// The run's id.
    pub activity_id: i64,
    /// The update, when the run has one.
    pub update: Option<RunUpdate>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The payload of a run: the facts a model may use, and nothing else.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UpdatePayloadResponse {
    /// The run's id.
    pub activity_id: i64,
    /// The hash of the payload's rendered text. `None` when there is no
    /// payload.
    pub payload_hash: Option<String>,
    /// The payload, as the core builds it. `None` when there is no value.
    pub payload: Option<stride_core::update::Payload>,
    /// The payload as the model reads it: the core's own wording.
    pub text: String,
    /// Why there is no payload, in the core's own words.
    pub unavailable: Option<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// What the rig should send to a model next.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UpdateRequestResponse {
    /// The role the prompt is for: `writer`, `reviewer` or `repair`.
    pub role: String,
    /// The prompt, in the core's own words.
    pub prompt: String,
    /// The hash of the payload the model is given.
    pub payload_hash: String,
    /// The payload as the model reads it.
    pub payload: String,
    /// The checks' findings, when the draft had any.
    pub findings: Vec<String>,
    /// When this answer was produced.
    pub generated_at: String,
}

/// The update's own settings.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UpdateSettingsResponse {
    /// Is the feature on? It is off until the athlete turns it on.
    pub enabled: bool,
    /// May anything leave the machine? It may not, until the athlete says
    /// so.
    pub allow_remote: bool,
    /// The writer's model, by name.
    pub writer_model: String,
    /// The writer's family.
    pub writer_family: String,
    /// The base URL the writer is called at.
    pub writer_base_url: String,
    /// The reviewer's model, by name.
    pub reviewer_model: String,
    /// The reviewer's family.
    pub reviewer_family: String,
    /// The base URL the reviewer is called at.
    pub reviewer_base_url: String,
    /// The environment variable the key is read from. The key itself is
    /// never in the answer.
    pub key_env: String,
    /// When these settings were last written.
    pub updated_at: String,
    /// When this answer was produced.
    pub generated_at: String,
}
