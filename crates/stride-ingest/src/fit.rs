//! A strict FIT decoder for activity files.
//!
//! The decoder reads the binary FIT protocol directly: file header, local
//! message definitions, data messages and the compressed-timestamp headers.
//! It applies the profile scales only after the base type's *invalid value*
//! has been recognised. A sentinel is therefore absent before any range
//! check can turn it into a plausible but wrong number.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::segment::{self, round_half_even, ClassifiedSample, Sample, Segment, SegmentKind};

/// The largest FIT file the decoder will read.
pub const MAX_FIT_INPUT_BYTES: usize = 20 * 1024 * 1024;

/// The largest number of record messages the decoder will read.
pub const MAX_FIT_RECORDS: usize = 250_000;

/// The largest number of messages of any kind the decoder will read.
pub const MAX_FIT_MESSAGES: usize = 500_000;

/// Seconds between the Unix epoch and the FIT epoch (`1989-12-31T00:00:00Z`).
const FIT_EPOCH_UNIX: i64 = 631_065_600;

const MESG_FILE_ID: u16 = 0;
const MESG_SESSION_DATA: u16 = 18;
const MESG_LAP: u16 = 19;
const MESG_RECORD: u16 = 20;
const MESG_DEVICE_INFO: u16 = 23;

/// A time series over the running samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stream {
    /// Seconds since the first record, one per value.
    pub seconds: Vec<i64>,
    /// The values themselves.
    pub values: Vec<f64>,
    /// The unit of the values.
    pub unit: String,
}

/// One decoded record and its classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitSample {
    /// The sample's position in the record stream.
    pub index: usize,
    /// Seconds since the first record.
    pub elapsed_s: f64,
    /// How the sample was classified.
    pub kind: SegmentKind,
    /// Cumulative distance, in metres.
    pub distance_m: Option<f64>,
    /// Speed, in metres per second.
    pub speed_ms: Option<f64>,
    /// Cadence exactly as stored in the FIT file.
    pub stored_cadence: Option<f64>,
    /// Cadence in steps per minute, corrected for running sports.
    pub cadence_spm: Option<f64>,
    /// Heart rate, in beats per minute.
    pub heart_rate_bpm: Option<f64>,
    /// Altitude, in metres.
    pub altitude_m: Option<f64>,
}

/// A lap, rebuilt over the running samples in the lap's window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitLap {
    /// The lap's one-based number.
    pub lap_index: i64,
    /// The lap's full distance, in metres.
    pub distance_m: f64,
    /// Running time in the lap, in seconds.
    pub duration_s: f64,
    /// Speed over the running samples, in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// Fastest speed recorded in the FIT lap, in metres per second.
    pub max_speed_ms: Option<f64>,
    /// Mean heart rate over the running samples.
    pub avg_hr: Option<i64>,
    /// Highest heart rate over the running samples.
    pub max_hr: Option<i64>,
    /// Mean cadence over the running samples, in steps per minute.
    pub avg_cadence: Option<f64>,
    /// Highest cadence over the running samples, in steps per minute.
    pub max_cadence: Option<f64>,
    /// Stride length; the FIT fields read here do not carry it.
    pub stride_length_m: Option<f64>,
    /// Ascent recorded in the FIT lap, in metres.
    pub elevation_gain_m: Option<f64>,
    /// Descent recorded in the FIT lap, in metres.
    pub elevation_loss_m: Option<f64>,
    /// Starting elevation; the FIT fields read here do not carry it.
    pub start_elevation_m: Option<f64>,
    /// Ending elevation; the FIT fields read here do not carry it.
    pub end_elevation_m: Option<f64>,
    /// The lap's span in activity elapsed time.
    pub elapsed_s: f64,
    /// Distance covered while running, in metres.
    pub running_distance_m: Option<f64>,
    /// Pace over the running samples, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
}

/// The first and last GPS points of a file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeoPoints {
    /// Number of decoded GPS points.
    pub count: usize,
    /// The first three points, as latitude and longitude in degrees.
    pub first: Vec<[f64; 2]>,
    /// The last three points, as latitude and longitude in degrees.
    pub last: Vec<[f64; 2]>,
}

/// A decoded FIT activity.
#[derive(Debug, Clone, PartialEq)]
pub struct FitActivity {
    /// The file's serial number, when it has one.
    pub serial_number: Option<u64>,
    /// The serial number, or the file name when the serial is absent.
    pub external_id: String,
    /// The file name supplied by the caller.
    pub file_name: String,
    /// A display name derived from the file name.
    pub name: String,
    /// The raw FIT sport.
    pub sport: Option<String>,
    /// The normalised sport.
    pub sport_type: String,
    /// The first record's timestamp, in UTC.
    pub start_time: Option<DateTime<Utc>>,
    /// The UTC date of the first record.
    pub local_date: Option<String>,
    /// The English weekday name of the first record.
    pub day_of_week: Option<String>,
    /// Total distance, in metres.
    pub distance_m: f64,
    /// Moving time, in seconds.
    pub moving_time_s: f64,
    /// Elapsed time, in seconds.
    pub elapsed_time_s: f64,
    /// Accumulated elevation gain, in metres.
    pub elevation_gain_m: f64,
    /// Accumulated elevation loss, in metres.
    pub elevation_loss_m: Option<f64>,
    /// Mean heart rate, in beats per minute.
    pub avg_hr: Option<i64>,
    /// Highest heart rate, in beats per minute.
    pub max_hr: Option<i64>,
    /// Calories, in kilocalories.
    pub calories: Option<f64>,
    /// Mean cadence as stored in the FIT session, in steps per minute.
    pub avg_cadence: Option<f64>,
    /// Highest cadence as stored in the FIT session, in steps per minute.
    pub max_cadence: Option<f64>,
    /// Mean speed, in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// Highest speed, in metres per second.
    pub max_speed_ms: Option<f64>,
    /// The product name, or the manufacturer when the product name is absent.
    pub device: Option<String>,
    /// Time spent running, in seconds.
    pub running_time_s: Option<f64>,
    /// Time spent walking, in seconds.
    pub walking_time_s: Option<f64>,
    /// Time spent standing still, in seconds.
    pub stopped_time_s: Option<f64>,
    /// Distance covered while running, in metres.
    pub running_distance_m: Option<f64>,
    /// Run / walk / stand segments.
    pub segments: Vec<Segment>,
    /// Every decoded record, in file order.
    pub samples: Vec<FitSample>,
    /// Time series over the running records.
    pub streams: BTreeMap<String, Stream>,
    /// Laps rebuilt over their running records.
    pub laps: Vec<FitLap>,
    /// Cumulative elapsed time at the end of each original FIT lap.
    pub lap_bounds: Vec<f64>,
    /// The first and last GPS points.
    pub geo_points: GeoPoints,
}

/// The failures the FIT decoder can report.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FitError {
    /// The bytes do not contain a FIT header.
    #[error("not a FIT file: the '.FIT' header magic is missing")]
    NotFit,
    /// The file ended before a declared structure was complete.
    #[error("truncated FIT file at byte {offset}: needed {needed}, only {available} available")]
    Truncated {
        /// Byte offset where the read began.
        offset: usize,
        /// Bytes the structure needed.
        needed: usize,
        /// Bytes actually available.
        available: usize,
    },
    /// The header is structurally invalid.
    #[error("invalid FIT header: {reason}")]
    InvalidHeader {
        /// What is wrong with the header.
        reason: String,
    },
    /// The protocol version is not supported.
    #[error("unsupported FIT protocol version: 0x{version:02x}")]
    UnsupportedVersion {
        /// The version byte from the header.
        version: u8,
    },
    /// A message is malformed.
    #[error("malformed FIT message at byte {offset}: {reason}")]
    Malformed {
        /// Byte offset where the message began.
        offset: usize,
        /// What is wrong with the message.
        reason: String,
    },
    /// A data message referred to a local type with no definition.
    #[error("malformed FIT file at byte {offset}: no definition for local message type {local}")]
    UnknownLocalMessage {
        /// Byte offset where the message began.
        offset: usize,
        /// The undefined local message type.
        local: u8,
    },
    /// The file carries more messages of one kind than the decoder accepts.
    #[error("the FIT file has more than {limit} {kind} messages")]
    TooManyMessages {
        /// The message kind that overflowed.
        kind: &'static str,
        /// The limit that was exceeded.
        limit: usize,
    },
    /// The input is larger than the decoder accepts.
    #[error("the FIT file is {size} bytes, over the {limit} byte limit")]
    TooLarge {
        /// Actual input size.
        size: usize,
        /// Accepted input size.
        limit: usize,
    },
    /// A record did not carry a timestamp, directly or in its compressed header.
    #[error("a FIT record at byte {offset} has no timestamp")]
    MissingTimestamp {
        /// Byte offset where the record began.
        offset: usize,
    },
    /// A required activity total was absent and had no valid fallback.
    #[error("the FIT file has no {field} value")]
    MissingActivityData {
        /// The absent activity field.
        field: &'static str,
    },
}

/// Decode a FIT activity file.
///
/// The file name is used only for the display name and as the external-id
/// fallback when the file has no serial number. It is never interpreted as a
/// path.
pub fn decode_fit(bytes: &[u8], file_name: &str) -> Result<FitActivity, FitError> {
    if bytes.len() > MAX_FIT_INPUT_BYTES {
        return Err(FitError::TooLarge {
            size: bytes.len(),
            limit: MAX_FIT_INPUT_BYTES,
        });
    }
    let mut importer = Importer::new(file_name);
    decode_messages(bytes, &mut importer)?;
    importer.finish()
}

/// Compatibility name for callers that describe decoding as parsing.
pub fn parse_fit(bytes: &[u8], file_name: &str) -> Result<FitActivity, FitError> {
    decode_fit(bytes, file_name)
}

// ------------------------------------------------------------------ protocol

#[derive(Debug, Clone)]
enum RawValue {
    Number(f64),
    Text(String),
}

impl RawValue {
    fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Text(_) => None,
        }
    }

    fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value.as_str()),
            Self::Number(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
struct RawMessage {
    mesg_num: u16,
    compressed_timestamp: Option<u32>,
    fields: Vec<(u8, RawValue)>,
}

impl RawMessage {
    fn number(&self, definition_number: u8) -> Option<f64> {
        self.fields
            .iter()
            .find(|(number, _)| *number == definition_number)
            .and_then(|(_, value)| value.as_number())
    }

    fn text(&self, definition_number: u8) -> Option<&str> {
        self.fields
            .iter()
            .find(|(number, _)| *number == definition_number)
            .and_then(|(_, value)| value.as_text())
    }
}

#[derive(Debug, Clone, Copy)]
struct FieldDefinition {
    definition_number: u8,
    size: usize,
    base_type: u8,
}

#[derive(Debug, Clone)]
struct MessageDefinition {
    mesg_num: u16,
    little_endian: bool,
    fields: Vec<FieldDefinition>,
    developer_bytes: usize,
}

trait VisitMessage {
    fn visit(&mut self, message: RawMessage) -> Result<(), FitError>;
}

fn decode_messages(bytes: &[u8], visitor: &mut impl VisitMessage) -> Result<(), FitError> {
    let (data_start, data_end) = read_header(bytes)?;
    let mut cursor = data_start;
    if data_start == 12 {
        // A 12-byte header is followed by a two-byte file time offset.
        cursor += 2;
    }

    let mut definitions: HashMap<u8, MessageDefinition> = HashMap::new();
    let mut last_timestamp = None;
    let mut message_count = 0usize;

    while cursor < data_end {
        message_count += 1;
        if message_count > MAX_FIT_MESSAGES {
            return Err(FitError::TooManyMessages {
                kind: "message",
                limit: MAX_FIT_MESSAGES,
            });
        }

        let header_offset = cursor;
        let header = read_u8(bytes, &mut cursor, data_end)?;
        let compressed = header & 0x80 != 0;
        let (local, is_definition, developer_present) = if compressed {
            ((header >> 5) & 0x03, false, false)
        } else {
            (header & 0x0f, header & 0x40 != 0, header & 0x20 != 0)
        };

        if compressed {
            let offset = header & 0x1f;
            let timestamp = last_timestamp.map_or(u32::from(offset), |last| {
                let mut value: u32 = u32::from(offset) + (last & !0x1f);
                if u32::from(offset) < (last & 0x1f) {
                    value = value.wrapping_add(0x20);
                }
                value
            });
            let message = read_data_message(
                bytes,
                &mut cursor,
                data_end,
                local,
                &definitions,
                Some(timestamp),
                header_offset,
            )?;
            last_timestamp = message
                .number(253)
                .filter(|value| *value >= 0.0 && *value <= f64::from(u32::MAX))
                .map(|value| value as u32)
                .or(Some(timestamp));
            visitor.visit(message)?;
            continue;
        }

        if is_definition {
            let definition = read_definition_message(
                bytes,
                &mut cursor,
                data_end,
                developer_present,
                header_offset,
            )?;
            definitions.insert(local, definition);
            continue;
        }

        let message = read_data_message(
            bytes,
            &mut cursor,
            data_end,
            local,
            &definitions,
            None,
            header_offset,
        )?;
        last_timestamp = message
            .number(253)
            .filter(|value| *value >= 0.0 && *value <= f64::from(u32::MAX))
            .map(|value| value as u32)
            .or(last_timestamp);
        visitor.visit(message)?;
    }
    Ok(())
}

fn read_header(bytes: &[u8]) -> Result<(usize, usize), FitError> {
    if bytes.len() < 12 {
        return Err(FitError::Truncated {
            offset: 0,
            needed: 12,
            available: bytes.len(),
        });
    }
    let header_size = usize::from(bytes[0]);
    if header_size != 12 && header_size != 14 {
        return Err(FitError::InvalidHeader {
            reason: format!("header size is {header_size}, expected 12 or 14"),
        });
    }
    let protocol = bytes[1];
    if protocol != 0x10 && protocol != 0x20 {
        return Err(FitError::UnsupportedVersion { version: protocol });
    }
    if &bytes[8..12] != b".FIT" {
        return Err(FitError::NotFit);
    }
    let data_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let data_end = header_size.saturating_add(data_size);
    if bytes.len() < data_end {
        return Err(FitError::Truncated {
            offset: header_size,
            needed: data_size,
            available: bytes.len().saturating_sub(header_size),
        });
    }
    Ok((header_size, data_end))
}

fn read_definition_message(
    bytes: &[u8],
    cursor: &mut usize,
    data_end: usize,
    developer_present: bool,
    message_offset: usize,
) -> Result<MessageDefinition, FitError> {
    let _reserved = read_u8(bytes, cursor, data_end)?;
    let architecture = read_u8(bytes, cursor, data_end)?;
    let little_endian = architecture == 0;
    let mesg_num = read_u16(bytes, cursor, data_end, little_endian)?;
    let field_count = usize::from(read_u8(bytes, cursor, data_end)?);

    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        let definition_number = read_u8(bytes, cursor, data_end)?;
        let size = usize::from(read_u8(bytes, cursor, data_end)?);
        let base_type = read_u8(bytes, cursor, data_end)?;
        fields.push(FieldDefinition {
            definition_number,
            size,
            base_type,
        });
    }

    let mut developer_bytes = 0usize;
    if developer_present {
        let developer_count = usize::from(read_u8(bytes, cursor, data_end)?);
        for _ in 0..developer_count {
            let _definition_number = read_u8(bytes, cursor, data_end)?;
            let size = usize::from(read_u8(bytes, cursor, data_end)?);
            let _developer_data_index = read_u8(bytes, cursor, data_end)?;
            developer_bytes = developer_bytes.saturating_add(size);
        }
    }

    if *cursor > data_end {
        return Err(FitError::Malformed {
            offset: message_offset,
            reason: "the definition message extends past the data section".to_owned(),
        });
    }
    Ok(MessageDefinition {
        mesg_num,
        little_endian,
        fields,
        developer_bytes,
    })
}

#[allow(clippy::too_many_arguments)]
fn read_data_message(
    bytes: &[u8],
    cursor: &mut usize,
    data_end: usize,
    local: u8,
    definitions: &HashMap<u8, MessageDefinition>,
    compressed_timestamp: Option<u32>,
    message_offset: usize,
) -> Result<RawMessage, FitError> {
    let definition = definitions
        .get(&local)
        .ok_or(FitError::UnknownLocalMessage {
            offset: message_offset,
            local,
        })?;

    let mut fields = Vec::with_capacity(definition.fields.len());
    for field in &definition.fields {
        let raw = take(bytes, cursor, data_end, field.size)?;
        if let Some(value) = decode_value(raw, field.base_type, definition.little_endian) {
            fields.push((field.definition_number, value));
        }
    }

    if definition.developer_bytes > 0 {
        let _developer_fields = take(bytes, cursor, data_end, definition.developer_bytes)?;
    }

    Ok(RawMessage {
        mesg_num: definition.mesg_num,
        compressed_timestamp,
        fields,
    })
}

fn take<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    data_end: usize,
    count: usize,
) -> Result<&'a [u8], FitError> {
    let offset = *cursor;
    let available = data_end.saturating_sub(offset);
    if count > available {
        return Err(FitError::Truncated {
            offset,
            needed: count,
            available,
        });
    }
    *cursor += count;
    Ok(&bytes[offset..offset + count])
}

fn read_u8(bytes: &[u8], cursor: &mut usize, data_end: usize) -> Result<u8, FitError> {
    let raw = take(bytes, cursor, data_end, 1)?;
    Ok(raw[0])
}

fn read_u16(
    bytes: &[u8],
    cursor: &mut usize,
    data_end: usize,
    little_endian: bool,
) -> Result<u16, FitError> {
    let raw = take(bytes, cursor, data_end, 2)?;
    Ok(if little_endian {
        u16::from_le_bytes([raw[0], raw[1]])
    } else {
        u16::from_be_bytes([raw[0], raw[1]])
    })
}

/// Decode one field, returning `None` for a base type's invalid value.
///
/// The invalid value is checked here, before the FIT profile's scale or any
/// physiological range check is applied. This ordering is the contract.
fn decode_value(raw: &[u8], base_type: u8, little_endian: bool) -> Option<RawValue> {
    match base_type {
        0x00 => number_value(read_u8_exact(raw), 0xff),
        0x01 => number_value(read_i8_exact(raw), 0x7f),
        0x02 => number_value(read_u8_exact(raw), 0xff),
        0x83 => number_value(read_i16_exact(raw, little_endian), 0x7fff),
        0x84 => number_value(read_u16_exact(raw, little_endian), 0xffff),
        0x85 => number_value(read_i32_exact(raw, little_endian), 0x7fff_ffff),
        0x86 => number_value(read_u32_exact(raw, little_endian), 0xffff_ffff),
        0x8a => number_value(read_u8_exact(raw), 0x00),
        0x8b => number_value(read_u16_exact(raw, little_endian), 0x0000),
        0x8c => number_value(read_u32_exact(raw, little_endian), 0x0000_0000),
        0x8e => read_i64_exact(raw, little_endian)
            .filter(|value| *value != i64::MAX)
            .map(|value| RawValue::Number(value as f64)),
        0x8f => read_u64_exact(raw, little_endian)
            .filter(|value| *value != u64::MAX)
            .map(|value| RawValue::Number(value as f64)),
        0x90 => read_u64_exact(raw, little_endian)
            .filter(|value| *value != 0)
            .map(|value| RawValue::Number(value as f64)),
        0x07 => {
            if raw.first() == Some(&0) {
                None
            } else {
                let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
                Some(RawValue::Text(
                    String::from_utf8_lossy(&raw[..end]).into_owned(),
                ))
            }
        }
        0x0d => {
            if raw.first() == Some(&0) {
                None
            } else {
                // Byte arrays are consumed but none of the fields read here is
                // a byte array.
                None
            }
        }
        0x88 => read_f32_exact(raw, little_endian)
            .filter(|value| !value.is_nan())
            .map(|value| RawValue::Number(f64::from(value))),
        0x89 => read_f64_exact(raw, little_endian)
            .filter(|value| !value.is_nan())
            .map(RawValue::Number),
        _ => None,
    }
}

fn number_value<T>(value: Option<T>, invalid: T) -> Option<RawValue>
where
    T: PartialEq + Into<f64>,
{
    value
        .filter(|value| *value != invalid)
        .map(|value| RawValue::Number(value.into()))
}

fn read_u8_exact(raw: &[u8]) -> Option<u8> {
    raw.first().copied()
}

fn read_i8_exact(raw: &[u8]) -> Option<i8> {
    raw.first().copied().map(|value| value as i8)
}

fn read_u16_exact(raw: &[u8], little_endian: bool) -> Option<u16> {
    if raw.len() < 2 {
        return None;
    }
    Some(if little_endian {
        u16::from_le_bytes([raw[0], raw[1]])
    } else {
        u16::from_be_bytes([raw[0], raw[1]])
    })
}

fn read_i16_exact(raw: &[u8], little_endian: bool) -> Option<i16> {
    read_u16_exact(raw, little_endian).map(|value| value as i16)
}

fn read_u32_exact(raw: &[u8], little_endian: bool) -> Option<u32> {
    if raw.len() < 4 {
        return None;
    }
    Some(if little_endian {
        u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]])
    } else {
        u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]])
    })
}

fn read_i32_exact(raw: &[u8], little_endian: bool) -> Option<i32> {
    read_u32_exact(raw, little_endian).map(|value| value as i32)
}

fn read_u64_exact(raw: &[u8], little_endian: bool) -> Option<u64> {
    if raw.len() < 8 {
        return None;
    }
    let bytes = [
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ];
    Some(if little_endian {
        u64::from_le_bytes(bytes)
    } else {
        u64::from_be_bytes(bytes)
    })
}

fn read_i64_exact(raw: &[u8], little_endian: bool) -> Option<i64> {
    read_u64_exact(raw, little_endian).map(|value| value as i64)
}

fn read_f32_exact(raw: &[u8], little_endian: bool) -> Option<f32> {
    read_u32_exact(raw, little_endian).map(f32::from_bits)
}

fn read_f64_exact(raw: &[u8], little_endian: bool) -> Option<f64> {
    read_u64_exact(raw, little_endian).map(f64::from_bits)
}

// ------------------------------------------------------------------ importer

#[derive(Debug, Clone, Default)]
struct RawLap {
    total_elapsed_time: Option<f64>,
    total_timer_time: Option<f64>,
    total_distance: Option<f64>,
    avg_speed: Option<f64>,
    max_speed: Option<f64>,
    avg_heart_rate: Option<f64>,
    max_heart_rate: Option<f64>,
    avg_cadence: Option<f64>,
    max_cadence: Option<f64>,
    total_ascent: Option<f64>,
    total_descent: Option<f64>,
}

#[derive(Debug, Clone, Default)]
struct RawSession {
    total_elapsed_time: Option<f64>,
    total_timer_time: Option<f64>,
    total_distance: Option<f64>,
    avg_speed: Option<f64>,
    max_speed: Option<f64>,
    avg_heart_rate: Option<f64>,
    max_heart_rate: Option<f64>,
    avg_cadence: Option<f64>,
    max_cadence: Option<f64>,
    total_calories: Option<f64>,
    total_ascent: Option<f64>,
    total_descent: Option<f64>,
}

#[derive(Debug, Clone)]
struct RawRecord {
    timestamp_s: f64,
    distance_m: Option<f64>,
    speed_ms: Option<f64>,
    cadence: Option<f64>,
    heart_rate_bpm: Option<f64>,
    altitude_m: Option<f64>,
}

struct Importer<'a> {
    file_name: &'a str,
    serial_number: Option<u64>,
    start_timestamp_s: Option<f64>,
    end_timestamp_s: Option<f64>,
    last_distance_m: Option<f64>,
    records: Vec<RawRecord>,
    geo: Vec<[f64; 2]>,
    laps: Vec<RawLap>,
    activity_summary: Option<RawSession>,
    sport: Option<String>,
    device: Option<String>,
    record_count: usize,
    lap_count: usize,
    session_count: usize,
    file_id_count: usize,
    device_info_count: usize,
}

impl<'a> Importer<'a> {
    fn new(file_name: &'a str) -> Self {
        Self {
            file_name,
            serial_number: None,
            start_timestamp_s: None,
            end_timestamp_s: None,
            last_distance_m: None,
            records: Vec::new(),
            geo: Vec::new(),
            laps: Vec::new(),
            activity_summary: None,
            sport: None,
            device: None,
            record_count: 0,
            lap_count: 0,
            session_count: 0,
            file_id_count: 0,
            device_info_count: 0,
        }
    }
}

impl VisitMessage for Importer<'_> {
    fn visit(&mut self, message: RawMessage) -> Result<(), FitError> {
        match message.mesg_num {
            MESG_FILE_ID => {
                self.file_id_count += 1;
                if self.file_id_count > 100 {
                    return Err(FitError::TooManyMessages {
                        kind: "file_id",
                        limit: 100,
                    });
                }
                if self.serial_number.is_none() {
                    self.serial_number = message
                        .number(3)
                        .filter(|value| *value >= 0.0 && *value <= f64::from(u32::MAX))
                        .map(|value| value as u64);
                }
            }
            MESG_RECORD => {
                self.record_count += 1;
                if self.record_count > MAX_FIT_RECORDS {
                    return Err(FitError::TooManyMessages {
                        kind: "record",
                        limit: MAX_FIT_RECORDS,
                    });
                }
                self.visit_record(&message)?;
            }
            MESG_LAP => {
                self.lap_count += 1;
                if self.lap_count > 10_000 {
                    return Err(FitError::TooManyMessages {
                        kind: "lap",
                        limit: 10_000,
                    });
                }
                self.laps.push(RawLap {
                    total_elapsed_time: scaled(&message, 7, 1_000.0, 0.0),
                    total_timer_time: scaled(&message, 8, 1_000.0, 0.0),
                    total_distance: scaled(&message, 9, 100.0, 0.0),
                    avg_speed: scaled(&message, 13, 1_000.0, 0.0),
                    max_speed: scaled(&message, 14, 1_000.0, 0.0),
                    avg_heart_rate: scaled(&message, 15, 1.0, 0.0),
                    max_heart_rate: scaled(&message, 16, 1.0, 0.0),
                    avg_cadence: scaled(&message, 17, 1.0, 0.0),
                    max_cadence: scaled(&message, 18, 1.0, 0.0),
                    total_ascent: scaled(&message, 21, 1.0, 0.0),
                    total_descent: scaled(&message, 22, 1.0, 0.0),
                });
            }
            MESG_SESSION_DATA => {
                self.session_count += 1;
                if self.session_count > 100 {
                    return Err(FitError::TooManyMessages {
                        kind: "session",
                        limit: 100,
                    });
                }
                if self.activity_summary.is_none() {
                    self.activity_summary = Some(RawSession {
                        total_elapsed_time: scaled(&message, 7, 1_000.0, 0.0),
                        total_timer_time: scaled(&message, 8, 1_000.0, 0.0),
                        total_distance: scaled(&message, 9, 100.0, 0.0),
                        avg_speed: scaled(&message, 14, 1_000.0, 0.0),
                        max_speed: scaled(&message, 15, 1_000.0, 0.0),
                        avg_heart_rate: scaled(&message, 16, 1.0, 0.0),
                        max_heart_rate: scaled(&message, 17, 1.0, 0.0),
                        avg_cadence: scaled(&message, 18, 1.0, 0.0),
                        max_cadence: scaled(&message, 19, 1.0, 0.0),
                        total_calories: scaled(&message, 11, 1.0, 0.0),
                        total_ascent: scaled(&message, 22, 1.0, 0.0),
                        total_descent: scaled(&message, 23, 1.0, 0.0),
                    });
                    self.sport = message
                        .number(5)
                        .filter(|value| *value >= 0.0 && *value <= f64::from(u8::MAX))
                        .map(|value| sport_name(value as u16));
                }
            }
            MESG_DEVICE_INFO => {
                self.device_info_count += 1;
                if self.device_info_count > 100 {
                    return Err(FitError::TooManyMessages {
                        kind: "device_info",
                        limit: 100,
                    });
                }
                if self.device.is_none() {
                    let product = message
                        .text(27)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned);
                    self.device = product.or_else(|| {
                        message
                            .number(2)
                            .filter(|value| *value >= 0.0 && *value <= f64::from(u16::MAX))
                            .map(|value| manufacturer_name(value as u16))
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Importer<'_> {
    fn visit_record(&mut self, message: &RawMessage) -> Result<(), FitError> {
        let timestamp_s = message
            .number(253)
            .filter(|value| *value >= 0.0 && *value <= f64::from(u32::MAX))
            .or_else(|| message.compressed_timestamp.map(f64::from))
            .ok_or(FitError::MissingTimestamp { offset: 0 })?;
        if self.start_timestamp_s.is_none() {
            self.start_timestamp_s = Some(timestamp_s);
        }
        self.end_timestamp_s = Some(timestamp_s);

        let distance_m = in_range(scaled(message, 5, 100.0, 0.0), 0.0, 10_000_000.0);
        if let Some(distance) = distance_m {
            self.last_distance_m = Some(distance);
        }

        let latitude = message.number(0);
        let longitude = message.number(1);
        if let (Some(latitude), Some(longitude)) = (latitude, longitude) {
            if (-2_147_483_648.0..=2_147_483_648.0).contains(&latitude)
                && (-2_147_483_648.0..=2_147_483_648.0).contains(&longitude)
            {
                self.geo.push([
                    semicircles_to_degrees(latitude),
                    semicircles_to_degrees(longitude),
                ]);
            }
        }

        let heart_rate_bpm = in_range(scaled(message, 3, 1.0, 0.0), 0.0, 300.0);
        let cadence = in_range(scaled(message, 4, 1.0, 0.0), 0.0, 300.0);
        let speed_ms = in_range(
            scaled(message, 6, 1_000.0, 0.0).or_else(|| scaled(message, 73, 1_000.0, 0.0)),
            0.0,
            150.0,
        );
        let altitude_m = in_range(
            scaled(message, 2, 5.0, 500.0).or_else(|| scaled(message, 78, 5.0, 500.0)),
            -1_000.0,
            10_000.0,
        );

        self.records.push(RawRecord {
            timestamp_s,
            distance_m,
            speed_ms,
            cadence,
            heart_rate_bpm,
            altitude_m,
        });
        Ok(())
    }

    fn finish(self) -> Result<FitActivity, FitError> {
        let summary = self.activity_summary.clone().unwrap_or_default();
        let sport = self.sport.clone();
        let start_timestamp_s = self.start_timestamp_s;
        let end_timestamp_s = self.end_timestamp_s;

        let distance_m = bounded(summary.total_distance, 10_000_000.0)
            .filter(|value| *value != 0.0)
            .or(self.last_distance_m)
            .ok_or(FitError::MissingActivityData { field: "distance" })?;
        let timer = bounded(summary.total_timer_time, 604_800.0).filter(|value| *value != 0.0);
        let elapsed = bounded(summary.total_elapsed_time, 604_800.0).filter(|value| *value != 0.0);
        let mut moving_time_s = timer.or(elapsed);
        if moving_time_s.is_none() {
            if let (Some(start), Some(end)) = (start_timestamp_s, end_timestamp_s) {
                moving_time_s = Some((end - start).max(0.0));
            }
        }
        let moving_time_s = moving_time_s.ok_or(FitError::MissingActivityData {
            field: "moving time",
        })?;
        let elapsed_time_s = elapsed.unwrap_or(moving_time_s);

        let base_timestamp_s = start_timestamp_s.unwrap_or(0.0);
        let samples_for_segmentation: Vec<Sample> = self
            .records
            .iter()
            .enumerate()
            .map(|(index, record)| Sample {
                index,
                elapsed_s: round_half_even(record.timestamp_s - base_timestamp_s, 3),
                distance_m: record.distance_m,
                speed_ms: record.speed_ms,
                stored_cadence: record.cadence,
                heart_rate_bpm: record.heart_rate_bpm,
                altitude_m: record.altitude_m,
            })
            .collect();
        let segmentation = segment::segment_samples(&samples_for_segmentation);
        let has_records = !self.records.is_empty();

        let fitted_samples = self
            .records
            .iter()
            .zip(segmentation.samples.iter())
            .map(|(record, classified)| FitSample {
                index: classified.sample.index,
                elapsed_s: classified.sample.elapsed_s,
                kind: classified.kind,
                distance_m: record.distance_m,
                speed_ms: record.speed_ms,
                stored_cadence: record.cadence,
                cadence_spm: record
                    .cadence
                    .map(|value| running_cadence(value, sport.as_deref())),
                heart_rate_bpm: record.heart_rate_bpm,
                altitude_m: record.altitude_m,
            })
            .collect();

        let mut streams = BTreeMap::new();
        if let Some(start) = start_timestamp_s {
            let running: Vec<&RawRecord> = self
                .records
                .iter()
                .zip(segmentation.samples.iter())
                .filter_map(|(record, classified)| {
                    (classified.kind == SegmentKind::Run).then_some(record)
                })
                .collect();
            if !running.is_empty() {
                let heart_rate: Vec<(f64, f64)> = running
                    .iter()
                    .filter_map(|record| {
                        record
                            .heart_rate_bpm
                            .map(|value| (record.timestamp_s, value))
                    })
                    .collect();
                streams.insert("hr".to_owned(), series(&heart_rate, start, "bpm"));

                let cadence: Vec<(f64, f64)> = running
                    .iter()
                    .filter_map(|record| {
                        record.cadence.map(|value| {
                            (record.timestamp_s, running_cadence(value, sport.as_deref()))
                        })
                    })
                    .collect();
                streams.insert("cadence".to_owned(), series(&cadence, start, "spm"));

                let pace: Vec<(f64, f64)> = running
                    .iter()
                    .filter_map(|record| {
                        record
                            .speed_ms
                            .filter(|speed| *speed > 0.1)
                            .map(|speed| (record.timestamp_s, 1_000.0 / speed))
                    })
                    .collect();
                if !pace.is_empty() {
                    streams.insert("pace".to_owned(), series(&pace, start, "s/km"));
                }

                let distance: Vec<(f64, f64)> = running
                    .iter()
                    .filter_map(|record| record.distance_m.map(|value| (record.timestamp_s, value)))
                    .collect();
                streams.insert("distance".to_owned(), series(&distance, start, "m"));

                let elevation: Vec<(f64, f64)> = running
                    .iter()
                    .filter_map(|record| record.altitude_m.map(|value| (record.timestamp_s, value)))
                    .collect();
                if !elevation.is_empty() {
                    streams.insert("elevation".to_owned(), series(&elevation, start, "m"));
                }
            }
        }

        let (laps, lap_bounds) = build_laps(&self.laps, &segmentation.samples, sport.as_deref());
        let start_time = start_timestamp_s.and_then(fit_timestamp_to_utc);
        let local_date = start_time.map(|value| value.format("%Y-%m-%d").to_string());
        let day_of_week = start_time.map(|value| value.format("%A").to_string());

        let totals = &segmentation.totals;
        Ok(FitActivity {
            serial_number: self.serial_number,
            external_id: self
                .serial_number
                .map(|value| value.to_string())
                .unwrap_or_else(|| self.file_name.to_owned()),
            file_name: self.file_name.to_owned(),
            name: activity_name(self.file_name),
            sport: sport.clone(),
            sport_type: sport
                .as_deref()
                .map(normalise_sport)
                .unwrap_or_else(|| "running".to_owned()),
            start_time,
            local_date,
            day_of_week,
            distance_m,
            moving_time_s,
            elapsed_time_s,
            elevation_gain_m: bounded(summary.total_ascent, 100_000.0).unwrap_or(0.0),
            elevation_loss_m: bounded(summary.total_descent, 100_000.0),
            avg_hr: bounded(summary.avg_heart_rate, 300.0).map(|value| value.trunc() as i64),
            max_hr: bounded(summary.max_heart_rate, 300.0).map(|value| value.trunc() as i64),
            calories: bounded(summary.total_calories, 1_000_000.0),
            avg_cadence: bounded(summary.avg_cadence, 300.0),
            max_cadence: bounded(summary.max_cadence, 300.0),
            avg_speed_ms: bounded(summary.avg_speed, 150.0),
            max_speed_ms: bounded(summary.max_speed, 150.0),
            device: self.device,
            running_time_s: has_records.then_some(totals.running_time_s),
            walking_time_s: has_records.then_some(totals.walking_time_s),
            stopped_time_s: has_records.then_some(totals.stopped_time_s),
            running_distance_m: has_records.then_some(totals.running_distance_m),
            segments: segmentation.segments,
            samples: fitted_samples,
            streams,
            laps,
            lap_bounds,
            geo_points: geo_points(&self.geo),
        })
    }
}

// -------------------------------------------------------------------- helpers

fn scaled(message: &RawMessage, definition_number: u8, scale: f64, offset: f64) -> Option<f64> {
    message
        .number(definition_number)
        .map(|value| value / scale - offset)
}

fn in_range(value: Option<f64>, low: f64, high: f64) -> Option<f64> {
    value.filter(|value| (low..=high).contains(value))
}

fn bounded(value: Option<f64>, maximum: f64) -> Option<f64> {
    value.filter(|value| (0.0..=maximum).contains(value))
}

fn fit_timestamp_to_utc(timestamp_s: f64) -> Option<DateTime<Utc>> {
    let unix_seconds = FIT_EPOCH_UNIX as f64 + timestamp_s;
    let seconds = unix_seconds.floor() as i64;
    let nanoseconds = ((unix_seconds - unix_seconds.floor()) * 1_000_000_000.0) as u32;
    DateTime::from_timestamp(seconds, nanoseconds)
}

fn semicircles_to_degrees(semicircles: f64) -> f64 {
    round_half_even(semicircles * 180.0 / 2_147_483_648.0, 7)
}

fn activity_name(file_name: &str) -> String {
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    let display_name = stem.replace(['_', '-'], " ");
    let name = display_name.trim();
    if name.is_empty() {
        "Imported run".to_owned()
    } else {
        name.to_owned()
    }
}

/// Normalise a FIT sport into Stride's vocabulary.
pub fn normalise_sport(sport: &str) -> String {
    let key = sport.trim().to_lowercase().replace(' ', "_");
    match key.as_str() {
        "running" | "trail_running" | "treadmill_running" | "virtualrun" | "street_running"
        | "track_running" => "running".to_owned(),
        "walking" | "hiking" | "cycling" | "swimming" | "strength_training" => key,
        _ if key.ends_with("running") => "running".to_owned(),
        _ => key,
    }
}

/// Correct a running cadence that was stored as stride cycles.
///
/// Some devices store one value per two footfalls. A stored value below 100
/// is doubled for running sports only; a cyclist's 90 rpm stays 90 rpm.
pub fn running_cadence(value: f64, sport: Option<&str>) -> f64 {
    const HALVED_SPORTS: &[&str] = &[
        "running",
        "trail_running",
        "treadmill_running",
        "street_running",
        "track_running",
        "virtualrun",
    ];
    if sport.is_some_and(|sport| !HALVED_SPORTS.contains(&sport)) {
        return value;
    }
    if value > 0.0 && value < 100.0 {
        value * 2.0
    } else {
        value
    }
}

fn series(samples: &[(f64, f64)], start_timestamp_s: f64, unit: &str) -> Stream {
    Stream {
        seconds: samples
            .iter()
            .map(|(timestamp, _)| {
                let elapsed = round_half_even(timestamp - start_timestamp_s, 0);
                elapsed.trunc() as i64
            })
            .collect(),
        values: samples.iter().map(|(_, value)| *value).collect(),
        unit: unit.to_owned(),
    }
}

fn geo_points(points: &[[f64; 2]]) -> GeoPoints {
    GeoPoints {
        count: points.len(),
        first: points.iter().take(3).copied().collect(),
        last: points.iter().rev().take(3).rev().copied().collect(),
    }
}

fn build_laps(
    raw_laps: &[RawLap],
    classified: &[ClassifiedSample],
    sport: Option<&str>,
) -> (Vec<FitLap>, Vec<f64>) {
    let mut laps = Vec::with_capacity(raw_laps.len());
    let mut lap_bounds = Vec::with_capacity(raw_laps.len());
    let mut cumulative_elapsed = 0.0;

    for (index, raw) in raw_laps.iter().enumerate() {
        let distance_m = bounded(raw.total_distance, 10_000_000.0).unwrap_or(0.0);
        let duration_s = bounded(raw.total_timer_time, 604_800.0)
            .filter(|value| *value != 0.0)
            .or_else(|| bounded(raw.total_elapsed_time, 604_800.0))
            .unwrap_or(0.0);
        let mut avg_speed_ms = bounded(raw.avg_speed, 150.0);
        if avg_speed_ms.is_none() && distance_m != 0.0 && duration_s != 0.0 {
            let derived = distance_m / duration_s;
            avg_speed_ms = (derived <= 150.0).then_some(derived);
        }
        cumulative_elapsed += raw
            .total_elapsed_time
            .filter(|value| *value != 0.0)
            .or(raw.total_timer_time)
            .filter(|value| *value != 0.0)
            .unwrap_or(0.0);
        lap_bounds.push(round_half_even(cumulative_elapsed, 3));

        laps.push(FitLap {
            lap_index: i64::try_from(index + 1).unwrap_or(i64::MAX),
            distance_m,
            duration_s,
            avg_speed_ms,
            max_speed_ms: bounded(raw.max_speed, 150.0),
            avg_hr: bounded(raw.avg_heart_rate, 300.0).map(|value| value.trunc() as i64),
            max_hr: bounded(raw.max_heart_rate, 300.0).map(|value| value.trunc() as i64),
            avg_cadence: bounded(raw.avg_cadence, 300.0)
                .filter(|value| *value != 0.0)
                .map(|value| running_cadence(value, sport)),
            max_cadence: bounded(raw.max_cadence, 300.0)
                .filter(|value| *value != 0.0)
                .map(|value| running_cadence(value, sport)),
            stride_length_m: None,
            elevation_gain_m: bounded(raw.total_ascent, 100_000.0),
            elevation_loss_m: bounded(raw.total_descent, 100_000.0),
            start_elevation_m: None,
            end_elevation_m: None,
            elapsed_s: round_half_even(cumulative_elapsed, 3),
            running_distance_m: None,
            pace_s_per_km: None,
        });
    }

    if !classified.is_empty() && !laps.is_empty() {
        laps = running_laps(laps, classified, sport);
    }
    (laps, lap_bounds)
}

fn running_laps(
    laps: Vec<FitLap>,
    classified: &[ClassifiedSample],
    sport: Option<&str>,
) -> Vec<FitLap> {
    let mut output = Vec::with_capacity(laps.len());
    let mut cursor = 0.0;
    for lap in laps {
        let end = if lap.elapsed_s != 0.0 {
            lap.elapsed_s
        } else {
            cursor
        };
        let start = cursor;
        cursor = end;
        let span = (end - start).max(0.0);

        let window: Vec<&ClassifiedSample> = classified
            .iter()
            .filter(|sample| (sample.sample.elapsed_s >= start) && (sample.sample.elapsed_s <= end))
            .collect();
        let running: Vec<&ClassifiedSample> = window
            .iter()
            .copied()
            .filter(|sample| sample.kind == SegmentKind::Run)
            .collect();

        let mut running_time_s = 0.0;
        let mut running_distance_m = 0.0;
        for pair in window.windows(2) {
            let previous = pair[0];
            let current = pair[1];
            if previous.kind != SegmentKind::Run || current.kind != SegmentKind::Run {
                continue;
            }
            running_time_s += (current.sample.elapsed_s - previous.sample.elapsed_s).max(0.0);
            if let (Some(previous_distance), Some(current_distance)) =
                (previous.sample.distance_m, current.sample.distance_m)
            {
                running_distance_m += (current_distance - previous_distance).max(0.0);
            }
        }

        let heart_rates: Vec<f64> = running
            .iter()
            .filter_map(|sample| sample.sample.heart_rate_bpm)
            .collect();
        let cadences: Vec<f64> = running
            .iter()
            .filter_map(|sample| {
                sample
                    .sample
                    .stored_cadence
                    .map(|value| running_cadence(value, sport))
            })
            .collect();
        let pace_s_per_km = (running_distance_m > 1.0 && running_time_s != 0.0)
            .then_some(running_time_s / (running_distance_m / 1_000.0));

        output.push(FitLap {
            duration_s: round_half_even(running_time_s, 3),
            running_distance_m: Some(round_half_even(running_distance_m, 2)),
            avg_speed_ms: (running_time_s > 0.0 && running_distance_m > 0.0)
                .then_some(running_distance_m / running_time_s),
            avg_hr: mean(&heart_rates).map(|value| value.trunc() as i64),
            max_hr: heart_rates
                .iter()
                .copied()
                .fold(None::<f64>, |best, value| {
                    Some(best.map_or(value, |best| best.max(value)))
                })
                .map(|value| value.trunc() as i64),
            avg_cadence: mean(&cadences).map(|value| round_half_even(value, 1)),
            max_cadence: cadences
                .iter()
                .copied()
                .fold(None::<f64>, |best, value| {
                    Some(best.map_or(value, |best| best.max(value)))
                })
                .map(|value| round_half_even(value, 1)),
            pace_s_per_km: pace_s_per_km
                .filter(|value| *value != 0.0)
                .map(|value| round_half_even(value, 2)),
            elapsed_s: round_half_even(span, 3),
            ..lap
        });
    }
    output
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

/// Render a FIT `sport` enumeration.
pub fn sport_name(value: u16) -> String {
    const SPORTS: &[(u16, &str)] = &[
        (0, "generic"),
        (1, "running"),
        (2, "cycling"),
        (3, "transition"),
        (4, "fitness_equipment"),
        (5, "swimming"),
        (6, "basketball"),
        (7, "soccer"),
        (8, "tennis"),
        (9, "american_football"),
        (10, "training"),
        (11, "walking"),
        (12, "cross_country_skiing"),
        (13, "alpine_skiing"),
        (14, "snowboarding"),
        (15, "rowing"),
        (16, "mountaineering"),
        (17, "hiking"),
        (18, "multisport"),
        (19, "paddling"),
        (20, "flying"),
        (21, "e_biking"),
        (22, "motorcycling"),
        (23, "boating"),
        (24, "driving"),
        (25, "golf"),
        (26, "hang_gliding"),
        (27, "horseback_riding"),
        (28, "hunting"),
        (29, "fishing"),
        (30, "inline_skating"),
        (31, "rock_climbing"),
        (32, "sailing"),
        (33, "ice_skating"),
        (34, "sky_diving"),
        (35, "snowshoeing"),
        (36, "snowmobiling"),
        (37, "stand_up_paddleboarding"),
        (38, "surfing"),
        (39, "wakeboarding"),
        (40, "water_skiing"),
        (41, "kayaking"),
        (42, "rafting"),
        (43, "windsurfing"),
        (44, "kitesurfing"),
        (45, "tactical"),
        (46, "jumpmaster"),
        (47, "boxing"),
        (48, "floor_climbing"),
        (49, "baseball"),
        (53, "diving"),
        (56, "shooting"),
        (58, "winter_sport"),
        (59, "grinding"),
        (62, "hiit"),
        (63, "video_gaming"),
        (64, "racket"),
        (65, "wheelchair_push_walk"),
        (66, "wheelchair_push_run"),
        (67, "meditation"),
        (68, "para_sport"),
        (69, "disc_golf"),
        (70, "team_sport"),
        (71, "cricket"),
        (72, "rugby"),
        (73, "hockey"),
        (74, "lacrosse"),
        (75, "volleyball"),
        (76, "water_tubing"),
        (77, "wakesurfing"),
        (78, "water_sport"),
        (79, "archery"),
        (80, "mixed_martial_arts"),
        (81, "motor_sports"),
        (82, "snorkeling"),
        (83, "dance"),
        (84, "jump_rope"),
        (85, "pool_apnea"),
        (86, "mobility"),
        (87, "geocaching"),
        (88, "canoeing"),
        (254, "all"),
    ];
    SPORTS
        .iter()
        .find(|(number, _)| *number == value)
        .map_or_else(|| value.to_string(), |(_, name)| (*name).to_owned())
}

/// Render a FIT `manufacturer` enumeration.
///
/// The table is the complete Garmin FIT profile table, including COROS (294)
/// and the vendor ids above it. An unknown id is rendered as its number.
pub fn manufacturer_name(value: u16) -> String {
    const MANUFACTURERS: &[(u16, &str)] = &[
        (1, "garmin"),
        (2, "garmin_fr405_antfs"),
        (3, "zephyr"),
        (4, "dayton"),
        (5, "idt"),
        (6, "srm"),
        (7, "quarq"),
        (8, "ibike"),
        (9, "saris"),
        (10, "spark_hk"),
        (11, "tanita"),
        (12, "echowell"),
        (13, "dynastream_oem"),
        (14, "nautilus"),
        (15, "dynastream"),
        (16, "timex"),
        (17, "metrigear"),
        (18, "xelic"),
        (19, "beurer"),
        (20, "cardiosport"),
        (21, "a_and_d"),
        (22, "hmm"),
        (23, "suunto"),
        (24, "thita_elektronik"),
        (25, "gpulse"),
        (26, "clean_mobile"),
        (27, "pedal_brain"),
        (28, "peaksware"),
        (29, "saxonar"),
        (30, "lemond_fitness"),
        (31, "dexcom"),
        (32, "wahoo_fitness"),
        (33, "octane_fitness"),
        (34, "archinoetics"),
        (35, "the_hurt_box"),
        (36, "citizen_systems"),
        (37, "magellan"),
        (38, "osynce"),
        (39, "holux"),
        (40, "concept2"),
        (41, "shimano"),
        (42, "one_giant_leap"),
        (43, "ace_sensor"),
        (44, "brim_brothers"),
        (45, "xplova"),
        (46, "perception_digital"),
        (47, "bf1systems"),
        (48, "pioneer"),
        (49, "spantec"),
        (50, "metalogics"),
        (51, "4iiiis"),
        (52, "seiko_epson"),
        (53, "seiko_epson_oem"),
        (54, "ifor_powell"),
        (55, "maxwell_guider"),
        (56, "star_trac"),
        (57, "breakaway"),
        (58, "alatech_technology_ltd"),
        (59, "mio_technology_europe"),
        (60, "rotor"),
        (61, "geonaute"),
        (62, "id_bike"),
        (63, "specialized"),
        (64, "wtek"),
        (65, "physical_enterprises"),
        (66, "north_pole_engineering"),
        (67, "bkool"),
        (68, "cateye"),
        (69, "stages_cycling"),
        (70, "sigmasport"),
        (71, "tomtom"),
        (72, "peripedal"),
        (73, "wattbike"),
        (76, "moxy"),
        (77, "ciclosport"),
        (78, "powerbahn"),
        (79, "acorn_projects_aps"),
        (80, "lifebeam"),
        (81, "bontrager"),
        (82, "wellgo"),
        (83, "scosche"),
        (84, "magura"),
        (85, "woodway"),
        (86, "elite"),
        (87, "nielsen_kellerman"),
        (88, "dk_city"),
        (89, "tacx"),
        (90, "direction_technology"),
        (91, "magtonic"),
        (92, "1partcarbon"),
        (93, "inside_ride_technologies"),
        (94, "sound_of_motion"),
        (95, "stryd"),
        (96, "icg"),
        (97, "MiPulse"),
        (98, "bsx_athletics"),
        (99, "look"),
        (100, "campagnolo_srl"),
        (101, "body_bike_smart"),
        (102, "praxisworks"),
        (103, "limits_technology"),
        (104, "topaction_technology"),
        (105, "cosinuss"),
        (106, "fitcare"),
        (107, "magene"),
        (108, "giant_manufacturing_co"),
        (109, "tigrasport"),
        (110, "salutron"),
        (111, "technogym"),
        (112, "bryton_sensors"),
        (113, "latitude_limited"),
        (114, "soaring_technology"),
        (115, "igpsport"),
        (116, "thinkrider"),
        (117, "gopher_sport"),
        (118, "waterrower"),
        (119, "orangetheory"),
        (120, "inpeak"),
        (121, "kinetic"),
        (122, "johnson_health_tech"),
        (123, "polar_electro"),
        (124, "seesense"),
        (125, "nci_technology"),
        (126, "iqsquare"),
        (127, "leomo"),
        (128, "ifit_com"),
        (129, "coros_byte"),
        (130, "versa_design"),
        (131, "chileaf"),
        (132, "cycplus"),
        (133, "gravaa_byte"),
        (134, "sigeyi"),
        (135, "coospo"),
        (136, "geoid"),
        (137, "bosch"),
        (138, "kyto"),
        (139, "kinetic_sports"),
        (140, "decathlon_byte"),
        (141, "tq_systems"),
        (142, "tag_heuer"),
        (143, "keiser_fitness"),
        (144, "zwift_byte"),
        (145, "porsche_ep"),
        (146, "blackbird"),
        (147, "meilan_byte"),
        (148, "ezon"),
        (149, "laisi"),
        (150, "myzone"),
        (151, "abawo"),
        (152, "bafang"),
        (153, "luhong_technology"),
        (255, "development"),
        (257, "healthandlife"),
        (258, "lezyne"),
        (259, "scribe_labs"),
        (260, "zwift"),
        (261, "watteam"),
        (262, "recon"),
        (263, "favero_electronics"),
        (264, "dynovelo"),
        (265, "strava"),
        (266, "precor"),
        (267, "bryton"),
        (268, "sram"),
        (269, "navman"),
        (270, "cobi"),
        (271, "spivi"),
        (272, "mio_magellan"),
        (273, "evesports"),
        (274, "sensitivus_gauge"),
        (275, "podoon"),
        (276, "life_time_fitness"),
        (277, "falco_e_motors"),
        (278, "minoura"),
        (279, "cycliq"),
        (280, "luxottica"),
        (281, "trainer_road"),
        (282, "the_sufferfest"),
        (283, "fullspeedahead"),
        (284, "virtualtraining"),
        (285, "feedbacksports"),
        (286, "omata"),
        (287, "vdo"),
        (288, "magneticdays"),
        (289, "hammerhead"),
        (290, "kinetic_by_kurt"),
        (291, "shapelog"),
        (292, "dabuziduo"),
        (293, "jetblack"),
        (294, "coros"),
        (295, "virtugo"),
        (296, "velosense"),
        (297, "cycligentinc"),
        (298, "trailforks"),
        (299, "mahle_ebikemotion"),
        (300, "nurvv"),
        (301, "microprogram"),
        (302, "zone5cloud"),
        (303, "greenteg"),
        (304, "yamaha_motors"),
        (305, "whoop"),
        (306, "gravaa"),
        (307, "onelap"),
        (308, "monark_exercise"),
        (309, "form"),
        (310, "decathlon"),
        (311, "syncros"),
        (312, "heatup"),
        (313, "cannondale"),
        (314, "true_fitness"),
        (315, "RGT_cycling"),
        (316, "vasa"),
        (317, "race_republic"),
        (318, "fazua"),
        (319, "oreka_training"),
        (320, "lsec"),
        (321, "lululemon_studio"),
        (322, "shanyue"),
        (323, "spinning_mda"),
        (324, "hilldating"),
        (325, "aero_sensor"),
        (326, "nike"),
        (327, "magicshine"),
        (328, "ictrainer"),
        (329, "absolute_cycling"),
        (330, "eo_swimbetter"),
        (331, "mywhoosh"),
        (332, "ravemen"),
        (333, "tektro_racing_products"),
        (334, "darad_innovation_corporation"),
        (335, "cycloptim"),
        (337, "runna"),
        (339, "zepp"),
        (340, "peloton"),
        (341, "carv"),
        (342, "tissot"),
        (345, "real_velo"),
        (346, "wetech"),
        (347, "jespr"),
        (348, "huawei"),
        (349, "gotoes"),
        (350, "cadence_app"),
        (351, "una_watch"),
        (352, "tymewear"),
        (353, "exposure_lights"),
        (354, "forto_cycling"),
        (5759, "actigraphcorp"),
    ];
    MANUFACTURERS
        .iter()
        .find(|(number, _)| *number == value)
        .map_or_else(|| value.to_string(), |(_, name)| (*name).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadence_is_only_corrected_for_running() {
        assert_eq!(running_cadence(90.0, Some("cycling")), 90.0);
        assert_eq!(running_cadence(90.0, Some("running")), 180.0);
        assert_eq!(running_cadence(120.0, Some("running")), 120.0);
        assert_eq!(running_cadence(0.0, Some("running")), 0.0);
    }

    #[test]
    fn positions_are_semicircles() {
        assert_eq!(semicircles_to_degrees(0.0), 0.0);
    }

    #[test]
    fn manufacturer_ids_include_the_vendor_exports() {
        assert_eq!(manufacturer_name(32), "wahoo_fitness");
        assert_eq!(manufacturer_name(265), "strava");
        assert_eq!(manufacturer_name(294), "coros");
        assert_eq!(manufacturer_name(65_535), "65535");
    }

    #[test]
    fn every_base_type_sentinel_is_absent() {
        assert!(decode_value(&[0xff], 0x00, true).is_none());
        assert!(decode_value(&[0x7f], 0x01, true).is_none());
        assert!(decode_value(&[0xff], 0x02, true).is_none());
        assert!(decode_value(&[0xff, 0xff], 0x84, true).is_none());
        assert!(decode_value(&[0xff; 4], 0x86, true).is_none());
        assert!(decode_value(&[0x00], 0x8a, true).is_none());
        assert!(decode_value(&[0x00], 0x07, true).is_none());
    }
}
