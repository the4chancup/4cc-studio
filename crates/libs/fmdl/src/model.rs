//! `Model`: the semantic layer the ops work on. `from_file` resolves every
//! index through the file's tables (strings, bounding boxes, bone groups,
//! materials, textures, the shared texture/parameter assignment table,
//! mesh-group assignments) and reads the extension headers from the string
//! table's tail. Every dangling index is an error, never a panic.

use std::collections::BTreeMap;

use crate::format::{FmdlError, FmdlFile, MeshVertices};

/// A parsed FMDL at the semantic level: bones, materials, meshes, groups
/// and the extension header, with every index resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// Bones in file order; `Bone::parent` indexes this vector.
    pub bones: Vec<Bone>,
    /// Material instances in file order.
    pub materials: Vec<MaterialInstance>,
    /// Meshes in file order.
    pub meshes: Vec<Mesh>,
    /// Mesh groups in file order; `MeshGroup::parent` indexes this vector.
    pub mesh_groups: Vec<MeshGroup>,
    /// The `X-FMDL-Extensions` header: which encodings the file declares.
    pub extensions: Extensions,
    /// Section-1 block 1, 64 bytes per bone in Konami files; carried as is
    /// (the add-on writes it empty).
    pub bone_matrices: Option<Vec<u8>>,
}

/// A bone with its name and parents resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    /// The bone's name (e.g. `sk_head`).
    pub name: String,
    /// Index into `Model::bones`, `None` for a root.
    pub parent: Option<usize>,
    /// The bone's bounding box.
    pub bounding_box: BoundingBox,
    /// Position relative to the parent bone.
    pub local_position: [f32; 4],
    /// Position in model space.
    pub world_position: [f32; 4],
}

/// An axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// The maximum corner.
    pub max: [f32; 4],
    /// The minimum corner.
    pub min: [f32; 4],
}

/// A texture reference with its strings resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// The texture file name (kept verbatim, extension included).
    pub file_name: String,
    /// The texture directory.
    pub directory: String,
}

/// A material instance: material, texture and parameter runs resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialInstance {
    /// The instance's name.
    pub name: String,
    /// The material's shader name.
    pub shader: String,
    /// The material's technique name.
    pub technique: String,
    /// (sampler name, texture), in file order.
    pub textures: Vec<(String, Texture)>,
    /// (parameter name, four floats), in file order.
    pub parameters: Vec<(String, [f32; 4])>,
}

/// A mesh with its vertex data, faces and references resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// The decoded vertices; bone indices index `bone_group`.
    pub vertices: MeshVertices,
    /// Triangles as indices into `vertices`.
    pub faces: Vec<[u16; 3]>,
    /// Indices into `Model::bones`, at most 32; empty when the mesh is not
    /// skinned (the record's `bone_group_id` is ignored then).
    pub bone_group: Vec<usize>,
    /// Index into `Model::materials`.
    pub material: usize,
    /// Transparency draw flags.
    pub alpha_flags: u8,
    /// Shadow draw flags.
    pub shadow_flags: u8,
    /// The `Has-Antiblur-Meshes` extension header lists this mesh.
    pub has_antiblur_meshes: bool,
    /// The `Is-Antiblur-Meshes` extension header lists this mesh.
    pub is_antiblur_mesh: bool,
    /// The mesh's custom bounding box, when the `Custom-Bounding-Box-Meshes`
    /// header applies.
    pub custom_bounding_box: Option<BoundingBox>,
}

/// A mesh group with its meshes, parent and bounding box resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshGroup {
    /// The group's name.
    pub name: String,
    /// Index into `Model::mesh_groups`, `None` for a root.
    pub parent: Option<usize>,
    /// Indices into `Model::meshes`, in assignment order.
    pub meshes: Vec<usize>,
    /// The bounding box its assignment names; `None` for a group no
    /// assignment covers.
    pub bounding_box: Option<BoundingBox>,
    /// Whether the group is visible.
    pub visible: bool,
    /// The `Split-Mesh-Groups` extension header lists this group.
    pub split_mesh_group: bool,
}

/// The `X-FMDL-Extensions` flags the file declares.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extensions {
    /// `mesh-splitting`: meshes may split for bone-group limits.
    pub mesh_splitting: bool,
    /// `antiblur`: the file carries anti-blur mesh data.
    pub antiblur: bool,
    /// `vertex-loop-preservation`: vertex order is meaningful.
    pub vertex_loop_preservation: bool,
    /// Extension values outside the known three, kept verbatim so a rewrite
    /// can emit them again.
    pub other: Vec<String>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const HIGHNECK: &[u8] = include_bytes!("../tests/fixtures/konami_highneck.fmdl");
    const MOUTH: &[u8] = include_bytes!("../tests/fixtures/konami_mouth.fmdl");
    const AU_LOW: &[u8] = include_bytes!("../tests/fixtures/konami_au_Low_parts.fmdl");
    const ORAL: &[u8] = include_bytes!("../tests/fixtures/addon_oral.fmdl");
    const PLACEHOLDER: &[u8] = include_bytes!("../tests/fixtures/addon_placeholder.fmdl");

    const FIXTURES: &[&[u8]] = &[HIGHNECK, MOUTH, AU_LOW, ORAL, PLACEHOLDER];

    fn model(bytes: &[u8]) -> Model {
        Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap()
    }

    #[test]
    fn every_fixture_loads() {
        for bytes in FIXTURES {
            Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap();
        }
    }

    /// (name, parent, local position) per bone.
    type ExpectedBones<'a> = &'a [(&'a str, Option<usize>, [f32; 4])];
    /// (sampler, directory, file name) per texture.
    type ExpectedTextures<'a> = &'a [(&'a str, &'a str, &'a str)];
    /// (name, values) per material parameter.
    type ExpectedParameters<'a> = &'a [(&'a str, [f32; 4])];
    /// (name, parent, visible, meshes, has a bounding box) per group.
    type ExpectedGroups<'a> = &'a [(&'a str, Option<usize>, bool, &'a [usize], bool)];
    /// (vertex count, face count, material, alpha, shadow, bone group).
    type ExpectedMeshes<'a> = &'a [(usize, usize, usize, u8, u8, &'a [usize])];

    fn check(
        bytes: &[u8],
        bones: ExpectedBones<'_>,
        materials: &[(
            &str,
            &str,
            &str,
            ExpectedTextures<'_>,
            ExpectedParameters<'_>,
        )],
        groups: ExpectedGroups<'_>,
        meshes: ExpectedMeshes<'_>,
    ) {
        let model = model(bytes);
        assert_eq!(model.bones.len(), bones.len());
        for (bone, (name, parent, local)) in model.bones.iter().zip(bones) {
            assert_eq!(bone.name, *name);
            assert_eq!(bone.parent, *parent);
            // The expected file prints positions at 4 decimals; compare at
            // that precision.
            for (component, want) in bone.local_position.iter().zip(local) {
                assert!(
                    (component - want).abs() < 1e-4,
                    "{name}: {component} != {want}"
                );
            }
        }
        assert_eq!(model.materials.len(), materials.len());
        for (material, (name, shader, technique, textures, parameters)) in
            model.materials.iter().zip(materials)
        {
            assert_eq!(material.name, *name);
            assert_eq!(material.shader, *shader);
            assert_eq!(material.technique, *technique);
            assert_eq!(material.textures.len(), textures.len());
            for ((sampler, texture), (want_sampler, want_dir, want_file)) in
                material.textures.iter().zip(*textures)
            {
                assert_eq!(sampler, want_sampler);
                assert_eq!(texture.directory, *want_dir);
                assert_eq!(texture.file_name, *want_file);
            }
            assert_eq!(material.parameters.len(), parameters.len());
            for ((param, values), (want_param, want_values)) in
                material.parameters.iter().zip(*parameters)
            {
                assert_eq!(param, want_param);
                assert_eq!(values, want_values);
            }
        }
        assert_eq!(model.mesh_groups.len(), groups.len());
        for (group, (name, parent, visible, meshes, bbox)) in model.mesh_groups.iter().zip(groups) {
            assert_eq!(group.name, *name);
            assert_eq!(group.parent, *parent);
            assert_eq!(group.visible, *visible);
            assert_eq!(group.meshes, *meshes);
            assert_eq!(group.bounding_box.is_some(), *bbox);
        }
        assert_eq!(model.meshes.len(), meshes.len());
        for (mesh, (verts, faces, material, alpha, shadow, bone_group)) in
            model.meshes.iter().zip(meshes)
        {
            assert_eq!(mesh.vertices.positions.len(), *verts);
            assert_eq!(mesh.faces.len(), *faces);
            assert_eq!(mesh.material, *material);
            assert_eq!(mesh.alpha_flags, *alpha);
            assert_eq!(mesh.shadow_flags, *shadow);
            assert_eq!(mesh.bone_group, *bone_group);
            assert_eq!(mesh.custom_bounding_box, None);
        }
    }

    #[test]
    fn highneck_content() {
        check(
            HIGHNECK,
            &[
                ("sk_chest", None, [-0.0, 0.1667, 0.0138, 1.0]),
                ("sk_neck", Some(0), [-0.0, 0.2693, 0.0197, 1.0]),
                ("sk_head", Some(1), [-0.0, 0.108, 0.0209, 1.0]),
                ("dsk_scm", None, [0.0, 0.0, 0.0, 1.0]),
                ("dsk_neckback", None, [0.0, 0.0, 0.0, 1.0]),
                ("dsk_clavicle_r", None, [0.0, 0.0, 0.0, 1.0]),
                ("dsk_clavicle_l", None, [0.0, 0.0, 0.0, 1.0]),
            ],
            &[(
                "accessory",
                "pes_3ddf_basic_color_translucent",
                "pes3DDF_Blin_Translucent_NC",
                &[
                    (
                        "Base_Tex_SRGB",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "accessory_bsm.tga",
                    ),
                    (
                        "NormalMap_Tex_NRM",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "accessory_nrm.tga",
                    ),
                    (
                        "SpecularMap_Tex_LIN",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "accessory_srm.tga",
                    ),
                    (
                        "Translucent_Tex_LIN",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "accessory_trm.tga",
                    ),
                ],
                &[
                    ("MatParamIndex_0", [39.0, 0.0, 0.0, 0.0]),
                    (
                        "SelfColor",
                        // `0.764706015586853`/`0.16493700444698334` from the
                        // expected file, shortened to the same f32 values.
                        [0.764706, 0.164937, 0.164937, 1.0],
                    ),
                ],
            )],
            &[("MESH_highneck", None, true, &[0], true)],
            &[(204, 320, 0, 0, 0, &[0, 1, 2, 3, 4, 5, 6])],
        );
    }

    #[test]
    fn mouth_content() {
        check(
            MOUTH,
            &[
                ("sk_head", None, [-0.0, 0.108, 0.0209, 1.0]),
                ("skf_jaw", None, [-0.0, -0.0032, 0.0252, 1.0]),
                ("skf_cheek_s_l", None, [0.0479, 1.6215, 0.1396, 1.0]),
                ("skf_cheek_s_r", None, [-0.0479, 1.6215, 0.1396, 1.0]),
                ("skf_lip_s_l", None, [0.0236, 1.6297, 0.1615, 1.0]),
                ("skf_lip_s_r", None, [-0.0236, 1.6297, 0.1615, 1.0]),
            ],
            &[(
                "oral_mat",
                "fox_3ddf_translucent",
                "fox3DDF_Blin_Translucent_LNM",
                &[
                    (
                        "Base_Tex_SRGB",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "oral_bsm.tga",
                    ),
                    (
                        "NormalMap_Tex_NRM",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "oral_nrm.tga",
                    ),
                    (
                        "SpecularMap_Tex_LIN",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "oral_srm.tga",
                    ),
                    (
                        "Translucent_Tex_LIN",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "oral_trm.tga",
                    ),
                ],
                &[("MatParamIndex_0", [13.0, 0.0, 0.0, 0.0])],
            )],
            &[("MESH_mouth", None, true, &[0], true)],
            &[(325, 530, 0, 0, 0, &[0, 1, 2, 3, 4, 5])],
        );
    }

    #[test]
    fn au_low_content() {
        check(
            AU_LOW,
            &[
                ("sk_belly", None, [0.0, 0.0, 0.0, 1.0]),
                ("sk_chest", Some(0), [-0.0, 0.1667, 0.0138, 1.0]),
                ("sk_neck", Some(1), [-0.0, 0.2693, 0.0197, 1.0]),
                ("sk_shoulder_l", Some(1), [0.1051, 0.2043, 0.0197, 1.0]),
                ("sk_upperarm_l", Some(3), [0.0899, 0.0, 0.0, 1.0]),
                ("sk_forearm_l", Some(4), [0.2041, -0.2051, -0.0197, 1.0]),
                ("sk_hand_l", Some(5), [0.164, -0.145, 0.1902, 1.0]),
                ("sk_shoulder_r", Some(1), [-0.1051, 0.2043, 0.0197, 1.0]),
                ("sk_upperarm_r", Some(7), [-0.0899, 0.0, 0.0, 1.0]),
                ("sk_forearm_r", Some(8), [-0.2041, -0.2051, -0.0197, 1.0]),
                ("sk_hand_r", Some(9), [-0.164, -0.145, 0.1902, 1.0]),
                ("sk_root_hip", None, [0.0, 1.0961, 0.0, 1.0]),
                ("sk_thigh_l", Some(11), [0.09, -0.0321, 0.0713, 1.0]),
                ("sk_leg_l", Some(12), [0.0513, -0.4167, 0.0349, 1.0]),
                ("sk_thigh_r", Some(11), [-0.09, -0.0321, 0.0713, 1.0]),
                ("sk_leg_r", Some(14), [-0.0513, -0.4167, 0.0349, 1.0]),
            ],
            &[(
                "audi_low",
                "pes_3ddf_crowd",
                "pes3DDF_Instancing_Crowd_Low",
                &[],
                &[
                    ("MatParamIndex_0", [10.0, 0.0, 0.0, 0.0]),
                    ("ModelId", [0.0, 0.0, 0.0, 0.0]),
                    ("ShirtId", [0.0, 0.0, 0.0, 0.0]),
                    ("FaceId", [28.0, 0.0, 0.0, 0.0]),
                    ("ClothId", [0.0, 0.0, 0.0, 0.0]),
                    ("IsUniform", [0.0, 0.0, 0.0, 0.0]),
                    ("IsAway", [1.0, 0.0, 0.0, 0.0]),
                    ("IsNationalFlag", [0.0, 0.0, 0.0, 0.0]),
                    ("MufflerId", [0.0, 0.0, 0.0, 0.0]),
                    ("ColorId", [12.0, 0.0, 0.0, 0.0]),
                ],
            )],
            &[
                ("MESH_au_Low", None, true, &[], false),
                ("MESH_lod_04", Some(0), true, &[0], true),
                ("MESH_lod_06", Some(0), true, &[1], true),
                ("MESH_lod_05", Some(0), true, &[2], true),
                ("MESH_lod_07", Some(0), true, &[3], true),
            ],
            &[
                (
                    36,
                    26,
                    0,
                    0,
                    0,
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                ),
                (
                    31,
                    18,
                    0,
                    0,
                    0,
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                ),
                (
                    33,
                    22,
                    0,
                    0,
                    0,
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                ),
                (
                    31,
                    18,
                    0,
                    0,
                    0,
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                ),
            ],
        );
    }

    #[test]
    fn oral_content() {
        check(
            ORAL,
            &[("sk_head", None, [0.0, 0.0, 0.0, 0.0])],
            &[(
                "Material",
                "fox3ddf_blin",
                "fox3DDF_Blin",
                &[
                    (
                        "Base_Tex_SRGB",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "dummy_bsm.dds",
                    ),
                    (
                        "NormalMap_Tex_NRM",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "dummy_nrm.dds",
                    ),
                    (
                        "SpecularMap_Tex_LIN",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "dummy_srm.dds",
                    ),
                ],
                &[("MatParamIndex_0", [0.0, 0.0, 0.0, 0.0])],
            )],
            &[("MESH_oral", None, true, &[0], true)],
            &[(4, 2, 0, 128, 131, &[0])],
        );
    }

    #[test]
    fn placeholder_content() {
        check(
            PLACEHOLDER,
            &[],
            &[(
                "basic_fx",
                "fox3ddf_blin",
                "fox3DDF_Blin",
                &[
                    (
                        "Base_Tex_SRGB",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "dummy.tga",
                    ),
                    (
                        "NormalMap_Tex_NRM",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "dummy_nrm.tga",
                    ),
                    (
                        "SpecularMap_Tex_LIN",
                        "/Assets/pes16/model/character/common/sourceimages/",
                        "dummy_srm.tga",
                    ),
                ],
                &[("MatParamIndex_0", [0.0, 0.0, 0.0, 0.0])],
            )],
            &[("MESH_placeholder", None, true, &[0], true)],
            &[(3, 1, 0, 128, 1, &[])],
        );
    }

    #[test]
    fn extensions_flags() {
        let oral = model(ORAL);
        assert!(oral.extensions.antiblur);
        assert!(oral.extensions.vertex_loop_preservation);
        assert!(!oral.extensions.mesh_splitting);
        assert_eq!(oral.extensions.other, Vec::<String>::new());
        for bytes in [HIGHNECK, MOUTH, AU_LOW, PLACEHOLDER] {
            let model = model(bytes);
            assert_eq!(model.extensions, Extensions::default());
        }
    }

    #[test]
    fn bad_material_reference() {
        let mut file = FmdlFile::read(HIGHNECK).unwrap();
        file.meshes[0].material_instance_id = 9;
        assert!(matches!(
            Model::from_file(&file),
            Err(FmdlError::BadReference {
                what: "material instance",
                index: 9,
            })
        ));
    }

    #[test]
    fn bone_parent_cycle() {
        let mut file = FmdlFile::read(HIGHNECK).unwrap();
        file.bones[1].parent_bone_id = 1;
        assert!(matches!(
            Model::from_file(&file),
            Err(FmdlError::ParentCycle("bone"))
        ));
    }

    #[test]
    fn mesh_without_group() {
        let mut file = FmdlFile::read(HIGHNECK).unwrap();
        file.mesh_group_assignments.clear();
        assert!(matches!(
            Model::from_file(&file),
            Err(FmdlError::BadMeshGroupAssignment(_))
        ));
    }
}
