//! The WESYS zlib wrapper PES puts around compressed files.
//!
//! Format: a 16-byte little-endian header `00 10 01 'W' 'E' 'S' 'Y' 'S'`, the
//! compressed length as u32, the uncompressed length as u32, then a zlib
//! stream. Files may also be stored unwrapped; [`is_wrapped`] tells them apart.

use std::borrow::Cow;
use std::io::Read;
use std::io::Write;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

const HEADER_LEN: usize = 16;
const HEADER_PREFIX: [u8; 8] = [0x00, 0x10, 0x01, b'W', b'E', b'S', b'Y', b'S'];

/// Why a buffer could not be unwrapped and inflated.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The input does not carry the 16-byte WESYS header.
    #[error("input is not WESYS-wrapped")]
    NotWrapped,
    /// The zlib stream after the header is corrupt or truncated.
    #[error("corrupt zlib stream: {0}")]
    Zlib(#[from] std::io::Error),
    /// The inflated payload length differs from the header's uncompressed
    /// length.
    #[error("uncompressed length mismatch: header says {expected}, got {actual}")]
    LengthMismatch {
        /// The uncompressed length the header declares.
        expected: u32,
        /// The length the zlib stream actually produced.
        actual: usize,
    },
}

/// True when `bytes` carry the WESYS header: at least 16 bytes with `WESYS` at
/// offset 3. The leading three bytes are not checked: game files carry both
/// `00 10 01` and `00 10 11` there.
pub fn is_wrapped(bytes: &[u8]) -> bool {
    bytes.len() >= HEADER_LEN && bytes[3..8] == *b"WESYS"
}

/// Unwraps and inflates a WESYS buffer. At most one byte past the declared
/// uncompressed length is inflated, so a stream longer than the header
/// says is [`Error::LengthMismatch`] without being held whole. Errors when
/// the input is not wrapped ([`Error::NotWrapped`]), the zlib stream is
/// corrupt ([`Error::Zlib`]), or the payload length differs from the
/// header's uncompressed length ([`Error::LengthMismatch`]).
pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    if !is_wrapped(bytes) {
        return Err(Error::NotWrapped);
    }
    let expected = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    let payload = inflate(&bytes[HEADER_LEN..], u64::from(expected) + 1)?;
    if payload.len() != expected as usize {
        return Err(Error::LengthMismatch {
            expected,
            actual: payload.len(),
        });
    }
    Ok(payload)
}

/// [`decompress`] when wrapped, the input itself otherwise.
pub fn decompress_if_wrapped(bytes: &[u8]) -> Result<Cow<'_, [u8]>, Error> {
    if is_wrapped(bytes) {
        Ok(Cow::Owned(decompress(bytes)?))
    } else {
        Ok(Cow::Borrowed(bytes))
    }
}

/// Deflates with zlib default compression and wraps the result in the WESYS
/// header. Always wraps.
///
/// The deflate bytes are miniz_oxide's, not zlib's, so a file compressed here
/// decompresses to the same payload as one compressed by zlib but is not
/// byte-identical to it: parity for WESYS-wrapped files is at the payload
/// level.
pub fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    // Writing to a Vec cannot fail.
    encoder
        .write_all(bytes)
        .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    let body = encoder
        .finish()
        .unwrap_or_else(|_| unreachable!("Vec finish is infallible"));
    let mut output = Vec::with_capacity(HEADER_LEN + body.len());
    output.extend_from_slice(&HEADER_PREFIX);
    output.extend_from_slice(&(body.len() as u32).to_le_bytes());
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(&body);
    output
}

fn inflate(stream: &[u8], limit: u64) -> Result<Vec<u8>, Error> {
    let mut payload = Vec::new();
    ZlibDecoder::new(stream)
        .take(limit)
        .read_to_end(&mut payload)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WRAPPED: &[u8] = include_bytes!("../tests/fixtures/RefereeColor.bin.wesys");
    const PAYLOAD: &[u8] = include_bytes!("../tests/fixtures/RefereeColor.bin");

    #[test]
    fn fixture_decompresses_to_the_expected_payload() {
        assert_eq!(decompress(WRAPPED).unwrap(), PAYLOAD);
        assert_eq!(decompress_if_wrapped(WRAPPED).unwrap().as_ref(), PAYLOAD);
        assert_eq!(decompress_if_wrapped(PAYLOAD).unwrap().as_ref(), PAYLOAD);
    }

    #[test]
    fn is_wrapped_recognizes_the_header() {
        assert!(is_wrapped(WRAPPED));
        assert!(!is_wrapped(PAYLOAD));
        assert!(!is_wrapped(&WRAPPED[..10]));
    }

    #[test]
    fn compress_round_trips_through_decompress() {
        let wrapped = compress(PAYLOAD);
        assert_eq!(&wrapped[..3], &[0x00, 0x10, 0x01]);
        assert_eq!(&wrapped[3..8], b"WESYS");
        assert_eq!(
            u32::from_le_bytes(wrapped[8..12].try_into().unwrap()) as usize,
            wrapped.len() - HEADER_LEN
        );
        assert_eq!(
            u32::from_le_bytes(wrapped[12..16].try_into().unwrap()),
            PAYLOAD.len() as u32
        );
        assert_eq!(decompress(&wrapped).unwrap(), PAYLOAD);
    }

    #[test]
    fn an_oversized_declaration_inflates_only_one_byte_past_it() {
        // 64 MiB of zeros compressed to ~62 KiB; the wrapper declares one
        // byte, so at most expected + 1 is inflated: the mismatch reports
        // 2 bytes produced, not the stream's whole 64 MiB.
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(3));
        encoder.write_all(&[0u8; 64 << 20]).unwrap();
        let stream = encoder.finish().unwrap();
        let mut wrapped = HEADER_PREFIX.to_vec();
        wrapped.extend_from_slice(&(stream.len() as u32).to_le_bytes());
        wrapped.extend_from_slice(&1u32.to_le_bytes());
        wrapped.extend_from_slice(&stream);
        assert!(matches!(
            decompress(&wrapped),
            Err(Error::LengthMismatch {
                expected: 1,
                actual: 2
            })
        ));
    }

    #[test]
    fn a_stream_cut_past_the_needed_bytes_still_reports_the_length() {
        // The stream's bytes past expected + 1 are missing: a bounded
        // inflate never reaches them, so the answer is LengthMismatch
        // rather than Zlib.
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(3));
        let content: Vec<u8> = (0..512u16).map(|i| (i % 256) as u8).collect();
        encoder.write_all(&content).unwrap();
        let mut stream = encoder.finish().unwrap();
        stream.truncate(stream.len() / 2);
        let mut wrapped = HEADER_PREFIX.to_vec();
        wrapped.extend_from_slice(&(stream.len() as u32).to_le_bytes());
        wrapped.extend_from_slice(&100u32.to_le_bytes());
        wrapped.extend_from_slice(&stream);
        assert!(matches!(
            decompress(&wrapped),
            Err(Error::LengthMismatch {
                expected: 100,
                actual: 101
            })
        ));
    }

    #[test]
    fn decompress_rejects_unwrapped_and_corrupt_input() {
        assert!(matches!(decompress(PAYLOAD), Err(Error::NotWrapped)));
        assert!(matches!(decompress(&WRAPPED[..100]), Err(Error::Zlib(_))));
        let mut patched = WRAPPED.to_vec();
        patched[12..16].copy_from_slice(&999u32.to_le_bytes());
        assert!(matches!(
            decompress(&patched),
            Err(Error::LengthMismatch {
                expected: 999,
                actual: 255
            })
        ));
    }
}
