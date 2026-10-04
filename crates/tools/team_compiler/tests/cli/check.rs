//! `check`: the structure pass's findings for folder exports, and the exit code they give.

use studio_core::{ExportId, PipelineEvent, RunId};

use crate::common::Sandbox;
use crate::{CLEAN_PLAYER, clean_model, snapshot};

#[test]
fn check_brackets_each_export_with_its_start_and_end() {
    let sandbox = Sandbox::new("envelopes");
    sandbox.write(
        &format!("exports/co - Spring/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    let run = sandbox.run("", &["check"]);
    assert_eq!(run.exit_code(), 0);
    let export = Some(ExportId(0));
    let kinds: Vec<(Option<ExportId>, &str)> = run
        .events
        .iter()
        .map(|envelope| {
            assert_eq!(envelope.run_id, RunId(1));
            assert_eq!(envelope.export_revision, None);
            let kind = match &envelope.event {
                PipelineEvent::ExportStarted {
                    export_id,
                    display_name,
                } => {
                    assert_eq!(
                        (*export_id, display_name.as_str()),
                        (ExportId(0), "co - Spring")
                    );
                    "started"
                }
                PipelineEvent::Message(_) => "message",
                PipelineEvent::ExportProcessed { export_id } => {
                    assert_eq!(*export_id, ExportId(0));
                    "processed"
                }
                PipelineEvent::FolderStatus { .. }
                | PipelineEvent::Progress { .. }
                | PipelineEvent::Complete { .. }
                | PipelineEvent::UnmigratedContentFound => "other",
            };
            (envelope.export_id, kind)
        })
        .collect();
    assert_eq!(
        kinds,
        [
            (export, "started"),
            (export, "message"),
            (export, "processed")
        ]
    );
}

// TC-SRC-05
#[test]
fn two_refs_exports_are_both_skipped_and_a_disabled_one_is_left_out() {
    let sandbox = Sandbox::new("multiple_refs");
    for name in ["refs a", "refs b"] {
        sandbox.write(&format!("exports/{name}/players.txt"), b"01 Keeper\n");
        sandbox.write(
            &format!("exports/{name}/Players/Keeper/face_high.fmdl"),
            &clean_model(),
        );
        // Validated, this would be root_file_unexpected: its absence shows neither is.
        sandbox.write(&format!("exports/{name}/extra.bin"), b"");
    }
    sandbox.write("exports/refs c/NO_USE", b"");
    sandbox.write("exports/refs c/players.txt", b"01 Keeper\n");

    let run = sandbox.run("", &["check"]);

    assert_eq!(
        run.messages(),
        [
            "Error multiple_ref_exports [DropExport] (exports=refs a, refs b)",
            "refs a: Error multiple_ref_exports [DropExport] ()",
            "refs b: Error multiple_ref_exports [DropExport] ()",
            "refs c: Info export_disabled [DropExport] ()",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-STR-09
#[test]
fn a_disallowed_file_is_an_error_when_strict_and_an_info_when_lenient() {
    let sandbox = Sandbox::new("strict_file_type_check");
    sandbox.write("exports/co - Spring/Players/03 - A/hair.dds", b"");
    sandbox.write("exports/co - Spring/Players/03 - A/readme.txt", b"");

    let strict = sandbox.run(
        "[team-compiler]\nstrict_file_type_check = true\n",
        &["check"],
    );
    assert_eq!(
        strict.messages(),
        [
            "co - Spring: Error file_type_disallowed [DropFolder] at Players/03 - A (file=readme.txt)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(strict.exit_code(), 1);

    let lenient = sandbox.run(
        "[team-compiler]\nstrict_file_type_check = false\n",
        &["check"],
    );
    assert_eq!(
        lenient.messages(),
        [
            "co - Spring: Info file_type_disallowed [Keep] at Players/03 - A (file=readme.txt)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(lenient.exit_code(), 0);
}

#[test]
fn pass_through_keeps_a_disallowed_file_at_its_error_severity() {
    let sandbox = Sandbox::new("pass_through");
    sandbox.write("exports/co - Spring/Players/03 - A/readme.txt", b"");
    let run = sandbox.run("[team-compiler]\npass_through = true\n", &["check"]);
    assert_eq!(
        run.messages(),
        [
            "co - Spring: Error file_type_disallowed [Keep] at Players/03 - A (file=readme.txt)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-CLI-01
#[test]
fn check_writes_nothing() {
    let sandbox = Sandbox::new("writes_nothing");
    let settings = "[team-compiler]\ncpk_name = \"cup\"\n";
    sandbox.write("data/settings.toml", settings.as_bytes());
    sandbox.write(
        &format!("exports/co - Spring/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    sandbox.write("exports/co - Spring/notes.txt", b"a note");
    sandbox.write("exports/zz - Autumn/Players/03 - B/readme.txt", b"text");
    let before = snapshot(&sandbox.root);

    let run = sandbox.run(settings, &["check"]);

    assert_eq!(run.exit_code(), 1);
    assert!(!sandbox.root.join("output").exists());
    // The settings file, the teams list and every export file, listing and bytes.
    assert_eq!(snapshot(&sandbox.root), before);
}

// TC-CLI-02
#[test]
fn check_exits_one_only_for_an_error_finding() {
    let sandbox = Sandbox::new("exit_codes");
    sandbox.write(
        &format!("exports/co - Notes/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    sandbox.write("exports/co - Notes/notes.txt", b"a note");
    sandbox.write("exports/co - Notes/extra.bin", b"");
    sandbox.write("exports/co - Error/Players/03 - A/readme.txt", b"");
    sandbox.write(
        &format!("exports/co - Clean/{CLEAN_PLAYER}"),
        &clean_model(),
    );

    let run = sandbox.run(
        "",
        &["check", "--export", &sandbox.arg("exports/co - Notes")],
    );
    assert_eq!(
        run.messages(),
        [
            "co - Notes: Warning root_file_unexpected [DropFile] at extra.bin ()",
            "co - Notes: Info notes_found [Keep] at notes.txt ()",
            "co - Notes: Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(run.exit_code(), 0);

    let run = sandbox.run(
        "",
        &["check", "--export", &sandbox.arg("exports/co - Error")],
    );
    assert_eq!(
        run.messages(),
        [
            "co - Error: Error file_type_disallowed [DropFolder] at Players/03 - A (file=readme.txt)",
            "co - Error: Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(run.exit_code(), 1);

    let run = sandbox.run(
        "",
        &["check", "--export", &sandbox.arg("exports/co - Clean")],
    );
    assert_eq!(
        run.messages(),
        ["co - Clean: Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn a_disabled_export_reports_only_that_whatever_the_marker_case() {
    let sandbox = Sandbox::new("disabled");
    for (name, marker) in [("co - One", "NO_USE.txt"), ("co - Two", "no_use")] {
        sandbox.write(&format!("exports/{name}/{marker}"), b"");
        sandbox.write(&format!("exports/{name}/extra.bin"), b"");
    }
    // A marker below the source's own root is the nested root's content, not a marker.
    sandbox.write("exports/co - Three/wrapper/NO_USE", b"");
    sandbox.write(
        &format!("exports/co - Three/wrapper/{CLEAN_PLAYER}"),
        &clean_model(),
    );

    let run = sandbox.run("", &["check"]);

    assert_eq!(
        run.messages(),
        [
            "co - One: Info export_disabled [DropExport] ()",
            "co - Three: Warning nested_folders_fixed [Keep] (folder=wrapper)",
            "co - Three: Warning root_file_unexpected [DropFile] at NO_USE ()",
            "co - Three: Info export_identified [Keep] (team=/co/, id=714)",
            "co - Two: Info export_disabled [DropExport] ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn a_balls_export_is_skipped_unread() {
    let sandbox = Sandbox::new("balls");
    sandbox.write("exports/balls Spring/extra.bin", b"");
    let run = sandbox.run("", &["check"]);
    assert_eq!(
        run.messages(),
        ["balls Spring: Info export_balls_skipped [DropExport] ()"]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn an_export_is_identified_by_its_first_word() {
    let sandbox = Sandbox::new("identified");
    sandbox.write(
        &format!("exports/co - Spring 2026/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    // A refs export needs no teams-list row.
    sandbox.write("exports/refs Cup/players.txt", b"01 Keeper\n");
    sandbox.write(
        "exports/refs Cup/Players/Keeper/face_high.fmdl",
        &clean_model(),
    );
    let run = sandbox.run("", &["check"]);
    assert_eq!(
        run.messages(),
        [
            "co - Spring 2026: Info export_identified [Keep] (team=/co/, id=714)",
            "refs Cup: Info export_identified [Keep] (team=referees)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn an_unknown_team_is_an_error_and_the_export_beside_it_is_still_checked() {
    let sandbox = Sandbox::new("unknown_team");
    sandbox.write(
        &format!("exports/zz - Spring/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    sandbox.write(&format!("exports/---/{CLEAN_PLAYER}"), &clean_model());
    sandbox.write(
        &format!("exports/co - Spring/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    let run = sandbox.run("", &["check"]);
    assert_eq!(
        run.messages(),
        [
            "---: Error team_name_unknown [DropExport] (team_name=)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=714)",
            "zz - Spring: Error team_name_unknown [DropExport] (team_name=/zz/)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn check_creates_the_missing_default_exports_folder_and_reports_it_empty() {
    let sandbox = Sandbox::new("missing_root");
    let exports = sandbox.root.join("exports");
    let run = sandbox.run("", &["check"]);
    assert!(exports.is_dir(), "the folder was created");
    assert_eq!(
        run.messages(),
        [format!(
            "Warning no_exports_found [Keep] (folder={})",
            exports.display()
        )]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn export_paths_restrict_the_run_to_the_named_sources() {
    let sandbox = Sandbox::new("named_sources");
    for name in ["co - A", "co - B", "co - C"] {
        sandbox.write(&format!("exports/{name}/{CLEAN_PLAYER}"), &clean_model());
    }
    sandbox.write(&format!("elsewhere/co - D/{CLEAN_PLAYER}"), &clean_model());
    let run = sandbox.run(
        "",
        &[
            "check",
            "--export",
            &sandbox.arg("elsewhere/co - D"),
            "--export",
            &sandbox.arg("exports/co - B"),
        ],
    );
    // The two named exports of one team conflict; `co - A` and `co - C`, not named, do not.
    let duplicate =
        "Error duplicate_aesthetics_export [DropExport] (id=714, exports=co - B, co - D)";
    assert_eq!(
        run.messages(),
        [
            "co - B: Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
            format!("co - B: {duplicate}"),
            "co - D: Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
            format!("co - D: {duplicate}"),
        ]
    );
    assert_eq!(run.exit_code(), 1);
}
