//! `activities` and `activity_samples`.
//!
//! The FIT file is the source of truth: it is stored, byte for byte, and
//! every derived series is recomputable from it. The derived columns are
//! written next to it so that the common question — how long was that run —
//! does not need a FIT decoder, but nothing is derived *only* here: prune the
//! samples and they come back from the blob.
//!
//! ## The write is one transaction
//!
//! A run arrives with its metrics, its blob and (optionally) its derived
//! series. They are written together or not at all: a run whose series failed
//! to write is not a run with a missing series, it is a run that was not
//! written.

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::hash;
use crate::model::{Activity, ActivitySamples, ActivitySummary, NewActivity, NewActivitySamples};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `activities`, in the order the schema declares them.
const ACTIVITY_COLUMNS: &str = "id, source, external_id, name, sport_type, start_time, \
    local_date, distance_m, moving_time_s, elapsed_time_s, elevation_gain_m, avg_hr, max_hr, \
    fit_blob, fit_sha256, created_at, updated_at";

/// The columns of `activities` without the FIT file.
const SUMMARY_COLUMNS: &str = "id, source, external_id, name, sport_type, start_time, \
    local_date, distance_m, moving_time_s, elapsed_time_s, elevation_gain_m, avg_hr, max_hr, \
    created_at, updated_at";

/// The columns of `activity_samples`.
const SAMPLE_COLUMNS: &str = "activity_id, series, sha256, first_s, last_s";

/// Read one row of `activities`.
fn map_activity(row: &Row<'_>) -> rusqlite::Result<Activity> {
    Ok(Activity {
        id: row.get("id")?,
        source: row.get("source")?,
        external_id: row.get("external_id")?,
        name: row.get("name")?,
        sport_type: row.get("sport_type")?,
        start_time: row.get::<_, DbDateTime>("start_time")?.0,
        local_date: row.get("local_date")?,
        distance_m: row.get("distance_m")?,
        moving_time_s: row.get("moving_time_s")?,
        elapsed_time_s: row.get("elapsed_time_s")?,
        elevation_gain_m: row.get("elevation_gain_m")?,
        avg_hr: row.get("avg_hr")?,
        max_hr: row.get("max_hr")?,
        fit_blob: row.get("fit_blob")?,
        fit_sha256: row.get("fit_sha256")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

/// Read one row of `activities` without the FIT file.
fn map_summary(row: &Row<'_>) -> rusqlite::Result<ActivitySummary> {
    Ok(ActivitySummary {
        id: row.get("id")?,
        source: row.get("source")?,
        external_id: row.get("external_id")?,
        name: row.get("name")?,
        sport_type: row.get("sport_type")?,
        start_time: row.get::<_, DbDateTime>("start_time")?.0,
        local_date: row.get("local_date")?,
        distance_m: row.get("distance_m")?,
        moving_time_s: row.get("moving_time_s")?,
        elapsed_time_s: row.get("elapsed_time_s")?,
        elevation_gain_m: row.get("elevation_gain_m")?,
        avg_hr: row.get("avg_hr")?,
        max_hr: row.get("max_hr")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
        updated_at: row.get::<_, DbDateTime>("updated_at")?.0,
    })
}

/// Read one row of `activity_samples`.
fn map_samples(row: &Row<'_>) -> rusqlite::Result<ActivitySamples> {
    Ok(ActivitySamples {
        activity_id: row.get("activity_id")?,
        series: row.get("series")?,
        sha256: row.get("sha256")?,
        first_s: row.get("first_s")?,
        last_s: row.get("last_s")?,
    })
}

/// The cursor of the activity feed.
///
/// The feed is ordered by the athlete's local date and then by the row's
/// primary key, both descending, so the newest run is first and the order is
/// total. The cursor is the last row of the page that was read, and the next
/// page starts strictly after it.
///
/// It is **opaque**: it is the hexadecimal form of the two sort keys, and
/// nothing outside this module knows that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedCursor {
    local_date: String,
    id: i64,
}

impl FeedCursor {
    /// A cursor for a position in the feed.
    pub fn new(local_date: impl Into<String>, id: i64) -> Self {
        FeedCursor {
            local_date: local_date.into(),
            id,
        }
    }

    /// The athlete's local date the cursor points at.
    pub fn local_date(&self) -> &str {
        &self.local_date
    }

    /// The row's primary key the cursor points at.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// The cursor as an opaque string.
    pub fn encode(&self) -> String {
        let raw = format!("{}|{}", self.local_date, self.id);
        let mut out = String::with_capacity(raw.len() * 2);
        for byte in raw.as_bytes() {
            use std::fmt::Write as _;
            let _ = write!(out, "{byte:02x}");
        }
        out
    }

    /// Read a cursor back from its opaque string.
    pub fn decode(value: &str) -> Result<FeedCursor> {
        let bytes = decode_hex(value)?;
        let text = String::from_utf8(bytes).map_err(|_| {
            StoreError::Invariant(format!(
                "the cursor {value:?} is not a cursor this build wrote"
            ))
        })?;
        let (local_date, id) = match text.rsplit_once('|') {
            Some((local_date, id)) => (local_date.to_string(), id.to_string()),
            None => {
                return Err(StoreError::Invariant(format!(
                    "the cursor {value:?} is not a cursor this build wrote"
                )))
            }
        };
        let id = id.parse::<i64>().map_err(|_| {
            StoreError::Invariant(format!(
                "the cursor {value:?} is not a cursor this build wrote"
            ))
        })?;
        Ok(FeedCursor { local_date, id })
    }
}

/// Decode a hexadecimal string into bytes.
fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if value.len() % 2 != 0 {
        return Err(StoreError::Invariant(format!(
            "the cursor {value:?} is not a cursor this build wrote"
        )));
    }
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let text = std::str::from_utf8(pair).map_err(|_| {
            StoreError::Invariant(format!(
                "the cursor {value:?} is not a cursor this build wrote"
            ))
        })?;
        let byte = u8::from_str_radix(text, 16).map_err(|_| {
            StoreError::Invariant(format!(
                "the cursor {value:?} is not a cursor this build wrote"
            ))
        })?;
        out.push(byte);
    }
    Ok(out)
}

/// One page of the activity feed.
#[derive(Debug, Clone, PartialEq)]
pub struct FeedPage {
    /// The runs in the page, newest first.
    pub activities: Vec<ActivitySummary>,
    /// The cursor of the next page, if there is one.
    pub next_cursor: Option<FeedCursor>,
}

impl Store {
    /// Write one run, with its metrics and its FIT file.
    ///
    /// The FIT file's SHA-256 is computed here, so the hash in the row is
    /// always the hash of the blob that is in the row. A run that is already
    /// there — the same source and the same external id — is updated in place
    /// and keeps its primary key and its creation time.
    ///
    /// Returns the run's primary key.
    pub fn upsert_activity(&self, activity: &NewActivity) -> Result<i64> {
        self.upsert_activity_with_samples(activity, None)
    }

    /// Write one run, with its metrics, its FIT file and its derived series.
    ///
    /// All of it happens in **one transaction**: either the run, its blob and
    /// its series are all there, or none of them are.
    pub fn upsert_activity_with_samples(
        &self,
        activity: &NewActivity,
        samples: Option<&NewActivitySamples>,
    ) -> Result<i64> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let digest = hash::sha256_hex(&activity.fit_blob);

        let id = transaction
            .query_row(
                "INSERT INTO activities (
                    source, external_id, name, sport_type, start_time, local_date,
                    distance_m, moving_time_s, elapsed_time_s, elevation_gain_m,
                    avg_hr, max_hr, fit_blob, fit_sha256, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                 ON CONFLICT(source, external_id) DO UPDATE SET
                    name = excluded.name,
                    sport_type = excluded.sport_type,
                    start_time = excluded.start_time,
                    local_date = excluded.local_date,
                    distance_m = excluded.distance_m,
                    moving_time_s = excluded.moving_time_s,
                    elapsed_time_s = excluded.elapsed_time_s,
                    elevation_gain_m = excluded.elevation_gain_m,
                    avg_hr = excluded.avg_hr,
                    max_hr = excluded.max_hr,
                    fit_blob = excluded.fit_blob,
                    fit_sha256 = excluded.fit_sha256,
                    updated_at = excluded.updated_at
                 RETURNING id",
                params![
                    activity.source,
                    activity.external_id,
                    activity.name,
                    activity.sport_type,
                    DbDateTime(activity.start_time),
                    activity.local_date,
                    activity.distance_m,
                    activity.moving_time_s,
                    activity.elapsed_time_s,
                    activity.elevation_gain_m,
                    activity.avg_hr,
                    activity.max_hr,
                    activity.fit_blob,
                    digest,
                    DbDateTime(activity.created_at),
                    DbDateTime(activity.updated_at),
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| StoreError::query("upsert activity", source))?;

        if let Some(samples) = samples {
            transaction
                .execute(
                    "INSERT INTO activity_samples (activity_id, series, sha256, first_s, last_s)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(activity_id) DO UPDATE SET
                        series = excluded.series,
                        sha256 = excluded.sha256,
                        first_s = excluded.first_s,
                        last_s = excluded.last_s",
                    params![
                        id,
                        samples.series,
                        samples.sha256,
                        samples.first_s,
                        samples.last_s,
                    ],
                )
                .map_err(|source| StoreError::query("upsert activity_samples", source))?;
        }

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// One run, with its FIT file.
    pub fn activity(&self, id: i64) -> Result<Option<Activity>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {ACTIVITY_COLUMNS} FROM activities WHERE id = ?1");
        conn.query_row(&sql, params![id], map_activity)
            .optional()
            .map_err(|source| StoreError::query("activity by id", source))
    }

    /// One run, by the vendor's id.
    pub fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<Activity>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {ACTIVITY_COLUMNS} FROM activities WHERE source = ?1 AND external_id = ?2"
        );
        conn.query_row(&sql, params![source, external_id], map_activity)
            .optional()
            .map_err(|source| StoreError::query("activity by source and external id", source))
    }

    /// One page of the activity feed, newest first.
    ///
    /// `after` is the cursor of the last row of the previous page; `None`
    /// starts at the newest run. The page holds at most `limit` runs — a
    /// limit of zero holds none.
    ///
    /// The page says whether there is another one: the query asks for one row
    /// more than the limit and throws it away, so the feed never claims a
    /// next page that is empty.
    pub fn activity_feed(&self, after: Option<&FeedCursor>, limit: u32) -> Result<FeedPage> {
        let conn = self.lock()?;
        let limit = i64::from(limit);
        let wanted = limit + 1;

        let mut rows: Vec<ActivitySummary> = match after {
            Some(cursor) => {
                let sql = format!(
                    "SELECT {SUMMARY_COLUMNS} FROM activities
                     WHERE (local_date < ?1) OR (local_date = ?1 AND id < ?2)
                     ORDER BY local_date DESC, id DESC
                     LIMIT ?3"
                );
                let mut statement = conn
                    .prepare(&sql)
                    .map_err(|source| StoreError::query("activity feed", source))?;
                let mapped = statement
                    .query_map(params![cursor.local_date, cursor.id, wanted], map_summary)
                    .map_err(|source| StoreError::query("activity feed", source))?;
                collect(mapped)?
            }
            None => {
                let sql = format!(
                    "SELECT {SUMMARY_COLUMNS} FROM activities
                     ORDER BY local_date DESC, id DESC
                     LIMIT ?1"
                );
                let mut statement = conn
                    .prepare(&sql)
                    .map_err(|source| StoreError::query("activity feed", source))?;
                let mapped = statement
                    .query_map(params![wanted], map_summary)
                    .map_err(|source| StoreError::query("activity feed", source))?;
                collect(mapped)?
            }
        };

        let has_more = rows.len() as i64 > limit;
        rows.truncate(limit as usize);

        let next_cursor = match (has_more, rows.last()) {
            (true, Some(last)) => Some(FeedCursor::new(last.local_date.clone(), last.id)),
            _ => None,
        };

        Ok(FeedPage {
            activities: rows,
            next_cursor,
        })
    }

    /// The runs between two athlete-local dates, both inclusive, newest first.
    pub fn activities_between(&self, from: &str, to: &str) -> Result<Vec<ActivitySummary>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SUMMARY_COLUMNS} FROM activities
             WHERE local_date >= ?1 AND local_date <= ?2
             ORDER BY local_date DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("activities between", source))?;
        let mapped = statement
            .query_map(params![from, to], map_summary)
            .map_err(|source| StoreError::query("activities between", source))?;
        collect(mapped)
    }

    /// The runs of one athlete-local date, oldest first.
    pub fn activities_for_date(&self, date: &str) -> Result<Vec<ActivitySummary>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SUMMARY_COLUMNS} FROM activities
             WHERE local_date = ?1
             ORDER BY start_time ASC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("activities for a date", source))?;
        let mapped = statement
            .query_map(params![date], map_summary)
            .map_err(|source| StoreError::query("activities for a date", source))?;
        collect(mapped)
    }

    /// The runs of one sport, newest first.
    pub fn activities_of_sport(&self, sport_type: &str) -> Result<Vec<ActivitySummary>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {SUMMARY_COLUMNS} FROM activities
             WHERE sport_type = ?1
             ORDER BY local_date DESC, id DESC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("activities of a sport", source))?;
        let mapped = statement
            .query_map(params![sport_type], map_summary)
            .map_err(|source| StoreError::query("activities of a sport", source))?;
        collect(mapped)
    }

    /// How many runs there are.
    pub fn count_activities(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM activities", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count activities", source))
    }

    /// The FIT file of one run.
    pub fn fit_blob(&self, activity_id: i64) -> Result<Option<Vec<u8>>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT fit_blob FROM activities WHERE id = ?1",
            params![activity_id],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()
        .map_err(|source| StoreError::query("fit blob", source))
    }

    /// The SHA-256 of the FIT file of one run.
    pub fn fit_sha256(&self, activity_id: i64) -> Result<Option<String>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT fit_sha256 FROM activities WHERE id = ?1",
            params![activity_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|source| StoreError::query("fit sha256", source))
    }

    /// The derived series of one run.
    pub fn activity_samples(&self, activity_id: i64) -> Result<Option<ActivitySamples>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {SAMPLE_COLUMNS} FROM activity_samples WHERE activity_id = ?1");
        conn.query_row(&sql, params![activity_id], map_samples)
            .optional()
            .map_err(|source| StoreError::query("activity_samples", source))
    }

    /// Write the derived series of one run.
    ///
    /// The series may be written on its own; it does not have to arrive with
    /// the run. It is a cache of what the FIT file already says.
    pub fn put_activity_samples(&self, samples: &ActivitySamples) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO activity_samples (activity_id, series, sha256, first_s, last_s)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(activity_id) DO UPDATE SET
                series = excluded.series,
                sha256 = excluded.sha256,
                first_s = excluded.first_s,
                last_s = excluded.last_s",
            params![
                samples.activity_id,
                samples.series,
                samples.sha256,
                samples.first_s,
                samples.last_s,
            ],
        )
        .map_err(|source| StoreError::query("upsert activity_samples", source))?;
        Ok(())
    }

    /// Throw the derived series of one run away.
    ///
    /// The FIT file is still there, so the series can be derived again. This
    /// is what "pruned" means in the schema: gone, but not lost.
    pub fn prune_activity_samples(&self, activity_id: i64) -> Result<usize> {
        let conn = self.lock()?;
        let removed = conn
            .execute(
                "DELETE FROM activity_samples WHERE activity_id = ?1",
                params![activity_id],
            )
            .map_err(|source| StoreError::query("prune activity_samples", source))?;
        Ok(removed)
    }

    /// Throw every derived series away.
    ///
    /// The FIT files are still there, so every series can be derived again.
    pub fn prune_all_activity_samples(&self) -> Result<usize> {
        let conn = self.lock()?;
        let removed = conn
            .execute("DELETE FROM activity_samples", [])
            .map_err(|source| StoreError::query("prune every activity_samples", source))?;
        Ok(removed)
    }

    /// How many runs have a derived series.
    pub fn count_activity_samples(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM activity_samples", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count activity_samples", source))
    }
}
