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
pub(crate) fn face(sandbox: &Sandbox, relative: &str) {
    for (name, bytes) in FACE_FILES {
        sandbox.write(&format!("{relative}/{name}"), bytes);
    }
}

/// The three files of the real tracer Fox boots folder, a few bytes each;
/// `shirt.dds` carries the face's bytes so only a scenario's own files decide
/// a merge's byte clash.
const BOOTS_FILES: [(&str, &[u8]); 3] = [
    ("boots.fmdl", b"boots"),
    ("boots.skl", b"skl"),
    ("shirt.dds", b"shirt"),
];

/// An old export's boots folder at `relative`.
pub(crate) fn boots(sandbox: &Sandbox, relative: &str) {
    for (name, bytes) in BOOTS_FILES {
        sandbox.write(&format!("{relative}/{name}"), bytes);
    }
}

/// A gloves folder at `relative` (`glove_l.fmdl`, `glove_r.fmdl`).
pub(crate) fn gloves(sandbox: &Sandbox, relative: &str) {
    sandbox.write(&format!("{relative}/glove_l.fmdl"), b"glove_l");
    sandbox.write(&format!("{relative}/glove_r.fmdl"), b"glove_r");
}

/// Sets `id`'s `boots_id` and `gloves_id` on its appearance record.
pub(crate) fn wears(file: &mut pes_savefile::EditFile, id: u32, boots_id: u32, gloves_id: u32) {
    let player = file.player_mut(id).unwrap();
    player.appearance.boots_id = boots_id;
    player.appearance.gloves_id = gloves_id;
}

// EU-PLY-02
#[test]
fn a_wearer_without_a_face_folder_gets_one_with_ingame_face() {
    let sandbox = Sandbox::new("ply_stray_boots");
    boots(&sandbox, "co Spring/Boots/k0430 - Crocs");
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
        wears(file, 71405, 430, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for (name, bytes) in BOOTS_FILES {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Players/05 - Crocs/{name}"))).unwrap(),
            bytes,
            "{name}"
        );
    }
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/05 - Crocs/ingame_face")).unwrap(),
        b""
    );
    assert!(!sandbox.path("co Full Spring/Boots").exists());
}

// EU-PLY-05
#[test]
fn folders_worn_by_two_share_the_folder_with_links() {
    let sandbox = Sandbox::new("ply_shared");
    for slot in ["XXX03 - A", "XXX04 - B", "XXX05 - C", "XXX06 - D"] {
        face(&sandbox, &format!("co Spring/Faces/{slot}"));
    }
    boots(&sandbox, "co Spring/Boots/k0430 - Crocs");
    boots(&sandbox, "co Spring/Boots/k0431");
    gloves(&sandbox, "co Spring/Gloves/g0432 - Mitts");
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
        wears(file, 71403, 430, 432);
        wears(file, 71404, 431, 432);
        wears(file, 71405, 430, 0);
        wears(file, 71406, 431, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for (name, bytes) in BOOTS_FILES {
        for folder in ["Boots/Crocs", "Boots/k0431"] {
            assert_eq!(
                fs::read(sandbox.path(&format!("co Full Spring/{folder}/{name}"))).unwrap(),
                bytes,
                "{folder}/{name}"
            );
        }
    }
    for name in ["glove_l.fmdl", "glove_r.fmdl"] {
        assert!(
            sandbox
                .path(&format!("co Full Spring/Gloves/Mitts/{name}"))
                .exists()
        );
    }
    for (player, links) in [
        ("03 - A", vec!["Crocs.boots", "Mitts.gloves"]),
        ("05 - C", vec!["Crocs.boots"]),
        ("04 - B", vec!["k0431.boots", "Mitts.gloves"]),
        ("06 - D", vec!["k0431.boots"]),
    ] {
        for link in links {
            assert_eq!(
                fs::read(sandbox.path(&format!("co Full Spring/Players/{player}/{link}"))).unwrap(),
                b"",
                "{player}/{link}"
            );
        }
        // No player folder holds a copy of a model of those folders.
        for file in ["boots.fmdl", "boots.skl", "glove_l.fmdl", "glove_r.fmdl"] {
            assert!(
                !sandbox
                    .path(&format!("co Full Spring/Players/{player}/{file}"))
                    .exists(),
                "{player}/{file}"
            );
        }
    }
}

// EU-PLY-06
#[test]
fn a_folder_nobody_wears_is_unmigrated_as_a_whole() {
    let sandbox = Sandbox::new("ply_unworn");
    face(&sandbox, "co Spring/Faces/XXX03 - A");
    gloves(&sandbox, "co Spring/Gloves/g0432 - Mitts");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1);
    assert!(
        run.stdout()
            .contains("- co Spring: Error unmigrated_content at Gloves/g0432 - Mitts\n"),
        "{}",
        run.stdout()
    );
    assert!(!sandbox.path("co Full Spring/Gloves").exists());
    assert_eq!(
        sandbox
            .listing("co Full Spring")
            .iter()
            .filter(|path| path.contains("glove_"))
            .count(),
        0
    );
}

// EU-PLY-08
#[test]
fn pre_fox_never_merges_the_alone_worn_folder() {
    let sandbox = Sandbox::new("ply_prefox");
    face(&sandbox, "co Spring/Faces/XXX05 - Bob");
    sandbox.write("co Spring/Faces/XXX05 - Bob/face.xml", b"<config/>");
    sandbox.write(
        "co Spring/Faces/XXX05 - Bob/face_high_win32.model",
        b"face_high",
    );
    boots(&sandbox, "co Spring/Boots/k0430 - Crocs");
    sandbox.write(
        "co Spring/Boots/k0430 - Crocs/oral_boots_win32.model",
        b"oral",
    );
    sandbox.write("co Spring/Boots/k0430 - Crocs/boots.mtl", b"mtl");
    sandbox.write("co Spring/Boots/k0430 - Crocs/k2012_c.dds", b"tex");
    sandbox.save_edited("EDIT16", pes_version::PesVersion::Pes16, |file| {
        wears(file, 71405, 430, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT16");
    let report = run.stdout();
    assert_eq!(run.exit_code(), 0, "{report}");
    // The boots model loses its `oral_`/`_win32` wrappers (step 7).
    for (input, output) in [
        ("oral_boots_win32.model", "boots.model"),
        ("boots.mtl", "boots.mtl"),
        ("k2012_c.dds", "k2012_c.dds"),
    ] {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Boots/Crocs/{output}"))).unwrap(),
            fs::read(sandbox.path(&format!("co Spring/Boots/k0430 - Crocs/{input}"))).unwrap(),
            "{output}"
        );
    }
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/05 - Bob/Crocs.boots")).unwrap(),
        b""
    );
    for name in ["boots.model", "boots.mtl", "k2012_c.dds"] {
        assert!(
            !sandbox
                .path(&format!("co Full Spring/Players/05 - Bob/{name}"))
                .exists(),
            "{name}"
        );
    }
}

// EU-PLY-09
#[test]
fn a_byte_clash_keeps_the_folder_shared_and_their_own_file() {
    let sandbox = Sandbox::new("ply_clash");
    face(&sandbox, "co Spring/Faces/XXX05 - A");
    sandbox.write("co Spring/Faces/XXX05 - A/skin.dds", b"mine");
    face(&sandbox, "co Spring/Faces/XXX06 - B");
    sandbox.write("co Spring/Faces/XXX06 - B/skin.dds", b"shared");
    boots(&sandbox, "co Spring/Boots/k0430 - Crocs");
    sandbox.write("co Spring/Boots/k0430 - Crocs/skin.dds", b"theirs");
    boots(&sandbox, "co Spring/Boots/k0431");
    sandbox.write("co Spring/Boots/k0431/skin.dds", b"shared");
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
        wears(file, 71405, 430, 0);
        wears(file, 71406, 431, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert!(
        run.stdout()
            .contains("- co Spring: Info merge_kept_shared at Boots/k0430 - Crocs (path=skin.dds)"),
        "{}",
        run.stdout()
    );
    // The shared folder holds its own files; 05 keeps his face's skin.dds.
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Boots/Crocs/skin.dds")).unwrap(),
        b"theirs"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Boots/Crocs/boots.fmdl")).unwrap(),
        b"boots"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/05 - A/skin.dds")).unwrap(),
        b"mine"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/05 - A/Crocs.boots")).unwrap(),
        b""
    );
    // 06 merges: one skin.dds, the face's bytes being equal to the item's.
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/06 - B/boots.fmdl")).unwrap(),
        b"boots"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/06 - B/skin.dds")).unwrap(),
        b"shared"
    );
    assert!(!sandbox.path("co Full Spring/Boots/k0431").exists());
}

// EU-PLY-10
#[test]
fn second_folders_of_a_slot_and_of_an_id_are_unmigrated() {
    let sandbox = Sandbox::new("ply_second");
    face(&sandbox, "co Spring/Faces/XXX23 - Bahsaka");
    face(&sandbox, "co Spring/Faces/XXX23 - Smug Redman");
    boots(&sandbox, "co Spring/Boots/k0430 - A");
    boots(&sandbox, "co Spring/Boots/k0430 - B");
    // The losing folders carry distinct bytes, so a copy of them would be
    // caught, not only the winners' presence seen.
    sandbox.write(
        "co Spring/Faces/XXX23 - Smug Redman/face_diff.bin",
        b"loser face",
    );
    sandbox.write("co Spring/Boots/k0430 - B/boots.fmdl", b"loser boots");
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
        wears(file, 71423, 430, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1);
    for name in [
        "fcl_hair.fmdl",
        "face_diff.bin",
        "shirt.dds",
        "portrait.dds",
    ] {
        assert!(
            sandbox
                .path(&format!("co Full Spring/Players/23 - Bahsaka/{name}"))
                .exists(),
            "{name}"
        );
    }
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/23 - Bahsaka/face_diff.bin")).unwrap(),
        b"diff"
    );
    for (name, bytes) in BOOTS_FILES {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Players/23 - Bahsaka/{name}"))).unwrap(),
            bytes,
            "{name}"
        );
    }
    let stdout = run.stdout();
    assert!(
        stdout.contains("unmigrated_content at Faces/XXX23 - Smug Redman"),
        "{stdout}"
    );
    assert!(
        stdout.contains("unmigrated_content at Boots/k0430 - B"),
        "{stdout}"
    );
    // The losing player folder is absent, and no output file carries a
    // losing folder's bytes.
    assert!(
        !sandbox
            .path("co Full Spring/Players/23 - Smug Redman")
            .exists()
    );
    for entry in sandbox.listing("co Full Spring") {
        let path = sandbox.path(&format!("co Full Spring/{entry}"));
        if path.is_file() {
            let bytes = fs::read(&path).unwrap();
            assert!(bytes != b"loser face" && bytes != b"loser boots", "{entry}");
        }
    }
}

// EU-PLY-12
#[test]
fn identical_shared_folders_merge_as_one() {
    let sandbox = Sandbox::new("ply_identical_shared");
    for slot in [
        "XXX03 - A",
        "XXX04 - B",
        "XXX05 - C",
        "XXX06 - D",
        "XXX07 - E",
        "XXX08 - F",
    ] {
        face(&sandbox, &format!("co Spring/Faces/{slot}"));
    }
    boots(&sandbox, "co Spring/Boots/k0430 - Crocs");
    boots(&sandbox, "co Spring/Boots/k0431 - Crocs");
    boots(&sandbox, "co Spring/Boots/k0432 - crocs");
    // `k0432 - crocs` differs from the two identical ones in its model alone.
    sandbox.write("co Spring/Boots/k0432 - crocs/boots.fmdl", b"other model");
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
        wears(file, 71403, 430, 0);
        wears(file, 71404, 431, 0);
        wears(file, 71405, 430, 0);
        wears(file, 71406, 431, 0);
        wears(file, 71407, 432, 0);
        wears(file, 71408, 432, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for (name, bytes) in BOOTS_FILES {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Boots/Crocs/{name}"))).unwrap(),
            bytes,
            "{name}"
        );
    }
    assert!(!sandbox.path("co Full Spring/Boots/k0431").exists());
    for (name, bytes) in [
        ("boots.fmdl", b"other model" as &[u8]),
        ("boots.skl", b"skl"),
        ("shirt.dds", b"shirt"),
    ] {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Boots/k0432/{name}"))).unwrap(),
            bytes,
            "{name}"
        );
    }
    for (player, link) in [
        ("03 - A", "Crocs.boots"),
        ("04 - B", "Crocs.boots"),
        ("05 - C", "Crocs.boots"),
        ("06 - D", "Crocs.boots"),
        ("07 - E", "k0432.boots"),
        ("08 - F", "k0432.boots"),
    ] {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Spring/Players/{player}/{link}"))).unwrap(),
            b"",
            "{player}/{link}"
        );
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
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
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
    sandbox.save_edited("EDIT", pes_version::PesVersion::Pes21, |file| {
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
