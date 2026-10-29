//! The sync: pulling the athlete's runs and wellness out of Garmin.
//!
//! The reference is `backend/app/services/personal_sync.py`. This is the
//! ingest layer: it fetches the activities from Garmin's Activity API, takes
//! the canonical summary, downloads the original FIT file where there is one,
//! and writes the result. Then it pulls a day of wellness at a time — sleep,
//! HRV, resting heart rate, stress, steps and the body battery — into the
//! physiology rows.
//!
//! **Two rules come straight from the reference and are reproduced here:**
//!
//! * the API's numbers win for distance, pace, heart rate and cadence — the
//!   FIT file stores running cadence in *cycles*, which is half the
//!   steps-per-minute the watch displays; the FIT wins for what the summary
//!   does not carry: the splits, the track and the per-second series;
//! * the API's local start time is authoritative. The FIT carries UTC, which
//!   would put a late-evening run on the wrong calendar day.
//!
//! The mappings are checked against the reference's own output: the wellness
//! fields and the merged activity are in `tests/fixtures/sync.json`,
//! recorded from the Python implementation.

use serde_json::{json, Value};

use crate::backend::{ActivityRow, Backend, LapRow, PhysiologyRow, StreamRow};
use crate::error::BackendError as Error;

/// The Garmin Personal API client.
type Client = stride_garmin::GarminClient;

/// What the sync needs from Garmin.
///
/// The client is behind a trait so that the pull can be *measured*: the
/// property that matters is how many calls a sync makes, and only a client
/// that counts them can prove that a second sync makes none.
pub trait GarminSource {
    /// The activities in a window.
    fn activities(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<stride_garmin::ActivitySummary>, stride_garmin::GarminError>;

    /// One activity's detail.
    fn activity_detail_canonical(
        &self,
        id: &str,
    ) -> Result<stride_garmin::ActivityDetail, stride_garmin::GarminError>;

    /// The original FIT file.
    fn activity_fit(&self, id: &str) -> Result<Vec<u8>, stride_garmin::GarminError>;

    /// A day of daily metrics.
    fn daily_summary(&self, day: &str) -> Result<Value, stride_garmin::GarminError>;

    /// A night of sleep.
    fn sleep(&self, day: &str) -> Result<Value, stride_garmin::GarminError>;

    /// A morning's heart-rate variability.
    fn hrv(&self, day: &str) -> Result<Value, stride_garmin::GarminError>;

    /// A day of heart rate.
    fn daily_heart_rate(&self, day: &str) -> Result<Value, stride_garmin::GarminError>;
}

impl GarminSource for Client {
    fn activities(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<stride_garmin::ActivitySummary>, stride_garmin::GarminError> {
        Client::activities(self, from, to)
    }

    fn activity_detail_canonical(
        &self,
        id: &str,
    ) -> Result<stride_garmin::ActivityDetail, stride_garmin::GarminError> {
        Client::activity_detail_canonical(self, id)
    }

    fn activity_fit(&self, id: &str) -> Result<Vec<u8>, stride_garmin::GarminError> {
        Client::activity_fit(self, id)
    }

    fn daily_summary(&self, day: &str) -> Result<Value, stride_garmin::GarminError> {
        Client::daily_summary(self, day)
    }

    fn sleep(&self, day: &str) -> Result<Value, stride_garmin::GarminError> {
        Client::sleep(self, day)
    }

    fn hrv(&self, day: &str) -> Result<Value, stride_garmin::GarminError> {
        Client::hrv(self, day)
    }

    fn daily_heart_rate(&self, day: &str) -> Result<Value, stride_garmin::GarminError> {
        Client::daily_heart_rate(self, day)
    }
}

/// The most days a sync may look back.
const MAX_ACTIVITY_DAYS: i64 = 3650;

/// The most activities a sync may pull.
const MAX_ACTIVITIES: usize = 300;

/// What a sync did.
#[derive(Debug, Clone, Default)]
pub struct SyncOutcome {
    /// How many days of health data the pull covers.
    pub days: usize,
    pub activities_found: usize,
    pub activities_new: usize,
    pub activities_updated: usize,
    pub errors: Vec<String>,
    /// How many detail calls the pull made.
    pub detail_calls: usize,
    /// How many FIT files the pull downloaded.
    pub fit_downloads: usize,
    /// How many days of wellness the pull fetched.
    pub health_days_fetched: usize,
    /// How many days it skipped because they were already here.
    pub health_days_skipped: usize,
}

impl SyncOutcome {
    /// The answer the API gives.
    pub fn to_value(&self) -> Value {
        json!({
            "started": true,
            "days": self.days,
            "activities_found": self.activities_found,
            "activities_new": self.activities_new,
            "activities_updated": self.activities_updated,
            "errors": self.errors,
        })
    }
}

/// A number from a Garmin payload, or nothing.
///
/// The reference's `safe_float`: a value that is not a number is absent, and
/// zero is a number.
pub fn safe_float(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// The canonical activity, as Garmin's detail payload describes it.
///
/// The mapping itself is `stride-garmin`'s (`activity_detail_canonical`); this
/// turns it into the row the store holds.
pub fn canonical_activity(canonical: &stride_garmin::ActivityDetail) -> ActivityRow {
    ActivityRow {
        id: 0,
        athlete_id: None,
        source: "garmin_personal".to_string(),
        external_id: canonical.external_id.clone(),
        name: canonical.name.clone(),
        sport_type: canonical.sport_type.clone(),
        description: None,
        start_time: canonical.start_time.clone(),
        local_date: canonical.local_date.clone().unwrap_or_default(),
        day_of_week: canonical.day_of_week.clone(),
        distance_m: canonical.distance_m,
        moving_time_s: canonical.moving_time_s,
        elapsed_time_s: canonical.elapsed_time_s,
        elevation_gain_m: canonical.elevation_gain_m,
        avg_hr: canonical.avg_hr,
        max_hr: canonical.max_hr,
        calories: canonical.calories,
        avg_cadence: canonical.avg_cadence,
        max_cadence: canonical.max_cadence,
        avg_speed_ms: canonical.avg_speed_ms,
        max_speed_ms: canonical.max_speed_ms,
        running_time_s: None,
        walking_time_s: None,
        stopped_time_s: None,
        running_distance_m: None,
        perceived_exertion: None,
        device: canonical.device.clone(),
        gear: None,
        is_trail: false,
        is_commute: false,
        geo_points: None,
        lap_count: 0,
        has_geo: None,
    }
}

/// The FIT file's splits, as the store holds them.
pub fn fit_laps(parsed: &stride_fit::ParsedActivity) -> Vec<LapRow> {
    parsed
        .laps
        .iter()
        .map(|lap| LapRow {
            activity_id: 0,
            lap_index: lap.lap_index,
            distance_m: lap.distance_m,
            duration_s: lap.duration_s,
            avg_speed_ms: lap.avg_speed_ms,
            max_speed_ms: lap.max_speed_ms,
            avg_hr: lap.avg_hr,
            max_hr: lap.max_hr,
            avg_cadence: lap.avg_cadence,
            max_cadence: lap.max_cadence,
            stride_length_m: lap.stride_length_m,
            elevation_gain_m: lap.elevation_gain_m,
            elevation_loss_m: lap.elevation_loss_m,
            start_elevation_m: lap.start_elevation_m,
            end_elevation_m: lap.end_elevation_m,
            running_distance_m: lap.running_distance_m,
            elapsed_s: Some(lap.elapsed_s),
        })
        .collect()
}

/// The FIT file's series, as the store holds them.
pub fn fit_streams(parsed: &stride_fit::ParsedActivity) -> Vec<StreamRow> {
    parsed
        .streams
        .iter()
        .map(|(kind, stream)| StreamRow {
            activity_id: 0,
            kind: kind.clone(),
            seconds: stream.seconds.iter().map(|value| *value as f64).collect(),
            values: stream.values.iter().map(|value| Some(*value)).collect(),
            label: None,
            unit: Some(stream.unit.clone()),
        })
        .collect()
}

/// Overlay the FIT file's splits, series and track onto the API's summary.
///
/// The reference: `personal_sync._merge_fit`.
pub fn merge_fit(
    activity: &mut ActivityRow,
    laps: &[LapRow],
    streams: &[StreamRow],
    parsed: &stride_fit::ParsedActivity,
) {
    // The run / walk / stand split comes from the FIT record stream.
    activity.running_time_s = parsed.running_time_s;
    activity.walking_time_s = parsed.walking_time_s;
    activity.stopped_time_s = parsed.stopped_time_s;
    activity.running_distance_m = parsed.running_distance_m;
    if !parsed
        .geo_points
        .as_ref()
        .is_some_and(|points| points.is_empty())
    {
        activity.geo_points = parsed.geo_points.clone();
    }
    if let Some(device) = &parsed.device {
        activity.device = Some(device.clone());
    }
    // Elevation: the API's summary is the accumulated gain for the activity.
    if activity.elevation_gain_m == 0.0 && parsed.elevation_gain_m > 0.0 {
        activity.elevation_gain_m = parsed.elevation_gain_m;
    }
    activity.lap_count = laps.len() as i64;
    let _ = streams;
}

/// Fill a day from `usersummary/daily`.
///
/// The reference: `personal_sync._apply_daily_summary`.
pub fn apply_daily_summary(row: &mut PhysiologyRow, payload: &Value) {
    let summary = match payload.get("summaryDTO") {
        Some(summary) => summary,
        None => payload,
    };
    let number = |key: &str| safe_float(summary.get(key));
    if let Some(steps) = number("totalSteps") {
        row.steps = Some(steps);
    }
    if let Some(resting) = number("restingHeartRate") {
        row.resting_hr = Some(resting);
    }
    if let Some(peak) = number("maxHeartRate") {
        row.max_hr = Some(peak);
    }
    if let Some(stress) = number("averageStressLevel") {
        row.stress_avg = Some(stress);
    }
    if let Some(stress) = number("maxStressLevel") {
        row.stress_max = Some(stress);
    }
    if let Some(battery) = number("bodyBatteryLowestValue") {
        row.body_battery_min = Some(battery);
    }
    if let Some(battery) = number("bodyBatteryHighestValue") {
        row.body_battery_max = Some(battery);
    }
    if let Some(battery) = number("bodyBatteryMostRecentValue") {
        row.body_battery_avg = Some(battery);
    }
    let vo2max = number("vo2Max").or_else(|| number("vo2MaxValue"));
    if let Some(vo2max) = vo2max {
        row.vo2max = Some(vo2max);
    }
}

/// Fill a day from the sleep endpoint.
///
/// The reference: `personal_sync._apply_sleep`.
pub fn apply_sleep(row: &mut PhysiologyRow, payload: &Value) {
    let set = |value: Option<f64>, target: &mut Option<f64>| {
        if let Some(value) = value {
            *target = Some(value);
        }
    };
    set(
        safe_float(payload.get("sleepTimeSeconds")),
        &mut row.sleep_s,
    );
    set(
        safe_float(payload.get("deepSleepSeconds")),
        &mut row.deep_sleep_s,
    );
    set(safe_float(payload.get("remSleepSeconds")), &mut row.rem_s);
    set(
        safe_float(payload.get("lightSleepSeconds")),
        &mut row.light_sleep_s,
    );
    set(
        safe_float(payload.get("awakeSleepSeconds")),
        &mut row.awake_s,
    );
}

/// Fill a day from the HRV endpoint.
///
/// The reference: `personal_sync._apply_hrv`.
pub fn apply_hrv(row: &mut PhysiologyRow, payload: &Value) {
    let summary = match payload.get("hrvSummary") {
        Some(summary) => summary,
        None => payload,
    };
    let value = safe_float(summary.get("lastNightAvg"))
        .or_else(|| safe_float(summary.get("weeklyAvg")))
        .or_else(|| safe_float(summary.get("value")));
    if let Some(value) = value {
        row.hrv_ms = Some(value);
    }
    let status = summary
        .get("status")
        .or_else(|| summary.get("feedbackPhrase"))
        .and_then(Value::as_str);
    if let Some(status) = status {
        row.hrv_status = Some(status.to_string());
    }
}

/// Fill a day from the daily heart rate.
///
/// The reference: `personal_sync._apply_heart_rate`.
pub fn apply_heart_rate(row: &mut PhysiologyRow, payload: &Value) {
    let summary = match payload.get("heartRateSummary") {
        Some(summary) => summary,
        None => payload,
    };
    let resting = safe_float(summary.get("restingHeartRate"))
        .or_else(|| safe_float(summary.get("restingHeartRateValue")))
        .or_else(|| safe_float(summary.get("lowestHeartRate")));
    if let Some(resting) = resting {
        row.resting_hr = Some(resting);
    }
    let peak = safe_float(summary.get("maxHeartRate"))
        .or_else(|| safe_float(summary.get("highestHeartRate")));
    if let Some(peak) = peak {
        row.max_hr = Some(peak);
    }
}

/// Mirror a day into the daily-metric table, so `/health/daily` stays useful.
///
/// The reference: `personal_sync._mirror_daily_metric`.
pub fn mirror_daily_metric(row: &PhysiologyRow) -> crate::backend::DailyMetricRow {
    crate::backend::DailyMetricRow {
        date: row.date.clone(),
        resting_hr: row.resting_hr,
        max_hr: row.max_hr,
        sleep_s: row.sleep_s,
        deep_sleep_s: row.deep_sleep_s,
        hrv_ms: row.hrv_ms,
        stress: row.stress_avg,
        steps: row.steps,
        body_battery: row.body_battery_avg,
        readiness: None,
        distance_m: 0.0,
    }
}

/// Pull the runs and the wellness.
///
/// `days` is how far back to look; `None` pulls everything the account
/// holds. The reference: `personal_sync.run`.
pub fn run(
    backend: &dyn Backend,
    key: &str,
    days: Option<i64>,
    health_days: Option<i64>,
    today: chrono::NaiveDate,
) -> Result<SyncOutcome, Error> {
    let client = Client::new(key.to_string());
    run_with(backend, &client, days, health_days, today)
}

/// The pull, against any client.
///
/// The client is a parameter so that the pull can be measured: what matters
/// is not that it works but **how many calls it makes**. A pull that fetches
/// every activity's detail and downloads every FIT file again, on every
/// sync, is 16 MB of mobile data a day for nothing.
pub fn run_with<S: GarminSource + ?Sized>(
    backend: &dyn Backend,
    source: &S,
    days: Option<i64>,
    health_days: Option<i64>,
    today: chrono::NaiveDate,
) -> Result<SyncOutcome, Error> {
    let mut outcome = SyncOutcome::default();

    // The runs.
    let window = days
        .unwrap_or(MAX_ACTIVITY_DAYS)
        .clamp(1, MAX_ACTIVITY_DAYS);
    let end = today
        .and_hms_opt(0, 0, 0)
        .unwrap_or_else(|| today.and_hms_opt(12, 0, 0).unwrap_or_default());
    let start = end - chrono::Duration::days(window - 1);
    let activities = source
        .activities(start.and_utc(), end.and_utc())
        .map_err(|err| Error::Domain(format!("Garmin would not answer: {err}")))?;
    outcome.activities_found = activities.len();
    for summary in activities.iter().take(MAX_ACTIVITIES) {
        let id = summary.external_id.as_str();
        if id.is_empty() {
            continue;
        }
        let mut existing = backend
            .activity_by_external("garmin_personal", id)
            .unwrap_or(None);
        let was_there = existing.is_some();

        // Has this activity already been rebuilt from its original FIT file?
        //
        // The FIT file is the source of truth. If this activity has already
        // been rebuilt from its FIT, its track and streams cannot have
        // changed, so the download is skipped and nothing is fetched for it
        // at all. An activity that only has the API's downsampled chart
        // series has *not* been imported — it gets its FIT downloaded.
        let imported = has_fit_records(existing.as_ref(), backend);

        let mut activity = if imported {
            // The stored row is the truth: it already holds the track, the
            // splits and the per-second series. Without this the canonical
            // `None` would overwrite the stored track and the streams.
            match existing.take() {
                Some(row) => row,
                None => continue,
            }
        } else {
            let canonical = match source.activity_detail_canonical(id) {
                Ok(canonical) => canonical,
                Err(err) => {
                    outcome.errors.push(format!("{id}: {err}"));
                    continue;
                }
            };
            outcome.detail_calls += 1;
            canonical_activity(&canonical)
        };

        let mut laps = Vec::new();
        let mut streams = Vec::new();
        // The FIT file, where the account has one — and only where the
        // activity has not been rebuilt from it already.
        if !imported {
            // The reference logs the failure and carries on: an activity
            // without its FIT is still an activity.
            if let Ok(bytes) = source.activity_fit(id) {
                outcome.fit_downloads += 1;
                if let Ok(parsed) = stride_fit::parse_fit(&bytes, &format!("{id}.fit")) {
                    laps = fit_laps(&parsed);
                    streams = fit_streams(&parsed);
                    merge_fit(&mut activity, &laps, &streams, &parsed);
                }
            }
        }
        if was_there {
            outcome.activities_updated += 1;
        } else {
            outcome.activities_new += 1;
        }
        if let Err(err) = backend.upsert_synced_activity(&activity, &laps, &streams) {
            outcome.errors.push(format!("{id}: {err}"));
        }
    }

    // The wellness.
    let wellness_days = health_days.unwrap_or(30).clamp(1, 365);
    let wellness_end = today;
    let wellness_start = wellness_end - chrono::Duration::days(wellness_days - 1);
    let mut day = wellness_start;
    while day <= wellness_end {
        let key = day.format("%Y-%m-%d").to_string();
        let existing = backend
            .physiology(&key, &key)
            .unwrap_or_default()
            .into_iter()
            .next();
        if existing.as_ref().is_some_and(has_wellness) {
            // The day is already here. Pulling it again costs four calls and
            // changes nothing, so it is skipped — and the cost is reported,
            // because a floor of zero calls a day is the whole point.
            outcome.health_days_skipped += 1;
            outcome.days += 1;
            day += chrono::Duration::days(1);
            continue;
        }
        let mut row = existing.unwrap_or_else(|| crate::backend::PhysiologyRow {
            date: key.clone(),
            source: Some("garmin_personal".to_string()),
            ..crate::backend::PhysiologyRow::default()
        });
        let mut failed = false;
        if let Ok(payload) = source.daily_summary(&key) {
            apply_daily_summary(&mut row, &payload);
        }
        if let Ok(payload) = source.sleep(&key) {
            apply_sleep(&mut row, &payload);
        }
        if let Ok(payload) = source.hrv(&key) {
            apply_hrv(&mut row, &payload);
        }
        if let Ok(payload) = source.daily_heart_rate(&key) {
            apply_heart_rate(&mut row, &payload);
        }
        if let Err(err) = backend.upsert_physiology(&row) {
            outcome.errors.push(format!("{key}: {err}"));
            failed = true;
        }
        if !failed {
            let _ = backend.upsert_daily_metric(&mirror_daily_metric(&row));
            outcome.health_days_fetched += 1;
            outcome.days += 1;
        }
        day += chrono::Duration::days(1);
    }
    // The cost of the pull, in the reference's own words. This is the line
    // that says whether a sync was worth making.
    eprintln!(
        "Personal API pull: {} activities, {} detail calls, {} FIT downloads",
        outcome.activities_found, outcome.detail_calls, outcome.fit_downloads
    );
    eprintln!(
        "wellness: {} days fetched, {} days skipped",
        outcome.health_days_fetched, outcome.health_days_skipped
    );
    Ok(outcome)
}

/// Has this activity already been rebuilt from its original FIT file?
///
/// The Activity API returns a *chart* series — a few hundred points that
/// Garmin downsamples for the web page. The FIT file carries the original
/// per-second record stream, which is what the segmentation, the fingerprint
/// and the drift metrics are computed from.
///
/// The two are easy to confuse and expensive to get wrong: an activity with a
/// chart series looks imported, so the FIT is never downloaded, and the
/// fingerprint silently never appears for it. So this checks the *resolution*
/// of the series, not merely its presence.
///
/// Activities imported with the marker carry it in their raw payload; older
/// ones are recognised by their per-second resolution.
///
/// The reference: `personal_sync._has_fit_records`.
pub fn has_fit_records(existing: Option<&ActivityRow>, backend: &dyn Backend) -> bool {
    let Some(existing) = existing else {
        return false;
    };
    // The explicit marker.
    if let Ok(Some(raw)) = backend.activity_raw(existing.id) {
        if raw
            .get("fit_imported")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return true;
        }
    }
    // The resolution of the series: per-second, not the chart's samples.
    let Ok(streams) = backend.activity_streams(existing.id) else {
        return false;
    };
    let longest = streams
        .iter()
        .map(|stream| stream.values.len())
        .max()
        .unwrap_or(0) as f64;
    let elapsed = existing.elapsed_time_s;
    if elapsed <= 0.0 {
        return false;
    }
    longest >= 0.8 * elapsed
}

/// Does this day already hold wellness data?
///
/// A day with nothing in it is a day that was never pulled; a day with sleep
/// or a heart rate in it is a day that is already here.
fn has_wellness(row: &crate::backend::PhysiologyRow) -> bool {
    row.sleep_s.is_some() || row.hrv_ms.is_some() || row.resting_hr.is_some() || row.steps.is_some()
}
