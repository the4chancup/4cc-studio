//! The face folders' migration into `Players/` (the plan's "Player folders"
//! scenarios): the folder naming order, portraits, `ingame_face` markers, the
//! empty roster slot and the doubled `Faces/Faces/` flattening.

use std::fs;

use studio_core::PipelineEvent;

use crate::common::Sandbox;

/// How many `UnmigratedContentFound` events a run emitted.
fn unmigrated_found(run: &crate::common::Run) -> usize {
    run.events
        .iter()
        .filter(|envelope| matches!(envelope.event, PipelineEvent::UnmigratedContentFound))
        .count()
}

/// `upgrade <input> --savefile <save>`.
fn upgrade(sandbox: &Sandbox, input: &str, save: &str) -> crate::common::Run {
    let input = sandbox.arg(input);
    let save = sandbox.arg(save);
    sandbox.run(&["upgrade", &input, "--savefile", &save])
}

/// The four files of the real tracer Fox face folder
/// (`team_compiler/tests/fixtures/tracer/old/egg Tracer/Faces/XXX05 - …/`),
/// a few bytes each (a test of names, not formats).
const FACE_FILES: [(&str, &[u8]); 4] = [
    ("fcl_hair.fmdl", b"hair"),
    ("face_diff.bin", b"diff"),
    ("shirt.dds", b"shirt"),
    ("portrait.dds", b"portrait"),
];

/// An old export's face folder at `relative` with the four face files.
fn face(sandbox: &Sandbox, relative: &str) {
    for (name, bytes) in FACE_FILES {
        sandbox.write(&format!("{relative}/{name}"), bytes);
    }
}

// EU-PLY-01
#[test]
fn the_face_folders_become_the_players_folders() {
    let sandbox = Sandbox::new("ply_folders");
    face(&sandbox, "co Spring/Faces/XXX15 - Snuffy");
    face(&sandbox, "co Spring/Faces/71416 Rex");
    face(&sandbox, "co Spring/Faces/XXX17- Ann");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for player in [
        "Players/15 - Snuffy",
        "Players/16 - Rex",
        "Players/17 - Ann",
    ] {
        for (name, bytes) in FACE_FILES {
            assert_eq!(
                fs::read(sandbox.path(&format!("co Full Spring/{player}/{name}"))).unwrap(),
                bytes,
                "{player}/{name}"
            );
        }
    }
    assert!(
        !run.stdout().contains("unmigrated_content"),
        "{}",
        run.stdout()
    );
}

// EU-PLY-01
#[test]
fn a_doubled_faces_layer_is_flattened() {
    let sandbox = Sandbox::new("ply_nested_faces");
    face(&sandbox, "co Spring/Faces/Faces/XXX15 - Snuffy");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for (name, _) in FACE_FILES {
        assert!(
            sandbox
                .path(&format!("co Full Spring/Players/15 - Snuffy/{name}"))
                .exists(),
            "{name}: {}",
            run.stdout()
        );
    }
}

// EU-PLY-01
#[test]
fn a_doubled_layer_in_another_letter_case_is_flattened_in_a_zip() {
    let sandbox = Sandbox::new("ply_nested_faces_zip");
    // A `.zip` holding `co Spring/Faces/faces/XXX15 - Snuffy/`, the doubled
    // layer spelled in another letter case.
    let options = zip::write::SimpleFileOptions::default();
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    writer
        .add_directory("co Spring/Faces/faces/", options)
        .unwrap();
    for (name, bytes) in FACE_FILES {
        writer
            .start_file(
                format!("co Spring/Faces/faces/XXX15 - Snuffy/{name}"),
                options,
            )
            .unwrap();
        std::io::Write::write_all(&mut writer, bytes).unwrap();
    }
    let bytes = writer.finish().unwrap().into_inner();
    sandbox.write("co Spring.zip", &bytes);
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring.zip", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for (name, bytes) in FACE_FILES {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Players/15 - Snuffy/{name}"))).unwrap(),
            bytes,
            "{name}: {}",
            run.stdout()
        );
    }
}

// EU-PLY-07
#[test]
fn portraits_are_renamed_and_two_are_reported_as_named() {
    let sandbox = Sandbox::new("ply_portraits");
    sandbox.write(
        "co Spring/Faces/XXX15 - Snuffy/Snuffy_Portrait.dds",
        b"portrait1",
    );
    face(&sandbox, "co Spring/Faces/XXX16 - Rex");
    sandbox.write("co Spring/Faces/XXX16 - Rex/rex_portrait.dds", b"portrait2");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/15 - Snuffy/portrait.dds")).unwrap(),
        b"portrait1"
    );
    let stdout = run.stdout();
    assert!(
        stdout.contains("unmigrated_content at Faces/XXX16 - Rex/portrait.dds"),
        "{stdout}"
    );
    assert!(
        stdout.contains("unmigrated_content at Faces/XXX16 - Rex/rex_portrait.dds"),
        "{stdout}"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/16 - Rex/portrait.dds")).unwrap(),
        b"portrait"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/16 - Rex/rex_portrait.dds")).unwrap(),
        b"portrait2"
    );
    // The two portraits are the only unmigrated content; the event marks it.
    assert_eq!(unmigrated_found(&run), 1);
}

// EU-PLY-03
#[test]
fn a_nameless_folder_takes_the_saves_display_name_cleaned() {
    let sandbox = Sandbox::new("ply_save_name");
    face(&sandbox, "co Spring/Faces/XXX09");
    sandbox.save_edited("EDIT", |file| {
        // The colour-coded name of fixture player 78205: the `\x11c` code
        // strips, the `>` is not a folder-name character.
        file.player_mut(71409).unwrap().name = "\u{11}c789922ff>skin".to_owned();
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert!(
        sandbox.path("co Full Spring/Players/09 - skin").is_dir(),
        "{}",
        run.stdout()
    );
}

// EU-PLY-04
#[test]
fn either_ingame_face_spelling_becomes_one_bare_marker() {
    let sandbox = Sandbox::new("ply_ingame");
    face(&sandbox, "co Spring/Faces/XXX03 - A");
    sandbox.write("co Spring/Faces/XXX03 - A/ingame_face", b"");
    face(&sandbox, "co Spring/Faces/XXX04 - B");
    sandbox.write("co Spring/Faces/XXX04 - B/ingame_face.txt", b"ignored");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for player in ["Players/03 - A", "Players/04 - B"] {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/{player}/ingame_face"))).unwrap(),
            b"",
            "{player}"
        );
        assert!(
            !sandbox
                .path(&format!("co Full Spring/{player}/ingame_face.txt"))
                .exists(),
            "{player}"
        );
    }
    // The referee half of EU-PLY-04 is 6.6's: a referee export's face folders
    // stay unmigrated content in this slice.
}

// EU-PLY-11
#[test]
fn an_empty_roster_slot_is_a_warning_and_the_folder_still_forms() {
    let sandbox = Sandbox::new("ply_empty_slot");
    face(&sandbox, "co Spring/Faces/XXX23 - Pinto");
    sandbox.save_edited("EDIT", |file| {
        let team = file.team_mut(714).unwrap();
        let position = team
            .roster
            .iter()
            .position(|slot| slot.player_id == 71423)
            .unwrap();
        team.roster[position].player_id = 0;
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert!(
        run.stdout()
            .contains("- co Spring: Warning roster_slot_empty at Faces/XXX23 - Pinto (slot=23)"),
        "{}",
        run.stdout()
    );
    for (name, _) in FACE_FILES {
        assert!(
            sandbox
                .path(&format!("co Full Spring/Players/23 - Pinto/{name}"))
                .exists(),
            "{name}"
        );
    }
    assert!(
        !sandbox
            .path("co Full Spring/Players/23 - Pinto/settings.toml")
            .exists()
    );
    // `roster_slot_empty` is a warning, not unmigrated content: no event.
    assert_eq!(unmigrated_found(&run), 0);
}
