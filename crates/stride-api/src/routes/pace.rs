//! The pace page: the training week by week, the pace at a heart rate, and
//! where the time went by pace.
//!
//! The page answers one question — how the pace has behaved — and it
//! answers it with three panels:
//!
//! * **the training, week by week** — the week's distance, the long run
//!   inside it, and the week's pace on the other axis;
//! * **the pace at a heart rate** — one point a week, the pace that was
//!   held at that week's median heart rate;
//! * **where the time went, by pace** — the pace bands.
//!
//! ## The window
//!
//! The window is the Progress window: the same `weeks` parameter, the same
//! bound and the same calendar weeks, the current partial week included.
//! The bands are a property of the window, so the answer **names** it.
//!
//! ## What it reads
//!
//! The weeks come from the stored runs; the bands come from the runs' own
//! **moving samples**, decoded from their FIT files on read. The pace at a
//! heart rate is the **separate weekly medians** the Progress screen
//! already answers with — the pace of the week's runs and the heart rate of
//! the week's runs — and this route does not turn them into a matched-effort
//! claim.
//!
//! ## No value is the absence of the value
//!
//! A window with no moving samples has **no bands**: the answer carries the
//! reason and no numbers, never a band of zero.

use serde_json::Value;
use stride_core::eddington::Period;
use stride_core::pace::PaceBands as CoreBands;

use crate::dto::{PaceBandRow, PaceBands, PaceInsight, PaceLongest, PaceWindow, TrainingWeek};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// How many weeks the window may cover — the Progress route's own bound.
const MAX_WEEKS: i64 = 104;

/// The pace page's answer over the selected window.
pub fn pace(api: &Api, request: &Request) -> ApiResult<Value> {
    let weeks = request.query.integer("weeks")?.unwrap_or(12);
    if !(1..=MAX_WEEKS).contains(&weeks) {
        return Err(ApiError::bad_request(
            "the pace window is not between 1 and 104 weeks",
        ));
    }

    let today = super::today(api)?;
    // The window: the same calendar weeks the Progress screen shows, the
    // current partial week included.
    let from = super::week_start(today).add_days(-7 * (weeks - 1));

    let runs = api
        .store()
        .activities_between(&from.to_iso(), &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;

    // The weeks: the distance, the long run inside it, and the separate
    // medians the pace-at-heart-rate panel draws.
    let weeks_of_window: Vec<TrainingWeek> =
        super::week_summaries(&runs).iter().map(week_of).collect();

    // The bands: the runs' own moving samples, decoded on read.
    let pace_runs = crate::pace::pace_runs(api, &runs);
    let bands = stride_core::pace::pace_bands(&pace_runs, Period::new(from, today));

    let answer = PaceInsight {
        generated_at: api.generated_at(),
        date: today.to_iso(),
        window: PaceWindow {
            from: from.to_iso(),
            from_label: super::day_label(from),
            to: today.to_iso(),
            to_label: super::day_label(today),
            weeks,
        },
        weeks: weeks_of_window,
        bands: if bands.is_empty() {
            None
        } else {
            Some(bands_of(&bands))
        },
        // Why there is no value, in the core's own words.
        unavailable: if bands.is_empty() {
            stride_core::strings::render("pace.no_samples", &[])
        } else {
            None
        },
    };
    ok(&answer)
}

/// One week, as the answer carries it.
///
/// The pace and the heart rate are the week's **separate medians**: the
/// pace of the week's runs and the heart rate of the week's runs. They are
/// descriptive, and this route does not pair them into a claim.
fn week_of(week: &super::WeekSummary) -> TrainingWeek {
    TrainingWeek {
        week_start: week.week_start.to_iso(),
        distance: render::distance_of(week.distance_m),
        sessions: week.sessions,
        longest: week.longest.map(|longest| PaceLongest {
            distance: render::distance_of(longest.distance_m),
            pace: render::pace(longest.pace_s_per_km),
        }),
        pace: render::pace(week.median_pace_s_per_km),
        heart_rate: render::heart_rate(week.median_hr_bpm),
    }
}

/// The bands, as the answer carries them.
///
/// The labels are the core's own words — "4:30–5:00 /km" — and the numbers
/// beside them are the core's rendering: the time, the distance and the
/// share of the window's time.
///
/// **Only the bands that carry a readable share of the time are
/// answered.** The grid is the whole range of paces, from a sprint to a
/// standstill, and a window of training touches a handful of them: the
/// empty bands are not rows a reader can use, and there are a hundred and
/// forty of them. The time in the bands that were too small to draw
/// comes beside them, so the numbers still add up.
fn bands_of(bands: &CoreBands) -> PaceBands {
    PaceBands {
        width_s_per_km: stride_core::pace::BAND_WIDTH_S_PER_KM,
        total: render::duration_of(bands.total_s),
        distance: render::distance_of(bands.total_m),
        rows: bands
            .bands
            .iter()
            .map(|band| PaceBandRow {
                from: band.from_s_per_km,
                to: band.to_s_per_km,
                label: band.label().unwrap_or_default(),
                time: render::duration_of(band.time_s),
                distance: render::distance_of(band.distance_m),
                share: render::share(Some(band.share_percent)),
            })
            .collect(),
        rest: render::duration_of(bands.rest_s),
    }
}
