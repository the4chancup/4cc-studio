//! Archive sources: `.zip` and `.7z` exports read beside folder ones, and the ones refused.

use std::fs;

use crate::common::Sandbox;
use crate::{CLEAN_PLAYER, findings_of, snapshot};

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
    "Info export_identified [Keep] (team=/co/, id=701)",
];

#[test]
fn one_export_as_a_folder_a_zip_and_a_7z_reports_the_same_findings() {
    let sandbox = Sandbox::new("same_export_three_ways");
    spring_folder(&sandbox);
    sandbox.copy_fixture("co - Spring.zip", "exports");
    sandbox.copy_fixture("co - Spring.7z", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    for source in ["co - Spring", "co - Spring.zip", "co - Spring.7z"] {
        assert_eq!(findings_of(&lines, source), SPRING_FINDINGS, "{lines:#?}");
    }
    // Three exports of one team draw no finding about each other.
    assert_eq!(lines.len(), 3 * SPRING_FINDINGS.len(), "{lines:#?}");
    assert_eq!(run.exit_code(), 1);
}

// TC-SRC-02
#[test]
fn a_folder_and_an_archive_sharing_a_stem_are_two_exports() {
    let sandbox = Sandbox::new("folder_and_archive");
    spring_folder(&sandbox);
    sandbox.copy_fixture("co - Spring.zip", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(findings_of(&lines, "co - Spring"), SPRING_FINDINGS);
    assert_eq!(findings_of(&lines, "co - Spring.zip"), SPRING_FINDINGS);
    assert_eq!(lines.len(), 2 * SPRING_FINDINGS.len(), "{lines:#?}");
}

#[test]
fn a_corrupt_archive_is_skipped_and_the_export_beside_it_is_still_checked() {
    let sandbox = Sandbox::new("corrupt_archive");
    // The scan compares the extension in any case.
    sandbox.write("exports/co - Broken.ZIP", b"not a zip at all");
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"");

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
        ["Info export_identified [Keep] (team=/co/, id=701)"]
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
    sandbox.write(&format!("exports/co - A/{CLEAN_PLAYER}"), b"");
    sandbox.write(&format!("exports/co - C/{CLEAN_PLAYER}"), b"");
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
    assert_eq!(
        findings_of(&lines, "co - A"),
        ["Info export_identified [Keep] (team=/co/, id=701)"]
    );
    assert_eq!(findings_of(&lines, "co - Spring.zip"), SPRING_FINDINGS);
    // co - C, in the root but not named, is not reported.
    assert_eq!(lines.len(), 1 + SPRING_FINDINGS.len(), "{lines:#?}");
}

#[test]
fn check_leaves_every_archive_and_nested_export_as_it_was() {
    let sandbox = Sandbox::new("archives_untouched");
    sandbox.copy_fixture("co - Spring.zip", "exports");
    sandbox.copy_fixture("co - Spring.7z", "exports");
    sandbox.write(
        &format!("exports/co - Nested/wrapper/{CLEAN_PLAYER}"),
        b"model",
    );
    let exports = sandbox.root.join("exports");
    let before = snapshot(&exports);

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Nested"),
        [
            "Warning nested_folders_fixed [Keep] (folder=wrapper)",
            "Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(findings_of(&lines, "co - Spring.7z"), SPRING_FINDINGS);
    assert_eq!(findings_of(&lines, "co - Spring.zip"), SPRING_FINDINGS);
    assert_eq!(snapshot(&exports), before);
}
