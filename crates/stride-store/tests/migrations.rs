//! The migrations: they run twice, they run on an old database, and the
//! datetimes are stored the way Python stores them.

mod common;

use chrono::SubsecRound;
use rusqlite::params;
use stride_store::{Activity, Store};

#[test]
fn the_migrations_run_twice_without_error() {
    let dir = common::temp_dir("migrate-twice");
    let path = dir.join("stride.db");
    let key = common::key();

    let store = Store::open(&path, &key).expect("could not open");
    store.migrate().expect("the first migration failed");
    store.migrate().expect("the second migration failed");
    store.migrate().expect("the third migration failed");

    // The schema is the same after the second run as after the first: the
    // migrations are idempotent, not merely harmless.
    let before = table_names(&store);
    store.migrate().expect("the fourth migration failed");
    let after = table_names(&store);
    assert_eq!(before, after);

    assert_eq!(before.len(), 17, "the tables are: {before:?}");
}

#[test]
fn an_old_database_gains_the_new_columns_and_loses_the_old_ones() {
    let dir = common::temp_dir("migrate-old");
    let path = dir.join("stride.db");
    let key = common::key();

    // A database from before the running/walking columns existed, and with
    // the `visibility` column the social layer left behind. It is written
    // through the store, so it is encrypted like every other database.
    {
        let store = Store::open(&path, &key).expect("could not open");
        store
            .with_connection(|conn| {
                conn.execute_batch(
                    "CREATE TABLE activity (
                        id INTEGER NOT NULL,
                        athlete_id INTEGER,
                        source VARCHAR(32) NOT NULL,
                        external_id VARCHAR(64) NOT NULL,
                        name VARCHAR(255) NOT NULL,
                        sport_type VARCHAR(40) NOT NULL,
                        description TEXT,
                        start_time DATETIME NOT NULL,
                        local_date VARCHAR(10) NOT NULL,
                        day_of_week VARCHAR(16),
                        distance_m FLOAT NOT NULL,
                        moving_time_s FLOAT NOT NULL,
                        elapsed_time_s FLOAT NOT NULL,
                        elevation_gain_m FLOAT NOT NULL,
                        avg_hr FLOAT,
                        max_hr FLOAT,
                        calories FLOAT,
                        avg_cadence FLOAT,
                        max_cadence FLOAT,
                        avg_speed_ms FLOAT,
                        max_speed_ms FLOAT,
                        perceived_exertion INTEGER,
                        device VARCHAR(120),
                        gear VARCHAR(120),
                        is_trail BOOLEAN NOT NULL,
                        is_commute BOOLEAN NOT NULL,
                        polyline JSON,
                        geo_points JSON,
                        raw JSON,
                        visibility VARCHAR(16) NOT NULL,
                        created_at DATETIME NOT NULL,
                        updated_at DATETIME NOT NULL,
                        PRIMARY KEY (id),
                        CONSTRAINT uq_activity_source_external UNIQUE (source, external_id)
                     );",
                )
                .map_err(|source| {
                    stride_store::StoreError::query("the old activity table".to_string(), source)
                })
            })
            .expect("could not create the old table");
    }

    let store = Store::open(&path, &key).expect("could not open");
    store.migrate().expect("the migration failed");

    let columns = column_names(&store, "activity");
    for added in [
        "running_time_s",
        "walking_time_s",
        "stopped_time_s",
        "running_distance_m",
    ] {
        assert!(
            columns.iter().any(|name| name == added),
            "`{added}` was not added to the old table"
        );
    }
    assert!(
        !columns.iter().any(|name| name == "visibility"),
        "`visibility` is still in the old table"
    );

    // The old table still works.
    let id = store
        .upsert_activity(&Activity {
            source: "garmin".to_string(),
            external_id: "1".to_string(),
            ..Activity::default()
        })
        .expect("could not write to the migrated table");
    assert!(id > 0);
}

#[test]
fn the_datetimes_are_stored_the_way_python_stores_them() {
    let dir = common::temp_dir("datetime-format");
    let path = dir.join("stride.db");
    let key = common::key();

    let store = Store::open_and_migrate(&path, &key).expect("could not open");
    let activity = Activity {
        source: "garmin".to_string(),
        external_id: "1".to_string(),
        start_time: chrono::Utc::now(),
        ..Activity::default()
    };
    store.upsert_activity(&activity).expect("could not write");

    let raw: String = store
        .with_connection(|conn| {
            conn.query_row(
                "SELECT start_time FROM activity WHERE external_id = ?1",
                params!["1"],
                |row| row.get(0),
            )
            .map_err(|source| stride_store::StoreError::query("start_time".to_string(), source))
        })
        .expect("could not read the raw text");

    // SQLAlchemy's SQLite storage format: `YYYY-MM-DD HH:MM:SS.ffffff`.
    assert!(
        raw.len() == 26,
        "`{raw}` is not the SQLAlchemy datetime format"
    );
    assert_eq!(&raw[4..5], "-");
    assert_eq!(&raw[7..8], "-");
    assert_eq!(&raw[10..11], " ");
    assert_eq!(&raw[13..14], ":");
    assert_eq!(&raw[16..17], ":");
    assert_eq!(&raw[19..20], ".");
    assert!(
        raw[20..]
            .chars()
            .all(|character| character.is_ascii_digit()),
        "`{raw}` does not end in six digits of microseconds"
    );

    // And it is the same instant — to the microsecond, which is what the
    // SQLAlchemy storage format keeps.
    let read = store
        .activity(1)
        .expect("could not read")
        .expect("the activity is not there");
    let truncated = activity.start_time.trunc_subsecs(6);
    assert_eq!(read.start_time, truncated);
}

/// The names of the tables in the database.
fn table_names(store: &Store) -> Vec<String> {
    store
        .with_connection(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type = 'table' \
                     AND name NOT LIKE 'sqlite_%' ORDER BY name",
                )
                .map_err(|source| {
                    stride_store::StoreError::query("sqlite_master".to_string(), source)
                })?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|source| {
                    stride_store::StoreError::query("sqlite_master".to_string(), source)
                })?;
            let mut names = Vec::new();
            for row in rows {
                names.push(row.map_err(|source| {
                    stride_store::StoreError::query("sqlite_master".to_string(), source)
                })?);
            }
            Ok(names)
        })
        .expect("could not list the tables")
}

/// The names of the columns of a table.
fn column_names(store: &Store, table: &str) -> Vec<String> {
    store
        .with_connection(|conn| {
            let mut statement = conn
                .prepare(&format!("PRAGMA table_info({table})"))
                .map_err(|source| {
                    stride_store::StoreError::query(format!("table_info({table})"), source)
                })?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(|source| {
                    stride_store::StoreError::query(format!("table_info({table})"), source)
                })?;
            let mut names = Vec::new();
            for row in rows {
                names.push(row.map_err(|source| {
                    stride_store::StoreError::query(format!("table_info({table})"), source)
                })?);
            }
            Ok(names)
        })
        .expect("could not list the columns")
}
