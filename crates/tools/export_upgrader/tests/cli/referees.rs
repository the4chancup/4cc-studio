//! Referee exports (the plan's "Referee exports" scenarios): `refs.txt`
//! rewritten as the root `players.txt`, `ref_marker.dds` carried, the referee
//! folders under `Players/` copied, and a `refs` export without a roster
//! refused with nothing written. The folder names are real referees'
//! (`arbiter`, `eiki`, from a real `refs` export).

use std::fs;

use crate::common::Sandbox;
use crate::players::face;

/// `upgrade <input>`, no flag: a referee export takes neither.
fn upgrade(sandbox: &Sandbox, input: &str) -> crate::common::Run {
    let input = sandbox.arg(input);
    sandbox.run(&["upgrade", &input])
}

// EU-REF-01
#[test]
fn refs_txt_becomes_the_canonical_players_txt_and_the_marker_is_carried() {
    let sandbox = Sandbox::new("ref_roster");
    // 35 slots, written unpadded as real exports write them, two folders
    // backing them all: the repeats are the rarity control.
    let folder = |n: u8| if n % 2 == 1 { "arbiter" } else { "eiki" };
    let refs_txt: String = (1..=35u8).map(|n| format!("{n} {}\n", folder(n))).collect();
    let expected: String = (1..=35u8)
        .map(|n| format!("{n:02} {}\n", folder(n)))
        .collect();
    assert_ne!(refs_txt, expected, "the input is not already canonical");
    sandbox.write("refs Spring/refs.txt", refs_txt.as_bytes());
    sandbox.write("refs Spring/ref_marker.dds", b"marker");
    sandbox.write("refs Spring/Players/arbiter/boots.fmdl", b"boots");
    sandbox.write("refs Spring/Players/arbiter/boots.skl", b"skl");
    sandbox.write("refs Spring/Players/arbiter/arbiter.dds", b"arbiter");
    sandbox.write("refs Spring/Players/eiki/boots.fmdl", b"boots");
    sandbox.write("refs Spring/Players/eiki/boots.skl", b"skl");

    let run = upgrade(&sandbox, "refs Spring");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        fs::read_to_string(sandbox.path("refs Spring upgraded/players.txt")).unwrap(),
        expected,
        "{}",
        run.stdout()
    );
    let listing = sandbox.listing("refs Spring upgraded");
    assert!(
        !listing
            .iter()
            .any(|path| path.to_ascii_lowercase().ends_with("refs.txt")),
        "{listing:?}"
    );
    assert_eq!(
        fs::read(sandbox.path("refs Spring upgraded/ref_marker.dds")).unwrap(),
        b"marker"
    );
    assert!(
        !run.stdout().contains("unmigrated_content"),
        "{}",
        run.stdout()
    );
}

// EU-REF-04
#[test]
fn a_subfolder_with_a_face_model_is_carried_in_both_kinds_of_player_folder() {
    let sandbox = Sandbox::new("ref_alt_subfolder");
    face(&sandbox, "co Spring/Faces/XXX07 - A");
    sandbox.write("co Spring/Faces/XXX07 - A/alt/face_high.fmdl", b"team alt");
    let save = sandbox.arg("EDIT");
    sandbox.save("EDIT");
    sandbox.write("refs Spring/refs.txt", b"01 arbiter\n");
    sandbox.write("refs Spring/Players/arbiter/boots.fmdl", b"boots");
    sandbox.write("refs Spring/Players/arbiter/alt/face_high.fmdl", b"ref alt");

    let input = sandbox.arg("co Spring");
    let team = sandbox.run(&["upgrade", &input, "--savefile", &save]);
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/07 - A/alt/face_high.fmdl")).unwrap(),
        b"team alt",
        "{}",
        team.stdout()
    );
    let referees = upgrade(&sandbox, "refs Spring");
    assert_eq!(
        fs::read(sandbox.path("refs Spring upgraded/Players/arbiter/alt/face_high.fmdl")).unwrap(),
        b"ref alt",
        "{}",
        referees.stdout()
    );
}

// EU-REF-07
#[test]
fn a_refs_export_without_refs_txt_is_reported_and_nothing_is_written() {
    let sandbox = Sandbox::new("ref_no_roster");
    sandbox.write("refs Summer/Players/arbiter/boots.fmdl", b"boots");
    let before = sandbox.listing("");

    let run = upgrade(&sandbox, "refs Summer");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    assert!(
        run.stdout().contains("refs_txt_missing"),
        "{}",
        run.stdout()
    );
    assert_eq!(sandbox.listing(""), before, "nothing is written");
}
