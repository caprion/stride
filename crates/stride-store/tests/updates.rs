//! The run update's table: what it keeps, and where it travels.
//!
//! An update that cannot be re-checked is just a sentence, so the table
//! keeps the writer, the reviewer, the hash of the payload the model was
//! given, the draft, the checks and their findings, and the verdict. And it
//! travels like every other table: the export, the import, and the delete.

mod common;

use stride_store::export;
use stride_store::{NewRunUpdate, Result, Store};

/// An update to write, for a run.
fn update(activity_id: i64, draft: &str) -> NewRunUpdate {
    NewRunUpdate {
        activity_id,
        writer: "the writer".to_string(),
        reviewer: "the reviewer".to_string(),
        payload_hash: "a-hash-of-the-payload".to_string(),
        draft: draft.to_string(),
        checks: "{\"findings\":[]}".to_string(),
        verdict: "ship".to_string(),
        created_at: common::instant(),
    }
}

/// A store with one run in it, and the run's id.
fn store_with_run(prefix: &str) -> Result<(stride_store::scratch::ScratchDir, Store, i64)> {
    let (dir, store) = common::store(prefix)?;
    let run = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    Ok((dir, store, run))
}

#[test]
fn an_update_is_kept_with_its_checks_and_its_verdict() -> Result<()> {
    let (dir, store, run) = store_with_run("update-write")?;

    store.write_run_update(&update(run, "The run was 10.0 km in 50 m."))?;

    let stored = store.run_update(run)?;
    let stored = match stored {
        Some(stored) => stored,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the update was not kept".to_string(),
            ))
        }
    };
    assert_eq!(stored.activity_id, run);
    assert_eq!(stored.writer, "the writer");
    assert_eq!(stored.reviewer, "the reviewer");
    assert_eq!(stored.payload_hash, "a-hash-of-the-payload");
    assert_eq!(stored.draft, "The run was 10.0 km in 50 m.");
    assert_eq!(stored.checks, "{\"findings\":[]}");
    assert_eq!(stored.verdict, "ship");
    assert_eq!(store.count_run_updates()?, 1);

    drop(dir);
    Ok(())
}

#[test]
fn a_run_has_one_update_and_a_newer_one_replaces_it() -> Result<()> {
    let (dir, store, run) = store_with_run("update-replace")?;

    store.write_run_update(&update(run, "The first reading."))?;
    store.write_run_update(&update(run, "The second reading."))?;

    assert_eq!(store.count_run_updates()?, 1);
    let stored = store.run_update(run)?;
    let stored = match stored {
        Some(stored) => stored,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the update was not kept".to_string(),
            ))
        }
    };
    assert_eq!(stored.draft, "The second reading.");

    drop(dir);
    Ok(())
}

#[test]
fn a_run_without_an_update_has_none() -> Result<()> {
    let (dir, store, run) = store_with_run("update-none")?;
    assert!(store.run_update(run)?.is_none());
    drop(dir);
    Ok(())
}

#[test]
fn the_update_travels_with_the_export_and_the_import() -> Result<()> {
    let (dir, store, run) = store_with_run("update-export")?;
    store.write_run_update(&update(run, "The run was 10.0 km in 50 m."))?;
    store.write_update_settings(&stride_store::UpdateSettings {
        enabled: true,
        allow_remote: false,
        writer_model: "the writer".to_string(),
        writer_family: "one family".to_string(),
        writer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        reviewer_model: "the reviewer".to_string(),
        reviewer_family: "another family".to_string(),
        reviewer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        key_env: "STRIDE_MODEL_KEY".to_string(),
        updated_at: common::instant(),
    })?;

    let target = dir.file("export");
    let manifest = export::export(&store, &target, &common::key()?)?;
    assert_eq!(manifest.tables.get("run_updates"), Some(&1));
    assert_eq!(manifest.tables.get("update_settings"), Some(&1));

    // A second database, empty, reads the export back.
    let other = Store::open_and_migrate(&dir.file("other.db"), &common::key()?)?;
    export::import(&other, &target, &common::key()?)?;

    let stored = other.run_update(run)?;
    let stored = match stored {
        Some(stored) => stored,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the update was not imported".to_string(),
            ))
        }
    };
    assert_eq!(stored.draft, "The run was 10.0 km in 50 m.");
    assert_eq!(stored.writer, "the writer");
    assert_eq!(stored.payload_hash, "a-hash-of-the-payload");

    let settings = other.update_settings(common::instant())?;
    assert!(settings.enabled);
    assert!(!settings.allow_remote);
    assert_eq!(settings.writer_model, "the writer");
    assert_eq!(settings.reviewer_family, "another family");

    drop(dir);
    Ok(())
}

#[test]
fn the_delete_takes_the_update_with_it() -> Result<()> {
    let (dir, store, run) = store_with_run("update-delete")?;
    store.write_run_update(&update(run, "The run was 10.0 km in 50 m."))?;
    store.write_update_settings(&stride_store::UpdateSettings::defaults(common::instant()))?;
    assert_eq!(store.count_run_updates()?, 1);

    store.delete_everything()?;

    assert_eq!(store.count_run_updates()?, 0);
    assert!(store.run_update(run)?.is_none());
    // The settings are the athlete's own configuration, and the delete
    // takes them too: nothing of this feature is left behind.
    let settings = store.update_settings(common::instant())?;
    assert!(!settings.enabled);

    drop(dir);
    Ok(())
}

#[test]
fn the_settings_are_off_until_the_athlete_turns_them_on() -> Result<()> {
    let (dir, store) = common::store("update-settings-off")?;
    let settings = store.update_settings(common::instant())?;
    assert!(!settings.enabled, "the feature is off by default");
    assert!(
        !settings.allow_remote,
        "nothing leaves the machine by default"
    );
    assert_eq!(settings.writer_model, "");
    assert_eq!(settings.reviewer_model, "");
    assert_eq!(settings.key_env, "STRIDE_MODEL_KEY");
    drop(dir);
    Ok(())
}
