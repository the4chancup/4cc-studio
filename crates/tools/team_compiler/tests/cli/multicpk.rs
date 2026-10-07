//! Multi-CPK mode (`team_compiler/pipeline.md` "5. Writer", step 6 "Multi-CPK mode: teams
//! parts"): whole teams filled first-fit into the official list's `teams` slots under
//! `cpk_part_max_size`, the placeholder in every slot left over, the overrides and the bins in
//! the bins CPK; and the single CPK's warning past the same cap.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use crate::bins::dpfl_bytes;
use crate::common::{Run, Sandbox};
use crate::compile::{cpk_entries, kit_texture, pes21_settings, tracer_export};
use crate::compile_exports::{TEAM_COLOR, UNI_COLOR, UNIFORM_PARAMETER};
use crate::deploy::templates_folder;
use crate::snapshot;

/// The tracer's copies, one per team: /a/ 702, /b/ 707 and /co/ 714, in canonical order.
const EXPORTS: [&str; 3] = ["a Midcup Tracer", "b Midcup Tracer", "co Midcup Tracer"];

/// The teams slots of the official list.
const TEAMS_SLOTS: [&str; 5] = [
    "4cc_41_teams.cpk",
    "4cc_42_teams.cpk",
    "4cc_43_teams.cpk",
    "4cc_44_teams.cpk",
    "4cc_45_teams.cpk",
];

/// A sandbox holding the tracer as each export of `exports`.
fn sandbox_with(name: &str, exports: &[&str]) -> Sandbox {
    let sandbox = Sandbox::new(name);
    for export in exports {
        sandbox.copy_tracer(export);
    }
    sandbox
}

/// PES 21 settings with `multicpk_mode` on and the `[team-compiler]` lines `extra`.
fn multicpk_settings(sandbox: &Sandbox, extra: &str) -> String {
    format!(
        "{}[team-compiler]\nmulticpk_mode = true\n{extra}",
        pes21_settings(sandbox)
    )
}

/// `compile --no-deploy` in `sandbox` with `settings`.
fn compile(sandbox: &Sandbox, settings: &str) -> Run {
    sandbox.run(settings, &["compile", "--no-deploy"])
}

/// The names of the files directly in the sandbox's `output/`.
fn output_files(sandbox: &Sandbox) -> BTreeSet<String> {
    fs::read_dir(sandbox.root.join("output"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect()
}

/// The shipped placeholder CPK's bytes.
fn placeholder() -> Vec<u8> {
    fs::read(templates_folder().join("placeholder.cpk")).unwrap()
}

/// The line `deploy_skipped_by_flag` reports for `output/<name>`.
fn skipped(sandbox: &Sandbox, name: &str) -> String {
    format!(
        "Info deploy_skipped_by_flag [Keep] (path={})",
        sandbox.display(&format!("output/{name}"))
    )
}

/// `cpk_part_max_size` as the findings show it: `N bytes` under 1 KiB, else in the largest of
/// KiB, MiB and GiB it reaches, to one decimal, rounded to the nearest, a trailing `.0` left
/// out.
fn size_text(bytes: u64) -> String {
    let units = [("GiB", 1u64 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];
    let Some((unit, size)) = units.into_iter().find(|(_, size)| bytes >= *size) else {
        return format!("{bytes} bytes");
    };
    let tenths = (bytes * 10 + size / 2) / size;
    match tenths % 10 {
        0 => format!("{} {unit}", tenths / 10),
        digit => format!("{}.{digit} {unit}", tenths / 10),
    }
}

/// The length of the first teams part, and its entries, when /a/ and /b/ alone are compiled
/// at the default cap in the sandbox `name`: a cap at which they fill one part exactly.
fn first_part_of_a_and_b(name: &str) -> (u64, BTreeMap<String, Vec<u8>>) {
    let sandbox = sandbox_with(name, &EXPORTS[..2]);
    let run = compile(&sandbox, &multicpk_settings(&sandbox, ""));
    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let part = sandbox.root.join("output/4cc_41_teams.cpk");
    (fs::metadata(&part).unwrap().len(), cpk_entries(&part))
}

// TC-OUT-12
#[test]
fn teams_fill_the_slots_first_fit_and_the_bins_go_into_the_bins_cpk() {
    let (cap, a_and_b) = first_part_of_a_and_b("multicpk_parts_a_and_b");
    let sandbox = sandbox_with("multicpk_parts", &EXPORTS);
    let settings = multicpk_settings(&sandbox, &format!("cpk_part_max_size = {cap}\n"));

    let run = compile(&sandbox, &settings);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let output = sandbox.root.join("output");
    let mut expected_files: BTreeSet<String> = TEAMS_SLOTS.map(str::to_owned).into();
    expected_files.insert("4cc_08_bins.cpk".to_owned());
    assert_eq!(
        output_files(&sandbox),
        expected_files,
        "no 99_test, no teams2"
    );
    assert!(
        cpk_entries(&output.join("4cc_41_teams.cpk")) == a_and_b,
        "the first part holds exactly /a/ and /b/"
    );
    let second = cpk_entries(&output.join("4cc_42_teams.cpk"));
    assert!(second.contains_key(&kit_texture("u0714g1")), "/co/'s kit");
    for team in ["u0702g1", "u0707g1"] {
        assert!(!second.contains_key(&kit_texture(team)), "{team}");
    }
    for slot in &TEAMS_SLOTS[2..] {
        assert!(
            fs::read(output.join(slot)).unwrap() == placeholder(),
            "{slot} is the placeholder"
        );
    }
    let bins: Vec<String> = cpk_entries(&output.join("4cc_08_bins.cpk"))
        .into_keys()
        .collect();
    let mut expected_bins = vec![
        TEAM_COLOR.to_owned(),
        UNI_COLOR.to_owned(),
        UNIFORM_PARAMETER.to_owned(),
    ];
    expected_bins.sort();
    assert_eq!(bins, expected_bins);
    let lines = run.messages();
    let mut expected_end = vec![skipped(&sandbox, "4cc_08_bins.cpk")];
    expected_end.extend(TEAMS_SLOTS.map(|slot| skipped(&sandbox, slot)));
    assert_eq!(lines[lines.len() - 6..], expected_end);
    assert!(!output.join(".staging").exists());
}

/// Asserts `run` in `sandbox` was aborted with the Fatal `line` last, and wrote no CPK.
fn assert_aborted_with(sandbox: &Sandbox, run: &Run, line: impl Fn(&str) -> bool) {
    assert_eq!(run.exit_code(), 3);
    let lines = run.messages();
    let last = lines.last().unwrap();
    assert!(line(last), "{lines:#?}");
    let written: Vec<_> = snapshot(&sandbox.root.join("output"))
        .into_keys()
        .filter(|path| path.extension().is_some_and(|extension| extension == "cpk"))
        .collect();
    assert_eq!(
        written,
        Vec::<std::path::PathBuf>::new(),
        "no part is written"
    );
    assert!(!sandbox.root.join("output/.staging").exists());
}

// TC-OUT-13
#[test]
fn a_team_larger_than_the_cap_aborts_the_run_and_writes_no_part() {
    let sandbox = sandbox_with("multicpk_team_over_cap", &EXPORTS);
    let settings = multicpk_settings(&sandbox, "cpk_part_max_size = 4096\n");

    let run = compile(&sandbox, &settings);

    assert_aborted_with(&sandbox, &run, |line| {
        line.starts_with("Fatal cpk_team_exceeds_cap [AbortRun] (export=a Midcup Tracer, size=")
            && line.ends_with(", cap=4 KiB)")
    });
}

// TC-OUT-14
#[test]
fn content_past_the_last_slot_aborts_the_run_naming_the_shortfall() {
    let (cap, _) = first_part_of_a_and_b("multicpk_slots_exhausted_a_and_b");
    let sandbox = sandbox_with("multicpk_slots_exhausted", &EXPORTS);
    sandbox.write(
        "data/templates/DpFileList.bin",
        &dpfl_bytes(&["4cc_08_bins.cpk", "4cc_41_teams.cpk", "4cc_99_test.cpk"]),
    );
    let settings = multicpk_settings(&sandbox, &format!("cpk_part_max_size = {cap}\n"));

    let run = compile(&sandbox, &settings);

    let expected = format!(
        "Fatal cpk_slots_exhausted [AbortRun] (export=co Midcup Tracer, stem=teams, slots=1, \
         cap={})",
        size_text(cap)
    );
    assert_aborted_with(&sandbox, &run, |line| line == expected);
}

// TC-OUT-15
#[test]
fn a_single_cpk_past_the_cap_is_a_warning_and_is_written_whole() {
    let sandbox = Sandbox::new("single_cpk_over_cap");
    let settings = format!(
        "{}[team-compiler]\ncpk_part_max_size = 4096\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(
        &settings,
        &["compile", "--no-deploy", "--export", &tracer_export()],
    );

    assert_eq!(run.exit_code(), 0);
    let lines = run.messages();
    let warning = lines
        .iter()
        .find(|line| line.contains("cpk_size_over_limit"))
        .unwrap_or_else(|| panic!("{lines:#?}"));
    assert!(
        warning.starts_with("Warning cpk_size_over_limit [Keep] (cpk=4cc_99_test.cpk, size=")
            && warning.ends_with(", cap=4 KiB)"),
        "{warning}"
    );
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(entries.contains_key(&kit_texture("u0792g1")), "the kit");
}

// TC-OUT-16
#[test]
fn a_second_slot_run_is_filled_by_its_own_stem_and_the_first_is_left_alone() {
    let sandbox = sandbox_with("multicpk_teams2", &EXPORTS);
    sandbox.write(
        "data/templates/DpFileList.bin",
        &dpfl_bytes(&["4cc_08_bins.cpk", "4cc_41_teams.cpk", "4cc_51_teams2.cpk"]),
    );
    let settings = multicpk_settings(&sandbox, "teams_cpk_name = \"teams2\"\n");

    let run = compile(&sandbox, &settings);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        output_files(&sandbox),
        BTreeSet::from(["4cc_08_bins.cpk".to_owned(), "4cc_51_teams2.cpk".to_owned()])
    );
    let part = cpk_entries(&sandbox.root.join("output/4cc_51_teams2.cpk"));
    for team in ["u0702g1", "u0707g1", "u0714g1"] {
        assert!(part.contains_key(&kit_texture(team)), "{team}");
    }
}

#[test]
fn multi_cpk_mode_needs_no_deploy_and_a_valid_bins_cpk_name() {
    let sandbox = sandbox_with("multicpk_refused", &EXPORTS[..1]);

    let run = sandbox.run(&multicpk_settings(&sandbox, ""), &["compile"]);
    run.assert_refused(2, &["multicpk_mode", "--no-deploy"]);

    let settings = multicpk_settings(&sandbox, "bins_cpk_name = \"con\"\n");
    let run = compile(&sandbox, &settings);
    run.assert_refused(2, &["bins_cpk_name = \"con\""]);

    assert!(!sandbox.root.join("output").exists(), "nothing is written");
}

#[test]
fn test_mode_writes_its_loose_files_as_without_multi_cpk_mode() {
    let mut trees = Vec::new();
    for (name, multicpk) in [
        ("multicpk_test_mode_on", true),
        ("multicpk_test_mode_off", false),
    ] {
        let sandbox = Sandbox::new(name);
        let settings = format!(
            "{}[team-compiler]\nmulticpk_mode = {multicpk}\n",
            pes21_settings(&sandbox)
        );
        let run = sandbox.run(
            &settings,
            &["compile", "--mode", "test", "--export", &tracer_export()],
        );
        assert_eq!(run.exit_code(), 0, "{name}");
        assert_eq!(
            output_files(&sandbox),
            BTreeSet::from(["test_output".to_owned()]),
            "{name}"
        );
        trees.push(snapshot(&sandbox.root.join("output/test_output")));
    }
    assert!(trees[0] == trees[1], "the same loose files");
}
