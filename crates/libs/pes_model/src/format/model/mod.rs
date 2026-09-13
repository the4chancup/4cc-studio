//! `PreFoxModel`: the typed layer over [`ModelContainer`]. Reading resolves
//! every cross-section pointer into an index; writing lays a fresh file out
//! the add-on's way (see `write.rs`).

use crate::format::datum::{DatumFormat, DatumType};
use crate::format::records::RecordArray;
use crate::format::{ModelContainer, ModelError, SectionKind};

#[cfg(test)]
mod tests;

/// A `.model` with every pointer resolved: what the file says, in the file's own terms (raw
/// vertex bytes, u16 face indices), nothing interpreted. `read(write(m)) == m`; the bytes differ
/// from Konami's (see the plan).
#[derive(Debug, Clone, PartialEq)]
pub struct PreFoxModel {
    /// The version word the file declared; `write` always emits 19, see the plan.
    pub version: u16,
    /// Header flags word (0; 4 in two face-montage models, meaning unknown).
    pub flags: u32,
    /// In file order; bone-group entries and vertex bone indices index this list.
    pub bones: Vec<Bone>,
    /// Section 0 entries after the matrices, in order: each a list of indices into `bones`.
    pub bone_groups: Vec<Vec<u16>>,
    /// Section 6's names, in order.
    pub material_names: Vec<String>,
    /// Section 2's strings, in order.
    pub annotation_strings: Vec<String>,
    /// Section 3: the 28-byte records annotations point at (`0 0 0 2 2 2 0` in every known file).
    pub annotation_records: Vec<[u32; 7]>,
    /// Section 1, in order.
    pub geometries: Vec<Geometry>,
    /// Section 4, in order.
    pub meshes: Vec<Mesh>,
    /// The model's bounds, section 7's first entry.
    pub bounds: BoundingBox,
    /// Section 7's second entry.
    pub lod: LodRecord,
}

/// A named bone and its inverse bind matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    /// From section 5.
    pub name: String,
    /// The 3x4 inverse bind matrix, twelve floats as stored.
    pub matrix: [f32; 12],
}

/// An axis-aligned box, `min` then `max` as the file stores them.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    /// The low corner (the fourth word is 0 in every known file).
    pub min: [f32; 4],
    /// The high corner (the fourth word is 0 in every known file).
    pub max: [f32; 4],
}

impl BoundingBox {
    /// The box around `positions`; an empty list gives the zero box.
    pub fn of(positions: &[[f32; 3]]) -> BoundingBox {
        let Some(first) = positions.first() else {
            return BoundingBox {
                min: [0.0; 4],
                max: [0.0; 4],
            };
        };
        let mut min = [first[0], first[1], first[2], 0.0];
        let mut max = min;
        for position in &positions[1..] {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }
        BoundingBox { min, max }
    }
}

/// Section 7's second entry: the LOD level count and three parameters (`0.0625, 4.0` and `0.3`
/// with LODs, `0.0` without, in every known file).
#[derive(Debug, Clone, PartialEq)]
pub struct LodRecord {
    /// How many levels of detail the face streams partition into.
    pub level_count: u32,
    /// The record's three floats.
    pub parameters: [f32; 3],
}

impl LodRecord {
    /// The record Konami writes for a model with `level_count` LOD levels
    /// (`0` for none).
    pub fn for_levels(level_count: u32) -> LodRecord {
        LodRecord {
            level_count,
            parameters: [0.0625, 4.0, if level_count > 0 { 0.3 } else { 0.0 }],
        }
    }
}

/// One mesh's geometry: its vertex fields, face stream and extras.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// In descriptor order.
    pub vertex_fields: Vec<VertexField>,
    /// The face index stream and LOD ranges.
    pub faces: FaceStream,
    /// The geometry's bounds, the extras array's first entry.
    pub bounds: BoundingBox,
    /// The third extras entry; `None` in the version-17 layout, which has two.
    pub order: Option<u32>,
}

/// One vertex attribute stream: raw bytes, `count` vertices of
/// `datum_format.size()` each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexField {
    /// What the field holds.
    pub datum_type: DatumType,
    /// How each vertex's value is stored.
    pub datum_format: DatumFormat,
    /// `count * datum_format.size()` bytes, as stored.
    pub data: Vec<u8>,
}

impl VertexField {
    /// How many vertices the field covers.
    pub fn count(&self) -> usize {
        self.data.len() / self.datum_format.size()
    }
}

/// The face index stream and, when the file has LODs, the `(start, end)` face-vertex ranges of
/// each level, level 0 first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceStream {
    /// `u16` indices, three per face.
    pub indices: Vec<u16>,
    /// `(start, end)` ranges partitioning `indices` in order, level 0 first.
    pub lod_ranges: Vec<(u32, u32)>,
}

/// One mesh record with its cross-section pointers resolved to indices.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Index into `geometries`.
    pub geometry: usize,
    /// Index into `material_names`.
    pub material: usize,
    /// Index into `bone_groups`; `None` when the mesh's bone-group array is empty.
    pub bone_group: Option<usize>,
    /// The mesh's annotations, in file order.
    pub annotations: Vec<Annotation>,
    /// `None` when the record has no editor-data pointer (version-17 layout) or a zero one
    /// (add-on files); `Some(vec![])` for Konami's present-but-empty array.
    pub editor_data: Option<Vec<EditorItem>>,
}

/// A mesh annotation: a section-2 string, an optional section-3 record and
/// two words (`unknown`, `kind`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// Index into `annotation_strings`.
    pub string: usize,
    /// Index into `annotation_records`; `None` for a zero pointer (add-on files).
    pub record: Option<usize>,
    /// 7 in every Konami file (8 in third-party props).
    pub unknown: u32,
    /// The annotation kind (see [`Annotation::MESH_NAME`] and
    /// [`Annotation::EXTENSION_HEADER`] for the add-on's; Konami uses 1, 2,
    /// 7 and 10).
    pub kind: u32,
}

impl Annotation {
    /// The add-on's mesh-name annotation kind.
    pub const MESH_NAME: u32 = 128;
    /// The add-on's extension-header annotation kind.
    pub const EXTENSION_HEADER: u32 = 129;
}

/// One editor-data item on a mesh. The layout comes from documented notes,
/// not from a file: none of the 2610 measured `.model` files carries a
/// non-empty editor-data array, so the decode is specified but unverified
/// against a real sample.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorItem {
    /// The item kind; 1 means the value is a string.
    pub kind: u32,
    /// The item's second word.
    pub unknown: u32,
    /// The item's value.
    pub value: EditorValue,
}

/// An editor-data item's value: a NUL-terminated string padded to 4 when
/// `kind == 1`, a `u32` otherwise.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorValue {
    /// A string value (`kind == 1`).
    Text(String),
    /// A word value (any other `kind`).
    Word(u32),
}

/// `base + offset`, `Truncated` on overflow.
fn offset_sum(base: usize, offset: usize) -> Result<usize, ModelError> {
    base.checked_add(offset).ok_or(ModelError::Truncated)
}

fn slice_at(bytes: &[u8], at: usize, len: usize) -> Result<&[u8], ModelError> {
    bytes
        .get(at..offset_sum(at, len)?)
        .ok_or(ModelError::Truncated)
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, ModelError> {
    Ok(u32::from_le_bytes(
        slice_at(bytes, at, 4)?
            .try_into()
            .map_err(|_| ModelError::Truncated)?,
    ))
}

fn i32_at(bytes: &[u8], at: usize) -> Result<i32, ModelError> {
    Ok(i32::from_le_bytes(
        slice_at(bytes, at, 4)?
            .try_into()
            .map_err(|_| ModelError::Truncated)?,
    ))
}

fn floats_at<const N: usize>(bytes: &[u8], at: usize) -> Result<[f32; N], ModelError> {
    let mut floats = [0.0f32; N];
    let (words, _) = slice_at(bytes, at, 4 * N)?.as_chunks::<4>();
    for (value, word) in floats.iter_mut().zip(words) {
        *value = f32::from_le_bytes(*word);
    }
    Ok(floats)
}

/// The first word of an entry header, zero-padded when shorter than 4.
fn header_word(header: &[u8]) -> u32 {
    let mut word = [0u8; 4];
    let len = header.len().min(4);
    word[..len].copy_from_slice(&header[..len]);
    u32::from_le_bytes(word)
}

/// A NUL-terminated string at `at` in `bytes`.
fn string_at(bytes: &[u8], at: usize) -> Result<String, ModelError> {
    let rest = bytes.get(at..).ok_or(ModelError::Truncated)?;
    let end = rest
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(ModelError::Truncated)?;
    String::from_utf8(rest[..end].to_vec()).map_err(|_| ModelError::BadReference {
        what: "string",
        offset: at as i64,
    })
}

/// The `u32` offset a pointer record holds.
fn pointer_of(record: &[u8]) -> Result<usize, ModelError> {
    let offset = u32::from_le_bytes(
        record
            .get(..4)
            .ok_or(ModelError::Truncated)?
            .try_into()
            .map_err(|_| ModelError::Truncated)?,
    );
    usize::try_from(offset).map_err(|_| ModelError::Truncated)
}

/// The bounds blob (32 bytes) at `at`: `min[4]` then `max[4]`.
fn bounds_at(bytes: &[u8], at: usize) -> Result<BoundingBox, ModelError> {
    Ok(BoundingBox {
        min: floats_at(bytes, at)?,
        max: floats_at(bytes, offset_sum(at, 16)?)?,
    })
}

/// Which section of `container` contains file address `address`, and the
/// address's offset inside that section.
fn locate(container: &ModelContainer, address: i64) -> Option<(SectionKind, usize)> {
    container.sections.iter().find_map(|section| {
        let start = container.section_offset(section.kind) as i64;
        let relative = address - start;
        if relative >= 0 && (relative as usize) < section.bytes.len() {
            Some((section.kind, relative as usize))
        } else {
            None
        }
    })
}

/// The index of the record of `array` sitting at `relative` in its
/// section.
fn record_at(array: &RecordArray, relative: usize) -> Option<usize> {
    array
        .records
        .iter()
        .position(|record| record.offset == relative)
}

/// Resolves a mesh record's signed offset (relative to the Meshes
/// section's file offset) to a record index in `array`, which must be the
/// array of `expected`'s section.
fn resolve(
    container: &ModelContainer,
    array: &RecordArray,
    expected: SectionKind,
    signed: i32,
    what: &'static str,
) -> Result<usize, ModelError> {
    let address = i64::from(signed) + container.section_offset(SectionKind::Meshes) as i64;
    let bad = || ModelError::BadReference {
        what,
        offset: address,
    };
    let (kind, relative) = locate(container, address).ok_or_else(bad)?;
    if kind != expected {
        return Err(bad());
    }
    record_at(array, relative).ok_or_else(bad)
}

fn read_u16s(bytes: &[u8], at: usize, count: usize) -> Result<Vec<u16>, ModelError> {
    let len = count.checked_mul(2).ok_or(ModelError::Truncated)?;
    let (words, _) = slice_at(bytes, at, len)?.as_chunks::<2>();
    Ok(words.iter().map(|word| u16::from_le_bytes(*word)).collect())
}

/// One section-0 entry past entry 0: a bone group.
fn read_bone_group(bytes: &[u8], entry: usize, bones: usize) -> Result<Vec<u16>, ModelError> {
    let array = RecordArray::read(bytes, entry)?;
    if array.header != [2, 0, 0, 0] {
        return Err(ModelError::UnexpectedConstant {
            what: "bone data entry header",
            value: header_word(&array.header),
        });
    }
    if array.records.len() != 1 {
        return Err(ModelError::UnexpectedConstant {
            what: "bone group record count",
            value: array.records.len() as u32,
        });
    }
    let record = &array.records[0];
    let (offset, kind, count) = (
        u32_at(&record.bytes, 0)?,
        u32_at(&record.bytes, 4)?,
        u32_at(&record.bytes, 8)?,
    );
    if kind != 1 {
        return Err(ModelError::UnexpectedConstant {
            what: "bone group record type",
            value: kind,
        });
    }
    let group = read_u16s(bytes, offset_sum(entry, offset as usize)?, count as usize)?;
    for bone in &group {
        if *bone as usize >= bones {
            return Err(ModelError::BadReference {
                what: "bone",
                offset: i64::from(*bone),
            });
        }
    }
    Ok(group)
}

/// A geometry record's vertex set, face stream and extras.
fn read_geometry(bytes: &[u8], record: &[u8]) -> Result<Geometry, ModelError> {
    let one = u32_at(record, 0)?;
    if one != 1 {
        return Err(ModelError::UnexpectedConstant {
            what: "geometry record word 0",
            value: one,
        });
    }
    let vertex_set_offset = u32_at(record, 4)? as usize;
    let face_descriptor_offset = u32_at(record, 8)? as usize;
    let zero = u32_at(record, 12)?;
    if zero != 0 {
        return Err(ModelError::UnexpectedConstant {
            what: "geometry record word 3",
            value: zero,
        });
    }
    let extras_offset = u32_at(record, 16)? as usize;

    // The vertex set is a record array of one u32, a section-relative
    // offset of the field-descriptor array.
    let vertex_set = RecordArray::read(bytes, vertex_set_offset)?;
    if vertex_set.records.len() != 1 {
        return Err(ModelError::UnexpectedConstant {
            what: "vertex set count",
            value: vertex_set.records.len() as u32,
        });
    }
    let descriptors_offset = pointer_of(&vertex_set.records[0].bytes)?;
    let descriptors = RecordArray::read(bytes, descriptors_offset)?;
    if descriptors.header.len() != 8 {
        return Err(ModelError::UnexpectedConstant {
            what: "vertex field header size",
            value: descriptors.header.len() as u32,
        });
    }
    let cloth = u32::from_le_bytes(
        descriptors.header[..4]
            .try_into()
            .map_err(|_| ModelError::Truncated)?,
    );
    if cloth != 0 {
        return Err(ModelError::Unsupported("cloth"));
    }
    let zero = u32::from_le_bytes(
        descriptors.header[4..8]
            .try_into()
            .map_err(|_| ModelError::Truncated)?,
    );
    if zero != 0 {
        return Err(ModelError::UnexpectedConstant {
            what: "vertex field header word",
            value: zero,
        });
    }
    let mut vertex_fields = Vec::with_capacity(descriptors.records.len());
    for descriptor in &descriptors.records {
        let data_offset = u32_at(&descriptor.bytes, 0)? as usize;
        let datum_type = u32_at(&descriptor.bytes, 4)?;
        let datum_format = u32_at(&descriptor.bytes, 8)?;
        let count = u32_at(&descriptor.bytes, 12)? as usize;
        let zero = u32_at(&descriptor.bytes, 16)?;
        if zero != 0 {
            return Err(ModelError::UnexpectedConstant {
                what: "vertex field descriptor word",
                value: zero,
            });
        }
        let (Some(datum_type), Some(datum_format)) = (
            DatumType::from_word(datum_type),
            DatumFormat::from_word(datum_format),
        ) else {
            return Err(ModelError::UnsupportedVertexFormat {
                datum_type,
                datum_format,
            });
        };
        let len = count
            .checked_mul(datum_format.size())
            .ok_or(ModelError::Truncated)?;
        let data = slice_at(bytes, data_offset, len)?.to_vec();
        vertex_fields.push(VertexField {
            datum_type,
            datum_format,
            data,
        });
    }

    // The face descriptor is one 24-byte record:
    // `start, 1, format, count, lod_levels, lod_table_offset`.
    let face_descriptors = RecordArray::read(bytes, face_descriptor_offset)?;
    if face_descriptors.records.len() != 1 {
        return Err(ModelError::UnexpectedConstant {
            what: "face descriptor count",
            value: face_descriptors.records.len() as u32,
        });
    }
    let face = &face_descriptors.records[0].bytes;
    let start = u32_at(face, 0)? as usize;
    let one = u32_at(face, 4)?;
    if one != 1 {
        return Err(ModelError::UnexpectedConstant {
            what: "face descriptor word 1",
            value: one,
        });
    }
    let format = u32_at(face, 8)?;
    if format != DatumFormat::Uint16.word() {
        return Err(ModelError::UnexpectedConstant {
            what: "face index format",
            value: format,
        });
    }
    let count = u32_at(face, 12)? as usize;
    let lod_levels = u32_at(face, 16)? as usize;
    let lod_table_offset = u32_at(face, 20)? as usize;
    let indices = read_u16s(bytes, start, count)?;
    // Slice the whole table before allocating: `8 * lod_levels` can
    // overflow or lie far past the section on a hostile count.
    let table_len = lod_levels.checked_mul(8).ok_or(ModelError::Truncated)?;
    let table = slice_at(bytes, lod_table_offset, table_len)?;
    let mut lod_ranges = Vec::with_capacity(lod_levels);
    for entry in table.as_chunks::<8>().0 {
        lod_ranges.push((
            u32::from_le_bytes(entry[..4].try_into().map_err(|_| ModelError::Truncated)?),
            u32::from_le_bytes(entry[4..].try_into().map_err(|_| ModelError::Truncated)?),
        ));
    }

    // The extras array's records are u32 offsets relative to the array's
    // own start: [0] bounds (32 bytes), [1] the material-combination flags
    // `1,0,0,0`, [2] the order word when present.
    let extras = RecordArray::read(bytes, extras_offset)?;
    if !(2..=3).contains(&extras.records.len()) {
        return Err(ModelError::UnexpectedConstant {
            what: "geometry extras count",
            value: extras.records.len() as u32,
        });
    }
    let bounds_offset = offset_sum(extras_offset, pointer_of(&extras.records[0].bytes)?)?;
    let flags_offset = offset_sum(extras_offset, pointer_of(&extras.records[1].bytes)?)?;
    let flags = slice_at(bytes, flags_offset, 16)?;
    let flags = [
        u32::from_le_bytes(flags[0..4].try_into().map_err(|_| ModelError::Truncated)?),
        u32::from_le_bytes(flags[4..8].try_into().map_err(|_| ModelError::Truncated)?),
        u32::from_le_bytes(flags[8..12].try_into().map_err(|_| ModelError::Truncated)?),
        u32::from_le_bytes(
            flags[12..16]
                .try_into()
                .map_err(|_| ModelError::Truncated)?,
        ),
    ];
    if flags != [1, 0, 0, 0] {
        return Err(ModelError::Unsupported("material combinations"));
    }
    let order = if extras.records.len() == 3 {
        let order_offset = offset_sum(extras_offset, pointer_of(&extras.records[2].bytes)?)?;
        Some(u32_at(bytes, order_offset)?)
    } else {
        None
    };

    Ok(Geometry {
        vertex_fields,
        faces: FaceStream {
            indices,
            lod_ranges,
        },
        bounds: bounds_at(bytes, bounds_offset)?,
        order,
    })
}

impl PreFoxModel {
    /// Parses a `.model`, WESYS-wrapped or not, into the typed layer.
    pub fn read(bytes: &[u8]) -> Result<Self, ModelError> {
        Self::from_container(&ModelContainer::read(bytes)?)
    }

    /// Resolves every cross-section pointer of `container` into indices.
    pub fn from_container(container: &ModelContainer) -> Result<Self, ModelError> {
        let bone_data = container.section(SectionKind::BoneData);
        let entries = RecordArray::read(bone_data, 0)?;
        if entries.records.is_empty() {
            return Err(ModelError::UnexpectedConstant {
                what: "bone data entry count",
                value: 0,
            });
        }
        let mut entry_offsets = Vec::with_capacity(entries.records.len());
        for record in &entries.records {
            entry_offsets.push(pointer_of(&record.bytes)?);
        }

        // Entry 0: one Float32Matrix34 record per bone, in bone-name order.
        let matrices = RecordArray::read(bone_data, entry_offsets[0])?;
        if matrices.header != [2, 0, 0, 0] {
            return Err(ModelError::UnexpectedConstant {
                what: "bone data entry header",
                value: header_word(&matrices.header),
            });
        }
        let mut matrices_data = Vec::with_capacity(matrices.records.len());
        for record in &matrices.records {
            let offset = u32_at(&record.bytes, 0)?;
            let kind = u32_at(&record.bytes, 4)?;
            let count = u32_at(&record.bytes, 8)?;
            if kind != DatumFormat::Float32Matrix34.word() || count != 1 {
                return Err(ModelError::UnexpectedConstant {
                    what: "bone matrix record",
                    value: kind,
                });
            }
            matrices_data.push(floats_at::<12>(
                bone_data,
                offset_sum(entry_offsets[0], offset as usize)?,
            )?);
        }

        let bone_names = RecordArray::read(container.section(SectionKind::BoneNames), 0)?;
        let name_bytes = container.section(SectionKind::BoneNames);
        let mut names = Vec::with_capacity(bone_names.records.len());
        for record in &bone_names.records {
            names.push(string_at(name_bytes, pointer_of(&record.bytes)?)?);
        }
        if names.len() != matrices_data.len() {
            return Err(ModelError::BoneTableMismatch {
                names: names.len(),
                matrices: matrices_data.len(),
            });
        }
        let bones: Vec<Bone> = names
            .into_iter()
            .zip(matrices_data)
            .map(|(name, matrix)| Bone { name, matrix })
            .collect();

        let mut bone_groups = Vec::with_capacity(entry_offsets.len() - 1);
        for entry in &entry_offsets[1..] {
            bone_groups.push(read_bone_group(bone_data, *entry, bones.len())?);
        }

        let material_section = container.section(SectionKind::MaterialNames);
        let material_array = RecordArray::read(material_section, 0)?;
        let mut material_names = Vec::with_capacity(material_array.records.len());
        for record in &material_array.records {
            material_names.push(string_at(material_section, pointer_of(&record.bytes)?)?);
        }

        let string_section = container.section(SectionKind::AnnotationStrings);
        let string_array = RecordArray::read(string_section, 0)?;
        let mut annotation_strings = Vec::with_capacity(string_array.records.len());
        for record in &string_array.records {
            let offset = u32_at(&record.bytes, 0)? as usize;
            let zero = u32_at(&record.bytes, 4)?;
            if zero != 0 {
                return Err(ModelError::UnexpectedConstant {
                    what: "annotation string record word",
                    value: zero,
                });
            }
            annotation_strings.push(string_at(string_section, offset)?);
        }

        let record_section = container.section(SectionKind::AnnotationRecords);
        let record_array = RecordArray::read(record_section, 0)?;
        if !record_array.records.is_empty() && record_array.record_size != 28 {
            return Err(ModelError::UnexpectedConstant {
                what: "annotation record size",
                value: record_array.record_size,
            });
        }
        let mut annotation_records = Vec::with_capacity(record_array.records.len());
        for record in &record_array.records {
            let mut words = [0u32; 7];
            for (index, word) in words.iter_mut().enumerate() {
                *word = u32_at(&record.bytes, 4 * index)?;
            }
            annotation_records.push(words);
        }

        let geometry_section = container.section(SectionKind::Geometry);
        let geometry_array = RecordArray::read(geometry_section, 0)?;
        if !geometry_array.records.is_empty() && geometry_array.record_size != 20 {
            return Err(ModelError::UnexpectedConstant {
                what: "geometry record size",
                value: geometry_array.record_size,
            });
        }
        let mut geometries = Vec::with_capacity(geometry_array.records.len());
        for record in &geometry_array.records {
            geometries.push(read_geometry(geometry_section, &record.bytes)?);
        }

        // Sections 8, 9 and 10 are empty in every known file.
        for (kind, what) in [
            (SectionKind::Cloth, "cloth"),
            (SectionKind::MaterialCombinations, "material combinations"),
            (SectionKind::Locators, "locators"),
        ] {
            let array = RecordArray::read(container.section(kind), 0)?;
            if !array.records.is_empty() {
                return Err(ModelError::Unsupported(what));
            }
        }

        let bounds_section = container.section(SectionKind::ModelBounds);
        let bounds_array = RecordArray::read(bounds_section, 0)?;
        if bounds_array.records.len() != 2 {
            return Err(ModelError::UnexpectedConstant {
                what: "section 7 entry count",
                value: bounds_array.records.len() as u32,
            });
        }
        let bounds = bounds_at(bounds_section, pointer_of(&bounds_array.records[0].bytes)?)?;
        let lod_offset = pointer_of(&bounds_array.records[1].bytes)?;
        let lod = LodRecord {
            level_count: u32_at(bounds_section, lod_offset)?,
            parameters: floats_at(bounds_section, offset_sum(lod_offset, 4)?)?,
        };

        let mesh_section = container.section(SectionKind::Meshes);
        let mesh_array = RecordArray::read(mesh_section, 0)?;
        if !mesh_array.records.is_empty()
            && mesh_array.record_size != 20
            && mesh_array.record_size != 24
        {
            return Err(ModelError::UnexpectedConstant {
                what: "mesh record size",
                value: mesh_array.record_size,
            });
        }
        let tables = Tables {
            geometries: &geometry_array,
            strings: &string_array,
            records: &record_array,
            materials: &material_array,
        };
        let mut meshes = Vec::with_capacity(mesh_array.records.len());
        for record in &mesh_array.records {
            meshes.push(read_mesh(
                container,
                mesh_section,
                &record.bytes,
                &entry_offsets,
                &tables,
            )?);
        }

        Ok(PreFoxModel {
            version: container.version,
            flags: container.flags,
            bones,
            bone_groups,
            material_names,
            annotation_strings,
            annotation_records,
            geometries,
            meshes,
            bounds,
            lod,
        })
    }
}

/// The record arrays a mesh record's pointers resolve through.
struct Tables<'a> {
    /// Section 1's geometry records.
    geometries: &'a RecordArray,
    /// Section 2's string records.
    strings: &'a RecordArray,
    /// Section 3's annotation records.
    records: &'a RecordArray,
    /// Section 6's material-name records.
    materials: &'a RecordArray,
}

/// One mesh record: `i32 geometry, u32 bone_group_array, u32 annotation_array,
/// i32 material, i32 locator` plus a `u32 editor_data` word in the 24-byte
/// layout.
fn read_mesh(
    container: &ModelContainer,
    section: &[u8],
    record: &[u8],
    entry_offsets: &[usize],
    tables: &Tables,
) -> Result<Mesh, ModelError> {
    let geometry = resolve(
        container,
        tables.geometries,
        SectionKind::Geometry,
        i32_at(record, 0)?,
        "geometry",
    )?;
    let material = resolve(
        container,
        tables.materials,
        SectionKind::MaterialNames,
        i32_at(record, 12)?,
        "material",
    )?;
    let locator = i32_at(record, 16)?;
    if locator != 0 {
        return Err(ModelError::Unsupported("locators"));
    }

    // The bone-group array holds 0 or 1 records, each an i32 signed offset
    // (from the Meshes section's file offset) to a section-0 *entry*.
    let bone_group_offset = u32_at(record, 4)? as usize;
    let bone_group = if bone_group_offset == 0 {
        None
    } else {
        let array = RecordArray::read(section, bone_group_offset)?;
        if array.records.len() > 1 {
            return Err(ModelError::UnexpectedConstant {
                what: "bone group record count",
                value: array.records.len() as u32,
            });
        }
        match array.records.first() {
            None => None,
            Some(record) => {
                let signed = i32_at(&record.bytes, 0)?;
                let address =
                    i64::from(signed) + container.section_offset(SectionKind::Meshes) as i64;
                let bone_data_start = container.section_offset(SectionKind::BoneData) as i64;
                let relative = address - bone_data_start;
                let entry = entry_offsets
                    .iter()
                    .position(|offset| *offset as i64 == relative)
                    .ok_or(ModelError::BadReference {
                        what: "bone group",
                        offset: address,
                    })?;
                if entry == 0 {
                    return Err(ModelError::BadReference {
                        what: "bone group",
                        offset: address,
                    });
                }
                Some(entry - 1)
            }
        }
    };

    let annotation_offset = u32_at(record, 8)? as usize;
    let mut annotations = Vec::new();
    if annotation_offset != 0 {
        let array = RecordArray::read(section, annotation_offset)?;
        for record in &array.records {
            let string = resolve(
                container,
                tables.strings,
                SectionKind::AnnotationStrings,
                i32_at(&record.bytes, 0)?,
                "annotation string",
            )?;
            let record_pointer = i32_at(&record.bytes, 4)?;
            let annotation_record = if record_pointer == 0 {
                None
            } else {
                Some(resolve(
                    container,
                    tables.records,
                    SectionKind::AnnotationRecords,
                    record_pointer,
                    "annotation record",
                )?)
            };
            annotations.push(Annotation {
                string,
                record: annotation_record,
                unknown: u32_at(&record.bytes, 8)?,
                kind: u32_at(&record.bytes, 12)?,
            });
        }
    }

    let editor_data = if record.len() >= 24 {
        match u32_at(record, 20)? as usize {
            0 => None,
            offset => Some(read_editor_data(section, offset)?),
        }
    } else {
        None
    };

    Ok(Mesh {
        geometry,
        material,
        bone_group,
        annotations,
        editor_data,
    })
}

/// The editor-data array at `offset` in the Meshes section: `u32` records,
/// each an offset relative to the array's own start, pointing at an item
/// `u32 kind, u32 unknown, value` (a NUL-terminated string padded to 4 when
/// `kind == 1`, a `u32` otherwise).
fn read_editor_data(section: &[u8], offset: usize) -> Result<Vec<EditorItem>, ModelError> {
    let array = RecordArray::read(section, offset)?;
    let mut items = Vec::with_capacity(array.records.len());
    for record in &array.records {
        let item = offset_sum(offset, pointer_of(&record.bytes)?)?;
        let kind = u32_at(section, item)?;
        let unknown = u32_at(section, offset_sum(item, 4)?)?;
        let value = if kind == 1 {
            EditorValue::Text(string_at(section, offset_sum(item, 8)?)?)
        } else {
            EditorValue::Word(u32_at(section, offset_sum(item, 8)?)?)
        };
        items.push(EditorItem {
            kind,
            unknown,
            value,
        });
    }
    Ok(items)
}
