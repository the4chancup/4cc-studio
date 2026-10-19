//! The command line's own rules (the plan's "CLI" scenarios): the output
//! name's tag, `-o`, the flag and path refusals, and the exit codes.

use std::fs;

use pes_version::PesVersion;

use crate::common::{Sandbox, snapshot};

/// `upgrade <input> <flags>`.
fn upgrade(sandbox: &Sandbox, input: &str, flags: &[&str]) -> crate::common::Run {
    let input = sandbox.arg(input);
    let mut args = vec!["upgrade", &input];
    args.extend(flags);
    sandbox.run(&args)
}

/// A minimal old export at `relative`: an `Other/` folder with a file, plus a
/// `co Note.txt` holding `Team: <team>` when `team` names one.
fn old_export(sandbox: &Sandbox, relative: &str, team: Option<&str>) {
    fs::create_dir_all(sandbox.path(&format!("{relative}/Other"))).unwrap();
    fs::write(sandbox.path(&format!("{relative}/Other/readme.txt")), b"x").unwrap();
    if let Some(team) = team {
        fs::write(
            sandbox.path(&format!("{relative}/co Note.txt")),
            format!("Team: {team}\r\n"),
        )
        .unwrap();
    }
}

// EU-CLI-01
#[test]
fn the_output_name_carries_the_key_the_tag_and_the_rest() {
    let sandbox = Sandbox::new("cli_output_names");
    for name in [
        "co - Spring",
        "co Day 8 Additions",
        "co midcup Day 5",
        "co Full Spring",
    ] {
        old_export(&sandbox, name, Some("/co/"));
    }
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");

    for (input, output) in [
        ("co - Spring", "co Full - Spring"),
        ("co Day 8 Additions", "co Midcup Day 8 Additions"),
        ("co midcup Day 5", "co midcup Day 5 upgraded"),
        ("co Full Spring", "co Full Spring upgraded"),
    ] {
        let run = upgrade(&sandbox, input, &["--savefile", &save]);
        assert_eq!(run.exit_code(), 1, "{input}: {}", run.stdout());
        assert!(sandbox.path(output).is_dir(), "{input} -> {output}");
    }
}

// EU-CLI-08
#[test]
fn an_input_named_with_a_trailing_dot_or_dot_dot_upgrades_beside_it() {
    let sandbox = Sandbox::new("cli_dot_input");
    sandbox.copy_fixture_dir("co Spring", "a/co Spring");
    sandbox.copy_fixture_dir("co Spring", "b/co Spring");
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");
    let inside_a = sandbox.listing("a/co Spring");
    let inside_b = sandbox.listing("b/co Spring");

    for (input, output) in [
        (
            format!(
                "{}{}.",
                sandbox.arg("a/co Spring"),
                std::path::MAIN_SEPARATOR
            ),
            "a/co Full Spring",
        ),
        (
            format!(
                "{}{}..",
                sandbox.arg("b/co Spring/Other"),
                std::path::MAIN_SEPARATOR
            ),
            "b/co Full Spring",
        ),
    ] {
        let run = sandbox.run(&["upgrade", &input, "--savefile", &save]);
        assert_eq!(run.exit_code(), 0, "{input}: {}", run.stdout());
        assert!(sandbox.path(output).is_dir(), "{input}: {}", run.stdout());
    }
    assert_eq!(
        sandbox.listing("a/co Spring"),
        inside_a,
        "nothing is written inside the input"
    );
    assert_eq!(
        sandbox.listing("b/co Spring"),
        inside_b,
        "nothing is written inside the input"
    );
}

// EU-CLI-02
#[test]
fn the_output_goes_inside_the_directory_o_names() {
    let sandbox = Sandbox::new("cli_output_dir");
    sandbox.copy_fixture_dir("co Spring", "in/co Spring");
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");
    let out = sandbox.arg("out");

    let run = upgrade(&sandbox, "in/co Spring", &["--savefile", &save, "-o", &out]);
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert!(sandbox.path("out/co Full Spring").is_dir());
    assert!(
        !sandbox.path("in/co Full Spring").exists(),
        "nothing is written beside the input"
    );
}

// EU-CLI-03
#[test]
fn a_team_export_takes_exactly_one_of_savefile_and_loose() {
    let sandbox = Sandbox::new("cli_flag_pair");
    old_export(&sandbox, "co Spring", Some("/co/"));
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");
    let before = sandbox.listing("");

    let both = upgrade(&sandbox, "co Spring", &["--savefile", &save, "--loose"]);
    both.assert_refused(2, &["--savefile", "--loose"]);
    let neither = upgrade(&sandbox, "co Spring", &[]);
    neither.assert_refused(2, &["--savefile", "--loose"]);
    assert_eq!(sandbox.listing(""), before, "nothing is written");
}

// EU-REF-05
#[test]
fn a_referee_export_takes_neither_flag_and_its_note_is_a_leftover() {
    let sandbox = Sandbox::new("cli_refs");
    // A `refs` export's name decides before anything is read (step 9): it
    // takes neither flag, and its Note is a leftover: nothing compiled reads
    // a referee's Note.
    fs::create_dir_all(sandbox.path("refs Spring")).unwrap();
    fs::write(sandbox.path("refs Spring/refs.txt"), b"01 Keeper\n").unwrap();
    fs::write(sandbox.path("refs Spring/refs Note.txt"), b"Team: /co/\n").unwrap();
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");
    let before = sandbox.listing("");

    upgrade(&sandbox, "refs Spring", &["--savefile", &save]).assert_refused(2, &["--savefile"]);
    upgrade(&sandbox, "refs Spring", &["--loose"]).assert_refused(2, &["--loose"]);
    assert_eq!(sandbox.listing(""), before, "nothing is written");

    let run = upgrade(&sandbox, "refs Spring", &[]);
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    assert!(
        sandbox.path("refs Spring upgraded").is_dir(),
        "{}",
        run.stdout()
    );
    assert!(
        run.stdout()
            .contains("- refs Spring: Warning leftover_dropped at refs Note.txt"),
        "{}",
        run.stdout()
    );
}

// EU-CLI-04
#[test]
fn the_refusals_before_anything_is_written() {
    let sandbox = Sandbox::new("cli_refusals");
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");
    fs::write(sandbox.path("notes.txt"), b"text").unwrap();
    old_export(&sandbox, "co Spring", Some("/co/"));
    fs::write(sandbox.path("not-a-save.bin"), b"no save").unwrap();
    // A savefile that does not hold the resolved team. Every fixture save
    // holds every resolvable team of the embedded list, so the sandbox's data
    // directory gets a one-row list naming /newteam/ at 903 — a team the list
    // gained after the save was written — and the save is the PES 18 fixture,
    // whose teams end at 902.
    fs::write(
        sandbox.path("data/teams_list.txt"),
        "ID\tName\n903\t/newteam/\n",
    )
    .unwrap();
    old_export(&sandbox, "newteam Spring", Some("/newteam/"));
    let save18 = sandbox.arg("EDIT18");
    fs::write(
        sandbox.path("EDIT18"),
        pes_savefile::fixture_save(PesVersion::Pes18),
    )
    .unwrap();
    // A filesystem root is not an export, savefile or not.
    let root = sandbox
        .path("co Spring")
        .ancestors()
        .last()
        .unwrap()
        .to_path_buf();
    let not_save = sandbox.arg("not-a-save.bin");
    let before = sandbox.listing("");

    upgrade(&sandbox, "gone", &["--savefile", &save])
        .assert_refused(2, &["gone", "no such folder or file"]);
    upgrade(&sandbox, "notes.txt", &["--savefile", &save])
        .assert_refused(2, &["notes.txt", "not a folder, .zip or .7z"]);
    upgrade(&sandbox, "co Spring", &["--savefile", &not_save])
        .assert_refused(2, &["not-a-save.bin"]);
    upgrade(&sandbox, "newteam Spring", &["--savefile", &save18]).assert_refused(2, &["/newteam/"]);
    let root_arg = root.display().to_string();
    let run = sandbox.run(&["upgrade", &root_arg, "--savefile", &save]);
    run.assert_refused(2, &["a filesystem root is not an export"]);

    assert_eq!(sandbox.listing(""), before, "nothing is written");
}

// EU-CLI-05
#[test]
fn an_output_folder_that_exists_or_cannot_be_created_is_refused() {
    let sandbox = Sandbox::new("cli_output_exists");
    sandbox.copy_fixture_dir("co Spring", "co Spring");
    // An existing output folder, with a member's edit inside.
    fs::create_dir_all(sandbox.path("co Full Spring")).unwrap();
    fs::write(sandbox.path("co Full Spring/keep.txt"), b"my edit").unwrap();
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");

    let before = snapshot(&sandbox);
    let run = upgrade(&sandbox, "co Spring", &["--savefile", &save]);
    run.assert_refused(3, &["co Full Spring"]);
    assert_eq!(snapshot(&sandbox), before, "nothing is written");

    // An output that cannot be created: -o names a file, not a directory.
    fs::write(sandbox.path("blocked"), b"a file").unwrap();
    let blocked = sandbox.arg("blocked");
    let before = snapshot(&sandbox);
    let run = upgrade(
        &sandbox,
        "co Spring",
        &["--savefile", &save, "-o", &blocked],
    );
    run.assert_refused(3, &["blocked"]);
    assert_eq!(snapshot(&sandbox), before, "nothing is written");
}

// EU-CLI-07
#[test]
fn a_corrupt_archive_is_export_extract_failed() {
    let sandbox = Sandbox::new("cli_corrupt_zip");
    fs::write(sandbox.path("broken.zip"), b"not a zip at all").unwrap();
    sandbox.save("EDIT");
    let save = sandbox.arg("EDIT");
    let before = sandbox.listing("");

    let run = upgrade(&sandbox, "broken.zip", &["--savefile", &save]);
    assert_eq!(run.exit_code(), 1);
    assert!(
        run.stdout().contains("Error export_extract_failed"),
        "{}",
        run.stdout()
    );
    assert!(run.stdout().contains("broken.zip"), "{}", run.stdout());
    assert_eq!(sandbox.listing(""), before, "nothing is written");
}
