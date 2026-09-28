use super::*;
use crate::format::container::FmdlContainer;
use crate::format::{FmdlError, FmdlFile};

const HIGHNECK: &[u8] = include_bytes!("../../tests/fixtures/konami_highneck.fmdl");
const MOUTH: &[u8] = include_bytes!("../../tests/fixtures/konami_mouth.fmdl");
const AU_LOW: &[u8] = include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl");
const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
const PLACEHOLDER: &[u8] = include_bytes!("../../tests/fixtures/addon_placeholder.fmdl");

const FIXTURES: &[&[u8]] = &[HIGHNECK, MOUTH, AU_LOW, ORAL, PLACEHOLDER];

fn model(bytes: &[u8]) -> Model {
    Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap()
}

/// A mesh over `vertices`, one triangle, material 0.
fn mesh_with(vertices: MeshVertices) -> Mesh {
    Mesh {
        vertices,
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

/// A `Model` of `meshes` split into one group per entry of `groups`.
fn grouped_model(meshes: Vec<Mesh>, groups: Vec<Vec<usize>>) -> Model {
    Model {
        bones: Vec::new(),
        materials: vec![MaterialInstance {
            name: "mat".to_owned(),
            shader: "shader".to_owned(),
            technique: "technique".to_owned(),
            textures: Vec::new(),
            parameters: Vec::new(),
        }],
        meshes,
        mesh_groups: groups
            .iter()
            .enumerate()
            .map(|(index, meshes)| MeshGroup {
                name: format!("group{index}"),
                parent: None,
                meshes: meshes.clone(),
                bounding_box: None,
                visible: true,
                split_mesh_group: false,
            })
            .collect(),
        extensions: Extensions::default(),
        bone_matrices: None,
    }
}

/// The usual smallest legal vertices: three positions, nothing else.
fn plain_vertices() -> MeshVertices {
    MeshVertices {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ..MeshVertices::default()
    }
}

fn plain_model() -> Model {
    grouped_model(vec![mesh_with(plain_vertices())], vec![vec![0]])
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
fn a_second_extensions_line_is_not_flags() {
    // Only the first line feeds the flags list; a repeated
    // `X-FMDL-Extensions` key lands in the object headers, which no
    // extension flag is ever looked up through.
    let mut file = FmdlFile::read(HIGHNECK).unwrap();
    let mut table = file.string_table.clone().unwrap();
    table.extend_from_slice(b"X-FMDL-Extensions: \nX-FMDL-Extensions: mesh-splitting\n\0");
    file.string_table = Some(table);
    let model = Model::from_file(&file).unwrap();
    assert!(!model.extensions.mesh_splitting);
}

#[test]
fn over_flagged_split_mesh_groups_read_as_ordinary() {
    // Writers also list roots and empty groups in `Split-Mesh-Groups`:
    // a listed group is a container only with a parent and a mesh.
    let mut source = plain_model();
    source.mesh_groups[0].split_mesh_group = true; // a root holding mesh 0
    source.mesh_groups.push(MeshGroup {
        name: "split-mesh".to_owned(),
        parent: Some(0),
        meshes: Vec::new(),
        bounding_box: None,
        visible: true,
        split_mesh_group: true,
    });
    source.extensions.mesh_splitting = true;
    let written = source.to_file().unwrap().write();
    let mut again = model(&written);
    assert!(!again.mesh_groups[0].split_mesh_group);
    assert!(!again.mesh_groups[1].split_mesh_group);
    // Decode sees no container; a rewrite lists neither group.
    crate::ops::split::decode(&mut again).unwrap();
    let rewritten = again.to_file().unwrap().write();
    assert_eq!(model(&rewritten), again);
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

#[test]
fn to_file_round_trips_semantically() {
    for bytes in FIXTURES {
        let model = model(bytes);
        let written = model.to_file().unwrap();
        let again = Model::from_file(&written).unwrap();
        assert_eq!(again, model);
    }
}

#[test]
fn written_file_is_a_valid_container() {
    for (bytes, has_bones) in [
        (HIGHNECK, true),
        (MOUTH, true),
        (AU_LOW, true),
        (ORAL, true),
        (PLACEHOLDER, false),
    ] {
        let model = model(bytes);
        let written = model.to_file().unwrap().write();
        let file = FmdlFile::read(&written).unwrap();
        let container = FmdlContainer::read(&written).unwrap();
        assert_eq!(file.bones.is_empty(), !has_bones);
        assert_eq!(
            container.section1.iter().any(|block| block.id == 1),
            has_bones || model.bone_matrices.is_some()
        );
        assert!(container.section1.iter().any(|block| block.id == 0));
        assert!(container.section1.iter().any(|block| block.id == 2));
        assert!(container.section1.iter().any(|block| block.id == 3));
    }
}

#[test]
fn written_layout_facts() {
    let written = model(HIGHNECK).to_file().unwrap();
    assert_eq!(written.buffer_offsets.len(), 3);
    assert_eq!(written.buffer_offsets[0].eof, 0);
    assert_eq!(written.buffer_offsets[1].eof, 0);
    assert_eq!(written.buffer_offsets[2].eof, 1);
    assert_eq!(written.buffer_offsets[0].offset, 0);
    assert_eq!(
        written.buffer_offsets[1].offset,
        written.buffer_offsets[0].length
    );
    assert_eq!(
        written.buffer_offsets[2].offset,
        written.buffer_offsets[0].length + written.buffer_offsets[1].length
    );
    assert_eq!(written.buffer_offsets[2].offset % 16, 0);
    assert_eq!(written.string(0).unwrap(), "");

    let assignment =
        &written.mesh_format_assignments[usize::from(written.meshes[0].mesh_format_id)];
    let first = usize::from(assignment.first_mesh_format_id);
    let end = first + usize::from(assignment.mesh_format_entry_count);
    let formats = &written.mesh_formats[first..end];
    let buffer0: Vec<_> = formats
        .iter()
        .filter(|format| format.buffer_id == 0)
        .collect();
    assert_eq!(buffer0.len(), 1);
    assert_eq!(buffer0[0].buffer_offset_increment, 12);
    let data_stride = formats
        .iter()
        .find(|format| format.buffer_id == 1)
        .unwrap()
        .buffer_offset_increment;
    assert!(
        formats
            .iter()
            .filter(|format| format.buffer_id == 1)
            .all(|format| format.buffer_offset_increment == data_stride)
    );
}

#[test]
fn unboxed_group_gets_a_computed_box() {
    let vertices = MeshVertices {
        positions: vec![[0.0, 0.0, 0.0], [-1.0, 2.0, 0.5], [3.0, -2.0, 1.0]],
        ..MeshVertices::default()
    };
    let mut model = Model {
        bones: Vec::new(),
        materials: vec![MaterialInstance {
            name: "mat".to_owned(),
            shader: "shader".to_owned(),
            technique: "technique".to_owned(),
            textures: Vec::new(),
            parameters: Vec::new(),
        }],
        meshes: vec![Mesh {
            vertices,
            faces: vec![[0, 1, 2]],
            bone_group: Vec::new(),
            material: 0,
            alpha_flags: 0,
            shadow_flags: 0,
            has_antiblur_meshes: false,
            is_antiblur_mesh: false,
            custom_bounding_box: None,
        }],
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
    };
    let written = model.to_file().unwrap();
    let again = Model::from_file(&written).unwrap();
    let bounding_box = again.mesh_groups[0].bounding_box.unwrap();
    assert_eq!(bounding_box.max, [3.0, 2.0, 1.0, 1.0]);
    assert_eq!(bounding_box.min, [-1.0, -2.0, 0.0, 1.0]);
    // The source group keeps its `None`; only the file carries the box.
    assert_eq!(model.mesh_groups[0].bounding_box, None);
    model.mesh_groups[0].bounding_box = Some(bounding_box);
    assert_eq!(again, model);
}

#[test]
fn to_file_errors() {
    let mut too_many = model(HIGHNECK);
    too_many.meshes[0].bone_group = vec![0; 33];
    assert!(matches!(
        too_many.to_file(),
        Err(FmdlError::TooManyBones(33))
    ));

    let mut bad_material = model(HIGHNECK);
    bad_material.meshes[0].material = 9;
    assert!(matches!(
        bad_material.to_file(),
        Err(FmdlError::BadReference {
            what: "material instance",
            index: 9,
        })
    ));

    // A name past the u16 string-length field overflows instead of
    // truncating.
    let mut long_name = model(HIGHNECK);
    long_name.bones[0].name = "x".repeat(70_000);
    assert!(matches!(
        long_name.to_file(),
        Err(FmdlError::TableOverflow {
            what: "string",
            count: 70_000
        })
    ));
}

// --- validate ---------------------------------------------------------------

#[test]
fn every_fixture_model_validates() {
    for bytes in FIXTURES {
        model(bytes).validate().unwrap();
    }
}

#[test]
fn validate_rejects_dangling_bone_and_group_indices() {
    // A bone parent past the end of the list.
    let mut bad = model(HIGHNECK);
    bad.bones[0].parent = Some(bad.bones.len());
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::BadReference {
            what: "bone",
            index: 7
        })
    ));

    // A bone-parent loop.
    let mut bad = model(HIGHNECK);
    bad.bones[0].parent = Some(0);
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::ParentCycle("bone"))
    ));

    // A mesh-group parent past the end of the list.
    let mut bad = model(HIGHNECK);
    bad.mesh_groups[0].parent = Some(1);
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::BadReference {
            what: "mesh group",
            index: 1
        })
    ));

    // A mesh-group parent loop.
    let mut bad = model(HIGHNECK);
    bad.mesh_groups[0].parent = Some(0);
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::ParentCycle("mesh group"))
    ));

    // A group naming a mesh that does not exist.
    let mut bad = model(HIGHNECK);
    bad.mesh_groups[0].meshes.push(9);
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::BadReference {
            what: "mesh",
            index: 9
        })
    ));
}

#[test]
fn validate_rejects_bad_mesh_references() {
    // A material past the end of the list.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].material = 9;
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::BadReference {
            what: "material instance",
            index: 9
        })
    ));

    // A bone-group entry past the end of the bone list.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].bone_group.push(bad.bones.len());
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::BadReference {
            what: "bone",
            index: 7
        })
    ));

    // A face index at the vertex count.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].faces[0][0] = u16::try_from(bad.meshes[0].vertices.positions.len()).unwrap();
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::BadReference {
            what: "face vertex",
            index: 204
        })
    ));
}

#[test]
fn validate_rejects_misshapen_vertices() {
    // An attribute vector one value short.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].vertices.normals.as_mut().unwrap().pop();
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::VertexMismatch("attribute count mismatch"))
    ));

    // A uv map one value short.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].vertices.uvs[0].pop();
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::VertexMismatch("attribute count mismatch"))
    ));

    // Five uv maps.
    let mut bad = model(HIGHNECK);
    let count = bad.meshes[0].vertices.positions.len();
    bad.meshes[0].vertices.uvs = vec![vec![[0.0, 0.0]; count]; 5];
    bad.meshes[0].vertices.uv_high_precision = vec![false; 5];
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::InvalidVertexFormat("more than four uv maps"))
    ));

    // Fewer precision flags than maps.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].vertices.uv_high_precision.pop();
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::InvalidVertexFormat(
            "uv precision flags do not match uv maps"
        ))
    ));

    // Weights without indices.
    let mut bad = model(HIGHNECK);
    bad.meshes[0].vertices.bone_indices = None;
    assert!(matches!(
        bad.validate(),
        Err(FmdlError::InvalidVertexFormat(
            "bone weights and bone indices must come together"
        ))
    ));
}

#[test]
fn from_file_rejects_a_face_index_at_the_vertex_count() {
    let mut file = FmdlFile::read(HIGHNECK).unwrap();
    let mut faces = file.decode_faces(0).unwrap();
    faces[0][0] = file.meshes[0].vertex_count;
    file.encode_faces(0, &faces).unwrap();
    assert!(matches!(
        Model::from_file(&file),
        Err(FmdlError::BadReference {
            what: "face vertex",
            index: 204
        })
    ));
}

#[test]
fn to_file_refuses_bad_group_assignment() {
    // Mesh 0 in two groups.
    let mut bad = model(HIGHNECK);
    bad.mesh_groups.push(MeshGroup {
        name: "second".to_owned(),
        parent: None,
        meshes: vec![0],
        bounding_box: None,
        visible: true,
        split_mesh_group: false,
    });
    assert!(matches!(
        bad.to_file(),
        Err(FmdlError::BadMeshGroupAssignment(
            "mesh assigned to two groups"
        ))
    ));

    // Mesh 0 in no group.
    let mut bad = model(HIGHNECK);
    bad.mesh_groups[0].meshes.clear();
    assert!(matches!(
        bad.to_file(),
        Err(FmdlError::BadMeshGroupAssignment(
            "mesh not assigned to a group"
        ))
    ));
}

#[test]
fn bone_parent_past_i16_overflows() {
    // A chain of 32770 bones: bone 32769's parent is 32768, one past the
    // signed 16-bit field's range.
    let mut model = plain_model();
    model.bones = (0usize..32_770)
        .map(|index| Bone {
            name: format!("bone{index}"),
            parent: index.checked_sub(1),
            bounding_box: BoundingBox {
                max: [0.0; 4],
                min: [0.0; 4],
            },
            local_position: [0.0; 4],
            world_position: [0.0; 4],
        })
        .collect();
    assert!(matches!(
        model.to_file(),
        Err(FmdlError::TableOverflow {
            what: "bones",
            count: 32_768
        })
    ));
}

#[test]
fn custom_bounding_boxes() {
    // A file whose tail marks mesh 0 reads that mesh with its group's box.
    let mut file = FmdlFile::read(HIGHNECK).unwrap();
    let mut table = file.string_table.clone().unwrap();
    table.extend_from_slice(b"X-FMDL-Extensions: \nCustom-Bounding-Box-Meshes: 0\n\0");
    file.string_table = Some(table);
    let marked = Model::from_file(&file).unwrap();
    assert_eq!(
        marked.meshes[0].custom_bounding_box,
        marked.mesh_groups[0].bounding_box
    );
    assert!(marked.meshes[0].custom_bounding_box.is_some());

    // It writes the header back and round-trips.
    let written = marked.to_file().unwrap();
    let tail = std::str::from_utf8(written.extension_tail()).unwrap();
    assert!(tail.contains("Custom-Bounding-Box-Meshes: 0"));
    assert_eq!(Model::from_file(&written).unwrap(), marked);

    // A group without a box uses a marked mesh's custom box (w forced to 1).
    let mut mesh = mesh_with(plain_vertices());
    mesh.custom_bounding_box = Some(BoundingBox {
        max: [10.0, 10.0, 10.0, 0.0],
        min: [-10.0, -10.0, -10.0, 0.0],
    });
    let file = grouped_model(vec![mesh.clone()], vec![vec![0]])
        .to_file()
        .unwrap();
    let assignment = file
        .mesh_group_assignments
        .iter()
        .find(|record| record.mesh_group_id == 0)
        .unwrap();
    let box_record = &file.bounding_boxes[usize::from(assignment.bounding_box_id)];
    assert_eq!(box_record.max, [10.0, 10.0, 10.0, 1.0]);
    assert_eq!(box_record.min, [-10.0, -10.0, -10.0, 1.0]);

    // A second, unmarked mesh in the group widens the box to the union.
    let wide = mesh_with(MeshVertices {
        positions: vec![[20.0, 0.0, 0.0], [0.0, 20.0, 0.0], [0.0, 0.0, 20.0]],
        ..MeshVertices::default()
    });
    let file = grouped_model(vec![mesh, wide], vec![vec![0, 1]])
        .to_file()
        .unwrap();
    let assignment = file
        .mesh_group_assignments
        .iter()
        .find(|record| record.mesh_group_id == 0)
        .unwrap();
    let box_record = &file.bounding_boxes[usize::from(assignment.bounding_box_id)];
    assert_eq!(box_record.max, [20.0, 20.0, 20.0, 1.0]);
    assert_eq!(box_record.min, [-10.0, -10.0, -10.0, 1.0]);
}

// --- to_file behavior pins --------------------------------------------------

#[test]
fn vertex_and_bone_group_limits() {
    // 65535 vertices fit the u16 count; 65536 do not.
    let vertices = MeshVertices {
        positions: vec![[0.0, 0.0, 0.0]; 65_535],
        ..MeshVertices::default()
    };
    grouped_model(vec![mesh_with(vertices)], vec![vec![0]])
        .to_file()
        .unwrap();
    let vertices = MeshVertices {
        positions: vec![[0.0, 0.0, 0.0]; 65_536],
        ..MeshVertices::default()
    };
    assert!(matches!(
        grouped_model(vec![mesh_with(vertices)], vec![vec![0]]).to_file(),
        Err(FmdlError::TooManyVertices(65_536))
    ));

    // 32 bone-group entries fit the fixed array; 33 do not.
    let skinned = || MeshVertices {
        positions: vec![[0.0, 0.0, 0.0]; 3],
        bone_weights: Some(vec![[255, 0, 0, 0]; 3]),
        bone_indices: Some(vec![[0, 0, 0, 0]; 3]),
        ..MeshVertices::default()
    };
    let mut model = grouped_model(vec![mesh_with(skinned())], vec![vec![0]]);
    model.bones = vec![Bone {
        name: "sk_root".to_owned(),
        parent: None,
        bounding_box: BoundingBox {
            max: [0.0; 4],
            min: [0.0; 4],
        },
        local_position: [0.0; 4],
        world_position: [0.0; 4],
    }];
    model.meshes[0].bone_group = vec![0; 32];
    model.to_file().unwrap();
    model.meshes[0].bone_group = vec![0; 33];
    assert!(matches!(model.to_file(), Err(FmdlError::TooManyBones(33))));
}

#[test]
fn identical_uv_maps_share_one_offset() {
    let map = vec![[0.5, 0.5]; 3];
    let write = |uvs: Vec<Vec<[f32; 2]>>, precision: Vec<bool>| {
        grouped_model(
            vec![mesh_with(MeshVertices {
                positions: vec![[0.0, 0.0, 0.0]; 3],
                uvs,
                uv_high_precision: precision,
                ..MeshVertices::default()
            })],
            vec![vec![0]],
        )
        .to_file()
        .unwrap()
    };
    let uv_offsets = |file: &FmdlFile| {
        file.vertex_formats
            .iter()
            .filter(|record| record.datum_type == 8 || record.datum_type == 9)
            .map(|record| record.offset)
            .collect::<Vec<_>>()
    };

    // Two identical maps at the same precision share one entry in the buffer.
    let shared = write(vec![map.clone(), map.clone()], vec![false, false]);
    assert_eq!(uv_offsets(&shared), &[0, 0]);
    let data = shared
        .mesh_formats
        .iter()
        .find(|record| record.buffer_id == 1)
        .unwrap();
    assert_eq!(data.buffer_offset_increment, 4);

    // Identical values at different precisions are two entries.
    let mixed = write(vec![map.clone(), map.clone()], vec![true, false]);
    assert_eq!(uv_offsets(&mixed), &[0, 8]);
    let data = mixed
        .mesh_formats
        .iter()
        .find(|record| record.buffer_id == 1)
        .unwrap();
    assert_eq!(data.buffer_offset_increment, 12);

    // Different values at the same precision are two entries.
    let other = vec![[0.25, 0.25]; 3];
    let distinct = write(vec![map, other], vec![false, false]);
    assert_eq!(uv_offsets(&distinct), &[0, 4]);
    let data = distinct
        .mesh_formats
        .iter()
        .find(|record| record.buffer_id == 1)
        .unwrap();
    assert_eq!(data.buffer_offset_increment, 8);
}

#[test]
fn a_minus_zero_uv_does_not_alias_plus_zero() {
    // == would call the maps equal; their bytes are not.
    let mut map = vec![[0.5, 0.5]; 3];
    map[1] = [0.0, 0.0];
    let mut negated = map.clone();
    negated[1] = [-0.0, 0.0];
    let file = grouped_model(
        vec![mesh_with(MeshVertices {
            positions: vec![[0.0, 0.0, 0.0]; 3],
            uvs: vec![map, negated],
            uv_high_precision: vec![true, true],
            ..MeshVertices::default()
        })],
        vec![vec![0]],
    )
    .to_file()
    .unwrap();
    let offsets: Vec<_> = file
        .vertex_formats
        .iter()
        .filter(|record| record.datum_type == 8 || record.datum_type == 9)
        .map(|record| record.offset)
        .collect();
    assert_eq!(offsets.len(), 2);
    assert_ne!(offsets[0], offsets[1]);
    let back = Model::from_file(&file).unwrap();
    assert_eq!(
        back.meshes[0].vertices.uvs[1][1][0].to_bits(),
        (-0.0f32).to_bits()
    );
}

#[test]
fn a_zero_vertex_skinned_mesh_keeps_its_layout() {
    let mut model = grouped_model(
        vec![mesh_with(MeshVertices {
            positions: Vec::new(),
            normals: Some(Vec::new()),
            uvs: vec![Vec::new()],
            uv_high_precision: vec![true],
            bone_weights: Some(Vec::new()),
            bone_indices: Some(Vec::new()),
            ..MeshVertices::default()
        })],
        vec![vec![0]],
    );
    model.meshes[0].faces = Vec::new();
    model.bones = (0usize..3)
        .map(|index| Bone {
            name: format!("bone{index}"),
            parent: index.checked_sub(1),
            bounding_box: BoundingBox {
                max: [0.0; 4],
                min: [0.0; 4],
            },
            local_position: [0.0; 4],
            world_position: [0.0; 4],
        })
        .collect();
    model.meshes[0].bone_group = vec![0, 1, 2];
    // A group with no vertices computes no box, and an existing
    // bone-matrices block reads back as an empty one.
    model.mesh_groups[0].bounding_box = Some(BoundingBox {
        max: [0.0; 4],
        min: [0.0; 4],
    });
    model.bone_matrices = Some(Vec::new());

    let mut file = model.to_file().unwrap();
    let back = Model::from_file(&file).unwrap();
    // The declared layout survives: every attribute is Some(empty), the
    // uv precision flag and the bone group included.
    assert_eq!(back, model);
    // The decoded vertices still encode into the file.
    file.encode_vertices(0, &back.meshes[0].vertices).unwrap();
}

#[test]
fn a_groups_box_covers_its_children() {
    // Group 0 holds mesh 0 (x 0..1); its child group 1 holds mesh 1
    // (x 0..11). The parent's written box covers both.
    let mut model = grouped_model(
        vec![
            mesh_with(plain_vertices()),
            mesh_with(MeshVertices {
                positions: vec![[0.0; 3], [11.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                ..MeshVertices::default()
            }),
        ],
        vec![vec![0], vec![1]],
    );
    model.mesh_groups[1].parent = Some(0);
    let group0_box = |file: &FmdlFile| {
        let assignment = &file.mesh_group_assignments[0];
        file.bounding_boxes[usize::from(assignment.bounding_box_id)].max[0]
    };
    assert_eq!(group0_box(&model.to_file().unwrap()), 11.0);

    // A child's explicit box wins over its meshes' extent.
    model.mesh_groups[1].bounding_box = Some(BoundingBox {
        max: [20.0, 0.0, 0.0, 1.0],
        min: [0.0; 4],
    });
    assert_eq!(group0_box(&model.to_file().unwrap()), 20.0);
}

#[test]
fn a_childless_groups_box_flows_through_an_empty_middle_group() {
    // Group 0 holds mesh 0 (x 0..1); its child group 1 is empty and
    // unboxed; its grandchild group 2 holds mesh 1 (x 0..11).
    let mut model = grouped_model(
        vec![
            mesh_with(plain_vertices()),
            mesh_with(MeshVertices {
                positions: vec![[0.0; 3], [11.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                ..MeshVertices::default()
            }),
        ],
        vec![vec![0], vec![], vec![1]],
    );
    model.mesh_groups[1].parent = Some(0);
    model.mesh_groups[2].parent = Some(1);
    let file = model.to_file().unwrap();
    // The empty middle group writes no assignment; records are in group
    // order, so [0] is group 0's and its box must reach x = 11.
    assert_eq!(file.mesh_group_assignments.len(), 2);
    let id = usize::from(file.mesh_group_assignments[0].bounding_box_id);
    assert_eq!(file.bounding_boxes[id].max[0], 11.0);
}

#[test]
fn consecutive_meshes_write_one_assignment() {
    let model = grouped_model(
        vec![
            mesh_with(plain_vertices()),
            mesh_with(plain_vertices()),
            mesh_with(plain_vertices()),
        ],
        vec![vec![0, 1, 2]],
    );
    let file = model.to_file().unwrap();
    assert_eq!(file.mesh_group_assignments.len(), 1);
    let record = &file.mesh_group_assignments[0];
    assert_eq!(record.first_mesh_id, 0);
    assert_eq!(record.mesh_count, 3);
}

#[test]
fn a_flat_extreme_is_not_the_zero_box() {
    // Every vertex sits at x = 5: max == min, but that is a real extent,
    // not "nothing to measure".
    let model = grouped_model(
        vec![mesh_with(MeshVertices {
            positions: vec![[5.0; 3]; 3],
            ..MeshVertices::default()
        })],
        vec![vec![0]],
    );
    let file = model.to_file().unwrap();
    let id = usize::from(file.mesh_group_assignments[0].bounding_box_id);
    assert_eq!(file.bounding_boxes[id].max[0], 5.0);
    assert_eq!(file.bounding_boxes[id].min[0], 5.0);
}

#[test]
fn an_empty_mesh_format_group_writes_no_record() {
    // An unskinned mesh with normals only: positions plus one data entry.
    let file = grouped_model(
        vec![mesh_with(MeshVertices {
            positions: vec![[0.0, 0.0, 0.0]; 3],
            normals: Some(vec![[0.0, 0.0, 1.0, 0.0]; 3]),
            ..MeshVertices::default()
        })],
        vec![vec![0]],
    )
    .to_file()
    .unwrap();
    assert_eq!(file.mesh_formats.len(), 2);
}

#[test]
fn written_record_details() {
    let file = model(HIGHNECK).to_file().unwrap();
    assert_eq!(file.meshes[0].face_vertex_count, 320 * 3);
    for record in &file.mesh_groups {
        assert_eq!(record.unknown_0x06, -1);
    }
    assert_eq!(&file.block_20[0].bytes[28..32], &(-1i32).to_le_bytes());
}

#[test]
fn non_adjacent_meshes_write_one_assignment_each() {
    // Group 0 lists meshes 0 and 2, group 1 lists mesh 1: two runs.
    let file = grouped_model(
        vec![
            mesh_with(plain_vertices()),
            mesh_with(plain_vertices()),
            mesh_with(plain_vertices()),
        ],
        vec![vec![0, 2], vec![1]],
    )
    .to_file()
    .unwrap();
    let group0: Vec<_> = file
        .mesh_group_assignments
        .iter()
        .filter(|record| record.mesh_group_id == 0)
        .collect();
    assert_eq!(group0.len(), 2);
    assert_eq!(group0[0].first_mesh_id, 0);
    assert_eq!(group0[0].mesh_count, 1);
    assert_eq!(group0[1].first_mesh_id, 2);
    assert_eq!(group0[1].mesh_count, 1);
}

#[test]
fn the_extension_tail_tracks_the_headers() {
    // No flags and no per-object headers: nothing after the last string.
    let file = plain_model().to_file().unwrap();
    assert!(file.extension_tail().is_empty());

    // One per-object header with no flags.
    let mut model = plain_model();
    model.meshes[0].has_antiblur_meshes = true;
    let file = model.to_file().unwrap();
    assert!(file.extension_tail().starts_with(b"X-FMDL-Extensions: \n"));
}

#[test]
fn the_bone_matrix_block_tracks_the_bone_list() {
    // Bones present and no matrices: the writer emits an empty block.
    let mut model = model(HIGHNECK);
    model.bone_matrices = None;
    let file = model.to_file().unwrap();
    assert_eq!(file.bone_matrices, Some(Vec::new()));

    // No bones and no matrices: no block at all.
    let file = plain_model().to_file().unwrap();
    assert_eq!(file.bone_matrices, None);
}

#[test]
fn a_group_with_nonconsecutive_meshes_reloads() {
    // A group's non-consecutive mesh list is written as one assignment
    // record per run, all naming the group's one bounding box; reading it
    // back must accept the repeated assignment.
    let boxed = BoundingBox {
        max: [10.0, 10.0, 10.0, 1.0],
        min: [-10.0, -10.0, -10.0, 1.0],
    };
    let mut model = grouped_model(
        vec![
            mesh_with(plain_vertices()),
            mesh_with(plain_vertices()),
            mesh_with(plain_vertices()),
        ],
        vec![vec![0, 2], vec![1]],
    );
    for group in &mut model.mesh_groups {
        group.bounding_box = Some(boxed);
    }
    let file = model.to_file().unwrap();
    // Three records: group 0's runs [0] and [2], group 1's [1].
    assert_eq!(file.mesh_group_assignments.len(), 3);
    let again = Model::from_file(&file).unwrap();
    assert_eq!(again, model);

    // A second assignment naming a *different* bounding box is refused.
    let mut file = model.to_file().unwrap();
    let other = file.mesh_group_assignments[2].bounding_box_id;
    file.mesh_group_assignments[1].bounding_box_id = other;
    assert!(matches!(
        Model::from_file(&file),
        Err(FmdlError::BadMeshGroupAssignment(
            "mesh group assigned two bounding boxes"
        ))
    ));
}

#[test]
fn a_split_mesh_among_siblings_reloads() {
    // [small, skinned-over-the-limit, small] in one group: the split's
    // components move to a child split group, leaving the group's own
    // meshes non-consecutive.
    let boxed = BoundingBox {
        max: [10.0, 10.0, 10.0, 1.0],
        min: [-10.0, -10.0, -10.0, 1.0],
    };
    let mut big = mesh_with(MeshVertices {
        positions: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        bone_weights: Some(vec![[255, 0, 0, 0]; 3]),
        bone_indices: Some(vec![[0; 4]; 3]),
        ..MeshVertices::default()
    });
    big.bone_group = (0..40).collect();
    let mut model = grouped_model(
        vec![
            mesh_with(plain_vertices()),
            big,
            mesh_with(plain_vertices()),
        ],
        vec![vec![0, 1, 2]],
    );
    model.bones = (0usize..40)
        .map(|index| Bone {
            name: format!("bone{index}"),
            parent: index.checked_sub(1),
            bounding_box: BoundingBox {
                max: [0.0; 4],
                min: [0.0; 4],
            },
            local_position: [0.0; 4],
            world_position: [0.0; 4],
        })
        .collect();
    model.bone_matrices = Some(Vec::new());
    for group in &mut model.mesh_groups {
        group.bounding_box = Some(boxed);
    }
    crate::ops::split::encode(&mut model, None).unwrap();
    // The group now names its siblings apart: [0, 2].
    assert_eq!(model.mesh_groups[0].meshes, [0, 2]);
    let file = model.to_file().unwrap();
    let again = Model::from_file(&file).unwrap();
    assert_eq!(again, model);
}

#[test]
fn a_group_with_no_vertices_gets_a_zero_box() {
    // Nothing to measure: the written box is zero, not an infinite one.
    let empty = MeshVertices::default();
    let mut mesh = mesh_with(empty);
    mesh.faces.clear();
    let file = grouped_model(vec![mesh], vec![vec![0]]).to_file().unwrap();
    let assignment = &file.mesh_group_assignments[0];
    let written = &file.bounding_boxes[usize::from(assignment.bounding_box_id)];
    assert_eq!(written.max, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(written.min, [0.0, 0.0, 0.0, 1.0]);
}
