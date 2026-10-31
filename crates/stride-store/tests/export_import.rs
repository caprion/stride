//! The export, the import, and the plaintext → SQLCipher conversion.

mod common;

use stride_store::export::{self, Format};
use stride_store::{DatabaseKey, Result, Store};

#[test]
fn an_export_is_a_database_and_a_manifest() -> Result<()> {
    let (dir, store) = common::store("export-make")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.upsert_wellness_daily(&common::wellness("2026-05-10"))?;

    let target = dir.file("export");
    let manifest = export::export(&store, &target, &common::key()?)?;

    assert_eq!(manifest.format, "stride-export");
    assert_eq!(manifest.format_version, 1);
    assert_eq!(manifest.schema_version, stride_store::SCHEMA_VERSION);
    assert_eq!(manifest.database, "stride.db");
    assert_eq!(manifest.tables.get("activities"), Some(&1));
    assert_eq!(manifest.tables.get("wellness_daily"), Some(&1));
    assert!(manifest.database_sha256.len() == 64);
    assert!(target.join("manifest.json").is_file());
    assert!(target.join("stride.db").is_file());

    // The exported database opens with the key, and it holds the run.
    let exported = Store::open(&target.join("stride.db"), &common::key()?)?;
    assert_eq!(exported.count_activities()?, 1);
    assert_eq!(exported.count_wellness_daily()?, 1);

    // And it is encrypted: the file does not start with SQLite's magic.
    let bytes = common::read(&target.join("stride.db"))?;
    assert!(!bytes.starts_with(b"SQLite format 3"));

    drop(dir);
    Ok(())
}

#[test]
fn an_import_reads_the_export_back() -> Result<()> {
    let (dir, store) = common::store("export-import")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.upsert_wellness_daily(&common::wellness("2026-05-10"))?;

    let target = dir.file("export");
    export::export(&store, &target, &common::key()?)?;

    // A second database, empty, reads the export.
    let other = Store::open_and_migrate(&dir.file("other.db"), &common::key()?)?;
    let report = export::import(&other, &target, &common::key()?)?;

    assert_eq!(report.activities, 1);
    assert_eq!(report.wellness, 1);
    assert_eq!(other.count_activities()?, 1);
    assert_eq!(other.count_wellness_daily()?, 1);

    let run = other.activity_by_external("garmin_personal", "run-1")?;
    let run = match run {
        Some(run) => run,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the run was not imported".to_string(),
            ))
        }
    };
    assert_eq!(run.local_date, "2026-05-10");
    assert_eq!(run.distance_m, 10_000.0);

    drop(dir);
    Ok(())
}

#[test]
fn an_import_twice_adds_nothing_the_second_time() -> Result<()> {
    let (dir, store) = common::store("export-import-twice")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.upsert_wellness_daily(&common::wellness("2026-05-10"))?;

    let target = dir.file("export");
    export::export(&store, &target, &common::key()?)?;

    let other = Store::open_and_migrate(&dir.file("other.db"), &common::key()?)?;
    export::import(&other, &target, &common::key()?)?;
    export::import(&other, &target, &common::key()?)?;

    assert_eq!(other.count_activities()?, 1, "the run was imported twice");
    assert_eq!(other.count_wellness_daily()?, 1);

    drop(dir);
    Ok(())
}

#[test]
fn an_import_refuses_an_export_of_another_schema_version() -> Result<()> {
    let (dir, store) = common::store("export-refuse")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;

    let target = dir.file("export");
    export::export(&store, &target, &common::key()?)?;

    // The manifest says a schema this build does not read.
    let path = target.join("manifest.json");
    let text = common::read_text(&path)?;
    let text = text.replace("\"schema_version\": 1", "\"schema_version\": 99");
    common::write(&path, &text)?;

    let other = Store::open_and_migrate(&dir.file("other.db"), &common::key()?)?;
    let refused = export::import(&other, &target, &common::key()?);
    let error = match refused {
        Ok(_) => {
            return Err(stride_store::StoreError::Invariant(
                "the import accepted another schema version".to_string(),
            ))
        }
        Err(error) => error,
    };
    let text = error.to_string();
    assert!(
        text.contains("99"),
        "the error does not say the version: {text}"
    );
    assert!(
        text.contains("1"),
        "the error does not say the version: {text}"
    );

    // Nothing was imported.
    assert_eq!(other.count_activities()?, 0);

    drop(dir);
    Ok(())
}

#[test]
fn an_import_refuses_a_manifest_it_does_not_know() -> Result<()> {
    let (dir, store) = common::store("export-unknown")?;
    let target = dir.file("export");
    export::export(&store, &target, &common::key()?)?;

    let path = target.join("manifest.json");
    let text = common::read_text(&path)?;
    common::write(&path, &text.replace("stride-export", "something-else"))?;

    let other = Store::open_and_migrate(&dir.file("other.db"), &common::key()?)?;
    assert!(export::import(&other, &target, &common::key()?).is_err());
    drop(dir);
    Ok(())
}

#[test]
fn the_format_of_a_file_is_told_from_its_header() -> Result<()> {
    let dir = common::scratch("export-format")?;

    let missing = dir.file("absent.db");
    assert_eq!(export::detect_format(&missing)?, Format::Missing);

    let plain = dir.file("plain.db");
    common::write(&plain, "SQLite format 3\0 and then some")?;
    assert_eq!(export::detect_format(&plain)?, Format::Plaintext);

    let encrypted = dir.file("encrypted.db");
    {
        let store = Store::open_and_migrate(&encrypted, &common::key()?)?;
        store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    }
    assert_eq!(export::detect_format(&encrypted)?, Format::Encrypted);

    Ok(())
}

#[test]
fn a_plain_database_is_converted_into_an_encrypted_one() -> Result<()> {
    let dir = common::scratch("export-convert")?;
    let path = dir.file("stride.db");

    // A plain database with the frozen schema, written by hand.
    {
        let connection = rusqlite::Connection::open(&path)?;
        for statement in stride_store::schema::all_statements() {
            connection.execute_batch(statement)?;
        }
        connection.execute(
            "INSERT INTO activities (
                source, external_id, name, sport_type, start_time, local_date,
                distance_m, moving_time_s, elapsed_time_s, elevation_gain_m,
                avg_hr, max_hr, fit_blob, fit_sha256, created_at, updated_at
             ) VALUES ('garmin_personal', 'run-1', 'a run', 'running',
                '2026-05-10T06:30:15.000000Z', '2026-05-10', 10000.0, 3000.0,
                3100.0, 42.0, 148.0, 171.0, x'0102', 'a hash',
                '2026-05-10T06:30:15.000000Z', '2026-05-10T06:30:15.000000Z')",
            [],
        )?;
    }

    assert_eq!(export::detect_format(&path)?, Format::Plaintext);

    let report = export::convert_plaintext_to_encrypted(&path, &common::key()?)?;

    assert_eq!(report.tables.len(), 7);
    assert_eq!(report.rows, 1);
    assert!(
        report.schema_matches,
        "the plain file held the frozen schema"
    );
    assert!(
        report.drift.is_empty(),
        "the drift is not empty: {:?}",
        report.drift
    );

    // The file is encrypted now, and it opens with the key.
    assert_eq!(export::detect_format(&path)?, Format::Encrypted);
    let store = Store::open(&path, &common::key()?)?;
    assert_eq!(store.count_activities()?, 1);
    let run = store.activity_by_external("garmin_personal", "run-1")?;
    let run = match run {
        Some(run) => run,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the run did not survive the conversion".to_string(),
            ))
        }
    };
    assert_eq!(run.distance_m, 10_000.0);
    assert_eq!(run.fit_blob, vec![1, 2]);

    // The plain copy is kept beside it.
    let backup = dir.file("stride.db.plaintext-backup");
    assert!(backup.is_file(), "the plain copy is not there");
    assert_eq!(export::detect_format(&backup)?, Format::Plaintext);

    Ok(())
}

#[test]
fn a_converted_file_does_not_open_with_another_key() -> Result<()> {
    let dir = common::scratch("export-convert-key")?;
    let path = dir.file("stride.db");
    {
        let connection = rusqlite::Connection::open(&path)?;
        for statement in stride_store::schema::all_statements() {
            connection.execute_batch(statement)?;
        }
    }
    export::convert_plaintext_to_encrypted(&path, &common::key()?)?;

    let wrong = DatabaseKey::from_text("a different key entirely")?;
    assert!(Store::open(&path, &wrong).is_err());
    Ok(())
}

#[test]
fn an_encrypted_file_is_not_converted_again() -> Result<()> {
    let dir = common::scratch("export-convert-twice")?;
    let path = dir.file("stride.db");
    {
        let store = Store::open_and_migrate(&path, &common::key()?)?;
        store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    }
    let refused = export::convert_plaintext_to_encrypted(&path, &common::key()?);
    assert!(refused.is_err(), "an encrypted file was converted again");
    Ok(())
}
