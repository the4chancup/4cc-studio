//! `compile`'s face folder for each player on PES 2021: none for a player folder holding
//! `ingame_face`, whose models the face would take go to the player's boots instead; a blank
//! one, the bundled face diff alone, for every other player folder with no face model; and a
//! face file in a folder with no face model, which nothing uses, reported by both commands.

use std::collections::BTreeMap;
use std::fs;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_player_file};
use crate::models::{
    body_skl, boots_mesh_count, compile_clean, package_names, template, tracer_boots_mesh_count,
};
use crate::{TEAM_COLORS_MISSING, findings_of};

const BOOTS_05: &str = "Asset/model/character/boots/k0625/#Win/boots.fpk";
const FACE_05: &str = "Asset/model/character/face/real/71405/#Win/face.fpk";
const FACE_07: &str = "Asset/model/character/face/real/71407/#Win/face.fpk";

/// The CPK paths of `entries`.
fn paths(entries: &BTreeMap<String, Vec<u8>>) -> Vec<&str> {
    entries.keys().map(String::as_str).collect()
}

/// Asserts the face package at `path` in `entries` is the blank one: the bundled
/// `face_diff.bin` alone, with its `.fpkd` beside it.
fn assert_blank_face(entries: &BTreeMap<String, Vec<u8>>, path: &str) {
    assert_eq!(package_names(&entries[path]), ["face_diff.bin"], "{path}");
    let package = fpk::FpkFile::read(&entries[path]).unwrap();
    assert_eq!(
        package.get("face_diff.bin").unwrap(),
        template("face_diff.bin"),
        "{path}"
    );
    let fpkd = path.replace(".fpk", ".fpkd");
    assert!(entries.contains_key(&fpkd), "{fpkd}");
}

// TC-MOD-16
#[test]
fn under_ingame_face_no_face_is_built_and_a_model_the_face_would_take_is_the_boots() {
    let sandbox = Sandbox::new("face_ingame_reroute");
    let player = "exports/co - Ingame/Players/05 - A";
    sandbox.write(&format!("{player}/ingame_face"), b"");
    sandbox.write(
        &format!("{player}/torso.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    // A real skeleton that differs from the bundled PES 21 one.
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    sandbox.write(&format!("{player}/torso.skl"), &custom);
    sandbox.write(
        &format!("{player}/glove_l.fmdl"),
        &tracer_player_file("glove_l.fmdl"),
    );
    let glove_fpk = "Asset/model/character/glove/g0625/#Win/glove.fpk";

    // No `fmdl_fcl_hair_fallback` for `torso.fmdl` and no `content_not_yet_compiled`.
    let entries = compile_clean(
        &sandbox,
        "co - Ingame",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=torso.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            TEAM_COLORS_MISSING,
        ],
    );

    assert_eq!(
        paths(&entries),
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            glove_fpk,
            "Asset/model/character/glove/g0625/#Win/glove.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no face/real/71405/ path"
    );
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );
    let boots = fpk::FpkFile::read(&entries[BOOTS_05]).unwrap();
    assert_eq!(boots.get("boots.skl").unwrap(), custom);
    assert_eq!(package_names(&entries[glove_fpk]), ["glove_l.fmdl"]);
}

// TC-MOD-17
#[test]
fn under_ingame_face_a_boots_link_beside_a_boots_model_combines_and_a_plain_one_loads_the_shared_folder()
 {
    let sandbox = Sandbox::new("face_ingame_link");
    let export = "exports/co - Linked";
    sandbox.write(&format!("{export}/Players/05 - A/ingame_face"), b"");
    sandbox.write(
        &format!("{export}/Players/05 - A/kit_boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(&format!("{export}/Players/07 - B/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    let k0644 = "Asset/model/character/boots/k0644/#Win/boots.fpk";

    let entries = compile_clean(
        &sandbox,
        "co - Linked",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            TEAM_COLORS_MISSING,
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
        ],
    );

    // Slot 07 has no marker: it gets the blank face; slot 05 none.
    assert_eq!(
        paths(&entries),
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            k0644,
            "Asset/model/character/boots/k0644/#Win/boots.fpkd",
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ]
    );
    assert_eq!(
        boots_mesh_count(&entries, BOOTS_05),
        2 * tracer_boots_mesh_count(),
        "Crocs's meshes plus the local model's"
    );
    assert_eq!(boots_mesh_count(&entries, k0644), tracer_boots_mesh_count());
    assert_blank_face(&entries, FACE_07);
}

// TC-MOD-18
#[test]
fn under_ingame_face_an_empty_face_subfolder_is_ignored() {
    let sandbox = Sandbox::new("face_ingame_empty_face");
    let player = "exports/co - Empty/Players/05 - A";
    sandbox.write(&format!("{player}/ingame_face"), b"");
    sandbox.write(
        &format!("{player}/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    // Slot 07 of TC-MOD-19's export, an empty `face/` alone, shows such a folder is seen.
    fs::create_dir_all(sandbox.root.join(format!("{player}/face"))).unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Empty"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            TEAM_COLORS_MISSING,
        ]
    );
    assert!(
        lines.iter().all(|line| !line.contains("face/")),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        paths(&entries),
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no face/real/71405/ path"
    );
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );
}

// TC-MOD-19
#[test]
fn a_player_folder_with_no_face_model_gets_the_blank_face() {
    let sandbox = Sandbox::new("face_blank");
    let export = "exports/co - Blank";
    sandbox.write(
        &format!("{export}/Players/05 - A/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    // Slot 07 is an empty `face/` alone: without that folder the export holds nothing for
    // slot 07, so its face shows the empty folder is seen.
    fs::create_dir_all(sandbox.root.join(format!("{export}/Players/07 - B/face"))).unwrap();

    let entries = compile_clean(
        &sandbox,
        "co - Blank",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            TEAM_COLORS_MISSING,
        ],
    );

    assert_eq!(
        paths(&entries),
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            FACE_05,
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            FACE_07,
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ]
    );
    assert_blank_face(&entries, FACE_05);
    assert_blank_face(&entries, FACE_07);
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );
}

// TC-MOD-32
#[test]
fn a_face_file_in_a_folder_with_no_face_model_is_not_used_and_reported() {
    let sandbox = Sandbox::new("face_file_not_used");
    let export = "exports/co - Unused";
    // The tracer's `face_diff.bin`: a face diff the game reads, other than the bundled one.
    let own = tracer_player_file("face_diff.bin");
    assert_ne!(own, template("face_diff.bin"));
    for player in ["Players/05 - A", "Players/07 - B"] {
        sandbox.write(
            &format!("{export}/{player}/boots.fmdl"),
            &tracer_player_file("boots.fmdl"),
        );
        sandbox.write(&format!("{export}/{player}/face_diff.bin"), &own);
    }
    sandbox.write(&format!("{export}/Players/07 - B/ingame_face"), b"");
    let validated = [
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
        "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
        "Info export_identified [Keep] (team=/co/, id=714)",
        "Info face_file_not_used [Keep] at Players/05 - A (file=face_diff.bin)",
        "Info face_file_not_used [Keep] at Players/07 - B (file=face_diff.bin)",
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(findings_of(&check.messages(), "co - Unused"), validated);
    assert_eq!(check.exit_code(), 0);
    let entries = compile_clean(
        &sandbox,
        "co - Unused",
        &[&validated[..], &[TEAM_COLORS_MISSING]].concat(),
    );

    assert_eq!(
        paths(&entries),
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            FACE_05,
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no face/real/71407/ path"
    );
    assert_blank_face(&entries, FACE_05);
}
