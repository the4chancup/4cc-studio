//! CRILAYLA, CRI's bitstream LZ compression, decode only.
//!
//! Layout: `CRILAYLA` magic (8 bytes), LE u32 compressed-stream payload size,
//! LE u32 offset of the raw prefix, then the bitstream and, at the end, 0x100
//! bytes stored raw. The raw prefix becomes the *first* 0x100 bytes of the
//! output; the bitstream is read from its end backwards, MSB-first, and fills
//! the rest of the output backwards too. There is no compressor here because
//! the legacy tools never produce CRILAYLA.

use crate::CpkError;

const PREFIX_LEN: usize = 0x100;

/// Inflates a CRILAYLA buffer to its payload.
///
/// # Errors
/// [`CpkError::Crilayla`] when the magic is wrong, the buffer is too short for
/// the header plus prefix, or the bitstream runs out (or a back-reference
/// points outside the output) before the payload is full.
pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, CpkError> {
    if bytes.len() < 16 || bytes[..8] != *b"CRILAYLA" {
        return Err(CpkError::Crilayla("bad magic"));
    }
    let payload_size = u32::from_le_bytes(bytes[8..12].try_into().unwrap_or([0; 4])) as usize;
    let prefix_offset = u32::from_le_bytes(bytes[12..16].try_into().unwrap_or([0; 4])) as usize;
    let prefix_start = 0x10usize
        .checked_add(prefix_offset)
        .ok_or(CpkError::Crilayla("buffer too short"))?;
    let prefix_end = prefix_start
        .checked_add(PREFIX_LEN)
        .ok_or(CpkError::Crilayla("buffer too short"))?;
    if prefix_end > bytes.len() {
        return Err(CpkError::Crilayla("buffer too short"));
    }
    let prefix = &bytes[prefix_start..prefix_end];
    let mut stream = ReverseBits::new(&bytes[0x10..prefix_start]);

    let mut payload = vec![0u8; payload_size];
    let mut written = 0usize;
    while written < payload.len() {
        if stream.read(1)? == 1 {
            // Back-reference into the already-written tail of the output.
            let offset = stream.read(13)? as usize + 3;
            let mut length = 3usize;
            for chunk_bits in [2u32, 3, 5].into_iter().chain(std::iter::repeat(8)) {
                let chunk = stream.read(chunk_bits)? as usize;
                length += chunk;
                if chunk + 1 != 1 << chunk_bits {
                    break;
                }
            }
            for _ in 0..length {
                let at = payload
                    .len()
                    .checked_sub(written + 1)
                    .ok_or(CpkError::Crilayla("back-reference out of range"))?;
                let source = at + offset;
                if source >= payload.len() {
                    return Err(CpkError::Crilayla("back-reference out of range"));
                }
                payload[at] = payload[source];
                written += 1;
            }
        } else {
            let byte = stream.read(8)? as u8;
            let at = payload.len() - written - 1;
            payload[at] = byte;
            written += 1;
        }
    }

    let mut output = Vec::with_capacity(PREFIX_LEN + payload.len());
    output.extend_from_slice(prefix);
    output.extend_from_slice(&payload);
    Ok(output)
}

/// MSB-first bits taken from the buffer's end backwards.
struct ReverseBits<'a> {
    bytes: &'a [u8],
    index: usize,
    pooled: u64,
    pooled_count: u32,
}

impl<'a> ReverseBits<'a> {
    fn new(bytes: &'a [u8]) -> ReverseBits<'a> {
        ReverseBits {
            bytes,
            index: 0,
            pooled: 0,
            pooled_count: 0,
        }
    }

    fn read(&mut self, bits: u32) -> Result<u64, CpkError> {
        while self.pooled_count < bits {
            if self.index >= self.bytes.len() {
                return Err(CpkError::Crilayla("bitstream exhausted"));
            }
            self.pooled =
                (self.pooled << 8) | u64::from(self.bytes[self.bytes.len() - self.index - 1]);
            self.pooled_count += 8;
            self.index += 1;
        }
        let result = self.pooled >> (self.pooled_count - bits);
        self.pooled_count -= bits;
        self.pooled &= (1 << self.pooled_count) - 1;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_decompress_to_the_expected_payloads() {
        let packed = include_bytes!("../tests/fixtures/crilayla/settings_json.crilayla");
        let plain = include_bytes!("../tests/fixtures/crilayla/settings_json.bin");
        assert_eq!(decompress(packed).unwrap(), plain);

        let packed = include_bytes!("../tests/fixtures/crilayla/symbol_816_dds.crilayla");
        let plain = include_bytes!("../tests/fixtures/crilayla/symbol_816_dds.bin");
        assert_eq!(decompress(packed).unwrap(), plain);
    }

    #[test]
    fn malformed_input_errors() {
        assert!(matches!(
            decompress(b"not-crilayla"),
            Err(CpkError::Crilayla("bad magic"))
        ));
        let mut header_only = *b"CRILAYLA\0\0\0\0\0\0\0\0";
        assert!(matches!(
            decompress(&header_only),
            Err(CpkError::Crilayla("buffer too short"))
        ));
        let packed = include_bytes!("../tests/fixtures/crilayla/settings_json.crilayla");
        header_only[..8].copy_from_slice(&packed[..8]);
        assert!(matches!(
            decompress(&packed[..200]),
            Err(CpkError::Crilayla("buffer too short"))
        ));
    }
}
