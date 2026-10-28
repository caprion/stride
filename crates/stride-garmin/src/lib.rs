//! The Garmin Connect **Personal API** client.
//!
//! One long-lived Personal API Key, exchanged for a 24-hour bearer token. No
//! developer registration, no OAuth dance, no refresh tokens.
//!
//! ```no_run
//! use stride_garmin::GarminClient;
//!
//! # fn main() -> Result<(), stride_garmin::GarminError> {
//! let client = GarminClient::new("the-athletes-personal-api-key");
//! let profile = client.validate()?;
//! println!("{:?}", profile.athlete_name());
//! # Ok(())
//! # }
//! ```
//!
//! The key is a credential for the athlete's whole Garmin account. It is
//! passed in by the caller, it is never written to disk, it never appears in a
//! URL, and it never appears in an error message or a `Debug` output.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod client;
mod error;
mod fit;
mod types;

pub use client::{
    GarminClient, ACTIVITIES_PATH, ACTIVITY_PATH, DEFAULT_BASE_URL, DOWNLOAD_PATH,
    HEART_RATE_ZONES_PATH, PROFILE_PATH, TOKEN_PATH,
};
pub use error::GarminError;
pub use fit::{extract_fit, MAX_FIT_BYTES, MAX_FIT_MEMBER_BYTES, MAX_ZIP_ENTRIES};
pub use types::{ActivityDetail, ActivitySummary, Profile, Zone, ZoneModel};
