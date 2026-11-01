//! Garmin Personal API ingestion and FIT decoding for Stride.
//!
//! This crate is the only part of Stride that talks to Garmin and the only
//! part that reads binary activity files. It deliberately stops at measured
//! primitives: deciding what a number means belongs to `stride-core`, and
//! persistence belongs to `stride-store`.

#![deny(missing_docs)]

pub mod fit;
pub mod garmin;
pub mod segment;

pub use fit::{
    decode_fit, manufacturer_name, normalise_sport, parse_fit, running_cadence, sport_name,
    FitActivity, FitError, FitLap, FitSample, GeoPoints, Stream, MAX_FIT_INPUT_BYTES,
    MAX_FIT_RECORDS,
};
pub use garmin::{
    extract_fit, GarminClient, GarminError, HttpMethod, HttpRequest, HttpResponse, HttpTransport,
    ReqwestTransport, TransportError, ACTIVITIES_PATH, ACTIVITY_PATH, BODY_BATTERY_PATH,
    DAILY_HEART_RATE_PATH, DEFAULT_BASE_URL, DOWNLOAD_PATH, HEART_RATE_ZONES_PATH, HRV_DAILY_PATH,
    HRV_RANGE_PATH, MAX_FIT_MEMBER_BYTES, MAX_RESPONSE_BYTES, MAX_ZIP_ENTRIES, PROFILE_PATH,
    SLEEP_DAILY_PATH, SLEEP_SUMMARY_PATH, STEPS_PATH, STRESS_PATH, TOKEN_PATH, USER_SUMMARY_PATH,
};
pub use segment::{
    classify_sample, segment_samples, ClassifiedSample, Sample, Segment, SegmentKind,
    SegmentTotals, Segmentation,
};
