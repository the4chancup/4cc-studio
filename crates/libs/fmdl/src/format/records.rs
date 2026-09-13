//! Typed views of the section-0 record blocks, one struct per block id.
//!
//! Every struct has one named field per record byte in file order, sized to
//! the block's record size; bytes whose meaning is not known are
//! `unknown_<offset>` fields so they round-trip verbatim. The fixture's
//! byte-identity test is the check that every field size is right.

use binrw::{BinRead, BinWrite};

/// A bone (block 0, 48 bytes): name, parent, bounding box and the two
/// bind-pose positions.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct BoneRecord {
    /// Index into the file's strings.
    pub name_string_id: u16,
    /// Index into `bones`, -1 for a root.
    pub parent_bone_id: i16,
    /// Index into `bounding_boxes`.
    pub bounding_box_id: u16,
    /// Unknown.
    pub unknown_0x06: u16,
    /// Unknown.
    pub unknown_0x08: u64,
    /// Position relative to the parent bone.
    pub local_position: [f32; 4],
    /// Position in model space.
    pub world_position: [f32; 4],
}

/// A mesh group (block 1, 8 bytes).
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MeshGroupRecord {
    /// Index into the file's strings.
    pub name_string_id: u16,
    /// 0 when the group is visible.
    pub invisible: u16,
    /// Index into `mesh_groups`, -1 for a root.
    pub parent_mesh_group_id: i16,
    /// Unknown.
    pub unknown_0x06: i16,
}

/// A mesh group assignment (block 2, 32 bytes): a run of consecutive meshes
/// and a bounding box handed to one mesh group.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MeshGroupAssignmentRecord {
    /// Unknown.
    pub unknown_0x00: [u8; 4],
    /// Index into `mesh_groups`.
    pub mesh_group_id: u16,
    /// How many consecutive meshes the group takes.
    pub mesh_count: u16,
    /// Index into `meshes` where the group's run starts.
    pub first_mesh_id: u16,
    /// Index into `bounding_boxes`.
    pub bounding_box_id: u16,
    /// Unknown.
    pub unknown_0x0c: [u8; 4],
    /// Unknown.
    pub unknown_0x10: u16,
    /// Unknown.
    pub unknown_0x12: [u8; 14],
}

/// A mesh (block 3, 48 bytes): draw flags and the ids that tie it to its
/// material, bone group, vertex format and face run.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MeshRecord {
    /// Transparency draw flags.
    pub alpha_flags: u8,
    /// Shadow draw flags.
    pub shadow_flags: u8,
    /// Unknown.
    pub unknown_0x02: [u8; 2],
    /// Index into `material_instances`.
    pub material_instance_id: u16,
    /// Index into `bone_groups`.
    pub bone_group_id: u16,
    /// Index into `mesh_format_assignments`.
    pub mesh_format_id: u16,
    /// Vertices in the mesh.
    pub vertex_count: u16,
    /// Unknown.
    pub unknown_0x0c: [u8; 4],
    /// Offset into the face buffer where the mesh's face run starts.
    pub first_face_vertex_index: u32,
    /// Vertex indices in the face run.
    pub face_vertex_count: u32,
    /// Index into `face_indices`.
    pub first_face_index_id: u64,
    /// Unknown.
    pub unknown_0x20: [u8; 16],
}

/// A material instance (block 4, 16 bytes): a material plus the texture and
/// parameter runs that fill it.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MaterialInstanceRecord {
    /// Index into the file's strings.
    pub name_string_id: u16,
    /// Unknown.
    pub unknown_0x02: u16,
    /// Index into `materials`.
    pub material_id: u16,
    /// Texture assignments the instance takes.
    pub texture_count: u8,
    /// Parameter assignments the instance takes.
    pub material_parameter_count: u8,
    /// Index into `parameter_assignments` where the texture run starts.
    pub first_texture_id: u16,
    /// Index into `parameter_assignments` where the parameter run starts.
    pub first_material_parameter_id: u16,
    /// Unknown.
    pub unknown_0x0c: u32,
}

/// A bone group (block 5, 68 bytes): up to 32 bone ids a skinned mesh maps
/// its bone indices onto.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct BoneGroupRecord {
    /// Unknown (4 in written files).
    pub unknown_0x00: u16,
    /// How many of `bone_ids` are used.
    pub entry_count: u16,
    /// Indices into `bones`; only the first `entry_count` are meaningful.
    pub bone_ids: [u16; 32],
}

/// A texture reference (block 6, 4 bytes).
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct TextureRecord {
    /// Index into the file's strings: the texture file name.
    pub filename_string_id: u16,
    /// Index into the file's strings: the directory.
    pub directory_string_id: u16,
}

/// A texture or material parameter assignment (block 7, 4 bytes): a
/// parameter name bound to a texture or parameter-table index.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct ParameterAssignmentRecord {
    /// Index into the file's strings: the parameter name.
    pub parameter_string_id: u16,
    /// Index into `textures` or into the material-parameter table.
    pub reference_id: u16,
}

/// A material (block 8, 4 bytes): shader and technique names.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MaterialRecord {
    /// Index into the file's strings: the shader name.
    pub shader_string_id: u16,
    /// Index into the file's strings: the technique name.
    pub technique_string_id: u16,
}

/// A mesh format assignment (block 9, 8 bytes): the runs of mesh-format and
/// vertex-format entries that together describe one mesh's vertex layout.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MeshFormatAssignmentRecord {
    /// Mesh-format entries taken.
    pub mesh_format_entry_count: u8,
    /// Vertex-format entries taken.
    pub vertex_format_entry_count: u8,
    /// Index into the entry list where the uv entries start.
    pub first_uv_index: u8,
    /// Uv entries taken.
    pub uv_index_count: u8,
    /// Index into `mesh_formats` where the run starts.
    pub first_mesh_format_id: u16,
    /// Index into `vertex_formats` where the run starts.
    pub first_vertex_format_id: u16,
}

/// A mesh format entry (block 10, 8 bytes): which buffer a vertex attribute
/// group lives in and how far apart its vertices sit.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct MeshFormatRecord {
    /// Index into `buffer_offsets`.
    pub buffer_id: u8,
    /// Vertex-format entries this entry covers.
    pub vertex_format_entry_count: u8,
    /// Bytes between consecutive vertices.
    pub buffer_offset_increment: u8,
    /// The entry's type byte.
    pub mesh_format_type: u8,
    /// Offset inside the buffer where the data starts.
    pub buffer_offset: u32,
}

/// A vertex format entry (block 11, 4 bytes): one vertex attribute's kind,
/// encoding and offset inside the vertex.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct VertexFormatRecord {
    /// Which attribute the entry carries (position, normal, uv, ...).
    pub datum_type: u8,
    /// The attribute's binary encoding.
    pub datum_format: u8,
    /// Byte offset inside the vertex.
    pub offset: u16,
}

/// A string descriptor (block 12, 8 bytes): a span inside a section-1 block.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct StringRecord {
    /// The section-1 block holding the string data.
    pub string_block_id: u16,
    /// Byte length of the string.
    pub length: u16,
    /// Byte offset inside the block.
    pub offset: u32,
}

/// A bounding box (block 13, 32 bytes): file order is max corner first.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct BoundingBoxRecord {
    /// The maximum corner.
    pub max: [f32; 4],
    /// The minimum corner.
    pub min: [f32; 4],
}

/// A buffer offset (block 14, 16 bytes): where one of the big buffers sits
/// inside the section-1 buffer block.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct BufferOffsetRecord {
    /// End-of-data marker (the format's own field).
    pub eof: u32,
    /// Byte length of the buffer's data.
    pub length: u32,
    /// Byte offset inside the buffer block.
    pub offset: u32,
    /// Unknown.
    pub unknown_0x0c: u32,
}

/// The level-of-detail declaration (block 16, 16 bytes, one record).
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct LevelOfDetailRecord {
    /// How many levels of detail the model carries.
    pub lod_count: u32,
    /// Unknown.
    pub unknown_0x04: [f32; 3],
}

/// A face-index run (block 17, 8 bytes): the level-of-detail face slice a
/// mesh draws.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct FaceIndexRecord {
    /// Offset into the mesh's face run.
    pub first_face_vertex_index: u32,
    /// Vertex indices in the slice.
    pub face_vertex_count: u32,
}

/// A block-18 record (8 bytes); purpose unknown.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct Block18Record {
    /// The record's raw bytes.
    pub bytes: [u8; 8],
}

/// A block-20 record (128 bytes); purpose unknown.
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct Block20Record {
    /// The record's raw bytes.
    pub bytes: [u8; 128],
}
