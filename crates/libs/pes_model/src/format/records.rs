//! The record array: the shape every `.model` section and every sub-table
//! inside a section stores its records in.

use std::io::Cursor;

use binrw::BinRead;

use crate::format::ModelError;

/// The 12-byte table of contents heading a record array.
#[derive(BinRead)]
#[brw(little)]
pub(crate) struct Toc {
    /// Offset of the first record relative to the array's start; the bytes
    /// between the table and it are the array's header.
    pub first_record_offset: u32,
    /// How many records follow.
    pub record_count: u32,
    /// Every record's size in bytes.
    pub record_size: u32,
}

/// A record array as every `.model` section and sub-table stores it: a
/// 12-byte table of contents (`u32 first_record_offset, u32 record_count,
/// u32 record_size`), an optional header of `first_record_offset - 12`
/// bytes, then `record_count` records of `record_size` bytes. Konami's
/// empty section 8 has `first_record_offset == 0`; an empty array with an
/// offset under 12 is read as having no header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordArray {
    /// The bytes between the table of contents and the first record.
    pub header: Vec<u8>,
    /// Every record's size in bytes.
    pub record_size: u32,
    /// The records, in file order.
    pub records: Vec<Record>,
}

/// One record and where it sits, as an offset from the start of the
/// buffer it was read from (the section), which is what the file's
/// cross-references point at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The record's offset in the buffer.
    pub offset: usize,
    /// The record's bytes, `record_size` long.
    pub bytes: Vec<u8>,
}

impl RecordArray {
    /// Reads the array whose table of contents starts at `at` in `buffer`.
    pub fn read(buffer: &[u8], at: usize) -> Result<Self, ModelError> {
        let toc_end = at.checked_add(12).ok_or(ModelError::Truncated)?;
        let toc = buffer
            .get(at..toc_end)
            .ok_or(ModelError::Truncated)
            .and_then(|span| {
                Toc::read(&mut Cursor::new(span)).map_err(|_| ModelError::Truncated)
            })?;
        if toc.record_count > 0 && toc.first_record_offset < 12 {
            return Err(ModelError::BadRecordArray { at });
        }
        if toc.record_size == 0 && toc.record_count > 0 {
            return Err(ModelError::BadRecordArray { at });
        }
        let header_end = at
            .checked_add(toc.first_record_offset as usize)
            .ok_or(ModelError::Truncated)?;
        let header = if toc.first_record_offset >= 12 {
            buffer
                .get(toc_end..header_end)
                .ok_or(ModelError::Truncated)?
                .to_vec()
        } else {
            Vec::new()
        };
        // The whole record run must fit before any of it is read; on a
        // 32-bit `usize` a hostile count * size wraps to a small value
        // instead of failing, hence the checked arithmetic.
        let record_count = toc.record_count as usize;
        let record_size = toc.record_size as usize;
        let records_end = record_count
            .checked_mul(record_size)
            .and_then(|len| header_end.checked_add(len))
            .ok_or(ModelError::Truncated)?;
        buffer
            .get(header_end..records_end)
            .ok_or(ModelError::Truncated)?;
        let mut records = Vec::with_capacity(record_count.min(buffer.len()));
        for index in 0..record_count {
            let offset = header_end + index * record_size;
            records.push(Record {
                offset,
                bytes: buffer[offset..offset + record_size].to_vec(),
            });
        }
        Ok(RecordArray {
            header,
            record_size: toc.record_size,
            records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &[u8] = include_bytes!("../../tests/fixtures/konami_card.model");

    fn section(kind: usize) -> Vec<u8> {
        use crate::format::{ModelContainer, SectionKind};
        let container = ModelContainer::read(CARD).unwrap();
        container.section(SectionKind::ALL[kind]).to_vec()
    }

    #[test]
    fn bone_data_array() {
        let array = RecordArray::read(&section(0), 0).unwrap();
        assert_eq!(array.header, Vec::<u8>::new());
        assert_eq!(array.record_size, 4);
        assert_eq!(array.records.len(), 2);
        assert_eq!(array.records[0].offset, 12);
        assert_eq!(array.records[1].offset, 16);
        // Record 0's u32 points at a second array at offset 20.
        let pointed = u32::from_le_bytes(array.records[0].bytes[..4].try_into().unwrap());
        assert_eq!(pointed, 20);
        let bone_data = section(0);
        let inner = RecordArray::read(&bone_data, pointed as usize).unwrap();
        assert_eq!(inner.header, vec![2, 0, 0, 0]);
        assert_eq!(inner.record_size, 12);
        assert_eq!(inner.records.len(), 1);
        assert_eq!(inner.records[0].offset, 36);
    }

    #[test]
    fn empty_arrays() {
        // Locators: a 4-byte zero header, record size 16, no records.
        let locators = RecordArray::read(&section(10), 0).unwrap();
        assert_eq!(locators.header, vec![0, 0, 0, 0]);
        assert_eq!(locators.record_size, 16);
        assert!(locators.records.is_empty());
        // Cloth: the offset-0 quirk, no header, no records.
        let cloth = RecordArray::read(&section(8), 0).unwrap();
        assert_eq!(cloth.header, Vec::<u8>::new());
        assert_eq!(cloth.record_size, 4);
        assert!(cloth.records.is_empty());
        // Material combinations: a normal empty array.
        let combinations = RecordArray::read(&section(9), 0).unwrap();
        assert_eq!(combinations.header, Vec::<u8>::new());
        assert_eq!(combinations.record_size, 4);
        assert!(combinations.records.is_empty());
    }

    #[test]
    fn malformed_arrays_error() {
        // One record at offset 4: before the table of contents itself ends.
        let bad = [4u8, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0];
        assert_eq!(
            RecordArray::read(&bad, 0),
            Err(ModelError::BadRecordArray { at: 0 })
        );
        // 1000 records of 4 bytes cannot fit in 12 bytes of buffer.
        let oversized = [12u8, 0, 0, 0, 232, 3, 0, 0, 4, 0, 0, 0];
        assert_eq!(RecordArray::read(&oversized, 0), Err(ModelError::Truncated));
        // u32::MAX records of u32::MAX bytes overflows `usize` on wasm32;
        // it must error, not wrap or panic.
        let hostile = [
            12u8, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        assert_eq!(RecordArray::read(&hostile, 0), Err(ModelError::Truncated));
    }
}
