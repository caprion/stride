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
//!
//! ## The one place a file is decoded
//!
//! [`decoded`] is the **single choke point** for every decode in the
//! router — the pace page, the rolling view, the year, the heatmap, the
//! run detail, the run film and the ledger all read their runs' files
//! through it. A twelve-week window is about sixty runs, and without a
//! cache every one of those files was decoded again on every read.
//!
//! The decode is held in the router's **bounded cache** (the `Api`'s
//! `DecodeCache`): the run's id and the file's own SHA-256 are the key,
//! the cache holds a fixed number of decoded runs, and the oldest is
//! thrown away when it is full. **Only the decode is cached** — every
//! derived series is recomputed on read, from the decoded file.

use std::sync::Arc;

use stride_core::pace::PacePoint;
use stride_core::zones::{ZoneFloors, ZoneSample};
use stride_ingest::{decode_fit, FitActivity, MAX_FIT_INPUT_BYTES};

use crate::dto::{FilmGround, FilmSample, FilmSplit, SamplePoint, Split};
use crate::render;
use crate::state::Api;
use stride_core::units;

/// The decoded FIT of a run, when there is one and it can be decoded.
///
/// **The decode is cached**, and the cache is bounded: the run's id and
/// the file's own digest are the key, and a file that was re-imported
/// has a new digest, so the old decode is never served for it.
///
/// The lock is not held across the decode: the file is read and decoded
/// outside the cache's lock, and the result is put in afterwards.
pub fn decoded(api: &Api, activity_id: i64) -> Option<Arc<FitActivity>> {
    // The file's digest: the key the cache is checked against. It is a
    // small read, and it is what makes a re-imported file invalidate its
    // own entry.
    let sha256 = api.store().fit_sha256(activity_id).ok()??;
    if let Some(activity) = api.decoded_cache().get(activity_id, &sha256) {
        return Some(activity);
    }
    let activity = decode(api, activity_id)?;
    api.decoded_cache()
        .put(activity_id, &sha256, Arc::clone(&activity));
    Some(activity)
}

/// The decoded FIT of a run, decoded **without being kept in the cache**.
///
/// **The cache is bounded**, and a window may hold more runs than the cache
/// holds: the runs it cannot keep are decoded here, used, and decoded again
/// on the next read. The answer is complete either way — the cache is a
/// cache, and it never decides what an answer covers.
///
/// A run the cache *does* hold is still served from it: a hit is free, and
/// keeping the decode costs nothing.
pub fn decoded_uncached(api: &Api, activity_id: i64) -> Option<Arc<FitActivity>> {
    let sha256 = api.store().fit_sha256(activity_id).ok()??;
    if let Some(activity) = api.decoded_cache().get(activity_id, &sha256) {
        return Some(activity);
    }
    decode(api, activity_id)
}

/// The file of a run, decoded.
fn decode(api: &Api, activity_id: i64) -> Option<Arc<FitActivity>> {
    let blob = api.store().fit_blob(activity_id).ok()??;
    if blob.is_empty() || blob.len() > MAX_FIT_INPUT_BYTES {
        return None;
    }
    Some(Arc::new(decode_fit(&blob, "activity.fit").ok()?))
}

/// The series of a run, read from its file.
pub fn samples_for(api: &Api, activity_id: i64) -> Vec<SamplePoint> {
    match decoded(api, activity_id) {
        Some(decoded) => samples(&decoded, api.config().max_samples_in_response),
        None => Vec::new(),
    }
}

/// The heart-rate samples of a run, for the time-in-zone computation.
///
/// **The stopped samples stay in the list.** A sample owns the span until
/// the next one, so a stretch the athlete stood through is owned by the
/// sample before it. Dropping the stopped samples does not drop the time:
/// it merges the stopped span into the neighbouring running one, and the
/// run then claims more time in a zone than it spent moving. The flag
/// travels with each sample so that the core can keep the stopped time
/// apart from the zones.
///
/// One implementation, so the per-run bands and the window's split cannot
/// disagree about which samples count.
pub fn zone_samples(activity: &FitActivity) -> Vec<ZoneSample> {
    activity
        .samples
        .iter()
        .map(|sample| {
            if sample.kind == stride_ingest::SegmentKind::Stand {
                ZoneSample::stopped(sample.elapsed_s, sample.heart_rate_bpm)
            } else {
                ZoneSample::moving(sample.elapsed_s, sample.heart_rate_bpm)
            }
        })
        .collect()
}

/// The samples of a run, for the pace bands.
///
/// **The bands are about the pace the athlete ran at.** The samples of a
/// stretch that was walked or stood through stay in the list — a point
/// owns the span until the next one, and dropping those points would
/// merge their spans into the neighbouring running one — and they are
/// marked, so the core can leave their time out of the bands. A sample
/// that carries no distance carries no pace either, and is left out.
pub fn pace_points(activity: &FitActivity) -> Vec<PacePoint> {
    activity
        .samples
        .iter()
        .filter_map(|sample| {
            let distance_m = sample.distance_m?;
            let elapsed_s = sample.elapsed_s;
            Some(if sample.kind == stride_ingest::SegmentKind::Run {
                PacePoint::new(elapsed_s, distance_m)
            } else {
                PacePoint::stopped(elapsed_s, distance_m)
            })
        })
        .collect()
}

/// The heart-rate samples of a run, decoded on read.
///
/// The stopped samples stay in the list: a sample owns the span until the
/// next one, so a stretch the athlete stood through is owned by the sample
/// before it. The flag travels with each sample so that the core can keep
/// the stopped time apart from the zones.
///
/// A run with no file has no samples: that is no value, never a zero.
pub fn zone_samples_of(api: &Api, activity_id: i64) -> Vec<ZoneSample> {
    match decoded(api, activity_id) {
        Some(decoded) => zone_samples(&decoded),
        None => Vec::new(),
    }
}

/// The heart-rate samples of a run, decoded without being kept in the
/// cache.
///
/// **The cache is bounded**, and a window may hold more runs than it holds:
/// the runs it cannot keep are decoded on the read and decoded again on the
/// next one. The answer still covers the whole window.
pub fn zone_samples_uncached(api: &Api, activity_id: i64) -> Vec<ZoneSample> {
    match decoded_uncached(api, activity_id) {
        Some(decoded) => zone_samples(&decoded),
        None => Vec::new(),
    }
}

/// The splits of a run.
pub fn splits(activity: &FitActivity) -> Vec<Split> {
    activity
        .laps
        .iter()
        .map(|lap| Split {
            lap_index: lap.lap_index,
            distance: render::distance_of(lap.distance_m),
            duration: render::duration_of(lap.duration_s),
            heart_rate: render::heart_rate(lap.avg_hr.map(|rate| rate as f64)),
            pace: render::pace(super::pace_of(lap.distance_m, lap.duration_s)),
        })
        .collect()
}

/// The series of a run.
pub fn samples(activity: &FitActivity, cap: usize) -> Vec<SamplePoint> {
    let mut points: Vec<SamplePoint> = activity
        .samples
        .iter()
        .map(|sample| SamplePoint {
            offset_s: units::round_time_s(sample.elapsed_s),
            distance_m: sample.distance_m.map(render::distance_of),
            pace: render::pace(pace_of_sample(sample.speed_ms)),
            heart_rate: render::heart_rate(sample.heart_rate_bpm),
            altitude_m: sample.altitude_m.map(render::elevation_of),
            kind: Some(sample.kind.as_str()),
        })
        .collect();
    thin(&mut points, cap);
    points
}

/// The pace of a sample, in seconds per kilometre.
///
/// A sample with no speed, or with a speed of zero, has no pace: there is
/// nothing to divide.
fn pace_of_sample(speed_ms: Option<f64>) -> Option<f64> {
    let speed = speed_ms?;
    if speed <= 0.0 {
        return None;
    }
    Some(units::round_pace_s_per_km(1000.0 / speed))
}

/// The series of a run's **film**: the samples with the zone each one sits
/// in.
///
/// The zone is the **core's**: the athlete's stored model, read through
/// `stride-core`'s own classifier — the one the time-in-zone computation
/// uses — so the ribbon behind the film's timeline and the bands on the
/// run detail cannot disagree about which zone a reading is in.
///
/// `floors` is `None` when the athlete has no usable zone model: the film
/// is still there, and every sample's zone is `None` — outside the zones,
/// which is what the core says when there are no zones to be in.
pub fn film_samples(
    activity: &FitActivity,
    floors: Option<ZoneFloors>,
    cap: usize,
) -> Vec<FilmSample> {
    let mut points: Vec<FilmSample> = activity
        .samples
        .iter()
        .map(|sample| FilmSample {
            elapsed: render::duration_of(units::round_time_s(sample.elapsed_s)),
            distance: sample.distance_m.map(render::distance_of),
            pace: render::pace(pace_of_sample(sample.speed_ms)),
            heart_rate: render::heart_rate(sample.heart_rate_bpm),
            zone: floors
                .and_then(|floors| stride_core::zones::zone_of(sample.heart_rate_bpm, floors))
                .map(|index| index as i64),
        })
        .collect();
    thin(&mut points, cap);
    points
}

/// The ground of a run's film: the altitude, on the run's own time axis.
///
/// The points are the ones the file carries: a run with no altitude has no
/// ground, and the answer says so with an empty list rather than with a
/// flat line.
pub fn ground(activity: &FitActivity, cap: usize) -> Vec<FilmGround> {
    let mut points: Vec<FilmGround> = activity
        .samples
        .iter()
        .filter_map(|sample| {
            let altitude = sample.altitude_m?;
            Some(FilmGround {
                offset_s: units::round_time_s(sample.elapsed_s),
                elevation: render::elevation_of(altitude),
            })
        })
        .collect();
    thin(&mut points, cap);
    points
}

/// The splits of a run's film: each with the time it began.
///
/// A split is a place on the timeline, and the film can be played from it.
/// The time comes from the file's own lap bounds — the elapsed time at the
/// end of each lap — so a lap that covers a stop still begins where the
/// file says it began.
pub fn film_splits(activity: &FitActivity) -> Vec<FilmSplit> {
    let mut offset_s = 0.0;
    activity
        .laps
        .iter()
        .enumerate()
        .map(|(index, lap)| {
            let split = FilmSplit {
                lap_index: lap.lap_index,
                offset_s: units::round_time_s(offset_s),
                distance: render::distance_of(lap.distance_m),
                duration: render::duration_of(lap.duration_s),
                heart_rate: render::heart_rate(lap.avg_hr.map(|rate| rate as f64)),
                pace: render::pace(super::pace_of(lap.distance_m, lap.duration_s)),
            };
            offset_s = activity
                .lap_bounds
                .get(index)
                .copied()
                .unwrap_or(offset_s + lap.duration_s);
            split
        })
        .collect()
}

/// The route over the whole run, bounded without dropping its endpoints.
pub fn geo_points(activity: &FitActivity, cap: usize) -> Vec<[f64; 2]> {
    let points = &activity.route_points;
    let count = if cap == 0 {
        points.len()
    } else {
        points.len().min(cap)
    };
    (0..count)
        .map(|index| {
            let position = if count < 2 {
                0
            } else {
                index * (points.len() - 1) / (count - 1)
            };
            let point = points[position];
            [
                units::round_gps_degrees(point[0]),
                units::round_gps_degrees(point[1]),
            ]
        })
        .collect()
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
