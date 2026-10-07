//! `compile`'s working bins taken from the installed CPKs: the walk of the PES folder's
//! `download/DpFileList.bin` (`team_compiler/pipeline.md` "Bins accumulation").

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};
use pes_version::PesVersion;
use uniparam::UniformParameter;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_kit, tracer_player_file};
use crate::compile_exports::{
    TEAM_COLOR, UNI_COLOR, UNIFORM_PARAMETER, bundled_team_color, bundled_uni_color,
    bundled_uniform_parameter, config_path, record_of, uni_record, with_uni_record,
};
use crate::models::body_skl;
use crate::{BUNDLED_BINS, clean_model};

/// Team `/co/`, whose `UniColor.bin` record the tests set.
const CO: usize = 714;

/// An installed `UniColor.bin` entry for team 714's kit 0 (p1).
const A: [u8; 8] = [0x00, 0x03, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

/// Another installed entry for team 714's kit 0, in a CPK loaded above `A`'s.
const B: [u8; 8] = [0x00, 0x03, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

/// The entry `p2_export`'s kit compiles to: kit 1, icon 3, the two colors of its `colors.txt`.
const P2: [u8; 8] = [0x01, 0x03, 0x0a, 0x1b, 0x2c, 0xd4, 0xe5, 0xf6];

/// The export every test compiles: a Midcup `/co/` export with only `p2/`, its kit texture and
/// a `colors.txt` giving two colors, and no root `colors.txt`.
fn p2_export(sandbox: &Sandbox) {
    sandbox.write("exports/co Midcup Kits/Kits/p2/kit.dds", &tracer_kit());
    sandbox.write(
        "exports/co Midcup Kits/Kits/p2/colors.txt",
        b"#0a1b2c\n#d4e5f6\n",
    );
}

/// What `p2_export` reports, after the walk's findings.
const P2_FINDINGS: [&str; 3] = [
    "co Midcup Kits: Info export_identified [Keep] (team=/co/, id=714)",
    "co Midcup Kits: Info team_colors_missing [Keep] ()",
    "co Midcup Kits: Info kit_config_generated [Keep] at Kits/p2 ()",
];

/// Writes `PES/download/DpFileList.bin` listing `4cc_08_bins.cpk`, `4cc_61_midcup.cpk` and
/// `4cc_99_test.cpk`, in that load order (the `walk.bin` fixture).
fn install_list(sandbox: &Sandbox) {
    let list = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dpfl/walk.bin"))
        .unwrap();
    sandbox.write("PES/download/DpFileList.bin", &list);
}

/// Writes `PES/download/DpFileList.bin` listing `names`, in that load order: a 16-byte header
/// holding the count, then one 48-byte record per CPK, its name NUL-padded.
pub(crate) fn install_names(sandbox: &Sandbox, names: &[&str]) {
    let count = u32::try_from(names.len()).unwrap();
    let mut list = vec![0; 4];
    list.extend(count.to_le_bytes());
    list.extend([0; 8]);
    for name in names {
        let mut record = [0; 48];
        record[..name.len()].copy_from_slice(name.as_bytes());
        list.extend(record);
    }
    sandbox.write("PES/download/DpFileList.bin", &list);
}

/// Writes `PES/download/<name>` as a CPK holding `entries` (CPK path, bytes).
pub(crate) fn install_cpk(sandbox: &Sandbox, name: &str, entries: &[(&str, &[u8])]) {
    let path = sandbox.root.join("PES/download").join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut cpk = cpk::CpkWriter::new(fs::File::create(path).unwrap(), "test").unwrap();
    for (entry, bytes) in entries {
        cpk.add(entry, bytes, None).unwrap();
    }
    cpk.finish().unwrap();
}

/// The bundled `UniColor.bin` with team 714's kit-0 entry (its first) replaced by `kit_0`.
fn uni_color_with_kit_0(kit_0: [u8; 8]) -> Vec<u8> {
    let base = bundled_uni_color();
    let mut record = uni_record(&base, CO).to_vec();
    assert_eq!(record[5], 0x00, "the base's first entry is kit 0");
    record[5..13].copy_from_slice(&kit_0);
    with_uni_record(&base, CO, &record)
}

/// `installed` with team 714's kit-1 entry (its second) set to `P2`: what a compile of
/// `p2_export` builds on it.
fn with_p2(installed: &[u8]) -> Vec<u8> {
    let mut record = uni_record(installed, CO).to_vec();
    assert_eq!(record[13], 0x01, "the record's second entry is kit 1");
    record[13..21].copy_from_slice(&P2);
    with_uni_record(installed, CO, &record)
}

/// The `bin_source` line naming `bin` and `cpk`.
fn source(bin: &str, cpk: &str) -> String {
    format!("Info bin_source [Keep] (bin={bin}, cpk={cpk})")
}

/// TC-BIN-05's install: `4cc_08_bins.cpk` holding the bundled `TeamColor.bin` and a
/// `UniColor.bin` whose team 714 kit 0 is `A`, `4cc_61_midcup.cpk` one where it is `B`.
fn install_two_cpks(sandbox: &Sandbox) {
    install_list(sandbox);
    install_cpk(
        sandbox,
        "4cc_08_bins.cpk",
        &[
            (TEAM_COLOR, &bundled_team_color()),
            (UNI_COLOR, &uni_color_with_kit_0(A)),
        ],
    );
    install_cpk(
        sandbox,
        "4cc_61_midcup.cpk",
        &[(UNI_COLOR, &uni_color_with_kit_0(B))],
    );
}

// TC-BIN-05
#[test]
fn each_bin_comes_from_the_nearest_cpk_listed_below_the_run_s_own() {
    let sandbox = Sandbox::new("bins_walk");
    install_two_cpks(&sandbox);
    p2_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let mut expected = vec![
        source("TeamColor.bin", "4cc_08_bins.cpk"),
        source("UniColor.bin", "4cc_61_midcup.cpk"),
        source("UniformParameter.bin", "bundled"),
    ];
    expected.extend(P2_FINDINGS.map(str::to_owned));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(
        entries[UNI_COLOR] == with_p2(&uni_color_with_kit_0(B)),
        "UniColor.bin is 4cc_61_midcup's with p2's entry set"
    );
    assert!(entries[TEAM_COLOR] == bundled_team_color());

    // The midcup CPK is the run's own: it and everything above it are skipped.
    let settings = format!(
        "{}[team-compiler]\ncpk_name = \"4cc_61_midcup\"\n",
        pes21_settings(&sandbox)
    );
    let run = sandbox.run(&settings, &["compile"]);

    let mut expected = vec![
        source("TeamColor.bin", "4cc_08_bins.cpk"),
        source("UniColor.bin", "4cc_08_bins.cpk"),
        source("UniformParameter.bin", "bundled"),
    ];
    expected.extend(P2_FINDINGS.map(str::to_owned));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_61_midcup.cpk"));
    assert!(
        entries[UNI_COLOR] == with_p2(&uni_color_with_kit_0(A)),
        "UniColor.bin is 4cc_08_bins' with p2's entry set"
    );
}

#[test]
fn the_walk_reads_the_pes_folder_with_its_version_placeholder_expanded() {
    let sandbox = Sandbox::new("bins_placeholder");
    install_list(&sandbox);
    install_cpk(
        &sandbox,
        "4cc_08_bins.cpk",
        &[(UNI_COLOR, &uni_color_with_kit_0(A))],
    );
    fs::rename(sandbox.root.join("PES"), sandbox.root.join("PES2021")).unwrap();
    p2_export(&sandbox);
    let settings = format!(
        "[common]\npes_version = 21\npes_folder_path = '{}'\n",
        sandbox.root.join("PES20**").display()
    );

    let run = sandbox.run(&settings, &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        run.messages()[..3],
        [
            source("TeamColor.bin", "bundled"),
            source("UniColor.bin", "4cc_08_bins.cpk"),
            source("UniformParameter.bin", "bundled"),
        ]
    );
}

// TC-BIN-08
#[test]
fn a_wesys_wrapped_installed_bin_is_read_unwrapped_and_written_plain() {
    let sandbox = Sandbox::new("bins_wrapped");
    install_list(&sandbox);
    let wrapped = wezlib::compress(&uni_color_with_kit_0(A));
    install_cpk(&sandbox, "4cc_08_bins.cpk", &[(UNI_COLOR, &wrapped)]);
    p2_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        run.messages()[..3],
        [
            source("TeamColor.bin", "bundled"),
            source("UniColor.bin", "4cc_08_bins.cpk"),
            source("UniformParameter.bin", "bundled"),
        ]
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(!wezlib::is_wrapped(&entries[UNI_COLOR]), "written plain");
    assert_eq!(uni_record(&entries[UNI_COLOR], CO)[5..13], A);
    assert!(entries[UNI_COLOR] == with_p2(&uni_color_with_kit_0(A)));
}

// TC-BIN-09
#[test]
fn with_no_dpfilelist_the_bins_are_bundled_and_the_list_s_absence_reported() {
    let alone = Sandbox::new("bins_no_pes");
    p2_export(&alone);
    let run = alone.run(&pes21_settings(&alone), &["compile"]);
    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let bundled = cpk_entries(&alone.root.join("output/4cc_99_test.cpk"));

    let sandbox = Sandbox::new("bins_no_list");
    fs::create_dir_all(sandbox.root.join("PES/download")).unwrap();
    p2_export(&sandbox);
    let list = sandbox.display("PES/download/DpFileList.bin");
    let bundled_sources = [
        source("TeamColor.bin", "bundled"),
        source("UniColor.bin", "bundled"),
        source("UniformParameter.bin", "bundled"),
    ];

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let mut expected = vec![format!("Error dpfilelist_missing [Keep] (path={list})")];
    expected.extend(bundled_sources.clone());
    expected.extend(P2_FINDINGS.map(str::to_owned));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 1);
    let cpk = sandbox.root.join("output/4cc_99_test.cpk");
    let entries = cpk_entries(&cpk);
    assert!(entries[UNI_COLOR] == bundled[UNI_COLOR]);
    assert!(entries[TEAM_COLOR] == bundled[TEAM_COLOR]);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let mut expected = vec![format!("Warning dpfilelist_missing [Keep] (path={list})")];
    expected.extend(bundled_sources);
    expected.extend(P2_FINDINGS.map(str::to_owned));
    expected.push(format!(
        "Info deploy_skipped_by_flag [Keep] (path={})",
        sandbox.display("output/4cc_99_test.cpk")
    ));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 0);
}

// TC-BIN-13
#[test]
fn an_installed_team_color_record_whose_header_holds_colors_gets_its_header_back() {
    let sandbox = Sandbox::new("bins_header_repaired");
    install_list(&sandbox);
    let mut installed = bundled_team_color();
    let record = (799 - 100) * 16;
    installed[record..record + 4].copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
    install_cpk(&sandbox, "4cc_08_bins.cpk", &[(TEAM_COLOR, &installed)]);
    p2_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let lines = run.messages();
    assert_eq!(lines[0], source("TeamColor.bin", "4cc_08_bins.cpk"));
    let repaired: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("bin_header_repaired"))
        .collect();
    assert_eq!(
        repaired,
        ["Warning bin_header_repaired [Keep] (bin=TeamColor.bin, teams=799)"]
    );
    let emitted = &cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"))[TEAM_COLOR];
    // Team ID 799 and count 4, little-endian, then the installed record's other bytes.
    assert_eq!(emitted[record..record + 4], [0x1f, 0x03, 0x04, 0x00]);
    assert_eq!(
        emitted[record + 4..record + 16],
        installed[record + 4..record + 16]
    );
    assert_eq!(emitted[..record], installed[..record]);
    assert_eq!(emitted[record + 16..], installed[record + 16..]);
}

// TC-BIN-21
#[test]
fn an_installed_cpk_that_cannot_be_read_stops_the_run_before_any_export_is_read() {
    let sandbox = Sandbox::new("bins_unreadable");
    install_two_cpks(&sandbox);
    sandbox.write("PES/download/4cc_61_midcup.cpk", b"not a cpk");
    p2_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let [line] = lines.as_slice() else {
        panic!("one finding, on no export: {lines:#?}");
    };
    let prefix = format!(
        "Fatal installed_bin_unreadable [AbortRun] (path={}, error=",
        sandbox.display("PES/download/4cc_61_midcup.cpk")
    );
    assert!(line.starts_with(&prefix), "{line}");
    assert_eq!(run.exit_code(), 3);
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());
}

#[test]
fn an_installed_bin_that_does_not_parse_stops_the_run_before_any_export_is_read() {
    let sandbox = Sandbox::new("bins_unparsable");
    install_two_cpks(&sandbox);
    // One byte short of a whole 85-byte record.
    install_cpk(&sandbox, "4cc_61_midcup.cpk", &[(UNI_COLOR, &[0; 84])]);
    p2_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [format!(
            "Fatal installed_bin_unreadable [AbortRun] (path={}, error=cannot parse {UNI_COLOR}: \
             UniColor.bin is 84 bytes, not a whole number of 85-byte records)",
            sandbox.display("PES/download/4cc_61_midcup.cpk")
        )],
        "one finding, on no export"
    );
    assert_eq!(run.exit_code(), 3);
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());
}

/// Team `team_id`'s kit config of `slot` holding shirt model 144, which lacks the FPC values,
/// with `fpc` the FPC values set, encoded for PES 21 with the slot's main texture name.
fn shirt_144_config(team_id: u16, slot: KitSlot, fpc: bool) -> Vec<u8> {
    let mut config = KitConfig::template();
    config.shirt.model = 144;
    assert!(
        !kit_config::matches_fpc(&config),
        "shirt model 144 is no FPC shirt"
    );
    if fpc {
        kit_config::apply_fpc(&mut config);
    }
    let names = texture_names(
        team_id,
        slot,
        TexturePresence {
            kit: true,
            ..TexturePresence::default()
        },
    );
    config.encode_with_names(PesVersion::Pes21, &names).to_vec()
}

/// A `UniformParameter.bin` holding `configs` (team ID, slot), each `shirt_144_config`.
fn installed_configs(configs: &[(u16, KitSlot)]) -> Vec<u8> {
    let mut bin = UniformParameter::new();
    for (team_id, slot) in configs {
        bin.insert(
            slot.config_name(*team_id),
            shirt_144_config(*team_id, *slot, false),
        )
        .unwrap();
    }
    bin.write()
}

/// The emitted `UniformParameter.bin`'s entries whose name starts with `prefix`, by name.
fn configs_of(entries: &BTreeMap<String, Vec<u8>>, prefix: &str) -> Vec<(String, Vec<u8>)> {
    UniformParameter::read(&entries[UNIFORM_PARAMETER])
        .unwrap()
        .entries()
        .filter(|(name, _)| name.starts_with(prefix))
        .map(|(name, bytes)| (name.to_owned(), bytes.to_vec()))
        .collect()
}

// TC-BIN-06
#[test]
fn a_midcup_fpc_on_export_patches_the_installed_configs_of_the_kits_it_does_not_hold() {
    let sandbox = Sandbox::new("bins_fpc_absent_slots");
    install_list(&sandbox);
    // Team 714's record holds kits 0, 1 and 2: p1, p2 and p3.
    let record = record_of(
        714,
        3,
        &[
            A,
            [0x01, 0x03, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            [0x02, 0x03, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c],
        ],
    );
    let uni_color = with_uni_record(&bundled_uni_color(), CO, &record);
    // `UniformParameter::new()` plus p1's config: team 714 has no p3 config.
    let installed = installed_configs(&[(714, KitSlot::P1)]);
    install_cpk(
        &sandbox,
        "4cc_08_bins.cpk",
        &[
            (TEAM_COLOR, &bundled_team_color()),
            (UNI_COLOR, &uni_color),
            (UNIFORM_PARAMETER, &installed),
        ],
    );
    p2_export(&sandbox);
    sandbox.write(
        "exports/co Midcup Kits/Players/05 - A/face_high.fmdl",
        &clean_model(),
    );
    sandbox.write("exports/co Midcup Kits/Players/05 - A/fpc_on", b"");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let mut expected = vec![
        source("TeamColor.bin", "4cc_08_bins.cpk"),
        source("UniColor.bin", "4cc_08_bins.cpk"),
        source("UniformParameter.bin", "4cc_08_bins.cpk"),
    ];
    expected.extend(P2_FINDINGS.map(str::to_owned));
    // p2 is the export's: nothing for it.
    expected.extend([
        "co Midcup Kits: Info kit_config_fpc_adjusted [Keep] (slot=p1)".to_owned(),
        "co Midcup Kits: Warning kit_config_fpc_unpatched [Keep] (slot=p3)".to_owned(),
    ]);
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let configs = configs_of(&entries, "714_");
    let names: Vec<&str> = configs.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        ["714_DEF_1st_realUni.bin", "714_DEF_2nd_realUni.bin"],
        "no 714_DEF_3rd"
    );
    assert!(
        configs[0].1 == shirt_144_config(714, KitSlot::P1, true),
        "p1 is the installed config with the FPC values"
    );
    assert!(
        configs[1].1 == entries[&config_path("2nd")],
        "p2 is the compiled kit's config"
    );
}

// TC-BIN-16
#[test]
fn a_full_export_removes_its_team_s_installed_configs_of_kits_it_does_not_hold() {
    let sandbox = Sandbox::new("bins_full_configs");
    install_list(&sandbox);
    let installed = installed_configs(&[
        (714, KitSlot::P1),
        (714, KitSlot::P2),
        (714, KitSlot::P3),
        (702, KitSlot::P1),
    ]);
    install_cpk(
        &sandbox,
        "4cc_08_bins.cpk",
        &[
            (TEAM_COLOR, &bundled_team_color()),
            (UNI_COLOR, &bundled_uni_color()),
            (UNIFORM_PARAMETER, &installed),
        ],
    );
    sandbox.write("exports/co Full Kits/Kits/p1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let configs = configs_of(&entries, "714_");
    let names: Vec<&str> = configs.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        ["714_DEF_1st_realUni.bin", "714_DEF_GK1st_realUni.bin"],
        "p2 and p3 removed, the placeholder g1 added"
    );
    assert!(configs[0].1 == entries[&config_path("1st")]);
    assert!(configs[1].1 == entries[&config_path("GK1st")]);
    assert_eq!(
        configs_of(&entries, "702_"),
        [(
            "702_DEF_1st_realUni.bin".to_owned(),
            shirt_144_config(702, KitSlot::P1, false)
        )],
        "team 702's config is the installed one"
    );
}

// TC-BIN-07
#[test]
fn a_templates_file_replaces_the_bundled_base_and_one_that_cannot_be_read_stops_the_run() {
    let sandbox = Sandbox::new("bins_templates");
    // The bundled base with one color byte of team 701's first kit entry changed, so a CPK
    // built on the template is told from one built on the base.
    let base = bundled_uni_color();
    let mut record = uni_record(&base, 701).to_vec();
    record[7] = record[7].wrapping_add(1);
    let template = with_uni_record(&base, 701, &record);
    sandbox.write("data/templates/UniColor.bin", &template);
    p2_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let mut expected = vec![format!(
        "Info template_override_active [Keep] (path={})",
        sandbox.display("data/templates/UniColor.bin")
    )];
    expected.extend(BUNDLED_BINS.map(str::to_owned));
    expected.extend(P2_FINDINGS.map(str::to_owned));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 0);
    let cpk = sandbox.root.join("output/4cc_99_test.cpk");
    let emitted = &cpk_entries(&cpk)[UNI_COLOR];
    assert!(
        *emitted == with_p2(&template),
        "UniColor.bin is the template with p2's entry set"
    );
    assert_eq!(uni_record(emitted, 701), record, "team 701's changed byte");

    // A folder where a template's file goes cannot be read as one.
    fs::remove_file(&cpk).unwrap();
    fs::create_dir_all(sandbox.root.join("data/templates/TeamColor.bin")).unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let [line] = lines.as_slice() else {
        panic!("one finding, on no export: {lines:#?}");
    };
    let prefix = format!(
        "Fatal template_override_unreadable [AbortRun] (path={}, error=",
        sandbox.display("data/templates/TeamColor.bin")
    );
    assert!(line.starts_with(&prefix), "{line}");
    assert_eq!(run.exit_code(), 3);
    assert!(!cpk.exists());
}

/// The boots list's path in a CPK.
const BOOTS_LIST: &str = "common/character0/model/character/boots/BootsList.bin";

/// The gloves list's path in a CPK.
const GLOVE_LIST: &str = "common/character0/model/character/glove/GloveList.bin";

/// The player appearance table's path in a CPK.
const PLAYER_APPEARANCE: &str = "common/character0/model/character/appearance/PlayerAppearance.bin";

/// A `BootsList.bin` or `GloveList.bin` holding `pairs` (player id, item id) as they are
/// given: two little-endian `u32` each.
fn item_list(pairs: &[(u32, u32)]) -> Vec<u8> {
    pairs
        .iter()
        .flat_map(|(player_id, item_id)| {
            player_id
                .to_le_bytes()
                .into_iter()
                .chain(item_id.to_le_bytes())
        })
        .collect()
}

/// Ten installed boots rows, sorted by player id, for players of `/a/` (702) and `/b/` (707):
/// none for a team 714 player.
const INSTALLED_BOOTS: [(u32, u32); 10] = [
    (70201, 11),
    (70202, 12),
    (70203, 13),
    (70204, 14),
    (70205, 15),
    (70701, 21),
    (70702, 22),
    (70703, 23),
    (70704, 24),
    (70705, 25),
];

/// Installed gloves rows for players of `/a/` and `/b/`.
const INSTALLED_GLOVES: [(u32, u32); 2] = [(70201, 31), (70701, 32)];

/// A `PlayerAppearance.bin` of two 60-byte rows, players 70201 and 70701, each row's 56
/// appearance bytes told apart by their value.
fn player_appearance() -> Vec<u8> {
    let mut bytes = Vec::new();
    for (player_id, fill) in [(70201u32, 0x0a), (70701, 0x0b)] {
        bytes.extend(player_id.to_le_bytes());
        bytes.extend([fill; 56]);
    }
    bytes
}

/// Installs `4cc_08_bins.cpk`, listed below the run's CPK, holding the bundled color and kit
/// config bins and `tables` (CPK path, bytes).
fn install_tables(sandbox: &Sandbox, tables: &[(&str, &[u8])]) {
    install_list(sandbox);
    let team_color = bundled_team_color();
    let uni_color = bundled_uni_color();
    let uniform_parameter = bundled_uniform_parameter();
    let mut entries: Vec<(&str, &[u8])> = vec![
        (TEAM_COLOR, &team_color),
        (UNI_COLOR, &uni_color),
        (UNIFORM_PARAMETER, &uniform_parameter),
    ];
    entries.extend_from_slice(tables);
    install_cpk(sandbox, "4cc_08_bins.cpk", &entries);
}

/// The CPK's `BootsList.bin` or `GloveList.bin` at `path` as its (player id, item id) pairs, in
/// file order.
fn pairs_of(entries: &BTreeMap<String, Vec<u8>>, path: &str) -> Vec<(u32, u32)> {
    let (pairs, rest) = entries[path].as_chunks::<8>();
    assert!(rest.is_empty(), "{path} is whole pairs");
    pairs
        .iter()
        .map(|&[p0, p1, p2, p3, i0, i1, i2, i3]| {
            (
                u32::from_le_bytes([p0, p1, p2, p3]),
                u32::from_le_bytes([i0, i1, i2, i3]),
            )
        })
        .collect()
}

// TC-BIN-10
#[test]
fn a_compiled_player_s_boots_row_joins_the_installed_list_and_the_other_tables_pass_through() {
    let sandbox = Sandbox::new("bins_player_tables");
    let gloves = item_list(&INSTALLED_GLOVES);
    let appearance = player_appearance();
    install_tables(
        &sandbox,
        &[
            (BOOTS_LIST, &item_list(&INSTALLED_BOOTS)),
            (GLOVE_LIST, &gloves),
            (PLAYER_APPEARANCE, &appearance),
        ],
    );
    sandbox.write(
        "exports/co Midcup Boots/Players/05 - A/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let lines = run.messages();
    assert_eq!(
        lines[..6],
        [
            source("TeamColor.bin", "4cc_08_bins.cpk"),
            source("UniColor.bin", "4cc_08_bins.cpk"),
            source("UniformParameter.bin", "4cc_08_bins.cpk"),
            source("BootsList.bin", "4cc_08_bins.cpk"),
            source("GloveList.bin", "4cc_08_bins.cpk"),
            source("PlayerAppearance.bin", "4cc_08_bins.cpk"),
        ]
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("player_table_missing")),
        "{lines:#?}"
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(entries.contains_key("Asset/model/character/boots/k0625/#Win/boots.fpk"));
    let mut expected = INSTALLED_BOOTS.to_vec();
    expected.push((71405, 625));
    assert_eq!(
        pairs_of(&entries, BOOTS_LIST),
        expected,
        "eleven pairs sorted by player id"
    );
    assert!(entries[GLOVE_LIST] == gloves, "GloveList.bin unchanged");
    assert!(
        entries[PLAYER_APPEARANCE] == appearance,
        "PlayerAppearance.bin unchanged"
    );
}

// TC-BIN-11
#[test]
fn a_player_whose_boots_task_fails_keeps_his_installed_boots_row() {
    let sandbox = Sandbox::new("bins_boots_failed");
    let installed = item_list(&[(70201, 11), (71405, 7)]);
    install_tables(&sandbox, &[(BOOTS_LIST, &installed)]);
    let folder = "exports/co Midcup Boots/Players/05 - A";
    // Two boots parts, one paired with a skeleton and one without: `skl_merge_conflict`
    // fails the boots task, and the folder's blank face still commits.
    for name in ["boots.fmdl", "kit_boots.fmdl"] {
        sandbox.write(
            &format!("{folder}/{name}"),
            &tracer_player_file("boots.fmdl"),
        );
    }
    sandbox.write(&format!("{folder}/boots.skl"), &body_skl("pes21"));
    sandbox.write(
        &format!("{folder}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert!(
        lines.iter().any(|line| line
            == "co Midcup Boots: Error skl_merge_conflict [DropFolder] at Players/05 - A (skeleton=differs)"),
        "{lines:#?}"
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(!entries.contains_key("Asset/model/character/boots/k0625/#Win/boots.fpk"));
    assert!(
        entries[BOOTS_LIST] == installed,
        "BootsList.bin keeps (71405, 7)"
    );
}

// TC-BIN-12
#[test]
fn a_player_s_own_gloves_and_a_linked_shared_folder_s_set_their_gloves_rows() {
    let sandbox = Sandbox::new("bins_gloves_rows");
    install_tables(&sandbox, &[(GLOVE_LIST, &item_list(&INSTALLED_GLOVES))]);
    let export = "exports/co Midcup Gloves";
    for name in ["glove_l.fmdl", "glove_r.fmdl"] {
        sandbox.write(
            &format!("{export}/Players/05 - A/{name}"),
            &tracer_player_file(name),
        );
        sandbox.write(
            &format!("{export}/Gloves/Keeper/{name}"),
            &tracer_player_file(name),
        );
    }
    sandbox.write(&format!("{export}/Players/07 - B/Keeper.gloves"), b"");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(entries.contains_key("Asset/model/character/glove/g0625/#Win/glove.fpk"));
    assert!(entries.contains_key("Asset/model/character/glove/g0644/#Win/glove.fpk"));
    assert_eq!(
        pairs_of(&entries, GLOVE_LIST),
        [(70201, 31), (70701, 32), (71405, 625), (71407, 644)]
    );
}

// TC-BIN-17
#[test]
fn a_full_export_removes_the_boots_row_of_a_player_with_no_boots_and_a_midcup_keeps_it() {
    for (coverage, expected) in [
        ("Full", vec![(70201, 11), (71405, 625)]),
        ("Midcup", vec![(70201, 11), (71405, 625), (71406, 9)]),
    ] {
        let sandbox = Sandbox::new(&format!("bins_boots_rows_{coverage}"));
        install_tables(
            &sandbox,
            &[(
                BOOTS_LIST,
                &item_list(&[(70201, 11), (71405, 7), (71406, 9)]),
            )],
        );
        let export = format!("exports/co {coverage} Boots");
        sandbox.write(
            &format!("{export}/Players/05 - A/boots.fmdl"),
            &tracer_player_file("boots.fmdl"),
        );
        sandbox.write(
            &format!("{export}/Players/06 - B/face_high.fmdl"),
            &clean_model(),
        );

        let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

        assert_eq!(run.exit_code(), 0, "{coverage}: {:#?}", run.messages());
        let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
        assert_eq!(pairs_of(&entries, BOOTS_LIST), expected, "{coverage}");
    }
}

// TC-BIN-22
#[test]
fn with_no_installed_list_the_boots_row_is_left_out_and_reported() {
    let sandbox = Sandbox::new("bins_no_tables");
    sandbox.write(
        "exports/co Midcup Boots/Players/05 - A/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let lines = run.messages();
    let missing: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("player_table_missing"))
        .collect();
    assert_eq!(
        missing,
        ["Warning player_table_missing [Keep] (table=BootsList.bin, rows=1)"]
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(entries.contains_key("Asset/model/character/boots/k0625/#Win/boots.fpk"));
    for table in [BOOTS_LIST, GLOVE_LIST, PLAYER_APPEARANCE] {
        assert!(!entries.contains_key(table), "no {table}");
    }
}
