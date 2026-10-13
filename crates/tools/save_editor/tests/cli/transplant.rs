//! `transplant`: players' looks copied from a donor save onto a base save.

use std::fs;

use pes_savefile::EditFile;
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_savefile::ops::transplant::transplant_player;
use pes_version::PesVersion;

use crate::{Sandbox, TEST_SALT, payload, payload_after};

impl Sandbox {
    /// `transplant <donor> --base <base> --out <out> --select <select>`.
    fn transplant(&self, donor: &str, base: &str, out: &str, select: &str) -> crate::common::Run {
        self.run(&[
            "transplant",
            &self.arg(donor),
            "--base",
            &self.arg(base),
            "--out",
            &self.arg(out),
            "--select",
            select,
        ])
    }

    /// Writes the PES 21 fixture at `relative` with the look of every player of `ids`
    /// changed (boots id, neck length, the face edit flag), so a transplant of any of them is
    /// visible.
    fn donor_fixture(&self, relative: &str, ids: &[u32]) {
        let mut file =
            EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        for &id in ids {
            let player = file.player_mut(id).unwrap();
            player.appearance.boots_id += 1;
            player.appearance.neck_length = (player.appearance.neck_length + 1) % 15;
            player.edit_flags.face = !player.edit_flags.face;
        }
        fs::write(self.path(relative), file.to_bytes(&TEST_SALT).unwrap()).unwrap();
    }
}

/// The ids of team `team`'s 23 slots.
fn team(team: u32) -> Vec<u32> {
    (1..=23).map(|slot| team * 100 + slot).collect()
}

// SE-TRN-01
#[test]
fn each_selection_form_copies_its_targets_look_and_nothing_else() {
    let sandbox = Sandbox::new("transplant_forms");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    let mut donors = vec![70105, 70207];
    donors.extend(team(703));
    donors.extend(team(704));
    donors.extend(team(705));
    sandbox.donor_fixture("DONOR", &donors);
    // The pairs the four forms expand to: 70105 onto itself, 70107 from 70207, 703 and
    // 704-705 slot by slot.
    let mut pairs = vec![(70105, 70105), (70107, 70207)];
    pairs.extend(team(703).into_iter().map(|id| (id, id)));
    pairs.extend(team(704).into_iter().map(|id| (id, id)));
    pairs.extend(team(705).into_iter().map(|id| (id, id)));

    let run = sandbox.transplant("DONOR", "EDIT", "OUT", "70105,70107:70207,703,704-705");
    assert_eq!(run.exit_code(), 0);
    assert!(run.messages().is_empty(), "{:?}", run.messages());

    let donor = EditFile::load(&sandbox.path("DONOR")).unwrap();
    let expected = payload_after(&sandbox.path("EDIT"), |file| {
        for &(target, source) in &pairs {
            transplant_player(
                file.player_mut(target).unwrap(),
                donor.player(source).unwrap(),
            );
        }
    });
    assert_ne!(
        expected,
        payload(&sandbox.path("EDIT")),
        "the transplant changes the save"
    );
    assert_eq!(payload(&sandbox.path("OUT")), expected);
    let written = EditFile::load(&sandbox.path("OUT")).unwrap();
    for &(target, source) in &pairs {
        let (moved, from) = (
            written.player(target).unwrap(),
            donor.player(source).unwrap(),
        );
        assert_eq!(moved.appearance, from.appearance, "{target}");
        assert_eq!(moved.edit_flags.face, from.edit_flags.face, "{target}");
        assert_eq!(moved.appearance_id, Some(target), "{target}: his own id");
    }
}

// SE-TRN-02
#[test]
fn a_version_pair_or_a_player_a_save_lacks_is_refused_with_nothing_written() {
    let sandbox = Sandbox::new("transplant_refusals");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    sandbox.fixture(PesVersion::Pes19, "EDIT19");
    sandbox.donor_fixture("DONOR", &[70105]);
    sandbox
        .transplant("EDIT19", "EDIT", "OUT", "70105")
        .assert_refused(2, &["PES 2021", "PES 2019"]);
    sandbox
        .transplant("DONOR", "EDIT", "OUT", "70105,70106:999999")
        .assert_refused(2, &["donor", "999999"]);
    sandbox
        .transplant("DONOR", "EDIT", "OUT", "999999:70105")
        .assert_refused(2, &["target", "999999"]);
    sandbox
        .transplant("DONOR", "EDIT", "OUT", "70105,abc")
        .assert_refused(2, &["abc"]);
    assert!(!sandbox.path("OUT").exists());
}

// SE-TRN-03
#[test]
fn a_stripped_target_or_donor_is_marked_and_the_targets_id_stays() {
    let sandbox = Sandbox::new("transplant_stripped");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    sandbox.stripped_fixture("STRIPPED");
    sandbox.donor_fixture("DONOR", &[70105]);
    let donor = EditFile::load(&sandbox.path("DONOR")).unwrap();

    let run = sandbox.transplant("DONOR", "STRIPPED", "OUT1", "70105");
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec!["Warning transplant_target_stripped (player=70105)"]
    );
    let written = EditFile::load(&sandbox.path("OUT1")).unwrap();
    let moved = written.player(70105).unwrap();
    assert_eq!(moved.appearance, donor.player(70105).unwrap().appearance);
    assert_eq!(moved.appearance_id, Some(STRIPPED_APPEARANCE_ID));

    let run = sandbox.transplant("STRIPPED", "EDIT", "OUT2", "70105");
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec!["Warning transplant_donor_stripped (player=70105)"]
    );
    let written = EditFile::load(&sandbox.path("OUT2")).unwrap();
    assert_eq!(written.player(70105).unwrap().appearance_id, Some(70105));
}
