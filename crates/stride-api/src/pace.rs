//! The seam to the core's pace engine.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's inputs: it reads the store
//! through its existing repositories, hands the core the facts, and reads
//! the core's answers back.
//!
//! It decides nothing. There is no rule about what a pace band is in here,
//! no wording of this crate's own, and no second implementation of the
//! engine's answers: the bands, their edges, their times and their shares
//! all come from the core.
//!
//! ## What it reads
//!
//! * the **runs** — their local day, and the moving samples of their own
//!   FIT files, decoded on read. The file is the source of truth, and a
//!   run whose file cannot be read has no samples: it is in no band.

use stride_core::date::Date;
use stride_core::pace::PaceRun;
use stride_store::ActivitySummary as StoredActivity;

use crate::state::Api;

/// The runs of a window, as the pace bands read them.
///
/// The moving samples come from the run's own file, decoded on read. A run
/// whose file cannot be decoded, or whose file carries no moving sample,
/// has no samples — and a run with no samples is in no band.
pub fn pace_runs(api: &Api, runs: &[StoredActivity]) -> Vec<PaceRun> {
    runs.iter()
        .filter_map(|run| {
            let date = Date::parse(&run.local_date)?;
            let decoded = crate::routes::series::decoded(api, run.id)?;
            let samples = crate::routes::series::pace_points(&decoded);
            if samples.is_empty() {
                return None;
            }
            Some(PaceRun::new(date, samples))
        })
        .collect()
}
