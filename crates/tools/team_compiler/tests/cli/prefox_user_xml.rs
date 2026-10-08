//! A player folder's own `face.xml` (`team_compiler/messages.md` "User-supplied `face.xml`"):
//! read and checked by `check` for PES 2015 to 2017, its errors dropping the folder; ignored
//! for PES 2018 to 2021, whose `compile` builds the folder as without it.

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings};
use crate::models::face_diff_fixture;
use crate::prefox_faces::{card_materials, card_model, materials_naming, pes16, pes17, small_dds};
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
