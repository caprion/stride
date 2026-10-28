//! `stride-fit` — the FIT parser and the run / walk / stand segmentation.
//!
//! The crate reads a Garmin FIT file and produces the canonical activity the
//! rest of Stride works with: the summary, the segments, the streams and the
//! laps rebuilt over the running samples only.
//!
//! It is a port of `backend/app/ingest/fit_importer.py`, and the golden files
//! under `backend/tests/golden/golden/` are the proof that it agrees with the
//! reference implementation.
//!
//! ```no_run
//! use stride_fit::parse_fit;
//!
//! let bytes = std::fs::read("activity.fit").unwrap_or_default();
//! match parse_fit(&bytes, "activity.fit") {
//!     Ok(activity) => println!("{} m", activity.distance_m),
//!     Err(error) => eprintln!("could not read the file: {error}"),
//! }
//! ```
//!
//! The three Garmin quirks the crate exists to get right:
//!
//! 1. **Running cadence is stored halved.** A watch recording 170 spm writes
//!    85. The correction applies to running sports only — a cycling cadence
//!    of 90 rpm stays 90.
//! 2. **Positions are semicircles**, not degrees:
//!    `degrees = round(semicircles * (180 / 2^31), 7)`.
//! 3. **Lap speed is sometimes absent.** It is never invented out of thin
//!    air: the laps are rebuilt from the running samples, and a lap with no
//!    running in it has no speed.
//!
//! Nothing in this crate panics: a malformed file is an [`FitError`].

#![deny(missing_docs)]

mod decoder;
mod error;
mod import;
mod profile;
mod round;

pub use error::FitError;
pub use import::{
    classify_sample, parse_fit, running_cadence, Lap, ParsedActivity, Sample, Segment,
    SegmentKind, Stream, MAX_FIT_INPUT_BYTES, MAX_FIT_RECORDS,
};
