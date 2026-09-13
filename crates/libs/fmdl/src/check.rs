//! Model lint: rules that flag things known to be wrong (or suspicious)
//! in a loaded `Model`. Structural corruption is already an error at
//! `Model::from_file`; `check` looks past it at semantics the game cares
//! about. Returns findings with stable codes — consuming tools map codes
//! to their message catalog.
//!
//! Rules, in firing order:
//!
//! - `fmdl_mesh_over_bone_limit` / `fmdl_mesh_over_vertex_limit` /
//!   `fmdl_mesh_over_face_limit` (Error): the mesh exceeds a hard FMDL
//!   limit; the game cannot load it. `ops::split::encode` is the fix.
//! - `fmdl_face_index_out_of_range` (Error): a face references a vertex
//!   that does not exist; the mesh reads out of bounds.
//! - `fmdl_bone_slot_out_of_range` (Warning): a weighted bone index
//!   points past the end of the mesh's bone group; the game ignores that
//!   weight, so the vertex is under-skinned. Real Konami models do this.
//! - `fmdl_weights_not_normalized` (Info): a vertex's four weights sum to
//!   neither 255 nor 0; the skinning result is off by that fraction.
//! - `fmdl_mesh_empty` (Warning): a mesh with no faces renders nothing.
//! - `fmdl_material_unused` (Info): a material instance no mesh uses.
//! - `fmdl_duplicate_bone_name` (Warning): a second bone with an already
//!   used name; name-based lookups (merges, skl matching) get ambiguous.
//! - `fmdl_mesh_unassigned` (Error): no mesh group lists the mesh, so the
//!   file would not contain it at all.

use std::collections::HashSet;

use crate::model::Model;
use crate::ops::split::{BONE_LIMIT_HARD, FACE_LIMIT_HARD, VERTEX_LIMIT_HARD};

/// How bad a finding is for the model in game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth knowing; harmless.
    Info,
    /// Suspicious; the game tolerates it but a modder should look.
    Warning,
    /// The game cannot load or renders wrongly.
    Error,
}

/// What a finding is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// The model as a whole.
    Model,
    /// The mesh at this index.
    Mesh(usize),
    /// The material at this index.
    Material(usize),
    /// The bone at this index.
    Bone(usize),
}

/// One rule that fired; the consuming tool maps `code` to its message
/// catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The stable finding code.
    pub code: &'static str,
    /// Its severity.
    pub severity: Severity,
    /// What it is about.
    pub subject: Subject,
    /// How many items (faces, vertices, bones) tripped the rule.
    pub count: usize,
}

/// Checks `model` for things known wrong or suspicious, returning
/// findings in model order (meshes, then materials, then bones).
pub fn check(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    let assigned: HashSet<usize> = model
        .mesh_groups
        .iter()
        .flat_map(|group| group.meshes.iter().copied())
        .collect();
    let mut used_materials = HashSet::new();

    for (index, mesh) in model.meshes.iter().enumerate() {
        let vertex_count = mesh.vertices.positions.len();
        used_materials.insert(mesh.material);
        let subject = Subject::Mesh(index);

        // The game cannot load a mesh over a hard limit; splitting with
        // `ops::split::encode` is the fix.
        if mesh.bone_group.len() > BONE_LIMIT_HARD {
            findings.push(Finding {
                code: "fmdl_mesh_over_bone_limit",
                severity: Severity::Error,
                subject: subject.clone(),
                count: mesh.bone_group.len(),
            });
        }
        if vertex_count > VERTEX_LIMIT_HARD {
            findings.push(Finding {
                code: "fmdl_mesh_over_vertex_limit",
                severity: Severity::Error,
                subject: subject.clone(),
                count: vertex_count,
            });
        }
        if mesh.faces.len() > FACE_LIMIT_HARD {
            findings.push(Finding {
                code: "fmdl_mesh_over_face_limit",
                severity: Severity::Error,
                subject: subject.clone(),
                count: mesh.faces.len(),
            });
        }

        let out_of_range = mesh
            .faces
            .iter()
            .filter(|face| {
                face.iter()
                    .any(|&vertex| usize::from(vertex) >= vertex_count)
            })
            .count();
        if out_of_range > 0 {
            findings.push(Finding {
                code: "fmdl_face_index_out_of_range",
                severity: Severity::Error,
                subject: subject.clone(),
                count: out_of_range,
            });
        }

        if let (Some(weights), Some(indices)) =
            (&mesh.vertices.bone_weights, &mesh.vertices.bone_indices)
        {
            // Real Konami models carry bone slots past the end of the
            // group; the game ignores a weight whose bone does not exist.
            let bad_slots = indices
                .iter()
                .zip(weights)
                .filter(|(index_row, weight_row)| {
                    (0..4).any(|component| {
                        usize::from(index_row[component]) >= mesh.bone_group.len()
                            && weight_row[component] > 0
                    })
                })
                .count();
            if bad_slots > 0 {
                findings.push(Finding {
                    code: "fmdl_bone_slot_out_of_range",
                    severity: Severity::Warning,
                    subject: subject.clone(),
                    count: bad_slots,
                });
            }
            let unnormalized = weights
                .iter()
                .filter(|row| {
                    let sum: usize = row.iter().map(|&w| usize::from(w)).sum();
                    sum != 255 && sum != 0
                })
                .count();
            if unnormalized > 0 {
                findings.push(Finding {
                    code: "fmdl_weights_not_normalized",
                    severity: Severity::Info,
                    subject: subject.clone(),
                    count: unnormalized,
                });
            }
        }

        if mesh.faces.is_empty() {
            findings.push(Finding {
                code: "fmdl_mesh_empty",
                severity: Severity::Warning,
                subject: subject.clone(),
                count: 0,
            });
        }
        if !assigned.contains(&index) {
            findings.push(Finding {
                code: "fmdl_mesh_unassigned",
                severity: Severity::Error,
                subject,
                count: 0,
            });
        }
    }

    for (index, _) in model.materials.iter().enumerate() {
        if !used_materials.contains(&index) {
            findings.push(Finding {
                code: "fmdl_material_unused",
                severity: Severity::Info,
                subject: Subject::Material(index),
                count: 0,
            });
        }
    }

    let mut seen = HashSet::new();
    for (index, bone) in model.bones.iter().enumerate() {
        if !seen.insert(bone.name.as_str()) {
            findings.push(Finding {
                code: "fmdl_duplicate_bone_name",
                severity: Severity::Warning,
                subject: Subject::Bone(index),
                count: 0,
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FmdlFile;
    use crate::format::MeshVertices;
    use crate::model::Extensions;
    use crate::model::{Bone, BoundingBox, MaterialInstance, Mesh, MeshGroup};

    const HIGHNECK: &[u8] = include_bytes!("../tests/fixtures/konami_highneck.fmdl");
    const MOUTH: &[u8] = include_bytes!("../tests/fixtures/konami_mouth.fmdl");
    const AU_LOW: &[u8] = include_bytes!("../tests/fixtures/konami_au_Low_parts.fmdl");
    const ORAL: &[u8] = include_bytes!("../tests/fixtures/addon_oral.fmdl");
    const PLACEHOLDER: &[u8] = include_bytes!("../tests/fixtures/addon_placeholder.fmdl");
    const FIXTURES: [&[u8]; 5] = [HIGHNECK, MOUTH, AU_LOW, ORAL, PLACEHOLDER];

    fn model(bytes: &[u8]) -> Model {
        Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap()
    }

    fn small_mesh() -> Mesh {
        Mesh {
            vertices: MeshVertices {
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: None,
                tangents: None,
                colors: None,
                uvs: Vec::new(),
                uv_high_precision: Vec::new(),
                bone_weights: None,
                bone_indices: None,
            },
            faces: vec![[0, 1, 2]],
            bone_group: Vec::new(),
            material: 0,
            alpha_flags: 0,
            shadow_flags: 0,
            has_antiblur_meshes: false,
            is_antiblur_mesh: false,
            custom_bounding_box: None,
        }
    }

    fn small_model(mesh: Mesh) -> Model {
        Model {
            bones: Vec::new(),
            materials: vec![MaterialInstance {
                name: "Material".to_owned(),
                shader: "fox3ddf_blin".to_owned(),
                technique: "fox3DDF_Blin".to_owned(),
                textures: Vec::new(),
                parameters: Vec::new(),
            }],
            meshes: vec![mesh],
            mesh_groups: vec![MeshGroup {
                name: "group".to_owned(),
                parent: None,
                meshes: vec![0],
                bounding_box: None,
                visible: true,
                split_mesh_group: false,
            }],
            extensions: Extensions::default(),
            bone_matrices: None,
        }
    }

    #[test]
    fn fixtures_have_no_errors() {
        for bytes in FIXTURES {
            let findings = check(&model(bytes));
            assert!(
                findings
                    .iter()
                    .all(|finding| finding.severity < Severity::Error)
            );
        }
    }

    #[test]
    fn highneck_findings() {
        let findings = check(&model(HIGHNECK));
        assert!(findings.is_empty());
    }

    #[test]
    fn over_bone_limit() {
        let mut mesh = small_mesh();
        mesh.bone_group = (0..33).collect();
        let findings = check(&small_model(mesh));
        assert!(findings.contains(&Finding {
            code: "fmdl_mesh_over_bone_limit",
            severity: Severity::Error,
            subject: Subject::Mesh(0),
            count: 33,
        }));
    }

    #[test]
    fn face_index_out_of_range() {
        let mut mesh = small_mesh();
        mesh.faces.push([0, 1, 9]);
        let findings = check(&small_model(mesh));
        assert!(findings.contains(&Finding {
            code: "fmdl_face_index_out_of_range",
            severity: Severity::Error,
            subject: Subject::Mesh(0),
            count: 1,
        }));
    }

    #[test]
    fn mesh_unassigned() {
        let mut model = small_model(small_mesh());
        model.mesh_groups[0].meshes.clear();
        let findings = check(&model);
        assert!(findings.contains(&Finding {
            code: "fmdl_mesh_unassigned",
            severity: Severity::Error,
            subject: Subject::Mesh(0),
            count: 0,
        }));
    }

    #[test]
    fn unused_material() {
        let mut model = small_model(small_mesh());
        model.materials.push(model.materials[0].clone());
        let findings = check(&model);
        assert!(findings.contains(&Finding {
            code: "fmdl_material_unused",
            severity: Severity::Info,
            subject: Subject::Material(1),
            count: 0,
        }));
    }

    #[test]
    fn duplicate_bone_name() {
        let mut model = small_model(small_mesh());
        let bone = |name: &str| Bone {
            name: name.to_owned(),
            parent: None,
            bounding_box: BoundingBox {
                max: [0.0; 4],
                min: [0.0; 4],
            },
            local_position: [0.0; 4],
            world_position: [0.0; 4],
        };
        model.bones = vec![bone("sk_head"), bone("sk_neck"), bone("sk_head")];
        let findings = check(&model);
        assert!(findings.contains(&Finding {
            code: "fmdl_duplicate_bone_name",
            severity: Severity::Warning,
            subject: Subject::Bone(2),
            count: 0,
        }));
    }
}
