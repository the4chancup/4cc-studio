//! The `.model` container: a 24-byte header, the section table (a record
//! array of eleven `u32` section offsets at file offset 24, relative to
//! 24, indexed by [`SectionKind`]), then the eleven sections back to back
//! from offset 80 in whatever order the writer chose. Konami writes them
//! 0 1 2 3 4 5 6 8 9 10 7, the add-on 7 0 5 6 3 8 9 10 1 2 4; the table
//! is indexed by kind so file order is the only thing `sections` records.

use std::io::Cursor;

use binrw::BinRead;

use crate::format::ModelError;
use crate::format::records::Toc;

/// Where the section table's record array starts.
const TABLE_OFFSET: usize = 24;
/// Where the first section starts: 24 (table) + 12 (its TOC) + 44
/// (eleven u32 entries).
const FIRST_SECTION_OFFSET: usize = 80;
/// A section offset the table names, relative to `TABLE_OFFSET`, for the
/// first section.
const FIRST_SECTION_RELATIVE: usize = FIRST_SECTION_OFFSET - TABLE_OFFSET;

#[derive(BinRead)]
#[brw(little)]
struct Header {
    magic: [u8; 8],
    table_offset: u32,
    reserved: u16,
    version: u16,
    nine: u32,
    flags: u32,
}

/// A `.model` at the container level: the header words and the eleven sections as opaque byte
/// runs, in the order they sit in the file. `write(read(x)) == x` for every file PES ships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelContainer {
    /// Header version word (19 for every PES 2017 part but the shadow model, which is 17).
    pub version: u16,
    /// Header flags word (0; 4 in two face-montage models, meaning unknown).
    pub flags: u32,
    /// The sections in file order; every `SectionKind` appears exactly once.
    pub sections: Vec<Section>,
}

/// One section: which of the eleven it is, and its bytes up to the start of the next section
/// in the file (so any zero padding a writer left belongs to the section before it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// Which of the eleven sections this is.
    pub kind: SectionKind,
    /// The section's bytes.
    pub bytes: Vec<u8>,
}

/// The eleven sections of a `.model`, numbered as the section table indexes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionKind {
    /// Bone data: entry 0 is the inverse bind matrices, the rest bone groups.
    BoneData = 0,
    /// Geometry: per mesh the vertex set, face descriptor and extras.
    Geometry = 1,
    /// Annotation strings.
    AnnotationStrings = 2,
    /// Annotation records.
    AnnotationRecords = 3,
    /// Meshes: the records carrying the cross-section signed offsets.
    Meshes = 4,
    /// Bone names.
    BoneNames = 5,
    /// Material names.
    MaterialNames = 6,
    /// Model bounds and the LOD record.
    ModelBounds = 7,
    /// Cloth.
    Cloth = 8,
    /// Material combinations.
    MaterialCombinations = 9,
    /// Locator geometry.
    Locators = 10,
}

impl SectionKind {
    /// Every section kind, in table-index order.
    pub const ALL: [SectionKind; 11] = [
        SectionKind::BoneData,
        SectionKind::Geometry,
        SectionKind::AnnotationStrings,
        SectionKind::AnnotationRecords,
        SectionKind::Meshes,
        SectionKind::BoneNames,
        SectionKind::MaterialNames,
        SectionKind::ModelBounds,
        SectionKind::Cloth,
        SectionKind::MaterialCombinations,
        SectionKind::Locators,
    ];

    /// The section-table index this kind sits at.
    pub fn index(self) -> usize {
        self as usize
    }
}

impl ModelContainer {
    /// Parses a `.model`, unwrapped or WESYS-wrapped.
    ///
    /// The sections are cut by the table's offsets in increasing order: a
    /// section's bytes run to the next section's start, so zero padding a
    /// writer left between them belongs to the section before it.
    pub fn read(bytes: &[u8]) -> Result<Self, ModelError> {
        let unwrapped = wezlib::decompress_if_wrapped(bytes)
            .map_err(|error| ModelError::Wesys(error.to_string()))?;
        let bytes: &[u8] = &unwrapped;
        let header = bytes
            .get(..24)
            .ok_or(ModelError::Truncated)
            .and_then(|span| {
                Header::read(&mut Cursor::new(span)).map_err(|_| ModelError::Truncated)
            })?;
        if header.magic != *b"MODEL\0\0\0" {
            return Err(ModelError::BadMagic);
        }
        if header.reserved != 0 {
            return Err(ModelError::UnexpectedConstant {
                what: "header word at 12",
                value: u32::from(header.reserved),
            });
        }
        if header.nine != 9 {
            return Err(ModelError::UnexpectedConstant {
                what: "header word at 16",
                value: header.nine,
            });
        }
        if header.table_offset != 16 {
            return Err(ModelError::BadSectionTable(header.table_offset));
        }
        let toc = bytes
            .get(TABLE_OFFSET..TABLE_OFFSET + 12)
            .ok_or(ModelError::Truncated)
            .and_then(|span| {
                Toc::read(&mut Cursor::new(span)).map_err(|_| ModelError::Truncated)
            })?;
        if toc.record_count != 11 {
            return Err(ModelError::BadSectionTable(toc.record_count));
        }
        if toc.record_size != 4 {
            return Err(ModelError::BadSectionTable(toc.record_size));
        }
        let entries_start = TABLE_OFFSET
            .checked_add(toc.first_record_offset as usize)
            .ok_or(ModelError::Truncated)?;
        let mut offsets = [0u32; 11];
        for (index, offset) in offsets.iter_mut().enumerate() {
            let at = entries_start
                .checked_add(4 * index)
                .ok_or(ModelError::Truncated)?;
            *offset = u32::from_le_bytes(
                bytes
                    .get(at..at + 4)
                    .ok_or(ModelError::Truncated)?
                    .try_into()
                    .map_err(|_| ModelError::Truncated)?,
            );
        }
        let mut in_file_order: Vec<(u32, SectionKind)> = SectionKind::ALL
            .iter()
            .copied()
            .zip(offsets)
            .map(|(kind, offset)| (offset, kind))
            .collect();
        in_file_order.sort_by_key(|(offset, _)| *offset);
        let mut sections = Vec::with_capacity(11);
        for (position, (offset, kind)) in in_file_order.iter().enumerate() {
            let start = TABLE_OFFSET
                .checked_add(*offset as usize)
                .ok_or(ModelError::Truncated)?;
            let end = match in_file_order.get(position + 1) {
                Some((next, _)) => TABLE_OFFSET
                    .checked_add(*next as usize)
                    .ok_or(ModelError::Truncated)?,
                None => bytes.len(),
            };
            let span = bytes.get(start..end).ok_or(ModelError::Truncated)?;
            sections.push(Section {
                kind: *kind,
                bytes: span.to_vec(),
            });
        }
        Ok(ModelContainer {
            version: header.version,
            flags: header.flags,
            sections,
        })
    }

    /// Serializes the container, unwrapped. Sections are written in their
    /// stored file order; the table keeps its kind indexing. Wrapping in
    /// WESYS, when wanted, is the caller's job.
    pub fn write(&self) -> Vec<u8> {
        let mut offset_of = [0u32; 11];
        let mut cursor = FIRST_SECTION_RELATIVE;
        for section in &self.sections {
            offset_of[section.kind.index()] = cursor as u32;
            cursor += section.bytes.len();
        }
        let mut output = Vec::with_capacity(cursor + TABLE_OFFSET);
        output.extend_from_slice(b"MODEL\0\0\0");
        output.extend_from_slice(&16u32.to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&self.version.to_le_bytes());
        output.extend_from_slice(&9u32.to_le_bytes());
        output.extend_from_slice(&self.flags.to_le_bytes());
        output.extend_from_slice(&12u32.to_le_bytes());
        output.extend_from_slice(&11u32.to_le_bytes());
        output.extend_from_slice(&4u32.to_le_bytes());
        for offset in offset_of {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        for section in &self.sections {
            output.extend_from_slice(&section.bytes);
        }
        output
    }

    /// The bytes of the section `kind`, however it is ordered in the file.
    pub fn section(&self, kind: SectionKind) -> &[u8] {
        &self
            .sections
            .iter()
            .find(|section| section.kind == kind)
            .expect("every SectionKind is present in a ModelContainer")
            .bytes
    }

    /// The byte offset `kind`'s section starts at in the serialized file
    /// `write` produces.
    pub fn section_offset(&self, kind: SectionKind) -> usize {
        let before: usize = self
            .sections
            .iter()
            .take_while(|section| section.kind != kind)
            .map(|section| section.bytes.len())
            .sum();
        FIRST_SECTION_OFFSET + before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::fixtures::*;

    use SectionKind as K;

    #[test]
    fn fixtures_rewrite_byte_identically() {
        for bytes in ALL {
            let unwrapped = wezlib::decompress_if_wrapped(bytes).unwrap();
            let container = ModelContainer::read(bytes).unwrap();
            assert_eq!(container.write(), unwrapped.as_ref());
        }
    }

    #[test]
    fn header_words_and_section_order() {
        let card = ModelContainer::read(CARD).unwrap();
        assert_eq!(card.version, 19);
        assert_eq!(card.flags, 0);
        assert_eq!(
            card.sections.iter().map(|s| s.kind).collect::<Vec<_>>(),
            [
                K::BoneData,
                K::Geometry,
                K::AnnotationStrings,
                K::AnnotationRecords,
                K::Meshes,
                K::BoneNames,
                K::MaterialNames,
                K::Cloth,
                K::MaterialCombinations,
                K::Locators,
                K::ModelBounds,
            ]
        );
        let cardhead = ModelContainer::read(CARDHEAD).unwrap();
        assert_eq!(
            cardhead.sections.iter().map(|s| s.kind).collect::<Vec<_>>(),
            [
                K::ModelBounds,
                K::BoneData,
                K::BoneNames,
                K::MaterialNames,
                K::AnnotationRecords,
                K::Cloth,
                K::MaterialCombinations,
                K::Locators,
                K::Geometry,
                K::AnnotationStrings,
                K::Meshes,
            ]
        );
        assert_eq!(ModelContainer::read(SHADOW).unwrap().version, 17);
    }

    #[test]
    fn card_section_lengths_and_offsets() {
        let card = ModelContainer::read(CARD).unwrap();
        let lengths: Vec<usize> = K::ALL
            .iter()
            .map(|kind| card.section(*kind).len())
            .collect();
        assert_eq!(lengths, [132, 9656, 12, 12, 76, 28, 32, 76, 12, 12, 16]);
        let offsets: Vec<usize> = K::ALL
            .iter()
            .map(|kind| card.section_offset(*kind))
            .collect();
        assert_eq!(
            offsets,
            [
                80, 212, 9868, 9880, 9892, 9968, 9996, 10068, 10028, 10040, 10052
            ]
        );
        let cardhead = ModelContainer::read(CARDHEAD).unwrap();
        assert_eq!(cardhead.section_offset(K::ModelBounds), 80);
        assert_eq!(cardhead.section_offset(K::Meshes), 1840);
        assert_eq!(cardhead.section(K::Geometry).len(), 1372);
    }

    #[test]
    fn wesys_files_unwrap_and_write_unwrapped() {
        assert_eq!(ModelContainer::read(COLLAR).unwrap().write().len(), 280848);
        assert_eq!(ModelContainer::read(SHADOW).unwrap().write().len(), 16928);
    }

    #[test]
    fn malformed_buffers_error() {
        let mut fmdl = vec![0u8; 32];
        fmdl[..4].copy_from_slice(b"FMDL");
        assert_eq!(ModelContainer::read(&fmdl), Err(ModelError::BadMagic));
        assert_eq!(
            ModelContainer::read(&CARD[..100]),
            Err(ModelError::Truncated)
        );
        let mut patched = CARD.to_vec();
        // The section table's record count sits at 24 + 4.
        patched[28] = 10;
        assert_eq!(
            ModelContainer::read(&patched),
            Err(ModelError::BadSectionTable(10))
        );
        // The constant header words at offsets 12 and 16.
        let mut patched = CARD.to_vec();
        patched[12] = 1;
        assert_eq!(
            ModelContainer::read(&patched),
            Err(ModelError::UnexpectedConstant {
                what: "header word at 12",
                value: 1
            })
        );
        let mut patched = CARD.to_vec();
        patched[16] = 8;
        assert_eq!(
            ModelContainer::read(&patched),
            Err(ModelError::UnexpectedConstant {
                what: "header word at 16",
                value: 8
            })
        );
    }
}
