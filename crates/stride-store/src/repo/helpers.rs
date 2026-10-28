//! Small helpers the repositories share.

use crate::error::{Result, StoreError};

/// Collect the rows of a query into a `Vec`.
pub(crate) fn collect<T>(
    rows: impl Iterator<Item = rusqlite::Result<T>>,
    context: &str,
) -> Result<Vec<T>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|source| StoreError::query(context.to_string(), source))?);
    }
    Ok(out)
}
