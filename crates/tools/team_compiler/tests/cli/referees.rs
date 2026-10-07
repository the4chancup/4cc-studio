//! `compile` over a refs export on Fox (`team_compiler/blue_port.md` "Referee export
//! processing"): each referee folder prepared once and emitted under every slot `players.txt`
//! maps it to, as `face/real/referee0NN`, `k99NN` and `g99NN`, its textures once in team 999's
//! common subfolder of its name, and a link of his made a part of his slots' own packages.

use std::collections::BTreeMap;

use crate::common::{Run, Sandbox};
use crate::common_links::texture_directories;
use crate::compile::{
    compiled_players, cpk_entries, pes21_settings, tracer_kit, tracer_player_file,
};
use crate::compile_exports::{TEAM_COLOR, bundled_team_color};
use crate::models::package_names;
use crate::sideload::slashed;
use crate::textures::tracer_model_renaming;
use crate::{clean_model, findings_of, snapshot};

/// The refs export's folder in the sandbox.
const REFS: &str = "exports/refs Cup";

/// Where `Ref A`'s textures go: team 999's common subfolder of his folder's name.
const REF_A_TEXTURES: &str = "Asset/model/character/common/999/Ref A/sourceimages/#windx11";

/// The CPK path of referee slot `slot`'s package `stem` (`face`, `boots`, `glove`) of `kind`
/// (`face/real/referee0`, `boots/k99`, `glove/g99`).
fn referee_package(kind: &str, slot: &str, stem: &str) -> String {
    format!("Asset/model/character/{kind}{slot}/#Win/{stem}.fpk")
}

/// Writes `Ref A` mapped to each of `slots`: a face model and a boots model, each naming
/// `skin.dds`, and `skin.dds` (TC-REF-01's referee).
fn write_ref_a(sandbox: &Sandbox, slots: &[&str]) {
    let roster: String = slots.iter().map(|slot| format!("{slot} Ref A\n")).collect();
    sandbox.write(&format!("{REFS}/players.txt"), roster.as_bytes());
    let folder = format!("{REFS}/Players/Ref A");
    sandbox.write(
        &format!("{folder}/face_high.fmdl"),
        &tracer_model_renaming("fcl_hair.fmdl", &[("shirt.dds", "skin.dds")]),
    );
    sandbox.write(
        &format!("{folder}/boots.fmdl"),
        &tracer_model_renaming("boots.fmdl", &[("shirt.dds", "skin.dds")]),
    );
    sandbox.write(
        &format!("{folder}/skin.dds"),
        &tracer_player_file("shirt.dds"),
    );
}

/// `compile --no-deploy` in `sandbox` for PES 21: the run and its CPK's entries.
fn compile(sandbox: &Sandbox) -> (Run, BTreeMap<String, Vec<u8>>) {
    let run = sandbox.run(&pes21_settings(sandbox), &["compile", "--no-deploy"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    (run, entries)
}

#[test]
fn a_referee_folder_is_emitted_under_each_of_his_slots_with_his_textures_once() {
    let sandbox = Sandbox::new("ref_slots");
    write_ref_a(&sandbox, &["01", "20", "35"]);

    let (run, entries) = compile(&sandbox);

    assert_eq!(
        findings_of(&run.messages(), "refs Cup"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/Ref A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/Ref A (file=face_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=referees)",
        ],
        "no team_colors_missing: a referee has no team record"
    );
    assert_eq!(run.exit_code(), 0);
    for slot in ["01", "20", "35"] {
        for (kind, stem) in [("face/real/referee0", "face"), ("boots/k99", "boots")] {
            let package = referee_package(kind, slot, stem);
            assert!(entries.contains_key(&package), "{package}");
            assert!(entries.contains_key(&format!("{package}d")), "{package}d");
        }
    }
    let skins: Vec<&String> = entries
        .keys()
        .filter(|path| path.ends_with("/skin.ftex"))
        .collect();
    assert_eq!(skins, [&format!("{REF_A_TEXTURES}/skin.ftex")]);
    let face =
        fpk::FpkFile::read(&entries[&referee_package("face/real/referee0", "01", "face")]).unwrap();
    let directories = texture_directories(face.get("face_high.fmdl").unwrap(), "skin.dds");
    assert!(!directories.is_empty());
    assert!(
        directories
            .iter()
            .all(|directory| directory
                == "/Assets/pes16/model/character/common/999/Ref A/sourceimages/"),
        "{directories:?}"
    );
    // No team record changes for the referees, and no note is collected without one.
    assert_eq!(entries[TEAM_COLOR], bundled_team_color());
    assert!(!sandbox.root.join("output/teamnotes.txt").exists());
}

// TC-REF-03
#[test]
fn a_shared_face_two_referees_link_is_merged_into_each_one_s_face() {
    let sandbox = Sandbox::new("ref_shared_face");
    sandbox.write(&format!("{REFS}/players.txt"), b"01 Ref A\n02 Ref B\n");
    sandbox.write(&format!("{REFS}/Faces/Base/face_high.fmdl"), &clean_model());
    sandbox.write(&format!("{REFS}/Players/Ref A/Base.face"), b"");
    sandbox.write(
        &format!("{REFS}/Players/Ref A/hair_high.fmdl"),
        &clean_model(),
    );
    sandbox.write(&format!("{REFS}/Players/Ref B/Base.face"), b"");

    let (run, entries) = compile(&sandbox);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        package_names(&entries[&referee_package("face/real/referee0", "01", "face")]),
        ["face_diff.bin", "face_high.fmdl", "hair_high.fmdl"]
    );
    assert_eq!(
        package_names(&entries[&referee_package("face/real/referee0", "02", "face")]),
        ["face_diff.bin", "face_high.fmdl"]
    );
    for path in entries.keys() {
        assert!(!path.contains("Base"), "{path}");
    }
}

// TC-REF-05
#[test]
fn test_mode_writes_a_referee_folder_s_processed_files_once() {
    let sandbox = Sandbox::new("ref_test_mode");
    write_ref_a(&sandbox, &["01", "20"]);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--mode", "test"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let written: Vec<String> = snapshot(&sandbox.root.join("output/test_output"))
        .into_keys()
        .map(|path| slashed(&path))
        .filter(|path| !path.starts_with("_bins/"))
        .collect();
    assert_eq!(
        written,
        [
            "refs Cup/Players/Ref A/boots.fmdl",
            "refs Cup/Players/Ref A/boots.skl",
            "refs Cup/Players/Ref A/face_diff.bin",
            "refs Cup/Players/Ref A/face_high.fmdl",
            "refs Cup/Players/Ref A/skin.ftex",
        ]
    );
    for path in snapshot(&sandbox.root.join("output/test_output")).keys() {
        assert!(!slashed(path).contains("referee020"), "{path:?}");
    }
}

// TC-REF-10
#[test]
fn a_referee_s_link_to_shared_boots_is_written_as_each_of_his_slots_boots() {
    let sandbox = Sandbox::new("ref_shared_boots");
    sandbox.write(&format!("{REFS}/players.txt"), b"01 Ref A\n20 Ref A\n");
    sandbox.write(&format!("{REFS}/Players/Ref A/Studs.boots"), b"");
    sandbox.write(
        &format!("{REFS}/Boots/Studs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{REFS}/Boots/Studs/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let (run, entries) = compile(&sandbox);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let boots: Vec<&str> = entries
        .keys()
        .filter_map(|path| path.strip_prefix("Asset/model/character/boots/"))
        .collect();
    assert_eq!(
        boots,
        [
            "k9901/#Win/boots.fpk",
            "k9901/#Win/boots.fpkd",
            "k9920/#Win/boots.fpk",
            "k9920/#Win/boots.fpkd",
        ]
    );
    for slot in ["01", "20"] {
        let names = package_names(&entries[&referee_package("boots/k99", slot, "boots")]);
        assert!(
            names.contains(&"boots.fmdl".to_owned()),
            "{slot}: {names:?}"
        );
    }
}

#[test]
fn a_refs_export_s_kit_is_named_and_the_export_beside_it_compiled() {
    let sandbox = Sandbox::new("ref_kit_named");
    write_ref_a(&sandbox, &["01"]);
    sandbox.write(&format!("{REFS}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.copy_tracer("egg Midcup Tracer");

    let (run, _) = compile(&sandbox);

    let lines = run.messages();
    assert!(
        lines.contains(
            &"refs Cup: Error content_not_yet_compiled [DropExport] (what=Kits/p1)".to_owned()
        ),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [79205]);
}
