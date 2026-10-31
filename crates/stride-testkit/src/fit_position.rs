//! A minimal FIT reader that extracts only `position_lat` / `position_long`
//! from the `record` message.
//!
//! This is **test infrastructure** for the privacy gate — it proves no corpus
//! FIT file carries a real coordinate — not the product FIT decoder (that is
//! Slice 2). It reads just enough of the FIT format (header, definition/data
//! records, the record message's two position fields) to answer one question:
//! is every position in this file within 1° of the synthetic anchor
//! (-35.0, -140.0)?
//!
//! Fail-safe by construction: anything it cannot decode is an error, and an
//! error fails the privacy check rather than passing it.

use std::path::Path;

pub const ANCHOR_LAT: f64 = -35.0;
pub const ANCHOR_LON: f64 = -140.0;
pub const ANCHOR_RADIUS_DEGREES: f64 = 1.0;

const GLOBAL_RECORD: u16 = 20;
const FIELD_POSITION_LAT: u8 = 0;
const FIELD_POSITION_LON: u8 = 1;
const FIELD_TIMESTAMP: u8 = 253;

// The FIT "no value" sentinel for sint32: a position of 0x7FFFFFFF semicircles
// is an absent position, not a coordinate (it would decode to 179.999…°).
const SINT32_INVALID: i64 = 0x7FFF_FFFF;

const BASE_ENUM: u8 = 0x00;
const BASE_SINT8: u8 = 0x01;
const BASE_UINT8: u8 = 0x02;
const BASE_SINT16: u8 = 0x83;
const BASE_UINT16: u8 = 0x84;
const BASE_SINT32: u8 = 0x85;
const BASE_UINT32: u8 = 0x86;
const BASE_STRING: u8 = 0x07;
const BASE_FLOAT32: u8 = 0x88;
const BASE_FLOAT64: u8 = 0x89;
const BASE_UINT8Z: u8 = 0x0A;
const BASE_UINT16Z: u8 = 0x8B;
const BASE_UINT32Z: u8 = 0x8C;
const BASE_BYTE_ARRAY: u8 = 0x0D;
const BASE_SINT64: u8 = 0x8E;
const BASE_UINT64: u8 = 0x8F;
const BASE_UINT64Z: u8 = 0x90;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Unsupported(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::Unsupported(m) => write!(f, "unsupported FIT: {m}"),
        }
    }
}

struct Field {
    def_num: u8,
    size: u8,
    base_type: u8,
}

struct Definition {
    global: u16,
    arch: u8,
    fields: Vec<Field>,
}

/// Every decoded (latitude, longitude) position in the file, in degrees.
pub fn positions_in(path: &Path) -> Result<Vec<(f64, f64)>, Error> {
    let bytes = std::fs::read(path)?;
    positions_from_bytes(&bytes)
}

/// Whether every position in `path` is within 1° of the synthetic anchor.
pub fn all_positions_at_anchor(path: &Path) -> Result<bool, Error> {
    let positions = positions_in(path)?;
    Ok(positions.iter().all(|&(lat, lon)| within_anchor(lat, lon)))
}

pub fn within_anchor(lat: f64, lon: f64) -> bool {
    degree_distance(lat, lon, ANCHOR_LAT, ANCHOR_LON) <= ANCHOR_RADIUS_DEGREES
}

fn degree_distance(a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    ((a_lat - b_lat).powi(2) + (a_lon - b_lon).powi(2)).sqrt()
}

pub fn positions_from_bytes(bytes: &[u8]) -> Result<Vec<(f64, f64)>, Error> {
    if bytes.len() < 12 {
        return Err(Error::Unsupported(
            "file shorter than the 12-byte header".into(),
        ));
    }
    let header_size = bytes[0] as usize;
    if header_size != 12 && header_size != 14 {
        return Err(Error::Unsupported("bad header size".into()));
    }
    if bytes.len() < header_size || &bytes[8..12] != b".FIT" {
        return Err(Error::Unsupported("bad FIT header".into()));
    }
    let data_size = read_u32(bytes, 4, 0)? as usize;
    let end = header_size
        .checked_add(data_size)
        .ok_or_else(|| Error::Unsupported("FIT data size overflow".into()))?;
    if bytes.len() < end {
        return Err(Error::Unsupported("truncated FIT data section".into()));
    }

    const NONE: Option<Definition> = None;
    let mut defs: [Option<Definition>; 16] = [NONE; 16];
    let mut out = Vec::new();

    let mut i = if header_size == 12 { 14 } else { header_size };
    while i < end {
        let hdr = bytes[i];

        // Compressed-timestamp header (bit 7): the local message type lives
        // in bits 5-6, and the timestamp field is omitted from the data body.
        if hdr & 0x80 != 0 {
            let local = ((hdr >> 5) & 0x03) as usize;
            let def = match &defs[local] {
                Some(d) => d,
                None => {
                    return Err(Error::Unsupported(
                        "compressed timestamp before a definition".into(),
                    ))
                }
            };
            if !def
                .fields
                .iter()
                .any(|field| field.def_num == FIELD_TIMESTAMP)
            {
                return Err(Error::Unsupported(
                    "compressed timestamp without a timestamp field".into(),
                ));
            }
            let (next, lat, lon) = parse_record_data(bytes, i, def, true)?;
            if let (Some(la), Some(lo)) = (lat, lon) {
                out.push((la, lo));
            }
            i = next;
            continue;
        }

        let local = (hdr & 0x0F) as usize;
        if hdr & 0x40 != 0 {
            i = parse_definition(bytes, i, hdr, &mut defs)?;
        } else {
            let def = match &defs[local] {
                Some(d) => d,
                None => return Err(Error::Unsupported("data before a definition".into())),
            };
            if hdr & 0x20 != 0 {
                return Err(Error::Unsupported(
                    "developer data in a data message".into(),
                ));
            }
            let (next, lat, lon) = parse_record_data(bytes, i, def, false)?;
            if let (Some(la), Some(lo)) = (lat, lon) {
                out.push((la, lo));
            }
            i = next;
        }
    }
    Ok(out)
}

fn parse_definition(
    bytes: &[u8],
    i: usize,
    hdr: u8,
    defs: &mut [Option<Definition>; 16],
) -> Result<usize, Error> {
    let local = (hdr & 0x0F) as usize;
    let mut j = i + 1;
    if hdr & 0x20 != 0 {
        // developer field count, then three bytes per developer field
        let dev_count = bytes
            .get(j)
            .copied()
            .ok_or_else(|| Error::Unsupported("truncated definition message".into()))?
            as usize;
        j += 1 + dev_count * 3;
        if j > bytes.len() {
            return Err(Error::Unsupported("truncated definition message".into()));
        }
    }
    if j + 5 >= bytes.len() {
        return Err(Error::Unsupported("truncated definition message".into()));
    }
    j += 1; // reserved
    let arch = bytes[j];
    j += 1;
    let global = read_u16(bytes, j, arch)?;
    j += 2;
    let field_count = bytes[j] as usize;
    j += 1;
    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        if j + 3 > bytes.len() {
            return Err(Error::Unsupported("truncated field definition".into()));
        }
        fields.push(Field {
            def_num: bytes[j],
            size: bytes[j + 1],
            base_type: bytes[j + 2],
        });
        j += 3;
    }
    defs[local] = Some(Definition {
        global,
        arch,
        fields,
    });
    Ok(j)
}

fn parse_record_data(
    bytes: &[u8],
    i: usize,
    def: &Definition,
    compressed_timestamp: bool,
) -> Result<(usize, Option<f64>, Option<f64>), Error> {
    let mut j = i + 1;
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    for field in &def.fields {
        if compressed_timestamp && field.def_num == FIELD_TIMESTAMP {
            continue;
        }
        let (consumed, value) = read_field(bytes, j, field, def.arch)?;
        if def.global == GLOBAL_RECORD {
            if field.def_num == FIELD_POSITION_LAT && value != SINT32_INVALID {
                lat = Some(semicircles_to_degrees(value));
            } else if field.def_num == FIELD_POSITION_LON && value != SINT32_INVALID {
                lon = Some(semicircles_to_degrees(value));
            }
        }
        j += consumed;
    }
    Ok((j, lat, lon))
}

/// Read one field, returning the bytes consumed and its numeric value (0 for
/// non-numeric fields). Strings are the only variable-length field.
fn read_field(bytes: &[u8], pos: usize, field: &Field, arch: u8) -> Result<(usize, i64), Error> {
    let size = field.size as usize;
    let base = field.base_type;
    if size == 0 {
        return Err(Error::Unsupported(format!(
            "zero-sized field at byte {pos}"
        )));
    }
    let raw = bytes
        .get(pos..pos.saturating_add(size))
        .filter(|raw| raw.len() == size)
        .ok_or_else(|| Error::Unsupported(format!("truncated field at byte {pos}")))?;
    match base {
        BASE_ENUM | BASE_UINT8 | BASE_UINT8Z => Ok((1, raw[0] as i64)),
        BASE_SINT8 => Ok((1, raw[0] as i8 as i64)),
        BASE_SINT16 => Ok((2, read_u16(bytes, pos, arch)? as i16 as i64)),
        BASE_UINT16 | BASE_UINT16Z => Ok((2, read_u16(bytes, pos, arch)? as i64)),
        BASE_SINT32 => Ok((4, read_u32(bytes, pos, arch)? as i32 as i64)),
        BASE_UINT32 | BASE_UINT32Z => Ok((4, read_u32(bytes, pos, arch)? as i64)),
        BASE_SINT64 => Ok((8, read_u64(bytes, pos, arch)? as i64)),
        BASE_UINT64 | BASE_UINT64Z => Ok((8, read_u64(bytes, pos, arch)? as i64)),
        BASE_FLOAT32 => Ok((4, 0)),
        BASE_FLOAT64 => Ok((8, 0)),
        BASE_BYTE_ARRAY => Ok((size, 0)),
        // A FIT string field always occupies exactly `size` bytes, null-padded
        // (the definition's size is the field width, not a maximum).
        BASE_STRING => Ok((size, 0)),
        other => Err(Error::Unsupported(format!(
            "base type {other:#x} at byte {pos}"
        ))),
    }
}

fn read_u16(bytes: &[u8], pos: usize, arch: u8) -> Result<u16, Error> {
    let b = bytes
        .get(pos..pos + 2)
        .ok_or_else(|| Error::Unsupported("truncated u16".into()))?;
    Ok(if arch == 0 {
        u16::from_le_bytes([b[0], b[1]])
    } else {
        u16::from_be_bytes([b[0], b[1]])
    })
}

fn read_u32(bytes: &[u8], pos: usize, arch: u8) -> Result<u32, Error> {
    let b = bytes
        .get(pos..pos + 4)
        .ok_or_else(|| Error::Unsupported("truncated u32".into()))?;
    Ok(if arch == 0 {
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    } else {
        u32::from_be_bytes([b[0], b[1], b[2], b[3]])
    })
}

fn read_u64(bytes: &[u8], pos: usize, arch: u8) -> Result<u64, Error> {
    let b = bytes
        .get(pos..pos + 8)
        .ok_or_else(|| Error::Unsupported("truncated u64".into()))?;
    Ok(if arch == 0 {
        u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    } else {
        u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    })
}

fn semicircles_to_degrees(semicircles: i64) -> f64 {
    semicircles as f64 * 180.0 / 2_147_483_648.0
}
