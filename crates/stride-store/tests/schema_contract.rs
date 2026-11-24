//! The schema is frozen, and this is the proof.
//!
//! `docs/contract/schema.sql` is the contract. The store creates a schema,
//! and the two must be the same schema — not "the same shape", the same
//! schema, table for table, column for column, index for index.
//!
//! The assertion is a golden one. The reference database is built **from the
//! contract file itself**, and SQLite's own `sqlite_master` output is
//! compared with the output of a database the store created. A change to
//! either side without the other fails the build.
//!
//! The comments are stripped from both sides before the comparison: the
//! contract's comments are part of its text, but a comment is not a schema,
//! and a comment may hold a semicolon that would break a naive split.

mod common;

use std::collections::BTreeMap;

use rusqlite::Connection;
use stride_store::schema;
use stride_store::{Result, Store};

/// One object of a schema: what it is, what it is called, and its text.
type Object = (String, String, String);

/// The schema of a database, as SQLite records it.
fn dump(connection: &Connection) -> Result<Vec<Object>> {
    let mut statement = connection
        .prepare(
            "SELECT type, name, sql FROM sqlite_master
             WHERE name NOT LIKE 'sqlite_%'
             ORDER BY type ASC, name ASC",
        )
        .map_err(|error| stride_store::StoreError::query("sqlite_master", error))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| stride_store::StoreError::query("sqlite_master", error))?;

    let mut out = Vec::new();
    for row in rows {
        let (kind, name, sql) =
            row.map_err(|error| stride_store::StoreError::query("sqlite_master", error))?;
        out.push((kind, name, common::normalise(&common::strip_comments(&sql))));
    }
    Ok(out)
}

/// The schema of a database, by name.
fn by_name(objects: &[Object]) -> BTreeMap<String, (String, String)> {
    objects
        .iter()
        .map(|(kind, name, sql)| (name.clone(), (kind.clone(), sql.clone())))
        .collect()
}

#[test]
fn the_store_creates_the_contract_s_schema() -> Result<()> {
    let contract = std::fs::read_to_string(common::contract_schema_path())
        .map_err(|error| stride_store::StoreError::io("the contract", error))?;

    // The reference: a database built from the contract file itself.
    let reference = Connection::open_in_memory()
        .map_err(|error| stride_store::StoreError::query("open in memory", error))?;
    for statement in common::statements(&contract) {
        reference
            .execute_batch(&statement)
            .map_err(|error| stride_store::StoreError::query(&statement, error))?;
    }

    // The store's: a database the store created.
    let (dir, store) = common::store("schema-contract")?;
    let mine = store.with_connection(dump)?;

    let reference = dump(&reference)?;

    assert_eq!(
        by_name(&mine).len(),
        by_name(&reference).len(),
        "the store made {} objects and the contract makes {}",
        by_name(&mine).len(),
        by_name(&reference).len()
    );

    for (name, (kind, sql)) in by_name(&reference) {
        let found = by_name(&mine)
            .get(&name)
            .ok_or_else(|| {
                stride_store::StoreError::Invariant(format!(
                    "the store did not create {kind} {name}"
                ))
            })?
            .clone();
        assert_eq!(found.0, kind, "the kind of {name} is different");
        assert_eq!(found.1, sql, "the text of {kind} {name} is different");
    }

    // The objects are the ones the contract names, and only those.
    let mut names: Vec<String> = by_name(&mine).keys().cloned().collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "activities".to_string(),
            "activity_observations".to_string(),
            "activity_samples".to_string(),
            "adopted_plans".to_string(),
            "adopted_sessions".to_string(),
            "deleted_garmin_activities".to_string(),
            "hr_zone_models".to_string(),
            "ix_activities_local_date".to_string(),
            "ix_activities_sport".to_string(),
            "ix_activities_start".to_string(),
            "ix_activity_observations_activity".to_string(),
            "ix_adopted_plans_race_date".to_string(),
            "ix_adopted_plans_start_date".to_string(),
            "ix_adopted_sessions_plan_date".to_string(),
            "ix_notes_activity_id".to_string(),
            "ix_plan_blocks_start_date".to_string(),
            "ix_plan_sessions_block_date".to_string(),
            "ix_plan_sessions_date".to_string(),
            "ix_race_observations_race_date".to_string(),
            "ix_run_updates_activity".to_string(),
            "ix_threshold_observations_entered_on".to_string(),
            "ix_wellness_observations_date".to_string(),
            "notes".to_string(),
            "plan_blocks".to_string(),
            "plan_sessions".to_string(),
            "race_observations".to_string(),
            "run_updates".to_string(),
            "sync_state".to_string(),
            "threshold_observations".to_string(),
            "update_settings".to_string(),
            "wellness_daily".to_string(),
            "wellness_observations".to_string(),
        ]
    );

    drop(dir);
    Ok(())
}

#[test]
fn the_ddl_in_the_crate_is_the_contract_s_ddl() -> Result<()> {
    let contract = std::fs::read_to_string(common::contract_schema_path())
        .map_err(|error| stride_store::StoreError::io("the contract", error))?;
    let expected = common::statements(&contract);

    let mine: Vec<String> = schema::all_statements()
        .iter()
        .map(|statement| common::normalise(&common::strip_comments(statement)))
        .filter(|statement| !statement.is_empty())
        .collect();

    assert_eq!(
        mine.len(),
        expected.len(),
        "the crate has {} statements and the contract has {}",
        mine.len(),
        expected.len()
    );
    for (index, statement) in expected.iter().enumerate() {
        assert_eq!(
            mine[index], *statement,
            "the statement {index} is different from the contract's"
        );
    }
    Ok(())
}

#[test]
fn the_tables_are_created_in_the_contract_s_order() -> Result<()> {
    let (dir, store) = common::store("schema-order")?;
    let names = store.with_connection(schema::table_names)?;
    assert_eq!(
        names,
        vec![
            "activities".to_string(),
            "activity_samples".to_string(),
            "wellness_daily".to_string(),
            "plan_blocks".to_string(),
            "plan_sessions".to_string(),
            "notes".to_string(),
            "sync_state".to_string(),
            "deleted_garmin_activities".to_string(),
            "hr_zone_models".to_string(),
            "wellness_observations".to_string(),
            "run_updates".to_string(),
            "update_settings".to_string(),
            "race_observations".to_string(),
            "threshold_observations".to_string(),
            "activity_observations".to_string(),
            "adopted_plans".to_string(),
            "adopted_sessions".to_string(),
        ]
    );
    drop(dir);
    Ok(())
}

#[test]
fn the_indexes_are_created_in_the_contract_s_order() -> Result<()> {
    let (dir, store) = common::store("schema-indexes")?;
    let names = store.with_connection(schema::index_names)?;
    assert_eq!(
        names,
        vec![
            "ix_activities_local_date".to_string(),
            "ix_activities_sport".to_string(),
            "ix_activities_start".to_string(),
            "ix_plan_blocks_start_date".to_string(),
            "ix_plan_sessions_block_date".to_string(),
            "ix_plan_sessions_date".to_string(),
            "ix_notes_activity_id".to_string(),
            "ix_wellness_observations_date".to_string(),
            "ix_run_updates_activity".to_string(),
            "ix_race_observations_race_date".to_string(),
            "ix_threshold_observations_entered_on".to_string(),
            "ix_activity_observations_activity".to_string(),
            "ix_adopted_plans_start_date".to_string(),
            "ix_adopted_plans_race_date".to_string(),
            "ix_adopted_sessions_plan_date".to_string(),
        ]
    );
    drop(dir);
    Ok(())
}

#[test]
fn the_migration_is_idempotent() -> Result<()> {
    let (dir, store) = common::store("schema-idempotent")?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);

    // Twice more: nothing changes and nothing fails.
    store.migrate()?;
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);

    let tables = store.with_connection(schema::table_names)?;
    assert_eq!(tables.len(), 17);
    drop(dir);
    Ok(())
}

#[test]
fn a_version_two_database_migrates_to_version_three() -> Result<()> {
    let dir = common::scratch("schema-migrate-v2-v3")?;
    let path = dir.file("stride.db");
    let store = Store::open(&path, &common::key()?)?;

    // Build the schema as it was at version 2, by running the first two
    // steps of the ladder and setting the version the store would have set.
    store.with_connection(|conn| {
        for migration in schema::migrations()
            .into_iter()
            .take_while(|step| step.version <= 2)
        {
            for statement in migration.statements {
                conn.execute_batch(statement)
                    .map_err(|error| stride_store::StoreError::migration(statement, error))?;
            }
            conn.pragma_update(None, "user_version", migration.version)
                .map_err(|error| stride_store::StoreError::migration("user_version", error))?;
        }
        Ok(())
    })?;
    assert_eq!(store.schema_version()?, 2);

    // One migrate() brings it to the current version, with the new table.
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    let tables = store.with_connection(schema::table_names)?;
    assert!(tables.iter().any(|table| table == "hr_zone_models"));
    assert!(tables.iter().any(|table| table == "wellness_observations"));
    drop(dir);
    Ok(())
}

#[test]
fn a_version_three_database_migrates_to_version_four() -> Result<()> {
    let dir = common::scratch("schema-migrate-v3-v4")?;
    let path = dir.file("stride.db");
    let store = Store::open(&path, &common::key()?)?;

    // Build the schema as it was at version 3.
    store.with_connection(|conn| {
        for migration in schema::migrations()
            .into_iter()
            .take_while(|step| step.version <= 3)
        {
            for statement in migration.statements {
                conn.execute_batch(statement)
                    .map_err(|error| stride_store::StoreError::migration(statement, error))?;
            }
            conn.pragma_update(None, "user_version", migration.version)
                .map_err(|error| stride_store::StoreError::migration("user_version", error))?;
        }
        Ok(())
    })?;
    assert_eq!(store.schema_version()?, 3);
    let tables = store.with_connection(schema::table_names)?;
    assert!(!tables.iter().any(|table| table == "wellness_observations"));

    // One migrate() adds the observations table and its index.
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    let tables = store.with_connection(schema::table_names)?;
    assert!(tables.iter().any(|table| table == "wellness_observations"));
    let indexes = store.with_connection(schema::index_names)?;
    assert!(indexes
        .iter()
        .any(|index| index == "ix_wellness_observations_date"));
    drop(dir);
    Ok(())
}

#[test]
fn a_version_four_database_migrates_to_version_five() -> Result<()> {
    let dir = common::scratch("schema-migrate-v4-v5")?;
    let path = dir.file("stride.db");
    let store = Store::open(&path, &common::key()?)?;

    // Build the schema as it was at version 4.
    store.with_connection(|conn| {
        for migration in schema::migrations()
            .into_iter()
            .take_while(|step| step.version <= 4)
        {
            for statement in migration.statements {
                conn.execute_batch(statement)
                    .map_err(|error| stride_store::StoreError::migration(statement, error))?;
            }
            conn.pragma_update(None, "user_version", migration.version)
                .map_err(|error| stride_store::StoreError::migration("user_version", error))?;
        }
        Ok(())
    })?;
    assert_eq!(store.schema_version()?, 4);
    let tables = store.with_connection(schema::table_names)?;
    assert!(!tables.iter().any(|table| table == "run_updates"));

    // One migrate() adds the update's table, the settings' table and the
    // index.
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    let tables = store.with_connection(schema::table_names)?;
    assert!(tables.iter().any(|table| table == "run_updates"));
    assert!(tables.iter().any(|table| table == "update_settings"));
    let indexes = store.with_connection(schema::index_names)?;
    assert!(indexes
        .iter()
        .any(|index| index == "ix_run_updates_activity"));
    drop(dir);
    Ok(())
}

#[test]
fn a_database_opened_again_keeps_its_schema_version() -> Result<()> {
    let dir = common::scratch("schema-reopen")?;
    let path = dir.file("stride.db");
    {
        let store = Store::open_and_migrate(&path, &common::key()?)?;
        assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    }
    {
        let store = Store::open_and_migrate(&path, &common::key()?)?;
        assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
        let tables = store.with_connection(schema::table_names)?;
        assert_eq!(tables.len(), 17);
    }
    Ok(())
}

#[test]
fn a_version_five_database_migrates_to_version_six() -> Result<()> {
    let dir = common::scratch("schema-migrate-v5-v6")?;
    let path = dir.file("stride.db");
    let store = Store::open(&path, &common::key()?)?;

    // Build the schema as it was at version 5.
    store.with_connection(|conn| {
        for migration in schema::migrations()
            .into_iter()
            .take_while(|step| step.version <= 5)
        {
            for statement in migration.statements {
                conn.execute_batch(statement)
                    .map_err(|error| stride_store::StoreError::migration(statement, error))?;
            }
            conn.pragma_update(None, "user_version", migration.version)
                .map_err(|error| stride_store::StoreError::migration("user_version", error))?;
        }
        Ok(())
    })?;
    assert_eq!(store.schema_version()?, 5);
    let tables = store.with_connection(schema::table_names)?;
    assert!(!tables.iter().any(|table| table == "race_observations"));
    assert!(!tables.iter().any(|table| table == "threshold_observations"));
    assert!(!tables.iter().any(|table| table == "activity_observations"));

    // One migrate() adds the three tables and their indexes.
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    let tables = store.with_connection(schema::table_names)?;
    assert!(tables.iter().any(|table| table == "race_observations"));
    assert!(tables.iter().any(|table| table == "threshold_observations"));
    assert!(tables.iter().any(|table| table == "activity_observations"));
    let indexes = store.with_connection(schema::index_names)?;
    assert!(indexes
        .iter()
        .any(|index| index == "ix_race_observations_race_date"));
    assert!(indexes
        .iter()
        .any(|index| index == "ix_threshold_observations_entered_on"));
    assert!(indexes
        .iter()
        .any(|index| index == "ix_activity_observations_activity"));
    drop(dir);
    Ok(())
}

#[test]
fn a_version_six_database_migrates_to_version_seven() -> Result<()> {
    let dir = common::scratch("schema-migrate-v6-v7")?;
    let path = dir.file("stride.db");
    let store = Store::open(&path, &common::key()?)?;

    // Build the schema as it was at version 6.
    store.with_connection(|conn| {
        for migration in schema::migrations()
            .into_iter()
            .take_while(|step| step.version <= 6)
        {
            for statement in migration.statements {
                conn.execute_batch(statement)
                    .map_err(|error| stride_store::StoreError::migration(statement, error))?;
            }
            conn.pragma_update(None, "user_version", migration.version)
                .map_err(|error| stride_store::StoreError::migration("user_version", error))?;
        }
        Ok(())
    })?;
    assert_eq!(store.schema_version()?, 6);
    let tables = store.with_connection(schema::table_names)?;
    assert!(!tables.iter().any(|table| table == "adopted_plans"));
    assert!(!tables.iter().any(|table| table == "adopted_sessions"));

    // One migrate() adds the two tables and their indexes.
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    let tables = store.with_connection(schema::table_names)?;
    assert!(tables.iter().any(|table| table == "adopted_plans"));
    assert!(tables.iter().any(|table| table == "adopted_sessions"));
    let indexes = store.with_connection(schema::index_names)?;
    assert!(indexes
        .iter()
        .any(|index| index == "ix_adopted_plans_start_date"));
    assert!(indexes
        .iter()
        .any(|index| index == "ix_adopted_plans_race_date"));
    assert!(indexes
        .iter()
        .any(|index| index == "ix_adopted_sessions_plan_date"));

    // And the migration is idempotent: running it again changes nothing.
    store.migrate()?;
    assert_eq!(store.schema_version()?, schema::SCHEMA_VERSION);
    let tables = store.with_connection(schema::table_names)?;
    assert_eq!(tables.len(), 17);
    drop(dir);
    Ok(())
}
