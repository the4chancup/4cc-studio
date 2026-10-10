//! `compile` for PES 2017: the hand auto-split of a face model whose vertices weigh on the
//! hand bones, its hands packed into the player's face CPK as two more models, typed `gloveL`
//! and `gloveR` in the generated `face.xml`.

use std::fs;
use std::path::Path;

use pes_model::format::PreFoxModel;
use pes_model::model::Model;

use crate::common::Sandbox;
use crate::compile::tracer_kit;
use crate::prefox_faces::{
    compile_pes17, face_cpk, face_folder, nested_entries, ordered_entries, pes17,
};
use crate::{Face, findings_of, sorted_faces};

/// The `hand_split` fixture `name`: the full-body strip with both hands on, as a pre-Fox
/// pair (`tests/fixtures/hand_split/README.md`).
fn hand_split_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hand_split")
            .join(name),
    )
    .unwrap()
}

/// The faces of the `.model` `bytes`, read back with `pes_model`, each in the `.model`'s own
/// winding, sorted (`sorted_faces`).
fn faces(bytes: &[u8]) -> Vec<Face> {
    let model = Model::from_file(&PreFoxModel::read(bytes).unwrap()).unwrap();
    sorted_faces(
        model
            .meshes
            .iter()
            .map(|mesh| (mesh.vertices.positions.as_slice(), mesh.faces.as_slice())),
    )
}

/// The (vertices, faces) of the `.model` `bytes`, read back with `pes_model`.
fn counts(bytes: &[u8]) -> (usize, usize) {
    let model = Model::from_file(&PreFoxModel::read(bytes).unwrap()).unwrap();
    model.meshes.iter().fold((0, 0), |(vertices, faces), mesh| {
        (
            vertices + mesh.vertices.positions.len(),
            faces + mesh.faces.len(),
        )
    })
}

// TC-MOD-43
#[test]
fn a_face_model_weighted_to_the_hand_bones_gives_its_face_two_glove_entries() {
    let sandbox = Sandbox::new("prefox_hand_split");
    let export = "co Midcup Hands";
    let player = format!("exports/{export}/Players/05 - A");
    // A full-body model with both hands on: 40 faces, of which each hand's split takes 8 and
    // the body keeps 24.
    sandbox.write(
        &format!("{player}/body.model"),
        &hand_split_fixture("body.model"),
    );
    sandbox.write(
        &format!("{player}/body.mtl"),
        &hand_split_fixture("body.mtl"),
    );

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info model_hand_split [Keep] at Players/05 - A (model=body.model, gloves=glove_l, glove_r)",
            "Info xml_face_neck_added [Keep] at Players/05 - A ()",
        ],
    );

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "body.mtl",
            "dummy.mtl",
            "face.xml",
            "oral_body_glove_l_win32.model",
            "oral_body_glove_r_win32.model",
            "oral_body_win32.model",
            "oral_dummy_win32.model",
        ]
    );
    let entry = |xml_type: &str, path: &str, material: &str| {
        (xml_type.to_owned(), path.to_owned(), material.to_owned())
    };
    assert_eq!(
        ordered_entries(&face[&format!("{folder}face.xml")]),
        [
            entry("parts", "./oral_body_*.model", "./body.mtl"),
            entry("gloveL", "./oral_body_glove_l_*.model", "./body.mtl"),
            entry("gloveR", "./oral_body_glove_r_*.model", "./body.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
    let parts = [
        "oral_body_win32.model",
        "oral_body_glove_l_win32.model",
        "oral_body_glove_r_win32.model",
    ]
    .map(|name| &face[&format!("{folder}{name}")]);
    assert_eq!(parts.map(|part| counts(part)), [(21, 24), (9, 8), (9, 8)]);
    // Source and parts are all `.model` files, compared in that format's winding.
    let mut packed: Vec<Face> = parts.iter().flat_map(|part| faces(part)).collect();
    packed.sort_unstable();
    assert_eq!(
        packed,
        faces(&hand_split_fixture("body.model")),
        "every face of the source, once"
    );
    assert!(
        !entries
            .keys()
            .any(|path| path.starts_with("common/character0/model/character/glove/")),
        "{:?}",
        entries.keys()
    );

    // `check` has nothing to say about the folder.
    let check = sandbox.run(&pes17(&sandbox), &["check"]);
    let lines = check.messages();
    assert_eq!(
        findings_of(&lines, export),
        ["Info export_identified [Keep] (team=/co/, id=714)"],
        "{lines:#?}"
    );
    assert_eq!(check.exit_code(), 0, "{lines:#?}");
}

#[test]
fn a_linked_face_folder_s_hand_split_model_is_named_by_its_export_path() {
    let sandbox = Sandbox::new("prefox_hand_split_shared");
    let export = "co Midcup Hands";
    let round = format!("exports/{export}/Faces/Round");
    sandbox.write(
        &format!("{round}/body.model"),
        &hand_split_fixture("body.model"),
    );
    sandbox.write(
        &format!("{round}/body.mtl"),
        &hand_split_fixture("body.mtl"),
    );
    sandbox.write(&format!("exports/{export}/Players/05 - A/Round.face"), b"");

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
            "Info model_hand_split [Keep] at Players/05 - A (model=Faces/Round/body.model, gloves=glove_l, glove_r)",
            "Info xml_face_neck_added [Keep] at Players/05 - A ()",
        ],
    );

    // Copied in under his face as his own would be.
    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let entry = |xml_type: &str, path: &str, material: &str| {
        (xml_type.to_owned(), path.to_owned(), material.to_owned())
    };
    assert_eq!(
        ordered_entries(&face[&format!("{folder}face.xml")]),
        [
            entry("parts", "./oral_body_*.model", "./body.mtl"),
            entry("gloveL", "./oral_body_glove_l_*.model", "./body.mtl"),
            entry("gloveR", "./oral_body_glove_r_*.model", "./body.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
}

#[test]
fn a_hand_weighted_per_kit_set_lists_its_hands_once_as_its_reference() {
    let sandbox = Sandbox::new("prefox_hand_split_kit_variants");
    let export = "co Midcup Hands";
    let player = format!("exports/{export}/Players/05 - A");
    for name in ["body_kit1.model", "body_kit2.model"] {
        sandbox.write(
            &format!("{player}/{name}"),
            &hand_split_fixture("body.model"),
        );
    }
    sandbox.write(
        &format!("{player}/body.mtl"),
        &hand_split_fixture("body.mtl"),
    );

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info model_hand_split [Keep] at Players/05 - A (model=body_kit1.model, gloves=glove_l, glove_r)",
            "Info model_hand_split [Keep] at Players/05 - A (model=body_kit2.model, gloves=glove_l, glove_r)",
            "Info xml_face_neck_added [Keep] at Players/05 - A ()",
        ],
    );

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "body.mtl",
            "dummy.mtl",
            "face.xml",
            "oral_body_kit1_glove_l_win32.model",
            "oral_body_kit1_glove_r_win32.model",
            "oral_body_kit1_win32.model",
            "oral_body_kit2_glove_l_win32.model",
            "oral_body_kit2_glove_r_win32.model",
            "oral_body_kit2_win32.model",
            "oral_dummy_win32.model",
        ]
    );
    let entry = |xml_type: &str, path: &str, material: &str| {
        (xml_type.to_owned(), path.to_owned(), material.to_owned())
    };
    assert_eq!(
        ordered_entries(&face[&format!("{folder}face.xml")]),
        [
            entry("parts", "./oral_body_kitN_*.model", "./body.mtl"),
            entry("gloveL", "./oral_body_kitN_glove_l_*.model", "./body.mtl"),
            entry("gloveR", "./oral_body_kitN_glove_r_*.model", "./body.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
}

#[test]
fn a_hand_weighted_per_kit_set_lacking_a_kit_number_gets_its_lowest_body_and_hands() {
    let sandbox = Sandbox::new("prefox_hand_split_kit_variant_missing");
    let export = "co Midcup Hands";
    let player = format!("exports/{export}/Players/05 - A");
    for name in ["body_kit1.model", "body_kit2.model"] {
        sandbox.write(
            &format!("{player}/{name}"),
            &hand_split_fixture("body.model"),
        );
    }
    sandbox.write(
        &format!("{player}/body.mtl"),
        &hand_split_fixture("body.mtl"),
    );
    for kit in ["p1", "p2", "p3"] {
        sandbox.write(
            &format!("exports/{export}/Kits/{kit}/kit.dds"),
            &tracer_kit(),
        );
    }

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_config_generated [Keep] at Kits/p3 ()",
            "Info model_hand_split [Keep] at Players/05 - A (model=body_kit1.model, gloves=glove_l, glove_r)",
            "Info model_hand_split [Keep] at Players/05 - A (model=body_kit2.model, gloves=glove_l, glove_r)",
            "Warning kit_variant_missing [Keep] at Players/05 - A (model=body_kitN, kit=3, copied=body_kit1)",
            "Info xml_face_neck_added [Keep] at Players/05 - A ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
            "Info kit_colors_derived [Keep] at Kits/p3 ()",
        ],
    );

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    // The shared `body.mtl` serves every number as it is.
    assert_eq!(
        names,
        [
            "body.mtl",
            "dummy.mtl",
            "face.xml",
            "oral_body_kit1_glove_l_win32.model",
            "oral_body_kit1_glove_r_win32.model",
            "oral_body_kit1_win32.model",
            "oral_body_kit2_glove_l_win32.model",
            "oral_body_kit2_glove_r_win32.model",
            "oral_body_kit2_win32.model",
            "oral_body_kit3_glove_l_win32.model",
            "oral_body_kit3_glove_r_win32.model",
            "oral_body_kit3_win32.model",
            "oral_dummy_win32.model",
        ]
    );
    for part in ["", "_glove_l", "_glove_r"] {
        let packed = |kit: u8| &face[&format!("{folder}oral_body_kit{kit}{part}_win32.model")];
        assert_eq!(packed(3), packed(1), "{part}");
    }
}

#[test]
fn a_hand_weighted_model_whose_mtl_is_a_common_file_is_split_with_it() {
    let sandbox = Sandbox::new("prefox_hand_split_common_mtl");
    let export = "co Midcup Hands";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/body.model"),
        &hand_split_fixture("body.model"),
    );
    sandbox.write(&format!("{player}/body.mtl.common"), b"");
    sandbox.write(
        &format!("exports/{export}/Common/body.mtl"),
        &hand_split_fixture("body.mtl"),
    );

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info model_hand_split [Keep] at Players/05 - A (model=body.model, gloves=glove_l, glove_r)",
            "Info xml_face_neck_added [Keep] at Players/05 - A ()",
        ],
    );

    // The body and its two hands, as with a local `.mtl`; the Common `.mtl` is the Common
    // output's, not the face's.
    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "dummy.mtl",
            "face.xml",
            "oral_body_glove_l_win32.model",
            "oral_body_glove_r_win32.model",
            "oral_body_win32.model",
            "oral_dummy_win32.model",
        ]
    );
    let entry = |xml_type: &str, path: &str, material: &str| {
        (xml_type.to_owned(), path.to_owned(), material.to_owned())
    };
    // Each entry names the Common `.mtl` by its path in the team's Common output, as an
    // unsplit model's entry does.
    let common_mtl = "model/character/uniform/common/714/body.mtl";
    assert_eq!(
        ordered_entries(&face[&format!("{folder}face.xml")]),
        [
            entry("parts", "./oral_body_*.model", common_mtl),
            entry("gloveL", "./oral_body_glove_l_*.model", common_mtl),
            entry("gloveR", "./oral_body_glove_r_*.model", common_mtl),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
    let split = [
        "oral_body_win32.model",
        "oral_body_glove_l_win32.model",
        "oral_body_glove_r_win32.model",
    ]
    .map(|name| counts(&face[&format!("{folder}{name}")]));
    assert_eq!(split, [(21, 24), (9, 8), (9, 8)]);
}
