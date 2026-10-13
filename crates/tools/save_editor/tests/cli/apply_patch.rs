//! `apply-patch`: the Team compiler's aesthetics patch applied to a save.

use std::fs;

use pes_savefile::EditFile;
use pes_savefile::interchange::aesthetics_patch::{ALLOCATION_SCHEME_VERSION, AestheticsPatch};
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_version::PesVersion;

use crate::{Sandbox, payload, payload_after};

/// Team `/co/`, 714 in the PES 21 fixture, whose slot 3 is player 71403.
const CO: u32 = 714;
const CO_SLOT_3: u32 = 71403;

/// A patch text for `version` under `scheme`, holding `teams` (`[[teams]]` blocks).
fn patch_text(version: u16, scheme: u32, teams: &str) -> String {
    format!(
        "pes_version = {version}\nstudio_version = \"0.1.0\"\nallocation_scheme_version = {scheme}\ncpks = []\ncompiled = 2026-10-13T00:00:00Z\n{teams}"
    )
}

/// Team `/co/`'s block: slot 03 renamed, with two motions.
fn co_block() -> String {
    format!(
        "\n[[teams]]\nid = {CO}\nname = \"/co/\"\n\n[teams.players.03]\nname = \"PATCHED\"\nshirt_name = \"PATCHED\"\n\n[teams.players.03.appearance.motion]\nhunching_dribbling = 2\ncorner_kick = 3\n"
    )
}

impl Sandbox {
    /// `apply-patch <file> --base <base> --out <out>`.
    fn apply(&self, file: &str, base: &str, out: &str) -> crate::common::Run {
        self.run(&[
            "apply-patch",
            &self.arg(file),
            "--base",
            &self.arg(base),
            "--out",
            &self.arg(out),
        ])
    }
}

/// The payload `base` has after `patch` is applied through the lib.
fn payload_with_patch(sandbox: &Sandbox, base: &str, patch: &str) -> Vec<u8> {
    let patch = AestheticsPatch::parse(patch).unwrap();
    payload_after(&sandbox.path(base), |file| {
        patch.apply(file).unwrap();
    })
}

// SE-PAT-01
#[test]
fn a_patch_writes_its_teams_motions_and_names_and_nothing_else() {
    let sandbox = Sandbox::new("patch_one_team");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    let before = EditFile::load(&sandbox.path("EDIT")).unwrap();
    assert_ne!(before.player(CO_SLOT_3).unwrap().name, "PATCHED");
    let text = patch_text(21, ALLOCATION_SCHEME_VERSION, &co_block());
    fs::write(sandbox.path("patch.toml"), &text).unwrap();

    let run = sandbox.apply("patch.toml", "EDIT", "OUT");
    assert_eq!(run.exit_code(), 0);
    assert_eq!(run.messages(), vec!["Info patch_team_applied (team=714)"]);
    let expected = payload_with_patch(&sandbox, "EDIT", &text);
    assert_ne!(
        expected,
        payload(&sandbox.path("EDIT")),
        "the patch changes the save"
    );
    assert_eq!(payload(&sandbox.path("OUT")), expected);
    let written = EditFile::load(&sandbox.path("OUT")).unwrap();
    let player = written.player(CO_SLOT_3).unwrap();
    assert_eq!(player.name, "PATCHED");
    assert_eq!(player.shirt_name, "PATCHED");
    assert_ne!(
        player.motion.hunching_dribbling,
        before.player(CO_SLOT_3).unwrap().motion.hunching_dribbling
    );
}

// SE-PAT-02
#[test]
fn a_team_the_save_lacks_is_an_error_and_the_rest_is_written() {
    let sandbox = Sandbox::new("patch_missing_team");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    let teams = format!(
        "{}\n[[teams]]\nid = 999\nname = \"/zz/\"\n\n[teams.players.01]\nname = \"NOBODY\"\n",
        co_block()
    );
    let text = patch_text(21, ALLOCATION_SCHEME_VERSION, &teams);
    fs::write(sandbox.path("patch.toml"), &text).unwrap();

    let run = sandbox.apply("patch.toml", "EDIT", "OUT");
    assert_eq!(run.exit_code(), 1);
    assert_eq!(
        run.messages(),
        vec![
            "Info patch_team_applied (team=714)",
            "Error patch_team_not_in_save (team=999)"
        ]
    );
    assert_eq!(
        payload(&sandbox.path("OUT")),
        payload_with_patch(&sandbox, "EDIT", &text)
    );
    let written = EditFile::load(&sandbox.path("OUT")).unwrap();
    assert_eq!(written.player(CO_SLOT_3).unwrap().name, "PATCHED");
}

// SE-PAT-03
#[test]
fn a_version_or_scheme_mismatch_is_refused_with_nothing_written() {
    let sandbox = Sandbox::new("patch_mismatch");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    fs::write(
        sandbox.path("version.toml"),
        patch_text(19, ALLOCATION_SCHEME_VERSION, &co_block()),
    )
    .unwrap();
    sandbox
        .apply("version.toml", "EDIT", "OUT")
        .assert_refused(2, &["19", "21"]);
    fs::write(
        sandbox.path("scheme.toml"),
        patch_text(21, ALLOCATION_SCHEME_VERSION + 1, &co_block()),
    )
    .unwrap();
    sandbox.apply("scheme.toml", "EDIT", "OUT").assert_refused(
        2,
        &[
            "allocation scheme",
            &(ALLOCATION_SCHEME_VERSION + 1).to_string(),
        ],
    );
    assert!(!sandbox.path("OUT").exists());
}

// SE-PAT-04
#[test]
fn a_stripped_save_takes_the_patch_with_its_ids_still_stripped() {
    let sandbox = Sandbox::new("patch_stripped");
    sandbox.stripped_fixture("STRIPPED");
    let text = patch_text(21, ALLOCATION_SCHEME_VERSION, &co_block());
    fs::write(sandbox.path("patch.toml"), &text).unwrap();

    let run = sandbox.apply("patch.toml", "STRIPPED", "OUT");
    assert_eq!(run.exit_code(), 0, "{:?}", run.messages());
    let written = EditFile::load(&sandbox.path("OUT")).unwrap();
    let player = written.player(CO_SLOT_3).unwrap();
    assert_eq!(player.name, "PATCHED");
    assert_eq!(player.appearance_id, Some(STRIPPED_APPEARANCE_ID));
    assert!(
        written
            .players()
            .iter()
            .all(|player| player.appearance_id == Some(STRIPPED_APPEARANCE_ID)),
        "every id is still -1"
    );
    assert_eq!(
        payload(&sandbox.path("OUT")),
        payload_with_patch(&sandbox, "STRIPPED", &text)
    );
}
