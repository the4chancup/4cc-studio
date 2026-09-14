//! The FMDL importer and exporter: `fmdl_to_ir` decodes the format extensions, reads the bind
//! pose from the companion SKL or PES21's template tables, and lifts the per-mesh flags to the
//! materials; `ir_to_fmdl` resolves each material for Fox, quantizes weights, derives the bone
//! positions an import did not supply, and runs the format crate's encoders.

use std::collections::HashMap;

use pes_version::PesVersion;

use crate::affine::Affine;
use crate::ir::{
    Bone, BoundingBox, CanonicalModel, Material, Mesh, MeshGroup, SourceFormat, Texture, Vertices,
    validate,
};
use crate::loss::{self, Finding, Subject};
use crate::materials::{FoxMaterial, MaterialFamily, TextureRole, family, to_fox};
use crate::skeletons::{self, skeletons};

pub use super::Imported;
use super::{ConvertError, drop_out_of_group_slots};

/// The directory the game's dummy normal and specular maps live in (the same dummies the
/// anti-blur materials use).
const TEXTURE_DIRECTORY: &str = "/Assets/pes16/model/character/common/sourceimages/";

/// A model exported to FMDL: the companion SKL only when a bone unknown to the template
/// tables survives (a model on the standard skeleton gets none; the compiler injects the
/// template).
#[derive(Debug)]
pub struct ExportedFox {
    /// The FMDL model.
    pub model: ::fmdl::Model,
    /// The companion skeleton for bones outside PES21's template tables.
    pub skl: Option<::fmdl::SklFile>,
    /// The findings the export produced.
    pub findings: Vec<loss::Finding>,
}

/// The bind pose `name` carries in PES21's template tables, body first then face and hands.
fn template_matrix(name: &str) -> Option<Affine> {
    skeletons::version_bone(skeletons(PesVersion::Pes21), name).map(|bone| bone.matrix)
}

/// FMDL → IR. Decodes the format extensions (mesh splitting, anti-blur) on a copy, reads the
/// bind pose from `skl` (bones matched by name) or PES21's template tables, lifts the
/// per-mesh flags to the materials, and validates the result.
pub fn fmdl_to_ir(
    model: &::fmdl::Model,
    skl: Option<&::fmdl::SklFile>,
) -> Result<Imported, ConvertError> {
    let mut model = model.clone();
    // The legacy pipeline's order: reassemble split meshes first, then strip the anti-blur
    // duplicates.
    ::fmdl::ops::split::decode(&mut model)?;
    ::fmdl::ops::antiblur::decode(&mut model);
    let model = &model;

    let mut findings = Vec::new();

    let mut bones = Vec::with_capacity(model.bones.len());
    for (index, bone) in model.bones.iter().enumerate() {
        let matrix = skl
            .and_then(|skl| skl.bones.iter().find(|skl_bone| skl_bone.name == bone.name))
            .map(|skl_bone| {
                Affine::from_rotation_translation(skl_bone.rotation, skl_bone.translation)
            })
            .or_else(|| template_matrix(&bone.name))
            .unwrap_or_else(|| {
                findings.push(Finding {
                    code: "bone_matrix_unknown",
                    subject: Subject::Bone(index),
                    detail: bone.name.clone(),
                });
                Affine::IDENTITY
            });
        bones.push(Bone {
            name: bone.name.clone(),
            parent: bone.parent,
            matrix,
            global_position: Some(bone.world_position),
            local_position: Some(bone.local_position),
            bounding_box: Some(BoundingBox {
                min: bone.bounding_box.min,
                max: bone.bounding_box.max,
            }),
        });
    }

    // Every texture the materials reference, deduped in first-occurrence order.
    let mut textures = Vec::new();
    let mut texture_index: HashMap<(&str, &str), usize> = HashMap::new();
    for instance in &model.materials {
        for (_, texture) in &instance.textures {
            let key = (texture.directory.as_str(), texture.file_name.as_str());
            if let std::collections::hash_map::Entry::Vacant(entry) = texture_index.entry(key) {
                entry.insert(textures.len());
                textures.push(Texture {
                    directory: texture.directory.clone(),
                    file_name: texture.file_name.clone(),
                });
            }
        }
    }
    let texture_of = |texture: &::fmdl::Texture| -> usize {
        texture_index[&(texture.directory.as_str(), texture.file_name.as_str())]
    };

    // One IR material per (instance, flags) combination; the first keeps the instance's
    // name, the rest are `name_2`, `name_3`, ....
    let mut materials = Vec::new();
    // Per instance: the IR material index each flag combination maps to.
    let mut instance_materials: Vec<Vec<(u8, u8, bool, usize)>> = Vec::new();
    for (instance_index, instance) in model.materials.iter().enumerate() {
        let mut combinations: Vec<(u8, u8, bool)> = Vec::new();
        for mesh in &model.meshes {
            if mesh.material == instance_index {
                let flags = (
                    mesh.alpha_flags,
                    mesh.shadow_flags,
                    mesh.has_antiblur_meshes,
                );
                if !combinations.contains(&flags) {
                    combinations.push(flags);
                }
            }
        }
        if combinations.is_empty() {
            combinations.push((0, 0, false));
        }
        let mut entries = Vec::new();
        for (split, (alpha_flags, shadow_flags, antiblur_meshes)) in combinations.iter().enumerate()
        {
            let name = if split == 0 {
                instance.name.clone()
            } else {
                format!("{}_{}", instance.name, split + 1)
            };
            if split > 0 {
                findings.push(Finding {
                    code: "material_split_by_flags",
                    subject: Subject::Material(materials.len()),
                    detail: name.clone(),
                });
            }
            let inferred = family::from_fox_shader(&instance.shader);
            if inferred.approximate {
                findings.push(Finding {
                    code: "material_family_approximated",
                    subject: Subject::Material(materials.len()),
                    detail: name.clone(),
                });
            }
            let mut canonical: Vec<(TextureRole, usize)> = Vec::new();
            let mut native: Vec<(String, usize)> = Vec::new();
            let mut base_linear = false;
            for (sampler, texture) in &instance.textures {
                if sampler == "Base_Tex_LIN" {
                    base_linear = true;
                }
                match to_fox::role_for_sampler(sampler) {
                    // The first sampler a role gets wins; a second `Base_Tex_*` is native.
                    Some(role) if !canonical.iter().any(|(seen, _)| *seen == role) => {
                        canonical.push((role, texture_of(texture)));
                    }
                    _ => native.push((sampler.clone(), texture_of(texture))),
                }
            }
            materials.push(Material {
                name,
                family: inferred.family,
                two_sided: Some(alpha_flags & 32 != 0),
                transparent: None,
                antiblur: Some(*antiblur_meshes),
                textures: canonical,
                parameters: vec![],
                fox: Some(FoxMaterial {
                    shader: instance.shader.clone(),
                    technique: instance.technique.clone(),
                    alpha_flags: *alpha_flags,
                    shadow_flags: *shadow_flags,
                    cast_shadow: None,
                    invisible: None,
                    base_linear,
                    textures: native,
                    parameters: instance.parameters.clone(),
                }),
                prefox: None,
            });
            entries.push((
                *alpha_flags,
                *shadow_flags,
                *antiblur_meshes,
                materials.len() - 1,
            ));
        }
        instance_materials.push(entries);
    }

    let mut meshes = Vec::with_capacity(model.meshes.len());
    for (index, mesh) in model.meshes.iter().enumerate() {
        let vertices = &mesh.vertices;
        let mut bone_weights: Option<Vec<[f32; 4]>> = vertices.bone_weights.as_ref().map(|rows| {
            rows.iter()
                .map(|row| row.map(|w| w as f32 / 255.0))
                .collect()
        });
        let dropped = match (&vertices.bone_indices, bone_weights.as_mut()) {
            (Some(indices), Some(weights)) => {
                drop_out_of_group_slots(indices, weights, mesh.bone_group.len())
            }
            _ => 0,
        };
        if dropped > 0 {
            findings.push(Finding {
                code: "bone_slot_dropped",
                subject: Subject::Mesh(index),
                detail: dropped.to_string(),
            });
        }
        let material = instance_materials[mesh.material]
            .iter()
            .find(|(alpha, shadow, antiblur, _)| {
                *alpha == mesh.alpha_flags
                    && *shadow == mesh.shadow_flags
                    && *antiblur == mesh.has_antiblur_meshes
            })
            .map(|(_, _, _, material)| *material)
            .expect("every mesh's flag combination was collected");
        meshes.push(Mesh {
            vertices: Vertices {
                positions: vertices.positions.clone(),
                normals: vertices.normals.clone(),
                tangents: vertices.tangents.clone(),
                bitangents: None,
                colors: vertices.colors.clone(),
                uvs: vertices.uvs.clone(),
                uv_high_precision: vertices.uv_high_precision.clone(),
                bone_indices: vertices.bone_indices.clone(),
                bone_weights,
                bone_weight_width: None,
            },
            faces: mesh.faces.clone(),
            bone_group: mesh.bone_group.clone(),
            material,
            extension_headers: Default::default(),
            custom_bounding_box: mesh.custom_bounding_box.map(|b| BoundingBox {
                min: b.min,
                max: b.max,
            }),
        });
    }

    let ir = CanonicalModel {
        bones,
        meshes,
        mesh_groups: model
            .mesh_groups
            .iter()
            .map(|group| MeshGroup {
                name: group.name.clone(),
                parent: group.parent,
                meshes: group.meshes.clone(),
                visible: group.visible,
            })
            .collect(),
        materials,
        textures,
        extension_headers: model.extensions.other.iter().cloned().collect(),
        source_format: SourceFormat::Fox,
    };
    validate(&ir)?;
    Ok(Imported {
        model: ir,
        findings,
    })
}

/// The per-vertex weight quantization: the byte total `round(sum * 255)` is preserved; slots
/// in decreasing weight (ties by slot index) get `round(weight / remaining_weight *
/// remaining_total)` each, the last gets what is left.
fn quantize_weights(weights: [f32; 4]) -> [u8; 4] {
    let sum: f32 = weights.iter().sum();
    let mut total = (sum * 255.0).round() as i32;
    let mut remaining_weight = sum;
    let mut order = [0usize, 1, 2, 3];
    order.sort_by(|a, b| weights[*b].total_cmp(&weights[*a]).then(a.cmp(b)));
    let mut out = [0u8; 4];
    for (position, slot) in order.iter().enumerate() {
        let byte = if position == 3 {
            total.clamp(0, 255)
        } else if remaining_weight > 0.0 {
            (weights[*slot] / remaining_weight * total as f32).round() as i32
        } else {
            0
        }
        .clamp(0, 255);
        out[*slot] = byte as u8;
        remaining_weight -= weights[*slot];
        total -= byte;
    }
    out
}

/// The bounding box (w 1.0 on both corners) of every vertex with a positive weight on
/// `bone` across all meshes; `[0.0; 4]`/`[0.0; 4]` when none.
fn bone_bounding_box(ir: &CanonicalModel, bone: usize) -> BoundingBox {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    let mut any = false;
    for mesh in &ir.meshes {
        if let (Some(indices), Some(weights)) =
            (&mesh.vertices.bone_indices, &mesh.vertices.bone_weights)
        {
            for (vertex, (row, row_weights)) in indices.iter().zip(weights).enumerate() {
                for (slot, index) in row.iter().enumerate() {
                    if row_weights[slot] > 0.0
                        && mesh
                            .bone_group
                            .get(usize::from(*index))
                            .is_some_and(|b| *b == bone)
                    {
                        let position = mesh.vertices.positions[vertex];
                        for axis in 0..3 {
                            min[axis] = min[axis].min(position[axis]);
                            max[axis] = max[axis].max(position[axis]);
                        }
                        any = true;
                    }
                }
            }
        }
    }
    if any {
        BoundingBox {
            min: [min[0], min[1], min[2], 1.0],
            max: [max[0], max[1], max[2], 1.0],
        }
    } else {
        BoundingBox {
            min: [0.0; 4],
            max: [0.0; 4],
        }
    }
}

/// The parent list `split::encode` works on: the bone's own parent, else its `render_parent`
/// looked up in `bones` by name.
fn split_parents(bones: &[::fmdl::Bone]) -> Vec<Option<usize>> {
    bones
        .iter()
        .map(|bone| {
            bone.parent.or_else(|| {
                skeletons::render_parent(&bone.name)
                    .and_then(|parent| bones.iter().position(|b| b.name == parent))
            })
        })
        .collect()
}

/// IR → FMDL. Validates, resolves each material for Fox, quantizes weights, derives the bone
/// positions an import did not supply, then runs the format crate's encoders (anti-blur,
/// vertex loops, mesh splitting — the legacy pipeline's order).
pub fn ir_to_fmdl(ir: &CanonicalModel) -> Result<ExportedFox, ConvertError> {
    validate(ir)?;
    let mut findings = Vec::new();

    let resolved: Vec<to_fox::ResolvedFox> = ir.materials.iter().map(to_fox::resolve).collect();
    let mut materials = Vec::with_capacity(ir.materials.len());
    for (index, (material, resolved)) in ir.materials.iter().zip(&resolved).enumerate() {
        for role in &resolved.unused_roles {
            findings.push(Finding {
                code: "material_texture_unused",
                subject: Subject::Material(index),
                detail: format!("{role:?}"),
            });
        }
        let mut textures: Vec<(String, ::fmdl::Texture)> = resolved
            .textures
            .iter()
            .map(|(sampler, texture)| {
                let texture = &ir.textures[*texture];
                (
                    sampler.clone(),
                    ::fmdl::Texture {
                        file_name: texture.file_name.clone(),
                        directory: texture.directory.clone(),
                    },
                )
            })
            .collect();
        // A lit material derived from its family gets the game's dummy normal and specular
        // maps (the compiler's texture step replaces them when the folder has real maps); a
        // Fox-native material that lacks them is left as the game shipped it.
        if material.fox.is_none()
            && matches!(
                material.family,
                MaterialFamily::Shaded | MaterialFamily::Metal
            )
        {
            for (sampler, file_name) in [
                ("NormalMap_Tex_NRM", "dummy_nrm.dds"),
                ("SpecularMap_Tex_LIN", "dummy_srm.dds"),
            ] {
                if !textures.iter().any(|(name, _)| name == sampler) {
                    textures.push((
                        sampler.to_string(),
                        ::fmdl::Texture {
                            file_name: file_name.to_string(),
                            directory: TEXTURE_DIRECTORY.to_string(),
                        },
                    ));
                    findings.push(Finding {
                        code: "dummy_texture_added",
                        subject: Subject::Material(index),
                        detail: sampler.to_string(),
                    });
                }
            }
        }
        materials.push(::fmdl::MaterialInstance {
            name: material.name.clone(),
            shader: resolved.shader.clone(),
            technique: resolved.technique.clone(),
            textures,
            parameters: resolved.parameters.clone(),
        });
    }

    let mut bones: Vec<::fmdl::Bone> = Vec::with_capacity(ir.bones.len());
    for (index, bone) in ir.bones.iter().enumerate() {
        let translation = bone.matrix.translation();
        let world =
            bone.global_position
                .unwrap_or([translation[0], translation[1], translation[2], 1.0]);
        let local = bone.local_position.unwrap_or_else(|| match bone.parent {
            Some(parent) => {
                let parent_world = bones[parent].world_position;
                [
                    world[0] - parent_world[0],
                    world[1] - parent_world[1],
                    world[2] - parent_world[2],
                    1.0,
                ]
            }
            None => [0.0, 0.0, 0.0, 1.0],
        });
        bones.push(::fmdl::Bone {
            name: bone.name.clone(),
            parent: bone.parent,
            bounding_box: bone
                .bounding_box
                .map(|b| ::fmdl::BoundingBox {
                    min: b.min,
                    max: b.max,
                })
                .unwrap_or_else(|| {
                    let b = bone_bounding_box(ir, index);
                    ::fmdl::BoundingBox {
                        min: b.min,
                        max: b.max,
                    }
                }),
            local_position: local,
            world_position: world,
        });
    }

    // FMDL binds every vertex to a bone: an unskinned mesh gets one `static` bone (the
    // legacy converter's values — the name no Konami skeleton has, so the game leaves the
    // mesh where it is placed) and is weighted fully to it.
    let static_bone = ir
        .meshes
        .iter()
        .any(|mesh| mesh.vertices.bone_indices.is_none())
        .then(|| {
            bones.push(::fmdl::Bone {
                name: "static".to_string(),
                parent: None,
                bounding_box: ::fmdl::BoundingBox {
                    min: [0.0; 4],
                    max: [0.0; 4],
                },
                local_position: [0.0; 4],
                world_position: [0.2, 0.0, 0.0, 1.0],
            });
            bones.len() - 1
        });

    let mut meshes = Vec::with_capacity(ir.meshes.len());
    for (index, mesh) in ir.meshes.iter().enumerate() {
        if mesh.vertices.bitangents.is_some() {
            findings.push(Finding {
                code: "vertex_bitangents_dropped",
                subject: Subject::Mesh(index),
                detail: String::new(),
            });
        }
        let material = &resolved[mesh.material];
        let (bone_weights, bone_indices, bone_group) =
            match (static_bone, &mesh.vertices.bone_indices) {
                (Some(static_bone), None) => {
                    findings.push(Finding {
                        code: "static_bone_added",
                        subject: Subject::Mesh(index),
                        detail: String::new(),
                    });
                    (
                        Some(vec![[255, 0, 0, 0]; mesh.vertices.len()]),
                        Some(vec![[0, 0, 0, 0]; mesh.vertices.len()]),
                        vec![static_bone],
                    )
                }
                _ => (
                    mesh.vertices
                        .bone_weights
                        .as_ref()
                        .map(|rows| rows.iter().map(|row| quantize_weights(*row)).collect()),
                    mesh.vertices.bone_indices.clone(),
                    mesh.bone_group.clone(),
                ),
            };
        meshes.push(::fmdl::Mesh {
            vertices: ::fmdl::format::MeshVertices {
                positions: mesh.vertices.positions.clone(),
                normals: mesh.vertices.normals.clone(),
                tangents: mesh.vertices.tangents.clone(),
                colors: mesh.vertices.colors.clone(),
                uvs: mesh.vertices.uvs.clone(),
                uv_high_precision: mesh.vertices.uv_high_precision.clone(),
                bone_weights,
                bone_indices,
            },
            faces: mesh.faces.clone(),
            bone_group,
            material: mesh.material,
            alpha_flags: material.alpha_flags,
            shadow_flags: material.shadow_flags,
            has_antiblur_meshes: material.antiblur,
            is_antiblur_mesh: false,
            custom_bounding_box: mesh.custom_bounding_box.map(|b| ::fmdl::BoundingBox {
                min: b.min,
                max: b.max,
            }),
        });
    }

    let mut model = ::fmdl::Model {
        bones,
        materials,
        meshes,
        mesh_groups: ir
            .mesh_groups
            .iter()
            .map(|group| ::fmdl::MeshGroup {
                name: group.name.clone(),
                parent: group.parent,
                meshes: group.meshes.clone(),
                bounding_box: None,
                visible: group.visible,
                split_mesh_group: false,
            })
            .collect(),
        extensions: ::fmdl::Extensions {
            mesh_splitting: false,
            antiblur: false,
            vertex_loop_preservation: false,
            other: ir.extension_headers.iter().cloned().collect(),
        },
        bone_matrices: None,
    };

    // The encoders in the legacy pipeline's order (model2fmdl.py): anti-blur duplicates,
    // then the vertex-loop convention, then mesh splitting.
    ::fmdl::ops::antiblur::encode(&mut model);
    let owners = ::fmdl::ops::vertex_enc::decode_model(&model);
    ::fmdl::ops::vertex_enc::encode_model(&mut model, &owners)?;
    let parents = split_parents(&model.bones);
    ::fmdl::ops::split::encode(&mut model, Some(&parents))?;

    // The SKL covers every exported bone the template lacks, from the IR matrices by name
    // (identity for `static`, which has no IR bone).
    let unknown = |bone: &::fmdl::Bone| template_matrix(&bone.name).is_none();
    let skl = model.bones.iter().any(unknown).then(|| ::fmdl::SklFile {
        bones: model
            .bones
            .iter()
            .map(|bone| {
                let matrix = ir
                    .bones
                    .iter()
                    .find(|b| b.name == bone.name)
                    .map(|b| b.matrix)
                    .unwrap_or(Affine::IDENTITY);
                ::fmdl::format::SklBone {
                    name: bone.name.clone(),
                    parent: bone.parent,
                    rotation: matrix.rotation(),
                    translation: matrix.translation(),
                }
            })
            .collect(),
    });

    Ok(ExportedFox {
        model,
        skl,
        findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::TextureRole;

    const HIGHNECK: &[u8] = include_bytes!("../../tests/fixtures/konami_highneck.fmdl");
    const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
    const AU: &[u8] = include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl");
    const AU_SKL: &[u8] = include_bytes!("../../tests/fixtures/konami_au00.skl");

    fn read_fmdl(bytes: &[u8]) -> ::fmdl::Model {
        ::fmdl::Model::from_file(&::fmdl::FmdlFile::read(bytes).expect("parse")).expect("model")
    }

    /// `fmdl_to_ir`'s decode half: the whole-mesh form of `model`.
    fn decoded(model: &::fmdl::Model) -> ::fmdl::Model {
        let mut model = model.clone();
        ::fmdl::ops::split::decode(&mut model).expect("split decode");
        ::fmdl::ops::antiblur::decode(&mut model);
        model
    }

    /// `ir_to_fmdl`'s encoder half, run on a decoded model for comparison.
    fn encoded(decoded: &::fmdl::Model) -> ::fmdl::Model {
        let mut model = decoded.clone();
        ::fmdl::ops::antiblur::encode(&mut model);
        let owners = ::fmdl::ops::vertex_enc::decode_model(&model);
        ::fmdl::ops::vertex_enc::encode_model(&mut model, &owners).expect("vertex encode");
        let parents = split_parents(&model.bones);
        ::fmdl::ops::split::encode(&mut model, Some(&parents)).expect("split encode");
        model
    }

    /// A one-bone, one-mesh `fmdl::Model` for the synthetic cases.
    fn fmdl_model(mesh: ::fmdl::Mesh, materials: Vec<::fmdl::MaterialInstance>) -> ::fmdl::Model {
        ::fmdl::Model {
            bones: vec![::fmdl::Bone {
                name: "sk_belly".to_string(),
                parent: None,
                bounding_box: ::fmdl::BoundingBox {
                    min: [0.0; 4],
                    max: [0.0; 4],
                },
                local_position: [0.0; 4],
                world_position: [0.0; 4],
            }],
            materials,
            meshes: vec![mesh],
            mesh_groups: vec![::fmdl::MeshGroup {
                name: "group".to_string(),
                parent: None,
                meshes: vec![0],
                bounding_box: None,
                visible: true,
                split_mesh_group: false,
            }],
            extensions: ::fmdl::Extensions::default(),
            bone_matrices: None,
        }
    }

    fn instance(name: &str) -> ::fmdl::MaterialInstance {
        ::fmdl::MaterialInstance {
            name: name.to_string(),
            shader: "fox3ddf_blin".to_string(),
            technique: "fox3DDF_Blin".to_string(),
            textures: vec![],
            parameters: vec![],
        }
    }

    /// A minimal consistent IR: one unskinned mesh, one `Shaded` material, one group.
    fn minimal_ir() -> CanonicalModel {
        CanonicalModel {
            bones: vec![],
            meshes: vec![Mesh {
                vertices: Vertices {
                    positions: vec![[0.0; 3]],
                    ..Vertices::default()
                },
                faces: vec![[0, 0, 0]],
                bone_group: vec![],
                material: 0,
                extension_headers: Default::default(),
                custom_bounding_box: None,
            }],
            mesh_groups: vec![MeshGroup {
                name: "group".to_string(),
                parent: None,
                meshes: vec![0],
                visible: true,
            }],
            materials: vec![Material {
                name: "mat".to_string(),
                family: MaterialFamily::Shaded,
                two_sided: None,
                transparent: None,
                antiblur: None,
                textures: vec![],
                parameters: vec![],
                fox: None,
                prefox: None,
            }],
            textures: vec![],
            extension_headers: Default::default(),
            source_format: SourceFormat::PreFox,
        }
    }

    #[test]
    fn highneck_imports_on_the_template_skeleton() {
        let imported = fmdl_to_ir(&read_fmdl(HIGHNECK), None).expect("import");
        let model = &imported.model;
        assert_eq!(model.bones.len(), 7);
        assert_eq!(model.meshes.len(), 1);
        assert_eq!(model.materials.len(), 1);
        let body = &skeletons(PesVersion::Pes21).body;
        for bone in &model.bones {
            let template = body.bone(&bone.name).expect("template bone");
            assert!(
                bone.matrix.max_component_delta(&template.matrix) < 1e-6,
                "{}",
                bone.name
            );
        }
        let material = &model.materials[0];
        assert_eq!(material.two_sided, Some(false));
        assert_eq!(
            material.fox.as_ref().expect("fox").shader,
            "pes_3ddf_basic_color_translucent"
        );
        assert_eq!(imported.findings, Vec::<Finding>::new());
    }

    #[test]
    fn audience_import_reads_the_skl() {
        let input = read_fmdl(AU);
        let skl = ::fmdl::SklFile::read(AU_SKL).expect("skl");
        let imported = fmdl_to_ir(&input, Some(&skl)).expect("import");
        let ir_hand = imported
            .model
            .bones
            .iter()
            .find(|bone| bone.name == "sk_hand_l")
            .expect("bone");
        let skl_hand = skl
            .bones
            .iter()
            .find(|bone| bone.name == "sk_hand_l")
            .expect("skl bone");
        assert_eq!(ir_hand.matrix.translation(), skl_hand.translation);
        // The SKL pose is not the template's: `sk_hand_l` differs by ~0.50.
        let template = skeletons(PesVersion::Pes21)
            .body
            .bone("sk_hand_l")
            .expect("template bone");
        assert!(ir_hand.matrix.max_component_delta(&template.matrix) > 1e-3);
        // The fixture's own convention the exporter relies on: every child's
        // `local_position` is `global - parent.global`.
        for bone in &input.bones {
            let Some(parent) = bone.parent else { continue };
            for axis in 0..3 {
                let delta = bone.local_position[axis]
                    - (bone.world_position[axis] - input.bones[parent].world_position[axis]);
                assert!(delta.abs() < 1e-5, "{} axis {axis}", bone.name);
            }
        }
    }

    #[test]
    fn fmdl_round_trip() {
        let skl = ::fmdl::SklFile::read(AU_SKL).expect("skl");
        for (input, skl, has_unknown_bone) in [
            (read_fmdl(HIGHNECK), None, false),
            (read_fmdl(ORAL), None, false),
            (read_fmdl(AU), Some(&skl), true),
        ] {
            let imported = fmdl_to_ir(&input, skl).expect("import");
            let exported = ir_to_fmdl(&imported.model).expect("export");
            let expected = encoded(&decoded(&input));
            assert_eq!(exported.model.bones, expected.bones);
            assert_eq!(exported.model.materials, expected.materials);
            assert_eq!(exported.model.meshes, expected.meshes);
            assert_eq!(exported.model.extensions, expected.extensions);
            assert_eq!(exported.model.mesh_groups.len(), expected.mesh_groups.len());
            for (got, want) in exported.model.mesh_groups.iter().zip(&expected.mesh_groups) {
                assert_eq!(
                    (
                        &got.name,
                        got.parent,
                        &got.meshes,
                        got.visible,
                        got.split_mesh_group
                    ),
                    (
                        &want.name,
                        want.parent,
                        &want.meshes,
                        want.visible,
                        want.split_mesh_group
                    )
                );
            }
            assert_eq!(exported.skl.is_some(), has_unknown_bone);
            assert_eq!(exported.findings, Vec::<Finding>::new());
        }
    }

    #[test]
    fn ir_is_a_fixed_point_after_one_encode() {
        for bytes in [HIGHNECK, ORAL] {
            let ir1 = fmdl_to_ir(&read_fmdl(bytes), None).expect("import").model;
            let ir2 = fmdl_to_ir(&ir_to_fmdl(&ir1).expect("export").model, None)
                .expect("reimport")
                .model;
            let ir3 = fmdl_to_ir(&ir_to_fmdl(&ir2).expect("export").model, None)
                .expect("reimport")
                .model;
            assert_eq!(ir2, ir3);
            assert_eq!(ir1.bones, ir2.bones);
            assert_eq!(ir1.materials, ir2.materials);
            assert_eq!(ir1.textures, ir2.textures);
            assert_eq!(ir1.mesh_groups, ir2.mesh_groups);
            assert_eq!(ir1.extension_headers, ir2.extension_headers);
            for (before, after) in ir1.meshes.iter().zip(&ir2.meshes) {
                assert_eq!(
                    before.vertices.positions.len(),
                    after.vertices.positions.len()
                );
                assert_eq!(before.faces.len(), after.faces.len());
                assert_eq!(before.bone_group, after.bone_group);
                assert_eq!(before.material, after.material);
            }
        }
    }

    #[test]
    fn a_bone_slot_past_the_group_is_dropped() {
        let mesh = ::fmdl::Mesh {
            vertices: ::fmdl::format::MeshVertices {
                positions: vec![[0.0; 3]],
                bone_indices: Some(vec![[0, 3, 0, 0]]),
                bone_weights: Some(vec![[128, 127, 0, 0]]),
                ..::fmdl::format::MeshVertices::default()
            },
            faces: vec![[0, 0, 0]],
            bone_group: vec![0],
            material: 0,
            alpha_flags: 0,
            shadow_flags: 0,
            has_antiblur_meshes: false,
            is_antiblur_mesh: false,
            custom_bounding_box: None,
        };
        let imported = fmdl_to_ir(&fmdl_model(mesh, vec![instance("mat")]), None).expect("import");
        assert_eq!(
            imported.model.meshes[0].vertices.bone_weights,
            Some(vec![[1.0, 0.0, 0.0, 0.0]])
        );
        assert_eq!(
            imported.findings,
            vec![Finding {
                code: "bone_slot_dropped",
                subject: Subject::Mesh(0),
                detail: "1".to_string(),
            }]
        );
    }

    #[test]
    fn missing_maps_get_the_game_dummies() {
        let mut ir = minimal_ir();
        ir.textures = vec![Texture {
            directory: "./".to_string(),
            file_name: "t.dds".to_string(),
        }];
        ir.materials[0].textures = vec![(TextureRole::Base, 0)];
        ir.meshes[0].vertices.bitangents = Some(vec![[0.0; 3]]);
        let exported = ir_to_fmdl(&ir).expect("export");
        let textures = &exported.model.materials[0].textures;
        assert_eq!(
            textures
                .iter()
                .map(|(sampler, texture)| (sampler.as_str(), texture.file_name.as_str()))
                .collect::<Vec<_>>(),
            [
                ("Base_Tex_SRGB", "t.dds"),
                ("NormalMap_Tex_NRM", "dummy_nrm.dds"),
                ("SpecularMap_Tex_LIN", "dummy_srm.dds"),
            ]
        );
        assert_eq!(
            exported.findings,
            vec![
                Finding {
                    code: "dummy_texture_added",
                    subject: Subject::Material(0),
                    detail: "NormalMap_Tex_NRM".to_string(),
                },
                Finding {
                    code: "dummy_texture_added",
                    subject: Subject::Material(0),
                    detail: "SpecularMap_Tex_LIN".to_string(),
                },
                Finding {
                    code: "vertex_bitangents_dropped",
                    subject: Subject::Mesh(0),
                    detail: String::new(),
                },
                Finding {
                    code: "static_bone_added",
                    subject: Subject::Mesh(0),
                    detail: String::new(),
                },
            ]
        );
    }

    #[test]
    fn one_instance_two_flag_sets_splits() {
        let mesh = |alpha| ::fmdl::Mesh {
            vertices: ::fmdl::format::MeshVertices {
                positions: vec![[0.0; 3]],
                ..::fmdl::format::MeshVertices::default()
            },
            faces: vec![[0, 0, 0]],
            bone_group: vec![],
            material: 0,
            alpha_flags: alpha,
            shadow_flags: 0,
            has_antiblur_meshes: false,
            is_antiblur_mesh: false,
            custom_bounding_box: None,
        };
        let mut model = fmdl_model(mesh(0), vec![instance("mat")]);
        model.meshes.push(mesh(32));
        model.mesh_groups[0].meshes.push(1);
        let imported = fmdl_to_ir(&model, None).expect("import");
        assert_eq!(imported.model.materials.len(), 2);
        assert_eq!(imported.model.materials[0].name, "mat");
        assert_eq!(imported.model.materials[0].two_sided, Some(false));
        assert_eq!(imported.model.materials[1].name, "mat_2");
        assert_eq!(imported.model.materials[1].two_sided, Some(true));
        assert_eq!(
            imported.findings,
            vec![Finding {
                code: "material_split_by_flags",
                subject: Subject::Material(1),
                detail: "mat_2".to_string(),
            }]
        );
    }

    #[test]
    fn quantization_preserves_the_total() {
        assert_eq!(quantize_weights([0.5, 0.5, 0.0, 0.0]), [128, 127, 0, 0]);
        let weights = quantize_weights([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 0.0]);
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 255);
    }
}
