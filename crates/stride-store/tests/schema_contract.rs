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
            "activity_samples".to_string(),
            "ix_activities_local_date".to_string(),
            "ix_activities_sport".to_string(),
            "ix_activities_start".to_string(),
            "ix_notes_activity_id".to_string(),
            "ix_plan_blocks_start_date".to_string(),
            "ix_plan_sessions_block_date".to_string(),
            "ix_plan_sessions_date".to_string(),
            "notes".to_string(),
            "plan_blocks".to_string(),
            "plan_sessions".to_string(),
            "sync_state".to_string(),
            "wellness_daily".to_string(),
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
    assert_eq!(tables.len(), 7);
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
        assert_eq!(tables.len(), 7);
    }
    Ok(())
}
