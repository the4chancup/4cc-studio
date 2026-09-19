//! `FmdlFile`: the typed layer over [`FmdlContainer`]. Reading parses each
//! known section-0 block into its record type; writing serializes the typed
//! vectors back into blocks and hands the container to
//! `FmdlContainer::write`.

use std::io::Cursor;

use binrw::{BinRead, BinWrite};

use crate::format::FmdlError;
use crate::format::container::{ByteBlock, FmdlContainer, RecordBlock};
use crate::format::records::*;

/// A parsed FMDL: every section-0 record typed, section-1 buffers raw.
#[derive(Debug, Clone, PartialEq)]
pub struct FmdlFile {
    /// The header version field (`0x4001eb85` for the 2.03 files PES ships).
    pub version: u32,
    /// The skeleton's bones (block 0): names, parents, bounding boxes, positions.
    pub bones: Vec<BoneRecord>,
    /// Named groups of meshes (block 1), the hierarchy Blender shows as objects.
    pub mesh_groups: Vec<MeshGroupRecord>,
    /// Which meshes each group owns (block 2).
    pub mesh_group_assignments: Vec<MeshGroupAssignmentRecord>,
    /// The meshes (block 3): vertex and face counts, material, bone group, format.
    pub meshes: Vec<MeshRecord>,
    /// Per-mesh material instances (block 4): material, textures, parameters.
    pub material_instances: Vec<MaterialInstanceRecord>,
    /// The bone subsets a mesh's vertices may weight to (block 5), up to 32 each.
    pub bone_groups: Vec<BoneGroupRecord>,
    /// Texture references (block 6): a name string and a directory string.
    pub textures: Vec<TextureRecord>,
    /// Texture-sampler and shader-parameter assignments (block 7).
    pub parameter_assignments: Vec<ParameterAssignmentRecord>,
    /// Shader and technique pairs (block 8).
    pub materials: Vec<MaterialRecord>,
    /// Which mesh formats and vertex formats a mesh uses (block 9).
    pub mesh_format_assignments: Vec<MeshFormatAssignmentRecord>,
    /// Vertex stream descriptions (block 10): buffer, stride, offset.
    pub mesh_formats: Vec<MeshFormatRecord>,
    /// Per-attribute layout inside a stream (block 11): type, format, offset.
    pub vertex_formats: Vec<VertexFormatRecord>,
    /// The string table entries (block 12): block, offset and length of each string.
    pub strings: Vec<StringRecord>,
    /// Axis-aligned bounds (block 13), max then min as the file stores them.
    pub bounding_boxes: Vec<BoundingBoxRecord>,
    /// Where each vertex/face buffer lives in section-1 block 2 (block 14).
    pub buffer_offsets: Vec<BufferOffsetRecord>,
    /// The level-of-detail record (block 16), exactly one per file.
    pub levels_of_detail: Vec<LevelOfDetailRecord>,
    /// Face index ranges per mesh (block 17).
    pub face_indices: Vec<FaceIndexRecord>,
    /// Block 18; purpose unknown.
    pub block_18: Vec<Block18Record>,
    /// Block 20; purpose unknown.
    pub block_20: Vec<Block20Record>,
    /// Section-0 blocks whose id has no known record type, kept raw.
    pub unknown_blocks: Vec<RecordBlock>,
    /// Section-1 block 0.
    pub material_parameters: Option<Vec<u8>>,
    /// Section-1 block 1.
    pub bone_matrices: Option<Vec<u8>>,
    /// Section-1 block 2.
    pub buffer: Option<Vec<u8>>,
    /// Section-1 block 3.
    pub string_table: Option<Vec<u8>>,
    /// Section-1 blocks whose id is not one of the named four, kept raw.
    pub unknown_buffers: Vec<ByteBlock>,
}

/// Parses every record of one section-0 block into `records`.
fn typed<T: for<'a> BinRead<Args<'a> = ()>>(
    container: &FmdlContainer,
    id: u16,
    records: &mut Vec<T>,
) -> Result<(), FmdlError> {
    let Some(block) = container.section0.iter().find(|block| block.id == id) else {
        return Ok(());
    };
    for record in &block.records {
        records.push(
            T::read_options(
                &mut Cursor::new(record.as_slice()),
                binrw::Endian::Little,
                (),
            )
            .map_err(|_| FmdlError::Truncated)?,
        );
    }
    Ok(())
}

/// Serializes a typed vector back into a section-0 block; `None` when
/// empty, so an absent block stays absent.
fn block<T: for<'a> BinWrite<Args<'a> = ()>>(id: u16, records: &[T]) -> Option<RecordBlock> {
    if records.is_empty() {
        return None;
    }
    let mut raw = Vec::with_capacity(records.len());
    for record in records {
        let mut cursor = Cursor::new(Vec::new());
        record
            .write_options(&mut cursor, binrw::Endian::Little, ())
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
        raw.push(cursor.into_inner());
    }
    Some(RecordBlock { id, records: raw })
}

/// The named section-1 block with this id, when present.
fn section1(file: &FmdlFile, id: u16) -> Option<&[u8]> {
    match id {
        0 => file.material_parameters.as_deref(),
        1 => file.bone_matrices.as_deref(),
        2 => file.buffer.as_deref(),
        3 => file.string_table.as_deref(),
        _ => file
            .unknown_buffers
            .iter()
            .find(|block| block.id == u32::from(id))
            .map(|block| block.bytes.as_slice()),
    }
}

/// A `Some` block is emitted even when empty (the add-on fixtures carry an
/// empty bone-name block 1); `None` stays absent.
fn byte_block(id: u32, bytes: &Option<Vec<u8>>) -> Option<ByteBlock> {
    bytes.as_ref().map(|bytes| ByteBlock {
        id,
        bytes: bytes.clone(),
    })
}

impl FmdlFile {
    /// Parses an FMDL buffer: the container first, then each known block's
    /// records.
    pub fn read(bytes: &[u8]) -> Result<Self, FmdlError> {
        let container = FmdlContainer::read(bytes)?;
        let mut file = FmdlFile {
            version: container.version,
            bones: Vec::new(),
            mesh_groups: Vec::new(),
            mesh_group_assignments: Vec::new(),
            meshes: Vec::new(),
            material_instances: Vec::new(),
            bone_groups: Vec::new(),
            textures: Vec::new(),
            parameter_assignments: Vec::new(),
            materials: Vec::new(),
            mesh_format_assignments: Vec::new(),
            mesh_formats: Vec::new(),
            vertex_formats: Vec::new(),
            strings: Vec::new(),
            bounding_boxes: Vec::new(),
            buffer_offsets: Vec::new(),
            levels_of_detail: Vec::new(),
            face_indices: Vec::new(),
            block_18: Vec::new(),
            block_20: Vec::new(),
            unknown_blocks: Vec::new(),
            material_parameters: None,
            bone_matrices: None,
            buffer: None,
            string_table: None,
            unknown_buffers: Vec::new(),
        };

        typed(&container, 0, &mut file.bones)?;
        typed(&container, 1, &mut file.mesh_groups)?;
        typed(&container, 2, &mut file.mesh_group_assignments)?;
        typed(&container, 3, &mut file.meshes)?;
        typed(&container, 4, &mut file.material_instances)?;
        typed(&container, 5, &mut file.bone_groups)?;
        typed(&container, 6, &mut file.textures)?;
        typed(&container, 7, &mut file.parameter_assignments)?;
        typed(&container, 8, &mut file.materials)?;
        typed(&container, 9, &mut file.mesh_format_assignments)?;
        typed(&container, 10, &mut file.mesh_formats)?;
        typed(&container, 11, &mut file.vertex_formats)?;
        typed(&container, 12, &mut file.strings)?;
        typed(&container, 13, &mut file.bounding_boxes)?;
        typed(&container, 14, &mut file.buffer_offsets)?;
        typed(&container, 16, &mut file.levels_of_detail)?;
        typed(&container, 17, &mut file.face_indices)?;
        typed(&container, 18, &mut file.block_18)?;
        typed(&container, 20, &mut file.block_20)?;
        file.unknown_blocks = container
            .section0
            .iter()
            .filter(|block| FmdlContainer::record_size(block.id).is_none())
            .cloned()
            .collect();

        for block1 in container.section1 {
            match block1.id {
                0 => file.material_parameters = Some(block1.bytes),
                1 => file.bone_matrices = Some(block1.bytes),
                2 => file.buffer = Some(block1.bytes),
                3 => file.string_table = Some(block1.bytes),
                _ => file.unknown_buffers.push(block1),
            }
        }
        Ok(file)
    }

    /// Serializes the typed records into section-0 blocks and the named
    /// buffers into section 1, then writes the container.
    pub fn write(&self) -> Vec<u8> {
        let mut section0 = Vec::new();
        for candidate in [
            block(0, &self.bones),
            block(1, &self.mesh_groups),
            block(2, &self.mesh_group_assignments),
            block(3, &self.meshes),
            block(4, &self.material_instances),
            block(5, &self.bone_groups),
            block(6, &self.textures),
            block(7, &self.parameter_assignments),
            block(8, &self.materials),
            block(9, &self.mesh_format_assignments),
            block(10, &self.mesh_formats),
            block(11, &self.vertex_formats),
            block(12, &self.strings),
            block(13, &self.bounding_boxes),
            block(14, &self.buffer_offsets),
            block(16, &self.levels_of_detail),
            block(17, &self.face_indices),
            block(18, &self.block_18),
            block(20, &self.block_20),
        ]
        .into_iter()
        .flatten()
        {
            section0.push(candidate);
        }
        section0.extend(self.unknown_blocks.iter().cloned());

        let mut section1: Vec<ByteBlock> = [
            byte_block(0, &self.material_parameters),
            byte_block(1, &self.bone_matrices),
            byte_block(2, &self.buffer),
            byte_block(3, &self.string_table),
        ]
        .into_iter()
        .flatten()
        .collect();
        section1.extend(self.unknown_buffers.iter().cloned());

        FmdlContainer {
            version: self.version,
            section0,
            section1,
        }
        .write()
    }

    /// The string `strings[index]` names: a span inside a section-1 block.
    pub fn string(&self, index: usize) -> Result<&str, FmdlError> {
        let record = self
            .strings
            .get(index)
            .ok_or(FmdlError::BadStringReference { index })?;
        let block = section1(self, record.string_block_id)
            .ok_or(FmdlError::BadStringReference { index })?;
        let end = usize::from(record.length)
            .checked_add(record.offset as usize)
            .ok_or(FmdlError::BadStringReference { index })?;
        let span = block
            .get(record.offset as usize..end)
            .ok_or(FmdlError::BadStringReference { index })?;
        std::str::from_utf8(span).map_err(|_| FmdlError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGHNECK: &[u8] = include_bytes!("../../tests/fixtures/konami_highneck.fmdl");
    const MOUTH: &[u8] = include_bytes!("../../tests/fixtures/konami_mouth.fmdl");
    const AU_LOW: &[u8] = include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl");
    const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
    const PLACEHOLDER: &[u8] = include_bytes!("../../tests/fixtures/addon_placeholder.fmdl");

    #[test]
    fn konami_files_rewrite_byte_identically_through_the_typed_layer() {
        for bytes in [HIGHNECK, MOUTH, AU_LOW] {
            let file = FmdlFile::read(bytes).unwrap();
            assert_eq!(file.write(), bytes);
        }
    }

    #[test]
    fn addon_files_round_trip_through_the_typed_layer() {
        for bytes in [ORAL, PLACEHOLDER] {
            let file = FmdlFile::read(bytes).unwrap();
            assert_eq!(FmdlFile::read(&file.write()).unwrap(), file);
        }
        // `oral`'s empty bone-name block stays emitted.
        assert_eq!(FmdlFile::read(ORAL).unwrap().bone_matrices, Some(vec![]));
    }

    #[test]
    fn typed_counts_match_the_readme() {
        let cases = [
            (HIGHNECK, 7usize, 1usize),
            (MOUTH, 6, 1),
            (AU_LOW, 16, 4),
            (ORAL, 1, 1),
            (PLACEHOLDER, 0, 1),
        ];
        for (bytes, bones, meshes) in cases {
            let file = FmdlFile::read(bytes).unwrap();
            assert_eq!(file.bones.len(), bones, "bones");
            assert_eq!(file.meshes.len(), meshes, "meshes");
            assert_eq!(file.material_instances.len(), 1, "material instances");
        }
    }

    #[test]
    fn bone_names_resolve_through_the_string_table() {
        let cases = [
            (
                HIGHNECK,
                [
                    "sk_chest",
                    "sk_neck",
                    "sk_head",
                    "dsk_scm",
                    "dsk_neckback",
                    "dsk_clavicle_r",
                    "dsk_clavicle_l",
                ]
                .as_slice(),
            ),
            (
                MOUTH,
                [
                    "sk_head",
                    "skf_jaw",
                    "skf_cheek_s_l",
                    "skf_cheek_s_r",
                    "skf_lip_s_l",
                    "skf_lip_s_r",
                ]
                .as_slice(),
            ),
            (
                AU_LOW,
                [
                    "sk_belly",
                    "sk_chest",
                    "sk_neck",
                    "sk_shoulder_l",
                    "sk_upperarm_l",
                    "sk_forearm_l",
                    "sk_hand_l",
                    "sk_shoulder_r",
                    "sk_upperarm_r",
                    "sk_forearm_r",
                    "sk_hand_r",
                    "sk_root_hip",
                    "sk_thigh_l",
                    "sk_leg_l",
                    "sk_thigh_r",
                    "sk_leg_r",
                ]
                .as_slice(),
            ),
            (ORAL, ["sk_head"].as_slice()),
        ];
        for (bytes, expected) in cases {
            let file = FmdlFile::read(bytes).unwrap();
            let names: Vec<&str> = file
                .bones
                .iter()
                .map(|bone| file.string(usize::from(bone.name_string_id)).unwrap())
                .collect();
            assert_eq!(names, expected);
        }
    }

    #[test]
    fn bone_matrix_blocks_match_the_bone_count() {
        for (bytes, bones) in [(HIGHNECK, 7usize), (MOUTH, 6), (AU_LOW, 16)] {
            let file = FmdlFile::read(bytes).unwrap();
            assert_eq!(
                file.bone_matrices.as_deref().map(<[u8]>::len),
                Some(64 * bones)
            );
        }
        assert_eq!(FmdlFile::read(ORAL).unwrap().bone_matrices, Some(vec![]));
        assert_eq!(FmdlFile::read(PLACEHOLDER).unwrap().bone_matrices, None);
    }

    #[test]
    fn bad_string_references_error() {
        let file = FmdlFile::read(HIGHNECK).unwrap();
        assert!(matches!(
            file.string(usize::MAX),
            Err(FmdlError::BadStringReference { index: usize::MAX })
        ));
        // A record whose span runs past its block.
        let mut file = FmdlFile::read(HIGHNECK).unwrap();
        file.strings.push(StringRecord {
            string_block_id: 3,
            length: u16::MAX,
            offset: 0,
        });
        assert!(matches!(
            file.string(file.strings.len() - 1),
            Err(FmdlError::BadStringReference { .. })
        ));
    }
}
