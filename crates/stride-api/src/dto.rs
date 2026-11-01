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

use serde::Serialize;

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
    pub raw: Option<f64>,
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
    /// The one sentence about what the run meant.
    ///
    /// `null` until the core has a story engine: the sentence is the core's
    /// to write, and this crate does not write one of its own.
    pub story: Option<String>,
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
    /// The split's number, from one.
    pub index: i64,
    /// The split's distance, as the core renders it.
    pub distance: RenderedValue,
    /// The split's duration, as the core renders it.
    pub duration: RenderedValue,
    /// The average heart rate over the split, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
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

/// One run's fingerprint: the shape of the run, in numbers.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Fingerprint {
    /// The run's id.
    pub activity_id: i64,
    /// The athlete's local date.
    pub local_date: String,
    /// The distance, as the core renders it.
    pub distance: Option<RenderedValue>,
    /// The moving time, as the core renders it.
    pub duration: Option<RenderedValue>,
    /// The average heart rate, as the core renders it.
    pub heart_rate: Option<RenderedValue>,
    /// The pace, as the core renders it.
    pub pace: Option<RenderedValue>,
    /// The elevation gain, as the core renders it.
    pub elevation_gain: Option<RenderedValue>,
}

/// The fingerprints of the runs that have been synced.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Fingerprints {
    /// The fingerprints, newest first.
    pub items: Vec<Fingerprint>,
    /// When this answer was produced.
    pub generated_at: String,
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

/// One heart-rate zone.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HeartRateZone {
    /// The zone's number, from one.
    pub zone_no: u8,
    /// The zone's name.
    pub name: String,
    /// The lower edge, in beats per minute.
    pub low_bpm: Option<f64>,
    /// The upper edge, in beats per minute.
    pub high_bpm: Option<f64>,
    /// The time in the zone, in seconds.
    pub time_s: f64,
}

/// The heart-rate zones.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Zones {
    /// The zones.
    pub zones: Vec<HeartRateZone>,
    /// The time in the zones, in seconds.
    pub total_time_s: f64,
    /// When this answer was produced.
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanSession {
    /// The day of the session.
    pub date: String,
    /// The plan's own vocabulary: `easy`, `long`, `tempo`, `rest`.
    pub session_type: String,
    /// The session's title.
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
    /// The block's id.
    pub id: i64,
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
    /// The week's target, as the core renders it.
    pub target: Option<RenderedValue>,
    /// The week's sessions, in the order they fall.
    pub sessions: Vec<PlanSession>,
    /// How many of the week's sessions were run.
    pub adherence: Option<Adherence>,
}

/// How many of the planned sessions were run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Adherence {
    /// How many were run.
    pub done: usize,
    /// How many were planned.
    pub planned: usize,
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
    /// When this answer was produced.
    pub generated_at: String,
}

/// The adherence, over a window.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanAdherence {
    /// How many of the planned sessions were run.
    pub adherence: Adherence,
    /// The weeks of the window.
    pub weeks: Vec<AdherenceWeek>,
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
}

/// One planned session and what happened.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanResult {
    /// The session's id.
    pub session_id: i64,
    /// The day of the session.
    pub date: String,
    /// The plan's own vocabulary.
    pub session_type: String,
    /// `done`, `planned` or `not_done`.
    pub status: String,
    /// The distance that was planned, as the core renders it.
    pub target: Option<RenderedValue>,
    /// The distance that was run, as the core renders it.
    pub actual: Option<RenderedValue>,
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
    pub block_id: i64,
    /// How many sessions were written.
    pub sessions: usize,
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
}

/// What a pull brought in.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyncRun {
    /// How many runs the pull brought in.
    pub activities: usize,
    /// How many days of wellness the pull brought in.
    pub wellness_days: usize,
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

/// The answer to an import.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FitImport {
    /// Whether the file was imported.
    pub imported: bool,
    /// The run the file became.
    pub activity_id: i64,
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
