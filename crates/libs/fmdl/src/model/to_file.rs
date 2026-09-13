use std::collections::HashMap;

use super::{BoundingBox, MeshGroup, Model};
use crate::format::f16::f32_to_f16;
use crate::format::records::*;
use crate::format::{DatumFormat, DatumType, FmdlError, FmdlFile, MeshVertices};

impl Model {
    /// Rebuilds a fresh `FmdlFile`: positions in buffer 0 (stride 12), the
    /// other attributes interleaved in buffer 1 in the order normal,
    /// tangent, color, bone weights, bone indices, uv maps (a uv map
    /// identical to an earlier one shares its offset), faces in buffer 2;
    /// strings de-duplicated with index 0 the empty string; a bounding box
    /// per bone and per mesh group; one level-of-detail record; the fixed
    /// blocks 18 and 20; the extension headers re-emitted after the last
    /// string.
    pub fn to_file(&self) -> Result<FmdlFile, FmdlError> {
        let mut file = FmdlFile {
            version: 0x4001eb85,
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
        let mut strings = Strings::default();
        strings.add(&mut file, "");

        for bone in &self.bones {
            if let Some(parent) = bone.parent
                && parent >= self.bones.len()
            {
                return Err(FmdlError::BadReference {
                    what: "bone",
                    index: parent,
                });
            }
            let name_string_id = strings.add(&mut file, &bone.name);
            let bounding_box_id = add_bounding_box(&mut file, bone.bounding_box);
            file.bones.push(BoneRecord {
                name_string_id,
                parent_bone_id: bone.parent.map_or(-1, |index| index as i16),
                bounding_box_id,
                unknown_0x06: 1,
                unknown_0x08: 0,
                local_position: bone.local_position,
                world_position: bone.world_position,
            });
        }

        let mut parameter_values = Vec::new();
        for instance in &self.materials {
            let name_string_id = strings.add(&mut file, &instance.name);
            let material_id = file.materials.len() as u16;
            let shader_string_id = strings.add(&mut file, &instance.shader);
            let technique_string_id = strings.add(&mut file, &instance.technique);
            file.materials.push(MaterialRecord {
                shader_string_id,
                technique_string_id,
            });
            let first_texture_id = file.parameter_assignments.len() as u16;
            for (sampler, texture) in &instance.textures {
                let filename_string_id = strings.add(&mut file, &texture.file_name);
                let directory_string_id = strings.add(&mut file, &texture.directory);
                let parameter_string_id = strings.add(&mut file, sampler);
                let texture_id = file.textures.len() as u16;
                file.textures.push(TextureRecord {
                    filename_string_id,
                    directory_string_id,
                });
                file.parameter_assignments.push(ParameterAssignmentRecord {
                    parameter_string_id,
                    reference_id: texture_id,
                });
            }
            let first_parameter_id = file.parameter_assignments.len() as u16;
            for (name, values) in &instance.parameters {
                let parameter_string_id = strings.add(&mut file, name);
                let value_id = (parameter_values.len() / 16) as u16;
                for value in values {
                    parameter_values.extend(value.to_le_bytes());
                }
                file.parameter_assignments.push(ParameterAssignmentRecord {
                    parameter_string_id,
                    reference_id: value_id,
                });
            }
            file.material_instances.push(MaterialInstanceRecord {
                name_string_id,
                unknown_0x02: 0,
                material_id,
                texture_count: u8::try_from(instance.textures.len()).map_err(|_| {
                    FmdlError::VertexMismatch("more than 255 textures on one material instance")
                })?,
                material_parameter_count: u8::try_from(instance.parameters.len()).map_err(
                    |_| {
                        FmdlError::VertexMismatch(
                            "more than 255 parameters on one material instance",
                        )
                    },
                )?,
                first_texture_id,
                first_material_parameter_id: first_parameter_id,
                unknown_0x0c: 0,
            });
        }

        file.levels_of_detail.push(LevelOfDetailRecord {
            lod_count: 1,
            unknown_0x04: [1.0, 1.0, 1.0],
        });

        let mut positions_buffer = Vec::new();
        let mut data_buffer = Vec::new();
        let mut face_buffer = Vec::new();
        for mesh in &self.meshes {
            if mesh.material >= self.materials.len() {
                return Err(FmdlError::BadReference {
                    what: "material instance",
                    index: mesh.material,
                });
            }
            let vertex_count = mesh.vertices.positions.len();
            if vertex_count > u16::MAX as usize {
                return Err(FmdlError::TooManyVertices(vertex_count));
            }
            let vertices = &mesh.vertices;
            check_vertex_lengths(vertices)?;
            let skinned = vertices.bone_indices.is_some();
            if skinned != vertices.bone_weights.is_some() {
                return Err(FmdlError::InvalidVertexFormat(
                    "bone weights and bone indices must come together",
                ));
            }

            let bone_group_id = if skinned {
                if mesh.bone_group.len() > 32 {
                    return Err(FmdlError::TooManyBones(mesh.bone_group.len()));
                }
                let mut bone_ids = [0u16; 32];
                for (slot, bone) in bone_ids.iter_mut().zip(&mesh.bone_group) {
                    if *bone >= self.bones.len() {
                        return Err(FmdlError::BadReference {
                            what: "bone",
                            index: *bone,
                        });
                    }
                    *slot = *bone as u16;
                }
                let id = file.bone_groups.len() as u16;
                file.bone_groups.push(BoneGroupRecord {
                    unknown_0x00: 4,
                    entry_count: mesh.bone_group.len() as u16,
                    bone_ids,
                });
                id
            } else {
                0
            };

            // The vertex format: position in buffer 0, everything else
            // interleaved in buffer 1 in a fixed order; equal uv maps share
            // the earlier map's offset.
            let first_mesh_format_id = file.mesh_formats.len() as u16;
            let first_vertex_format_id = file.vertex_formats.len() as u16;
            let position_base = positions_buffer.len();
            let data_base = data_buffer.len();
            let mut type_entries = [0u8; 4];
            let mut data_offset = 0usize;
            let mut vertex_format = |datum_type: DatumType, format: DatumFormat, offset: usize| {
                file.vertex_formats.push(VertexFormatRecord {
                    datum_type: datum_type.id(),
                    datum_format: format.id(),
                    offset: offset as u16,
                });
            };
            vertex_format(DatumType::Position, DatumFormat::TripleFloat32, 0);
            type_entries[0] += 1;
            if vertices.normals.is_some() {
                vertex_format(DatumType::Normal, DatumFormat::QuadFloat16, data_offset);
                data_offset += 8;
                type_entries[1] += 1;
            }
            if vertices.tangents.is_some() {
                vertex_format(DatumType::Tangent, DatumFormat::QuadFloat16, data_offset);
                data_offset += 8;
                type_entries[1] += 1;
            }
            if vertices.colors.is_some() {
                vertex_format(DatumType::Color, DatumFormat::QuadFloat8, data_offset);
                data_offset += 4;
                type_entries[2] += 1;
            }
            if skinned {
                vertex_format(DatumType::BoneWeights, DatumFormat::QuadFloat8, data_offset);
                data_offset += 4;
                vertex_format(DatumType::BoneIndices, DatumFormat::QuadInt8, data_offset);
                data_offset += 4;
                type_entries[3] += 2;
            }
            let uv_types = [
                DatumType::Uv0,
                DatumType::Uv1,
                DatumType::Uv2,
                DatumType::Uv3,
            ];
            let mut uv_offsets: Vec<usize> = Vec::with_capacity(vertices.uvs.len());
            let mut uv_shared: Vec<bool> = Vec::with_capacity(vertices.uvs.len());
            for (map, uvs) in vertices.uvs.iter().enumerate() {
                let format = if vertices.uv_high_precision[map] {
                    DatumFormat::DoubleFloat32
                } else {
                    DatumFormat::DoubleFloat16
                };
                let shared = (0..map).find(|earlier| {
                    vertices.uv_high_precision[*earlier] == vertices.uv_high_precision[map]
                        && vertices.uvs[*earlier] == *uvs
                });
                match shared {
                    Some(earlier) => {
                        vertex_format(uv_types[map], format, uv_offsets[earlier]);
                        uv_offsets.push(uv_offsets[earlier]);
                        uv_shared.push(true);
                    }
                    None => {
                        vertex_format(uv_types[map], format, data_offset);
                        uv_offsets.push(data_offset);
                        uv_shared.push(false);
                        data_offset += format.element_size();
                    }
                }
                type_entries[3] += 1;
            }
            let data_stride = data_offset;

            file.mesh_formats.push(MeshFormatRecord {
                buffer_id: 0,
                vertex_format_entry_count: type_entries[0],
                buffer_offset_increment: 12,
                mesh_format_type: 0,
                buffer_offset: position_base as u32,
            });
            for (mesh_format_type, count) in type_entries.iter().enumerate().skip(1) {
                if *count > 0 {
                    file.mesh_formats.push(MeshFormatRecord {
                        buffer_id: 1,
                        vertex_format_entry_count: *count,
                        buffer_offset_increment: u8::try_from(data_stride).map_err(|_| {
                            FmdlError::VertexMismatch("vertex data stride exceeds 255")
                        })?,
                        mesh_format_type: mesh_format_type as u8,
                        buffer_offset: data_base as u32,
                    });
                }
            }
            let mesh_format_id = file.mesh_format_assignments.len() as u16;
            file.mesh_format_assignments
                .push(MeshFormatAssignmentRecord {
                    mesh_format_entry_count: (file.mesh_formats.len()
                        - usize::from(first_mesh_format_id))
                        as u8,
                    vertex_format_entry_count: (file.vertex_formats.len()
                        - usize::from(first_vertex_format_id))
                        as u8,
                    first_uv_index: 0,
                    uv_index_count: vertices.uvs.len() as u8,
                    first_mesh_format_id,
                    first_vertex_format_id,
                });

            // The vertex bytes themselves, in the same interleaved order.
            for vertex in 0..vertex_count {
                positions_buffer.extend(vertices.positions[vertex][0].to_le_bytes());
                positions_buffer.extend(vertices.positions[vertex][1].to_le_bytes());
                positions_buffer.extend(vertices.positions[vertex][2].to_le_bytes());
                if let Some(normals) = &vertices.normals {
                    for value in normals[vertex] {
                        data_buffer.extend(f32_to_f16(value).to_le_bytes());
                    }
                }
                if let Some(tangents) = &vertices.tangents {
                    for value in tangents[vertex] {
                        data_buffer.extend(f32_to_f16(value).to_le_bytes());
                    }
                }
                if let Some(colors) = &vertices.colors {
                    data_buffer.extend(colors[vertex]);
                }
                if let Some(weights) = &vertices.bone_weights {
                    data_buffer.extend(weights[vertex]);
                }
                if let Some(indices) = &vertices.bone_indices {
                    data_buffer.extend(indices[vertex]);
                }
                for (map, uvs) in vertices.uvs.iter().enumerate() {
                    if uv_shared[map] {
                        continue;
                    }
                    if vertices.uv_high_precision[map] {
                        data_buffer.extend(uvs[vertex][0].to_le_bytes());
                        data_buffer.extend(uvs[vertex][1].to_le_bytes());
                    } else {
                        data_buffer.extend(f32_to_f16(uvs[vertex][0]).to_le_bytes());
                        data_buffer.extend(f32_to_f16(uvs[vertex][1]).to_le_bytes());
                    }
                }
            }
            while positions_buffer.len() % 16 != 0 {
                positions_buffer.push(0);
            }
            while data_buffer.len() % 16 != 0 {
                data_buffer.push(0);
            }

            let first_face_index_id = file.face_indices.len() as u64;
            file.face_indices.push(FaceIndexRecord {
                first_face_vertex_index: 0,
                face_vertex_count: (mesh.faces.len() * 3) as u32,
            });
            let first_face_vertex_index = (face_buffer.len() / 2) as u32;
            for face in &mesh.faces {
                for index in face {
                    face_buffer.extend(index.to_le_bytes());
                }
            }

            file.meshes.push(MeshRecord {
                alpha_flags: mesh.alpha_flags,
                shadow_flags: mesh.shadow_flags,
                unknown_0x02: [0; 2],
                material_instance_id: mesh.material as u16,
                bone_group_id,
                mesh_format_id,
                vertex_count: vertex_count as u16,
                unknown_0x0c: [0; 4],
                first_face_vertex_index,
                face_vertex_count: (mesh.faces.len() * 3) as u32,
                first_face_index_id,
                unknown_0x20: [0; 16],
            });
        }
        file.buffer_offsets.push(BufferOffsetRecord {
            eof: 0,
            length: positions_buffer.len() as u32,
            offset: 0,
            unknown_0x0c: 0,
        });
        file.buffer_offsets.push(BufferOffsetRecord {
            eof: 0,
            length: data_buffer.len() as u32,
            offset: positions_buffer.len() as u32,
            unknown_0x0c: 0,
        });
        file.buffer_offsets.push(BufferOffsetRecord {
            eof: 1,
            length: face_buffer.len() as u32,
            offset: (positions_buffer.len() + data_buffer.len()) as u32,
            unknown_0x0c: 0,
        });
        let mut buffer = positions_buffer;
        buffer.extend(data_buffer);
        buffer.extend(face_buffer);
        file.buffer = Some(buffer);

        for (index, group) in self.mesh_groups.iter().enumerate() {
            if let Some(parent) = group.parent
                && parent >= self.mesh_groups.len()
            {
                return Err(FmdlError::BadReference {
                    what: "mesh group",
                    index: parent,
                });
            }
            let name_string_id = strings.add(&mut file, &group.name);
            file.mesh_groups.push(MeshGroupRecord {
                name_string_id,
                invisible: u16::from(!group.visible),
                parent_mesh_group_id: group.parent.map_or(-1, |index| index as i16),
                unknown_0x06: -1,
            });
            let bounding_box_id = match group.bounding_box {
                Some(bounding_box) => Some(add_bounding_box(&mut file, bounding_box)),
                None if group.meshes.is_empty() => None,
                None => Some(add_bounding_box(
                    &mut file,
                    compute_bounding_box(self, group),
                )),
            };
            // Runs of consecutive mesh indices, one assignment record each,
            // all naming the group's bounding box.
            let mut runs: Vec<(usize, usize)> = Vec::new();
            for mesh in &group.meshes {
                if *mesh >= self.meshes.len() {
                    return Err(FmdlError::BadReference {
                        what: "mesh",
                        index: *mesh,
                    });
                }
                match runs.last_mut() {
                    Some((first, count)) if *mesh == *first + *count => *count += 1,
                    _ => runs.push((*mesh, 1)),
                }
            }
            if let Some(bounding_box_id) = bounding_box_id {
                for (first, count) in &runs {
                    file.mesh_group_assignments.push(MeshGroupAssignmentRecord {
                        unknown_0x00: [0; 4],
                        mesh_group_id: index as u16,
                        mesh_count: *count as u16,
                        first_mesh_id: *first as u16,
                        bounding_box_id,
                        unknown_0x0c: [0; 4],
                        unknown_0x10: 0,
                        unknown_0x12: [0; 14],
                    });
                }
                if runs.is_empty() {
                    file.mesh_group_assignments.push(MeshGroupAssignmentRecord {
                        unknown_0x00: [0; 4],
                        mesh_group_id: index as u16,
                        mesh_count: 0,
                        first_mesh_id: 0,
                        bounding_box_id,
                        unknown_0x0c: [0; 4],
                        unknown_0x10: 0,
                        unknown_0x12: [0; 14],
                    });
                }
            }
        }

        // The extension header text after the last string.
        let mut flags = Vec::new();
        if self.extensions.mesh_splitting {
            flags.push("mesh-splitting".to_owned());
        }
        if self.extensions.antiblur {
            flags.push("antiblur".to_owned());
        }
        if self.extensions.vertex_loop_preservation {
            flags.push("vertex-loop-preservation".to_owned());
        }
        flags.extend(self.extensions.other.iter().cloned());
        let mut header_lines: Vec<(String, Vec<String>)> = Vec::new();
        for (name, values) in [
            (
                "Has-Antiblur-Meshes",
                self.meshes
                    .iter()
                    .enumerate()
                    .filter(|(_, mesh)| mesh.has_antiblur_meshes)
                    .map(|(index, _)| index.to_string())
                    .collect::<Vec<_>>(),
            ),
            (
                "Is-Antiblur-Meshes",
                self.meshes
                    .iter()
                    .enumerate()
                    .filter(|(_, mesh)| mesh.is_antiblur_mesh)
                    .map(|(index, _)| index.to_string())
                    .collect::<Vec<_>>(),
            ),
            (
                "Split-Mesh-Groups",
                self.mesh_groups
                    .iter()
                    .enumerate()
                    .filter(|(_, group)| group.split_mesh_group)
                    .map(|(index, _)| index.to_string())
                    .collect::<Vec<_>>(),
            ),
        ] {
            if !values.is_empty() {
                header_lines.push((name.to_owned(), values));
            }
        }
        if !flags.is_empty() || !header_lines.is_empty() {
            let mut text = String::from("X-FMDL-Extensions: ");
            text.push_str(&flags.join(", "));
            text.push('\n');
            for (name, values) in header_lines {
                text.push_str(&name);
                text.push_str(": ");
                text.push_str(&values.join(", "));
                text.push('\n');
            }
            strings.table.extend(text.as_bytes());
            strings.table.push(0);
        }

        file.block_18.push(Block18Record { bytes: [0; 8] });
        let mut block_20 = [0u8; 128];
        block_20[0..4].copy_from_slice(&0.0f32.to_le_bytes());
        block_20[4..8].copy_from_slice(&1.0f32.to_le_bytes());
        block_20[8..12].copy_from_slice(&1.0f32.to_le_bytes());
        block_20[12..16].copy_from_slice(&1.0f32.to_le_bytes());
        block_20[28..32].copy_from_slice(&(-1i32).to_le_bytes());
        file.block_20.push(Block20Record { bytes: block_20 });

        file.material_parameters = Some(parameter_values);
        file.bone_matrices = match &self.bone_matrices {
            Some(matrices) => Some(matrices.clone()),
            None if self.bones.is_empty() => None,
            None => Some(Vec::new()),
        };
        file.string_table = Some(strings.table);

        Ok(file)
    }
}

/// Every attribute vector must name exactly one value per vertex.
fn check_vertex_lengths(vertices: &MeshVertices) -> Result<(), FmdlError> {
    let count = vertices.positions.len();
    let option_lengths = [
        vertices.normals.as_ref().map(Vec::len),
        vertices.tangents.as_ref().map(Vec::len),
        vertices.colors.as_ref().map(Vec::len),
        vertices.bone_weights.as_ref().map(Vec::len),
        vertices.bone_indices.as_ref().map(Vec::len),
    ];
    for length in option_lengths.into_iter().flatten() {
        if length != count {
            return Err(FmdlError::VertexMismatch("attribute count mismatch"));
        }
    }
    if vertices.uvs.len() > 4 {
        return Err(FmdlError::InvalidVertexFormat("more than four uv maps"));
    }
    if vertices.uv_high_precision.len() != vertices.uvs.len() {
        return Err(FmdlError::InvalidVertexFormat(
            "uv precision flags do not match uv maps",
        ));
    }
    for uvs in &vertices.uvs {
        if uvs.len() != count {
            return Err(FmdlError::VertexMismatch("attribute count mismatch"));
        }
    }
    Ok(())
}

/// The bounding box of a group's meshes' vertex positions, `w` set to the
/// 1.0 the Konami boxes carry.
fn compute_bounding_box(model: &Model, group: &MeshGroup) -> BoundingBox {
    let mut max = [f32::NEG_INFINITY; 4];
    let mut min = [f32::INFINITY; 4];
    for mesh in &group.meshes {
        for position in &model.meshes[*mesh].vertices.positions {
            for axis in 0..3 {
                max[axis] = max[axis].max(position[axis]);
                min[axis] = min[axis].min(position[axis]);
            }
        }
    }
    max[3] = 1.0;
    min[3] = 1.0;
    BoundingBox { max, min }
}

/// Appends a bounding-box record, returning its id.
fn add_bounding_box(file: &mut FmdlFile, bounding_box: BoundingBox) -> u16 {
    let id = file.bounding_boxes.len() as u16;
    file.bounding_boxes.push(BoundingBoxRecord {
        max: bounding_box.max,
        min: bounding_box.min,
    });
    id
}

/// The string table builder: index 0 is the empty string, every other
/// string de-duplicated in first-use order, NUL-terminated in block 3.
#[derive(Default)]
struct Strings {
    /// The section-1 block 3 bytes.
    table: Vec<u8>,
    /// String to index, for de-duplication.
    indices: HashMap<String, u16>,
}

impl Strings {
    /// Adds `string`, returning its index in the strings block.
    fn add(&mut self, file: &mut FmdlFile, string: &str) -> u16 {
        if let Some(index) = self.indices.get(string) {
            return *index;
        }
        let index = file.strings.len() as u16;
        file.strings.push(StringRecord {
            string_block_id: 3,
            length: string.len() as u16,
            offset: self.table.len() as u32,
        });
        self.table.extend(string.as_bytes());
        self.table.push(0);
        self.indices.insert(string.to_owned(), index);
        index
    }
}
