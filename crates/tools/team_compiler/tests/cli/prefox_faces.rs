//! `compile` for PES 2017: a player folder's own `.model` files, with their `.mtl` files and
//! textures, built into the player's face CPK nested in the output CPK, with the generated
//! `face.xml` typing every model, and the blank face of a player folder with no model; the
//! linked shared folders; and the export's `Common/` folder, its files written once into the
//! team's Common output and named from the `face.xml` and the `.mtl` files through `.common`
//! links.

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
pub(crate) fn face_cpk(slot: u8) -> String {
    format!("common/character0/model/character/face/real/714{slot:02}.cpk")
}

/// The folder every entry of slot `slot`'s face CPK sits in.
pub(crate) fn face_folder(slot: u8) -> String {
    format!("common/character0/model/character/face/real/714{slot:02}/")
}

/// The bytes of `pes_model`'s fixture `name`: real pre-Fox models and material sets.
pub(crate) fn pre_fox_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../libs/pes_model/tests/fixtures")
            .join(name),
    )
    .unwrap()
}

/// The card-head template's face model, a clean pre-Fox model with one material, `card`.
pub(crate) fn card_model() -> Vec<u8> {
    pre_fox_fixture("cardhead_face_high.model")
}

/// The card-head template's material set, its one texture path `./texture.dds` renamed to
/// `./skin.dds`, the texture the tests write beside it.
pub(crate) fn card_materials() -> Vec<u8> {
    materials_naming("skin")
}

/// The card-head template's material set, its one texture path `./texture.dds` renamed to
/// `./<stem>.dds`.
pub(crate) fn materials_naming(stem: &str) -> Vec<u8> {
    let text = String::from_utf8(pre_fox_fixture("cardhead_materials.mtl")).unwrap();
    assert!(text.contains("./texture.dds"), "{text}");
    text.replace("./texture.dds", &format!("./{stem}.dds"))
        .into_bytes()
}

/// The texture paths of the `.mtl` `bytes`, each its directory then its file name.
pub(crate) fn sampler_paths(bytes: &[u8]) -> Vec<String> {
    let materials = pes_model::format::mtl::MaterialSet::read(bytes).unwrap();
    pes_model::ops::paths::texture_paths(&materials)
        .into_iter()
        .map(|path| format!("{}{}", path.directory, path.file_name))
        .collect()
}

/// The entries of `entries` under the folder `folder`, by their names in it.
pub(crate) fn entries_under<'a>(
    entries: &'a BTreeMap<String, Vec<u8>>,
    folder: &str,
) -> BTreeMap<&'a str, &'a Vec<u8>> {
    entries
        .iter()
        .filter_map(|(path, bytes)| Some((path.strip_prefix(folder)?, bytes)))
        .collect()
}

/// The outer CPK folder of team 714's first shared boots output.
pub(crate) const BOOTS_K0644: &str = "common/character0/model/character/boots/k0644/";

/// Writes slot 05 of the export `export` holding `face_high.model`, its `.mtl` and `skin.dds`.
pub(crate) fn write_slot_05_face(sandbox: &Sandbox, export: &str) {
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/face_high.model"), &card_model());
    sandbox.write(&format!("{player}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
}

/// The findings of an export compiled with nothing to report.
pub(crate) const CLEAN: [&str; 2] = [
    "Info export_identified [Keep] (team=/co/, id=714)",
    "Info team_colors_missing [Keep] ()",
];

/// A small DDS, 12x12 BC3 with one level (`tests/fixtures/textures/single_level.dds`).
pub(crate) fn small_dds() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/textures")
            .join("single_level.dds"),
    )
    .unwrap()
}

/// The PES 17 settings of `sandbox`, whose game folder does not exist.
pub(crate) fn pes17(sandbox: &Sandbox) -> String {
    pes_settings(sandbox, 17)
}

/// The PES 16 settings of `sandbox`, whose game folder does not exist.
pub(crate) fn pes16(sandbox: &Sandbox) -> String {
    pes_settings(sandbox, 16)
}

/// The PES 15 settings of `sandbox`, whose game folder does not exist.
pub(crate) fn pes15(sandbox: &Sandbox) -> String {
    pes_settings(sandbox, 15)
}

/// Runs `compile --no-deploy` for PES 17, asserts the export `name`'s findings are `findings`
/// and the run succeeded, and returns the output CPK's entries.
pub(crate) fn compile_pes17(
    sandbox: &Sandbox,
    name: &str,
    findings: &[&str],
) -> BTreeMap<String, Vec<u8>> {
    compile_for(sandbox, 17, name, findings)
}

/// `compile_pes17` for PES `version`.
fn compile_for(
    sandbox: &Sandbox,
    version: u8,
    name: &str,
    findings: &[&str],
) -> BTreeMap<String, Vec<u8>> {
    let run = sandbox.run(&pes_settings(sandbox, version), &["compile", "--no-deploy"]);
    let lines = run.messages();
    assert_eq!(findings_of(&lines, name), findings, "{lines:#?}");
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"))
}

/// Every entry of the nested CPK `bytes`, by its path, with its bytes.
pub(crate) fn nested_entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
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

/// The (type, path, material) of every `<model>` of the `face.xml` `bytes`, in file order.
pub(crate) fn ordered_entries(bytes: &[u8]) -> Vec<(String, String, String)> {
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
    // The texture the set names, so its folder is compiled.
    sandbox.write(
        &format!("{export}/Players/06 - B/texture.dds"),
        &small_dds(),
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

// TC-MOD-22
#[test]
fn a_linked_boots_folder_is_written_once_as_boots_model_and_boots_mtl_under_its_shared_id() {
    let sandbox = Sandbox::new("prefox_shared_boots");
    let export = "co Midcup Crocs";
    write_slot_05_face(&sandbox, export);
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/kit_boots.model"), &card_model());
    sandbox.write(&format!("{player}/kit_boots.mtl"), &card_materials());
    sandbox.write(&format!("{player}/Crocs.boots"), b"");
    let crocs = format!("exports/{export}/Boots/Crocs");
    sandbox.write(&format!("{crocs}/boots.model"), &card_model());
    sandbox.write(&format!("{crocs}/boots.mtl"), &materials_naming("crocs"));
    sandbox.write(&format!("{crocs}/crocs.dds"), &small_dds());

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let boots = entries_under(&entries, "common/character0/model/character/boots/");
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(
        names,
        ["k0644/boots.model", "k0644/boots.mtl", "k0644/crocs.dds"]
    );
    assert_eq!(*boots["k0644/boots.model"], card_model());
    assert_eq!(sampler_paths(boots["k0644/boots.mtl"]), ["./crocs.dds"]);
    // Slot 05's own boots model is a part of his face, and the shared boots are not.
    let face = nested_entries(&entries[&face_cpk(5)]);
    let xml = String::from_utf8(face[&format!("{}face.xml", face_folder(5))].clone()).unwrap();
    assert!(
        xml.contains(
            "<model level=\"0\" type=\"parts\" path=\"./oral_kit_boots_*.model\" material=\"./kit_boots.mtl\" />"
        ),
        "{xml}"
    );
    assert!(!xml.contains("./oral_boots_"), "{xml}");
    assert!(!xml.contains("\"./boots"), "{xml}");
}

// TC-MOD-57
#[test]
fn a_shared_boots_folder_with_no_model_takes_no_id_and_warns() {
    let sandbox = Sandbox::new("prefox_shared_boots_no_model");
    let export = "co Midcup Studs";
    write_slot_05_face(&sandbox, export);
    sandbox.write(&format!("exports/{export}/Players/05 - A/Studs.boots"), b"");
    sandbox.write(&format!("exports/{export}/Players/06 - B/Zebra.boots"), b"");
    sandbox.write(
        &format!("exports/{export}/Boots/Studs/studs.dds"),
        &small_dds(),
    );
    let zebra = format!("exports/{export}/Boots/Zebra");
    sandbox.write(&format!("{zebra}/boots.model"), &card_model());
    sandbox.write(&format!("{zebra}/boots.mtl"), &materials_naming("zebra"));
    sandbox.write(&format!("{zebra}/zebra.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            CLEAN[0],
            "Warning shared_folder_no_model [Keep] at Boots/Studs (player=Players/05 - A)",
            CLEAN[1],
        ],
    );

    // Zebra takes team 714's first shared id, 644 (101 + 13 * 40 + 23), and Studs none: no
    // other boots folder is written.
    let boots = entries_under(&entries, "common/character0/model/character/boots/");
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(
        names,
        ["k0644/boots.model", "k0644/boots.mtl", "k0644/zebra.dds"]
    );
    // Slot 05 wears the game's boots: his face names none, and Studs's texture goes nowhere.
    let face = nested_entries(&entries[&face_cpk(5)]);
    let xml = String::from_utf8(face[&format!("{}face.xml", face_folder(5))].clone()).unwrap();
    assert!(!xml.contains("boots"), "{xml}");
    assert!(
        entries.keys().all(|path| !path.contains("studs")),
        "{:?}",
        entries.keys()
    );
    assert!(
        face.keys().all(|path| !path.contains("studs")),
        "{:?}",
        face.keys()
    );
}

#[test]
fn a_shared_boots_folder_s_model_of_another_name_is_written_as_boots_model_with_its_own_mtl() {
    let sandbox = Sandbox::new("prefox_shared_boots_name");
    let export = "co Midcup Mud";
    write_slot_05_face(&sandbox, export);
    sandbox.write(&format!("exports/{export}/Players/05 - A/Mud.boots"), b"");
    let mud = format!("exports/{export}/Boots/Mud");
    sandbox.write(&format!("{mud}/kit_boots.model"), &card_model());
    sandbox.write(&format!("{mud}/kit_boots.mtl"), &materials_naming("mud"));
    // A second material set, which the search does not pick for `kit_boots.model`: with no
    // model paired with it, its missing texture is only an Info.
    sandbox.write(&format!("{mud}/materials.mtl"), &materials_naming("other"));
    sandbox.write(&format!("{mud}/mud.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Info mtl_texture_unused_missing [Keep] at Boots/Mud (file=materials.mtl, texture=./other.dds, materials=card)",
            CLEAN[0],
            CLEAN[1],
        ],
    );

    let boots = entries_under(&entries, BOOTS_K0644);
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(names, ["boots.model", "boots.mtl", "mud.dds"]);
    assert_eq!(*boots["boots.model"], card_model());
    assert_eq!(sampler_paths(boots["boots.mtl"]), ["./mud.dds"]);
}

// TC-MOD-38
#[test]
fn a_linked_gloves_folder_is_written_with_a_generated_glove_xml_under_its_shared_id() {
    let sandbox = Sandbox::new("prefox_shared_gloves");
    let export = "co Midcup Keeper";
    write_slot_05_face(&sandbox, export);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/Keeper.gloves"),
        b"",
    );
    let keeper = format!("exports/{export}/Gloves/Keeper");
    for hand in ["glove_l", "glove_r"] {
        sandbox.write(&format!("{keeper}/{hand}.model"), &card_model());
        sandbox.write(&format!("{keeper}/{hand}.mtl"), &card_materials());
    }
    // The texture both sets name: a shared folder's `.mtl` is checked against its own files.
    sandbox.write(&format!("{keeper}/skin.dds"), &small_dds());

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let gloves = entries_under(&entries, "common/character0/model/character/glove/");
    let names: Vec<&str> = gloves.keys().copied().collect();
    assert_eq!(
        names,
        [
            "g0644/glove.xml",
            "g0644/glove_l.model",
            "g0644/glove_l.mtl",
            "g0644/glove_r.model",
            "g0644/glove_r.mtl",
            "g0644/skin.dds",
        ]
    );
    assert_eq!(*gloves["g0644/glove_l.model"], card_model());
    assert_eq!(
        String::from_utf8(gloves["g0644/glove.xml"].clone()).unwrap(),
        "<?xml version='1.0' encoding='UTF-8'?>\r\n\
         <config>\r\n   \
         <model level=\"0\" type=\"gloveL\" path=\"./glove_l.model\" material=\"./glove_l.mtl\" />\r\n   \
         <model level=\"0\" type=\"gloveR\" path=\"./glove_r.model\" material=\"./glove_r.mtl\" />\r\n\
         </config>"
    );
}

/// Writes the export `export`: `Faces/Longhair/` holding `hair_high.model`, `hair_high.mtl`
/// naming `./hair.dds` and `hair.dds`, linked by slot 05, which holds `face_high.model`, its
/// `.mtl` and `skin.dds`.
fn write_longhair(sandbox: &Sandbox, export: &str) {
    write_slot_05_face(sandbox, export);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/Longhair.face"),
        b"",
    );
    let longhair = format!("exports/{export}/Faces/Longhair");
    sandbox.write(&format!("{longhair}/hair_high.model"), &card_model());
    sandbox.write(
        &format!("{longhair}/hair_high.mtl"),
        &materials_naming("hair"),
    );
    sandbox.write(&format!("{longhair}/hair.dds"), &small_dds());
}

/// The findings of a `write_longhair` export.
const LONGHAIR_FINDINGS: [&str; 3] = [
    "Info export_identified [Keep] (team=/co/, id=714)",
    "Info team_colors_missing [Keep] ()",
    "Info link_combined [Keep] at Players/05 - A (link=Longhair.face)",
];

// TC-MOD-40
#[test]
fn a_linked_face_folder_is_copied_into_the_player_s_face_cpk() {
    let sandbox = Sandbox::new("prefox_shared_face");
    let export = "co Midcup Longhair";
    write_longhair(&sandbox, export);

    let entries = compile_pes17(&sandbox, export, &LONGHAIR_FINDINGS);

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "face.xml",
            "face_high.mtl",
            "hair_high.mtl",
            "oral_face_high_win32.model",
            "oral_hair_high_win32.model",
        ]
    );
    assert_eq!(
        face[&format!("{folder}face.xml")],
        expected_face_xml(
            // In the order of the models' export paths: `Faces/` before `Players/`.
            &[
                ("parts", "./oral_hair_high_*.model", "./hair_high.mtl", None),
                (
                    "face_neck",
                    "./oral_face_high_*.model",
                    "./face_high.mtl",
                    None
                ),
            ],
            &template("face_diff.bin")
        )
    );
    // The shared face's textures are the player's, in his common folder.
    assert_eq!(
        sampler_paths(&face[&format!("{folder}hair_high.mtl")]),
        ["model/character/uniform/common/714/05 - A/hair.dds"]
    );
    assert!(
        entries
            .contains_key("common/character1/model/character/uniform/common/714/05 - A/hair.dds"),
        "{:?}",
        entries.keys()
    );
    assert!(
        entries.keys().all(|path| !path.contains("Longhair")),
        "{:?}",
        entries.keys()
    );
}

#[test]
fn a_player_s_own_file_replaces_the_linked_face_folder_s_file_of_its_packed_name() {
    let sandbox = Sandbox::new("prefox_shared_face_local");
    let export = "co Midcup Longhair";
    write_longhair(&sandbox, export);
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/hair_high.mtl"),
        &materials_naming("skin"),
    );

    let entries = compile_pes17(&sandbox, export, &LONGHAIR_FINDINGS);

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    assert_eq!(
        sampler_paths(&face[&format!("{folder}hair_high.mtl")]),
        ["model/character/uniform/common/714/05 - A/skin.dds"]
    );
    let xml = &face[&format!("{folder}face.xml")];
    assert_eq!(
        model_entries(xml).len(),
        2,
        "{}",
        String::from_utf8_lossy(xml)
    );
}

/// Writes the export `export`: `Faces/Round/hat.fmdl` holding `model`, linked by slot 05,
/// which holds nothing else.
pub(crate) fn write_round_hat(sandbox: &Sandbox, export: &str, model: &[u8]) {
    sandbox.write(&format!("exports/{export}/Players/05 - A/Round.face"), b"");
    sandbox.write(&format!("exports/{export}/Faces/Round/hat.fmdl"), model);
}

#[test]
fn a_linked_face_folder_s_fmdl_is_converted_into_the_player_s_face_cpk() {
    let sandbox = Sandbox::new("prefox_shared_face_fmdl");
    let export = "co Midcup Round";
    // The tracer's hair, its skeleton (the conversion's bind pose) and the texture it names.
    write_round_hat(&sandbox, export, &tracer_player_file("fcl_hair.fmdl"));
    let round = format!("exports/{export}/Faces/Round");
    sandbox.write(
        &format!("{round}/hat.skl"),
        &tracer_player_file("fcl_hair.skl"),
    );
    sandbox.write(
        &format!("{round}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    let player = "at Players/05 - A";
    // The conversion's losses are the face task's, on the player: TC-MOD-27's for the hair.
    assert_eq!(
        findings_of(&lines, export),
        [
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=hat.fmdl, count=1662)"
                .to_owned(),
            "Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
            "Info team_colors_missing [Keep] ()".to_owned(),
            format!("Info link_combined [Keep] {player} (link=Round.face)"),
            format!(
                "Info native_field_dropped [Keep] {player} (model=Faces/Round/hat.fmdl, field=bone_matrices)"
            ),
            format!(
                "Info native_field_dropped [Keep] {player} (model=Faces/Round/hat.fmdl, bone=8, field=skl_parent)"
            ),
            format!(
                "Info native_field_dropped [Keep] {player} (model=Faces/Round/hat.fmdl, bone=25, field=skl_parent)"
            ),
            format!(
                "Warning mesh_flags_dropped [Keep] {player} (model=Faces/Round/hat.fmdl, material=1, field=no_shadow_cast)"
            ),
            format!("Info xml_face_neck_added [Keep] {player} ()"),
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
            "hat.mtl",
            "oral_dummy_win32.model",
            "oral_hat_win32.model",
        ]
    );
    // Converted: `pes_model` reads both, the hair's anti-blur mesh folded back.
    let hat = &face[&format!("{folder}oral_hat_win32.model")];
    let model = pes_model::format::PreFoxModel::read(hat).unwrap();
    assert_eq!(
        pes_model::model::Model::from_file(&model)
            .unwrap()
            .meshes
            .len(),
        2
    );
    let materials = &face[&format!("{folder}hat.mtl")];
    assert!(
        sampler_paths(materials)
            .contains(&"model/character/uniform/common/714/05 - A/shirt.dds".to_owned()),
        "{:?}",
        sampler_paths(materials)
    );
    let entry = |xml_type: &str, stem: &str| {
        (
            xml_type.to_owned(),
            format!("./oral_{stem}_*.model"),
            format!("./{stem}.mtl"),
        )
    };
    assert_eq!(
        ordered_entries(&face[&format!("{folder}face.xml")]),
        [entry("parts", "hat"), entry("face_neck", "dummy")]
    );
    assert!(
        entries.keys().all(|path| !path.contains("Round")
            && [".fmdl", ".skl"]
                .iter()
                .all(|extension| !path.ends_with(extension))),
        "{:?}",
        entries.keys()
    );
}

#[test]
fn a_linked_boots_folder_s_fmdl_is_converted_into_boots_model_and_boots_mtl_on_pes_17() {
    let sandbox = Sandbox::new("prefox_shared_boots_fmdl");
    let export = "co Midcup Mud";
    write_slot_05_face(&sandbox, export);
    sandbox.write(&format!("exports/{export}/Players/05 - A/Mud.boots"), b"");
    // The tracer's hair as the boots, its skeleton (the conversion's bind pose) and the
    // texture it names: the tracer's boots have no skeleton of their own.
    let mud = format!("exports/{export}/Boots/Mud");
    sandbox.write(
        &format!("{mud}/boots.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{mud}/boots.skl"),
        &tracer_player_file("fcl_hair.skl"),
    );
    sandbox.write(
        &format!("{mud}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    // The conversion's losses are the shared folder's task's, on the folder; the `skl_parent`
    // drops come from the skeleton, which the conversion read.
    let folder = "at Boots/Mud";
    assert_eq!(
        findings_of(&lines, export),
        [
            format!(
                "Info fmdl_weights_not_normalized [Keep] {folder} (file=boots.fmdl, count=1662)"
            ),
            CLEAN[0].to_owned(),
            CLEAN[1].to_owned(),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=boots.fmdl, field=bone_matrices)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=boots.fmdl, bone=8, field=skl_parent)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=boots.fmdl, bone=25, field=skl_parent)"
            ),
            format!(
                "Warning mesh_flags_dropped [Keep] {folder} (model=boots.fmdl, material=1, field=no_shadow_cast)"
            ),
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let boots = entries_under(&entries, BOOTS_K0644);
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(names, ["boots.model", "boots.mtl", "shirt.dds"]);
    // Converted: `pes_model` reads it, the hair's anti-blur mesh folded back.
    let model = pes_model::format::PreFoxModel::read(boots["boots.model"]).unwrap();
    assert_eq!(
        pes_model::model::Model::from_file(&model)
            .unwrap()
            .meshes
            .len(),
        2
    );
    // The shared folder's textures sit beside its models.
    assert!(
        sampler_paths(boots["boots.mtl"]).contains(&"./shirt.dds".to_owned()),
        "{:?}",
        sampler_paths(boots["boots.mtl"])
    );
    assert!(
        entries.keys().all(|path| [".fmdl", ".skl"]
            .iter()
            .all(|extension| !path.ends_with(extension))),
        "{:?}",
        entries.keys()
    );
}

#[test]
fn a_linked_gloves_folder_s_fmdls_are_converted_and_listed_in_its_glove_xml_on_pes_17() {
    let sandbox = Sandbox::new("prefox_shared_gloves_fmdl");
    let export = "co Midcup Keeper";
    write_slot_05_face(&sandbox, export);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/Keeper.gloves"),
        b"",
    );
    let keeper = format!("exports/{export}/Gloves/Keeper");
    for hand in ["glove_l", "glove_r"] {
        sandbox.write(
            &format!("{keeper}/{hand}.fmdl"),
            &tracer_player_file(&format!("{hand}.fmdl")),
        );
    }
    sandbox.write(
        &format!("{keeper}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    let folder = "at Gloves/Keeper";
    assert_eq!(
        findings_of(&lines, export),
        [
            format!(
                "Info fmdl_weights_not_normalized [Keep] {folder} (file=glove_l.fmdl, count=2)"
            ),
            CLEAN[0].to_owned(),
            CLEAN[1].to_owned(),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=glove_l.fmdl, field=bone_matrices)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=glove_r.fmdl, field=bone_matrices)"
            ),
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let gloves = entries_under(&entries, "common/character0/model/character/glove/g0644/");
    let names: Vec<&str> = gloves.keys().copied().collect();
    assert_eq!(
        names,
        [
            "glove.xml",
            "glove_l.model",
            "glove_l.mtl",
            "glove_r.model",
            "glove_r.mtl",
            "shirt.dds",
        ]
    );
    for hand in ["glove_l", "glove_r"] {
        let model = pes_model::format::PreFoxModel::read(gloves[format!("{hand}.model").as_str()]);
        assert!(model.is_ok(), "{hand}.model: {model:?}");
    }
    assert_eq!(
        String::from_utf8(gloves["glove.xml"].clone()).unwrap(),
        "<?xml version='1.0' encoding='UTF-8'?>\r\n\
         <config>\r\n   \
         <model level=\"0\" type=\"gloveL\" path=\"./glove_l.model\" material=\"./glove_l.mtl\" />\r\n   \
         <model level=\"0\" type=\"gloveR\" path=\"./glove_r.model\" material=\"./glove_r.mtl\" />\r\n\
         </config>"
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

/// The outer CPK folder of team 714's Common output.
const COMMON_714: &str = "common/character1/model/character/uniform/common/714/";

/// The entries directly in team 714's Common output, by their names in it: the players'
/// common subfolders below it left out.
pub(crate) fn common_output(entries: &BTreeMap<String, Vec<u8>>) -> BTreeMap<&str, &Vec<u8>> {
    entries_under(entries, COMMON_714)
        .into_iter()
        .filter(|(name, _)| !name.contains('/'))
        .collect()
}

/// The entries under slot 05's common subfolder of team 714's Common output.
fn slot_05_common(entries: &BTreeMap<String, Vec<u8>>) -> Vec<&str> {
    entries_under(entries, &format!("{COMMON_714}05 - A/"))
        .into_keys()
        .collect()
}

/// Writes the export `export`: slot 05 holding `legs.model.common` and no model of its own,
/// and `Common/` holding `legs.model` (the card model), `legs.mtl` naming `./cloth.dds` and
/// `cloth.dds`.
fn write_common_legs(sandbox: &Sandbox, export: &str) {
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/legs.model.common"),
        b"",
    );
    let common = format!("exports/{export}/Common");
    sandbox.write(&format!("{common}/legs.model"), &card_model());
    sandbox.write(&format!("{common}/legs.mtl"), &materials_naming("cloth"));
    sandbox.write(&format!("{common}/cloth.dds"), &small_dds());
}

/// The findings of a `write_common_legs` export: slot 05's face has no `face_neck` model.
const COMMON_LEGS_FINDINGS: [&str; 3] = [
    "Info export_identified [Keep] (team=/co/, id=714)",
    "Info team_colors_missing [Keep] ()",
    "Info xml_face_neck_added [Keep] at Players/05 - A ()",
];

/// The `face.xml` of a `write_common_legs` export's slot 05: the Common `legs` model with its
/// Common `.mtl`, then the dummy.
fn common_legs_face_xml() -> Vec<u8> {
    expected_face_xml(
        &[
            (
                "parts",
                "model/character/uniform/common/714/oral_legs_*.model",
                "model/character/uniform/common/714/legs.mtl",
                None,
            ),
            ("face_neck", "./oral_dummy_*.model", "./dummy.mtl", None),
        ],
        &template("face_diff.bin"),
    )
}

// TC-MOD-24
#[test]
fn a_common_model_link_names_the_common_output_which_holds_the_model_and_its_mtl() {
    let sandbox = Sandbox::new("prefox_common_legs");
    let export = "co Midcup Legs";
    write_common_legs(&sandbox, export);

    let entries = compile_pes17(&sandbox, export, &COMMON_LEGS_FINDINGS);

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(names, ["dummy.mtl", "face.xml", "oral_dummy_win32.model"]);
    assert_eq!(face[&format!("{folder}face.xml")], common_legs_face_xml());
    let common = common_output(&entries);
    let names: Vec<&str> = common.keys().copied().collect();
    assert_eq!(names, ["cloth.dds", "legs.mtl", "oral_legs_win32.model"]);
    assert_eq!(*common["oral_legs_win32.model"], card_model());
    assert_eq!(
        sampler_paths(common["legs.mtl"]),
        ["model/character/uniform/common/714/cloth.dds"]
    );
    assert_eq!(slot_05_common(&entries), Vec::<&str>::new());
}

// TC-MOD-24
#[test]
fn a_common_model_link_takes_a_local_mtl_of_its_name_over_the_common_one() {
    let sandbox = Sandbox::new("prefox_common_legs_local");
    let export = "co Midcup Legs";
    write_common_legs(&sandbox, export);
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/legs.mtl"), &materials_naming("skin"));
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());

    let entries = compile_pes17(&sandbox, export, &COMMON_LEGS_FINDINGS);

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    assert_eq!(
        face[&format!("{folder}face.xml")],
        expected_face_xml(
            &[
                (
                    "parts",
                    "model/character/uniform/common/714/oral_legs_*.model",
                    "./legs.mtl",
                    None
                ),
                ("face_neck", "./oral_dummy_*.model", "./dummy.mtl", None),
            ],
            &template("face_diff.bin")
        )
    );
    assert_eq!(
        sampler_paths(&face[&format!("{folder}legs.mtl")]),
        ["model/character/uniform/common/714/05 - A/skin.dds"]
    );
    let common = common_output(&entries);
    let names: Vec<&str> = common.keys().copied().collect();
    assert_eq!(names, ["cloth.dds", "legs.mtl", "oral_legs_win32.model"]);
    assert_eq!(slot_05_common(&entries), ["skin.dds"]);
}

// TC-MOD-37
#[test]
fn a_material_link_names_the_common_mtl_which_the_common_output_holds_once() {
    let sandbox = Sandbox::new("prefox_common_mtl");
    let export = "co Midcup Skin";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/face_high.model"), &card_model());
    sandbox.write(&format!("{player}/face_high.mtl.common"), b"");
    let common = format!("exports/{export}/Common");
    sandbox.write(&format!("{common}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{common}/skin.dds"), &small_dds());

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let face = nested_entries(&entries[&face_cpk(5)]);
    let folder = face_folder(5);
    let names: Vec<&str> = face
        .keys()
        .map(|path| path.strip_prefix(folder.as_str()).unwrap())
        .collect();
    assert_eq!(names, ["face.xml", "oral_face_high_win32.model"]);
    assert_eq!(
        face[&format!("{folder}face.xml")],
        expected_face_xml(
            &[(
                "face_neck",
                "./oral_face_high_*.model",
                "model/character/uniform/common/714/face_high.mtl",
                None
            )],
            &template("face_diff.bin")
        )
    );
    let common = common_output(&entries);
    let names: Vec<&str> = common.keys().copied().collect();
    assert_eq!(names, ["face_high.mtl", "skin.dds"]);
    assert_eq!(
        sampler_paths(common["face_high.mtl"]),
        ["model/character/uniform/common/714/skin.dds"]
    );
    assert_eq!(slot_05_common(&entries), Vec::<&str>::new());
}

#[test]
fn a_texture_link_points_the_player_s_mtl_at_the_common_texture() {
    let sandbox = Sandbox::new("prefox_common_texture");
    let export = "co Midcup Hair";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/face_high.model"), &card_model());
    sandbox.write(
        &format!("{player}/face_high.mtl"),
        &materials_naming("hair"),
    );
    sandbox.write(&format!("{player}/hair.dds.common"), b"");
    sandbox.write(&format!("exports/{export}/Common/hair.dds"), &small_dds());

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let face = nested_entries(&entries[&face_cpk(5)]);
    assert_eq!(
        sampler_paths(&face[&format!("{}face_high.mtl", face_folder(5))]),
        ["model/character/uniform/common/714/hair.dds"]
    );
    let common = common_output(&entries);
    let names: Vec<&str> = common.keys().copied().collect();
    assert_eq!(names, ["hair.dds"]);
    assert_eq!(slot_05_common(&entries), Vec::<&str>::new());
}

#[test]
fn two_spellings_of_one_model_link_give_one_entry() {
    let sandbox = Sandbox::new("prefox_common_legs_twice");
    let export = "co Midcup Legs";
    write_common_legs(&sandbox, export);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/legs.model.common.txt"),
        b"",
    );

    let entries = compile_pes17(&sandbox, export, &COMMON_LEGS_FINDINGS);

    let face = nested_entries(&entries[&face_cpk(5)]);
    let xml = &face[&format!("{}face.xml", face_folder(5))];
    assert_eq!(
        String::from_utf8_lossy(xml),
        String::from_utf8_lossy(&common_legs_face_xml())
    );
}

#[test]
fn a_model_link_whose_only_mtl_is_a_broken_common_one_drops_its_folder() {
    let sandbox = Sandbox::new("prefox_common_mtl_broken");
    let export = "co Midcup Legs";
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/legs.model.common"),
        b"",
    );
    let common = format!("exports/{export}/Common");
    sandbox.write(&format!("{common}/legs.model"), &card_model());
    sandbox.write(&format!("{common}/legs.mtl"), b"not a material set");
    let error = pes_model::format::mtl::MaterialSet::read(b"not a material set")
        .unwrap_err()
        .to_string();
    let broken =
        format!("Error mtl_broken [DropFile] at Common/legs.mtl (file=legs.mtl, error={error})");
    let undefined =
        "Error model_material_undefined [DropFolder] at Players/05 - A (file=legs.model.common)";

    let check = sandbox.run(&pes17(&sandbox), &["check"]);
    let lines = check.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            undefined,
            broken.as_str(),
            "Info export_identified [Keep] (team=/co/, id=714)"
        ],
        "{lines:#?}"
    );
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);
    let lines = compile.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            undefined,
            broken.as_str(),
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
        "{lines:#?}"
    );
    assert_eq!(compile.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(!entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
}

// TC-CMN-12
#[test]
fn a_common_gltf_is_dropped_and_the_export_compiles() {
    let sandbox = Sandbox::new("prefox_common_glb");
    let export = "co Midcup Card";
    write_slot_05_face(&sandbox, export);
    sandbox.write(&format!("exports/{export}/Common/x.glb"), b"glTF");

    let run = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Error model_gltf_unsupported [DropFile] at Common/x.glb (file=Common/x.glb)",
        ],
        "{lines:#?}"
    );
    // The Error drops the file alone, and still sets the run's exit code.
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let face = nested_entries(&entries[&face_cpk(5)]);
    assert!(
        face.contains_key(&format!("{}oral_face_high_win32.model", face_folder(5))),
        "{:#?}",
        face.keys()
    );
}

/// Writes the export `export`: slot 05 with his own face (`write_slot_05_face`) and
/// `legs.fmdl.common`, and `Common/` holding `legs.fmdl` (the tracer's hair), `legs.skl` (its
/// skeleton) and `shirt.dds`, the texture the hair names.
fn write_common_fmdl_legs(sandbox: &Sandbox, export: &str) {
    write_slot_05_face(sandbox, export);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/legs.fmdl.common"),
        b"",
    );
    let common = format!("exports/{export}/Common");
    sandbox.write(
        &format!("{common}/legs.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{common}/legs.skl"),
        &tracer_player_file("fcl_hair.skl"),
    );
    sandbox.write(
        &format!("{common}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
}

#[test]
fn a_common_fmdl_is_converted_once_into_the_common_output_which_a_link_names() {
    let sandbox = Sandbox::new("prefox_common_fmdl");
    let export = "co Midcup Legs";
    write_common_fmdl_legs(&sandbox, export);

    let run = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    // The conversion's losses are the Common models task's, on `Common`; the `skl_parent` drops
    // come from the skeleton, which the conversion read.
    let common = "at Common";
    assert_eq!(
        findings_of(&lines, export),
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)"
                .to_owned(),
            CLEAN[0].to_owned(),
            CLEAN[1].to_owned(),
            format!(
                "Info native_field_dropped [Keep] {common} (model=legs.fmdl, field=bone_matrices)"
            ),
            format!(
                "Info native_field_dropped [Keep] {common} (model=legs.fmdl, bone=8, field=skl_parent)"
            ),
            format!(
                "Info native_field_dropped [Keep] {common} (model=legs.fmdl, bone=25, field=skl_parent)"
            ),
            format!(
                "Warning mesh_flags_dropped [Keep] {common} (model=legs.fmdl, material=1, field=no_shadow_cast)"
            ),
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let output = common_output(&entries);
    let names: Vec<&str> = output.keys().copied().collect();
    assert_eq!(names, ["legs.mtl", "oral_legs_win32.model", "shirt.dds"]);
    // Converted: `pes_model` reads it, the hair's anti-blur mesh folded back.
    let model = pes_model::format::PreFoxModel::read(output["oral_legs_win32.model"]).unwrap();
    assert_eq!(
        pes_model::model::Model::from_file(&model)
            .unwrap()
            .meshes
            .len(),
        2
    );
    assert!(
        sampler_paths(output["legs.mtl"])
            .contains(&"model/character/uniform/common/714/shirt.dds".to_owned()),
        "{:?}",
        sampler_paths(output["legs.mtl"])
    );
    // His face lists the Common model with its converted set, and packs neither.
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
    let entry = |xml_type: &str, path: &str, material: &str| {
        (xml_type.to_owned(), path.to_owned(), material.to_owned())
    };
    assert_eq!(
        ordered_entries(&face[&format!("{folder}face.xml")]),
        [
            entry("face_neck", "./oral_face_high_*.model", "./face_high.mtl"),
            entry(
                "parts",
                "model/character/uniform/common/714/oral_legs_*.model",
                "model/character/uniform/common/714/legs.mtl"
            ),
        ]
    );
    assert!(
        entries.keys().all(|path| [".fmdl", ".skl"]
            .iter()
            .all(|extension| !path.ends_with(extension))),
        "{:?}",
        entries.keys()
    );
}

#[test]
fn a_common_mtl_of_a_converted_fmdl_s_set_name_fails_the_common_models_task() {
    let sandbox = Sandbox::new("prefox_common_fmdl_mtl");
    let export = "co Midcup Legs";
    write_common_fmdl_legs(&sandbox, export);
    let common = format!("exports/{export}/Common");
    sandbox.write(&format!("{common}/Legs.mtl"), &materials_naming("shirt"));

    let run = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);

    // Two files of one name, folded: the task fails as a whole, its conversion's notes with
    // it, and the Common textures and slot 05's face still commit.
    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)",
            CLEAN[0],
            CLEAN[1],
            "Error folder_pack_failed [DropFolder] at Common (error=two files of Common/ are packed as Legs.mtl)",
        ],
        "{lines:#?}"
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let names: Vec<&str> = common_output(&entries).keys().copied().collect();
    assert_eq!(names, ["shirt.dds"]);
    assert!(entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
}

// TC-MOD-23
#[test]
fn a_uniform_model_is_typed_uniform_sub_on_pes_15_and_uniform_on_16_and_17() {
    let sandbox = Sandbox::new("prefox_uniform");
    let export = "co Midcup Uniform";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/body_uniform.model"), &card_model());
    sandbox.write(&format!("{player}/body_uniform.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    let face_neck_added = "Info xml_face_neck_added [Keep] at Players/05 - A ()";
    let renamed = "Info xml_uniform_pes15 [Keep] at Players/05 - A (file=body_uniform.model)";

    for (version, xml_type, findings) in [
        (
            15,
            "uniform_sub",
            [&CLEAN[..], &[renamed, face_neck_added]].concat(),
        ),
        (16, "uniform", [&CLEAN[..], &[face_neck_added]].concat()),
        (17, "uniform", [&CLEAN[..], &[face_neck_added]].concat()),
    ] {
        let entries = compile_for(&sandbox, version, export, &findings);

        let face = nested_entries(&entries[&face_cpk(5)]);
        let xml = &face[&format!("{}face.xml", face_folder(5))];
        // Every version names the packed model with the `oral_` prefix PES 16 needs.
        assert_eq!(
            String::from_utf8_lossy(xml),
            String::from_utf8_lossy(&expected_face_xml(
                &[
                    (
                        xml_type,
                        "./oral_body_uniform_*.model",
                        "./body_uniform.mtl",
                        None
                    ),
                    ("face_neck", "./oral_dummy_*.model", "./dummy.mtl", None),
                ],
                &template("face_diff.bin")
            )),
            "PES {version}"
        );
    }
}

/// Runs `check` for PES 17 and asserts the export `name`'s findings are `findings` and the
/// exit code is 1.
fn check_pes17_fails(sandbox: &Sandbox, name: &str, findings: &[&str]) {
    let check = sandbox.run(&pes17(sandbox), &["check"]);
    let lines = check.messages();
    assert_eq!(findings_of(&lines, name), findings, "{lines:#?}");
    assert_eq!(check.exit_code(), 1, "{lines:#?}");
}

// TC-MOD-25
#[test]
fn an_edit_hair_file_and_an_unsuffixed_shared_boots_model_drop_their_folders_at_check() {
    let sandbox = Sandbox::new("prefox_edithair");
    let export = "co Midcup Edithair";
    write_slot_05_face(&sandbox, export);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/face_edithair.xml"),
        b"<config />",
    );
    check_pes17_fails(
        &sandbox,
        export,
        &[
            "Error edithair_unsupported [DropFolder] at Players/05 - A (file=face_edithair.xml)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
    );

    let sandbox = Sandbox::new("prefox_boots_name");
    let export = "co Midcup Mud";
    write_slot_05_face(&sandbox, export);
    sandbox.write(&format!("exports/{export}/Players/07 - B/Mud.boots"), b"");
    sandbox.write(
        &format!("exports/{export}/Players/07 - B/face_high.model"),
        &card_model(),
    );
    sandbox.write(
        &format!("exports/{export}/Players/07 - B/face_high.mtl"),
        &card_materials(),
    );
    let mud = format!("exports/{export}/Boots/Mud");
    sandbox.write(&format!("{mud}/hat.model"), &card_model());
    sandbox.write(&format!("{mud}/hat.mtl"), &card_materials());
    check_pes17_fails(
        &sandbox,
        export,
        &[
            "Error model_name_invalid [DropFolder] at Boots/Mud (file=hat.model)",
            "Error link_target_dropped [DropFolder] at Players/07 - B (link=Mud.boots, target=Boots/Mud, finding=model_name_invalid)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
    );
}

// TC-XML-08
#[test]
fn a_model_material_its_mtl_does_not_define_drops_the_folder_at_check_and_compile() {
    let sandbox = Sandbox::new("prefox_material_undefined");
    let export = "co Midcup Skin";
    // The card head's model with its one material, `card`, renamed `skin`; its material set
    // defines only `card`.
    let file = pes_model::format::PreFoxModel::read(&card_model()).unwrap();
    let mut model = pes_model::model::Model::from_file(&file).unwrap();
    assert_eq!(model.materials, ["card"]);
    model.materials[0] = "skin".to_owned();
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/face_high.model"),
        &model.to_file().unwrap().write().unwrap(),
    );
    sandbox.write(&format!("{player}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    // A clean player beside it, so the CPK is written and slot 05's absence observable.
    let clean = format!("exports/{export}/Players/07 - B");
    sandbox.write(&format!("{clean}/face_high.model"), &card_model());
    sandbox.write(&format!("{clean}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{clean}/skin.dds"), &small_dds());
    let undefined = "Error model_material_undefined [DropFolder] at Players/05 - A (file=face_high.model, mtl=face_high.mtl, materials=skin)";

    check_pes17_fails(
        &sandbox,
        export,
        &[
            undefined,
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
    );
    let compile = sandbox.run(&pes17(&sandbox), &["compile", "--no-deploy"]);
    let lines = compile.messages();
    assert_eq!(
        findings_of(&lines, export),
        [&[undefined][..], &CLEAN[..]].concat(),
        "{lines:#?}"
    );
    assert_eq!(compile.exit_code(), 1, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(entries.contains_key(&face_cpk(7)), "{:?}", entries.keys());
    assert!(!entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
}
