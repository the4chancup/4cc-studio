use super::export::{quantize_weights, split_parents};
use super::*;

use crate::affine::Affine;
use crate::formats::ConvertError;
use crate::ir::{Bone, CanonicalModel, Material, Mesh, MeshGroup, SourceFormat, Texture, Vertices};
use crate::loss::{Finding, Subject};
use crate::materials::MaterialFamily;

use crate::materials::TextureRole;

const HIGHNECK: &[u8] = include_bytes!("../../../tests/fixtures/konami_highneck.fmdl");
const HEAD_HI: &[u8] =
    include_bytes!("../../../../pes_model/tests/fixtures/konami_headHi.wesys.model");
const HEAD_HI_MTL: &[u8] = include_bytes!("../../../../pes_model/tests/fixtures/konami_headHi.mtl");
const ORAL: &[u8] = include_bytes!("../../../tests/fixtures/addon_oral.fmdl");
const AU: &[u8] = include_bytes!("../../../tests/fixtures/konami_au_Low_parts.fmdl");
const AU_SKL: &[u8] = include_bytes!("../../../tests/fixtures/konami_au00.skl");

fn read_fmdl(bytes: &[u8]) -> ::fmdl::Model {
    ::fmdl::Model::from_file(&::fmdl::FmdlFile::read(bytes).expect("parse")).expect("model")
}

/// `fmdl_to_ir`'s decode half: the whole-mesh form of `model`.
fn decoded(model: &::fmdl::Model) -> ::fmdl::Model {
    let mut model = model.clone();
    ::fmdl::ops::split::decode(&mut model).expect("split decode");
    ::fmdl::ops::antiblur::decode(&mut model).expect("antiblur decode");
    model
}

/// `ir_to_fmdl`'s encoder half, run on a decoded model for comparison.
fn encoded(decoded: &::fmdl::Model) -> ::fmdl::Model {
    let mut model = decoded.clone();
    ::fmdl::ops::antiblur::encode(&mut model).expect("antiblur encode");
    let owners: Vec<Vec<usize>> = model
        .meshes
        .iter()
        .map(::fmdl::ops::vertex_enc::decode)
        .collect::<Result<Vec<_>, _>>()
        .expect("vertex decode");
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
    // The file carries the redundant local-space bone-matrix block, and the
    // `translucent` shader is one of the rare names with no family.
    assert_eq!(
        imported.findings,
        vec![
            Finding {
                code: "native_field_dropped",
                subject: Subject::Model,
                detail: "bone_matrices".to_string(),
            },
            Finding {
                code: "material_family_approximated",
                subject: Subject::Material(0),
                detail: "accessory".to_string(),
            },
        ]
    );
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
fn an_exported_skl_carries_each_bones_ir_matrix() {
    let skl = ::fmdl::SklFile::read(AU_SKL).expect("skl");
    let imported = fmdl_to_ir(&read_fmdl(AU), Some(&skl)).expect("import");
    let exported = ir_to_fmdl(&imported.model).expect("export");
    let exported_skl = exported.skl.expect("skl");
    for bone in &exported_skl.bones {
        let matrix = imported
            .model
            .bones
            .iter()
            .find(|b| b.name == bone.name)
            .map(|b| b.matrix)
            .unwrap_or(Affine::IDENTITY);
        assert_eq!(bone.rotation, matrix.rotation(), "bone {}", bone.name);
        assert_eq!(bone.translation, matrix.translation(), "bone {}", bone.name);
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
fn a_zero_weight_slot_past_the_group_is_kept() {
    // Only a weighted stale slot drops; a zero-weight one is inert and the
    // vertex keeps the file's exact (unnormalized) weights.
    let mesh = ::fmdl::Mesh {
        vertices: ::fmdl::format::MeshVertices {
            positions: vec![[0.0; 3]],
            bone_indices: Some(vec![[0, 3, 0, 0]]),
            bone_weights: Some(vec![[128, 0, 0, 0]]),
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
        Some(vec![[128.0 / 255.0, 0.0, 0.0, 0.0]])
    );
    assert_eq!(imported.findings, vec![]);
}

#[test]
fn a_vertex_that_loses_all_its_slots_keeps_zero_weights() {
    // Every weighted slot is out of group: they drop to an all-zero row,
    // not a division by the remaining zero sum.
    let mesh = ::fmdl::Mesh {
        vertices: ::fmdl::format::MeshVertices {
            positions: vec![[0.0; 3]],
            bone_indices: Some(vec![[1, 1, 1, 1]]),
            bone_weights: Some(vec![[255, 0, 0, 0]]),
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
        Some(vec![[0.0, 0.0, 0.0, 0.0]])
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
fn a_bone_listed_before_its_parent_moves_after_it() {
    let mesh = ::fmdl::Mesh {
        vertices: ::fmdl::format::MeshVertices {
            positions: vec![[0.0; 3]],
            bone_indices: Some(vec![[0, 1, 0, 0]]),
            bone_weights: Some(vec![[128, 127, 0, 0]]),
            ..::fmdl::format::MeshVertices::default()
        },
        faces: vec![[0, 0, 0]],
        bone_group: vec![0, 1],
        material: 0,
        alpha_flags: 0,
        shadow_flags: 0,
        has_antiblur_meshes: false,
        is_antiblur_mesh: false,
        custom_bounding_box: None,
    };
    let mut model = fmdl_model(mesh, vec![instance("mat")]);
    // File order: the child first, its parent second — the `body.skl` order.
    let parent_of = |name: &str, parent| ::fmdl::Bone {
        name: name.to_string(),
        parent,
        bounding_box: ::fmdl::BoundingBox {
            min: [0.0; 4],
            max: [0.0; 4],
        },
        local_position: [0.0; 4],
        world_position: [0.0; 4],
    };
    model.bones = vec![parent_of("sk_chest", Some(1)), parent_of("sk_belly", None)];
    let imported = fmdl_to_ir(&model, None).expect("import");
    assert_eq!(
        imported
            .model
            .bones
            .iter()
            .map(|bone| (bone.name.as_str(), bone.parent))
            .collect::<Vec<_>>(),
        [("sk_belly", None), ("sk_chest", Some(0))]
    );
    // The group remapped to the new order; the vertex slots did not move.
    assert_eq!(imported.model.meshes[0].bone_group, [1, 0]);
    assert_eq!(
        imported.model.meshes[0].vertices.bone_indices,
        Some(vec![[0, 1, 0, 0]])
    );
    // Export goes back through the same parents.
    let exported = ir_to_fmdl(&imported.model).expect("export");
    assert_eq!(exported.model.bones[1].parent, Some(0));
}

#[test]
fn a_bone_parent_cycle_stays_an_error() {
    let mut model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat")]);
    let parent_of = |name: &str, parent| ::fmdl::Bone {
        name: name.to_string(),
        parent,
        bounding_box: ::fmdl::BoundingBox {
            min: [0.0; 4],
            max: [0.0; 4],
        },
        local_position: [0.0; 4],
        world_position: [0.0; 4],
    };
    model.bones = vec![parent_of("a", Some(1)), parent_of("b", Some(0))];
    assert!(matches!(
        fmdl_to_ir(&model, None),
        Err(ConvertError::Fmdl(::fmdl::FmdlError::ParentCycle("bone")))
    ));
}

#[test]
fn a_group_that_stays_split_imports_as_an_ordinary_group() {
    let component = |offset: usize, count: usize, faces: usize| ::fmdl::Mesh {
        vertices: ::fmdl::format::MeshVertices {
            positions: (0..count)
                .map(|index| [(offset + index) as f32, 0.0, 0.0])
                .collect(),
            ..::fmdl::format::MeshVertices::default()
        },
        faces: (0..faces)
            .map(|face| {
                [
                    (3 * face) as u16,
                    (3 * face + 1) as u16,
                    (3 * face + 2) as u16,
                ]
            })
            .collect(),
        bone_group: Vec::new(),
        material: 0,
        alpha_flags: 0,
        shadow_flags: 0,
        has_antiblur_meshes: false,
        is_antiblur_mesh: false,
        custom_bounding_box: None,
    };
    // Two components referencing 70000 distinct vertices: u16 faces cannot
    // index the combined mesh, so the decode keeps the container split.
    let mut model = fmdl_model(component(0, 40000, 13333), vec![instance("mat")]);
    model.meshes.push(component(40000, 30000, 10000));
    model.mesh_groups[0].meshes = Vec::new();
    model.mesh_groups.push(::fmdl::MeshGroup {
        name: "split-mesh".to_string(),
        parent: Some(0),
        meshes: vec![0, 1],
        bounding_box: None,
        visible: true,
        split_mesh_group: true,
    });
    model.extensions.mesh_splitting = true;
    let imported = fmdl_to_ir(&model, None).expect("import");
    // The container reads as an ordinary group holding its two components.
    assert_eq!(imported.model.meshes.len(), 2);
    let group = imported
        .model
        .mesh_groups
        .iter()
        .find(|group| group.name == "split-mesh")
        .expect("the container's group");
    assert_eq!(group.meshes, vec![0, 1]);
    // Export and re-import keep working.
    let exported = ir_to_fmdl(&imported.model).expect("export");
    fmdl_to_ir(&exported.model, exported.skl.as_ref()).expect("reimport");
}

#[test]
fn a_vertexless_mesh_gets_an_empty_bone_group() {
    // A face export's marker mesh: no vertices or faces, a 33-entry bone
    // group holding the whole skeleton as data — over the format's 32.
    let mut ir = minimal_ir();
    ir.bones = (0..33)
        .map(|index| Bone {
            name: format!("b{index}"),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: None,
            local_position: None,
            bounding_box: None,
        })
        .collect();
    ir.meshes[0].vertices = Vertices {
        bone_indices: Some(vec![]),
        bone_weights: Some(vec![]),
        ..Vertices::default()
    };
    ir.meshes[0].faces = vec![];
    ir.meshes[0].bone_group = (0..33).collect();
    let exported = ir_to_fmdl(&ir).expect("export");
    assert_eq!(exported.model.meshes[0].bone_group, Vec::<usize>::new());
    assert_eq!(
        exported
            .findings
            .iter()
            .filter(|f| f.code == "empty_mesh_bone_group_dropped")
            .map(|f| (f.subject.clone(), f.detail.as_str()))
            .collect::<Vec<_>>(),
        [(Subject::Mesh(0), "33")]
    );
    // It writes and re-imports — before, to_file met the 33-entry group.
    let bytes = exported.model.to_file().expect("to_file").write();
    let model = ::fmdl::Model::from_file(&::fmdl::FmdlFile::read(&bytes).expect("read"))
        .expect("from_file");
    fmdl_to_ir(&model, exported.skl.as_ref()).expect("reimport");
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

#[test]
fn quantization_preserves_the_total_past_one() {
    // A vertex the IR accepted unnormalized keeps `round(sum * 255)`
    // across the positive lanes, each at most 255.
    assert_eq!(quantize_weights([0.7, 0.0, 0.0, 0.0]), [178, 0, 0, 0]);
    assert_eq!(quantize_weights([1.0, 1.0, 0.0, 0.0]), [255, 255, 0, 0]);
    // One lane alone can hold 255; the rest of its total has nowhere to
    // go — never onto a zero-weight lane.
    assert_eq!(quantize_weights([2.0, 0.0, 0.0, 0.0]), [255, 0, 0, 0]);
    assert_eq!(quantize_weights([1.5, 0.0, 0.0, 0.0]), [255, 0, 0, 0]);
    assert_eq!(quantize_weights([1.0000001, 0.0, 0.0, 0.0]), [255, 0, 0, 0]);
    assert_eq!(quantize_weights([2.0, 2.0, 2.0, 2.0]), [255, 255, 255, 255]);
    // The clamped lanes keep *their* total: `round(1.5 * 255)` = 383 lands as
    // `round(1.0 / 1.5 * 383)` = 255 then `round(0.5 / 0.5 * 128)` = 128 —
    // the excess does not carry onto the second lane.
    assert_eq!(quantize_weights([1.5, 0.5, 0.0, 0.0]), [255, 128, 0, 0]);
}

#[test]
fn a_lane_above_one_reports_weight_clamped() {
    let skinned = |weights: [f32; 4]| CanonicalModel {
        bones: vec![Bone {
            name: "sk_belly".to_string(),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: None,
            local_position: None,
            bounding_box: None,
        }],
        meshes: vec![Mesh {
            vertices: Vertices {
                positions: vec![[0.0; 3]],
                bone_indices: Some(vec![[0, 0, 0, 0]]),
                bone_weights: Some(vec![weights]),
                ..Vertices::default()
            },
            faces: vec![[0, 0, 0]],
            bone_group: vec![0],
            material: 0,
            extension_headers: Default::default(),
            custom_bounding_box: None,
        }],
        ..minimal_ir()
    };
    let clamped = |weights| {
        ir_to_fmdl(&skinned(weights))
            .expect("export")
            .findings
            .iter()
            .filter(|f| f.code == "weight_clamped")
            .map(|f| (f.subject.clone(), f.detail.clone()))
            .collect::<Vec<_>>()
    };
    let exported = ir_to_fmdl(&skinned([1.5, 0.0, 0.0, 0.0])).expect("export");
    assert_eq!(
        exported.model.meshes[0].vertices.bone_weights,
        Some(vec![[255, 0, 0, 0]])
    );
    assert_eq!(
        clamped([1.5, 0.0, 0.0, 0.0]),
        [(Subject::Mesh(0), "1".to_string())]
    );
    // `[1.5, 0.5]` quantizes to `[255, 128]` — the clamped lanes' total —
    // and still reports the lane above one.
    let exported = ir_to_fmdl(&skinned([1.5, 0.5, 0.0, 0.0])).expect("export");
    assert_eq!(
        exported.model.meshes[0].vertices.bone_weights,
        Some(vec![[255, 128, 0, 0]])
    );
    assert_eq!(
        clamped([1.5, 0.5, 0.0, 0.0]),
        [(Subject::Mesh(0), "1".to_string())]
    );
    // Float noise at and below the threshold clamps silently.
    let exported = ir_to_fmdl(&skinned([1.0000001, 0.0, 0.0, 0.0])).expect("export");
    assert_eq!(
        exported.model.meshes[0].vertices.bone_weights,
        Some(vec![[255, 0, 0, 0]])
    );
    assert!(clamped([1.0000001, 0.0, 0.0, 0.0]).is_empty());
    assert!(clamped([1.0 + 1e-6, 0.0, 0.0, 0.0]).is_empty());
    // Unnormalized but under 1 clamps nothing.
    let exported = ir_to_fmdl(&skinned([0.7, 0.0, 0.0, 0.0])).expect("export");
    assert_eq!(
        exported.model.meshes[0].vertices.bone_weights,
        Some(vec![[178, 0, 0, 0]])
    );
    assert!(clamped([0.7, 0.0, 0.0, 0.0]).is_empty());
}

#[test]
fn uncarried_prefox_samplers_and_parameters_are_findings() {
    // `konami_headHi`'s native `.mtl` samplers `Normal2` and `Mapping` bind
    // textures the Fox resolver does not carry; the canonical `NormalMap`
    // maps to `NormalMap_Tex_NRM` and is not reported.
    let file = ::pes_model::format::PreFoxModel::read(HEAD_HI).expect("read");
    let model = ::pes_model::model::Model::from_file(&file).expect("model");
    let mtl = ::pes_model::format::mtl::MaterialSet::read(HEAD_HI_MTL).expect("mtl");
    let ir = crate::formats::pes_model::model_to_ir(&model, &mtl)
        .expect("import")
        .model;
    let exported = ir_to_fmdl(&ir).expect("export");
    assert_eq!(
        exported.findings,
        [
            Finding {
                code: "material_texture_unused",
                subject: Subject::Material(0),
                detail: "Normal2".to_string(),
            },
            Finding {
                code: "material_texture_unused",
                subject: Subject::Material(0),
                detail: "Mapping".to_string(),
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
        ]
    );
}

#[test]
fn a_mesh_matches_its_own_flag_combination() {
    // Two flag combinations on one instance: a mesh's material is the
    // entry matching all three flags, not the first sharing one.
    let mesh = |alpha, shadow| ::fmdl::Mesh {
        vertices: ::fmdl::format::MeshVertices {
            positions: vec![[0.0; 3]],
            ..::fmdl::format::MeshVertices::default()
        },
        faces: vec![[0, 0, 0]],
        bone_group: vec![],
        material: 0,
        alpha_flags: alpha,
        shadow_flags: shadow,
        has_antiblur_meshes: false,
        is_antiblur_mesh: false,
        custom_bounding_box: None,
    };
    let mut model = fmdl_model(mesh(1, 1), vec![instance("mat")]);
    model.meshes.push(mesh(0, 1));
    model.mesh_groups[0].meshes = vec![0, 1];
    let imported = fmdl_to_ir(&model, None).expect("import");
    assert_eq!(
        imported
            .model
            .materials
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>(),
        ["mat", "mat_2"]
    );
    assert_eq!(imported.model.meshes[0].material, 0);
    assert_eq!(imported.model.meshes[1].material, 1);
}

#[test]
fn a_second_sampler_for_one_role_goes_native() {
    // The first `Base_Tex_*` wins the canonical Base role; a second is a
    // native sampler, not a duplicate canonical entry.
    let mut inst = instance("mat");
    inst.textures = vec![
        (
            "Base_Tex_LIN".to_string(),
            ::fmdl::Texture {
                file_name: "a.tga".to_string(),
                directory: String::new(),
            },
        ),
        (
            "Base_Tex_SRGB".to_string(),
            ::fmdl::Texture {
                file_name: "b.tga".to_string(),
                directory: String::new(),
            },
        ),
    ];
    let imported = fmdl_to_ir(&fmdl_model(fmdl_mesh(0, 0), vec![inst]), None).expect("import");
    let material = &imported.model.materials[0];
    assert_eq!(material.textures, vec![(TextureRole::Base, 0)]);
    assert_eq!(
        material.fox.as_ref().expect("fox").textures,
        vec![("Base_Tex_SRGB".to_string(), 1)]
    );
}

#[test]
fn exported_bone_bounding_boxes_cover_their_weighted_vertices() {
    // A bone's SKL box spans the vertices weighted to it: unweighted
    // slots and other bones' vertices do not reach in.
    let mut ir = minimal_ir();
    ir.bones = vec![
        Bone {
            name: "sk_a".to_string(),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: None,
            local_position: None,
            bounding_box: None,
        },
        Bone {
            name: "sk_b".to_string(),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: None,
            local_position: None,
            bounding_box: None,
        },
    ];
    ir.meshes = vec![Mesh {
        vertices: Vertices {
            positions: vec![
                [1.0, 2.0, 3.0],
                [4.0, 5.0, 6.0],
                [99.0, 99.0, 99.0],
                [-50.0, -50.0, -50.0],
            ],
            bone_indices: Some(vec![[0, 0, 0, 0]; 4]),
            bone_weights: Some(vec![
                [1.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
            ]),
            bone_weight_width: Some(4),
            ..Vertices::default()
        },
        faces: vec![[0, 1, 3]],
        bone_group: vec![0, 1],
        material: 0,
        extension_headers: Default::default(),
        custom_bounding_box: None,
    }];
    // v3 weights slot 1 -> bone group[1] = bone 1; bone_indices slot 0
    // is a decoy pointing at group[0] on an unweighted slot.
    ir.meshes[0]
        .vertices
        .bone_indices
        .as_mut()
        .expect("indices")[3] = [0, 1, 0, 0];
    let exported = ir_to_fmdl(&ir).expect("export");
    assert_eq!(
        exported.model.bones[0].bounding_box.min,
        [1.0, 2.0, 3.0, 1.0]
    );
    assert_eq!(
        exported.model.bones[0].bounding_box.max,
        [4.0, 5.0, 6.0, 1.0]
    );
    assert_eq!(
        exported.model.bones[1].bounding_box.min,
        [-50.0, -50.0, -50.0, 1.0]
    );
    assert_eq!(
        exported.model.bones[1].bounding_box.max,
        [-50.0, -50.0, -50.0, 1.0]
    );
}

#[test]
fn split_parents_uses_the_render_parent_name() {
    let bone = |name: &str, parent: Option<usize>| ::fmdl::Bone {
        name: name.to_string(),
        parent,
        bounding_box: ::fmdl::BoundingBox {
            min: [0.0; 4],
            max: [0.0; 4],
        },
        local_position: [0.0; 4],
        world_position: [0.0; 4],
    };
    let parents = split_parents(&[bone("dsk_hip", None), bone("sk_belly", None)]);
    assert_eq!(parents, vec![None, Some(0)]);
}

#[test]
fn exported_local_position_is_world_minus_parent_world() {
    let mut ir = minimal_ir();
    ir.bones = vec![
        Bone {
            name: "sk_a".to_string(),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: Some([10.0, 10.0, 10.0, 1.0]),
            local_position: None,
            bounding_box: None,
        },
        Bone {
            name: "sk_b".to_string(),
            parent: Some(0),
            matrix: Affine::IDENTITY,
            global_position: Some([11.0, 13.0, 15.0, 1.0]),
            local_position: None,
            bounding_box: None,
        },
    ];
    let exported = ir_to_fmdl(&ir).expect("export");
    assert_eq!(exported.model.bones[1].local_position, [1.0, 3.0, 5.0, 1.0]);
    assert_eq!(exported.model.bones[0].local_position, [0.0, 0.0, 0.0, 1.0]);
}

/// A flat `fmdl::Mesh` over no bone group with the given alpha flags and material.
fn fmdl_mesh(alpha: u8, material: usize) -> ::fmdl::Mesh {
    ::fmdl::Mesh {
        vertices: ::fmdl::format::MeshVertices {
            positions: vec![[0.0; 3]],
            ..::fmdl::format::MeshVertices::default()
        },
        faces: vec![[0, 0, 0]],
        bone_group: vec![],
        material,
        alpha_flags: alpha,
        shadow_flags: 0,
        has_antiblur_meshes: false,
        is_antiblur_mesh: false,
        custom_bounding_box: None,
    }
}

/// A skinned IR whose vertices 1 and 2 share position and skinning and differ only
/// in UV: one topological vertex encoded as two loops.
fn seam_ir() -> CanonicalModel {
    let mut ir = minimal_ir();
    ir.bones = vec![Bone {
        name: "sk_belly".to_string(),
        parent: None,
        matrix: Affine::IDENTITY,
        global_position: None,
        local_position: None,
        bounding_box: None,
    }];
    ir.source_format = SourceFormat::Fox;
    let mesh = &mut ir.meshes[0];
    mesh.vertices = Vertices {
        positions: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        uvs: vec![vec![[0.0, 0.0], [0.0, 0.0], [0.5, 0.0], [0.0, 1.0]]],
        uv_high_precision: vec![false],
        bone_indices: Some(vec![[0, 0, 0, 0]; 4]),
        bone_weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 4]),
        ..Vertices::default()
    };
    mesh.faces = vec![[0, 1, 3], [1, 2, 3]];
    mesh.bone_group = vec![0];
    ir
}

#[test]
fn vertex_owners_come_from_the_ir_order() {
    let exported = ir_to_fmdl(&seam_ir()).expect("export");
    assert_eq!(exported.model.meshes.len(), 1);
    let owner = ::fmdl::ops::vertex_enc::decode(&exported.model.meshes[0]).expect("decode");
    // The flag-gated `decode_model` returned identity owners [0, 1, 2, 3]; here
    // vertex 2 is the second loop of vertex 1.
    assert_eq!(owner, vec![0, 1, 1, 3]);
}

#[test]
fn a_split_material_never_reuses_an_instance_name() {
    // `mat` splits on two flag combinations while `mat_2` already exists: the
    // generated name must skip the taken one.
    let mut model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat"), instance("mat_2")]);
    model.meshes.push(fmdl_mesh(32, 0));
    model.meshes.push(fmdl_mesh(0, 1));
    model.mesh_groups[0].meshes = vec![0, 1, 2];
    let imported = fmdl_to_ir(&model, None).expect("import");
    let names: Vec<&str> = imported
        .model
        .materials
        .iter()
        .map(|material| material.name.as_str())
        .collect();
    assert_eq!(names, ["mat", "mat_3", "mat_2"]);
    assert_eq!(
        imported.findings,
        vec![Finding {
            code: "material_split_by_flags",
            subject: Subject::Material(1),
            detail: "mat_3".to_string(),
        }]
    );
}

#[test]
fn bone_matrices_report_native_field_dropped() {
    let mut model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat")]);
    model.bone_matrices = Some(vec![0; 64]);
    let imported = fmdl_to_ir(&model, None).expect("import");
    assert_eq!(
        imported.findings,
        vec![Finding {
            code: "native_field_dropped",
            subject: Subject::Model,
            detail: "bone_matrices".to_string(),
        }]
    );
}

#[test]
fn a_disagreeing_skl_parent_reports_native_field_dropped() {
    // The SKL parents `sk_chest` under `sk_belly`; the FMDL leaves it a root and
    // the FMDL wins.
    let mut model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat")]);
    model.bones.push(::fmdl::Bone {
        name: "sk_chest".to_string(),
        parent: None,
        bounding_box: ::fmdl::BoundingBox {
            min: [0.0; 4],
            max: [0.0; 4],
        },
        local_position: [0.0; 4],
        world_position: [0.0; 4],
    });
    let skl = ::fmdl::SklFile::new(vec![
        ::fmdl::format::SklBone {
            name: "sk_belly".to_string(),
            parent: None,
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        },
        ::fmdl::format::SklBone {
            name: "sk_chest".to_string(),
            parent: Some(0),
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        },
    ]);
    let imported = fmdl_to_ir(&model, Some(&skl)).expect("import");
    assert_eq!(imported.model.bones[1].parent, None);
    assert_eq!(
        imported.findings,
        vec![Finding {
            code: "native_field_dropped",
            subject: Subject::Bone(1),
            detail: "skl_parent".to_string(),
        }]
    );
}

#[test]
fn out_of_range_parents_error() {
    let model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat")]);
    // An SKL bone parented past the skeleton's own bone count.
    let skl = ::fmdl::SklFile::new(vec![::fmdl::format::SklBone {
        name: "sk_belly".to_string(),
        parent: Some(5),
        rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        translation: [0.0; 3],
    }]);
    assert!(matches!(
        fmdl_to_ir(&model, Some(&skl)),
        Err(ConvertError::Fmdl(::fmdl::FmdlError::BadReference {
            what: "skl parent",
            index: 5
        }))
    ));
    // The same for the model's own parent index — caught by the model
    // validation `split::decode` runs before the importer's name lookup.
    let mut model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat")]);
    model.bones[0].parent = Some(7);
    assert!(matches!(
        fmdl_to_ir(&model, None),
        Err(ConvertError::Fmdl(::fmdl::FmdlError::BadReference {
            what: "bone",
            index: 7
        }))
    ));
}
