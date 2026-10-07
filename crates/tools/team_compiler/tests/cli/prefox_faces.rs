//! `compile` for PES 2017: a player folder's own `.model` files, with their `.mtl` files and
//! textures, built into the player's face CPK nested in the output CPK, with the generated
//! `face.xml` typing every model, and the blank face of a player folder with no model.

use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes_settings, pes21_settings, tracer_player_file};
use crate::models::template;
use crate::{clean_model, findings_of};

/// The outer CPK path of slot `slot`'s face CPK in team 714's export.
fn face_cpk(slot: u8) -> String {
    format!("common/character0/model/character/face/real/714{slot:02}.cpk")
}

/// The folder every entry of slot `slot`'s face CPK sits in.
fn face_folder(slot: u8) -> String {
    format!("common/character0/model/character/face/real/714{slot:02}/")
}

/// The bytes of `pes_model`'s fixture `name`: real pre-Fox models and material sets.
fn pre_fox_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../libs/pes_model/tests/fixtures")
            .join(name),
    )
    .unwrap()
}

/// The card-head template's face model, a clean pre-Fox model with one material, `card`.
fn card_model() -> Vec<u8> {
    pre_fox_fixture("cardhead_face_high.model")
}

/// The card-head template's material set, its one texture path `./texture.dds` renamed to
/// `./skin.dds`, the texture the tests write beside it.
fn card_materials() -> Vec<u8> {
    let text = String::from_utf8(pre_fox_fixture("cardhead_materials.mtl")).unwrap();
    assert!(text.contains("./texture.dds"), "{text}");
    text.replace("./texture.dds", "./skin.dds").into_bytes()
}

/// A small DDS, 12x12 BC3 with one level (`tests/fixtures/textures/single_level.dds`).
fn small_dds() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/textures")
            .join("single_level.dds"),
    )
    .unwrap()
}

/// The PES 17 settings of `sandbox`, whose game folder does not exist.
fn pes17(sandbox: &Sandbox) -> String {
    pes_settings(sandbox, 17)
}

/// Runs `compile --no-deploy` for PES 17, asserts the export `name`'s findings are `findings`
/// and the run succeeded, and returns the output CPK's entries.
fn compile_pes17(sandbox: &Sandbox, name: &str, findings: &[&str]) -> BTreeMap<String, Vec<u8>> {
    let run = sandbox.run(&pes17(sandbox), &["compile", "--no-deploy"]);
    let lines = run.messages();
    assert_eq!(findings_of(&lines, name), findings, "{lines:#?}");
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"))
}

/// Every entry of the nested CPK `bytes`, by its path, with its bytes.
fn nested_entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut cpk = cpk::CpkArchive::open(Cursor::new(bytes)).unwrap();
    let entries = cpk.entries().to_vec();
    entries
        .iter()
        .map(|entry| (entry.path.clone(), cpk.read(entry).unwrap()))
        .collect()
}

/// A `face.xml` entry as the test expects it: type, path, material and ratio.
type Entry<'a> = (&'a str, &'a str, &'a str, Option<&'a str>);

/// The `face.xml` the compiler generates for `entries` and the face diff `dif`, written out
/// here on its own: the XML declaration, `<config>`, one three-space-indented `<model>` per
/// entry, the `<dif>` base64 on one line, CRLF line ends, no final line end.
fn expected_face_xml(entries: &[Entry], dif: &[u8]) -> Vec<u8> {
    let mut text = String::from("<?xml version='1.0' encoding='UTF-8'?>\r\n<config>\r\n");
    for (xml_type, path, material, ratio) in entries {
        let ratio = ratio.map_or(String::new(), |ratio| format!(" ratio=\"{ratio}\""));
        text.push_str(&format!(
            "   <model level=\"0\" type=\"{xml_type}\" path=\"{path}\" material=\"{material}\"{ratio} />\r\n"
        ));
    }
    text.push_str(&format!(
        "<dif>\r\n{}\r\n</dif>\r\n</config>",
        STANDARD.encode(dif)
    ));
    text.into_bytes()
}

/// The (type, ratio, material) of every `<model>` of the `face.xml` `bytes`, sorted.
fn model_entries(bytes: &[u8]) -> Vec<(String, Option<String>, String)> {
    let text = std::str::from_utf8(bytes).unwrap();
    let document = roxmltree::Document::parse(text).unwrap();
    let mut entries: Vec<(String, Option<String>, String)> = document
        .root_element()
        .children()
        .filter(|node| node.has_tag_name("model"))
        .map(|node| {
            (
                node.attribute("type").unwrap().to_owned(),
                node.attribute("ratio").map(str::to_owned),
                node.attribute("material").unwrap().to_owned(),
            )
        })
        .collect();
    entries.sort();
    entries
}

// TC-MOD-20
#[test]
fn a_player_s_face_model_compiles_into_its_face_cpk_with_the_generated_face_xml() {
    let sandbox = Sandbox::new("prefox_face");
    let player = "exports/co Midcup Card/Players/05 - A";
    let face_diff = tracer_player_file("face_diff.bin");
    sandbox.write(&format!("{player}/face_high.model"), &card_model());
    sandbox.write(&format!("{player}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    sandbox.write(&format!("{player}/face_diff.bin"), &face_diff);

    let entries = compile_pes17(
        &sandbox,
        "co Midcup Card",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let texture = "common/character1/model/character/uniform/common/714/05 - A/skin.dds";
    assert!(entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
    assert!(entries.contains_key(texture), "{:?}", entries.keys());
    assert!(
        entries.keys().all(|path| !path.starts_with("Asset/")),
        "{:?}",
        entries.keys()
    );
    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(
        names,
        ["face.xml", "face_high.mtl", "oral_face_high_win32.model"]
    );
    assert_eq!(
        face[&format!("{folder}oral_face_high_win32.model")],
        card_model()
    );
    let xml = &face[&format!("{folder}face.xml")];
    // Its first line and its entry written out as literals, so the expected bytes below
    // cannot pass by sharing a mistake with the compiler.
    let text = String::from_utf8(xml.clone()).unwrap();
    assert!(
        text.starts_with("<?xml version='1.0' encoding='UTF-8'?>\r\n<config>\r\n"),
        "{text}"
    );
    assert!(
        text.contains(
            "\r\n   <model level=\"0\" type=\"face_neck\" path=\"./oral_face_high_*.model\" material=\"./face_high.mtl\" />\r\n"
        ),
        "{text}"
    );
    assert_eq!(
        *xml,
        expected_face_xml(
            &[(
                "face_neck",
                "./oral_face_high_*.model",
                "./face_high.mtl",
                None
            )],
            &face_diff
        )
    );
    let materials =
        pes_model::format::mtl::MaterialSet::read(&face[&format!("{folder}face_high.mtl")])
            .unwrap();
    let paths: Vec<String> = pes_model::ops::paths::texture_paths(&materials)
        .into_iter()
        .map(|path| format!("{}{}", path.directory, path.file_name))
        .collect();
    assert_eq!(
        paths,
        ["model/character/uniform/common/714/05 - A/skin.dds"]
    );
}

// TC-MOD-21
#[test]
fn every_model_of_a_player_folder_is_typed_in_its_face_xml_boots_and_gloves_included() {
    let sandbox = Sandbox::new("prefox_typed");
    let player = "exports/co Midcup Typed/Players/05 - A";
    for name in [
        "face_high",
        "kit_boots",
        "hat_parts",
        "arm_gloveL",
        "cape_model_type_cape",
        "visor_ratio_2_parts",
    ] {
        sandbox.write(&format!("{player}/{name}.model"), &card_model());
    }
    sandbox.write(&format!("{player}/materials.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        "co Midcup Typed",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let face = nested_entries(&entries[&face_cpk(5)]);
    let xml = &face[&format!("{}face.xml", face_folder(5))];
    let material = "./materials.mtl".to_owned();
    let entry = |xml_type: &str, ratio: Option<&str>| {
        (
            xml_type.to_owned(),
            ratio.map(str::to_owned),
            material.clone(),
        )
    };
    assert_eq!(
        model_entries(xml),
        [
            entry("cape", None),
            entry("face_neck", None),
            entry("gloveL", None),
            entry("parts", None),
            entry("parts", None),
            entry("parts", Some("2")),
        ]
    );
    assert!(
        entries.keys().all(|path| {
            !path.starts_with("common/character0/model/character/boots/")
                && !path.starts_with("common/character0/model/character/glove/")
        }),
        "{:?}",
        entries.keys()
    );
}

// TC-MOD-39
#[test]
fn a_player_folder_with_no_model_gets_the_pre_fox_blank_face_with_the_bundled_face_diff() {
    let sandbox = Sandbox::new("prefox_blank");
    let player = "exports/co Midcup Blank/Players/07 - B";
    let own = tracer_player_file("face_diff.bin");
    assert_ne!(own, template("face_diff.bin"));
    sandbox.write(&format!("{player}/face_diff.bin"), &own);
    fs::create_dir_all(sandbox.root.join(format!("{player}/face"))).unwrap();

    let entries = compile_pes17(
        &sandbox,
        "co Midcup Blank",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info face_file_not_used [Keep] at Players/07 - B (file=face_diff.bin)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let face = nested_entries(&entries[&face_cpk(7)]);
    let folder = face_folder(7);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(names, ["dummy.mtl", "face.xml", "oral_dummy_win32.model"]);
    assert_eq!(
        face[&format!("{folder}face.xml")],
        expected_face_xml(
            &[("face_neck", "./oral_dummy_*.model", "./dummy.mtl", None)],
            &template("face_diff.bin")
        )
    );
    assert_eq!(
        face[&format!("{folder}oral_dummy_win32.model")],
        template("dummy.model")
    );
    assert_eq!(face[&format!("{folder}dummy.mtl")], template("dummy.mtl"));
}

#[test]
fn a_model_with_no_mtl_drops_its_folder_and_a_face_without_face_neck_gets_the_dummy() {
    let sandbox = Sandbox::new("prefox_undefined");
    let export = "exports/co Midcup Hats";
    sandbox.write(&format!("{export}/Players/05 - A/hat.model"), &card_model());
    sandbox.write(
        &format!("{export}/Players/06 - B/hat_parts.model"),
        &card_model(),
    );
    sandbox.write(
        &format!("{export}/Players/06 - B/hat_parts.mtl"),
        &pre_fox_fixture("cardhead_materials.mtl"),
    );
    let undefined =
        "Error model_material_undefined [DropFolder] at Players/05 - A (file=hat.model)";

    let check = sandbox.run(&pes17(&sandbox), &["check"]);
    let lines = check.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Hats"),
        [
            undefined,
            "Info export_identified [Keep] (team=/co/, id=714)"
        ],
        "{lines:#?}"
    );
    assert_eq!(check.exit_code(), 1);
    let compile = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);
    let lines = compile.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Hats"),
        [
            undefined,
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info xml_face_neck_added [Keep] at Players/06 - B ()",
        ],
        "{lines:#?}"
    );
    // The Error makes the run's exit code 1; slot 06 is compiled all the same.
    assert_eq!(compile.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));

    assert!(!entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
    let face = nested_entries(&entries[&face_cpk(6)]);
    let folder = face_folder(6);
    assert_eq!(
        face[&format!("{folder}face.xml")],
        expected_face_xml(
            &[
                ("parts", "./oral_hat_parts_*.model", "./hat_parts.mtl", None),
                ("face_neck", "./oral_dummy_*.model", "./dummy.mtl", None),
            ],
            &template("face_diff.bin")
        )
    );
    assert_eq!(
        face[&format!("{folder}oral_dummy_win32.model")],
        template("dummy.model")
    );
}

#[test]
fn a_model_with_no_mtl_beside_an_fmdl_of_its_name_is_not_undefined_on_fox() {
    let sandbox = Sandbox::new("prefox_undefined_fox");
    let player = "exports/co Midcup Boots/Players/05 - A";
    sandbox.write(&format!("{player}/boots.fmdl"), &clean_model());
    sandbox.write(&format!("{player}/boots.model"), &card_model());

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    // The FMDL is the target's source (TC-MOD-26), so the `.model`'s missing `.mtl` is no
    // Error, and the folder is kept.
    let lines = check.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Boots"),
        ["Info export_identified [Keep] (team=/co/, id=714)"],
        "{lines:#?}"
    );
    assert_eq!(check.exit_code(), 0, "{lines:#?}");
}
