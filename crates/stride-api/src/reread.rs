//! Read the stored files again.
//!
//! **The file is the source of truth, and the reader is code.** A change to
//! the way a FIT file is read — a barometric reset taken out of the
//! elevation, say — reaches the runs that are already in the log only when
//! they are read again, and this is that reading.
//!
//! Nothing is lost: the file stays as it was, the run keeps its id, its
//! day and its name, and only the numbers the file gives are written
//! again. A file that cannot be read is left exactly as it was, and is
//! named in the answer.

use crate::state::Api;
use stride_store::NewActivity;

/// What a re-read found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RereadSummary {
    /// The runs whose file was read again.
    pub read: usize,
    /// The runs whose file could not be read.
    pub unreadable: Vec<String>,
    /// The runs whose numbers changed, and how.
    pub changed: Vec<String>,
}

/// Read every stored file again, and write what it says.
pub fn reread(api: &Api) -> crate::error::ApiResult<RereadSummary> {
    let store = api.store();
    let held = store
        .activities_with_files()
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let mut summary = RereadSummary::default();
    for (activity_id, bytes) in held {
        let Some(existing) = store
            .activity(activity_id)
            .map_err(|error| api.store_error("the run could not be read", &error))?
        else {
            continue;
        };
        let decoded = match stride_ingest::decode_fit(&bytes, "stored.fit") {
            Ok(decoded) => decoded,
            Err(_) => {
                summary.unreadable.push(format!("the run {activity_id}"));
                continue;
            }
        };
        let before = existing.elevation_gain_m;
        let row = NewActivity {
            source: existing.source.clone(),
            external_id: existing.external_id.clone(),
            name: existing.name.clone(),
            sport_type: existing.sport_type.clone(),
            start_time: existing.start_time,
            local_date: existing.local_date.clone(),
            distance_m: decoded.distance_m,
            moving_time_s: decoded.moving_time_s,
            elapsed_time_s: decoded.elapsed_time_s,
            elevation_gain_m: decoded.elevation_gain_m,
            avg_hr: decoded.avg_hr.map(|rate| rate as f64),
            max_hr: decoded.max_hr.map(|rate| rate as f64),
            fit_blob: bytes,
            created_at: existing.created_at,
            updated_at: existing.updated_at,
        };
        store
            .upsert_activity(&row)
            .map_err(|error| api.store_error("the run could not be written", &error))?;
        summary.read += 1;
        if (decoded.elevation_gain_m - before).abs() > 0.5 {
            summary.changed.push(format!(
                "the run {activity_id}: the climb is now {:.0} m, and it was {before:.0} m",
                decoded.elevation_gain_m
            ));
        }
    }
    Ok(summary)
}
