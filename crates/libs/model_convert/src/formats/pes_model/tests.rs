use super::import::bone_weights;
use super::*;

use ::pes_model::format::mtl;
use ::pes_model::model::Model;

use crate::affine::Affine;
use crate::formats::{ConvertError, fmdl};
use crate::ir::{Bone, CanonicalModel, Material, Mesh, MeshGroup, SourceFormat, Texture, Vertices};
use crate::loss::{Finding, Subject};
use crate::materials::{PreFoxMaterial, TextureRole, to_prefox};
use crate::skeletons;

use crate::materials::MaterialFamily;
use pes_version::PesVersion;

const CAP_M: &[u8] = include_bytes!("../../../tests/fixtures/konami_modD_cap.model");
const CAP_T: &[u8] = include_bytes!("../../../tests/fixtures/konami_modD_cap.mtl");
const CARD_M: &[u8] = include_bytes!("../../../tests/fixtures/konami_card.model");
const CARD_T: &[u8] = include_bytes!("../../../tests/fixtures/konami_card_red.mtl");
const GLASSES_M: &[u8] = include_bytes!("../../../tests/fixtures/konami_glasses_02.wesys.model");
const GLASSES_T: &[u8] = include_bytes!("../../../tests/fixtures/konami_accessory.mtl");
const CARDHEAD_M: &[u8] = include_bytes!("../../../tests/fixtures/cardhead_face_high.model");
const CARDHEAD_T: &[u8] = include_bytes!("../../../tests/fixtures/cardhead_materials.mtl");
const HIGHNECK: &[u8] = include_bytes!("../../../tests/fixtures/konami_highneck.fmdl");

fn model(bytes: &[u8]) -> Model {
    Model::from_file(&::pes_model::format::PreFoxModel::read(bytes).expect("parse")).expect("model")
}

fn set(bytes: &[u8]) -> mtl::MaterialSet {
    mtl::MaterialSet::read(bytes).expect("mtl")
}

/// `ir_to_model`'s output applied to `model` the same way: split decode, the
/// indices-only weight synthesis, then the vertex-loop and split encoders with the
/// render parents.
fn expected(model: &Model) -> Model {
    let mut model = model.clone();
    ::pes_model::ops::split::decode(&mut model).expect("split decode");
    for mesh in &mut model.meshes {
        let synthesized =
            mesh.vertices.bone_indices.is_some() && mesh.vertices.bone_weights.is_none();
        mesh.vertices.bone_weights = bone_weights(&mesh.vertices);
        if synthesized {
            mesh.vertices.bone_weight_width = 4;
        }
    }
    let owners: Vec<Vec<usize>> = model
        .meshes
        .iter()
        .map(::pes_model::ops::vertex_enc::decode)
        .collect();
    ::pes_model::ops::vertex_enc::encode_model(&mut model, &owners).expect("vertex encode");
    let parents: Vec<Option<usize>> = model
        .bones
        .iter()
        .map(|bone| {
            skeletons::render_parent(&bone.name)
                .and_then(|parent| model.bones.iter().position(|b| b.name == parent))
        })
        .collect();
    ::pes_model::ops::split::encode(&mut model, &parents).expect("split encode");
    model
}

/// One `.mtl` definition with no entries.
fn mtl_of(name: &str, shader: &str) -> mtl::MaterialSet {
    mtl::MaterialSet {
        materials: vec![mtl::Material {
            name: name.to_string(),
            shader: shader.to_string(),
            entries: vec![],
        }],
        style: mtl::MtlStyle::default(),
    }
}

/// A minimal mesh over `bone_group`, weighted to its first two slots.
fn pes_mesh(bone_group: Vec<usize>) -> ::pes_model::model::Mesh {
    ::pes_model::model::Mesh {
        name: None,
        extension_headers: vec![],
        tags: vec![],
        vertices: ::pes_model::format::MeshVertices {
            positions: vec![[0.0; 3]],
            normals: None,
            tangents: None,
            bitangents: None,
            colors: None,
            uvs: vec![],
            bone_indices: Some(vec![[0, 1, 0, 0]]),
            bone_weights: Some(vec![[0.5, 0.5, 0.0, 0.0]]),
            bone_weight_width: 4,
        },
        faces: vec![[0, 0, 0]],
        lower_lods: vec![],
        bone_group,
        material: 0,
        bounds: ::pes_model::format::BoundingBox::of(&[[0.0; 3]]),
        order: 0,
        editor_data: vec![],
    }
}

/// A one-mesh `Model` over `bones` for the synthetic cases.
fn pes_model(bones: Vec<&str>, mesh: ::pes_model::model::Mesh) -> Model {
    let bounds = ::pes_model::format::BoundingBox::of(&mesh.vertices.positions);
    Model {
        flags: 0,
        bones: bones
            .into_iter()
            .map(|name| ::pes_model::format::Bone {
                name: name.to_string(),
                matrix: Affine::IDENTITY.0,
            })
            .collect(),
        materials: vec!["mat".to_string()],
        meshes: vec![mesh],
        extension_headers: vec![],
        bounds,
        lod: ::pes_model::format::LodRecord::for_levels(0),
    }
}

/// The entries of `entries` that are samplers (kind 0), states (1) or vectors (2).
fn of_kind(entries: &[mtl::MaterialEntry], kind: u8) -> Vec<mtl::MaterialEntry> {
    entries
        .iter()
        .filter(|entry| {
            matches!(
                (entry, kind),
                (mtl::MaterialEntry::Sampler(_), 0)
                    | (mtl::MaterialEntry::State(_), 1)
                    | (mtl::MaterialEntry::Vector(_), 2)
            )
        })
        .cloned()
        .collect()
}

#[test]
fn cap_imports_on_the_pes17_skeleton() {
    let imported = model_to_ir(&model(CAP_M), &set(CAP_T)).expect("import");
    let model = &imported.model;
    // The file order was already parent-first.
    assert_eq!(
        model
            .bones
            .iter()
            .map(|bone| bone.name.as_str())
            .collect::<Vec<_>>(),
        [
            "sk_shoulder_l",
            "sk_upperarm_l",
            "dsk_deltoid_l",
            "dsk_upperarm_long_l"
        ]
    );
    // `dsk_deltoid_l` under `sk_shoulder_l`; `dsk_upperarm_long_l`'s render parent
    // `dsk_upperarm_l` is absent, so it is a root.
    assert_eq!(model.bones[2].parent, Some(0));
    assert_eq!(model.bones[3].parent, None);
    let body = &crate::skeletons::skeletons(PesVersion::Pes17).body;
    for bone in &model.bones {
        let template = body.bone(&bone.name).expect("template bone");
        assert!(
            bone.matrix.max_component_delta(&template.matrix) < 2e-5,
            "{}",
            bone.name
        );
    }
    let material = &model.materials[0];
    assert_eq!(material.family, MaterialFamily::Shaded);
    assert_eq!(material.two_sided, None);
    assert_eq!(material.transparent, None);
    assert_eq!(
        imported.findings,
        vec![Finding {
            code: "native_field_dropped",
            subject: Subject::Mesh(0),
            detail: "tags".to_string(),
        }]
    );
}

#[test]
fn pes_model_round_trip() {
    for (model_bytes, mtl_bytes, konami) in [
        (CAP_M, CAP_T, true),
        (CARD_M, CARD_T, true),
        (GLASSES_M, GLASSES_T, true),
        (CARDHEAD_M, CARDHEAD_T, false),
    ] {
        let input = model(model_bytes);
        let input_mtl = set(mtl_bytes);
        let imported = model_to_ir(&input, &input_mtl).expect("import");
        let exported = ir_to_model(&imported.model).expect("export");
        let expected = expected(&input);

        assert_eq!(
            exported
                .model
                .bones
                .iter()
                .map(|bone| bone.name.as_str())
                .collect::<Vec<_>>(),
            expected
                .bones
                .iter()
                .map(|bone| bone.name.as_str())
                .collect::<Vec<_>>()
        );
        for (got, want) in exported.model.bones.iter().zip(&expected.bones) {
            assert!(
                Affine(got.matrix).max_component_delta(&Affine(want.matrix)) < 1e-5,
                "{}",
                got.name
            );
        }
        assert_eq!(exported.model.materials, expected.materials);
        for (index, (got, want)) in exported
            .model
            .meshes
            .iter()
            .zip(&expected.meshes)
            .enumerate()
        {
            assert_eq!(got.vertices, want.vertices, "mesh {index}");
            assert_eq!(got.faces, want.faces, "mesh {index}");
            assert!(got.lower_lods.is_empty(), "mesh {index}");
            assert_eq!(got.bone_group, want.bone_group, "mesh {index}");
            assert_eq!(got.material, want.material, "mesh {index}");
            assert!(got.tags.is_empty(), "mesh {index}");
            assert_eq!(got.order, 0, "mesh {index}");
            assert!(got.editor_data.is_empty(), "mesh {index}");
            assert_eq!(
                got.extension_headers, want.extension_headers,
                "mesh {index}"
            );
            if konami {
                assert_eq!(got.name, Some(format!("mesh_{index}")), "mesh {index}");
            } else {
                assert_eq!(got.name, want.name, "mesh {index}");
            }
            assert_eq!(got.bounds, want.bounds, "mesh {index}");
        }
        assert_eq!(exported.model.extension_headers, expected.extension_headers);
        assert_eq!(exported.model.flags, 0);
        assert_eq!(
            exported.model.lod,
            ::pes_model::format::LodRecord::for_levels(0)
        );
        assert_eq!(exported.model.bounds, expected.bounds);

        // The `.mtl`: the input's definitions filtered to the bound names in model
        // order, compared per entry kind (the export regroups to samplers, states,
        // vectors — Konami's majority order).
        let want: Vec<&mtl::Material> = input
            .materials
            .iter()
            .map(|name| {
                input_mtl
                    .materials
                    .iter()
                    .find(|material| material.name == *name)
                    .expect("bound material")
            })
            .collect();
        assert_eq!(exported.mtl.materials.len(), want.len());
        for (got, want) in exported.mtl.materials.iter().zip(want) {
            assert_eq!(got.name, want.name);
            assert_eq!(got.shader, want.shader);
            for kind in 0..3 {
                assert_eq!(
                    of_kind(&got.entries, kind),
                    of_kind(&want.entries, kind),
                    "{} kind {kind}",
                    got.name
                );
            }
        }
        assert_eq!(exported.mtl.style, mtl::MtlStyle::default());
    }
}

#[test]
fn pes_model_round_trip_findings() {
    // cap: one Konami tag dropped.
    let imported = model_to_ir(&model(CAP_M), &set(CAP_T)).expect("import");
    assert_eq!(
        imported.findings,
        vec![Finding {
            code: "native_field_dropped",
            subject: Subject::Mesh(0),
            detail: "tags".to_string(),
        }]
    );
    assert_eq!(
        ir_to_model(&imported.model).expect("export").findings,
        Vec::<Finding>::new()
    );
    // card: nothing to report.
    let imported = model_to_ir(&model(CARD_M), &set(CARD_T)).expect("import");
    assert_eq!(imported.findings, Vec::<Finding>::new());
    assert_eq!(
        ir_to_model(&imported.model).expect("export").findings,
        Vec::<Finding>::new()
    );
    // glasses: `Accessory` is no shader rule's name; both meshes carry Konami tags.
    let imported = model_to_ir(&model(GLASSES_M), &set(GLASSES_T)).expect("import");
    assert_eq!(
        imported.findings,
        vec![
            Finding {
                code: "material_family_approximated",
                subject: Subject::Material(1),
                detail: "glasses_02C".to_string(),
            },
            Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(0),
                detail: "tags".to_string(),
            },
            Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(1),
                detail: "tags".to_string(),
            },
        ]
    );
    assert_eq!(
        ir_to_model(&imported.model).expect("export").findings,
        Vec::<Finding>::new()
    );
    // card head: nothing to report.
    let imported = model_to_ir(&model(CARDHEAD_M), &set(CARDHEAD_T)).expect("import");
    assert_eq!(imported.findings, Vec::<Finding>::new());
    assert_eq!(
        ir_to_model(&imported.model).expect("export").findings,
        Vec::<Finding>::new()
    );
}

#[test]
fn a_bone_listed_before_its_parent_moves_after_it() {
    let input = pes_model(
        vec!["dsk_forearm_l", "dsk_forearm_t_l"],
        pes_mesh(vec![0, 1]),
    );
    let imported = model_to_ir(&input, &mtl_of("mat", "Basic_C")).expect("import");
    let bones = &imported.model.bones;
    assert_eq!(bones[0].name, "dsk_forearm_t_l");
    assert_eq!(bones[0].parent, None);
    assert_eq!(bones[1].name, "dsk_forearm_l");
    assert_eq!(bones[1].parent, Some(0));
    assert_eq!(imported.model.meshes[0].bone_group, [1, 0]);
    assert_eq!(
        imported.model.meshes[0].vertices.bone_indices,
        Some(vec![[0, 1, 0, 0]])
    );
}

#[test]
fn a_bound_material_without_a_definition_is_an_error() {
    let mut input = pes_model(vec!["sk_belly"], pes_mesh(vec![0]));
    input.materials = vec!["nope".to_string()];
    let empty = mtl::MaterialSet {
        materials: vec![],
        style: mtl::MtlStyle::default(),
    };
    let Err(ConvertError::MaterialUndefined(name)) = model_to_ir(&input, &empty) else {
        panic!("expected MaterialUndefined")
    };
    assert_eq!(name, "nope");
}

#[test]
fn a_bone_group_past_the_bone_table_is_an_error() {
    let input = pes_model(vec!["sk_belly", "sk_chest"], pes_mesh(vec![0, 5]));
    let Err(ConvertError::PesModel(::pes_model::format::ModelError::BadReference { what, offset })) =
        model_to_ir(&input, &mtl_of("mat", "Basic_C"))
    else {
        panic!("expected BadReference")
    };
    assert_eq!(what, "bone");
    assert_eq!(offset, 5);
}

#[test]
fn dropped_fields_report_each_field() {
    let mut mesh = pes_mesh(vec![0, 1]);
    mesh.lower_lods = vec![vec![[0, 0, 0]]];
    mesh.tags = vec![(1, "part".to_string())];
    mesh.order = 3;
    let input = pes_model(vec!["sk_belly", "sk_chest"], mesh);
    let imported = model_to_ir(&input, &mtl_of("mat", "Basic_C")).expect("import");
    assert_eq!(
        imported.findings,
        vec![
            Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(0),
                detail: "lower_lods".to_string(),
            },
            Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(0),
                detail: "tags".to_string(),
            },
            Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(0),
                detail: "order".to_string(),
            },
        ]
    );
}

#[test]
fn fox_ir_exports_to_pre_fox() {
    let highneck = fmdl::fmdl_to_ir(
        &::fmdl::Model::from_file(&::fmdl::FmdlFile::read(HIGHNECK).expect("parse"))
            .expect("model"),
        None,
    )
    .expect("import");
    let exported = ir_to_model(&highneck.model).expect("export");
    assert_eq!(exported.mtl.materials.len(), 1);
    let material = &highneck.model.materials[0];
    let roles: Vec<TextureRole> = material.textures.iter().map(|(role, _)| *role).collect();
    assert_eq!(
        exported.mtl.materials[0].shader,
        to_prefox::default_shader(material.family, &roles)
    );
    assert_eq!(exported.findings, Vec::<Finding>::new());
}

/// A minimal consistent IR around `vertices`/`faces`: one bone, one mesh weighted
/// to it, one `Shaded` material, one group.
fn ir_over(vertices: Vertices, faces: Vec<[u16; 3]>) -> CanonicalModel {
    CanonicalModel {
        bones: vec![Bone {
            name: "sk_belly".to_string(),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: None,
            local_position: None,
            bounding_box: None,
        }],
        meshes: vec![Mesh {
            vertices,
            faces,
            bone_group: vec![0],
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
fn vertex_owners_come_from_the_ir_order() {
    // Vertices 1 and 2 share position and skinning, differ only in UV: one
    // topological vertex encoded as two loops.
    let ir = ir_over(
        Vertices {
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
        },
        vec![[0, 1, 3], [1, 2, 3]],
    );
    let exported = ir_to_model(&ir).expect("export");
    assert_eq!(exported.model.meshes.len(), 1);
    let owner = ::pes_model::ops::vertex_enc::decode(&exported.model.meshes[0]);
    // The flag-gated `decode_model` returned identity owners [0, 1, 2, 3]; here
    // vertex 2 is the second loop of vertex 1.
    assert_eq!(owner, vec![0, 1, 1, 3]);
}

#[test]
fn a_non_unit_normal_or_tangent_w_reports() {
    let vertices = |normals, tangents| Vertices {
        positions: vec![[0.0; 3]; 3],
        normals,
        tangents,
        uvs: vec![vec![[0.0, 0.0]; 3]],
        uv_high_precision: vec![false],
        bone_indices: Some(vec![[0, 0, 0, 0]; 3]),
        bone_weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
        ..Vertices::default()
    };
    let finding = |detail: &str| Finding {
        code: "native_field_dropped",
        subject: Subject::Mesh(0),
        detail: detail.to_string(),
    };
    let normals = vertices(
        Some(vec![[0.0, 0.0, 1.0, 0.0]; 3]),
        Some(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
    );
    assert_eq!(
        ir_to_model(&ir_over(normals, vec![[0, 1, 2]]))
            .expect("export")
            .findings,
        vec![finding("normal_w")]
    );
    let tangents = vertices(
        Some(vec![[0.0, 0.0, 1.0, 1.0]; 3]),
        Some(vec![[1.0, 0.0, 0.0, -1.0]; 3]),
    );
    assert_eq!(
        ir_to_model(&ir_over(tangents, vec![[0, 1, 2]]))
            .expect("export")
            .findings,
        vec![finding("tangent_w")]
    );
}

#[test]
fn a_stored_sampler_name_without_settings_reports() {
    // `prefox.textures` carries a sampler the `prefox.samplers` table never
    // defined: it exports with the default settings, and that is a finding.
    let mut ir = ir_over(
        Vertices {
            positions: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            ..Vertices::default()
        },
        vec![[0, 1, 2]],
    );
    ir.textures = vec![Texture {
        directory: "./".to_string(),
        file_name: "t.dds".to_string(),
    }];
    ir.materials[0].prefox = Some(PreFoxMaterial {
        shader: "Basic_C".to_string(),
        states: vec![],
        samplers: vec![],
        textures: vec![("ExtraMap".to_string(), 0)],
        parameters: vec![],
    });
    let exported = ir_to_model(&ir).expect("export");
    assert_eq!(
        exported.mtl.materials[0]
            .entries
            .iter()
            .filter_map(|entry| match entry {
                mtl::MaterialEntry::Sampler(sampler) => Some(sampler.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        ["ExtraMap"]
    );
    assert_eq!(
        exported.findings,
        vec![Finding {
            code: "sampler_settings_defaulted",
            subject: Subject::Material(0),
            detail: "ExtraMap".to_string(),
        }]
    );
}
