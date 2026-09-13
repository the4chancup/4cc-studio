//! Vertex and face data: how a mesh's vertex-format entries resolve to
//! absolute spans of the section-1 buffer block, and the codec for each
//! datum encoding. Decode and encode at the same offsets and formats, so a
//! round trip is byte-identical.

use crate::format::FmdlError;
use crate::format::FmdlFile;
use crate::format::f16::{f16_to_f32, f32_to_f16};

/// What a vertex attribute holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatumType {
    /// Vertex position.
    Position,
    /// How strongly each bone pulls the vertex.
    BoneWeights,
    /// The surface normal.
    Normal,
    /// Per-vertex color.
    Color,
    /// Which bones the weights name.
    BoneIndices,
    /// Texture map 0.
    Uv0,
    /// Texture map 1.
    Uv1,
    /// Texture map 2.
    Uv2,
    /// Texture map 3.
    Uv3,
    /// The surface tangent.
    Tangent,
}

impl DatumType {
    /// The id the vertex-format record stores.
    pub fn id(self) -> u8 {
        match self {
            DatumType::Position => 0,
            DatumType::BoneWeights => 1,
            DatumType::Normal => 2,
            DatumType::Color => 3,
            DatumType::BoneIndices => 7,
            DatumType::Uv0 => 8,
            DatumType::Uv1 => 9,
            DatumType::Uv2 => 10,
            DatumType::Uv3 => 11,
            DatumType::Tangent => 14,
        }
    }

    /// The type for an id, `None` outside the set the format defines.
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => DatumType::Position,
            1 => DatumType::BoneWeights,
            2 => DatumType::Normal,
            3 => DatumType::Color,
            7 => DatumType::BoneIndices,
            8 => DatumType::Uv0,
            9 => DatumType::Uv1,
            10 => DatumType::Uv2,
            11 => DatumType::Uv3,
            14 => DatumType::Tangent,
            _ => return None,
        })
    }
}

/// How a vertex attribute is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatumFormat {
    /// Three `f32`.
    TripleFloat32,
    /// Two `f32`.
    DoubleFloat32,
    /// Four half floats.
    QuadFloat16,
    /// Two half floats.
    DoubleFloat16,
    /// Four bytes as fractions of 255.
    QuadFloat8,
    /// Four bytes as integers.
    QuadInt8,
}

impl DatumFormat {
    /// The id the vertex-format record stores.
    pub fn id(self) -> u8 {
        match self {
            DatumFormat::TripleFloat32 => 1,
            DatumFormat::DoubleFloat32 => 2,
            DatumFormat::QuadFloat16 => 6,
            DatumFormat::DoubleFloat16 => 7,
            DatumFormat::QuadFloat8 => 8,
            DatumFormat::QuadInt8 => 9,
        }
    }

    /// The format for an id, `None` outside the set the format defines.
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            1 => DatumFormat::TripleFloat32,
            2 => DatumFormat::DoubleFloat32,
            6 => DatumFormat::QuadFloat16,
            7 => DatumFormat::DoubleFloat16,
            8 => DatumFormat::QuadFloat8,
            9 => DatumFormat::QuadInt8,
            _ => return None,
        })
    }

    /// Bytes one vertex's attribute occupies.
    fn element_size(self) -> usize {
        match self {
            DatumFormat::TripleFloat32 => 12,
            DatumFormat::DoubleFloat32 => 8,
            DatumFormat::QuadFloat16 => 8,
            DatumFormat::DoubleFloat16 => 4,
            DatumFormat::QuadFloat8 => 4,
            DatumFormat::QuadInt8 => 4,
        }
    }
}

/// The only storage a datum type allows; `None` pairs are rejected.
fn allowed_format(datum_type: DatumType, format: DatumFormat) -> bool {
    match datum_type {
        DatumType::Position => format == DatumFormat::TripleFloat32,
        DatumType::BoneWeights | DatumType::Color => format == DatumFormat::QuadFloat8,
        DatumType::Normal | DatumType::Tangent => format == DatumFormat::QuadFloat16,
        DatumType::BoneIndices => format == DatumFormat::QuadInt8,
        DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3 => {
            matches!(
                format,
                DatumFormat::DoubleFloat16 | DatumFormat::DoubleFloat32
            )
        }
    }
}

/// Which uv map a uv datum type names; `None` for non-uv types.
fn uv_index(datum_type: DatumType) -> Option<usize> {
    match datum_type {
        DatumType::Uv0 => Some(0),
        DatumType::Uv1 => Some(1),
        DatumType::Uv2 => Some(2),
        DatumType::Uv3 => Some(3),
        _ => None,
    }
}

/// One attribute of a mesh's vertices, resolved to where it lives in
/// section-1 block 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    /// What the attribute holds.
    pub datum_type: DatumType,
    /// How it is stored.
    pub format: DatumFormat,
    /// Byte offset inside the buffer block where vertex 0's value sits.
    pub offset: usize,
    /// Bytes between consecutive vertices.
    pub stride: usize,
}

/// A mesh's vertices, decoded. Weights, indices and colors stay as the
/// bytes the file stores (a weight or color channel is `value / 255` of
/// full strength); floats are exact for what a half holds.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeshVertices {
    /// One `[x, y, z]` per vertex.
    pub positions: Vec<[f32; 3]>,
    /// One `[x, y, z, w]` per vertex when the format carries normals.
    pub normals: Option<Vec<[f32; 4]>>,
    /// One `[x, y, z, w]` per vertex when the format carries tangents.
    pub tangents: Option<Vec<[f32; 4]>>,
    /// One `[r, g, b, a]` per vertex when the format carries colors.
    pub colors: Option<Vec<[u8; 4]>>,
    /// Up to four UV maps in order; `uv_high_precision[i]` says whether map
    /// `i` is stored as two `f32` rather than two halves.
    pub uvs: Vec<Vec<[f32; 2]>>,
    /// Parallel to `uvs`: high-precision (double-float) storage flags.
    pub uv_high_precision: Vec<bool>,
    /// One `[w0..w3]` weight quad per vertex when the mesh is skinned.
    pub bone_weights: Option<Vec<[u8; 4]>>,
    /// One bone-index quad per vertex when the mesh is skinned.
    pub bone_indices: Option<Vec<[u8; 4]>>,
}

/// The byte span of attribute `attribute`'s value for vertex `vertex`.
fn span(attribute: &VertexAttribute, vertex: usize) -> std::ops::Range<usize> {
    let start = attribute.offset + vertex * attribute.stride;
    start..start + attribute.format.element_size()
}

fn read_f32_triplet(buffer: &[u8], range: std::ops::Range<usize>) -> Result<[f32; 3], FmdlError> {
    let bytes = buffer.get(range).ok_or(FmdlError::Truncated)?;
    Ok([
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
    ])
}

fn read_quad16(buffer: &[u8], range: std::ops::Range<usize>) -> Result<[f32; 4], FmdlError> {
    let bytes = buffer.get(range).ok_or(FmdlError::Truncated)?;
    let mut out = [0f32; 4];
    for (i, lane) in out.iter_mut().enumerate() {
        *lane = f16_to_f32(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
    }
    Ok(out)
}

fn read_uv(
    buffer: &[u8],
    range: std::ops::Range<usize>,
    high_precision: bool,
) -> Result<[f32; 2], FmdlError> {
    let bytes = buffer.get(range).ok_or(FmdlError::Truncated)?;
    Ok(if high_precision {
        [
            f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        ]
    } else {
        [
            f16_to_f32(u16::from_le_bytes([bytes[0], bytes[1]])),
            f16_to_f32(u16::from_le_bytes([bytes[2], bytes[3]])),
        ]
    })
}

fn read_quad8(buffer: &[u8], range: std::ops::Range<usize>) -> Result<[u8; 4], FmdlError> {
    let bytes = buffer.get(range).ok_or(FmdlError::Truncated)?;
    Ok([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn write_f32_triplet(
    buffer: &mut [u8],
    range: std::ops::Range<usize>,
    value: [f32; 3],
) -> Result<(), FmdlError> {
    let bytes = buffer.get_mut(range).ok_or(FmdlError::Truncated)?;
    bytes[0..4].copy_from_slice(&value[0].to_le_bytes());
    bytes[4..8].copy_from_slice(&value[1].to_le_bytes());
    bytes[8..12].copy_from_slice(&value[2].to_le_bytes());
    Ok(())
}

fn write_quad16(
    buffer: &mut [u8],
    range: std::ops::Range<usize>,
    value: [f32; 4],
) -> Result<(), FmdlError> {
    let bytes = buffer.get_mut(range).ok_or(FmdlError::Truncated)?;
    for (i, lane) in value.iter().enumerate() {
        bytes[2 * i..2 * i + 2].copy_from_slice(&f32_to_f16(*lane).to_le_bytes());
    }
    Ok(())
}

fn write_uv(
    buffer: &mut [u8],
    range: std::ops::Range<usize>,
    high_precision: bool,
    value: [f32; 2],
) -> Result<(), FmdlError> {
    let bytes = buffer.get_mut(range).ok_or(FmdlError::Truncated)?;
    if high_precision {
        bytes[0..4].copy_from_slice(&value[0].to_le_bytes());
        bytes[4..8].copy_from_slice(&value[1].to_le_bytes());
    } else {
        bytes[0..2].copy_from_slice(&f32_to_f16(value[0]).to_le_bytes());
        bytes[2..4].copy_from_slice(&f32_to_f16(value[1]).to_le_bytes());
    }
    Ok(())
}

fn write_quad8(
    buffer: &mut [u8],
    range: std::ops::Range<usize>,
    value: [u8; 4],
) -> Result<(), FmdlError> {
    let bytes = buffer.get_mut(range).ok_or(FmdlError::Truncated)?;
    bytes.copy_from_slice(&value);
    Ok(())
}

impl FmdlFile {
    /// The attributes of mesh `index`, resolved through its mesh-format
    /// assignment, mesh formats, vertex formats and buffer offsets to
    /// absolute spans of section-1 block 2.
    pub fn vertex_attributes(&self, mesh: usize) -> Result<Vec<VertexAttribute>, FmdlError> {
        let mesh_record = self.meshes.get(mesh).ok_or(FmdlError::BadReference {
            what: "mesh",
            index: mesh,
        })?;
        let format_id = usize::from(mesh_record.mesh_format_id);
        let assignment =
            self.mesh_format_assignments
                .get(format_id)
                .ok_or(FmdlError::BadReference {
                    what: "mesh format assignment",
                    index: format_id,
                })?;

        // Each mesh-format entry contributes a base offset and stride for
        // each of its vertex-format entries.
        let first_mesh_format = usize::from(assignment.first_mesh_format_id);
        let mesh_format_end = first_mesh_format + usize::from(assignment.mesh_format_entry_count);
        let mut bases: Vec<(usize, usize)> = Vec::new();
        for format_index in first_mesh_format..mesh_format_end {
            let mesh_format =
                self.mesh_formats
                    .get(format_index)
                    .ok_or(FmdlError::BadReference {
                        what: "mesh format",
                        index: format_index,
                    })?;
            let buffer_offset = self
                .buffer_offsets
                .get(usize::from(mesh_format.buffer_id))
                .ok_or(FmdlError::BadReference {
                    what: "buffer offset",
                    index: usize::from(mesh_format.buffer_id),
                })?;
            let base = buffer_offset.offset as usize + mesh_format.buffer_offset as usize;
            for _ in 0..mesh_format.vertex_format_entry_count {
                bases.push((base, usize::from(mesh_format.buffer_offset_increment)));
            }
        }

        let first_vertex_format = usize::from(assignment.first_vertex_format_id);
        if bases.len() != usize::from(assignment.vertex_format_entry_count) {
            return Err(FmdlError::InvalidVertexFormat(
                "mesh format entries do not cover the vertex format entries",
            ));
        }

        let mut attributes = Vec::new();
        for (entry, (base, stride)) in bases.iter().enumerate() {
            let record = self.vertex_formats.get(first_vertex_format + entry).ok_or(
                FmdlError::BadReference {
                    what: "vertex format",
                    index: first_vertex_format + entry,
                },
            )?;
            let datum_type = DatumType::from_id(record.datum_type).ok_or(
                FmdlError::UnsupportedVertexFormat {
                    datum_type: record.datum_type,
                    datum_format: record.datum_format,
                },
            )?;
            let format = DatumFormat::from_id(record.datum_format).ok_or(
                FmdlError::UnsupportedVertexFormat {
                    datum_type: record.datum_type,
                    datum_format: record.datum_format,
                },
            )?;
            if !allowed_format(datum_type, format) {
                return Err(FmdlError::UnsupportedVertexFormat {
                    datum_type: record.datum_type,
                    datum_format: record.datum_format,
                });
            }
            attributes.push(VertexAttribute {
                datum_type,
                format,
                offset: base + usize::from(record.offset),
                stride: *stride,
            });
        }

        // Set-level rules: no datum type twice, uv maps monotonic, bone
        // weights and indices paired.
        let mut seen = Vec::new();
        let mut uv_seen = [false; 4];
        let mut weights = false;
        let mut indices = false;
        for attribute in &attributes {
            if seen.contains(&attribute.datum_type) {
                return Err(FmdlError::InvalidVertexFormat(
                    "a datum type appears twice in one vertex format",
                ));
            }
            seen.push(attribute.datum_type);
            if let Some(index) = uv_index(attribute.datum_type) {
                uv_seen[index] = true;
            }
            weights |= attribute.datum_type == DatumType::BoneWeights;
            indices |= attribute.datum_type == DatumType::BoneIndices;
        }
        for index in 1..4 {
            if uv_seen[index] && !uv_seen[index - 1] {
                return Err(FmdlError::InvalidVertexFormat(
                    "a uv map is present without the lower ones",
                ));
            }
        }
        if weights != indices {
            return Err(FmdlError::InvalidVertexFormat(
                "bone weights and bone indices must come together",
            ));
        }
        Ok(attributes)
    }

    /// Decodes the mesh's vertex attributes into `MeshVertices`.
    pub fn decode_vertices(&self, mesh: usize) -> Result<MeshVertices, FmdlError> {
        let attributes = self.vertex_attributes(mesh)?;
        let mesh_record = self.meshes.get(mesh).ok_or(FmdlError::BadReference {
            what: "mesh",
            index: mesh,
        })?;
        let vertex_count = usize::from(mesh_record.vertex_count);
        let buffer = self.buffer.as_deref().ok_or(FmdlError::BadReference {
            what: "vertex buffer",
            index: 2,
        })?;

        let mut vertices = MeshVertices {
            positions: Vec::with_capacity(vertex_count),
            ..MeshVertices::default()
        };
        let mut uv_slots: [Option<Vec<[f32; 2]>>; 4] = [None, None, None, None];
        let mut uv_precision = [false; 4];
        for attribute in &attributes {
            for vertex in 0..vertex_count {
                let range = span(attribute, vertex);
                match attribute.datum_type {
                    DatumType::Position => {
                        vertices.positions.push(read_f32_triplet(buffer, range)?);
                    }
                    DatumType::BoneWeights => {
                        vertices
                            .bone_weights
                            .get_or_insert_with(Vec::new)
                            .push(read_quad8(buffer, range)?);
                    }
                    DatumType::Normal => {
                        vertices
                            .normals
                            .get_or_insert_with(Vec::new)
                            .push(read_quad16(buffer, range)?);
                    }
                    DatumType::Color => {
                        vertices
                            .colors
                            .get_or_insert_with(Vec::new)
                            .push(read_quad8(buffer, range)?);
                    }
                    DatumType::BoneIndices => {
                        vertices
                            .bone_indices
                            .get_or_insert_with(Vec::new)
                            .push(read_quad8(buffer, range)?);
                    }
                    DatumType::Tangent => {
                        vertices
                            .tangents
                            .get_or_insert_with(Vec::new)
                            .push(read_quad16(buffer, range)?);
                    }
                    DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3 => {
                        let index = uv_index(attribute.datum_type)
                            .unwrap_or_else(|| unreachable!("matched uv types only"));
                        uv_precision[index] = attribute.format == DatumFormat::DoubleFloat32;
                        uv_slots[index].get_or_insert_with(Vec::new).push(read_uv(
                            buffer,
                            range,
                            uv_precision[index],
                        )?);
                    }
                }
            }
        }
        for slot in uv_slots.iter_mut().flatten() {
            vertices.uvs.push(std::mem::take(slot));
        }
        for high in uv_precision.iter().take(vertices.uvs.len()) {
            vertices.uv_high_precision.push(*high);
        }
        Ok(vertices)
    }

    /// The mesh's triangles as indices into its own vertices.
    pub fn decode_faces(&self, mesh: usize) -> Result<Vec<[u16; 3]>, FmdlError> {
        let mesh_record = self.meshes.get(mesh).ok_or(FmdlError::BadReference {
            what: "mesh",
            index: mesh,
        })?;
        let face_id = usize::try_from(mesh_record.first_face_index_id).map_err(|_| {
            FmdlError::BadReference {
                what: "face index record",
                index: usize::MAX,
            }
        })?;
        let face_record = self
            .face_indices
            .get(face_id)
            .ok_or(FmdlError::BadReference {
                what: "face index record",
                index: face_id,
            })?;
        let face_buffer = self.buffer_offsets.get(2).ok_or(FmdlError::BadReference {
            what: "face buffer",
            index: 2,
        })?;
        let buffer = self.buffer.as_deref().ok_or(FmdlError::BadReference {
            what: "vertex buffer",
            index: 2,
        })?;

        let first = mesh_record.first_face_vertex_index as usize
            + face_record.first_face_vertex_index as usize;
        let count = face_record.face_vertex_count as usize;
        let mut faces = Vec::with_capacity(count / 3);
        for start in (first..first + count).step_by(3) {
            let position = face_buffer.offset as usize + start * 2;
            let bytes = buffer
                .get(position..position + 6)
                .ok_or(FmdlError::Truncated)?;
            faces.push([
                u16::from_le_bytes([bytes[0], bytes[1]]),
                u16::from_le_bytes([bytes[2], bytes[3]]),
                u16::from_le_bytes([bytes[4], bytes[5]]),
            ]);
        }
        Ok(faces)
    }

    /// Writes `vertices` back into the buffer at the mesh's existing
    /// attribute offsets and formats. The vertex count and the attribute
    /// set must match the format.
    pub fn encode_vertices(
        &mut self,
        mesh: usize,
        vertices: &MeshVertices,
    ) -> Result<(), FmdlError> {
        let attributes = self.vertex_attributes(mesh)?;
        let mesh_record = self.meshes.get(mesh).ok_or(FmdlError::BadReference {
            what: "mesh",
            index: mesh,
        })?;
        let vertex_count = usize::from(mesh_record.vertex_count);
        if vertices.positions.len() != vertex_count {
            return Err(FmdlError::VertexMismatch(
                "position count does not match the mesh's vertex count",
            ));
        }
        let has = |datum_type: DatumType| {
            attributes
                .iter()
                .any(|attribute| attribute.datum_type == datum_type)
        };
        for (count, present) in [
            (
                vertices.normals.as_ref().map(Vec::len),
                has(DatumType::Normal),
            ),
            (
                vertices.tangents.as_ref().map(Vec::len),
                has(DatumType::Tangent),
            ),
            (
                vertices.colors.as_ref().map(Vec::len),
                has(DatumType::Color),
            ),
            (
                vertices.bone_weights.as_ref().map(Vec::len),
                has(DatumType::BoneWeights),
            ),
            (
                vertices.bone_indices.as_ref().map(Vec::len),
                has(DatumType::BoneIndices),
            ),
        ] {
            match (count, present) {
                (Some(count), true) if count != vertex_count => {
                    return Err(FmdlError::VertexMismatch("wrong vertex count"));
                }
                (Some(_), false) => {
                    return Err(FmdlError::VertexMismatch(
                        "attribute not in the mesh format",
                    ));
                }
                (None, true) => {
                    return Err(FmdlError::VertexMismatch("attribute missing"));
                }
                _ => {}
            }
        }
        let uv_attributes: Vec<&VertexAttribute> = attributes
            .iter()
            .filter(|attribute| uv_index(attribute.datum_type).is_some())
            .collect();
        if vertices.uvs.len() != uv_attributes.len()
            || vertices.uv_high_precision.len() != vertices.uvs.len()
        {
            return Err(FmdlError::VertexMismatch(
                "uv maps do not match the mesh format",
            ));
        }
        for attribute in &uv_attributes {
            let map = uv_index(attribute.datum_type)
                .unwrap_or_else(|| unreachable!("filtered uv types only"));
            if vertices.uvs[map].len() != vertex_count {
                return Err(FmdlError::VertexMismatch("wrong vertex count"));
            }
            if vertices.uv_high_precision[map] != (attribute.format == DatumFormat::DoubleFloat32) {
                return Err(FmdlError::VertexMismatch(
                    "uv precision does not match the mesh format",
                ));
            }
        }

        let buffer = self.buffer.as_deref_mut().ok_or(FmdlError::BadReference {
            what: "vertex buffer",
            index: 2,
        })?;
        for attribute in &attributes {
            for (vertex, range) in (0..vertex_count).map(|v| (v, span(attribute, v))) {
                match attribute.datum_type {
                    DatumType::Position => {
                        write_f32_triplet(buffer, range, vertices.positions[vertex])?;
                    }
                    DatumType::BoneWeights => {
                        let values = vertices
                            .bone_weights
                            .as_ref()
                            .unwrap_or_else(|| unreachable!("checked above"));
                        write_quad8(buffer, range, values[vertex])?;
                    }
                    DatumType::Normal => {
                        let values = vertices
                            .normals
                            .as_ref()
                            .unwrap_or_else(|| unreachable!("checked above"));
                        write_quad16(buffer, range, values[vertex])?;
                    }
                    DatumType::Color => {
                        let values = vertices
                            .colors
                            .as_ref()
                            .unwrap_or_else(|| unreachable!("checked above"));
                        write_quad8(buffer, range, values[vertex])?;
                    }
                    DatumType::BoneIndices => {
                        let values = vertices
                            .bone_indices
                            .as_ref()
                            .unwrap_or_else(|| unreachable!("checked above"));
                        write_quad8(buffer, range, values[vertex])?;
                    }
                    DatumType::Tangent => {
                        let values = vertices
                            .tangents
                            .as_ref()
                            .unwrap_or_else(|| unreachable!("checked above"));
                        write_quad16(buffer, range, values[vertex])?;
                    }
                    DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3 => {
                        let map = uv_index(attribute.datum_type)
                            .unwrap_or_else(|| unreachable!("matched uv types only"));
                        write_uv(
                            buffer,
                            range,
                            vertices.uv_high_precision[map],
                            vertices.uvs[map][vertex],
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes `faces` back at the mesh's existing face range; the count must
    /// equal the record's.
    pub fn encode_faces(&mut self, mesh: usize, faces: &[[u16; 3]]) -> Result<(), FmdlError> {
        let mesh_record = self.meshes.get(mesh).ok_or(FmdlError::BadReference {
            what: "mesh",
            index: mesh,
        })?;
        let face_id = usize::try_from(mesh_record.first_face_index_id).map_err(|_| {
            FmdlError::BadReference {
                what: "face index record",
                index: usize::MAX,
            }
        })?;
        let face_record = self
            .face_indices
            .get(face_id)
            .ok_or(FmdlError::BadReference {
                what: "face index record",
                index: face_id,
            })?;
        if faces.len() * 3 != face_record.face_vertex_count as usize {
            return Err(FmdlError::VertexMismatch(
                "face count does not match the mesh's face record",
            ));
        }
        let face_buffer = self.buffer_offsets.get(2).ok_or(FmdlError::BadReference {
            what: "face buffer",
            index: 2,
        })?;
        let first = mesh_record.first_face_vertex_index as usize
            + face_record.first_face_vertex_index as usize;
        let buffer = self.buffer.as_deref_mut().ok_or(FmdlError::BadReference {
            what: "vertex buffer",
            index: 2,
        })?;
        for (face_index, face) in faces.iter().enumerate() {
            let position = face_buffer.offset as usize + (first + face_index * 3) * 2;
            let bytes = buffer
                .get_mut(position..position + 6)
                .ok_or(FmdlError::Truncated)?;
            for (lane, index) in face.iter().enumerate() {
                bytes[2 * lane..2 * lane + 2].copy_from_slice(&index.to_le_bytes());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::f16::{f16_to_f32, f32_to_f16};

    const HIGHNECK: &[u8] = include_bytes!("../../tests/fixtures/konami_highneck.fmdl");
    const MOUTH: &[u8] = include_bytes!("../../tests/fixtures/konami_mouth.fmdl");
    const AU_LOW: &[u8] = include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl");
    const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
    const PLACEHOLDER: &[u8] = include_bytes!("../../tests/fixtures/addon_placeholder.fmdl");

    const FIXTURES: [&[u8]; 5] = [HIGHNECK, MOUTH, AU_LOW, ORAL, PLACEHOLDER];

    #[test]
    fn vertex_and_face_round_trips_are_byte_identical() {
        for bytes in FIXTURES {
            let file = FmdlFile::read(bytes).unwrap();
            let mut edited = file.clone();
            for mesh in 0..file.meshes.len() {
                let vertices = file.decode_vertices(mesh).unwrap();
                edited.encode_vertices(mesh, &vertices).unwrap();
                let faces = file.decode_faces(mesh).unwrap();
                edited.encode_faces(mesh, &faces).unwrap();
            }
            assert_eq!(edited.buffer, file.buffer);
        }
    }

    #[test]
    fn half_exhaustive_round_trip() {
        for bits in 0u16..=u16::MAX {
            let value = f16_to_f32(bits);
            let back = f32_to_f16(value);
            if bits & 0x7C00 == 0x7C00 && bits & 0x03FF != 0 {
                // NaN in, NaN out.
                assert!(value.is_nan(), "{bits:#06x}");
                assert!(back & 0x7C00 == 0x7C00 && back & 0x03FF != 0, "{bits:#06x}");
            } else {
                assert_eq!(back, bits, "{bits:#06x}");
            }
        }
    }

    #[test]
    fn half_rounding_cases() {
        assert_eq!(f32_to_f16(1.0), 0x3C00);
        assert_eq!(f32_to_f16(-2.0), 0xC000);
        assert_eq!(f32_to_f16(65504.0), 0x7BFF);
        assert_eq!(f32_to_f16(65520.0), 0x7C00);
        assert_eq!(f32_to_f16(1.0 + 2f32.powi(-11)), 0x3C00);
        assert_eq!(f32_to_f16(1.0 + 3.0 * 2f32.powi(-11)), 0x3C02);
        assert_eq!(f32_to_f16(2f32.powi(-25)), 0x0000);
        assert_eq!(f32_to_f16(2f32.powi(-24)), 0x0001);
    }

    #[test]
    fn decoded_content_is_sane() {
        let file = FmdlFile::read(HIGHNECK).unwrap();
        let mesh = &file.meshes[0];
        let vertices = file.decode_vertices(0).unwrap();
        let count = usize::from(mesh.vertex_count);
        assert_eq!(vertices.positions.len(), count);
        assert_eq!(vertices.normals.as_ref().unwrap().len(), count);
        assert!(!vertices.uvs.is_empty());
        let weights = vertices.bone_weights.as_ref().unwrap();
        let indices = vertices.bone_indices.as_ref().unwrap();
        assert_eq!(weights.len(), count);
        assert_eq!(indices.len(), count);
        let mut bad_weight_sums = 0;
        for quad in weights {
            let sum: u32 = quad.iter().map(|w| u32::from(*w)).sum();
            if sum != 255 && quad.iter().any(|w| *w != 0) {
                bad_weight_sums += 1;
            }
        }
        assert_eq!(bad_weight_sums, 0, "weight quads summing off 255");

        let placeholder = FmdlFile::read(PLACEHOLDER).unwrap();
        assert!(
            placeholder
                .decode_vertices(0)
                .unwrap()
                .bone_weights
                .is_none()
        );

        for bytes in FIXTURES {
            let file = FmdlFile::read(bytes).unwrap();
            for mesh_index in 0..file.meshes.len() {
                let mesh = &file.meshes[mesh_index];
                let face_record = &file.face_indices[mesh.first_face_index_id as usize];
                let faces = file.decode_faces(mesh_index).unwrap();
                assert_eq!(faces.len() * 3, face_record.face_vertex_count as usize);
                for face in &faces {
                    for index in face {
                        assert!((*index) < mesh.vertex_count);
                    }
                }
            }
        }
    }

    #[test]
    fn out_of_range_and_mismatch_errors() {
        let file = FmdlFile::read(HIGHNECK).unwrap();
        assert!(matches!(
            file.decode_vertices(99),
            Err(FmdlError::BadReference {
                what: "mesh",
                index: 99
            })
        ));
        let mut edited = file.clone();
        assert!(matches!(
            edited.encode_faces(0, &[]),
            Err(FmdlError::VertexMismatch(_))
        ));

        let mut broken = file.clone();
        // Corrupt the datum type of the first vertex-format entry mesh 0's
        // assignment covers.
        let first = broken.mesh_format_assignments[broken.meshes[0].mesh_format_id as usize]
            .first_vertex_format_id as usize;
        broken.vertex_formats[first].datum_type = 5;
        assert!(matches!(
            broken.vertex_attributes(0),
            Err(FmdlError::UnsupportedVertexFormat { datum_type: 5, .. })
        ));
    }
}
