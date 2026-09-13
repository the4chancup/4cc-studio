use super::*;
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
        let container = crate::FmdlContainer::read(&written).unwrap();
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
    too_many.meshes[0].bone_group = (0..33).collect();
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
}
