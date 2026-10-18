//! Team identity and the root files (the plan's "Team identity and root
//! files" scenarios): the Note's `Team:` line first, the export name's first
//! word as the fallback.

use std::fs;

use studio_core::PipelineEvent;

use crate::common::Sandbox;

/// `upgrade <input> --savefile <save>`.
fn upgrade(sandbox: &Sandbox, input: &str, save: &str) -> crate::common::Run {
    let input = sandbox.arg(input);
    let save = sandbox.arg(save);
    sandbox.run(&["upgrade", &input, "--savefile", &save])
}

// EU-ROOT-01
#[test]
fn a_note_team_line_overrides_the_exports_name() {
    let sandbox = Sandbox::new("root_note_team");
    // "Numbers" holds the same export as the `co Spring` fixture: its Note's
    // Team: line names /co/.
    sandbox.copy_fixture_dir("co Spring", "Numbers");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "Numbers", "EDIT");
    assert_eq!(run.exit_code(), 1);
    // The name only: the players' settings.toml files the scenario also names
    // are step 8's (6.5), not this slice's.
    assert!(sandbox.path("co Full Numbers").is_dir(), "{}", run.stdout());
}

// EU-ROOT-01
#[test]
fn a_note_only_export_reports_no_unmigrated_content() {
    let sandbox = Sandbox::new("root_note_only");
    // A `Team:` Note alone makes an old export; there is nothing else to
    // migrate or leave behind.
    fs::create_dir_all(sandbox.path("Numbers")).unwrap();
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/co Spring/co Note.txt"),
        sandbox.path("Numbers/co Note.txt"),
    )
    .unwrap();
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "Numbers", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(
        !run.stdout().contains("unmigrated_content"),
        "{}",
        run.stdout()
    );
    assert!(
        run.events
            .iter()
            .all(|envelope| !matches!(envelope.event, PipelineEvent::UnmigratedContentFound)),
        "no UnmigratedContentFound when nothing is unmigrated"
    );
    assert!(sandbox.path("co Full Numbers").is_dir(), "{}", run.stdout());
}

// EU-ROOT-02
#[test]
fn without_a_note_the_first_word_resolves() {
    let sandbox = Sandbox::new("root_no_note");
    fs::create_dir_all(sandbox.path("co Spring 26/Other")).unwrap();
    fs::write(
        sandbox.path("co Spring 26/Other/readme.txt"),
        b"not migrated",
    )
    .unwrap();
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring 26", "EDIT");
    assert_eq!(run.exit_code(), 1);
    let output = sandbox.path("co Full Spring 26");
    assert!(output.is_dir(), "{}", run.stdout());
    // This slice writes an empty draft: no colors.txt, notes.txt or Note txt.
    assert_eq!(sandbox.listing("co Full Spring 26"), Vec::<String>::new());
}

// EU-ROOT-03
#[test]
fn a_team_the_list_lacks_is_reported_nothing_written() {
    let sandbox = Sandbox::new("root_unknown_team");
    // "co Spring 26" holds a Note whose Team: line names no team of the teams
    // list (`/newteam/` is no row of the embedded list).
    fs::create_dir_all(sandbox.path("co Spring 26")).unwrap();
    fs::write(
        sandbox.path("co Spring 26/co Note.txt"),
        b"Team: /newteam/\r\n\r\nTeam Colours:\r\n- 1st: 048 146 082",
    )
    .unwrap();
    // "Numbers" holds an old export (`Other/`) and no Note; `/numbers/` is no
    // row of the list either.
    fs::create_dir_all(sandbox.path("Numbers/Other")).unwrap();
    fs::write(sandbox.path("Numbers/Other/readme.txt"), b"x").unwrap();
    sandbox.save("EDIT");
    let before = sandbox.listing("");

    let run = upgrade(&sandbox, "co Spring 26", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(
        run.stdout()
            .contains("- co Spring 26: Error team_name_unknown at co Note.txt (name=/newteam/)"),
        "{}",
        run.stdout()
    );

    let run = upgrade(&sandbox, "Numbers", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(
        run.stdout()
            .contains("- Numbers: Error team_name_unknown (name=Numbers)"),
        "{}",
        run.stdout()
    );

    assert_eq!(sandbox.listing(""), before, "nothing is written");
}
