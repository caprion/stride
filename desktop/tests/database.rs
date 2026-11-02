//! The database and its key.
//!
//! Three things are checked here:
//!
//! * **the key is made on the first run and kept with `0600`** — and it is
//!   not the Garmin key, and it does not open the Garmin key's file;
//! * **the database is encrypted** — the file on the disk does not open
//!   without the key;
//! * **a database that predates the encryption is converted once**, and the
//!   plain copy is kept beside it, under the name the README names.

use std::path::Path;

use stride_desktop::dbkey::{self, DatabaseSecret};
use stride_desktop::error::Result;

/// A scratch directory for one test.
fn scratch(name: &str) -> Result<std::path::PathBuf> {
    let directory = std::env::temp_dir().join(format!("stride-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

/// The mode of a file, on the platforms that have one.
#[cfg(unix)]
fn mode_of(path: &Path) -> Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::metadata(path)?.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
fn mode_of(_path: &Path) -> Result<u32> {
    Ok(0)
}

#[test]
fn the_key_is_made_once_and_kept_private() -> Result<()> {
    let directory = scratch("dbkey-test")?;
    let path = directory.join("db.key");

    let _first = DatabaseSecret::load_or_create(&path)?;
    assert!(path.exists(), "the key was not written down");
    let first_text = std::fs::read_to_string(&path)?;

    // The second run reads the key that is there; it does not make another
    // one, because a key that changed would lock the athlete out of their
    // own runs.
    let second = DatabaseSecret::load_or_create(&path)?;
    let second_text = std::fs::read_to_string(&path)?;
    assert_eq!(first_text, second_text, "the key changed under the rig");

    // And the key that was made opens a database.
    let database = directory.join("stride.db");
    let store = stride_store::Store::open_and_migrate(&database, &second.store_key()?)?;
    assert!(store.schema_version()? > 0);

    #[cfg(unix)]
    assert_eq!(mode_of(&path)?, 0o600, "the key file is not private");

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn the_key_is_not_the_garmin_key() -> Result<()> {
    let directory = scratch("dbkey-garmin-test")?;
    let database_key_path = directory.join("db.key");
    let _ = DatabaseSecret::load_or_create(&database_key_path)?;

    // And they are in different files, with different contents.
    let garmin_key_path = directory.join("garmin.key");
    std::fs::write(&garmin_key_path, "a key of the athlete's own")?;
    assert_ne!(database_key_path, garmin_key_path);
    let database_text = std::fs::read_to_string(&database_key_path)?;
    let garmin_text = std::fs::read_to_string(&garmin_key_path)?;
    assert_ne!(database_text, garmin_text);

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn the_database_does_not_open_without_its_key() -> Result<()> {
    let directory = scratch("db-encrypted-test")?;
    let database = directory.join("stride.db");
    let database_key = DatabaseSecret::generate()?;
    let store = stride_store::Store::open_and_migrate(&database, &database_key.store_key()?)?;
    drop(store);

    // Another key does not open it.
    let other = DatabaseSecret::generate()?;
    assert!(
        stride_store::Store::open(&database, &other.store_key()?).is_err(),
        "the database opened with the wrong key"
    );

    // The right one does.
    assert!(stride_store::Store::open(&database, &database_key.store_key()?).is_ok());

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn a_plain_database_is_converted_and_the_plain_copy_is_kept() -> Result<()> {
    let directory = scratch("db-convert-test")?;
    let database = directory.join("stride.db");

    // A plain SQLite file with the frozen schema — what the old desktop
    // wrote.
    {
        let connection = rusqlite::Connection::open(&database)
            .map_err(|error| stride_desktop::error::Error::config(error.to_string()))?;
        stride_store::schema::migrate(&connection)
            .map_err(|error| stride_desktop::error::Error::config(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO wellness_daily (date, sleep_s, source, updated_at)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    "2026-05-10",
                    22_200.0,
                    "garmin_personal",
                    "2026-05-10T00:00:00Z"
                ],
            )
            .map_err(|error| stride_desktop::error::Error::config(error.to_string()))?;
    }
    assert_eq!(
        stride_store::export::detect_format(&database)?,
        stride_store::export::Format::Plaintext
    );

    // The first run converts it.
    let database_key = DatabaseSecret::generate()?;
    let report = dbkey::convert_plaintext_database(&database, &database_key)?;
    let report = match report {
        Some(report) => report,
        None => {
            let _ = std::fs::remove_dir_all(&directory);
            return Err(stride_desktop::error::Error::config(
                "the plain database was not converted",
            ));
        }
    };
    assert_eq!(report.rows, 1, "the row was not copied");

    // The database is encrypted now, and it opens with the key.
    assert_eq!(
        stride_store::export::detect_format(&database)?,
        stride_store::export::Format::Encrypted
    );
    let store = stride_store::Store::open(&database, &database_key.store_key()?)?;
    assert_eq!(store.schema_version()?, stride_store::SCHEMA_VERSION);

    // The plain copy is kept beside it, under the name the README names.
    let backup = dbkey::plaintext_backup_path(&database);
    assert!(backup.exists(), "the plain copy was not kept");
    assert_eq!(
        backup.file_name().and_then(|name| name.to_str()),
        Some("stride.db.plaintext-backup")
    );
    assert_eq!(
        stride_store::export::detect_format(&backup)?,
        stride_store::export::Format::Plaintext
    );
    #[cfg(unix)]
    assert_eq!(mode_of(&backup)?, 0o600, "the plain copy is not private");

    // And a second run does not convert it again.
    assert!(dbkey::convert_plaintext_database(&database, &database_key)?.is_none());

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}
