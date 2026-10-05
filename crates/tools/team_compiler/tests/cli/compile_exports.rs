//! `compile` over each export's content: the roster, the kits, a nested root, the root
//! files, the source kinds, and the run across several exports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use dds_convert::{BlockCodec, SourceFormat, decode};
use kit_config::{KitConfig, KitSlot};
use pes_version::PesVersion;
use studio_core::PipelineEvent;

use crate::common::Sandbox;
use crate::compile::{
    compiled_kits, compiled_players, compiled_portraits, cpk_entries, pass_through_settings,
    pes_settings, pes21_settings, tracer_kit, tracer_player_file, tracer_portrait,
};
use crate::textures::{bc1_dds, texture_fixture};
use crate::{CLEAN_PLAYER, TEAM_COLORS_MISSING, clean_model, findings_of, source_fixture};

// TC-ROS-01
#[test]
fn without_players_txt_each_folder_is_the_player_its_number_names() {
    let sandbox = Sandbox::new("ros_numbered");
    sandbox.copy_tracer_face("exports/co - Numbers/Players/03 - A");
    sandbox.copy_tracer_face("exports/co - Numbers/Players/15 - B");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Numbers"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/15 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/15 - B (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/15 - B (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 71415]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROS-04
#[test]
fn players_txt_maps_a_folder_by_name_and_leaves_an_unlisted_one_out() {
    let sandbox = Sandbox::new("ros_named");
    sandbox.write("exports/co - Roster/players.txt", b"03 snuffy\n");
    sandbox.copy_tracer_face("exports/co - Roster/Players/Snuffy");
    sandbox.copy_tracer_face("exports/co - Roster/Players/15 - B");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Roster"),
        [
            "Warning player_unlisted [DropFolder] at Players/15 - B ()",
            "Info fmdl_weights_not_normalized [Keep] at Players/Snuffy (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/Snuffy (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/Snuffy (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROS-05
#[test]
fn a_folder_listed_under_two_slots_is_compiled_for_both() {
    let sandbox = Sandbox::new("ros_two_slots");
    sandbox.write("exports/co - Twice/players.txt", b"03 A\n07 A\n");
    sandbox.copy_tracer_face("exports/co - Twice/Players/A");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Twice"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 71407]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROS-06
#[test]
fn each_bad_roster_line_is_reported_and_only_the_good_one_compiled() {
    let sandbox = Sandbox::new("ros_bad_lines");
    sandbox.write(
        "exports/co - Lines/players.txt",
        b"x3 A\n24 B\n05 Nobody\n07 C\n",
    );
    for folder in ["A", "B", "C"] {
        sandbox.copy_tracer_face(&format!("exports/co - Lines/Players/{folder}"));
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Lines"),
        [
            "Error players_txt_line_invalid [DropSlot] at players.txt line 1 slot None ()",
            "Error players_txt_slot_invalid [DropSlot] at players.txt line 2 slot Some(24) ()",
            "Error players_txt_target_missing [DropSlot] at players.txt line 3 slot Some(5) (folder=Nobody)",
            "Warning player_unlisted [DropFolder] at Players/A ()",
            "Info fmdl_weights_not_normalized [Keep] at Players/C (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/C (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/C (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71407]);
    assert_eq!(run.exit_code(), 1);
}

// TC-ROS-09
#[test]
fn an_empty_players_txt_compiles_the_kits_and_no_player() {
    let sandbox = Sandbox::new("ros_empty_roster");
    sandbox.write("exports/co - Empty/players.txt", b"");
    sandbox.copy_tracer_face("exports/co - Empty/Players/03 - A");
    sandbox.write("exports/co - Empty/Kits/p1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Empty"),
        [
            "Warning player_unlisted [DropFolder] at Players/03 - A ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
        ]
    );
    assert!(compiled_players(&sandbox).is_empty());
    assert_eq!(compiled_kits(&sandbox), ["u0714p1"]);
    assert_eq!(run.exit_code(), 0);
}

// TC-KIT-01
#[test]
fn kit_folders_are_compiled_as_the_slot_their_name_starts_with() {
    let sandbox = Sandbox::new("kit_named");
    sandbox.write("exports/co - Kits/Kits/p1 - Lakers/kit.dds", &tracer_kit());
    sandbox.write("exports/co - Kits/Kits/g1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Kits"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 - Lakers ()",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 - Lakers ()",
            "Info kit_colors_derived [Keep] at Kits/g1 ()",
        ]
    );
    assert_eq!(compiled_kits(&sandbox), ["u0714g1", "u0714p1"]);
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn a_kit_holding_only_a_back_texture_gets_the_placeholder_main_texture() {
    let sandbox = Sandbox::new("kit_back_only");
    sandbox.write("exports/co - Back/Kits/p4/kit_back.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Back"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p4 ()",
            "Info kit_placeholder [Keep] at Kits/p4 ()",
            "Warning kit_colors_missing [Keep] at Kits/p4 ()",
        ]
    );
    assert_eq!(compiled_kits(&sandbox), ["u0714p4", "u0714p4_back"]);
    let names = kit_config::texture_names(
        714,
        kit_config::KitSlot::P4,
        kit_config::TexturePresence {
            kit: true,
            back: true,
            ..kit_config::TexturePresence::default()
        },
    );
    let config =
        kit_config::KitConfig::template().encode_with_names(pes_version::PesVersion::Pes21, &names);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        entries["common/character0/model/character/uniform/team/714/714_DEF_4th_realUni.bin"],
        config
    );
    assert_eq!(run.exit_code(), 0);
}

/// The CPK path of the portrait file `name` (`71405.dds`, `player_71405.dds`).
fn portrait_path(name: &str) -> String {
    format!("common/render/symbol/player/{name}")
}

#[test]
fn dds_portraits_from_both_sources_are_emitted_as_they_are_under_the_version_s_name() {
    let sandbox = Sandbox::new("portraits_dds");
    sandbox.copy_tracer_face("exports/co - Portraits/Players/05 - A");
    sandbox.write(
        "exports/co - Portraits/Portraits/player_07.dds",
        &tracer_kit(),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Portraits"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_portraits(&sandbox), ["71405.dds", "71407.dds"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(entries[&portrait_path("71405.dds")], tracer_portrait());
    assert_eq!(entries[&portrait_path("71407.dds")], tracer_kit());

    // PES 18 names the same files with the `player_` prefix.
    let run = sandbox.run("[common]\npes_version = 18\n", &["compile"]);

    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        compiled_portraits(&sandbox),
        ["player_71405.dds", "player_71407.dds"]
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        entries[&portrait_path("player_71405.dds")],
        tracer_portrait()
    );
    assert_eq!(entries[&portrait_path("player_71407.dds")], tracer_kit());
}

#[test]
fn a_folder_listed_under_two_slots_emits_its_portrait_for_both() {
    let sandbox = Sandbox::new("portraits_two_slots");
    sandbox.write("exports/co - Twice/players.txt", b"03 A\n07 A\n");
    sandbox.copy_tracer_face("exports/co - Twice/Players/A");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_portraits(&sandbox), ["71403.dds", "71407.dds"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    for name in ["71403.dds", "71407.dds"] {
        assert_eq!(entries[&portrait_path(name)], tracer_portrait(), "{name}");
    }
}

// TC-PRT-02
#[test]
fn a_slot_s_two_portraits_skip_the_export_when_they_differ_and_are_one_when_identical() {
    let sandbox = Sandbox::new("portraits_both_sources");
    sandbox.copy_tracer_face("exports/co - Both/Players/05 - A");
    sandbox.write("exports/co - Both/Portraits/player_05.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Both"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Error portrait_conflict [DropExport] (folder_portrait=Players/05 - A/portrait.dds, portraits_file=Portraits/player_05.dds)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());

    // The same bytes in both places are one portrait.
    sandbox.write(
        "exports/co - Both/Portraits/player_05.dds",
        &tracer_portrait(),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Both"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_portraits(&sandbox), ["71405.dds"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(entries[&portrait_path("71405.dds")], tracer_portrait());
}

/// Asserts `dds` is a BC3 DDS of 128x128 with the full 8-level chain: what a raster portrait
/// of the fixtures' size is encoded to.
fn assert_bc3_portrait(dds: &[u8], label: &str) {
    assert!(dds.starts_with(b"DDS "), "{label}: a DDS");
    let decoded = decode(dds, SourceFormat::Dds).unwrap();
    assert_eq!(
        decoded.blocks.map(|blocks| blocks.codec),
        Some(BlockCodec::Bc3),
        "{label}"
    );
    assert_eq!(
        (decoded.width, decoded.height, decoded.mips.len()),
        (128, 128, 8),
        "{label}"
    );
}

// TC-PRT-01
#[test]
fn a_dds_portrait_passes_through_and_a_png_one_is_encoded_to_bc3_under_the_version_s_name() {
    let sandbox = Sandbox::new("portraits_dds_and_png");
    sandbox.copy_tracer_face("exports/co - Portraits/Players/05 - A");
    sandbox.write(
        "exports/co - Portraits/Portraits/player_07.png",
        &texture_fixture("portrait.png"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Portraits"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_portraits(&sandbox), ["71405.dds", "71407.dds"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(entries[&portrait_path("71405.dds")], tracer_portrait());
    assert_bc3_portrait(&entries[&portrait_path("71407.dds")], "PES 21");

    // PES 18 names the same files with the `player_` prefix.
    let run = sandbox.run("[common]\npes_version = 18\n", &["compile"]);

    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        compiled_portraits(&sandbox),
        ["player_71405.dds", "player_71407.dds"]
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        entries[&portrait_path("player_71405.dds")],
        tracer_portrait()
    );
    assert_bc3_portrait(&entries[&portrait_path("player_71407.dds")], "PES 18");
}

/// Writes `exports/co - Odd`: slot 03 the tracer's player, its portrait included, and
/// `Portraits/player_05.dds`, a single-level DDS of 300x300, not a power of two on either side.
fn write_odd_portrait_export(sandbox: &Sandbox) {
    sandbox.copy_tracer_face("exports/co - Odd/Players/03 - A");
    sandbox.write(
        "exports/co - Odd/Portraits/player_05.dds",
        &bc1_dds(300, 300),
    );
}

/// The lines `exports/co - Odd` gets, its portrait's finding with `disposition`.
fn odd_portrait_findings(disposition: &str) -> [String; 5] {
    [
        "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)".to_owned(),
        "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)".to_owned(),
        "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)".to_owned(),
        format!("Error texture_not_pow2 [{disposition}] at Portraits/player_05.dds (file=player_05.dds)"),
        "Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
    ]
}

// TC-PRT-03
#[test]
fn a_portrait_whose_side_is_not_a_power_of_two_is_checked_and_left_out_alone() {
    let sandbox = Sandbox::new("portraits_odd");
    write_odd_portrait_export(&sandbox);

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(
        findings_of(&check.messages(), "co - Odd"),
        odd_portrait_findings("DropFile")
    );
    assert_eq!(check.exit_code(), 1);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Odd"),
        [
            &odd_portrait_findings("DropFile")[..],
            &[TEAM_COLORS_MISSING.to_owned()]
        ]
        .concat()
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [71403]);
    assert_eq!(compiled_portraits(&sandbox), ["71403.dds"]);
}

#[test]
fn pass_through_keeps_a_portrait_whose_side_is_not_a_power_of_two() {
    let sandbox = Sandbox::new("portraits_odd_pass_through");
    write_odd_portrait_export(&sandbox);

    let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Odd"),
        [
            &odd_portrait_findings("Keep")[..],
            &[TEAM_COLORS_MISSING.to_owned()]
        ]
        .concat()
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_portraits(&sandbox), ["71403.dds", "71405.dds"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(entries[&portrait_path("71405.dds")], bc1_dds(300, 300));
}

// TC-KIT-10
#[test]
fn a_kit_mask_on_a_fox_target_is_reported_once_and_not_emitted() {
    let sandbox = Sandbox::new("kit_mask_fox");
    sandbox.write("exports/co - Mask/Kits/p1/kit.dds", &tracer_kit());
    sandbox.write("exports/co - Mask/Kits/p1/kit_mask.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Mask"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_texture_not_used [DropFile] at Kits/p1 (file=kit_mask.dds)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_kits(&sandbox), ["u0714p1"]);
}

// TC-STR-01
#[test]
fn content_nested_one_folder_down_compiles_as_if_at_the_root() {
    let sandbox = Sandbox::new("str_nested");
    sandbox.write("exports/co - Wrapped/notes.txt", b"Notes.\n");
    sandbox.copy_tracer_face("exports/co - Wrapped/wrapper/Players/03 - A");
    sandbox.copy_tracer_face("exports/dbg - Doubled/Players/Players/03 - A");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Wrapped"),
        [
            "Warning nested_folders_fixed [Keep] (folder=wrapper)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info notes_found [Keep] at notes.txt ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(
        findings_of(&lines, "dbg - Doubled"),
        [
            "Warning nested_folders_fixed [Keep] at Players (folder=Players/Players)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/dbg/, id=790)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 79003]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROOT-02
#[test]
fn pass_through_drops_notes_that_are_not_utf8_and_compiles_the_export() {
    let sandbox = Sandbox::new("root_notes_encoding");
    sandbox.copy_tracer("egg Tracer");
    sandbox.write("exports/egg Tracer/notes.txt", b"caf\xe9\n");

    let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "egg Tracer"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)",
            "Error notes_encoding_invalid [DropFile] at notes.txt ()",
            "Info export_identified [Keep] (team=/egg/, id=792)"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [79205]);
    assert_eq!(run.exit_code(), 1);
}

/// Where `compile` collects the notes, in `sandbox`'s output folder, spelled as a finding
/// names it.
fn teamnotes(sandbox: &Sandbox) -> std::path::PathBuf {
    sandbox.root.join("output").join("teamnotes.txt")
}

/// The export `exports/<name>`: one player that compiles with no finding, and a root
/// `notes.txt` of `notes` when given.
fn export_with_notes(sandbox: &Sandbox, name: &str, notes: Option<&[u8]>) {
    sandbox.write(&format!("exports/{name}/{CLEAN_PLAYER}"), &clean_model());
    if let Some(notes) = notes {
        sandbox.write(&format!("exports/{name}/notes.txt"), notes);
    }
}

/// The export `exports/b - Skipped`, with a root `notes.txt`, which validation skips: its
/// `players.txt` maps slot 03 twice.
fn skipped_export_with_notes(sandbox: &Sandbox) {
    sandbox.write("exports/b - Skipped/players.txt", b"03 A\n03 B\n");
    sandbox.write(
        "exports/b - Skipped/Players/A/face_high.fmdl",
        &clean_model(),
    );
    sandbox.write(
        "exports/b - Skipped/Players/B/face_high.fmdl",
        &clean_model(),
    );
    sandbox.write("exports/b - Skipped/notes.txt", b"Not compiled.\n");
}

// TC-ROOT-09
#[test]
fn compile_collects_the_compiled_exports_notes_into_teamnotes_txt_and_removes_a_stale_one() {
    let sandbox = Sandbox::new("root_teamnotes");
    export_with_notes(&sandbox, "co - Notes", Some(b"Hello"));
    export_with_notes(
        &sandbox,
        "a - Notes",
        Some(b"First line\r\nsecond line\r\n"),
    );
    skipped_export_with_notes(&sandbox);
    let cpk = sandbox.root.join("output/4cc_99_test.cpk");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "a - Notes: Info notes_found [Keep] at notes.txt ()",
            "a - Notes: Info export_identified [Keep] (team=/a/, id=702)",
            "b - Skipped: Error players_txt_slot_duplicate [DropExport] at players.txt line 2 slot Some(3) ()",
            "b - Skipped: Info notes_found [Keep] at notes.txt ()",
            "co - Notes: Info notes_found [Keep] at notes.txt ()",
            "co - Notes: Info export_identified [Keep] (team=/co/, id=714)",
            "a - Notes: Info team_colors_missing [Keep] ()",
            "co - Notes: Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [70203, 71403]);
    assert_eq!(
        fs::read_to_string(teamnotes(&sandbox)).unwrap(),
        "--- /a/ ---\nFirst line\nsecond line\n\n--- /co/ ---\nHello\n"
    );

    // Compiled again without the notes: the file left over would show notes no export of
    // this run has. The CPK is removed first, so the run is seen writing its own.
    fs::remove_file(sandbox.root.join("exports/co - Notes/notes.txt")).unwrap();
    fs::remove_file(sandbox.root.join("exports/a - Notes/notes.txt")).unwrap();
    fs::remove_file(&cpk).unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 1);
    assert!(cpk.is_file());
    assert!(!teamnotes(&sandbox).exists());

    let fresh = Sandbox::new("root_teamnotes_none");
    export_with_notes(&fresh, "co - Plain", None);

    let run = fresh.run(&pes21_settings(&fresh), &["compile"]);

    assert_eq!(run.exit_code(), 0);
    assert!(fresh.root.join("output/4cc_99_test.cpk").is_file());
    assert!(!teamnotes(&fresh).exists());
}

// TC-ROOT-12
#[test]
fn a_compile_that_writes_no_cpk_leaves_the_previous_teamnotes_txt() {
    let sandbox = Sandbox::new("root_teamnotes_no_cpk");
    skipped_export_with_notes(&sandbox);
    sandbox.write("output/teamnotes.txt", b"--- /co/ ---\nPrevious.\n");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 1);
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());
    assert_eq!(
        fs::read(teamnotes(&sandbox)).unwrap(),
        b"--- /co/ ---\nPrevious.\n"
    );
}

// TC-ROOT-11
#[test]
fn a_teamnotes_txt_that_cannot_be_written_is_reported_and_the_cpk_stays() {
    let sandbox = Sandbox::new("root_teamnotes_write_failed");
    export_with_notes(&sandbox, "co - Notes", Some(b"Hello"));
    // A folder in the file's place, with a file in it so no platform replaces it.
    sandbox.write("output/teamnotes.txt/kept", b"");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let (last, first) = lines.split_last().unwrap();
    assert_eq!(
        first,
        [
            "co - Notes: Info notes_found [Keep] at notes.txt ()",
            "co - Notes: Info export_identified [Keep] (team=/co/, id=714)",
            "co - Notes: Info team_colors_missing [Keep] ()"
        ]
    );
    // The error ends with the platform's own text, so only its shape is fixed.
    let prefix = format!(
        "Error teamnotes_write_failed [Keep] (path={path}, error={path}: ",
        path = teamnotes(&sandbox).display()
    );
    assert!(last.starts_with(&prefix) && last.ends_with(')'), "{last}");
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [71403]);
    assert!(teamnotes(&sandbox).join("kept").is_file());
}

/// The bundled `TeamColor.bin`, which a run with no installed bin builds on.
fn bundled_team_color() -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/bins/TeamColor.bin"))
        .unwrap()
}

/// Where the CPK keeps `TeamColor.bin`.
const TEAM_COLOR: &str = "common/etc/TeamColor.bin";

// TC-ROOT-10
#[test]
fn a_root_colors_txt_sets_the_team_s_colors_and_an_export_without_one_reports_it() {
    let sandbox = Sandbox::new("root_team_colors");
    sandbox.write("exports/co - Colors/colors.txt", b"#c11200\n#414141\n");
    sandbox.write("exports/co - Colors/Kits/p1/kit.dds", &tracer_kit());
    sandbox.write("elsewhere/dbg - Plain/Kits/p1/kit.dds", &tracer_kit());
    let cpk = sandbox.root.join("output/4cc_99_test.cpk");
    let base = bundled_team_color();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "co - Colors: Info export_identified [Keep] (team=/co/, id=714)",
            "co - Colors: Info kit_config_generated [Keep] at Kits/p1 ()",
            "co - Colors: Info kit_colors_derived [Keep] at Kits/p1 ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    // Team 714's record: its ID and count, then its first two colors; the rest is the base's.
    let record = (714 - 100) * 16;
    let mut expected = base.clone();
    expected[record + 4..record + 10].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
    assert!(
        cpk_entries(&cpk).get(TEAM_COLOR) == Some(&expected),
        "TeamColor.bin is the base with team 714's colors set"
    );

    let run = sandbox.run(
        &pes21_settings(&sandbox),
        &["compile", "--export", &sandbox.arg("elsewhere/dbg - Plain")],
    );

    assert_eq!(
        run.messages(),
        [
            "dbg - Plain: Info export_identified [Keep] (team=/dbg/, id=790)",
            "dbg - Plain: Info team_colors_missing [Keep] ()",
            "dbg - Plain: Info kit_config_generated [Keep] at Kits/p1 ()",
            "dbg - Plain: Info kit_colors_derived [Keep] at Kits/p1 ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert!(
        cpk_entries(&cpk).get(TEAM_COLOR) == Some(&base),
        "TeamColor.bin is the bundled base"
    );
}

/// The bundled `UniColor.bin`, which a run with no installed bin builds on.
pub(crate) fn bundled_uni_color() -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/bins/UniColor.bin"))
        .unwrap()
}

/// Where the CPK keeps `UniColor.bin`.
pub(crate) const UNI_COLOR: &str = "common/character0/model/character/uniform/team/UniColor.bin";

/// Team `team_id`'s 85-byte record in the `UniColor.bin` `bin`: the `u32` team ID, the kit
/// count, then ten 8-byte entries (kit number, icon, two colors).
pub(crate) fn uni_record(bin: &[u8], team_id: usize) -> &[u8] {
    let start = (team_id - 100) * 85;
    &bin[start..start + 85]
}

/// `bin` with team `team_id`'s `UniColor.bin` record replaced by `record`.
pub(crate) fn with_uni_record(bin: &[u8], team_id: usize, record: &[u8]) -> Vec<u8> {
    let start = (team_id - 100) * 85;
    let mut replaced = bin.to_vec();
    replaced[start..start + 85].copy_from_slice(record);
    replaced
}

/// The two menu colors `extract_kit_colors` gives for the tracer's `kit.dds`, decoded, as the
/// six bytes a `UniColor.bin` entry holds them in.
fn tracer_kit_colors() -> [u8; 6] {
    let decoded = decode(&tracer_kit(), SourceFormat::Dds).unwrap();
    let colors =
        color_tools::kit::extract_kit_colors(&decoded.mips[0], decoded.width, decoded.height)
            .expect("the tracer's kit texture gives colors");
    let [r1, g1, b1] = colors.color1;
    let [r2, g2, b2] = colors.color2;
    [r1, g1, b1, r2, g2, b2]
}

/// The bundled `UniformParameter.bin` base PES 19 to 21 build on.
fn bundled_uniform_parameter() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../resources/bins/UniformParameter19.bin"),
    )
    .unwrap()
}

/// Where the CPK keeps `UniformParameter.bin`.
const UNIFORM_PARAMETER: &str =
    "common/character0/model/character/uniform/team/UniformParameter.bin";

/// The names of the entries in which the `UniformParameter.bin` `ours` differs from `base`: an
/// entry added, removed or with other bytes.
fn uniform_parameter_changes(ours: &[u8], base: &[u8]) -> Vec<String> {
    let ours = uniparam::UniformParameter::read(ours).unwrap();
    let base = uniparam::UniformParameter::read(base).unwrap();
    let mut names: Vec<String> = ours
        .entries()
        .filter(|(name, bytes)| base.get(name) != Some(*bytes))
        .map(|(name, _)| name.to_owned())
        .collect();
    names.extend(
        base.entries()
            .filter(|(name, _)| ours.get(name).is_none())
            .map(|(name, _)| name.to_owned()),
    );
    names
}

// TC-KIT-11
// TC-BIN-01
#[test]
fn each_kit_s_menu_colors_come_from_its_colors_txt_its_texture_or_the_missing_pair() {
    let sandbox = Sandbox::new("kit_colors");
    let export = "exports/co - Colors";
    sandbox.write(&format!("{export}/colors.txt"), b"#c11200\n#414141\n");
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(
        &format!("{export}/Kits/p1/colors.txt"),
        b"#0a1b2c\n#d4e5f6\n",
    );
    sandbox.write(&format!("{export}/Kits/p2/kit.dds"), &tracer_kit());
    fs::create_dir_all(sandbox.root.join(format!("{export}/Kits/p3"))).unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Colors"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_config_generated [Keep] at Kits/p3 ()",
            "Info kit_placeholder [Keep] at Kits/p3 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
            "Warning kit_colors_missing [Keep] at Kits/p3 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));

    // Team 714's record: kits 0 to 2 (p1 to p3) replaced, the base's kits 3 to 6 and 0x10
    // kept, still eight kits; every other record is the base's.
    let base = bundled_uni_color();
    let mut record = uni_record(&base, 714).to_vec();
    assert_eq!(record[4], 8, "the base's record holds eight kits");
    record[5..13].copy_from_slice(&[0x00, 0x03, 0x0a, 0x1b, 0x2c, 0xd4, 0xe5, 0xf6]);
    record[13..15].copy_from_slice(&[0x01, 0x03]);
    record[15..21].copy_from_slice(&tracer_kit_colors());
    record[21..29].copy_from_slice(&[0x02, 0x03, 0xff, 0x00, 0xff, 0x00, 0x00, 0x00]);
    assert!(
        entries[UNI_COLOR] == with_uni_record(&base, 714, &record),
        "UniColor.bin is the base with team 714's p1 to p3 entries set"
    );

    // The other two bins differ from their bases in team 714's entries alone.
    let mut team_color = bundled_team_color();
    let team = (714 - 100) * 16;
    team_color[team + 4..team + 10].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
    assert!(
        entries[TEAM_COLOR] == team_color,
        "TeamColor.bin is the base with team 714's colors set"
    );
    let changed =
        uniform_parameter_changes(&entries[UNIFORM_PARAMETER], &bundled_uniform_parameter());
    assert_eq!(
        changed,
        [
            "714_DEF_1st_realUni.bin",
            "714_DEF_2nd_realUni.bin",
            "714_DEF_3rd_realUni.bin"
        ]
    );
}

// TC-KIT-25
#[test]
fn a_kit_without_a_texture_derives_its_colors_from_the_inherited_all_kit_dds() {
    let sandbox = Sandbox::new("kit_colors_inherited");
    let export = "exports/co - Shared";
    sandbox.write(&format!("{export}/Kits/all/kit.dds"), &tracer_kit());
    // A config, so the kit is no placeholder; no colors.txt and no kit.dds of its own.
    sandbox.write(&format!("{export}/Kits/p2/config.toml"), SHIRT_144);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Shared"),
        [
            "Info kit_textures_inherited [Keep] at Kits/p2 (stems=kit)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_kits(&sandbox), ["u0714p2"]);
    // p2's entry: kit number 1, icon 3, the pair the extraction gives for all/kit.dds.
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let record = uni_record(&entries[UNI_COLOR], 714);
    let mut entry = vec![0x01, 0x03];
    entry.extend_from_slice(&tracer_kit_colors());
    assert_eq!(record[13..21], entry);
}

// TC-KIT-12
#[test]
fn a_kit_colors_txt_with_one_valid_color_reports_its_bad_line_and_derives_both_colors() {
    let sandbox = Sandbox::new("kit_colors_one_valid");
    let export = "exports/co - Colors";
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(
        &format!("{export}/Kits/p1/colors.txt"),
        b"#0a1b2c\nnot a color\n",
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Colors"),
        [
            "Warning color_entry_invalid [Keep] at Kits/p1/colors.txt (line=2, reason=not one color)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let record = uni_record(&entries[UNI_COLOR], 714);
    assert_eq!(record[5..7], [0x00, 0x03]);
    assert_eq!(record[7..13], tracer_kit_colors());
}

// TC-KIT-13
#[test]
fn a_kit_s_icon_marker_gives_its_menu_icon_and_a_kit_without_one_gets_icon_3() {
    // Team 790's base record is a placeholder whose entries carry icon 0, so icon 3 is
    // written, not kept.
    let sandbox = Sandbox::new("kit_icons");
    let export = "exports/dbg - Icons";
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(&format!("{export}/Kits/p1/icon_7"), b"");
    sandbox.write(&format!("{export}/Kits/p2/kit.dds"), &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let base = bundled_uni_color();
    assert_eq!(
        uni_record(&base, 790)[5..7],
        [0x00, 0x00],
        "the base's icon 0"
    );
    let record = uni_record(&entries[UNI_COLOR], 790);
    assert_eq!(record[4], 2, "two kits");
    assert_eq!(record[5..7], [0x00, 0x07], "p1: icon 7");
    assert_eq!(record[13..15], [0x01, 0x03], "p2: icon 3");
}

// TC-KIT-14
#[test]
fn a_kit_whose_task_fails_keeps_its_base_entries_and_the_kit_beside_it_commits() {
    let sandbox = Sandbox::new("kit_colors_failed_task");
    let export = "exports/co - Fails";
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(
        &format!("{export}/Kits/p1/colors.txt"),
        b"#0a1b2c\n#d4e5f6\n",
    );
    // Bytes no decoder reads, which the deep pass does not refuse: the kit's task fails.
    sandbox.write(&format!("{export}/Kits/p2/kit.dds"), b"not a texture");
    sandbox.write(
        &format!("{export}/Kits/p2/colors.txt"),
        b"#111111\n#222222\n",
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let findings = findings_of(&lines, "co - Fails");
    assert_eq!(
        findings[..4],
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
        ]
    );
    let [failed] = &findings[4..] else {
        panic!("{findings:#?}");
    };
    assert!(
        failed.starts_with(
            "Error folder_pack_failed [DropFolder] at Kits/p2 (error=kit.dds: cannot convert"
        ),
        "{failed}"
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_kits(&sandbox), ["u0714p1"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));

    // p1's entry is written, p2's is the base's.
    let base = bundled_uni_color();
    let mut record = uni_record(&base, 714).to_vec();
    record[5..13].copy_from_slice(&[0x00, 0x03, 0x0a, 0x1b, 0x2c, 0xd4, 0xe5, 0xf6]);
    assert!(
        entries[UNI_COLOR] == with_uni_record(&base, 714, &record),
        "UniColor.bin is the base with team 714's p1 entry set"
    );
    let changed =
        uniform_parameter_changes(&entries[UNIFORM_PARAMETER], &bundled_uniform_parameter());
    assert_eq!(changed, ["714_DEF_1st_realUni.bin"], "nothing of p2");
}

/// The CPK path of team 714's kit config for the kit `ordinal` (`1st`, `2nd`).
fn config_path(ordinal: &str) -> String {
    format!("common/character0/model/character/uniform/team/714/714_DEF_{ordinal}_realUni.bin")
}

/// The kit config team 714's kit `ordinal` was emitted as in `entries`, decoded for `version`.
fn emitted_config(
    entries: &BTreeMap<String, Vec<u8>>,
    ordinal: &str,
    version: PesVersion,
) -> KitConfig {
    KitConfig::decode(&entries[&config_path(ordinal)], version).unwrap()
}

/// A kit config carrying shirt model 144 and the template's values otherwise, the FPC shorts
/// and collars among them.
const SHIRT_144: &[u8] = b"[shirt]\nmodel = 144\n";

// TC-KIT-15
#[test]
fn an_fpc_on_player_writes_the_fpc_values_into_every_kit_config() {
    let sandbox = Sandbox::new("kit_fpc_on");
    let export = "exports/co - Fpc";
    sandbox.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &clean_model(),
    );
    sandbox.write(&format!("{export}/Players/05 - A/fpc_on"), b"");
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(&format!("{export}/Kits/p1/config.toml"), SHIRT_144);
    sandbox.write(&format!("{export}/Kits/p2/kit.dds"), &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Fpc"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_config_fpc_adjusted [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert_eq!(compiled_players(&sandbox), [71405]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    for ordinal in ["1st", "2nd"] {
        let config = emitted_config(&entries, ordinal, PesVersion::Pes21);
        assert!(kit_config::matches_fpc(&config), "{ordinal}");
    }
}

// TC-KIT-16
#[test]
fn without_fpc_on_supplied_kit_configs_are_emitted_as_they_are() {
    let sandbox = Sandbox::new("kit_fpc_unknown");
    let export = "exports/co - Plain";
    // `fpc_off` is a statement about its player alone.
    sandbox.write(&format!("{export}/{CLEAN_PLAYER}"), &clean_model());
    sandbox.write(&format!("{export}/Players/03 - A/fpc_off"), b"");
    let fpc: &[u8] =
        b"[shirt]\nmodel = 176\ncollar = 105\nwinter_collar = 105\n\n[shorts]\nmodel = 16\n";
    for (slot, config) in [("p1", fpc), ("p2", SHIRT_144)] {
        sandbox.write(&format!("{export}/Kits/{slot}/kit.dds"), &tracer_kit());
        sandbox.write(&format!("{export}/Kits/{slot}/config.toml"), config);
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert!(
        lines
            .iter()
            .all(|line| !line.contains("kit_config_fpc_adjusted")),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    for (slot, ordinal, config) in [(KitSlot::P1, "1st", fpc), (KitSlot::P2, "2nd", SHIRT_144)] {
        let names = kit_config::texture_names(
            714,
            slot,
            kit_config::TexturePresence {
                kit: true,
                ..kit_config::TexturePresence::default()
            },
        );
        let supplied = KitConfig::from_toml(std::str::from_utf8(config).unwrap()).unwrap();
        assert_eq!(
            entries[&config_path(ordinal)],
            supplied.encode_with_names(PesVersion::Pes21, &names),
            "{ordinal}"
        );
    }
    assert!(kit_config::matches_fpc(&emitted_config(
        &entries,
        "1st",
        PesVersion::Pes21
    )));
    assert_eq!(
        emitted_config(&entries, "2nd", PesVersion::Pes21)
            .shirt
            .model,
        144
    );
}

// TC-KIT-17
#[test]
fn a_kit_config_value_the_version_cannot_hold_is_reported_and_clamped() {
    let sandbox = Sandbox::new("kit_config_clamped");
    let export = "exports/co - Clamp";
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(
        &format!("{export}/Kits/p1/config.toml"),
        b"[name]\ny = 30\n",
    );
    let clamped = "Warning kit_config_version_clamped [Keep] at Kits/p1/config.toml (field=name.y, value=30, max=16)";

    let run = sandbox.run(&pes_settings(&sandbox, 18), &["compile"]);

    let lines = run.messages();
    assert!(
        findings_of(&lines, "co - Clamp").contains(&clamped),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        emitted_config(&entries, "1st", PesVersion::Pes18).name.y,
        16
    );

    let check = sandbox.run(&pes_settings(&sandbox, 18), &["check"]);
    let lines = check.messages();
    assert!(
        findings_of(&lines, "co - Clamp").contains(&clamped),
        "{lines:#?}"
    );
}

// TC-SRC-03
#[test]
fn a_disabled_export_is_reported_once_by_check_and_compile_and_not_compiled() {
    let sandbox = Sandbox::new("src_disabled");
    sandbox.write("exports/co - One/NO_USE", b"");
    sandbox.copy_tracer_face("exports/co - One/Players/03 - A");
    sandbox.write("exports/dbg - Two/no_use.txt", b"");
    sandbox.copy_tracer_face("exports/dbg - Two/Players/03 - A");

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            run.messages(),
            [
                "co - One: Info export_disabled [DropExport] ()",
                "dbg - Two: Info export_disabled [DropExport] ()",
            ],
            "{command}"
        );
        assert_eq!(run.exit_code(), 0, "{command}");
    }
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());
    assert!(!sandbox.root.join("output/.staging").exists());
}

// TC-SRC-04
#[test]
fn a_balls_export_is_skipped_by_check_and_compile() {
    let sandbox = Sandbox::new("src_balls");
    sandbox.copy_tracer_face("exports/balls Spring/Players/03 - A");
    sandbox.copy_tracer("egg Tracer");

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            findings_of(&run.messages(), "balls Spring"),
            ["Info export_balls_skipped [DropExport] ()"],
            "{command}"
        );
        assert_eq!(run.exit_code(), 0, "{command}");
    }
    assert_eq!(compiled_players(&sandbox), [79205]);
}

// TC-SRC-06
#[test]
fn a_corrupt_zip_is_skipped_and_the_export_beside_it_compiled() {
    let sandbox = Sandbox::new("src_corrupt_zip");
    sandbox.write("exports/co - Broken.zip", b"not a zip at all");
    sandbox.copy_tracer("egg Tracer");

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            findings_of(&run.messages(), "co - Broken.zip"),
            [format!(
                "Error export_extract_failed [DropExport] (path={}, error=zip: invalid Zip archive: Could not find EOCD)",
                sandbox.display("exports/co - Broken.zip")
            )],
            "{command}"
        );
        assert_eq!(run.exit_code(), 1, "{command}");
    }
    assert_eq!(compiled_players(&sandbox), [79205]);
}

// TC-SRC-08
#[test]
fn export_paths_restrict_check_and_compile_to_the_named_exports() {
    let sandbox = Sandbox::new("src_named_exports");
    sandbox.copy_tracer_face("exports/co - A/Players/03 - A");
    sandbox.copy_tracer("egg Tracer");
    sandbox.copy_tracer_face("elsewhere/dbg - D/Players/03 - A");
    let named = [
        "--export",
        &sandbox.arg("elsewhere/dbg - D"),
        "--export",
        &sandbox.arg("exports/co - A"),
    ];

    for command in ["check", "compile"] {
        let args: Vec<&str> = std::iter::once(command).chain(named).collect();
        let run = sandbox.run(&pes21_settings(&sandbox), &args);
        let validated = [
            "co - A: Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "co - A: Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "co - A: Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "co - A: Info export_identified [Keep] (team=/co/, id=714)",
            "dbg - D: Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "dbg - D: Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "dbg - D: Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "dbg - D: Info export_identified [Keep] (team=/dbg/, id=790)",
        ];
        // Planning's notes, which only `compile` makes.
        let planned: &[&str] = if command == "compile" {
            &[
                "co - A: Info team_colors_missing [Keep] ()",
                "dbg - D: Info team_colors_missing [Keep] ()",
            ]
        } else {
            &[]
        };
        assert_eq!(
            run.messages(),
            [&validated[..], planned].concat(),
            "{command}"
        );
        assert_eq!(run.exit_code(), 0, "{command}");
    }
    assert_eq!(compiled_players(&sandbox), [71403, 79003]);
    assert!(compiled_kits(&sandbox).is_empty());
}

// TC-ID-01
#[test]
fn a_zip_export_is_compiled_as_the_team_its_name_starts_with() {
    let sandbox = Sandbox::new("id_zip");
    sandbox.write(
        "exports/co - Spring 2026.zip",
        &source_fixture("egg Tracer.zip"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "co - Spring 2026.zip: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "co - Spring 2026.zip: Info export_identified [Keep] (team=/co/, id=714)",
            "co - Spring 2026.zip: Info team_colors_missing [Keep] ()",
            "co - Spring 2026.zip: Info kit_colors_derived [Keep] at Kits/g1 ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71405]);
    assert_eq!(compiled_kits(&sandbox), ["u0714g1"]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROOT-05
#[test]
fn a_root_notes_txt_that_cannot_be_read_is_dropped_and_the_rest_compiled() {
    let sandbox = Sandbox::new("root_notes_unreadable");
    sandbox.copy_fixture("egg Tracer bad notes.zip", "exports");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "egg Tracer bad notes.zip: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "egg Tracer bad notes.zip: Error source_read_failed [DropFile] at notes.txt (reason=Invalid checksum)",
            "egg Tracer bad notes.zip: Info export_identified [Keep] (team=/egg/, id=792)",
            "egg Tracer bad notes.zip: Info team_colors_missing [Keep] ()",
            "egg Tracer bad notes.zip: Info kit_colors_derived [Keep] at Kits/g1 ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [79205]);
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn a_7z_over_the_memory_cap_compiles() {
    let sandbox = Sandbox::new("seven_z_over_cap");
    sandbox.copy_fixture("egg Tracer.7z", "exports");
    // A cap of a few hundred bytes against the archive's 151 KB decompressed: its permit is
    // oversized, so a task asking the budget for its own while it is held would wait forever.
    let settings = format!(
        "{}memory_cap_percent = 0.000001\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "egg Tracer.7z: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "egg Tracer.7z: Info export_identified [Keep] (team=/egg/, id=792)",
            "egg Tracer.7z: Info team_colors_missing [Keep] ()",
            "egg Tracer.7z: Info kit_colors_derived [Keep] at Kits/g1 ()"
        ]
    );
    assert_eq!(compiled_players(&sandbox), [79205]);
    assert_eq!(compiled_kits(&sandbox), ["u0792g1"]);
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn an_export_is_processed_after_its_last_task_or_after_planning_when_it_has_none() {
    let sandbox = Sandbox::new("processed_order");
    sandbox.write("exports/co - Kits/Kits/p1/kit.dds", &tracer_kit());
    // Fails on the pool, so its finding is the writer's to report.
    sandbox.write("exports/co - Kits/Kits/g1/kit.dds", b"not a texture");
    sandbox.write("exports/dbg - Off/NO_USE", b"");
    sandbox.write(&format!("exports/dbg - Off/{CLEAN_PLAYER}"), &clean_model());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let events: Vec<String> = run
        .events
        .iter()
        .map(|envelope| match &envelope.event {
            PipelineEvent::ExportStarted { export_id, .. } => format!("started {}", export_id.0),
            PipelineEvent::Message(message) => format!("message {}", message.code.code),
            PipelineEvent::ExportProcessed { export_id } => format!("processed {}", export_id.0),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        events,
        [
            "started 0",
            "message export_identified",
            "started 1",
            "message export_disabled",
            "message team_colors_missing",
            "message kit_config_generated",
            "message kit_config_generated",
            "processed 1",
            "message kit_colors_derived",
            "message folder_pack_failed",
            "processed 0",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn the_worker_count_changes_neither_the_findings_nor_the_cpk() {
    let mut outcomes = Vec::new();
    for threads in [1, 8] {
        let sandbox = Sandbox::new(&format!("worker_count_{threads}"));
        sandbox.copy_tracer("egg Tracer");
        sandbox.write("exports/dbg Seven.7z", &source_fixture("egg Tracer.7z"));
        sandbox.write("exports/co - Kits/Kits/p1/kit.dds", &tracer_kit());
        sandbox.write("exports/co - Kits/Kits/g1/kit.dds", &tracer_kit());
        let settings = format!("{}thread_count = {threads}\n", pes21_settings(&sandbox));

        let run = sandbox.run(&settings, &["compile"]);

        assert_eq!(run.exit_code(), 0, "{threads} threads");
        let cpk = fs::read(sandbox.root.join("output/4cc_99_test.cpk")).unwrap();
        outcomes.push((run.messages(), cpk));
    }
    assert_eq!(outcomes[0].0, outcomes[1].0);
    assert_eq!(
        outcomes[0].0,
        [
            "co - Kits: Info export_identified [Keep] (team=/co/, id=714)",
            "dbg Seven.7z: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "dbg Seven.7z: Info export_identified [Keep] (team=/dbg/, id=790)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)",
            "egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)",
            "co - Kits: Info team_colors_missing [Keep] ()",
            "co - Kits: Info kit_config_generated [Keep] at Kits/p1 ()",
            "co - Kits: Info kit_config_generated [Keep] at Kits/g1 ()",
            "dbg Seven.7z: Info team_colors_missing [Keep] ()",
            "co - Kits: Info kit_colors_derived [Keep] at Kits/p1 ()",
            "co - Kits: Info kit_colors_derived [Keep] at Kits/g1 ()",
            "dbg Seven.7z: Info kit_colors_derived [Keep] at Kits/g1 ()"
        ]
    );
    assert!(outcomes[0].1 == outcomes[1].1, "the CPKs differ");
}

// TC-PLN-05
#[test]
fn two_teams_with_players_and_kits_compile_to_the_same_cpk_on_one_worker_and_on_eight() {
    let mut cpks = Vec::new();
    for threads in [1, 8] {
        let sandbox = Sandbox::new(&format!("pln_worker_count_{threads}"));
        for export in ["a - Home", "co - Away"] {
            sandbox.copy_tracer_face(&format!("exports/{export}/Players/03 - A"));
            sandbox.copy_tracer_face(&format!("exports/{export}/Players/07 - B"));
            sandbox.write(&format!("exports/{export}/Kits/p1/kit.dds"), &tracer_kit());
        }
        let settings = format!("{}thread_count = {threads}\n", pes21_settings(&sandbox));

        let run = sandbox.run(&settings, &["compile"]);

        assert_eq!(
            run.exit_code(),
            0,
            "{threads} threads: {:#?}",
            run.messages()
        );
        assert_eq!(compiled_players(&sandbox), [70203, 70207, 71403, 71407]);
        assert_eq!(compiled_kits(&sandbox), ["u0702p1", "u0714p1"]);
        cpks.push(fs::read(sandbox.root.join("output/4cc_99_test.cpk")).unwrap());
    }
    let first_difference = cpks[0]
        .iter()
        .zip(&cpks[1])
        .position(|(one, eight)| one != eight);
    assert_eq!(
        (first_difference, cpks[0].len()),
        (None, cpks[1].len()),
        "the first differing offset, and the two sizes"
    );
}

/// Writes `exports/a - Home`, a valid `/a/` export: one player and one kit.
fn write_a_home(sandbox: &Sandbox) {
    sandbox.write(&format!("exports/a - Home/{CLEAN_PLAYER}"), &clean_model());
    sandbox.write("exports/a - Home/Kits/p1/kit.dds", &tracer_kit());
}

/// Writes `a - Home` beside two exports of `/co/`: the folder `co - A`, holding a goalkeeper
/// kit, and `co - B.zip`, the tracer's player and goalkeeper kit. Both compiled, the two kits
/// would write one CPK path twice.
fn write_two_exports_of_one_team(sandbox: &Sandbox) {
    write_a_home(sandbox);
    sandbox.write("exports/co - A/Kits/g1/kit.dds", &tracer_kit());
    sandbox.write("exports/co - B.zip", &source_fixture("egg Tracer.zip"));
}

/// The finding each of the two `/co/` exports of `write_two_exports_of_one_team` gets.
const CO_DUPLICATE: &str =
    "Error duplicate_aesthetics_export [DropExport] (id=714, exports=co - A, co - B.zip)";

// TC-PLN-03
#[test]
fn two_exports_of_one_team_are_both_skipped_naming_each_other_and_the_other_team_compiles() {
    let sandbox = Sandbox::new("pln_duplicate_team");
    write_two_exports_of_one_team(&sandbox);
    let validated = [
        "a - Home: Info export_identified [Keep] (team=/a/, id=702)".to_owned(),
        "co - A: Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
        format!("co - A: {CO_DUPLICATE}"),
        "co - B.zip: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)".to_owned(),
        "co - B.zip: Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
        format!("co - B.zip: {CO_DUPLICATE}"),
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(check.messages(), validated);
    assert_eq!(check.exit_code(), 1);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    // Planning's notes are `a - Home`'s alone: nothing of `/co/` is planned.
    let planned = [
        "a - Home: Info team_colors_missing [Keep] ()".to_owned(),
        "a - Home: Info kit_config_generated [Keep] at Kits/p1 ()".to_owned(),
        "a - Home: Info kit_colors_derived [Keep] at Kits/p1 ()".to_owned(),
    ];
    assert_eq!(lines, [&validated[..], &planned[..]].concat());
    assert_eq!(compiled_players(&sandbox), [70203]);
    assert_eq!(compiled_kits(&sandbox), ["u0702p1"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let of_714: Vec<&String> = entries.keys().filter(|path| path.contains("714")).collect();
    assert_eq!(of_714, Vec::<&String>::new());
}

// TC-PLN-07
#[test]
fn a_team_beside_two_skipped_exports_compiles_to_the_cpk_it_compiles_to_alone() {
    let beside = Sandbox::new("pln_skipped_beside");
    write_two_exports_of_one_team(&beside);
    let alone = Sandbox::new("pln_skipped_alone");
    write_a_home(&alone);

    let beside_run = beside.run(&pes21_settings(&beside), &["compile"]);
    let alone_run = alone.run(&pes21_settings(&alone), &["compile"]);

    assert_eq!(beside_run.exit_code(), 1, "the duplicate's errors");
    assert_eq!(alone_run.exit_code(), 0, "{:#?}", alone_run.messages());
    let beside_cpk = fs::read(beside.root.join("output/4cc_99_test.cpk")).unwrap();
    let alone_cpk = fs::read(alone.root.join("output/4cc_99_test.cpk")).unwrap();
    let first_difference = beside_cpk
        .iter()
        .zip(&alone_cpk)
        .position(|(beside, alone)| beside != alone);
    assert_eq!(
        (first_difference, beside_cpk.len()),
        (None, alone_cpk.len()),
        "the first differing offset, and the two sizes"
    );
}

/// Slot 05's boots, the player's own, as `/co/` compiles them.
const BOOTS_05: &str = "Asset/model/character/boots/k0625/#Win/boots.fpk";

/// What `write_overrides` puts at `BOOTS_05`.
const BOOTS_OVERRIDE: &[u8] = b"the operator's boots";

/// What `write_overrides` puts at `TEAM_COLOR`.
const TEAM_COLOR_OVERRIDE: &[u8] = b"the operator's TeamColor.bin";

/// Writes the sandbox's `data/overrides/` folder: a `TeamColor.bin` and slot 05's boots for
/// `/co/`, and returns the folder as `overrides_active` names it.
fn write_overrides(sandbox: &Sandbox) -> String {
    sandbox.write(&format!("data/overrides/{TEAM_COLOR}"), TEAM_COLOR_OVERRIDE);
    sandbox.write(&format!("data/overrides/{BOOTS_05}"), BOOTS_OVERRIDE);
    sandbox.display("data/overrides")
}

// TC-PLN-04
#[test]
fn the_overrides_replace_the_export_s_boots_and_the_team_color_bin_and_are_reported() {
    let sandbox = Sandbox::new("pln_overrides");
    let folder = write_overrides(&sandbox);
    sandbox.write(
        "exports/co - Boots/Players/05 - A/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "co - Boots: Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)".to_owned(),
            "co - Boots: Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
            "co - Boots: Info team_colors_missing [Keep] ()".to_owned(),
            format!("Info overrides_active [Keep] (folder={folder}, files=2)"),
            format!("Warning duplicate_path [Keep] (path={BOOTS_05})"),
            format!("Warning duplicate_path [Keep] (path={TEAM_COLOR})"),
        ]
    );
    assert_eq!(run.exit_code(), 0, "a Warning does not fail the run");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(entries[BOOTS_05], BOOTS_OVERRIDE);
    assert_eq!(entries[TEAM_COLOR], TEAM_COLOR_OVERRIDE);
    // The boots task was not dropped: its other entry is in the CPK.
    assert!(
        entries.contains_key("Asset/model/character/boots/k0625/#Win/boots.fpkd"),
        "{:#?}",
        entries.keys()
    );
}

#[test]
fn the_overrides_are_written_with_the_bins_when_no_export_compiles() {
    let sandbox = Sandbox::new("pln_overrides_no_export");
    let folder = write_overrides(&sandbox);
    fs::create_dir_all(sandbox.root.join("exports")).unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            format!(
                "Warning no_exports_found [Keep] (folder={})",
                sandbox.display("exports")
            ),
            format!("Info overrides_active [Keep] (folder={folder}, files=2)"),
            format!("Warning duplicate_path [Keep] (path={TEAM_COLOR})"),
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            BOOTS_05,
            "common/character0/model/character/uniform/team/UniColor.bin",
            TEAM_COLOR,
        ]
    );
    assert_eq!(entries[BOOTS_05], BOOTS_OVERRIDE);
    assert_eq!(entries[TEAM_COLOR], TEAM_COLOR_OVERRIDE);
}
