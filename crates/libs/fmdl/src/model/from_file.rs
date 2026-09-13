use std::collections::BTreeMap;

use super::{Bone, BoundingBox, Extensions, MaterialInstance, Mesh, MeshGroup, Model, Texture};
use crate::format::{FmdlError, FmdlFile};

/// The extension header block: the `x-fmdl-extensions` value list plus the
/// per-object headers, keys lower-cased, first occurrence of a key winning.
struct ExtensionHeaders {
    /// The `x-fmdl-extensions` values.
    flags: Vec<String>,
    /// Other headers, key to value list.
    objects: BTreeMap<String, Vec<String>>,
}

impl ExtensionHeaders {
    /// Whether header `key` lists object `index`.
    fn applies_to(&self, key: &str, index: usize) -> bool {
        self.objects
            .get(key)
            .is_some_and(|values| values.iter().any(|value| value == &index.to_string()))
    }
}

/// Parses the string table's tail: after the last block-3 string, a
/// NUL-terminated `X-FMDL-Extensions:` text (case-insensitive) with
/// HTTP-style `key: v1, v2` lines. `None` when no such text is present.
fn extension_headers(file: &FmdlFile) -> Option<ExtensionHeaders> {
    let table = file.string_table.as_deref()?;
    let last_end = file
        .strings
        .iter()
        .filter(|record| record.string_block_id == 3)
        .map(|record| record.offset as usize + usize::from(record.length))
        .max()?;
    let position = last_end + 1;
    if position >= table.len() {
        return None;
    }
    let end = table[position..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|index| position + index)
        .unwrap_or(table.len());
    let text = std::str::from_utf8(&table[position..end]).ok()?;
    if !text.to_lowercase().starts_with("x-fmdl-extensions:") {
        return None;
    }

    let mut flags = Vec::new();
    let mut objects = BTreeMap::new();
    for (line_index, line) in text.split('\n').enumerate() {
        let Some(colon) = line.find(':') else {
            continue;
        };
        let key = line[..colon].trim().to_lowercase();
        if key.is_empty()
            || key
                .chars()
                .any(|c| c == ' ' || c == '\t' || c == '\r' || c == '\n')
        {
            continue;
        }
        let values: Vec<String> = line[colon + 1..]
            .split(',')
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .collect();
        if line_index == 0 && key == "x-fmdl-extensions" {
            flags = values;
            continue;
        }
        // First occurrence of a key wins.
        objects.entry(key).or_insert(values);
    }
    Some(ExtensionHeaders { flags, objects })
}

/// A resolved parent id: negative is `None`, otherwise an index into
/// `table_len` entries.
fn parent(id: i16, table_len: usize, what: &'static str) -> Result<Option<usize>, FmdlError> {
    if id < 0 {
        return Ok(None);
    }
    let index = id as usize;
    if index >= table_len {
        return Err(FmdlError::BadReference { what, index });
    }
    Ok(Some(index))
}

/// Every parent chain must reach a root; a loop is `ParentCycle`.
fn check_cycles(parents: &[Option<usize>], what: &'static str) -> Result<(), FmdlError> {
    for start in 0..parents.len() {
        let mut seen = vec![false; parents.len()];
        let mut current = Some(start);
        while let Some(index) = current {
            if seen[index] {
                return Err(FmdlError::ParentCycle(what));
            }
            seen[index] = true;
            current = parents[index];
        }
    }
    Ok(())
}

impl Model {
    /// Builds a `Model` from a parsed file, resolving every index through
    /// the file's tables.
    pub fn from_file(file: &FmdlFile) -> Result<Model, FmdlError> {
        let headers = extension_headers(file);

        let mut bones = Vec::with_capacity(file.bones.len());
        for record in &file.bones {
            let bounding_box = file
                .bounding_boxes
                .get(usize::from(record.bounding_box_id))
                .ok_or(FmdlError::BadReference {
                    what: "bounding box",
                    index: usize::from(record.bounding_box_id),
                })?;
            bones.push(Bone {
                name: file.string(usize::from(record.name_string_id))?.to_owned(),
                parent: parent(record.parent_bone_id, file.bones.len(), "bone")?,
                bounding_box: BoundingBox {
                    max: bounding_box.max,
                    min: bounding_box.min,
                },
                local_position: record.local_position,
                world_position: record.world_position,
            });
        }
        check_cycles(
            &bones.iter().map(|bone| bone.parent).collect::<Vec<_>>(),
            "bone",
        )?;

        // The tables the material instances resolve through.
        let mut material_names = Vec::with_capacity(file.materials.len());
        for record in &file.materials {
            material_names.push((
                file.string(usize::from(record.technique_string_id))?
                    .to_owned(),
                file.string(usize::from(record.shader_string_id))?
                    .to_owned(),
            ));
        }
        let mut textures = Vec::with_capacity(file.textures.len());
        for record in &file.textures {
            textures.push(Texture {
                file_name: file
                    .string(usize::from(record.filename_string_id))?
                    .to_owned(),
                directory: file
                    .string(usize::from(record.directory_string_id))?
                    .to_owned(),
            });
        }
        // Section-1 block 0 is a run of 16-byte float quads.
        let mut parameter_values = Vec::new();
        if let Some(block) = &file.material_parameters {
            for quad in block.as_chunks::<16>().0 {
                parameter_values.push([
                    f32::from_le_bytes([quad[0], quad[1], quad[2], quad[3]]),
                    f32::from_le_bytes([quad[4], quad[5], quad[6], quad[7]]),
                    f32::from_le_bytes([quad[8], quad[9], quad[10], quad[11]]),
                    f32::from_le_bytes([quad[12], quad[13], quad[14], quad[15]]),
                ]);
            }
        }
        // One shared table of (parameter name, reference) serves both the
        // texture and the parameter runs.
        let mut assignments = Vec::with_capacity(file.parameter_assignments.len());
        for record in &file.parameter_assignments {
            assignments.push((
                file.string(usize::from(record.parameter_string_id))?
                    .to_owned(),
                record.reference_id,
            ));
        }

        let mut materials = Vec::with_capacity(file.material_instances.len());
        for record in &file.material_instances {
            let (technique, shader) = material_names.get(usize::from(record.material_id)).ok_or(
                FmdlError::BadReference {
                    what: "material",
                    index: usize::from(record.material_id),
                },
            )?;
            let mut instance_textures = Vec::new();
            for index in usize::from(record.first_texture_id)
                ..usize::from(record.first_texture_id) + usize::from(record.texture_count)
            {
                let (sampler, reference) =
                    assignments.get(index).ok_or(FmdlError::BadReference {
                        what: "texture / parameter assignment",
                        index,
                    })?;
                let texture =
                    textures
                        .get(usize::from(*reference))
                        .ok_or(FmdlError::BadReference {
                            what: "texture",
                            index: usize::from(*reference),
                        })?;
                instance_textures.push((sampler.clone(), texture.clone()));
            }
            let mut instance_parameters = Vec::new();
            for index in usize::from(record.first_material_parameter_id)
                ..usize::from(record.first_material_parameter_id)
                    + usize::from(record.material_parameter_count)
            {
                let (name, reference) = assignments.get(index).ok_or(FmdlError::BadReference {
                    what: "texture / parameter assignment",
                    index,
                })?;
                let values = parameter_values.get(usize::from(*reference)).ok_or(
                    FmdlError::BadReference {
                        what: "material parameter",
                        index: usize::from(*reference),
                    },
                )?;
                instance_parameters.push((name.clone(), *values));
            }
            materials.push(MaterialInstance {
                name: file.string(usize::from(record.name_string_id))?.to_owned(),
                shader: shader.clone(),
                technique: technique.clone(),
                textures: instance_textures,
                parameters: instance_parameters,
            });
        }

        let mut mesh_groups = Vec::with_capacity(file.mesh_groups.len());
        for (index, record) in file.mesh_groups.iter().enumerate() {
            mesh_groups.push(MeshGroup {
                name: file.string(usize::from(record.name_string_id))?.to_owned(),
                parent: parent(
                    record.parent_mesh_group_id,
                    file.mesh_groups.len(),
                    "mesh group",
                )?,
                meshes: Vec::new(),
                bounding_box: None,
                visible: record.invisible == 0,
                split_mesh_group: headers
                    .as_ref()
                    .is_some_and(|h| h.applies_to("split-mesh-groups", index)),
            });
        }
        check_cycles(
            &mesh_groups
                .iter()
                .map(|group| group.parent)
                .collect::<Vec<_>>(),
            "mesh group",
        )?;

        // Assignments hand runs of consecutive meshes and one bounding box
        // to a group; every mesh must land in exactly one group.
        let mut assigned: Vec<Option<usize>> = vec![None; file.meshes.len()];
        for record in &file.mesh_group_assignments {
            let group = usize::from(record.mesh_group_id);
            if group >= mesh_groups.len() {
                return Err(FmdlError::BadReference {
                    what: "mesh group",
                    index: group,
                });
            }
            let first = usize::from(record.first_mesh_id);
            let end = first + usize::from(record.mesh_count);
            if end > file.meshes.len() {
                return Err(FmdlError::BadReference {
                    what: "mesh",
                    index: end.saturating_sub(1),
                });
            }
            let bounding_box = file
                .bounding_boxes
                .get(usize::from(record.bounding_box_id))
                .ok_or(FmdlError::BadReference {
                    what: "bounding box",
                    index: usize::from(record.bounding_box_id),
                })?;
            for (mesh_index, slot) in assigned.iter_mut().enumerate().take(end).skip(first) {
                if slot.is_some() {
                    return Err(FmdlError::BadMeshGroupAssignment(
                        "mesh assigned to two groups",
                    ));
                }
                *slot = Some(group);
                mesh_groups[group].meshes.push(mesh_index);
            }
            if mesh_groups[group].bounding_box.is_some() {
                return Err(FmdlError::BadMeshGroupAssignment(
                    "mesh group assigned two bounding boxes",
                ));
            }
            mesh_groups[group].bounding_box = Some(BoundingBox {
                max: bounding_box.max,
                min: bounding_box.min,
            });
        }
        for assignment in &assigned {
            if assignment.is_none() {
                return Err(FmdlError::BadMeshGroupAssignment(
                    "mesh not assigned to a group",
                ));
            }
        }

        let mut meshes = Vec::with_capacity(file.meshes.len());
        for (index, record) in file.meshes.iter().enumerate() {
            let material = usize::from(record.material_instance_id);
            if material >= materials.len() {
                return Err(FmdlError::BadReference {
                    what: "material instance",
                    index: material,
                });
            }
            let vertices = file.decode_vertices(index)?;
            // The record's bone group applies only when the format carries
            // bone weights and indices.
            let mut bone_group = Vec::new();
            if vertices.bone_indices.is_some() {
                let group = file
                    .bone_groups
                    .get(usize::from(record.bone_group_id))
                    .ok_or(FmdlError::BadReference {
                        what: "bone group",
                        index: usize::from(record.bone_group_id),
                    })?;
                let count = usize::from(group.entry_count).min(32);
                for bone_id in &group.bone_ids[..count] {
                    let bone = usize::from(*bone_id);
                    if bone >= bones.len() {
                        return Err(FmdlError::BadReference {
                            what: "bone",
                            index: bone,
                        });
                    }
                    bone_group.push(bone);
                }
            }
            let has_antiblur = headers
                .as_ref()
                .is_some_and(|h| h.applies_to("has-antiblur-meshes", index));
            let is_antiblur = headers
                .as_ref()
                .is_some_and(|h| h.applies_to("is-antiblur-meshes", index));
            meshes.push(Mesh {
                vertices,
                faces: file.decode_faces(index)?,
                bone_group,
                material,
                alpha_flags: record.alpha_flags,
                shadow_flags: record.shadow_flags,
                has_antiblur_meshes: has_antiblur,
                is_antiblur_mesh: is_antiblur,
                // The header only flags the mesh; the format has no per-mesh
                // bounding box to fill this from.
                custom_bounding_box: None,
            });
        }

        let mut extensions = Extensions::default();
        if let Some(headers) = &headers {
            for flag in &headers.flags {
                match flag.as_str() {
                    "mesh-splitting" => extensions.mesh_splitting = true,
                    "antiblur" => extensions.antiblur = true,
                    "vertex-loop-preservation" => extensions.vertex_loop_preservation = true,
                    _ => extensions.other.push(flag.clone()),
                }
            }
        }

        Ok(Model {
            bones,
            materials,
            meshes,
            mesh_groups,
            extensions,
            bone_matrices: file.bone_matrices.clone(),
        })
    }
}
