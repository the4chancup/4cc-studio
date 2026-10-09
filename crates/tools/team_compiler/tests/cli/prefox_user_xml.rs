//! A player folder's own `face.xml` (`team_compiler/messages.md` "User-supplied `face.xml`"):
//! read and checked by `check` for PES 2015 to 2017, its errors dropping the folder, and
//! written back by `compile` with only the files it names packed; ignored for PES 2018 to 2021,
//! whose `compile` builds the folder as without it.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings};
use crate::models::{face_diff_fixture, template};
use crate::prefox_faces::{
    CLEAN, card_materials, card_model, common_output, compile_pes17, face_cpk, face_folder,
    materials_naming, nested_entries, ordered_entries, pes15, pes16, pes17, sampler_paths,
    small_dds,
};
use crate::{clean_model, findings_of};

/// `/co/`'s identity line.
const IDENTIFIED: &str = "Info export_identified [Keep] (team=/co/, id=714)";

/// Writes the player folder `folder` of the export `export` holding the card head's model as
/// `<stem>.model`, its `.mtl` naming `skin.dds`, that texture, and the `face.xml` `xml`.
fn write_folder(sandbox: &Sandbox, export: &str, folder: &str, stem: &str, xml: &str) {
    let player = format!("exports/{export}/Players/{folder}");
    sandbox.write(&format!("{player}/{stem}.model"), &card_model());
    sandbox.write(&format!("{player}/{stem}.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    sandbox.write(&format!("{player}/face.xml"), xml.as_bytes());
}

/// The `face.xml` listing `entries`, each a `<model>`'s attributes.
fn face_xml(entries: &[&str]) -> String {
    let models: String = entries
        .iter()
        .map(|attributes| format!("   <model level=\"0\" {attributes}/>\n"))
        .collect();
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<config>\n{models}</config>\n")
}

/// The `face_high` entry, which names its two files.
const FACE_HIGH: &str = r#"type="face_neck" path="./face_high.model" material="./face_high.mtl""#;

// TC-XML-03
#[test]
fn an_entry_without_path_and_one_naming_an_absent_model_each_drop_their_folder() {
    let sandbox = Sandbox::new("user_xml_errors");
    let export = "co Midcup Xml";
    write_folder(
        &sandbox,
        export,
        "05 - A",
        "face_high",
        &face_xml(&[r#"type="face_neck" material="./face_high.mtl""#]),
    );
    write_folder(
        &sandbox,
        export,
        "07 - B",
        "face_high",
        &face_xml(&[FACE_HIGH, r#"type="parts" path="./hat.model""#]),
    );

    let run = sandbox.run(&pes17(&sandbox), &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            "Error xml_model_path_missing [DropFolder] at Players/05 - A (entry=1, type=face_neck)",
            "Warning xml_model_unlisted [Keep] at Players/05 - A (file=face_high.model)",
            "Error xml_model_not_found [DropFolder] at Players/07 - B (attribute=path, value=./hat.model)",
            IDENTIFIED,
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-XML-04
#[test]
fn a_dif_in_the_xml_beside_a_face_diff_xml_drops_the_folder() {
    let sandbox = Sandbox::new("user_xml_dif_conflict");
    let export = "co Midcup Xml";
    let dif = face_diff_fixture("dif.xml");
    let dif_text = String::from_utf8(dif.clone()).unwrap();
    // The fixture's `<dif>` text, the same face diff given twice.
    let start = dif_text.find("<dif>").unwrap() + "<dif>".len();
    let end = dif_text.find("</dif>").unwrap();
    let base64 = &dif_text[start..end];
    let xml =
        format!("<config>\n   <model level=\"0\" {FACE_HIGH}/>\n<dif>{base64}</dif>\n</config>\n");
    write_folder(&sandbox, export, "05 - A", "face_high", &xml);
    sandbox.write(
        &format!("exports/{export}/Players/05 - A/face_diff.xml"),
        &dif,
    );

    let run = sandbox.run(&pes17(&sandbox), &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            "Error xml_dif_conflict [DropFolder] at Players/05 - A (file=face.xml)",
            IDENTIFIED,
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-XML-06
#[test]
fn on_pes_21_the_xml_is_ignored_and_the_folder_compiles_as_without_it() {
    let compiled = |name: &str, with_xml: bool| {
        let sandbox = Sandbox::new(name);
        let player = "exports/co Midcup Xml/Players/05 - A";
        sandbox.write(&format!("{player}/face_high.fmdl"), &clean_model());
        if with_xml {
            sandbox.write(
                &format!("{player}/face.xml"),
                face_xml(&[r#"type="face_neck" path="./face_high.model""#]).as_bytes(),
            );
        }
        let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);
        let lines = run.messages();
        assert_eq!(run.exit_code(), 0, "{lines:#?}");
        let findings: Vec<String> = findings_of(&lines, "co Midcup Xml")
            .into_iter()
            .map(str::to_owned)
            .collect();
        (
            findings,
            cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk")),
        )
    };

    let (with_findings, with_entries) = compiled("user_xml_fox", true);
    let (without_findings, without_entries) = compiled("user_xml_fox_without", false);

    assert_eq!(
        with_findings,
        [
            IDENTIFIED,
            "Info xml_ignored_fox [Keep] at Players/05 - A (file=face.xml)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert_eq!(
        without_findings,
        [IDENTIFIED, "Info team_colors_missing [Keep] ()"]
    );
    assert!(
        with_entries
            .keys()
            .any(|path| path.contains("face/real/71405")),
        "{:?}",
        with_entries.keys()
    );
    assert_eq!(with_entries, without_entries);
}

// TC-XML-09
#[test]
fn on_pes_16_a_model_name_without_its_prefix_drops_the_folder_and_on_pes_17_it_does_not() {
    let sandbox = Sandbox::new("user_xml_oral_prefix");
    let export = "co Midcup Xml";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/body_uniform.model"), &card_model());
    sandbox.write(
        &format!("{player}/body_uniform.mtl"),
        &materials_naming("body"),
    );
    sandbox.write(&format!("{player}/body.dds"), &small_dds());
    sandbox.write(
        &format!("{player}/face.xml"),
        face_xml(&[r#"type="uniform" path="./body_uniform.model" material="./body_uniform.mtl""#])
            .as_bytes(),
    );

    let pes_16 = sandbox.run(&pes16(&sandbox), &["check"]);
    let lines = pes_16.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            "Error xml_oral_prefix_missing [DropFolder] at Players/05 - A (path=./body_uniform.model)",
            IDENTIFIED,
        ],
        "{lines:#?}"
    );
    assert_eq!(pes_16.exit_code(), 1);

    let pes_17 = sandbox.run(&pes17(&sandbox), &["check"]);
    let lines = pes_17.messages();
    assert_eq!(findings_of(&lines, export), [IDENTIFIED], "{lines:#?}");
    assert_eq!(pes_17.exit_code(), 0);
}

/// Slot 05's face CPK among the output CPK's `entries`, each file by its name in the face.
fn slot_05_face(entries: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    let folder = face_folder(5);
    nested_entries(&entries[&face_cpk(5)])
        .into_iter()
        .map(|(path, bytes)| {
            (
                path.strip_prefix(folder.as_str()).unwrap().to_owned(),
                bytes,
            )
        })
        .collect()
}

/// The names of the files of `face`, in their order.
fn names(face: &BTreeMap<String, Vec<u8>>) -> Vec<&str> {
    face.keys().map(String::as_str).collect()
}

/// The first two lines every written `face.xml` starts with, as the game's own files spell
/// them.
const HEAD: &str = "<?xml version='1.0' encoding='UTF-8'?>\r\n<config>\r\n";

/// The `<dif>` lines and the root's end every written `face.xml` ends with, for the face diff
/// `dif`.
fn dif_tail(dif: &[u8]) -> String {
    format!("<dif>\r\n{}\r\n</dif>\r\n</config>", STANDARD.encode(dif))
}

/// The texture home of slot 05's textures in team 714's Common output, as a `.mtl` names it.
const SLOT_05_HOME: &str = "model/character/uniform/common/714/05 - A/";

/// Writes slot 05 of `export` holding `face_high.model` and `hat_parts.model` (the card head's
/// model), their `.mtl` files naming `skin.dds`, that texture, and `face.xml` when given.
fn write_face_and_hat(sandbox: &Sandbox, export: &str, xml: Option<&str>) {
    let player = format!("exports/{export}/Players/05 - A");
    for stem in ["face_high", "hat_parts"] {
        sandbox.write(&format!("{player}/{stem}.model"), &card_model());
        sandbox.write(&format!("{player}/{stem}.mtl"), &card_materials());
    }
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    if let Some(xml) = xml {
        sandbox.write(&format!("{player}/face.xml"), xml.as_bytes());
    }
}

// TC-XML-01
#[test]
fn a_member_s_xml_is_written_back_with_its_own_names_and_the_common_model_s_packed_one() {
    let sandbox = Sandbox::new("user_xml_compiled");
    let export = "co Midcup Xml";
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<config>\n\
        \t<model level=\"0\" type=\"face_neck\" path=\"./face_high.model\" material=\"./face_high.mtl\"/>\n\
        \t<model level=\"1\" type=\"cape\" glow=\"1\" path=\"./hat_parts.model\" material=\"./hat_parts.mtl\"/>\n\
        \t<model level=\"0\" type=\"parts\" path=\"model/character/uniform/common/XXX/legs.model\" material=\"model/character/uniform/common/XXX/legs.mtl\"/>\n\
        </config>\n";
    write_face_and_hat(&sandbox, export, Some(xml));
    let common = format!("exports/{export}/Common");
    sandbox.write(&format!("{common}/legs.model"), &card_model());
    sandbox.write(&format!("{common}/legs.mtl"), &materials_naming("studs"));
    sandbox.write(&format!("{common}/studs.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Warning xml_attribute_unknown [Keep] at Players/05 - A (entry=2, attribute=glow)",
            "Warning xml_type_unknown [Keep] at Players/05 - A (type=cape)",
            "Info xml_level_lod [Keep] at Players/05 - A (level=1)",
            IDENTIFIED,
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let face = slot_05_face(&entries);
    assert_eq!(
        names(&face),
        [
            "face.xml",
            "face_high.model",
            "face_high.mtl",
            "hat_parts.model",
            "hat_parts.mtl",
        ]
    );
    assert_eq!(face["face_high.model"], card_model());
    assert_eq!(face["hat_parts.model"], card_model());
    let expected = format!(
        "{HEAD}   \
         <model level=\"0\" type=\"face_neck\" path=\"./face_high.model\" material=\"./face_high.mtl\" />\r\n   \
         <model level=\"1\" type=\"cape\" glow=\"1\" path=\"./hat_parts.model\" material=\"./hat_parts.mtl\" />\r\n   \
         <model level=\"0\" type=\"parts\" path=\"model/character/uniform/common/714/oral_legs_*.model\" material=\"model/character/uniform/common/714/legs.mtl\" />\r\n\
         {}",
        dif_tail(&template("face_diff.bin"))
    );
    assert_eq!(String::from_utf8_lossy(&face["face.xml"]), expected);
    for mtl in ["face_high.mtl", "hat_parts.mtl"] {
        assert_eq!(
            sampler_paths(&face[mtl]),
            [format!("{SLOT_05_HOME}skin.dds")],
            "{mtl}"
        );
    }
    let common = common_output(&entries);
    let common_names: Vec<&str> = common.keys().copied().collect();
    assert_eq!(
        common_names,
        ["legs.mtl", "oral_legs_win32.model", "studs.dds"]
    );
}

// TC-XML-02
#[test]
fn without_its_xml_the_same_folder_s_models_are_typed_by_their_names() {
    let sandbox = Sandbox::new("user_xml_removed");
    let export = "co Midcup Xml";
    write_face_and_hat(&sandbox, export, None);

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let face = slot_05_face(&entries);
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [
            (
                "face_neck".to_owned(),
                "./oral_face_high_*.model".to_owned(),
                "./face_high.mtl".to_owned()
            ),
            (
                "parts".to_owned(),
                "./oral_hat_parts_*.model".to_owned(),
                "./hat_parts.mtl".to_owned()
            ),
        ]
    );
}

// TC-XML-10
#[test]
fn a_shared_face_s_own_face_xml_is_ignored_on_pes_17_and_the_face_lists_its_models() {
    let sandbox = Sandbox::new("user_xml_shared_face");
    let export = "co Midcup Xml";
    sandbox.write(&format!("exports/{export}/Players/05 - A/Round.face"), b"");
    let round = format!("exports/{export}/Faces/Round");
    sandbox.write(&format!("{round}/face_high.model"), &card_model());
    sandbox.write(&format!("{round}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{round}/skin.dds"), &small_dds());
    let xml = face_xml(&[FACE_HIGH]);
    sandbox.write(&format!("{round}/face.xml"), xml.as_bytes());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            IDENTIFIED,
            "Info xml_ignored_shared [Keep] at Faces/Round (file=face.xml)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
        ],
    );

    // The face is generated as without the xml: the model packed under its generated name and
    // listed by it, not the member's `./face_high.model` written back.
    let face = slot_05_face(&entries);
    assert_eq!(
        names(&face),
        ["face.xml", "face_high.mtl", "oral_face_high_win32.model"]
    );
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [(
            "face_neck".to_owned(),
            "./oral_face_high_*.model".to_owned(),
            "./face_high.mtl".to_owned()
        )]
    );
    assert!(face["face.xml"] != xml.as_bytes(), "the member's xml");
}

/// The `face.xml` naming `./hat.model` typed `parts` with `./hat.mtl`, and no `face_neck`.
fn hat_xml() -> String {
    face_xml(&[r#"type="parts" path="./hat.model" material="./hat.mtl""#])
}

/// Each (type, path, material) as `ordered_entries` gives it.
fn owned_entries(entries: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    entries
        .iter()
        .map(|(xml_type, path, material)| {
            (
                (*xml_type).to_owned(),
                (*path).to_owned(),
                (*material).to_owned(),
            )
        })
        .collect()
}

/// The finding of slot 05's face given the dummy.
const FACE_NECK_ADDED: &str = "Info xml_face_neck_added [Keep] at Players/05 - A ()";

// TC-XML-04
#[test]
fn a_member_s_xml_with_no_face_neck_entry_gets_the_dummy_after_its_entries() {
    let sandbox = Sandbox::new("user_xml_face_neck");
    let export = "co Midcup Xml";
    write_folder(&sandbox, export, "05 - A", "hat", &hat_xml());

    let entries = compile_pes17(&sandbox, export, &[&CLEAN[..], &[FACE_NECK_ADDED]].concat());

    let face = slot_05_face(&entries);
    assert_eq!(
        names(&face),
        [
            "dummy.mtl",
            "face.xml",
            "hat.model",
            "hat.mtl",
            "oral_dummy_win32.model"
        ]
    );
    assert_eq!(face["oral_dummy_win32.model"], template("dummy.model"));
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        owned_entries(&[
            ("parts", "./hat.model", "./hat.mtl"),
            ("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ])
    );
}

// TC-XML-05
#[test]
fn a_model_the_member_s_xml_does_not_list_is_reported_and_not_packed() {
    let sandbox = Sandbox::new("user_xml_unlisted");
    let export = "co Midcup Xml";
    write_folder(&sandbox, export, "05 - A", "hat", &hat_xml());
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/hat2.model"), &card_model());
    sandbox.write(&format!("{player}/hat2.mtl"), &card_materials());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Warning xml_model_unlisted [Keep] at Players/05 - A (file=hat2.model)",
            IDENTIFIED,
            "Info team_colors_missing [Keep] ()",
            FACE_NECK_ADDED,
        ],
    );

    let face = slot_05_face(&entries);
    assert!(
        face.keys().all(|name| !name.contains("hat2")),
        "{:?}",
        face.keys()
    );
    assert!(face.contains_key("hat.model"), "{:?}", face.keys());
}

#[test]
fn an_entry_without_material_is_written_without_one_whatever_the_folder_s_mtl_files() {
    let sandbox = Sandbox::new("user_xml_no_material");
    let export = "co Midcup Xml";
    let player = format!("exports/{export}/Players/05 - A");
    // No `.mtl` anywhere: the name search would find none for `hat.model`.
    sandbox.write(&format!("{player}/hat.model"), &card_model());
    sandbox.write(
        &format!("{player}/face.xml"),
        face_xml(&[r#"type="face_neck" path="./hat.model""#]).as_bytes(),
    );

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let face = slot_05_face(&entries);
    assert_eq!(names(&face), ["face.xml", "hat.model"]);
    assert_eq!(
        String::from_utf8_lossy(&face["face.xml"]),
        format!(
            "{HEAD}   <model level=\"0\" type=\"face_neck\" path=\"./hat.model\" />\r\n{}",
            dif_tail(&template("face_diff.bin"))
        )
    );
}

#[test]
fn a_kit_reference_packs_every_variant_of_its_model_and_mtl_sets_under_their_own_names() {
    let sandbox = Sandbox::new("user_xml_kit_set");
    let export = "co Midcup Xml";
    let player = format!("exports/{export}/Players/05 - A");
    write_folder(
        &sandbox,
        export,
        "05 - A",
        "face_high",
        &face_xml(&[
            FACE_HIGH,
            r#"type="parts" path="./pants_kitN.model" material="./pants_kitN.mtl""#,
        ]),
    );
    for kit in [1, 2] {
        sandbox.write(&format!("{player}/pants_kit{kit}.model"), &card_model());
        sandbox.write(&format!("{player}/pants_kit{kit}.mtl"), &card_materials());
    }

    let entries = compile_pes17(&sandbox, export, &CLEAN);

    let face = slot_05_face(&entries);
    assert_eq!(
        names(&face),
        [
            "face.xml",
            "face_high.model",
            "face_high.mtl",
            "pants_kit1.model",
            "pants_kit1.mtl",
            "pants_kit2.model",
            "pants_kit2.mtl",
        ]
    );
    assert_eq!(
        sampler_paths(&face["pants_kit2.mtl"]),
        [format!("{SLOT_05_HOME}skin.dds")]
    );
    // The game respells the entry for the kit picked.
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        owned_entries(&[
            ("face_neck", "./face_high.model", "./face_high.mtl"),
            ("parts", "./pants_kitN.model", "./pants_kitN.mtl"),
        ])
    );
}

/// The `<dif>` text of the `face.xml` `bytes` decoded, its line ends and indent left out.
fn decoded_dif(bytes: &[u8]) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).unwrap();
    let start = text.find("<dif>").unwrap() + "<dif>".len();
    let end = text.find("</dif>").unwrap();
    let base64: String = text[start..end]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    STANDARD.decode(base64).unwrap()
}

// the pre-Fox tracer's member xml
#[test]
fn the_fumos_xml_compiles_with_its_references_and_its_dif_as_written() {
    let sandbox = Sandbox::new("user_xml_fumos");
    let export = "co Midcup Fumos";
    let fumos = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/tracer_prefox/old/jp Tracer/Faces/XXX20 - Fumos/face.xml"),
    )
    .unwrap();
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/face.xml"), &fumos);
    let models = [
        "face_high_win32.model",
        "oral_boots_win32.model",
        "oral_glove_l_win32.model",
        "oral_glove_r_win32.model",
    ];
    let materials = ["boots.mtl", "face.mtl", "glove_l.mtl", "glove_r.mtl"];
    for model in models {
        sandbox.write(&format!("{player}/{model}"), &card_model());
    }
    for mtl in materials {
        sandbox.write(&format!("{player}/{mtl}"), &card_materials());
    }
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            "Warning xml_type_unknown [Keep] at Players/05 - A (type=boots)",
            IDENTIFIED,
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let face = slot_05_face(&entries);
    let mut expected: Vec<&str> = models.iter().chain(&materials).copied().collect();
    expected.push("face.xml");
    expected.sort_unstable();
    assert_eq!(names(&face), expected);
    for model in models {
        assert_eq!(face[model], card_model(), "{model}");
    }
    let xml = &face["face.xml"];
    assert_eq!(
        ordered_entries(xml),
        owned_entries(&[
            ("face_neck", "./face_high_*.model", "./face.mtl"),
            ("gloveL", "./oral_glove_l_*.model", "./glove_l.mtl"),
            ("gloveR", "./oral_glove_r_*.model", "./glove_r.mtl"),
            ("boots", "./oral_boots_*.model", "./boots.mtl"),
        ])
    );
    assert_eq!(decoded_dif(xml), decoded_dif(&fumos));
    assert!(xml.ends_with(b"\r\n</dif>\r\n</config>"));
}

#[test]
fn on_pes_15_a_member_s_uniform_entry_is_written_uniform_sub_and_named_by_its_path() {
    let sandbox = Sandbox::new("user_xml_uniform");
    let export = "co Midcup Xml";
    write_folder(
        &sandbox,
        export,
        "05 - A",
        "body",
        &face_xml(&[r#"type="uniform" path="./body.model" material="./body.mtl""#]),
    );

    let run = sandbox.run(&pes15(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, export),
        [
            IDENTIFIED,
            "Info team_colors_missing [Keep] ()",
            "Info xml_uniform_pes15 [Keep] at Players/05 - A (path=./body.model)",
            FACE_NECK_ADDED,
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let face = slot_05_face(&entries);
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        owned_entries(&[
            ("uniform_sub", "./body.model", "./body.mtl"),
            ("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ])
    );
}
