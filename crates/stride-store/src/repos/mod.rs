//! The repositories: one per table.
//!
//! Every query in this crate is **parameterised**: no value is ever
//! interpolated into a statement, and the values are bound. The one thing a
//! statement may interpolate is a **column list**, and that list is a
//! compile-time constant — Rust's naming says so, and the lints enforce it,
//! because a local variable cannot be written in upper case.
//!
//! The table a statement names is always written out in the statement
//! itself. A name that came from a variable could be anything, and a
//! statement that is built at runtime is a statement nobody read.
//!
//! `scripts/check-contract-greps.sh` fails the build if an interpolated
//! value or an interpolated identifier that is not a constant ever appears
//! in a statement.
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
