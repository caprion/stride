//! The heatmap: a grid over the stored positions, computed on the device.
//!
//! **Nothing leaves the device.** This route fetches nothing, asks nobody
//! and sends nothing: it reads the athlete's own stored runs, decodes their
//! stored FIT files — the same files the run detail's route decodes — and
//! hands the positions to the core's grid. There is no tile server behind
//! it, no geocoder and no third-party service, and the answer is a count
//! per cell.
//!
//! ## What it reads
//!
//! The runs, newest first, up to [`MAX_RUNS`]. The bound is the answer's
//! own: decoding a file per run is work, and an answer that reads
//! everything a decade of running holds would take long enough on a phone
//! to be a bad surface. When the store holds more runs than the answer
//! read, the answer says so — a partial map must not read as the whole
//! history.
//!
//! ## What it does not do
//!
//! It stores nothing. There is no grid, no tile and no heatmap blob in the
//! database: like every other derived series, the grid is recomputed on
//! read and never written down.

use serde_json::Value;
use stride_core::heatmap::{cell_size_m, heatmap as grid_of, Heatmap};
use stride_core::units;
use stride_ingest::FitActivity;

use crate::dto::{HeatmapCell, HeatmapExtent, HeatmapInsight, RenderedValue};
use crate::error::ApiResult;
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// The most runs one answer may read.
///
/// The answer is bounded: the files are decoded on read, and a decade of
/// running is more files than a phone should decode for one picture.
const MAX_RUNS: usize = 500;

/// The grid over the stored positions.
pub fn heatmap(api: &Api, _request: &Request) -> ApiResult<Value> {
    let today = super::today(api)?;
    let store = api.store();

    let stored = store
        .count_activities()
        .map_err(|error| api.store_error("the runs could not be counted", &error))?;
    // The feed's own order: the newest first, so the bound keeps the runs
    // the athlete is looking at.
    let activities = store
        .activities_between("0001-01-01", &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let selected: Vec<_> = activities.iter().take(MAX_RUNS).collect();
    let more_runs = stored > selected.len() as i64;

    // The positions, decoded from the files the runs were stored with.
    let decoded: Vec<FitActivity> = selected
        .iter()
        .filter_map(|run| super::series::decoded(api, run.id))
        .collect();
    let positions: Vec<&[[f64; 2]]> = decoded
        .iter()
        .map(|activity| activity.route_points.as_slice())
        .collect();

    let grid = grid_of(&positions);
    let answer = HeatmapInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        runs: grid.runs,
        runs_outside: grid.runs_outside,
        more_runs,
        cell_size: cell_size_of(&grid),
        cells: cells_of(&grid),
        extent: grid.extent.map(|bounds| HeatmapExtent {
            south: units::round_gps_degrees(bounds.south),
            west: units::round_gps_degrees(bounds.west),
            north: units::round_gps_degrees(bounds.north),
            east: units::round_gps_degrees(bounds.east),
        }),
        unavailable: if grid.is_empty() {
            Some("no_positions".to_string())
        } else {
            None
        },
    };
    ok(&answer)
}

/// The cells of the grid, as the answer carries them.
///
/// The corners are rounded the way every other coordinate in this crate is
/// rounded, so the cells of a row still tile the row exactly.
fn cells_of(grid: &Heatmap) -> Vec<HeatmapCell> {
    grid.cells
        .iter()
        .map(|cell| HeatmapCell {
            x: cell.cell.x,
            y: cell.cell.y,
            south: units::round_gps_degrees(cell.bounds.south),
            west: units::round_gps_degrees(cell.bounds.west),
            north: units::round_gps_degrees(cell.bounds.north),
            east: units::round_gps_degrees(cell.bounds.east),
            runs: cell.runs,
        })
        .collect()
}

/// The size of one cell on the ground, where the athlete runs.
///
/// The cells of the projection are squares, and they shrink towards the
/// poles: the size is the one at the middle of the area the map covers.
fn cell_size_of(grid: &Heatmap) -> Option<RenderedValue> {
    let extent = grid.extent?;
    let middle = (extent.north + extent.south) / 2.0;
    let metres = cell_size_m(middle);
    render::distance(Some(metres))
}
