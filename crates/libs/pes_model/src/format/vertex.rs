//! The vertex and face codec: `Geometry`'s fields decoded into
//! `MeshVertices` and re-encoded in place, and `FaceStream`'s index list as
//! face triples per LOD level. Decode and encode at the same formats, so a
//! round trip is byte-identical.
//!
//! The encodings are all `f32` and bytes (there are no half floats in
//! `.model`): positions, normals, tangents and bitangents are
//! `TripleFloat32`, uv maps `DoubleFloat32`, colors `QuadFloat8`, bone
//! indices `QuadInt8`, and bone weights `DoubleFloat32`, `TripleFloat32` or
//! `QuadFloat32` — the file may store only two or three weights per vertex;
//! `MeshVertices::bone_weight_width` remembers which.

use crate::format::ModelError;
use crate::format::datum::{DatumFormat, DatumType};
use crate::format::model::{FaceStream, Geometry, VertexField};

/// A geometry's vertices decoded from its fields, one entry per vertex in each list. Bone
/// weights are the `f32`s the file stores (two, three or four per vertex, zero-padded to four
/// here; `bone_weight_width` remembers how many the file stores so a re-encode is byte-identical).
#[derive(Debug, Clone, PartialEq)]
pub struct MeshVertices {
    /// One `[x, y, z]` per vertex.
    pub positions: Vec<[f32; 3]>,
    /// One `[x, y, z]` per vertex, when the file stores normals.
    pub normals: Option<Vec<[f32; 3]>>,
    /// One `[x, y, z]` per vertex, when the file stores tangents.
    pub tangents: Option<Vec<[f32; 3]>>,
    /// One `[x, y, z]` per vertex, when the file stores bitangents.
    pub bitangents: Option<Vec<[f32; 3]>>,
    /// Four bytes per vertex, as stored (`255` is full).
    pub colors: Option<Vec<[u8; 4]>>,
    /// `uvs[i]` is map `Uv{i}`; maps are consecutive from `Uv0`.
    pub uvs: Vec<Vec<[f32; 2]>>,
    /// Indices into the mesh's bone group.
    pub bone_indices: Option<Vec<[u8; 4]>>,
    /// How strongly each bone pulls the vertex, zero-padded to four.
    pub bone_weights: Option<Vec<[f32; 4]>>,
    /// 2, 3 or 4: how many weights per vertex the file stores (`DoubleFloat32`,
    /// `TripleFloat32`, `QuadFloat32`). Meaningless when `bone_weights` is `None`; 4 for new
    /// meshes.
    pub bone_weight_width: u8,
}

/// Three little-endian `f32` per element.
fn triples(data: &[u8]) -> Vec<[f32; 3]> {
    let (words, _) = data.as_chunks::<4>();
    words
        .as_chunks::<3>()
        .0
        .iter()
        .map(|triple| {
            [
                f32::from_le_bytes(triple[0]),
                f32::from_le_bytes(triple[1]),
                f32::from_le_bytes(triple[2]),
            ]
        })
        .collect()
}

/// Two little-endian `f32` per element.
fn doubles(data: &[u8]) -> Vec<[f32; 2]> {
    let (words, _) = data.as_chunks::<4>();
    words
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| [f32::from_le_bytes(pair[0]), f32::from_le_bytes(pair[1])])
        .collect()
}

/// Four bytes per element.
fn quads(data: &[u8]) -> Vec<[u8; 4]> {
    data.as_chunks::<4>().0.to_vec()
}

/// Bone weights, `width` `f32`s per vertex zero-padded to four.
fn weights(data: &[u8], width: usize) -> Vec<[f32; 4]> {
    let (words, _) = data.as_chunks::<4>();
    words
        .chunks_exact(width)
        .map(|vertex| {
            let mut quad = [0.0f32; 4];
            for (slot, word) in quad.iter_mut().zip(vertex) {
                *slot = f32::from_le_bytes(*word);
            }
            quad
        })
        .collect()
}

/// The format a width corresponds to; 2, 3 or 4.
fn weight_format(width: u8) -> Option<DatumFormat> {
    match width {
        2 => Some(DatumFormat::DoubleFloat32),
        3 => Some(DatumFormat::TripleFloat32),
        4 => Some(DatumFormat::QuadFloat32),
        _ => None,
    }
}

/// How many weights a bone-weight format stores per vertex.
fn weight_width(format: DatumFormat) -> Option<usize> {
    match format {
        DatumFormat::DoubleFloat32 => Some(2),
        DatumFormat::TripleFloat32 => Some(3),
        DatumFormat::QuadFloat32 => Some(4),
        DatumFormat::Uint16
        | DatumFormat::Uint32
        | DatumFormat::Float32
        | DatumFormat::Float32Matrix34
        | DatumFormat::QuadInt8
        | DatumFormat::QuadFloat8 => None,
    }
}

/// Whether the pairing is one the codec knows; the raw words go to
/// `UnsupportedVertexFormat` when not.
fn check_pairing(field: &VertexField) -> Result<(), ModelError> {
    let ok = match field.datum_type {
        DatumType::Position | DatumType::Normal | DatumType::Tangent | DatumType::Bitangent => {
            field.datum_format == DatumFormat::TripleFloat32
        }
        DatumType::Color => field.datum_format == DatumFormat::QuadFloat8,
        DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3 => {
            field.datum_format == DatumFormat::DoubleFloat32
        }
        DatumType::BoneIndices => field.datum_format == DatumFormat::QuadInt8,
        DatumType::BoneWeights => matches!(
            field.datum_format,
            DatumFormat::DoubleFloat32 | DatumFormat::TripleFloat32 | DatumFormat::QuadFloat32
        ),
    };
    if ok {
        Ok(())
    } else {
        Err(ModelError::UnsupportedVertexFormat {
            datum_type: field.datum_type.word(),
            datum_format: field.datum_format.word(),
        })
    }
}

/// Encodes `[f32; 3]` values into `TripleFloat32` bytes.
fn encode_triples(values: &[[f32; 3]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(12 * values.len());
    for value in values {
        for component in value {
            data.extend_from_slice(&component.to_le_bytes());
        }
    }
    data
}

/// Encodes `[f32; 2]` values into `DoubleFloat32` bytes.
fn encode_doubles(values: &[[f32; 2]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 * values.len());
    for value in values {
        for component in value {
            data.extend_from_slice(&component.to_le_bytes());
        }
    }
    data
}

/// Encodes `[u8; 4]` values into bytes.
fn encode_quads(values: &[[u8; 4]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 * values.len());
    for value in values {
        data.extend_from_slice(value);
    }
    data
}

/// Encodes bone weights at `width` `f32`s per vertex.
fn encode_weights(values: &[[f32; 4]], width: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 * width * values.len());
    for value in values {
        for component in &value[..width] {
            data.extend_from_slice(&component.to_le_bytes());
        }
    }
    data
}

impl MeshVertices {
    /// How many vertices.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether there are no vertices.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Fresh fields for these vertices in the order every Konami file uses: position, normal,
    /// bitangent, tangent, uv0, uv1, ..., color, bone indices, bone weights (each only when
    /// present). This is `Geometry::vertex_fields` for a new geometry.
    pub fn to_fields(&self) -> Result<Vec<VertexField>, ModelError> {
        let len = self.positions.len();
        for (list, what) in [
            (self.normals.as_deref(), "normal count"),
            (self.bitangents.as_deref(), "bitangent count"),
            (self.tangents.as_deref(), "tangent count"),
        ] {
            if let Some(list) = list
                && list.len() != len
            {
                return Err(ModelError::VertexMismatch(what));
            }
        }
        for (list, what) in [
            (self.colors.as_deref(), "color count"),
            (self.bone_indices.as_deref(), "bone index count"),
        ] {
            if let Some(list) = list
                && list.len() != len
            {
                return Err(ModelError::VertexMismatch(what));
            }
        }
        if let Some(weights) = &self.bone_weights
            && weights.len() != len
        {
            return Err(ModelError::VertexMismatch("bone weight count"));
        }
        for uvs in &self.uvs {
            if uvs.len() != len {
                return Err(ModelError::VertexMismatch("uv count"));
            }
        }
        if self.uvs.len() > 4 {
            return Err(ModelError::VertexMismatch("more than four uv maps"));
        }

        let mut fields = Vec::new();
        fields.push(VertexField {
            datum_type: DatumType::Position,
            datum_format: DatumFormat::TripleFloat32,
            data: encode_triples(&self.positions),
        });
        if let Some(normals) = &self.normals {
            fields.push(VertexField {
                datum_type: DatumType::Normal,
                datum_format: DatumFormat::TripleFloat32,
                data: encode_triples(normals),
            });
        }
        if let Some(bitangents) = &self.bitangents {
            fields.push(VertexField {
                datum_type: DatumType::Bitangent,
                datum_format: DatumFormat::TripleFloat32,
                data: encode_triples(bitangents),
            });
        }
        if let Some(tangents) = &self.tangents {
            fields.push(VertexField {
                datum_type: DatumType::Tangent,
                datum_format: DatumFormat::TripleFloat32,
                data: encode_triples(tangents),
            });
        }
        for (index, uvs) in self.uvs.iter().enumerate() {
            let datum_type = match index {
                0 => DatumType::Uv0,
                1 => DatumType::Uv1,
                2 => DatumType::Uv2,
                _ => DatumType::Uv3,
            };
            fields.push(VertexField {
                datum_type,
                datum_format: DatumFormat::DoubleFloat32,
                data: encode_doubles(uvs),
            });
        }
        if let Some(colors) = &self.colors {
            fields.push(VertexField {
                datum_type: DatumType::Color,
                datum_format: DatumFormat::QuadFloat8,
                data: encode_quads(colors),
            });
        }
        if let Some(indices) = &self.bone_indices {
            fields.push(VertexField {
                datum_type: DatumType::BoneIndices,
                datum_format: DatumFormat::QuadInt8,
                data: encode_quads(indices),
            });
        }
        if let Some(weights) = &self.bone_weights {
            let format = weight_format(self.bone_weight_width)
                .ok_or(ModelError::VertexMismatch("bone weight width"))?;
            fields.push(VertexField {
                datum_type: DatumType::BoneWeights,
                datum_format: format,
                data: encode_weights(weights, self.bone_weight_width as usize),
            });
        }
        Ok(fields)
    }
}

impl Geometry {
    /// Decodes the vertex fields. Every field must have the same vertex count, no datum type
    /// may appear twice, a position field is required, uv maps must be `Uv0..UvN` without a gap.
    pub fn decode_vertices(&self) -> Result<MeshVertices, ModelError> {
        for field in &self.vertex_fields {
            check_pairing(field)?;
        }
        let count = self.vertex_fields.first().map_or(0, |field| field.count());
        let mut seen = Vec::with_capacity(self.vertex_fields.len());
        for field in &self.vertex_fields {
            if field.count() != count {
                return Err(ModelError::InvalidVertexFormat(
                    "field vertex counts differ",
                ));
            }
            if seen.contains(&field.datum_type) {
                return Err(ModelError::InvalidVertexFormat("duplicate field"));
            }
            seen.push(field.datum_type);
        }
        if !seen.contains(&DatumType::Position) {
            return Err(ModelError::InvalidVertexFormat("no position field"));
        }

        let mut vertices = MeshVertices {
            positions: Vec::new(),
            normals: None,
            tangents: None,
            bitangents: None,
            colors: None,
            uvs: Vec::new(),
            bone_indices: None,
            bone_weights: None,
            bone_weight_width: 4,
        };
        for field in &self.vertex_fields {
            match field.datum_type {
                DatumType::Position => vertices.positions = triples(&field.data),
                DatumType::Normal => vertices.normals = Some(triples(&field.data)),
                DatumType::Tangent => vertices.tangents = Some(triples(&field.data)),
                DatumType::Bitangent => vertices.bitangents = Some(triples(&field.data)),
                DatumType::Color => vertices.colors = Some(quads(&field.data)),
                DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3 => {
                    // Uv maps must be Uv0..UvN without a gap.
                    if field.datum_type as usize - DatumType::Uv0 as usize != vertices.uvs.len() {
                        return Err(ModelError::InvalidVertexFormat("uv maps not consecutive"));
                    }
                    vertices.uvs.push(doubles(&field.data));
                }
                DatumType::BoneIndices => vertices.bone_indices = Some(quads(&field.data)),
                DatumType::BoneWeights => {
                    let width = weight_width(field.datum_format).ok_or(
                        ModelError::UnsupportedVertexFormat {
                            datum_type: DatumType::BoneWeights.word(),
                            datum_format: field.datum_format.word(),
                        },
                    )?;
                    vertices.bone_weights = Some(weights(&field.data, width));
                    vertices.bone_weight_width = width as u8;
                }
            }
        }
        if vertices.bone_weights.is_some() && vertices.bone_indices.is_none() {
            return Err(ModelError::InvalidVertexFormat(
                "bone weights without bone indices",
            ));
        }
        Ok(vertices)
    }

    /// Re-encodes `vertices` into the existing fields, in place: same field order, same
    /// formats, so `decode` then `encode` leaves every field's bytes identical. The attribute
    /// set must match the fields exactly (a field with no attribute, or an attribute with no
    /// field, is `VertexMismatch`), and `vertices.bone_weight_width` must match the weight
    /// field's format.
    pub fn encode_vertices(&mut self, vertices: &MeshVertices) -> Result<(), ModelError> {
        let has = |kind: DatumType| {
            self.vertex_fields
                .iter()
                .any(|field| field.datum_type == kind)
        };
        if !has(DatumType::Position) {
            return Err(ModelError::VertexMismatch("no position field"));
        }
        for (present, kind, what) in [
            (vertices.normals.is_some(), DatumType::Normal, "normals"),
            (vertices.tangents.is_some(), DatumType::Tangent, "tangents"),
            (
                vertices.bitangents.is_some(),
                DatumType::Bitangent,
                "bitangents",
            ),
            (vertices.colors.is_some(), DatumType::Color, "colors"),
            (
                vertices.bone_indices.is_some(),
                DatumType::BoneIndices,
                "bone indices",
            ),
            (
                vertices.bone_weights.is_some(),
                DatumType::BoneWeights,
                "bone weights",
            ),
        ] {
            if present != has(kind) {
                return Err(ModelError::VertexMismatch(what));
            }
        }
        let uv_fields = self
            .vertex_fields
            .iter()
            .filter(|field| {
                matches!(
                    field.datum_type,
                    DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3
                )
            })
            .count();
        if vertices.uvs.len() != uv_fields {
            return Err(ModelError::VertexMismatch("uv maps"));
        }

        for field in &mut self.vertex_fields {
            check_pairing(field)?;
            let data = match field.datum_type {
                DatumType::Position => encode_triples(&vertices.positions),
                DatumType::Normal => encode_triples(
                    vertices
                        .normals
                        .as_deref()
                        .ok_or(ModelError::VertexMismatch("normals"))?,
                ),
                DatumType::Tangent => encode_triples(
                    vertices
                        .tangents
                        .as_deref()
                        .ok_or(ModelError::VertexMismatch("tangents"))?,
                ),
                DatumType::Bitangent => encode_triples(
                    vertices
                        .bitangents
                        .as_deref()
                        .ok_or(ModelError::VertexMismatch("bitangents"))?,
                ),
                DatumType::Color => encode_quads(
                    vertices
                        .colors
                        .as_deref()
                        .ok_or(ModelError::VertexMismatch("colors"))?,
                ),
                DatumType::Uv0 | DatumType::Uv1 | DatumType::Uv2 | DatumType::Uv3 => {
                    let index = field.datum_type as usize - DatumType::Uv0 as usize;
                    encode_doubles(
                        vertices
                            .uvs
                            .get(index)
                            .ok_or(ModelError::VertexMismatch("uv maps"))?,
                    )
                }
                DatumType::BoneIndices => encode_quads(
                    vertices
                        .bone_indices
                        .as_deref()
                        .ok_or(ModelError::VertexMismatch("bone indices"))?,
                ),
                DatumType::BoneWeights => {
                    let width = weight_width(field.datum_format).ok_or(
                        ModelError::UnsupportedVertexFormat {
                            datum_type: DatumType::BoneWeights.word(),
                            datum_format: field.datum_format.word(),
                        },
                    )?;
                    if vertices.bone_weight_width != width as u8 {
                        return Err(ModelError::VertexMismatch("bone weight width"));
                    }
                    let bone_weights = vertices
                        .bone_weights
                        .as_deref()
                        .ok_or(ModelError::VertexMismatch("bone weights"))?;
                    for quad in bone_weights {
                        if quad[width..].iter().any(|weight| *weight != 0.0) {
                            return Err(ModelError::VertexMismatch(
                                "bone weight beyond the stored width",
                            ));
                        }
                    }
                    encode_weights(bone_weights, width)
                }
            };
            if data.len() != field.data.len() {
                return Err(ModelError::VertexMismatch("vertex count"));
            }
            field.data = data;
        }
        Ok(())
    }
}

/// Validates `indices` and `lod_ranges`; shared by `faces` and `level`.
fn check_stream(stream: &FaceStream) -> Result<(), ModelError> {
    if !stream.indices.len().is_multiple_of(3) {
        return Err(ModelError::InvalidFaceStream(
            "index count not a multiple of 3",
        ));
    }
    for (start, end) in &stream.lod_ranges {
        if *end as usize > stream.indices.len() || end < start || (end - start) % 3 != 0 {
            return Err(ModelError::InvalidFaceStream("lod range"));
        }
    }
    Ok(())
}

impl FaceStream {
    /// Every face of the stream, all LOD levels included, as index triples.
    pub fn faces(&self) -> Result<Vec<[u16; 3]>, ModelError> {
        check_stream(self)?;
        Ok(self.indices.as_chunks::<3>().0.to_vec())
    }

    /// The faces of one LOD level (level 0 is the full-detail mesh). Without a LOD table the
    /// whole stream is level 0 and there is no level 1.
    pub fn level(&self, level: usize) -> Result<Vec<[u16; 3]>, ModelError> {
        check_stream(self)?;
        if level >= self.lod_ranges.len().max(1) {
            return Err(ModelError::BadReference {
                what: "lod level",
                offset: level as i64,
            });
        }
        let (start, end) = if self.lod_ranges.is_empty() {
            (0, self.indices.len())
        } else {
            (
                self.lod_ranges[level].0 as usize,
                self.lod_ranges[level].1 as usize,
            )
        };
        Ok(self.indices[start..end].as_chunks::<3>().0.to_vec())
    }

    /// From level-0 faces only: a stream with no LOD table.
    pub fn from_faces(faces: &[[u16; 3]]) -> FaceStream {
        let mut indices = Vec::with_capacity(3 * faces.len());
        for face in faces {
            indices.extend_from_slice(face);
        }
        FaceStream {
            indices,
            lod_ranges: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::PreFoxModel;
    use crate::format::fixtures::*;

    fn round5(values: &[f32]) -> Vec<f32> {
        values
            .iter()
            .map(|value| (value * 100000.0).round() / 100000.0)
            .collect()
    }

    #[test]
    fn decode_encode_is_byte_identical() {
        for bytes in ALL {
            let model = PreFoxModel::read(bytes).unwrap();
            for geometry in &model.geometries {
                let vertices = geometry.decode_vertices().unwrap();
                let mut again = geometry.clone();
                again.encode_vertices(&vertices).unwrap();
                assert_eq!(again, *geometry);
                assert_eq!(vertices.to_fields().unwrap(), geometry.vertex_fields);
            }
        }
    }

    #[test]
    fn decoded_content() {
        let cap = PreFoxModel::read(CAP).unwrap();
        let vertices = cap.geometries[0].decode_vertices().unwrap();
        assert_eq!(vertices.len(), 56);
        assert_eq!(round5(&vertices.positions[0]), [0.34969, 1.30588, -0.02537]);
        assert_eq!(
            round5(&vertices.normals.as_ref().unwrap()[0]),
            [0.62709, -0.54091, -0.56051]
        );
        assert!(vertices.tangents.is_some());
        assert!(vertices.bitangents.is_some());
        assert!(vertices.colors.is_none());
        assert_eq!(vertices.uvs.len(), 2);
        assert_eq!(vertices.bone_weight_width, 4);
        assert_eq!(vertices.bone_indices.as_ref().unwrap()[0], [1, 2, 3, 0]);
        assert_eq!(
            round5(&vertices.bone_weights.as_ref().unwrap()[0]),
            [0.69, 0.15, 0.16, 0.0]
        );
        for quad in vertices.bone_weights.as_ref().unwrap() {
            assert!((quad.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        }

        let hair_d = PreFoxModel::read(HAIR_D).unwrap();
        let vertices = hair_d.geometries[0].decode_vertices().unwrap();
        assert_eq!(vertices.bone_weight_width, 2);
        assert_eq!(
            round5(&vertices.bone_weights.as_ref().unwrap()[0]),
            [0.98625, 0.01375, 0.0, 0.0]
        );
        assert_eq!(vertices.bone_indices.as_ref().unwrap()[0], [0, 1, 0, 0]);

        let taping = PreFoxModel::read(TAPING).unwrap();
        let vertices = taping.geometries[0].decode_vertices().unwrap();
        assert_eq!(vertices.bone_weight_width, 3);
        assert_eq!(
            round5(&vertices.bone_weights.as_ref().unwrap()[0]),
            [0.3, 0.5, 0.2, 0.0]
        );
        assert_eq!(vertices.bone_indices.as_ref().unwrap()[0], [1, 3, 4, 0]);

        let card = PreFoxModel::read(CARD).unwrap();
        let vertices = card.geometries[0].decode_vertices().unwrap();
        assert!(vertices.bone_indices.is_some());
        assert!(vertices.bone_weights.is_none());
        assert!(vertices.bitangents.is_some());

        let hair = PreFoxModel::read(HAIR_HIGH).unwrap();
        let vertices = hair.geometries[0].decode_vertices().unwrap();
        assert_eq!(vertices.colors.as_ref().unwrap()[0], [255, 255, 255, 255]);
        assert_eq!(vertices.bone_indices.as_ref().unwrap()[0], [1, 2, 3, 10]);

        let shadow = PreFoxModel::read(SHADOW).unwrap();
        let vertices = shadow.geometries[0].decode_vertices().unwrap();
        assert!(vertices.normals.is_none());
        assert!(vertices.uvs.is_empty());
        assert_eq!(vertices.bone_weight_width, 2);

        let cardhead = PreFoxModel::read(CARDHEAD).unwrap();
        let vertices = cardhead.geometries[0].decode_vertices().unwrap();
        assert_eq!(
            vertices.bone_weights.as_ref().unwrap()[0],
            [1.0, 0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn faces_and_levels() {
        let cap = PreFoxModel::read(CAP).unwrap();
        let stream = &cap.geometries[0].faces;
        assert_eq!(stream.faces().unwrap()[0], [0, 4, 1]);
        assert_eq!(stream.faces().unwrap().len(), 78);
        assert_eq!(stream.level(0).unwrap().len(), 78);
        assert!(matches!(
            stream.level(1),
            Err(ModelError::BadReference {
                what: "lod level",
                ..
            })
        ));
        assert_eq!(FaceStream::from_faces(&stream.faces().unwrap()), *stream);

        let collar = PreFoxModel::read(COLLAR).unwrap();
        let stream = &collar.geometries[0].faces;
        assert_eq!(stream.faces().unwrap().len(), 12832);
        assert_eq!(stream.level(0).unwrap().len(), 4080);
        assert_eq!(stream.level(5).unwrap().len(), 662);
        assert!(matches!(
            stream.level(6),
            Err(ModelError::BadReference {
                what: "lod level",
                ..
            })
        ));
    }

    #[test]
    fn malformed_fields_error() {
        let cap = PreFoxModel::read(CAP).unwrap();

        let mut geometry = cap.geometries[0].clone();
        geometry.vertex_fields[0].datum_format = DatumFormat::DoubleFloat32;
        assert_eq!(
            geometry.decode_vertices(),
            Err(ModelError::UnsupportedVertexFormat {
                datum_type: 2,
                datum_format: 4
            })
        );

        let mut geometry = cap.geometries[0].clone();
        let normal = geometry
            .vertex_fields
            .iter_mut()
            .find(|field| field.datum_type == DatumType::Normal)
            .unwrap();
        normal.data.truncate(normal.data.len() / 2);
        assert!(matches!(
            geometry.decode_vertices(),
            Err(ModelError::InvalidVertexFormat(_))
        ));

        let vertices = cap.geometries[0].decode_vertices().unwrap();
        let mut geometry = cap.geometries[0].clone();
        geometry
            .vertex_fields
            .retain(|field| field.datum_type != DatumType::Normal);
        assert!(matches!(
            geometry.encode_vertices(&vertices),
            Err(ModelError::VertexMismatch(_))
        ));

        let mut vertices = cap.geometries[0].decode_vertices().unwrap();
        vertices.uvs.clear();
        let mut geometry = cap.geometries[0].clone();
        assert!(matches!(
            geometry.encode_vertices(&vertices),
            Err(ModelError::VertexMismatch(_))
        ));

        let hair_d = PreFoxModel::read(HAIR_D).unwrap();
        let mut vertices = hair_d.geometries[0].decode_vertices().unwrap();
        vertices.bone_weights.as_mut().unwrap()[0][2] = 0.5;
        let mut geometry = hair_d.geometries[0].clone();
        assert!(matches!(
            geometry.encode_vertices(&vertices),
            Err(ModelError::VertexMismatch(_))
        ));

        let stream = FaceStream {
            indices: vec![0, 1],
            lod_ranges: vec![],
        };
        assert!(matches!(
            stream.faces(),
            Err(ModelError::InvalidFaceStream(_))
        ));

        let collar = PreFoxModel::read(COLLAR).unwrap();
        let mut stream = collar.geometries[0].faces.clone();
        stream.lod_ranges[5].1 += 1;
        assert!(matches!(
            stream.level(5),
            Err(ModelError::InvalidFaceStream(_))
        ));
    }
}
