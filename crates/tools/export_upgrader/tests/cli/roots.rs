//! Team identity and the root files (the plan's "Team identity and root
//! files" scenarios): the Note's `Team:` line first, the export name's first
//! word as the fallback.

use std::fs;

use pes_version::PesVersion;
use studio_core::PipelineEvent;

use crate::common::Sandbox;
use crate::players::wears;

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
    // Team: line names /co/. A face folder gives the output a player folder
    // for the settings.toml half of the scenario.
    sandbox.copy_fixture_dir("co Spring", "Numbers");
    sandbox.write("Numbers/Faces/XXX05 - PLACEHOLDER/fcl_hair.fmdl", b"hair");
    // Slot 05's record carries the boots ID, so his generated settings.toml
    // proves it came from /co/'s records. A stock model's ID: one in the
    // team's old block (426-450) names an export's own boots and is not
    // written.
    sandbox.save_edited("EDIT", PesVersion::Pes21, |file| wears(file, 71405, 42, 0));

    let run = upgrade(&sandbox, "Numbers", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(sandbox.path("co Full Numbers").is_dir(), "{}", run.stdout());
    // Every output player folder holds its settings.toml, generated from
    // /co/'s records in the save.
    let players = sandbox.listing("co Full Numbers/Players");
    assert!(!players.is_empty(), "the output holds a player folder");
    for folder in players.iter().filter(|path| !path.contains('/')) {
        assert!(
            sandbox
                .path(&format!("co Full Numbers/Players/{folder}/settings.toml"))
                .is_file(),
            "{folder}"
        );
    }
    let text =
        fs::read_to_string(sandbox.path("co Full Numbers/Players/05 - PLACEHOLDER/settings.toml"))
            .unwrap();
    assert!(
        text.contains("boots_id = 42"),
        "slot 05's record's ID: {text}"
    );
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
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
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

// EU-ROOT-04
#[test]
fn the_notes_team_colors_and_other_notes_become_the_root_files() {
    let sandbox = Sandbox::new("root_files");
    sandbox.copy_fixture_dir("co Spring", "co Spring");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/colors.txt")).unwrap(),
        b"048 146 082\n221 000 000\n000 000 000\n000 000 000\n"
    );
    // The Note's two lines after `Other Notes:`, joined by `\n`, CRLF dropped,
    // a final `\n` added.
    let note = fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/co Spring/co Note.txt"),
    )
    .unwrap();
    let note_text = String::from_utf8_lossy(&note);
    let after = note_text.split("Other Notes:").nth(1).unwrap();
    let expected = format!("{}\n", after.trim().replace("\r\n", "\n"));
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/notes.txt")).unwrap(),
        expected.as_bytes()
    );
    // The output holds no `.txt` whose name holds `note` other than
    // `notes.txt`: the Note itself is not written.
    let note_txts: Vec<String> = sandbox
        .listing("co Full Spring")
        .into_iter()
        .filter(|path| {
            let lowered = path.to_ascii_lowercase();
            lowered.ends_with(".txt") && lowered.contains("note")
        })
        .collect();
    assert_eq!(note_txts, ["notes.txt"], "{note_txts:?}");
}

// EU-ROOT-06
#[test]
fn a_team_entry_whose_colors_do_not_read_is_unmigrated_content() {
    let sandbox = Sandbox::new("root_bad_entry");
    // The export holds the Note alone; the good entries are still written and
    // the unreadable one is reported.
    fs::create_dir_all(sandbox.path("co Spring")).unwrap();
    fs::write(
        sandbox.path("co Spring/co Note.txt"),
        "Team: /co/\r\n\r\nTeam Colours:\r\n- 1st: 048 146 082\r\n- 2nd: blue\r\n- 3rd: 000 000 000\r\n",
    )
    .unwrap();
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(
        run.stdout()
            .contains("- co Spring: Error unmigrated_content at co Note.txt (line=- 2nd: blue)"),
        "{}",
        run.stdout()
    );
    let found = run
        .events
        .iter()
        .filter(|envelope| matches!(envelope.event, PipelineEvent::UnmigratedContentFound))
        .count();
    assert_eq!(found, 1, "one UnmigratedContentFound");
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/colors.txt")).unwrap(),
        b"048 146 082\n000 000 000\n"
    );
}

// EU-ROOT-06
#[test]
fn a_team_entry_with_nothing_after_its_colon_is_unmigrated_content() {
    let sandbox = Sandbox::new("root_empty_entry");
    // `- 2nd:`, an AET template line left unfilled, reads as no color.
    fs::create_dir_all(sandbox.path("co Spring")).unwrap();
    fs::write(
        sandbox.path("co Spring/co Note.txt"),
        "Team: /co/\r\n\r\nTeam Colours:\r\n- 1st: 048 146 082\r\n- 2nd:\r\n",
    )
    .unwrap();
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(
        run.stdout()
            .contains("- co Spring: Error unmigrated_content at co Note.txt (line=- 2nd:)"),
        "{}",
        run.stdout()
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/colors.txt")).unwrap(),
        b"048 146 082\n"
    );
}

// EU-ROOT-05
#[test]
fn a_shift_jis_notes_ascii_colors_read() {
    let sandbox = Sandbox::new("root_jp_note");
    sandbox.copy_fixture_dir("jp Summer 18", "jp Summer 18");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "jp Summer 18", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("jp Full Summer 18/colors.txt")).unwrap(),
        b"058 080 177\n255 255 255\n"
    );
    // The kit entries' colors, each written as the Note spells it.
    for (slot, colors) in [
        ("p1", "#212D62\n#DE344F\n"),
        ("p2", "#EEEEEE\n#70C8DE\n"),
        ("p3", "#050505\n#EFD0C3\n"),
        ("p4", "#D6384A\n#4466A7\n"),
        ("p5", "#000400\n#7B0000\n"),
        ("g1", "#FFFFFF\n#000000\n"),
    ] {
        assert_eq!(
            fs::read(sandbox.path(&format!("jp Full Summer 18/Kits/{slot}/colors.txt"))).unwrap(),
            colors.as_bytes(),
            "{slot}"
        );
    }
    // The "Other Notes:" section is empty: no notes.txt.
    assert!(!sandbox.path("jp Full Summer 18/notes.txt").exists());
    assert!(
        !run.stdout().contains("unmigrated_content"),
        "{}",
        run.stdout()
    );
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
