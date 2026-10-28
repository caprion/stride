//! Getting the original FIT file out of Garmin's download container.
//!
//! Garmin serves the original FIT inside a zip archive. The reference
//! implementation (`backend/app/ingest/garmin_personal.py`) opens it, refuses
//! archives with more than eight entries, refuses any entry with an unsafe
//! name or an implausible size, and returns the first `.fit` member. A few
//! activities come back as a bare FIT file instead, and those are taken as
//! they are.

use std::io::{Cursor, Read};

use crate::error::GarminError;

/// The most entries the download archive may contain.
pub const MAX_ZIP_ENTRIES: usize = 8;

/// The most bytes a single archived member may expand to.
pub const MAX_FIT_MEMBER_BYTES: u64 = 20 * 1024 * 1024;

/// The most bytes the compressed `.fit` member may occupy.
pub const MAX_FIT_BYTES: u64 = 5 * 1024 * 1024;

/// The first byte of a FIT file: the header's size field.
const FIT_HEADER_BYTE: u8 = 0x0e;

/// Extracts the FIT file from Garmin's download for an activity.
///
/// The `activity_id` only appears in the error, to say which download failed.
pub fn extract_fit(blob: &[u8], activity_id: &str) -> Result<Vec<u8>, GarminError> {
    if blob.starts_with(b"PK") {
        return extract_from_zip(blob, activity_id);
    }
    if blob.first() == Some(&FIT_HEADER_BYTE) && blob.len() as u64 <= MAX_FIT_MEMBER_BYTES {
        return Ok(blob.to_vec());
    }
    Err(GarminError::FitUnavailable {
        activity_id: activity_id.to_owned(),
    })
}

fn extract_from_zip(blob: &[u8], activity_id: &str) -> Result<Vec<u8>, GarminError> {
    let unavailable = || GarminError::FitUnavailable {
        activity_id: activity_id.to_owned(),
    };
    let mut archive = zip::ZipArchive::new(Cursor::new(blob)).map_err(|_| unavailable())?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(unavailable());
    }
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|_| unavailable())?;
        let name = entry.name().to_owned();
        if unsafe_name(&name) {
            return Err(unavailable());
        }
        if entry.size() > MAX_FIT_MEMBER_BYTES {
            return Err(unavailable());
        }
        if !name.to_ascii_lowercase().ends_with(".fit") {
            continue;
        }
        if entry.compressed_size() > MAX_FIT_BYTES {
            return Err(unavailable());
        }
        let mut buffer = Vec::new();
        entry
            .by_ref()
            .take(MAX_FIT_MEMBER_BYTES + 1)
            .read_to_end(&mut buffer)
            .map_err(|_| unavailable())?;
        if buffer.len() as u64 > MAX_FIT_MEMBER_BYTES {
            return Err(unavailable());
        }
        return Ok(buffer);
    }
    Err(unavailable())
}

/// True when an archive member's name tries to escape the archive.
///
/// The reference implementation rejects absolute paths, `..` components and
/// Windows drive letters; it rejects the whole archive, not just the member.
fn unsafe_name(name: &str) -> bool {
    if name.starts_with('/') || name.starts_with('\\') {
        return true;
    }
    let normalized = name.replace('\\', "/");
    if normalized.split('/').any(|part| part == "..") {
        return true;
    }
    let bytes = name.as_bytes();
    bytes.len() > 1 && bytes[1] == b':'
}
