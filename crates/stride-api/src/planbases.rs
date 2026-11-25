//! The seam to the core's programmes: the templates the athlete can adopt.
//!
//! **A programme is an intent, not a baseline.** It is what the athlete
//! adopts, and the app never suggests one: the tier's entry requirement is
//! carried beside the athlete's own mileage, and the athlete picks.
//!
//! ## The catalogue is the core's
//!
//! The core holds the four programmes that were read cell by cell from the
//! book's own tables (`stride_core::plan_bases`), and this module is the
//! one place the core's own types become the api's shape, so the route
//! does not change when the catalogue does.

use crate::dto::{BaseRow, BasesInsight};
use crate::state::Api;

/// The programmes the app can adopt.
pub fn bases(api: &Api) -> BasesInsight {
    let bases = stride_core::plan::bases();
    // The athlete's own recent mileage: the fact the tier's entry
    // requirement is stated beside. The core says it, and the app never
    // compares the two.
    let mileage = crate::adoption::mileage(api).unwrap_or_default();
    BasesInsight {
        generated_at: api.generated_at(),
        bases: bases
            .iter()
            .map(|base| {
                let programme = stride_core::plan_bases::programme(&base.id);
                BaseRow {
                    id: base.id.to_string(),
                    label: base.label.to_string(),
                    method: base.method.to_string(),
                    distance: crate::render::distance_of(base.distance_m),
                    distance_label: base.distance_label.to_string(),
                    entry_requirement: base.entry_m_per_week.and_then(|(from, to)| {
                        stride_core::units::render_distance_range(Some(from), Some(to))
                    }),
                    entry_line: programme.and_then(stride_core::plan_bases::entry_requirement_line),
                    entry_from_m: base.entry_m_per_week.map(|(from, _)| from),
                    entry_to_m: base.entry_m_per_week.map(|(_, to)| to),
                    weeks: base.weeks,
                }
            })
            .collect(),
        athlete_mileage: stride_core::plan_bases::athlete_mileage_line(&mileage),
        unavailable: if bases.is_empty() {
            crate::routes::core_line("plan.bases.none")
        } else {
            None
        },
    }
}
