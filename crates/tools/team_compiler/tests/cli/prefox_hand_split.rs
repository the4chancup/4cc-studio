//! `compile` for PES 2017: the hand auto-split of a face model whose vertices weigh on the
//! hand bones, its hands packed into the player's face CPK as two more models, typed `gloveL`
//! and `gloveR` in the generated `face.xml`.

use std::fs;
use std::path::Path;

use pes_model::format::PreFoxModel;
use pes_model::model::Model;

use crate::common::Sandbox;
use crate::findings_of;
use crate::prefox_faces::{compile_pes17, face_cpk, face_folder, nested_entries, pes17};

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

/// The (type, path, material) of every `<model>` of the `face.xml` `bytes`, in file order.
fn ordered_entries(bytes: &[u8]) -> Vec<(String, String, String)> {
    let text = std::str::from_utf8(bytes).unwrap();
    let document = roxmltree::Document::parse(text).unwrap();
    document
        .root_element()
        .children()
        .filter(|node| node.has_tag_name("model"))
        .map(|node| {
            let attribute = |name: &str| node.attribute(name).unwrap().to_owned();
            (attribute("type"), attribute("path"), attribute("material"))
        })
        .collect()
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
    let split = [
        "oral_body_win32.model",
        "oral_body_glove_l_win32.model",
        "oral_body_glove_r_win32.model",
    ]
    .map(|name| counts(&face[&format!("{folder}{name}")]));
    assert_eq!(split, [(21, 24), (9, 8), (9, 8)]);
    assert_eq!(
        split.iter().map(|(_, faces)| faces).sum::<usize>(),
        counts(&hand_split_fixture("body.model")).1,
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
fn a_hand_weighted_model_whose_mtl_is_a_common_file_fails_its_folder() {
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

    let compile = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);
    let lines = compile.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Error model_conversion_failed [DropFolder] at Players/05 - A (model=body.model, error=its .mtl, Common/body.mtl, is a Common file, which the face does not read)",
        ],
        "{lines:#?}"
    );
    assert_eq!(compile.exit_code(), 1, "{lines:#?}");
    let entries = crate::compile::cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(!entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
}
