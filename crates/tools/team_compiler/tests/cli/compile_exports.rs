//! `compile` over each export's content: the roster, the kits, a nested root, the root
//! files, the source kinds, and the run across several exports.

use std::fs;
use std::path::Path;

use dds_convert::{BlockCodec, SourceFormat, decode};
use studio_core::PipelineEvent;

use crate::common::Sandbox;
use crate::compile::{
    compiled_kits, compiled_players, compiled_portraits, cpk_entries, pass_through_settings,
    pes21_settings, tracer_kit, tracer_portrait,
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
            "co - Colors: Info kit_config_generated [Keep] at Kits/p1 ()"
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
            "dbg - Plain: Info kit_config_generated [Keep] at Kits/p1 ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    assert!(
        cpk_entries(&cpk).get(TEAM_COLOR) == Some(&base),
        "TeamColor.bin is the bundled base"
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
            "co - Spring 2026.zip: Info team_colors_missing [Keep] ()"
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
            "egg Tracer bad notes.zip: Info team_colors_missing [Keep] ()"
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
            "egg Tracer.7z: Info team_colors_missing [Keep] ()"
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
            "dbg Seven.7z: Info team_colors_missing [Keep] ()"
        ]
    );
    assert!(outcomes[0].1 == outcomes[1].1, "the CPKs differ");
}
