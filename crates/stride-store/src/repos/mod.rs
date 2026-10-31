//! The repositories: one per table.
//!
//! Every query in this crate is **parameterised**. There is no
//! string-built SQL anywhere in the tree: the statements are `format!`ed
//! only where a *column list* is interpolated, and never where a value is.
//! `scripts/check-contract-greps.sh` fails the build if that ever stops being
//! true.
//!
//! | module | table |
//! |---|---|
//! | [`activity`] | `activities`, `activity_samples` |
//! | [`wellness`] | `wellness_daily` |
//! | [`plan`] | `plan_blocks`, `plan_sessions` |
//! | [`notes`] | `notes` |
//! | [`sync`] | `sync_state` |

use crate::error::{Result, StoreError};

pub mod activity;
pub mod notes;
pub mod plan;
pub mod sync;
pub mod wellness;

/// Collect the rows of a query into a vector.
///
/// A row that cannot be read is an error, not a row that is not there:
/// swallowing it would make an empty table look like a table with rows in it.
pub(crate) fn collect<T>(rows: impl Iterator<Item = rusqlite::Result<T>>) -> Result<Vec<T>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|source| StoreError::query("a row", source))?);
    }
    Ok(out)
}
