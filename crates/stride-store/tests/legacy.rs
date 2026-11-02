//! The one-time importer.
//!
//! The tests build a database with the **previous version's schema** — the
//! verbatim DDL is in `common/old_schema.rs` — and check that the history
//! comes across, that it comes across exactly once, and that a failure half
//! way leaves everything as it was.
//!
//! The old phone database was encrypted, so the fixture is encrypted too: it
//! is the same file, opened with the same key, that the importer will meet.

mod common;

use common::old_schema;
use stride_store::legacy::{
    map_activity, map_description, map_plan_block, map_session_type, map_source, map_wellness,
    normalise_sport, OldActivity, OldPlanBlock, OldWellness, SchemaKind,
};
use stride_store::{DatabaseKey, Result, Store};

/// A database with the previous version's schema, in a scratch directory.
fn old_database(dir: &common::ScratchDir) -> Result<Store> {
    let store = Store::open(&dir.file("stride.db"), &common::key()?)?;
    store.with_connection(|connection| {
        for statement in old_schema::TABLES {
            connection
                .execute_batch(statement)
                .map_err(|error| stride_store::StoreError::query("the old schema", error))?;
        }
        Ok(())
    })?;
    Ok(store)
}

/// The same file, with the new schema created beside the old one — which is
/// what the migration does.
fn migrated(dir: &common::ScratchDir) -> Result<Store> {
    let store = Store::open_and_migrate(&dir.file("stride.db"), &common::key()?)?;
    Ok(store)
}

/// One old run.
fn old_run() -> OldActivity {
    OldActivity {
        id: 1,
        source: "garmin".to_string(),
        external_id: "test-run".to_string(),
        name: "a morning run".to_string(),
        sport_type: "trail_running".to_string(),
        start_time: "2024-05-10 06:30:15.123456".to_string(),
        local_date: "2024-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_100.0,
        elevation_gain_m: 42.0,
        avg_hr: Some(148.0),
        max_hr: Some(171.0),
        description: Some("the legs were heavy".to_string()),
        created_at: "2024-05-10 09:00:00.000000".to_string(),
        updated_at: "2024-05-10 09:00:00.000000".to_string(),
    }
}

/// Write one old run.
fn write_old_run(store: &Store, run: &OldActivity) -> Result<()> {
    store.with_connection(|connection| {
        connection
            .execute(
                "INSERT INTO activity (
                    id, source, external_id, name, sport_type, start_time, local_date,
                    distance_m, moving_time_s, elapsed_time_s, elevation_gain_m, avg_hr, max_hr,
                    description, is_trail, is_commute, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                    ?17, ?18)",
                rusqlite::params![
                    run.id,
                    run.source,
                    run.external_id,
                    run.name,
                    run.sport_type,
                    run.start_time,
                    run.local_date,
                    run.distance_m,
                    run.moving_time_s,
                    run.elapsed_time_s,
                    run.elevation_gain_m,
                    run.avg_hr,
                    run.max_hr,
                    run.description,
                    0,
                    0,
                    run.created_at,
                    run.updated_at,
                ],
            )
            .map_err(|error| stride_store::StoreError::query("the old run", error))?;
        Ok(())
    })
}

/// An old database with a run, a day of wellness, a plan block and a
/// session in it.
fn populated(dir: &common::ScratchDir) -> Result<Store> {
    let store = old_database(dir)?;
    write_old_run(&store, &old_run())?;
    store.with_connection(|connection| {
        connection
            .execute(
                "INSERT INTO daily_physiology (
                    date, sleep_s, deep_sleep_s, rem_s, light_sleep_s, awake_s, hrv_ms,
                    resting_hr, stress_avg, source, updated_at
                 ) VALUES ('2024-05-10', 22200.0, 5400.0, 4800.0, 12000.0, 1800.0, 48.0, 63.0,
                    28.0, 'garmin', '2024-05-11 06:00:00.000000')",
                [],
            )
            .map_err(|error| stride_store::StoreError::query("the old wellness", error))?;
        connection
            .execute(
                "INSERT INTO plan_block (id, name, start_date, end_date, goal, status, source,
                    notes, created_at)
                 VALUES (1, 'a block', '2024-05-01', '2024-05-31', 'a goal', 'active', 'garmin',
                    NULL, '2024-04-30 12:00:00.000000')",
                [],
            )
            .map_err(|error| stride_store::StoreError::query("the old block", error))?;
        connection
            .execute(
                "INSERT INTO plan_session (id, week_id, block_id, date, session_type, title,
                    details, target_distance_m, pace_low_s_km, pace_high_s_km, hr_zone_target,
                    status, source, notes)
                 VALUES (1, NULL, 1, '2024-05-10', 'easy', 'an easy run', NULL, 10000.0, 300.0,
                    330.0, NULL, 'planned', 'garmin', NULL)",
                [],
            )
            .map_err(|error| stride_store::StoreError::query("the old session", error))?;
        Ok(())
    })?;
    Ok(store)
}

// ---------------------------------------------------------------- the mapping

#[test]
fn a_run_maps_to_the_new_shape() -> Result<()> {
    let new = map_activity(&old_run())?;
    assert_eq!(new.source, "garmin_personal");
    assert_eq!(new.external_id, "test-run");
    assert_eq!(new.name, "a morning run");
    // The old vocabulary kept the flavours of running apart; this one does not.
    assert_eq!(new.sport_type, "running");
    assert_eq!(new.local_date, "2024-05-10");
    assert_eq!(new.distance_m, 10_000.0);
    assert_eq!(new.moving_time_s, 3_000.0);
    assert_eq!(new.elapsed_time_s, 3_100.0);
    assert_eq!(new.elevation_gain_m, 42.0);
    assert_eq!(new.avg_hr, Some(148.0));
    assert_eq!(new.max_hr, Some(171.0));
    // The old database kept no FIT file, so the run carries none.
    assert!(new.fit_blob.is_empty());
    // The old timestamps were SQLAlchemy's text; this one's is ISO-8601.
    assert_eq!(
        new.start_time.to_rfc3339(),
        "2024-05-10T06:30:15.123456+00:00"
    );
    Ok(())
}

#[test]
fn the_description_becomes_a_note() -> Result<()> {
    let note = map_description(&old_run())?;
    let note = match note {
        Some(note) => note,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the description did not become a note".to_string(),
            ))
        }
    };
    assert_eq!(note.kind, "note");
    assert_eq!(note.text, Some("the legs were heavy".to_string()));

    // A run with no description has no note.
    let mut empty = old_run();
    empty.description = None;
    assert!(map_description(&empty)?.is_none());
    let mut blank = old_run();
    blank.description = Some("   ".to_string());
    assert!(map_description(&blank)?.is_none());
    Ok(())
}

#[test]
fn the_wellness_maps_to_the_new_shape() -> Result<()> {
    let old = OldWellness {
        date: "2024-05-10".to_string(),
        sleep_s: Some(22_200.0),
        deep_sleep_s: Some(5_400.0),
        rem_s: Some(4_800.0),
        light_sleep_s: Some(12_000.0),
        awake_s: Some(1_800.0),
        hrv_ms: Some(48.0),
        resting_hr: Some(63.0),
        stress_avg: Some(28.0),
        source: "garmin".to_string(),
        updated_at: "2024-05-11 06:00:00.000000".to_string(),
    };
    let new = map_wellness(&old)?;
    assert_eq!(new.date, "2024-05-10");
    assert_eq!(new.sleep_s, Some(22_200.0));
    assert_eq!(new.deep_sleep_s, Some(5_400.0));
    assert_eq!(new.rem_s, Some(4_800.0));
    assert_eq!(new.light_sleep_s, Some(12_000.0));
    assert_eq!(new.awake_s, Some(1_800.0));
    assert_eq!(new.hrv_ms, Some(48.0));
    assert_eq!(new.resting_hr, Some(63.0));
    assert_eq!(new.stress_avg, Some(28.0));
    assert_eq!(new.source, "garmin_personal");
    Ok(())
}

#[test]
fn a_plan_block_maps_to_the_new_shape() -> Result<()> {
    let old = OldPlanBlock {
        id: 1,
        name: "a block".to_string(),
        source: "garmin".to_string(),
        start_date: "2024-05-01".to_string(),
        end_date: Some("2024-05-31".to_string()),
        goal: Some("a goal".to_string()),
        status: "active".to_string(),
        notes: None,
        created_at: "2024-04-30 12:00:00.000000".to_string(),
    };
    let new = map_plan_block(&old)?;
    assert_eq!(new.name, "a block");
    assert_eq!(new.source, "prescribed");
    assert_eq!(new.start_date, "2024-05-01");
    assert_eq!(new.end_date, Some("2024-05-31".to_string()));
    assert_eq!(new.goal, Some("a goal".to_string()));
    assert_eq!(new.status, "active");
    Ok(())
}

#[test]
fn the_names_map_as_the_contract_says() {
    assert_eq!(map_source("garmin"), Some("garmin_personal"));
    assert_eq!(map_source("demo"), None);
    assert_eq!(map_session_type("recovery"), "unclassified");
    assert_eq!(normalise_sport("treadmill_running"), "running");
}

// ----------------------------------------------------------------- detection

#[test]
fn an_old_database_is_told_from_a_new_one() -> Result<()> {
    let dir = common::scratch("legacy-detect")?;

    // The old schema.
    let old = old_database(&dir)?;
    let kind = old.with_connection(stride_store::detect_schema)?;
    assert_eq!(kind, SchemaKind::Old);
    assert_eq!(kind.describe(), "the previous version's database");

    // An empty database.
    let empty = rusqlite::Connection::open_in_memory()?;
    assert_eq!(stride_store::detect_schema(&empty)?, SchemaKind::Empty);

    // This version's schema.
    let (new_dir, store) = common::store("legacy-detect-new")?;
    let kind = store.with_connection(stride_store::detect_schema)?;
    assert_eq!(kind, SchemaKind::New);
    assert_eq!(kind.describe(), "this version's database");

    // And a file that is neither.
    let strange = rusqlite::Connection::open_in_memory()?;
    strange.execute_batch("CREATE TABLE something_else (a INTEGER)")?;
    assert_eq!(stride_store::detect_schema(&strange)?, SchemaKind::Unknown);

    drop(new_dir);
    Ok(())
}

#[test]
fn a_migrated_old_database_is_both() -> Result<()> {
    let dir = common::scratch("legacy-mixed")?;
    let old = old_database(&dir)?;
    // The new schema, created beside the old one — which is what the
    // migration does.
    old.with_connection(|connection| {
        for statement in stride_store::schema::all_statements() {
            connection
                .execute_batch(statement)
                .map_err(|error| stride_store::StoreError::query("the schema", error))?;
        }
        Ok(())
    })?;
    let kind = old.with_connection(stride_store::detect_schema)?;
    assert_eq!(kind, SchemaKind::Mixed);
    Ok(())
}

// ------------------------------------------------------------------- the import

#[test]
fn the_history_is_carried_across() -> Result<()> {
    let dir = common::scratch("legacy-import")?;
    let _old = populated(&dir)?;
    let store = migrated(&dir)?;

    let report = stride_store::import_legacy(&store)?;

    assert_eq!(report.runs, 1);
    assert_eq!(report.wellness, 1);
    assert_eq!(report.notes, 1);
    assert_eq!(report.plan_blocks, 1);
    assert_eq!(report.plan_sessions, 1);
    assert_eq!(report.already_there, 0);

    // The run.
    let run = store.activity_by_external("garmin_personal", "test-run")?;
    let run = match run {
        Some(run) => run,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the run was not carried across".to_string(),
            ))
        }
    };
    assert_eq!(run.name, "a morning run");
    assert_eq!(run.sport_type, "running");
    assert_eq!(run.local_date, "2024-05-10");
    assert_eq!(run.distance_m, 10_000.0);
    assert_eq!(run.avg_hr, Some(148.0));
    assert_eq!(run.max_hr, Some(171.0));
    // No FIT file: the old database kept none.
    assert!(run.fit_blob.is_empty());
    assert_eq!(run.fit_sha256.len(), 64);

    // The note.
    let notes = store.notes_for_activity(run.id)?;
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].kind, "note");
    assert_eq!(notes[0].text, Some("the legs were heavy".to_string()));

    // The wellness.
    let wellness = store.wellness_daily("2024-05-10")?;
    let wellness = match wellness {
        Some(wellness) => wellness,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the wellness was not carried across".to_string(),
            ))
        }
    };
    assert_eq!(wellness.sleep_s, Some(22_200.0));
    assert_eq!(wellness.hrv_ms, Some(48.0));
    assert_eq!(wellness.source, "garmin_personal");

    // The plan.
    let blocks = store.plan_blocks()?;
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].name, "a block");
    assert_eq!(blocks[0].source, "prescribed");
    assert_eq!(blocks[0].status, "active");

    let sessions = store.plan_sessions_between("2024-05-01", "2024-05-31")?;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_type, "easy");
    assert_eq!(sessions[0].block_id, blocks[0].id);
    assert_eq!(sessions[0].target_distance_m, Some(10_000.0));

    Ok(())
}

#[test]
fn the_old_tables_are_still_there_afterwards() -> Result<()> {
    let dir = common::scratch("legacy-intact")?;
    let _old = populated(&dir)?;
    let store = migrated(&dir)?;

    stride_store::import_legacy(&store)?;

    // The old database is untouched: its rows are still there.
    let old = Store::open(&dir.file("stride.db"), &common::key()?)?;
    let runs: i64 = old.with_connection(|connection| {
        connection
            .query_row("SELECT count(*) FROM activity", [], |row| row.get(0))
            .map_err(|error| stride_store::StoreError::query("the old runs", error))
    })?;
    assert_eq!(runs, 1);
    let days: i64 = old.with_connection(|connection| {
        connection
            .query_row("SELECT count(*) FROM daily_physiology", [], |row| {
                row.get(0)
            })
            .map_err(|error| stride_store::StoreError::query("the old wellness", error))
    })?;
    assert_eq!(days, 1);
    let blocks: i64 = old.with_connection(|connection| {
        connection
            .query_row("SELECT count(*) FROM plan_block", [], |row| row.get(0))
            .map_err(|error| stride_store::StoreError::query("the old blocks", error))
    })?;
    assert_eq!(blocks, 1);
    Ok(())
}

#[test]
fn running_it_again_changes_nothing() -> Result<()> {
    let dir = common::scratch("legacy-twice")?;
    let _old = populated(&dir)?;
    let store = migrated(&dir)?;

    let first = stride_store::import_legacy(&store)?;
    assert_eq!(first.carried(), 5);

    let second = stride_store::import_legacy(&store)?;
    assert_eq!(
        second.carried(),
        0,
        "the second import carried something again"
    );
    assert_eq!(second.runs, 0);
    assert_eq!(second.wellness, 0);
    assert_eq!(second.notes, 0);
    assert_eq!(second.plan_blocks, 0);
    assert_eq!(second.plan_sessions, 0);
    assert!(second.already_there > 0);

    // And the database has not grown.
    assert_eq!(store.count_activities()?, 1);
    assert_eq!(store.count_wellness_daily()?, 1);
    assert_eq!(store.count_notes()?, 1);
    assert_eq!(store.plan_blocks()?.len(), 1);
    assert_eq!(store.count_plan_sessions()?, 1);
    Ok(())
}

#[test]
fn a_failure_leaves_the_new_tables_empty_and_the_old_data_intact() -> Result<()> {
    let dir = common::scratch("legacy-failure")?;
    let _old = populated(&dir)?;
    let store = migrated(&dir)?;

    // A trigger that refuses the wellness rows, so that the import fails
    // after the runs have been written.
    store.with_connection(|connection| {
        connection
            .execute_batch(
                "CREATE TRIGGER refuse_the_wellness BEFORE INSERT ON wellness_daily
                 BEGIN SELECT RAISE(ABORT, 'no wellness today'); END;",
            )
            .map_err(|error| stride_store::StoreError::query("the trigger", error))
    })?;

    let refused = stride_store::import_legacy(&store);
    assert!(refused.is_err(), "the import should have failed");

    // The new tables are as empty as they were.
    assert_eq!(store.count_activities()?, 0);
    assert_eq!(store.count_wellness_daily()?, 0);
    assert_eq!(store.count_notes()?, 0);
    assert_eq!(store.plan_blocks()?.len(), 0);
    assert_eq!(store.count_plan_sessions()?, 0);

    // And the old data is intact.
    let old = Store::open(&dir.file("stride.db"), &common::key()?)?;
    let runs: i64 = old.with_connection(|connection| {
        connection
            .query_row("SELECT count(*) FROM activity", [], |row| row.get(0))
            .map_err(|error| stride_store::StoreError::query("the old runs", error))
    })?;
    assert_eq!(runs, 1);
    Ok(())
}

#[test]
fn the_synthetic_rows_of_an_older_install_are_not_carried() -> Result<()> {
    let dir = common::scratch("legacy-demo")?;
    let store = old_database(&dir)?;
    let mut demo = old_run();
    demo.id = 2;
    demo.source = "demo".to_string();
    demo.external_id = "test-demo".to_string();
    write_old_run(&store, &demo)?;
    write_old_run(&store, &old_run())?;
    let store = migrated(&dir)?;

    let report = stride_store::import_legacy(&store)?;
    assert_eq!(report.runs, 1, "a synthetic row was carried across");
    assert_eq!(store.count_activities()?, 1);
    Ok(())
}

#[test]
fn a_run_a_pull_had_marked_deleted_is_not_brought_back() -> Result<()> {
    let dir = common::scratch("legacy-tombstone")?;
    let _old = populated(&dir)?;
    let store = migrated(&dir)?;

    // A pull has already marked the run deleted: Garmin no longer lists it.
    store.with_connection(|connection| {
        connection
            .execute(
                "INSERT INTO deleted_garmin_activities (external_id, local_date, marked_at)
                 VALUES ('test-run', '2024-05-10', '2024-05-11 09:00:00.000000')",
                [],
            )
            .map_err(|error| stride_store::StoreError::query("the tombstone", error))?;
        Ok(())
    })?;

    let report = stride_store::import_legacy(&store)?;
    assert_eq!(
        report.runs, 0,
        "a run that was marked deleted was brought back"
    );
    assert_eq!(store.count_activities()?, 0);

    // And the importer did not write to the tombstones: it only reads them.
    let marked: i64 = store.with_connection(|connection| {
        connection
            .query_row(
                "SELECT count(*) FROM deleted_garmin_activities",
                [],
                |row| row.get(0),
            )
            .map_err(|error| stride_store::StoreError::query("the tombstones", error))
    })?;
    assert_eq!(marked, 1, "the importer wrote to the tombstones");
    Ok(())
}

#[test]
fn an_import_of_a_new_database_carries_nothing() -> Result<()> {
    let (dir, store) = common::store("legacy-new")?;
    let report = stride_store::import_legacy(&store)?;
    assert!(report.is_empty());
    assert_eq!(report.left_behind.len(), 0);
    drop(dir);
    Ok(())
}

#[test]
fn the_report_says_what_was_left_behind() -> Result<()> {
    let dir = common::scratch("legacy-left")?;
    let old = populated(&dir)?;
    // The old tables this version has no place for.
    old.with_connection(|connection| {
        connection
            .execute(
                "INSERT INTO activity_stream (activity_id, kind, seconds, \"values\", label, unit)
                 VALUES (1, 'hr', '[]', '[]', NULL, NULL)",
                [],
            )
            .map_err(|error| stride_store::StoreError::query("the old streams", error))?;
        connection
            .execute(
                "INSERT INTO readiness (date, score, grade, drivers, recommendation, rationale,
                    computed_at)
                 VALUES ('2024-05-10', 63.0, 'B', NULL, NULL, NULL, '2024-05-10 09:00:00.000000')",
                [],
            )
            .map_err(|error| stride_store::StoreError::query("the old readiness", error))?;
        Ok(())
    })?;
    let store = migrated(&dir)?;

    let report = stride_store::import_legacy(&store)?;

    let left: Vec<&str> = report
        .left_behind
        .iter()
        .map(|entry| entry.what.as_str())
        .collect();
    assert!(left.contains(&"activity_stream"), "{left:?}");
    assert!(left.contains(&"readiness"), "{left:?}");

    let streams = report
        .left_behind
        .iter()
        .find(|entry| entry.what == "activity_stream");
    let streams = match streams {
        Some(entry) => entry,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the streams were not reported".to_string(),
            ))
        }
    };
    assert_eq!(streams.rows, 1);
    assert!(streams.why.contains("re-derives"), "{}", streams.why);
    Ok(())
}

#[test]
fn the_file_is_told_from_the_outside_too() -> Result<()> {
    let dir = common::scratch("legacy-file")?;
    let _old = populated(&dir)?;
    // The old phone database was encrypted, so the file is opened with the
    // key the caller has.
    let key = DatabaseKey::from_text(common::TEST_KEY)?;
    let kind = stride_store::detect_schema_file(&dir.file("stride.db"), &key)?;
    assert_eq!(kind, SchemaKind::Old);
    Ok(())
}
