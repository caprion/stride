//! The repositories.
//!
//! One method per table the API reads, plus the writes the sync needs. No ORM,
//! no clever abstraction: a method per query, and the SQL is right there.

pub mod activity;
pub mod gear;
pub mod helpers;
pub mod physiology;
pub mod plan;
pub mod sync;
