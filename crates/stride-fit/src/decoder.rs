//! A small, strict FIT decoder.
//!
//! The FIT format is a sequence of records in a data section: definition
//! messages describe the layout of the data messages that follow them, and
//! data messages carry the values. Everything here is bounded, checked and
//! fallible — a truncated or corrupt file is an error, never a panic.
//!
//! The decoder deliberately does *not* apply the profile's scale and offset;
//! it hands back the raw values and the definition numbers, and
//! [`crate::profile`] turns them into engineering units. That keeps the two
//! concerns separate: this module knows the binary format, the profile module
//! knows the units.

use std::collections::HashMap;

use crate::error::FitError;

/// Seconds between the Unix epoch and the FIT epoch (1989-12-31T00:00:00Z).
pub(crate) const FIT_EPOCH_UNIX: i64 = 631_065_600;

/// The field definition number of the FIT timestamp field.
pub(crate) const TIMESTAMP_DEF_NUM: u8 = 253;

/// A decoded field value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FieldValue {
    /// A number, still in the field's raw units.
    Number(f64),
    /// A UTF-8 string.
    Text(String),
}

impl FieldValue {
    /// The value as a number, if it is one.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            FieldValue::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// The value as text, if it is a string.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FieldValue::Text(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

/// One decoded FIT message.
#[derive(Debug, Clone)]
pub(crate) struct Message {
    /// The global message number.
    pub mesg_num: u16,
    /// The timestamp, when the record header carried a compressed one.
    pub compressed_timestamp: Option<u32>,
    /// The fields, in file order, as `(definition number, value)`.
    pub fields: Vec<(u8, FieldValue)>,
}

impl Message {
    /// The raw value of a field, by definition number.
    pub fn number(&self, def_num: u8) -> Option<f64> {
        self.fields
            .iter()
            .find(|(num, _)| *num == def_num)
            .and_then(|(_, value)| value.as_number())
    }

    /// The text of a field, by definition number.
    pub fn text(&self, def_num: u8) -> Option<&str> {
        self.fields
            .iter()
            .find(|(num, _)| *num == def_num)
            .and_then(|(_, value)| value.as_text())
    }
}

/// The layout of one local message type.
#[derive(Debug, Clone)]
struct Definition {
    mesg_num: u16,
    little_endian: bool,
    fields: Vec<FieldDefinition>,
    developer_bytes: usize,
}

/// One field of a message layout.
#[derive(Debug, Clone, Copy)]
struct FieldDefinition {
    def_num: u8,
    size: usize,
    base_type: u8,
}

/// Decode every message in a FIT file, in file order.
///
/// `visit` is called once per message; it may stop the walk by returning an
/// error.
pub(crate) fn for_each_message<F>(bytes: &[u8], mut visit: F) -> Result<(), FitError>
where
    F: FnMut(Message) -> Result<(), FitError>,
{
    let (data_start, data_end) = read_header(bytes)?;
    let mut cursor = data_start;

    // A 12-byte header has no CRC; the two bytes that follow it are the
    // file's time offset, and the records start after them.
    if data_start == 12 {
        cursor += 2;
    }

    let mut definitions: HashMap<u8, Definition> = HashMap::new();
    let mut last_timestamp: Option<u32> = None;

    while cursor < data_end {
        let header_offset = cursor;
        let header = bytes[cursor];
        cursor += 1;

        let compressed = header & 0x80 != 0;
        let (local, is_definition, developer_present) = if compressed {
            // A compressed timestamp header carries the local message type in
            // bits 5-6 and the five-bit time offset in bits 0-4.
            ((header >> 5) & 0x3, false, false)
        } else {
            (header & 0x0f, header & 0x40 != 0, header & 0x20 != 0)
        };

        if compressed {
            let offset = header & 0x1f;
            let timestamp = match last_timestamp {
                Some(last) => {
                    // The reference's compressed accumulation: the offset is
                    // added to the accumulated timestamp, and the field
                    // rolls over when the offset is smaller than the low
                    // five bits of the accumulator.
                    let mut value = u32::from(offset) + (last & !0x1f);
                    if u32::from(offset) < (last & 0x1f) {
                        value = value.wrapping_add(0x20);
                    }
                    value
                }
                None => u32::from(offset),
            };
            let message = read_data_message(
                bytes,
                &mut cursor,
                data_end,
                local,
                &definitions,
                Some(timestamp),
                header_offset,
            )?;
            if let Some(raw) = message.number(TIMESTAMP_DEF_NUM) {
                last_timestamp = Some(raw as u32);
            } else {
                last_timestamp = Some(timestamp);
            }
            visit(message)?;
            continue;
        }

        if is_definition {
            let definition =
                read_definition_message(bytes, &mut cursor, data_end, developer_present)?;
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
        if let Some(raw) = message.number(TIMESTAMP_DEF_NUM) {
            last_timestamp = Some(raw as u32);
        }
        visit(message)?;
    }

    Ok(())
}

/// Read the file header and return the bounds of the data section.
fn read_header(bytes: &[u8]) -> Result<(usize, usize), FitError> {
    if bytes.len() < 12 {
        return Err(FitError::Truncated {
            offset: 0,
            needed: 12,
            available: bytes.len(),
        });
    }
    let header_size = bytes[0] as usize;
    if header_size != 12 && header_size != 14 {
        return Err(FitError::InvalidHeader {
            reason: format!("header size is {header_size}, expected 12 or 14"),
        });
    }
    let protocol = bytes[1];
    if protocol != 0x10 && protocol != 0x20 {
        return Err(FitError::UnsupportedVersion {
            version: u16::from(protocol),
        });
    }
    if &bytes[8..12] != b".FIT" {
        return Err(FitError::NotFit);
    }
    let data_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let data_start = header_size;
    let data_end = data_start.saturating_add(data_size);
    if bytes.len() < data_end {
        return Err(FitError::Truncated {
            offset: data_start,
            needed: data_size,
            available: bytes.len().saturating_sub(data_start),
        });
    }
    Ok((data_start, data_end))
}

/// Read a definition message and remember its layout.
fn read_definition_message(
    bytes: &[u8],
    cursor: &mut usize,
    data_end: usize,
    developer_present: bool,
) -> Result<Definition, FitError> {
    let need = |cursor: &mut usize, count: usize| -> Result<(), FitError> {
        if *cursor + count > data_end {
            return Err(FitError::Truncated {
                offset: *cursor,
                needed: count,
                available: data_end.saturating_sub(*cursor),
            });
        }
        *cursor += count;
        Ok(())
    };

    // reserved byte, architecture byte
    need(cursor, 2)?;
    let architecture = bytes[*cursor - 1];
    let little_endian = architecture == 0;

    need(cursor, 2)?;
    let mesg_num = read_u16(&bytes[*cursor - 2..*cursor], little_endian).unwrap_or(0);

    need(cursor, 1)?;
    let field_count = bytes[*cursor - 1] as usize;

    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        need(cursor, 3)?;
        let tail = *cursor;
        fields.push(FieldDefinition {
            def_num: bytes[tail - 3],
            size: bytes[tail - 2] as usize,
            base_type: bytes[tail - 1],
        });
    }

    let mut developer_bytes = 0usize;
    if developer_present {
        need(cursor, 1)?;
        let developer_count = bytes[*cursor - 1] as usize;
        for _ in 0..developer_count {
            need(cursor, 3)?;
            // A developer field definition is: number, size, developer data
            // index.
            developer_bytes += bytes[*cursor - 2] as usize;
        }
    }

    Ok(Definition {
        mesg_num,
        little_endian,
        fields,
        developer_bytes,
    })
}

/// Read one data message using the layout of its local message type.
#[allow(clippy::too_many_arguments)]
fn read_data_message(
    bytes: &[u8],
    cursor: &mut usize,
    data_end: usize,
    local: u8,
    definitions: &HashMap<u8, Definition>,
    compressed_timestamp: Option<u32>,
    offset: usize,
) -> Result<Message, FitError> {
    let definition = definitions
        .get(&local)
        .ok_or(FitError::UnknownLocalMessage { offset, local })?;

    let mut fields = Vec::with_capacity(definition.fields.len());
    for field in &definition.fields {
        if *cursor + field.size > data_end {
            return Err(FitError::Truncated {
                offset: *cursor,
                needed: field.size,
                available: data_end.saturating_sub(*cursor),
            });
        }
        let raw = &bytes[*cursor..*cursor + field.size];
        *cursor += field.size;
        if let Some(value) = decode_value(raw, field.base_type, definition.little_endian) {
            fields.push((field.def_num, value));
        }
    }

    // Developer fields are carried at the end of the data message.
    if definition.developer_bytes > 0 {
        if *cursor + definition.developer_bytes > data_end {
            return Err(FitError::Truncated {
                offset: *cursor,
                needed: definition.developer_bytes,
                available: data_end.saturating_sub(*cursor),
            });
        }
        *cursor += definition.developer_bytes;
    }

    Ok(Message {
        mesg_num: definition.mesg_num,
        compressed_timestamp,
        fields,
    })
}

/// Decode one field's raw bytes.
///
/// Every FIT base type has a *null value*: the bit pattern that means "not
/// set". A watch writes it when it has nothing to say, and the reference
/// decoder turns it into an absent field. Reading it as a number is how a
/// missing average speed becomes an average speed of 65.535 m/s.
fn decode_value(raw: &[u8], base_type: u8, little_endian: bool) -> Option<FieldValue> {
    /// A number, unless it is the base type's null value.
    macro_rules! number {
        ($value:expr, $null:expr) => {
            match $value {
                Some(value) if value != $null => Some(FieldValue::Number(value as f64)),
                _ => None,
            }
        };
    }

    match base_type {
        0x00 => number!(raw.first().copied(), 0xff), // enum
        0x01 => number!(raw.first().copied().map(|value| value as i8), 0x7f), // sint8
        0x02 => number!(raw.first().copied(), 0xff), // uint8
        0x8a => number!(raw.first().copied(), 0x00), // uint8z
        0x83 => number!(read_i16(raw, little_endian), 0x7fff), // sint16
        0x84 => number!(read_u16(raw, little_endian), 0xffff), // uint16
        0x8b => number!(read_u16(raw, little_endian), 0x0000), // uint16z
        0x85 => number!(read_i32(raw, little_endian), 0x7fff_ffff), // sint32
        0x86 => number!(read_u32(raw, little_endian), 0xffff_ffff), // uint32
        0x8c => number!(read_u32(raw, little_endian), 0x0000_0000), // uint32z
        0x8e => number!(read_i64(raw, little_endian), 0x7fff_ffff_ffff_ffff), // sint64
        0x8f => number!(read_u64(raw, little_endian), u64::MAX), // uint64
        0x90 => number!(read_u64(raw, little_endian), 0), // uint64z
        0x88 => read_f32(raw, little_endian)
            .filter(|value| !value.is_nan())
            .map(|value| FieldValue::Number(f64::from(value))), // float32
        0x89 => read_f64(raw, little_endian)
            .filter(|value| !value.is_nan())
            .map(FieldValue::Number), // float64
        0x07 => {
            // string, null terminated
            let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
            let text = String::from_utf8_lossy(&raw[..end]).into_owned();
            if text.is_empty() {
                None
            } else {
                Some(FieldValue::Text(text))
            }
        }
        0x0d => {
            // A byte array; the bytes are consumed but no field Stride reads
            // is of this type.
            None
        }
        // An unknown base type: the bytes are consumed by the caller, but the
        // value is not usable.
        _ => None,
    }
}

fn read_u16(raw: &[u8], little_endian: bool) -> Option<u16> {
    if raw.len() < 2 {
        return None;
    }
    let bytes = [raw[0], raw[1]];
    Some(if little_endian {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    })
}

fn read_i16(raw: &[u8], little_endian: bool) -> Option<i16> {
    read_u16(raw, little_endian).map(|value| value as i16)
}

fn read_u32(raw: &[u8], little_endian: bool) -> Option<u32> {
    if raw.len() < 4 {
        return None;
    }
    let bytes = [raw[0], raw[1], raw[2], raw[3]];
    Some(if little_endian {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    })
}

fn read_i32(raw: &[u8], little_endian: bool) -> Option<i32> {
    read_u32(raw, little_endian).map(|value| value as i32)
}

fn read_f32(raw: &[u8], little_endian: bool) -> Option<f32> {
    read_u32(raw, little_endian).map(f32::from_bits)
}

fn read_u64(raw: &[u8], little_endian: bool) -> Option<u64> {
    if raw.len() < 8 {
        return None;
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&raw[..8]);
    Some(if little_endian {
        u64::from_le_bytes(bytes)
    } else {
        u64::from_be_bytes(bytes)
    })
}

fn read_i64(raw: &[u8], little_endian: bool) -> Option<i64> {
    read_u64(raw, little_endian).map(|value| value as i64)
}

fn read_f64(raw: &[u8], little_endian: bool) -> Option<f64> {
    read_u64(raw, little_endian).map(f64::from_bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal, valid FIT file: a 14-byte header, one definition message
    /// and one data message.
    fn tiny_fit() -> Vec<u8> {
        let mut bytes = Vec::new();
        // A definition message (local 0, record, one field: the timestamp)
        // followed by one data message.
        let mut body = vec![
            0x40, // definition, local message type 0
            0,    // reserved
            0,    // little endian
            20,   // global message number: record
            0,    //
            1,    // one field
            253,  // timestamp
            4,    // size
            0x86, // uint32
        ];
        body.push(0x00); // data, local message type 0
        body.extend_from_slice(&1_000_000u32.to_le_bytes());

        bytes.push(14); // header size
        bytes.push(0x20); // protocol version
        bytes.extend_from_slice(&2140u16.to_le_bytes());
        bytes.extend_from_slice(&(body.len() as u32).to_le_bytes());
        bytes.extend_from_slice(b".FIT");
        bytes.extend_from_slice(&[0, 0]); // header CRC
        bytes.extend_from_slice(&body);
        bytes
    }

    #[test]
    fn reads_a_minimal_file() {
        let bytes = tiny_fit();
        let mut seen = Vec::new();
        for_each_message(&bytes, |message| {
            seen.push(message);
            Ok(())
        })
        .expect("the file parses");
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].mesg_num, 20);
        assert_eq!(seen[0].number(253), Some(1_000_000.0));
    }

    #[test]
    fn a_short_file_is_an_error() {
        let error = for_each_message(&[], |_| Ok(())).unwrap_err();
        assert!(matches!(error, FitError::Truncated { .. }), "{error}");
    }

    #[test]
    fn a_file_without_the_magic_is_an_error() {
        let mut bytes = tiny_fit();
        bytes[8] = b'X';
        let error = for_each_message(&bytes, |_| Ok(())).unwrap_err();
        assert!(matches!(error, FitError::NotFit), "{error}");
    }

    #[test]
    fn an_unsupported_protocol_is_an_error() {
        let mut bytes = tiny_fit();
        bytes[1] = 0x99;
        let error = for_each_message(&bytes, |_| Ok(())).unwrap_err();
        assert!(
            matches!(error, FitError::UnsupportedVersion { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_truncated_body_is_an_error() {
        let bytes = tiny_fit();
        let error = for_each_message(&bytes[..bytes.len() - 3], |_| Ok(())).unwrap_err();
        assert!(matches!(error, FitError::Truncated { .. }), "{error}");
    }

    #[test]
    fn a_data_message_without_a_definition_is_an_error() {
        let mut bytes = tiny_fit();
        // Claim a data message for a local type that was never defined.
        bytes[14 + 9] = 0x05;
        let error = for_each_message(&bytes, |_| Ok(())).unwrap_err();
        assert!(
            matches!(error, FitError::UnknownLocalMessage { .. }),
            "{error}"
        );
    }

    #[test]
    fn reads_a_file_with_developer_fields() {
        // A definition message with the developer-data flag set and one
        // developer field of two bytes, followed by a data message that
        // carries those two bytes at the end.
        let mut body = vec![
            0x60, // definition, developer data present, local message type 0
            0,    // reserved
            0,    // little endian
            20,   // global message number: record
            0,    //
            1,    // one field
            253,  // timestamp
            4,    // size
            0x86, // uint32
            1,    // one developer field
            0,    // field number
            2,    // size
            0,    // developer data index
        ];
        body.push(0x00); // data, local message type 0
        body.extend_from_slice(&2_000_000u32.to_le_bytes());
        body.extend_from_slice(&[0xaa, 0xbb]); // the developer field's bytes

        let mut bytes = Vec::new();
        bytes.push(14);
        bytes.push(0x20);
        bytes.extend_from_slice(&2140u16.to_le_bytes());
        bytes.extend_from_slice(&(body.len() as u32).to_le_bytes());
        bytes.extend_from_slice(b".FIT");
        bytes.extend_from_slice(&[0, 0]);
        bytes.extend_from_slice(&body);

        let mut seen = Vec::new();
        for_each_message(&bytes, |message| {
            seen.push(message);
            Ok(())
        })
        .expect("the file parses");
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].number(253), Some(2_000_000.0));
    }

    #[test]
    fn a_null_value_is_an_absent_field() {
        // Every FIT base type has a null value; a watch writes it when it has
        // nothing to say. Reading it as a number turns a missing average
        // speed into an average speed of 65.535 m/s.
        assert_eq!(decode_value(&[0xff], 0x02, true), None); // uint8
        assert_eq!(decode_value(&[0x7f], 0x01, true), None); // sint8
        assert_eq!(decode_value(&[0x00], 0x8a, true), None); // uint8z
        assert_eq!(decode_value(&[0xff, 0xff], 0x84, true), None); // uint16
        assert_eq!(decode_value(&[0xff, 0x7f], 0x83, true), None); // sint16
        assert_eq!(decode_value(&[0x00, 0x00], 0x8b, true), None); // uint16z
        assert_eq!(decode_value(&[0xff, 0xff, 0xff, 0xff], 0x86, true), None); // uint32
        assert_eq!(decode_value(&[0xff, 0xff, 0xff, 0x7f], 0x85, true), None); // sint32
        assert_eq!(decode_value(&[0x00, 0x00, 0x00, 0x00], 0x8c, true), None); // uint32z
        assert_eq!(decode_value(&[0xff; 8], 0x8f, true), None); // uint64
        assert_eq!(decode_value(&[0xff; 4], 0x00, true), None); // enum
                                                                // and a real value still decodes
        assert_eq!(
            decode_value(&[0xe8, 0x03], 0x84, true),
            Some(FieldValue::Number(1000.0))
        );
        assert_eq!(
            decode_value(&[0x64], 0x02, true),
            Some(FieldValue::Number(100.0))
        );
        assert_eq!(
            decode_value(&[0x64], 0x8a, true),
            Some(FieldValue::Number(100.0)),
            "a uint8z field is a number, not an absent field"
        );
    }

    #[test]
    fn corruption_never_panics() {
        let bytes = tiny_fit();
        for index in 0..bytes.len() {
            for bit in 0..8 {
                let mut broken = bytes.clone();
                broken[index] ^= 1 << bit;
                // The result may be an error or a parse; it must never panic.
                let _ = for_each_message(&broken, |_| Ok(()));
            }
        }
    }
}
