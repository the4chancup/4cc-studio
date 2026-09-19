use super::export::{quantize_weights, split_parents};
use super::*;

use crate::formats::ConvertError;
use crate::ir::{Bone, CanonicalModel, Material, Mesh, MeshGroup, SourceFormat, Texture, Vertices};
use crate::loss::{Finding, Subject};
use crate::materials::MaterialFamily;

use crate::materials::TextureRole;

const HIGHNECK: &[u8] = include_bytes!("../../../tests/fixtures/konami_highneck.fmdl");
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
    ::fmdl::ops::antiblur::decode(&mut model);
    model
}

/// `ir_to_fmdl`'s encoder half, run on a decoded model for comparison.
fn encoded(decoded: &::fmdl::Model) -> ::fmdl::Model {
    let mut model = decoded.clone();
    ::fmdl::ops::antiblur::encode(&mut model);
    let owners: Vec<Vec<usize>> = model
        .meshes
        .iter()
        .map(::fmdl::ops::vertex_enc::decode)
        .collect();
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
    // The file carries the redundant local-space bone-matrix block.
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
    let owner = ::fmdl::ops::vertex_enc::decode(&exported.model.meshes[0]);
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
    let skl = ::fmdl::SklFile {
        bones: vec![
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
        ],
    };
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
    let skl = ::fmdl::SklFile {
        bones: vec![::fmdl::format::SklBone {
            name: "sk_belly".to_string(),
            parent: Some(5),
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        }],
    };
    assert!(matches!(
        fmdl_to_ir(&model, Some(&skl)),
        Err(ConvertError::Fmdl(::fmdl::FmdlError::BadReference {
            what: "skl parent",
            index: 5
        }))
    ));
    // The same for the model's own parent index.
    let mut model = fmdl_model(fmdl_mesh(0, 0), vec![instance("mat")]);
    model.bones[0].parent = Some(7);
    assert!(matches!(
        fmdl_to_ir(&model, None),
        Err(ConvertError::Fmdl(::fmdl::FmdlError::BadReference {
            what: "bone parent",
            index: 7
        }))
    ));
}
