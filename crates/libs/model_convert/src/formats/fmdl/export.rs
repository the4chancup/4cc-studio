//! The IR → FMDL half: `ir_to_fmdl`.

use crate::affine::Affine;
use crate::ir::{BoundingBox, CanonicalModel, validate};
use crate::loss::{self, Finding, Subject};
use crate::materials::{MaterialFamily, to_fox};
use crate::skeletons;

use super::super::ConvertError;
use super::template_matrix;

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

/// The per-vertex weight quantization: the byte total `round(sum * 255)` is preserved; slots
/// in decreasing weight (ties by slot index) get `round(weight / remaining_weight *
/// remaining_total)` each, the last gets what is left.
pub(super) fn quantize_weights(weights: [f32; 4]) -> [u8; 4] {
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
pub(super) fn split_parents(bones: &[::fmdl::Bone]) -> Vec<Option<usize>> {
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
    // Recover the loops from the IR vertex order itself — the flag-gated `decode_model`
    // returns identity owners on a model built from the IR, losing an add-on's loops.
    let owners: Vec<Vec<usize>> = model
        .meshes
        .iter()
        .map(::fmdl::ops::vertex_enc::decode)
        .collect();
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
