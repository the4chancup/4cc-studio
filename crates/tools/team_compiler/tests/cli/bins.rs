//! `compile`'s working bins taken from the installed CPKs: the walk of the PES folder's
//! `download/DpFileList.bin` (`team_compiler/pipeline.md` "Bins accumulation").

use std::fs;
use std::path::Path;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_kit};
use crate::compile_exports::{
    TEAM_COLOR, UNI_COLOR, bundled_team_color, bundled_uni_color, uni_record, with_uni_record,
};

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

/// Writes `PES/download/<name>` as a CPK holding `entries` (CPK path, bytes).
fn install_cpk(sandbox: &Sandbox, name: &str, entries: &[(&str, &[u8])]) {
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
