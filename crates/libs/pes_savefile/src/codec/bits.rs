//! Bit-run reads and writes. Every run is LSB-first little-endian across byte
//! boundaries: field bit `i` is bit `(offset + i) % 8` of byte `(offset + i) / 8`.
//! A run is at most 32 bits, so a 40-bit little-endian window covers it.

/// The `width` low bits set.
fn mask(width: u32) -> u64 {
    if width >= 32 {
        u64::from(u32::MAX)
    } else {
        (1u64 << width) - 1
    }
}

/// The record's bytes at `bit_offset / 8` onward as a little-endian u64 window.
fn window(data: &[u8], byte_offset: usize) -> u64 {
    let mut window = 0u64;
    for (i, &b) in data[byte_offset..].iter().take(8).enumerate() {
        window |= u64::from(b) << (8 * i);
    }
    window
}

/// The `width`-bit run at `bit_offset`, LSB-first.
pub(crate) fn read_bits(data: &[u8], bit_offset: u32, width: u32) -> u32 {
    debug_assert!(width <= 32, "a bit run is at most 32 bits");
    let shift = bit_offset % 8;
    ((window(data, (bit_offset / 8) as usize) >> shift) & mask(width)) as u32
}

/// Patches `value` into the `width`-bit run at `bit_offset`, LSB-first. Every
/// bit outside the run keeps its value; `value` must already fit `width`.
pub(crate) fn write_bits(data: &mut [u8], bit_offset: u32, width: u32, value: u32) {
    debug_assert!(width <= 32, "a bit run is at most 32 bits");
    debug_assert!(
        width >= 32 || u64::from(value) < (1u64 << width),
        "write_bits callers refuse a value too wide for its run"
    );
    let byte = (bit_offset / 8) as usize;
    let take = (data.len() - byte).min(8);
    let shift = bit_offset % 8;
    let keep = !(mask(width) << shift);
    let mut w = window(data, byte);
    w = (w & keep) | (u64::from(value) << shift);
    for i in 0..take {
        data[byte + i] = (w >> (8 * i)) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_crossing_a_byte_boundary_reads_and_writes() {
        let mut buf = [0xFFu8; 3];
        // A 7-bit run at bit 5 spans byte 0 bit 5 through byte 1 bit 3.
        write_bits(&mut buf, 5, 7, 0b101_1010);
        assert_eq!(read_bits(&buf, 5, 7), 0b101_1010);
        assert_eq!(buf, [0x5F, 0xFB, 0xFF]);
    }

    #[test]
    fn a_write_touches_only_the_runs_bits() {
        let mut buf = [0xA5, 0x5A, 0xFF];
        write_bits(&mut buf, 5, 7, 0);
        assert_eq!(buf[0] & 0b0001_1111, 0xA5 & 0b0001_1111);
        assert_eq!(buf[0] & 0b1110_0000, 0);
        assert_eq!(buf[1] & 0b1111_0000, 0x5A & 0b1111_0000);
        assert_eq!(buf[2], 0xFF);
        write_bits(&mut buf, 5, 7, 0x7F);
        assert_eq!(buf, [0xE5, 0x5F, 0xFF]);
    }

    #[test]
    fn reads_and_writes_at_the_records_edges() {
        let mut buf = [0u8; 5];
        write_bits(&mut buf, 0, 32, 0xDEAD_BEEF);
        assert_eq!(read_bits(&buf, 0, 32), 0xDEAD_BEEF);
        // Setting bit 8 (byte 1 bit 0) changes the word: 0xBE -> 0xBF.
        write_bits(&mut buf, 8, 1, 1);
        assert_eq!(read_bits(&buf, 0, 32), 0xDEAD_BFEF);
        write_bits(&mut buf, 39, 1, 1);
        assert_eq!(read_bits(&buf, 39, 1), 1);
        assert_eq!(buf[4], 0x80);
    }
}
