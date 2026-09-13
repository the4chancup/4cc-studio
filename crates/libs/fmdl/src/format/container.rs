//! The two-section FMDL container: section 0 holds fixed-size records keyed
//! by block id, section 1 holds raw byte blocks. Parsing and writing a
//! Konami file reproduces it byte for byte.
//!
//! Layout (little-endian): a 64-byte header, the descriptor tables (u16 id /
//! u16 entry count / u32 section-relative offset for section 0; u32 id /
//! u32 offset / u32 length for section 1) padded to 16, section 0 with its
//! blocks in ascending id order (block 13 starts 16-byte aligned, the
//! section padded to 16 at the end), then section 1 with its blocks back to
//! back.

use std::io::Cursor;

use binrw::{BinRead, BinWrite};

use crate::format::FmdlError;

/// An FMDL at the container level: every section-0 block as its raw
/// fixed-size records, every section-1 block as raw bytes. Blocks are kept
/// in the order they were read; `write` sorts them by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FmdlContainer {
    /// The header version field (`0x4001eb85` for the 2.03 files PES ships).
    pub version: u32,
    /// The record blocks of section 0.
    pub section0: Vec<RecordBlock>,
    /// The raw byte blocks of section 1.
    pub section1: Vec<ByteBlock>,
}

/// A section-0 block: `records` are all `record_size(id)` bytes long, or a
/// single raw span when the id is not in the size table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordBlock {
    /// The block id the descriptor table names.
    pub id: u16,
    /// The block's records, or one span holding the whole block for an
    /// unknown id.
    pub records: Vec<Vec<u8>>,
}

/// A section-1 block: raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteBlock {
    /// The block id the descriptor table names.
    pub id: u32,
    /// The block's content.
    pub bytes: Vec<u8>,
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
struct Header {
    magic: [u8; 4],
    version: u32,
    descriptors_offset: u64,
    section0_bitmap: u64,
    section1_bitmap: u64,
    section0_block_count: u32,
    section1_block_count: u32,
    section0_offset: u32,
    section0_length: u32,
    section1_offset: u32,
    section1_length: u32,
    zero: u64,
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
struct Section0Descriptor {
    id: u16,
    entry_count: u16,
    offset: u32,
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
struct Section1Descriptor {
    id: u32,
    offset: u32,
    length: u32,
}

/// The section-0 record size for each block id the format defines.
const RECORD_SIZES: &[(u16, usize)] = &[
    (0, 48),
    (1, 8),
    (2, 32),
    (3, 48),
    (4, 16),
    (5, 68),
    (6, 4),
    (7, 4),
    (8, 4),
    (9, 8),
    (10, 8),
    (11, 4),
    (12, 8),
    (13, 32),
    (14, 16),
    (16, 16),
    (17, 8),
    (18, 8),
    (20, 128),
];

/// The section-0 block that starts on a 16-byte boundary even when the
/// block before it does not end on one.
const ALIGNED_BLOCK: u16 = 13;

impl FmdlContainer {
    /// Parses an FMDL buffer. Blocks keep the order their descriptors give;
    /// a section-0 block whose id has no known record size is kept as one
    /// raw span running to the next block's offset (or the section end).
    pub fn read(bytes: &[u8]) -> Result<Self, FmdlError> {
        if bytes.len() < 64 {
            return Err(FmdlError::Truncated);
        }
        let mut cursor = Cursor::new(bytes);
        let header = Header::read(&mut cursor).map_err(|_| FmdlError::Truncated)?;
        if header.magic != *b"FMDL" {
            return Err(FmdlError::BadMagic);
        }

        cursor.set_position(header.descriptors_offset);
        let mut section0_descriptors = Vec::new();
        for _ in 0..header.section0_block_count {
            section0_descriptors
                .push(Section0Descriptor::read(&mut cursor).map_err(|_| FmdlError::Truncated)?);
        }
        let mut section1_descriptors = Vec::new();
        for _ in 0..header.section1_block_count {
            section1_descriptors
                .push(Section1Descriptor::read(&mut cursor).map_err(|_| FmdlError::Truncated)?);
        }
        if cursor.position() as usize > bytes.len() {
            return Err(FmdlError::Truncated);
        }

        let section0_offset =
            usize::try_from(header.section0_offset).map_err(|_| FmdlError::Truncated)?;
        let section1_offset =
            usize::try_from(header.section1_offset).map_err(|_| FmdlError::Truncated)?;

        let mut section0 = Vec::new();
        for (index, descriptor) in section0_descriptors.iter().enumerate() {
            if section0
                .iter()
                .any(|block: &RecordBlock| block.id == descriptor.id)
            {
                return Err(FmdlError::DuplicateBlock {
                    section: 0,
                    id: u32::from(descriptor.id),
                });
            }
            let start = section0_offset
                .checked_add(descriptor.offset as usize)
                .ok_or(FmdlError::Truncated)?;
            let records = match Self::record_size(descriptor.id) {
                Some(record_size) => {
                    let length = record_size * usize::from(descriptor.entry_count);
                    let span = bytes
                        .get(start..start + length)
                        .ok_or(FmdlError::Truncated)?;
                    span.chunks(record_size).map(<[u8]>::to_vec).collect()
                }
                None => {
                    // An unknown id has no record size: the block runs to the
                    // next block's offset or to the end of the section.
                    let end_offset = section0_descriptors[index + 1..]
                        .iter()
                        .map(|next| next.offset)
                        .min()
                        .unwrap_or(header.section0_length);
                    let length = end_offset
                        .checked_sub(descriptor.offset)
                        .ok_or(FmdlError::Truncated)?;
                    let span = bytes
                        .get(start..start + length as usize)
                        .ok_or(FmdlError::Truncated)?;
                    vec![span.to_vec()]
                }
            };
            section0.push(RecordBlock {
                id: descriptor.id,
                records,
            });
        }

        let mut section1 = Vec::new();
        for descriptor in &section1_descriptors {
            if section1
                .iter()
                .any(|block: &ByteBlock| block.id == descriptor.id)
            {
                return Err(FmdlError::DuplicateBlock {
                    section: 1,
                    id: descriptor.id,
                });
            }
            let start = section1_offset
                .checked_add(descriptor.offset as usize)
                .ok_or(FmdlError::Truncated)?;
            // Stored lengths occasionally run past the file end; the block
            // data is whatever is left. Block 3 is always read to the end.
            let remaining = bytes.len().saturating_sub(start);
            let length = if u64::from(descriptor.length) > remaining as u64 || descriptor.id == 3 {
                remaining
            } else {
                descriptor.length as usize
            };
            let span = bytes
                .get(start..start + length)
                .ok_or(FmdlError::Truncated)?;
            section1.push(ByteBlock {
                id: descriptor.id,
                bytes: span.to_vec(),
            });
        }

        Ok(FmdlContainer {
            version: header.version,
            section0,
            section1,
        })
    }

    /// Serializes the container: blocks sorted by id, descriptors padded to
    /// 16, section-0 block 13 16-byte aligned, section 0 padded to 16 at the
    /// end, section 1 written back to back.
    pub fn write(&self) -> Vec<u8> {
        let mut section0: Vec<&RecordBlock> = self.section0.iter().collect();
        section0.sort_by_key(|block| block.id);
        let mut section1: Vec<&ByteBlock> = self.section1.iter().collect();
        section1.sort_by_key(|block| block.id);

        let mut descriptor_writer = Cursor::new(Vec::new());
        let mut section0_data = Vec::new();
        let mut section1_data = Vec::new();
        let mut section0_bitmap = 0u64;
        let mut section1_bitmap = 0u64;

        for (index, block) in section0.iter().enumerate() {
            section0_bitmap |= 1 << block.id;
            let descriptor = Section0Descriptor {
                id: block.id,
                entry_count: block.records.len() as u16,
                offset: section0_data.len() as u32,
            };
            descriptor
                .write(&mut descriptor_writer)
                .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
            for record in &block.records {
                section0_data.extend_from_slice(record);
            }
            // Block 13 starts 16-byte aligned, so the padding lands at the
            // end of whatever block precedes it.
            if section0
                .get(index + 1)
                .is_some_and(|next| next.id == ALIGNED_BLOCK)
            {
                section0_data.resize(section0_data.len() + pad16(section0_data.len()), 0);
            }
        }
        let section0_length = section0_data.len() + pad16(section0_data.len());
        section0_data.resize(section0_length, 0);

        for block in &section1 {
            section1_bitmap |= 1 << block.id;
            let descriptor = Section1Descriptor {
                id: block.id,
                offset: section1_data.len() as u32,
                length: block.bytes.len() as u32,
            };
            descriptor
                .write(&mut descriptor_writer)
                .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
            section1_data.extend_from_slice(&block.bytes);
        }
        let mut descriptors = descriptor_writer.into_inner();
        descriptors.resize(descriptors.len() + pad16(descriptors.len()), 0);

        let section0_offset = 64 + descriptors.len();
        let section1_offset = section0_offset + section0_length;
        let header = Header {
            magic: *b"FMDL",
            version: self.version,
            descriptors_offset: 64,
            section0_bitmap,
            section1_bitmap,
            section0_block_count: section0.len() as u32,
            section1_block_count: section1.len() as u32,
            section0_offset: section0_offset as u32,
            section0_length: section0_length as u32,
            section1_offset: section1_offset as u32,
            section1_length: section1_data.len() as u32,
            zero: 0,
        };
        let mut output = Vec::new();
        header
            .write(&mut Cursor::new(&mut output))
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
        output.extend_from_slice(&descriptors);
        output.extend_from_slice(&section0_data);
        output.extend_from_slice(&section1_data);
        output
    }

    /// The record size for a section-0 block id, when the format defines
    /// one.
    pub fn record_size(id: u16) -> Option<usize> {
        RECORD_SIZES
            .iter()
            .find(|(block_id, _)| *block_id == id)
            .map(|(_, size)| *size)
    }
}

/// Bytes needed to bring `len` up to the next multiple of 16.
fn pad16(len: usize) -> usize {
    (16 - len % 16) % 16
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGHNECK: &[u8] = include_bytes!("../../tests/fixtures/konami_highneck.fmdl");
    const MOUTH: &[u8] = include_bytes!("../../tests/fixtures/konami_mouth.fmdl");
    const AU_LOW: &[u8] = include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl");
    const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
    const PLACEHOLDER: &[u8] = include_bytes!("../../tests/fixtures/addon_placeholder.fmdl");

    fn block0_count(container: &FmdlContainer) -> Option<usize> {
        container
            .section0
            .iter()
            .find(|block| block.id == 0)
            .map(|block| block.records.len())
    }

    fn block3_count(container: &FmdlContainer) -> Option<usize> {
        container
            .section0
            .iter()
            .find(|block| block.id == 3)
            .map(|block| block.records.len())
    }

    #[test]
    fn konami_files_rewrite_byte_identically() {
        for bytes in [HIGHNECK, MOUTH, AU_LOW] {
            let container = FmdlContainer::read(bytes).unwrap();
            assert_eq!(container.write(), bytes);
        }
    }

    #[test]
    fn addon_files_round_trip_semantically() {
        for bytes in [ORAL, PLACEHOLDER] {
            let container = FmdlContainer::read(bytes).unwrap();
            let written = container.write();
            assert_eq!(FmdlContainer::read(&written).unwrap(), container);
        }
        // The add-on pads every section-0 block to 16 in `oral`; ours does
        // not. `placeholder`'s blocks are all naturally 16-aligned, so its
        // rewrite comes out the same length.
        let oral = FmdlContainer::read(ORAL).unwrap();
        assert!(oral.write().len() < ORAL.len());
    }

    #[test]
    fn fixture_block_counts_match_the_readme() {
        let cases = [
            (HIGHNECK, Some(7usize), Some(1usize)),
            (MOUTH, Some(6), Some(1)),
            (AU_LOW, Some(16), Some(4)),
            (ORAL, Some(1), Some(1)),
            (PLACEHOLDER, None, Some(1)),
        ];
        for (bytes, block0, block3) in cases {
            let container = FmdlContainer::read(bytes).unwrap();
            assert_eq!(block0_count(&container), block0, "block 0");
            assert_eq!(block3_count(&container), block3, "block 3");
        }
        // Section-1 block 1 is the bone-name table, 64 bytes per bone.
        for (bytes, bones) in [(HIGHNECK, 7usize), (MOUTH, 6), (AU_LOW, 16)] {
            let container = FmdlContainer::read(bytes).unwrap();
            let names = container
                .section1
                .iter()
                .find(|block| block.id == 1)
                .unwrap();
            assert_eq!(names.bytes.len(), 64 * bones);
        }
    }

    #[test]
    fn unknown_section0_block_survives_a_round_trip() {
        let mut container = FmdlContainer::read(HIGHNECK).unwrap();
        // Unknown ids are stored as one raw span. Id 15 is not in the size
        // table and sorts mid-section, so the span's extent is exact; a
        // trailing unknown block would absorb the section-end padding on
        // reread.
        container.section0.push(RecordBlock {
            id: 15,
            records: vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]],
        });
        let reread = FmdlContainer::read(&container.write()).unwrap();
        let unknown = reread.section0.iter().find(|block| block.id == 15).unwrap();
        assert_eq!(
            unknown.records,
            [vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]]
        );
        assert_eq!(
            reread
                .section0
                .iter()
                .filter(|block| block.id != 15)
                .collect::<Vec<_>>(),
            container
                .section0
                .iter()
                .filter(|block| block.id != 15)
                .collect::<Vec<_>>()
        );
        assert_eq!(reread.section1, container.section1);
    }

    #[test]
    fn malformed_buffers_error() {
        assert!(matches!(
            FmdlContainer::read(&HIGHNECK[..40]),
            Err(FmdlError::Truncated)
        ));
        let mut bad_magic = HIGHNECK.to_vec();
        bad_magic[3] = b'X';
        assert!(matches!(
            FmdlContainer::read(&bad_magic),
            Err(FmdlError::BadMagic)
        ));
        // Section-0 descriptor table starts at 64; the second descriptor's
        // id sits at 64 + 8.
        let mut duplicate = HIGHNECK.to_vec();
        duplicate[72..74].copy_from_slice(&0u16.to_le_bytes());
        duplicate[76..80].copy_from_slice(&0u32.to_le_bytes());
        assert!(matches!(
            FmdlContainer::read(&duplicate),
            Err(FmdlError::DuplicateBlock { section: 0, id: 0 })
        ));
    }
}
