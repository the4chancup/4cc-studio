//! The Fox skeleton (`.skl`) codec: a 12-byte header, one 56-byte record
//! per bone (name offset, parent index, 3x4 row-major transform), then the
//! NUL-terminated name table padded to 4.

use std::io::Cursor;

use binrw::{BinRead, BinWrite};

use crate::format::FmdlError;

const MAGIC: u32 = 12;
const RECORD_SIZE: u32 = 56;

/// A Fox skeleton: bone names, parent indices and 3x4 bind transforms.
#[derive(Debug, Clone, PartialEq)]
pub struct SklFile {
    /// The bones in file order. Parents need not precede children: the game's
    /// `body.skl` files put bones 1 and 2 under bone 18.
    pub bones: Vec<SklBone>,
}

/// One SKL bone record resolved to a name and a transform.
#[derive(Debug, Clone, PartialEq)]
pub struct SklBone {
    /// The bone's name from the trailing name table.
    pub name: String,
    /// The index of the parent bone, `None` for a root.
    pub parent: Option<usize>,
    /// The 3x3 bind-pose rotation, row-major.
    pub rotation: [[f32; 3]; 3],
    /// The bind-pose position.
    pub translation: [f32; 3],
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
struct Header {
    magic: u32,
    bone_count: u32,
    record_size: u32,
}

impl SklFile {
    /// Parses an SKL buffer: header, bone records and the name table each
    /// record's name offset points into.
    pub fn read(bytes: &[u8]) -> Result<Self, FmdlError> {
        if bytes.len() < 12 {
            return Err(FmdlError::Truncated);
        }
        let mut cursor = Cursor::new(bytes);
        let header = Header::read(&mut cursor).map_err(|_| FmdlError::Truncated)?;
        if header.magic != MAGIC {
            return Err(FmdlError::BadSklMagic(header.magic));
        }
        if header.record_size != RECORD_SIZE {
            return Err(FmdlError::BadSklRecordSize(header.record_size));
        }
        let records_end = (header.bone_count as usize)
            .checked_mul(RECORD_SIZE as usize)
            .and_then(|bytes| 12usize.checked_add(bytes))
            .ok_or(FmdlError::Truncated)?;
        let records = bytes.get(12..records_end).ok_or(FmdlError::Truncated)?;

        let mut bones = Vec::with_capacity(header.bone_count as usize);
        for record in records.as_chunks::<{ RECORD_SIZE as usize }>().0 {
            // The record is exactly RECORD_SIZE bytes, so the fixed-width
            // field slices always convert.
            let name_offset = u32::from_le_bytes(
                record[0..4]
                    .try_into()
                    .unwrap_or_else(|_| unreachable!("record slice is {RECORD_SIZE} bytes")),
            ) as usize;
            let parent_index = i32::from_le_bytes(
                record[4..8]
                    .try_into()
                    .unwrap_or_else(|_| unreachable!("record slice is {RECORD_SIZE} bytes")),
            );
            let mut floats = [0f32; 12];
            for (index, slot) in floats.iter_mut().enumerate() {
                *slot = f32::from_le_bytes(
                    record[8 + index * 4..12 + index * 4]
                        .try_into()
                        .unwrap_or_else(|_| unreachable!("record slice is {RECORD_SIZE} bytes")),
                );
            }
            let name_region = bytes.get(name_offset..).ok_or(FmdlError::Truncated)?;
            let name_end = name_region
                .iter()
                .position(|byte| *byte == 0)
                .map(|relative| name_offset + relative)
                .ok_or(FmdlError::InvalidName)?;
            let name_bytes = bytes
                .get(name_offset..name_end)
                .ok_or(FmdlError::Truncated)?;
            let name =
                String::from_utf8(name_bytes.to_vec()).map_err(|_| FmdlError::InvalidName)?;
            // A parent may come later in the file, but it must exist.
            let parent = if parent_index >= 0 {
                let parent = parent_index as usize;
                if parent >= header.bone_count as usize {
                    return Err(FmdlError::BadReference {
                        what: "skl parent",
                        index: parent,
                    });
                }
                Some(parent)
            } else {
                None
            };
            bones.push(SklBone {
                name,
                parent,
                rotation: [
                    [floats[0], floats[1], floats[2]],
                    [floats[4], floats[5], floats[6]],
                    [floats[8], floats[9], floats[10]],
                ],
                translation: [floats[3], floats[7], floats[11]],
            });
        }
        Ok(SklFile { bones })
    }

    /// Serializes the skeleton: records in bone order, the name table
    /// concatenated behind them, the file padded to 4.
    pub fn write(&self) -> Vec<u8> {
        let name_table_offset = 12 + self.bones.len() * RECORD_SIZE as usize;
        let mut name_table = Vec::new();
        let mut name_offsets = Vec::with_capacity(self.bones.len());
        for bone in &self.bones {
            name_offsets.push(name_table_offset + name_table.len());
            name_table.extend_from_slice(bone.name.as_bytes());
            name_table.push(0);
        }

        let mut output = Vec::new();
        Header {
            magic: MAGIC,
            bone_count: self.bones.len() as u32,
            record_size: RECORD_SIZE,
        }
        .write(&mut Cursor::new(&mut output))
        .unwrap_or_else(|_| unreachable!("Vec write is infallible"));

        for (index, bone) in self.bones.iter().enumerate() {
            output.extend_from_slice(&(name_offsets[index] as u32).to_le_bytes());
            output.extend_from_slice(&bone.parent.map_or(-1, |parent| parent as i32).to_le_bytes());
            for row in 0..3 {
                output.extend_from_slice(&bone.rotation[row][0].to_le_bytes());
                output.extend_from_slice(&bone.rotation[row][1].to_le_bytes());
                output.extend_from_slice(&bone.rotation[row][2].to_le_bytes());
                output.extend_from_slice(&bone.translation[row].to_le_bytes());
            }
        }
        output.extend_from_slice(&name_table);
        while output.len() % 4 != 0 {
            output.push(0);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AU00: &[u8] = include_bytes!("../../tests/fixtures/konami_au00.skl");
    const BOOTS: &[u8] = include_bytes!("../../tests/fixtures/konami_boots.skl");
    const REFEREE: &[u8] = include_bytes!("../../tests/fixtures/konami_referee_f_close.skl");

    #[test]
    fn konami_files_rewrite_byte_identically() {
        for bytes in [AU00, BOOTS, REFEREE] {
            assert_eq!(SklFile::read(bytes).unwrap().write(), bytes);
        }
    }

    #[test]
    fn fixture_bone_counts_and_parents() {
        for (bytes, count) in [(AU00, 16usize), (BOOTS, 4), (REFEREE, 2)] {
            let skl = SklFile::read(bytes).unwrap();
            assert_eq!(skl.bones.len(), count);
            let mut roots = 0;
            for (index, bone) in skl.bones.iter().enumerate() {
                match bone.parent {
                    None => roots += 1,
                    Some(parent) => assert!(parent < index, "bone {index}"),
                }
            }
            assert!(roots >= 1);
        }
    }

    #[test]
    fn malformed_buffers_error() {
        let mut bad_magic = BOOTS.to_vec();
        bad_magic[0..4].copy_from_slice(&13u32.to_le_bytes());
        assert!(matches!(
            SklFile::read(&bad_magic),
            Err(FmdlError::BadSklMagic(13))
        ));
        let mut bad_size = BOOTS.to_vec();
        bad_size[8..12].copy_from_slice(&40u32.to_le_bytes());
        assert!(matches!(
            SklFile::read(&bad_size),
            Err(FmdlError::BadSklRecordSize(40))
        ));
        // Point the first bone's name past the end of the file: no NUL
        // terminator exists there.
        let mut bad_name = BOOTS.to_vec();
        bad_name[12..16].copy_from_slice(&(BOOTS.len() as u32).to_le_bytes());
        assert!(matches!(
            SklFile::read(&bad_name),
            Err(FmdlError::InvalidName)
        ));
    }

    #[test]
    fn out_of_range_parent_errors() {
        // Record N starts at 12 + 56 * N; its parent index is at offset 4.
        let mut forward = BOOTS.to_vec();
        forward[12 + 4..12 + 8].copy_from_slice(&4i32.to_le_bytes()); // == bone_count
        assert!(matches!(
            SklFile::read(&forward),
            Err(FmdlError::BadReference {
                what: "skl parent",
                index: 4
            })
        ));
        // A parent later in the file is valid (the game's body.skl does it).
        let mut forward_ok = BOOTS.to_vec();
        forward_ok[12 + 4..12 + 8].copy_from_slice(&3i32.to_le_bytes());
        assert_eq!(SklFile::read(&forward_ok).unwrap().bones[0].parent, Some(3));
    }
}
