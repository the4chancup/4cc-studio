//! Archive sources: `.zip` and `.7z` exports read beside folder ones, and the ones refused.

use std::fs;
use std::path::Path;

use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};
use pes_version::PesVersion;

use crate::common::Sandbox;
use crate::compile::{compiled_players, cpk_entries, kit_texture, pes21_settings};
use crate::compile_exports::{UNI_COLOR, bundled_uni_color, uni_record, with_uni_record};
use crate::{CLEAN_PLAYER, clean_model, findings_of, snapshot, source_fixture};

/// The folder `exports/co - Spring/`, the same export as the `co - Spring` fixtures: git keeps no
/// empty folder, so it is built here.
fn spring_folder(sandbox: &Sandbox) {
    sandbox.write("exports/co - Spring/players.txt", b"01 Keeper\n");
    sandbox.write(
        "exports/co - Spring/notes.txt",
        b"Spring kit placeholder.\n",
    );
    fs::create_dir_all(sandbox.root.join("exports/co - Spring/Kits/p2")).unwrap();
}

/// What `check` reports about the `co - Spring` export, in any of its three forms. The roster
/// line names a player folder the export does not hold, so its finding shows the roster was read.
const SPRING_FINDINGS: [&str; 3] = [
    "Error players_txt_target_missing [DropSlot] at players.txt line 1 slot Some(1) (folder=Keeper)",
    "Info notes_found [Keep] at notes.txt ()",
    "Info export_identified [Keep] (team=/co/, id=714)",
];

/// `SPRING_FINDINGS` of a `co - Spring` source checked beside other exports of `/co/`, then
/// the finding that skips each of them, naming `exports`.
fn spring_findings_beside(exports: &str) -> Vec<String> {
    SPRING_FINDINGS
        .iter()
        .map(|line| (*line).to_owned())
        .chain([format!(
            "Error duplicate_aesthetics_export [DropExport] (id=714, exports={exports})"
        )])
        .collect()
}

// TC-SRC-01
#[test]
fn one_export_as_a_folder_a_zip_and_a_7z_reports_and_compiles_the_same() {
    let sandbox = Sandbox::new("same_export_three_ways");
    spring_folder(&sandbox);
    sandbox.copy_fixture("co - Spring.zip", "exports");
    sandbox.copy_fixture("co - Spring.7z", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    // Three exports of one team: each is skipped, naming the three.
    let expected = spring_findings_beside("co - Spring, co - Spring.7z, co - Spring.zip");
    for source in ["co - Spring", "co - Spring.zip", "co - Spring.7z"] {
        assert_eq!(findings_of(&lines, source), expected, "{lines:#?}");
    }
    assert_eq!(lines.len(), 3 * expected.len(), "{lines:#?}");
    assert_eq!(run.exit_code(), 1);

    // Each source compiled alone: the empty kit folder p2 is the placeholder kit.
    let settings = pes21_settings(&sandbox);
    let mut archives = Vec::new();
    for source in ["co - Spring", "co - Spring.zip", "co - Spring.7z"] {
        let run = sandbox.run(
            &settings,
            &[
                "compile",
                "--export",
                &sandbox.arg(&format!("exports/{source}")),
            ],
        );
        let lines = run.messages();
        let expected: Vec<&str> = SPRING_FINDINGS
            .iter()
            .copied()
            .chain([
                "Info team_colors_missing [Keep] ()",
                "Info kit_config_generated [Keep] at Kits/p2 ()",
                "Info kit_placeholder [Keep] at Kits/p2 ()",
                "Warning kit_colors_missing [Keep] at Kits/p2 ()",
            ])
            .collect();
        assert_eq!(findings_of(&lines, source), expected, "{lines:#?}");
        assert_eq!(run.exit_code(), 1, "{source}");
        archives.push(cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk")));
    }
    assert!(
        archives[0] == archives[1],
        "the folder's and the zip's CPKs differ"
    );
    assert!(
        archives[0] == archives[2],
        "the folder's and the 7z's CPKs differ"
    );

    let placeholder = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/kits/placeholder_kit.dds"),
    )
    .unwrap();
    let names = texture_names(
        714,
        KitSlot::P2,
        TexturePresence {
            kit: true,
            ..TexturePresence::default()
        },
    );
    let config = KitConfig::template().encode_with_names(PesVersion::Pes21, &names);
    let paths: Vec<&str> = archives[0].keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            kit_texture("u0714p2"),
            "common/character0/model/character/uniform/team/714/714_DEF_2nd_realUni.bin".to_owned(),
            "common/character0/model/character/uniform/team/UniColor.bin".to_owned(),
            "common/character0/model/character/uniform/team/UniformParameter.bin".to_owned(),
            "common/etc/TeamColor.bin".to_owned(),
        ]
    );
    assert!(
        archives[0][&kit_texture("u0714p2")]
            == ftex::dds_to_ftex(&placeholder, ftex::ColorSpace::Normal).unwrap(),
        "the p2 texture is not the placeholder converted"
    );
    assert_eq!(
        archives[0]["common/character0/model/character/uniform/team/714/714_DEF_2nd_realUni.bin"],
        config
    );
}

// TC-BIN-02
#[test]
fn the_empty_kit_folder_of_tc_src_01_s_export_gets_magenta_black_and_icon_3() {
    let sandbox = Sandbox::new("same_export_kit_colors");
    spring_folder(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 1, "the roster line naming no folder");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // p2 is kit 1, the second of team 714's eight kits in the base: replaced in place.
    let base = bundled_uni_color();
    let mut record = uni_record(&base, 714).to_vec();
    record[13..21].copy_from_slice(&[0x01, 0x03, 0xff, 0x00, 0xff, 0x00, 0x00, 0x00]);
    assert!(
        entries[UNI_COLOR] == with_uni_record(&base, 714, &record),
        "UniColor.bin is the base with p2's entry magenta, black and icon 3"
    );
}

// TC-SRC-02
#[test]
fn a_folder_and_an_archive_sharing_a_stem_are_two_exports() {
    let sandbox = Sandbox::new("folder_and_archive");
    spring_folder(&sandbox);
    sandbox.copy_fixture("co - Spring.zip", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    let expected = spring_findings_beside("co - Spring, co - Spring.zip");
    assert_eq!(findings_of(&lines, "co - Spring"), expected);
    assert_eq!(findings_of(&lines, "co - Spring.zip"), expected);
    assert_eq!(lines.len(), 2 * expected.len(), "{lines:#?}");
}

#[test]
fn a_corrupt_archive_is_skipped_and_the_export_beside_it_is_still_checked() {
    let sandbox = Sandbox::new("corrupt_archive");
    // The scan compares the extension in any case.
    sandbox.write("exports/co - Broken.ZIP", b"not a zip at all");
    sandbox.write(
        &format!("exports/co - Spring/{CLEAN_PLAYER}"),
        &clean_model(),
    );

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Broken.ZIP"),
        [format!(
            "Error export_extract_failed [DropExport] (path={}, error=zip: invalid Zip archive: Could not find EOCD)",
            sandbox.display("exports/co - Broken.ZIP")
        )]
    );
    assert_eq!(
        findings_of(&lines, "co - Spring"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-SRC-07
#[test]
fn an_archive_whose_names_collide_or_escape_is_skipped_naming_the_path() {
    let sandbox = Sandbox::new("refused_listings");
    sandbox.copy_fixture("co - Case.zip", "exports");
    sandbox.copy_fixture("co - Escape.zip", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Case.zip"),
        [
            "Error export_extract_failed [DropExport] (path=Players.txt, error=path collides with existing entry players.txt)"
        ]
    );
    assert_eq!(
        findings_of(&lines, "co - Escape.zip"),
        [format!(
            "Error export_extract_failed [DropExport] (path={}, error=invalid entry name \"../x\")",
            sandbox.display("exports/co - Escape.zip")
        )]
    );
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn export_paths_may_name_an_archive_outside_the_root() {
    let sandbox = Sandbox::new("named_archive");
    sandbox.write(&format!("exports/co - A/{CLEAN_PLAYER}"), &clean_model());
    sandbox.write(&format!("exports/co - C/{CLEAN_PLAYER}"), &clean_model());
    sandbox.copy_fixture("co - Spring.zip", "elsewhere");

    let run = sandbox.run(
        "",
        &[
            "check",
            "--export",
            &sandbox.arg("elsewhere/co - Spring.zip"),
            "--export",
            &sandbox.arg("exports/co - A"),
        ],
    );

    let lines = run.messages();
    let duplicate =
        "Error duplicate_aesthetics_export [DropExport] (id=714, exports=co - A, co - Spring.zip)";
    assert_eq!(
        findings_of(&lines, "co - A"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            duplicate
        ]
    );
    let spring = spring_findings_beside("co - A, co - Spring.zip");
    assert_eq!(findings_of(&lines, "co - Spring.zip"), spring);
    // co - C, in the root but not named, is not reported.
    assert_eq!(lines.len(), 2 + spring.len(), "{lines:#?}");
}

#[test]
fn check_leaves_every_archive_and_nested_export_as_it_was() {
    let sandbox = Sandbox::new("archives_untouched");
    sandbox.copy_fixture("co - Spring.zip", "exports");
    sandbox.copy_fixture("co - Spring.7z", "exports");
    sandbox.write(
        &format!("exports/co - Nested/wrapper/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    let exports = sandbox.root.join("exports");
    let before = snapshot(&exports);

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    let exports_of_714 = "co - Nested, co - Spring.7z, co - Spring.zip";
    assert_eq!(
        findings_of(&lines, "co - Nested"),
        [
            "Warning nested_folders_fixed [Keep] (folder=wrapper)".to_owned(),
            "Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
            format!(
                "Error duplicate_aesthetics_export [DropExport] (id=714, exports={exports_of_714})"
            ),
        ]
    );
    let spring = spring_findings_beside(exports_of_714);
    assert_eq!(findings_of(&lines, "co - Spring.7z"), spring);
    assert_eq!(findings_of(&lines, "co - Spring.zip"), spring);
    assert_eq!(snapshot(&exports), before);
}

// TC-SRC-09
#[test]
fn check_and_compile_leave_every_source_as_it_was() {
    let sandbox = Sandbox::new("sources_untouched");
    sandbox.copy_tracer("egg Tracer");
    sandbox.write("exports/co - Zip.zip", &source_fixture("egg Tracer.zip"));
    sandbox.write("exports/dbg - Seven.7z", &source_fixture("egg Tracer.7z"));
    sandbox.copy_tracer_face("exports/esg - Nested/wrapper/Players/03 - A");
    let exports = sandbox.root.join("exports");
    let before = snapshot(&exports);

    let settings = pes21_settings(&sandbox);
    assert_eq!(sandbox.run(&settings, &["check"]).exit_code(), 0);
    assert_eq!(snapshot(&exports), before, "check");
    assert_eq!(sandbox.run(&settings, &["compile"]).exit_code(), 0);
    assert_eq!(snapshot(&exports), before, "compile");
    assert_eq!(compiled_players(&sandbox), [71405, 79005, 79205, 79303]);
}
