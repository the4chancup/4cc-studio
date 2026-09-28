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
    let tail = file.extension_tail();
    let end = tail
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(tail.len());
    let text = std::str::from_utf8(&tail[..end]).ok()?;
    if !text.to_lowercase().starts_with("x-fmdl-extensions:") {
        return None;
    }

    let mut flags = Vec::new();
    let mut objects = BTreeMap::new();
    for (line_index, line) in text.split('\n').enumerate() {
        let Some(colon) = line.find(':') else {
            continue;
        };
        // Only the fixed keys are ever looked up, so a key that could never
        // be one (empty, inner whitespace) needs no filter.
        let key = line[..colon].trim().to_lowercase();
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

/// A stored parent id: negative is a root. The range is `validate`'s check.
fn parent(id: i16) -> Option<usize> {
    usize::try_from(id).ok()
}

/// The bones, resolved: names, parents and bounding boxes looked up in
/// their tables.
fn read_bones(file: &FmdlFile) -> Result<Vec<Bone>, FmdlError> {
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
            parent: parent(record.parent_bone_id),
            bounding_box: BoundingBox {
                max: bounding_box.max,
                min: bounding_box.min,
            },
            local_position: record.local_position,
            world_position: record.world_position,
        });
    }
    Ok(bones)
}

/// The material instances, resolved: names, techniques, texture and
/// parameter assignments looked up in their tables.
fn read_materials(file: &FmdlFile) -> Result<Vec<MaterialInstance>, FmdlError> {
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
        let (technique, shader) =
            material_names
                .get(usize::from(record.material_id))
                .ok_or(FmdlError::BadReference {
                    what: "material",
                    index: usize::from(record.material_id),
                })?;
        let mut instance_textures = Vec::new();
        for index in usize::from(record.first_texture_id)
            ..usize::from(record.first_texture_id) + usize::from(record.texture_count)
        {
            let (sampler, reference) = assignments.get(index).ok_or(FmdlError::BadReference {
                what: "texture / parameter assignment",
                index,
            })?;
            let texture = textures
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
            let values =
                parameter_values
                    .get(usize::from(*reference))
                    .ok_or(FmdlError::BadReference {
                        what: "material parameter",
                        index: usize::from(*reference),
                    })?;
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
    Ok(materials)
}

/// The mesh groups and, per mesh, the bounding box of the group its
/// assignment put it in (`None` at a slot no assignment covered — the
/// all-assigned check below errors before anything reads it).
fn read_mesh_groups(
    file: &FmdlFile,
    headers: Option<&ExtensionHeaders>,
) -> Result<(Vec<MeshGroup>, Vec<Option<BoundingBox>>), FmdlError> {
    let mut mesh_groups = Vec::with_capacity(file.mesh_groups.len());
    for (index, record) in file.mesh_groups.iter().enumerate() {
        mesh_groups.push(MeshGroup {
            name: file.string(usize::from(record.name_string_id))?.to_owned(),
            parent: parent(record.parent_mesh_group_id),
            meshes: Vec::new(),
            bounding_box: None,
            visible: record.invisible == 0,
            split_mesh_group: headers.is_some_and(|h| h.applies_to("split-mesh-groups", index)),
        });
    }

    // Assignments hand runs of consecutive meshes and one bounding box
    // to a group; every mesh must land in exactly one group. A group's
    // non-consecutive runs repeat the assignment with the same box —
    // a different box id is a read error.
    let mut assigned: Vec<Option<usize>> = vec![None; file.meshes.len()];
    let mut mesh_boxes: Vec<Option<BoundingBox>> = vec![None; file.meshes.len()];
    let mut group_boxes: Vec<Option<u16>> = vec![None; mesh_groups.len()];
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
        let bounding_box = BoundingBox {
            max: bounding_box.max,
            min: bounding_box.min,
        };
        for (mesh_index, slot) in assigned.iter_mut().enumerate().take(end).skip(first) {
            if slot.is_some() {
                return Err(FmdlError::BadMeshGroupAssignment(
                    "mesh assigned to two groups",
                ));
            }
            *slot = Some(group);
            mesh_boxes[mesh_index] = Some(bounding_box);
            mesh_groups[group].meshes.push(mesh_index);
        }
        match group_boxes[group] {
            Some(first_id) if first_id != record.bounding_box_id => {
                return Err(FmdlError::BadMeshGroupAssignment(
                    "mesh group assigned two bounding boxes",
                ));
            }
            _ => {
                group_boxes[group] = Some(record.bounding_box_id);
                mesh_groups[group].bounding_box = Some(bounding_box);
            }
        }
    }
    for assignment in &assigned {
        if assignment.is_none() {
            return Err(FmdlError::BadMeshGroupAssignment(
                "mesh not assigned to a group",
            ));
        }
    }
    // A `Split-Mesh-Groups` entry is a container only when the group has
    // a parent and at least one mesh: writers also list roots and empty
    // groups, which read as ordinary groups so a rewrite drops them.
    for group in &mut mesh_groups {
        if group.split_mesh_group && (group.parent.is_none() || group.meshes.is_empty()) {
            group.split_mesh_group = false;
        }
    }
    Ok((mesh_groups, mesh_boxes))
}

/// The meshes, resolved: vertices and faces decoded, bone groups,
/// materials and the per-object extension flags.
fn read_meshes(
    file: &FmdlFile,
    headers: Option<&ExtensionHeaders>,
    mesh_boxes: &[Option<BoundingBox>],
) -> Result<Vec<Mesh>, FmdlError> {
    let mut meshes = Vec::with_capacity(file.meshes.len());
    for (index, record) in file.meshes.iter().enumerate() {
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
            bone_group.extend(group.bone_ids[..count].iter().map(|id| usize::from(*id)));
        }
        meshes.push(Mesh {
            vertices,
            faces: file.decode_faces(index)?,
            bone_group,
            material: usize::from(record.material_instance_id),
            alpha_flags: record.alpha_flags,
            shadow_flags: record.shadow_flags,
            has_antiblur_meshes: headers
                .is_some_and(|h| h.applies_to("has-antiblur-meshes", index)),
            is_antiblur_mesh: headers.is_some_and(|h| h.applies_to("is-antiblur-meshes", index)),
            // The header marks the mesh; the box itself is the one its
            // group got from the assignment.
            custom_bounding_box: if headers
                .is_some_and(|h| h.applies_to("custom-bounding-box-meshes", index))
            {
                mesh_boxes[index]
            } else {
                None
            },
        });
    }
    Ok(meshes)
}

/// The model-level extension flags; unknown ones keep their text.
fn read_extensions(headers: Option<&ExtensionHeaders>) -> Extensions {
    let mut extensions = Extensions::default();
    if let Some(headers) = headers {
        for flag in &headers.flags {
            match flag.as_str() {
                "mesh-splitting" => extensions.mesh_splitting = true,
                "antiblur" => extensions.antiblur = true,
                "vertex-loop-preservation" => extensions.vertex_loop_preservation = true,
                _ => extensions.other.push(flag.clone()),
            }
        }
    }
    extensions
}

impl Model {
    /// Builds a `Model` from a parsed file, resolving every index through
    /// the file's tables.
    pub fn from_file(file: &FmdlFile) -> Result<Model, FmdlError> {
        let headers = extension_headers(file);
        let bones = read_bones(file)?;
        let materials = read_materials(file)?;
        let (mesh_groups, mesh_boxes) = read_mesh_groups(file, headers.as_ref())?;
        let meshes = read_meshes(file, headers.as_ref(), &mesh_boxes)?;
        let extensions = read_extensions(headers.as_ref());
        let model = Model {
            bones,
            materials,
            meshes,
            mesh_groups,
            extensions,
            bone_matrices: file.bone_matrices.clone(),
        };
        model.validate()?;
        Ok(model)
    }
}
