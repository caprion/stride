//! What the FIT file says, read on demand.
//!
//! **The FIT file is the source of truth.** The derived series are
//! recomputable from it, so the detail of a run is read from the file that
//! was stored with it, and a file that cannot be decoded leaves the run
//! without a series rather than inventing one.
//!
//! Every answer here is bounded: the file is bounded by the ingest crate's
//! ceiling, and the number of points in an answer is bounded by the
//! configuration.

use stride_ingest::{decode_fit, FitActivity, MAX_FIT_INPUT_BYTES};

use crate::dto::{ElevationPoint, SamplePoint, Split};
use crate::render;
use crate::state::Api;
use stride_core::units;

/// The decoded FIT of a run, when there is one and it can be decoded.
pub fn decoded(api: &Api, activity_id: i64) -> Option<FitActivity> {
    let blob = api.store().fit_blob(activity_id).ok()??;
    if blob.is_empty() || blob.len() > MAX_FIT_INPUT_BYTES {
        return None;
    }
    decode_fit(&blob, "activity.fit").ok()
}

/// The series of a run, read from its file.
pub fn samples_for(api: &Api, activity_id: i64) -> Vec<SamplePoint> {
    match decoded(api, activity_id) {
        Some(decoded) => samples(&decoded, api.config().max_samples_in_response),
        None => Vec::new(),
    }
}

/// The splits of a run.
pub fn splits(activity: &FitActivity) -> Vec<Split> {
    activity
        .laps
        .iter()
        .map(|lap| Split {
            index: lap.lap_index,
            distance: render::distance_of(lap.distance_m),
            duration: render::duration_of(lap.duration_s),
            heart_rate: render::heart_rate(lap.avg_hr.map(|rate| rate as f64)),
        })
        .collect()
}

/// The elevation profile of a run.
///
/// The points are the ones the file carries: a run with no altitude has no
/// profile, and the answer says so with an empty list rather than with a
/// flat line.
pub fn elevation(activity: &FitActivity, cap: usize) -> Vec<ElevationPoint> {
    let mut points: Vec<ElevationPoint> = activity
        .samples
        .iter()
        .filter_map(|sample| {
            let distance = sample.distance_m?;
            let altitude = sample.altitude_m?;
            Some(ElevationPoint {
                distance: render::distance_of(distance),
                elevation: render::elevation_of(altitude),
            })
        })
        .collect();
    thin(&mut points, cap);
    points
}

/// The series of a run.
pub fn samples(activity: &FitActivity, cap: usize) -> Vec<SamplePoint> {
    let mut points: Vec<SamplePoint> = activity
        .samples
        .iter()
        .map(|sample| SamplePoint {
            offset_s: units::round_time_s(sample.elapsed_s),
            pace: render::pace(sample.speed_ms.and_then(|speed| {
                if speed <= 0.0 {
                    return None;
                }
                Some(units::round_pace_s_per_km(1000.0 / speed))
            })),
            heart_rate: render::heart_rate(sample.heart_rate_bpm),
        })
        .collect();
    thin(&mut points, cap);
    points
}

/// The route of a run: the first and the last points the file carries.
pub fn geo_points(activity: &FitActivity) -> Vec<[f64; 2]> {
    let mut points: Vec<[f64; 2]> = Vec::new();
    for point in activity
        .geo_points
        .first
        .iter()
        .chain(activity.geo_points.last.iter())
    {
        let latitude = units::round_gps_degrees(point[0]);
        let longitude = units::round_gps_degrees(point[1]);
        points.push([latitude, longitude]);
    }
    points
}

/// The fastest time to cover a distance, in seconds.
///
/// The window is walked over the file's own cumulative distance: for every
/// point, the tightest window that still covers the distance is timed. The
/// fastest of those windows is the record.
pub fn fastest_split(activity: &FitActivity, target_m: f64) -> Option<f64> {
    if target_m <= 0.0 {
        return None;
    }
    let mut best: Option<f64> = None;
    let mut start = 0usize;
    for end in 0..activity.samples.len() {
        let end_distance = match activity.samples[end].distance_m {
            Some(distance) => distance,
            None => continue,
        };
        while start < end {
            let start_distance = match activity.samples[start].distance_m {
                Some(distance) => distance,
                None => break,
            };
            if end_distance - start_distance < target_m {
                break;
            }
            let elapsed = activity.samples[end].elapsed_s - activity.samples[start].elapsed_s;
            if elapsed > 0.0 {
                best = Some(match best {
                    Some(current) => current.min(elapsed),
                    None => elapsed,
                });
            }
            start += 1;
        }
    }
    best
}

/// Keep at most `cap` points, spread over the whole run.
fn thin<T: Clone>(points: &mut Vec<T>, cap: usize) {
    if cap == 0 || points.len() <= cap {
        return;
    }
    let step = points.len().div_ceil(cap);
    let mut kept: Vec<T> = Vec::with_capacity(cap);
    let mut index = 0usize;
    while index < points.len() {
        kept.push(points[index].clone());
        index += step;
    }
    *points = kept;
}
