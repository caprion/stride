//! Compression.
//!
//! The Python app compresses its JSON with `GZipMiddleware`: a browsing
//! session pulled about 1.1 MB of JSON, and the same responses are 131 KB,
//! 13 KB and 9 KB once they are compressed.
//!
//! Two rules, and the second is the one that matters:
//!
//! * a body is compressed only when the client says it can read a gzip
//!   stream;
//! * a body is compressed only when it is worth compressing. A compressed
//!   40-byte body is *larger* than the 40-byte body, and the CPU is spent
//!   either way.
//!
//! The threshold is Starlette's: 500 bytes. The reference's middleware uses
//! the same number, so the two servers behave alike.

use std::io::Write;

use crate::error::BackendError;

/// The smallest body worth compressing.
///
/// The reference's middleware uses the same number.
pub const GZIP_MINIMUM_SIZE: usize = 500;

/// Compress a body with gzip.
///
/// The answer is the gzip stream: the magic number, the deflate data and
/// the trailer, which is what `Content-Encoding: gzip` means.
pub fn gzip(bytes: &[u8]) -> Result<Vec<u8>, BackendError> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(bytes)
        .map_err(|err| BackendError::Domain(format!("the body could not be compressed: {err}")))?;
    encoder
        .finish()
        .map_err(|err| BackendError::Domain(format!("the body could not be compressed: {err}")))
}

/// Whether a body of this size is worth compressing.
pub fn is_worth_compressing(len: usize) -> bool {
    len >= GZIP_MINIMUM_SIZE
}

/// Compress a body if that makes it smaller, and say how it was encoded.
///
/// The answer is the compressed body and the encoding to advertise. When the
/// body is small, or when the compressed form would not be smaller, the
/// body is returned as it is and the encoding is `None`.
pub fn compress_if_worth_it(
    bytes: Vec<u8>,
) -> Result<(Vec<u8>, Option<&'static str>), BackendError> {
    if !is_worth_compressing(bytes.len()) {
        return Ok((bytes, None));
    }
    let compressed = gzip(&bytes)?;
    if compressed.len() >= bytes.len() {
        // The data does not compress — random bytes never do — and a
        // client that asked for gzip can read the plain body as well.
        return Ok((bytes, None));
    }
    Ok((compressed, Some("gzip")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_body_is_left_alone() {
        let (body, encoding) = compress_if_worth_it(b"{\"ok\":true}".to_vec()).expect("it works");
        assert_eq!(body, b"{\"ok\":true}");
        assert_eq!(encoding, None);
    }

    #[test]
    fn a_big_body_is_compressed() {
        let payload = "x".repeat(4_000);
        let (body, encoding) =
            compress_if_worth_it(payload.clone().into_bytes()).expect("it works");
        assert_eq!(encoding, Some("gzip"));
        assert!(body.len() < payload.len());
        // The stream is a gzip stream: the magic number is there.
        assert_eq!(&body[..2], &[0x1f, 0x8b]);
    }

    #[test]
    fn a_body_that_does_not_compress_is_left_alone() {
        // 4 KB of noise: the compressed form is bigger than the plain one.
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let noise: Vec<u8> = (0..4_096)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state & 0xff) as u8
            })
            .collect();
        let (body, encoding) = compress_if_worth_it(noise.clone()).expect("it works");
        assert_eq!(encoding, None);
        assert_eq!(body, noise);
    }
}
