//! `stride-store` — the encrypted SQLite database.
//!
//! The only crate that touches the database. The schema is **frozen** in
//! `docs/contract/schema.sql`: seven tables, their columns, their types and
//! their indices. This crate implements that schema exactly, and
//! `tests/schema_contract.rs` proves it, by comparing SQLite's own
//! `sqlite_master` output with the contract's.
//!
//! ```no_run
//! use std::path::Path;
//! use stride_store::{DatabaseKey, Store};
//!
//! # fn main() -> Result<(), stride_store::StoreError> {
//! // The key comes from the caller: the Android Keystore on the phone, the
//! // Key Vault on the desktop. It is never written down and never read back.
//! let key = DatabaseKey::from_text("the key the caller holds")?;
//! let store = Store::open_and_migrate(Path::new("stride.db"), &key)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## The parts
//!
//! | module | what it is |
//! |---|---|
//! | [`store`] | the database: open it, migrate it, hand it out |
//! | [`key`] | the key: it goes in, and it does not come out |
//! | [`schema`] | the DDL, the indexes and the migrations |
//! | [`repos`] | one repository per table |
//! | [`sync`] | the cursor rule: it advances only after a successful parse |
//! | [`export`] | export, import, and the plaintext → SQLCipher conversion |
//! | [`model`] | the rows |
//! | [`hash`] | SHA-256, for the FIT blob's content hash |
//! | [`logging`] | the store's log, and the sink a test reads |
//!
//! ## The rules
//!
//! * **The key never comes back out.** There is no accessor for it, no
//!   `Display` for it, and the `Debug` output is a fixed phrase.
//! * **Every query is parameterised.** There is no string-built SQL in this
//!   tree, and `scripts/check-contract-greps.sh` fails the build if that ever
//!   stops being true.
//! * **The FIT file is the source of truth.** It is stored, byte for byte,
//!   with its SHA-256; every derived series is recomputable from it.
//! * **No score, no verdict, no grade.** The product never shows a device's
//!   opinion of the athlete, so none is stored.

pub mod datetimes;
pub mod error;
pub mod export;
pub mod hash;
pub mod key;
pub mod logging;
pub mod model;
pub mod repos;
pub mod schema;
pub mod scratch;
pub mod store;
pub mod sync;

pub use error::{Result, StoreError};
pub use export::{ConversionReport, ExportManifest, ImportReport};
pub use key::DatabaseKey;
pub use model::{
    Activity, ActivitySamples, ActivitySummary, NewActivity, NewActivitySamples, NewNote,
    NewPlanBlock, NewPlanSession, Note, PlanBlock, PlanSession, SyncState, WellnessDaily,
};
pub use repos::activity::{FeedCursor, FeedPage};
pub use schema::SCHEMA_VERSION;
pub use store::Store;
pub use sync::SyncCursor;
