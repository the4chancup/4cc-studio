//! `diff`: the comparator's report over two saves.

use std::fs;

use pes_savefile::EditFile;
use pes_version::PesVersion;

use crate::{STRIPPED_APPEARANCE_ID, Sandbox, TEST_SALT};

/// A player id no save holds, so renumbering a player to it leaves him in one save alone.
const NOWHERE: u32 = 9_999_999;

impl Sandbox {
    /// `diff <first> <second> <flags>`.
    fn diff(&self, first: &str, second: &str, flags: &[&str]) -> crate::common::Run {
        let (first, second) = (self.arg(first), self.arg(second));
        let mut args = vec!["diff", &first, &second];
        args.extend(flags);
        self.run(&args)
    }

    /// Two PES 21 saves: `A` is the fixture with player 70110 renumbered to `NOWHERE`
    /// (so 70110 is in `B` alone and `NOWHERE` in `A` alone); `B` is the fixture with
    /// one player, 70105, carrying both changes: his boots id and his speed. The two
    /// changed values, old and new.
    fn diff_pair(&self) -> [(u32, u32); 2] {
        // The boots id is a `u32` field, the speed a `u8`; both land as `u32` for the lines.
        let mut a = EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        a.player_mut(70110).unwrap().id = NOWHERE;
        fs::write(self.path("A"), a.to_bytes(&TEST_SALT).unwrap()).unwrap();
        let mut b = EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let boots = &mut b.player_mut(70105).unwrap().appearance.boots_id;
        let boots_change = (*boots, *boots + 1);
        *boots += 1;
        let speed = &mut b.player_mut(70105).unwrap().stats.speed;
        let speed_change = (*speed, (*speed + 1) % 100);
        *speed = speed_change.1;
        fs::write(self.path("B"), b.to_bytes(&TEST_SALT).unwrap()).unwrap();
        [
            boots_change,
            (u32::from(speed_change.0), u32::from(speed_change.1)),
        ]
    }
}

// SE-DIF-01
#[test]
fn each_changed_field_is_a_row_and_the_scopes_keep_their_own() {
    let sandbox = Sandbox::new("diff_scopes");
    let [(boots_old, boots_new), (speed_old, speed_new)] = sandbox.diff_pair();
    let boots =
        format!("Info player_diff (player=70105, field=BootsId, old={boots_old}, new={boots_new})");
    // Both rows are 70105's; the comparator lists a player's fields in schema order, the
    // stat before the appearance field.
    let speed =
        format!("Info player_diff (player=70105, field=Speed, old={speed_old}, new={speed_new})");
    let only_in = [
        format!("Info player_only_in (player={NOWHERE}, save=first)"),
        "Info player_only_in (player=70110, save=second)".to_owned(),
    ];

    let run = sandbox.diff("A", "B", &[]);
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec![
            speed.clone(),
            boots.clone(),
            only_in[0].clone(),
            only_in[1].clone()
        ]
    );
    let run = sandbox.diff("A", "B", &["--aesthetics"]);
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec![boots, only_in[0].clone(), only_in[1].clone()]
    );
    let run = sandbox.diff("A", "B", &["--gameplay"]);
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec![speed, only_in[0].clone(), only_in[1].clone()]
    );
    sandbox
        .diff("A", "B", &["--aesthetics", "--gameplay"])
        .assert_refused(2, &["--gameplay"]);
}

// SE-DIF-02
#[test]
fn json_flag_writes_the_same_report_as_one_document() {
    let sandbox = Sandbox::new("diff_json");
    let [(boots_old, boots_new), (speed_old, speed_new)] = sandbox.diff_pair();

    let run = sandbox.diff("A", "B", &["--json"]);
    assert_eq!(run.exit_code(), 0);
    let stdout = run.stdout();
    let report: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("stdout does not parse as JSON: {error}\n{stdout}"));
    assert_eq!(
        report["players"],
        serde_json::json!([
            {
                "id": 70105,
                "stripped": null,
                "rows": [
                    { "field": "Speed", "old": speed_old.to_string(), "new": speed_new.to_string() },
                    { "field": "BootsId", "old": boots_old.to_string(), "new": boots_new.to_string() }
                ]
            },
        ])
    );
    assert_eq!(
        report["only_in_first"],
        serde_json::json!([{ "id": NOWHERE, "stripped": null }])
    );
    assert_eq!(
        report["only_in_second"],
        serde_json::json!([{ "id": 70110, "stripped": null }])
    );
    for line in stdout.lines() {
        assert!(
            !line.starts_with("- ") && !line.starts_with("Info "),
            "a finding line in the JSON output: {line}"
        );
    }
}

// SE-DIF-02
#[test]
fn json_report_keeps_the_scope() {
    let sandbox = Sandbox::new("diff_json_scope");
    let [(boots_old, boots_new), (speed_old, speed_new)] = sandbox.diff_pair();

    for (flag, field, old, new) in [
        ("--aesthetics", "BootsId", boots_old, boots_new),
        ("--gameplay", "Speed", speed_old, speed_new),
    ] {
        let run = sandbox.diff("A", "B", &["--json", flag]);
        assert_eq!(run.exit_code(), 0);
        let stdout = run.stdout();
        let report: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|error| panic!("stdout does not parse as JSON: {error}\n{stdout}"));
        assert_eq!(
            report["players"],
            serde_json::json!([{
                "id": 70105,
                "stripped": null,
                "rows": [{ "field": field, "old": old.to_string(), "new": new.to_string() }]
            }])
        );
        assert_eq!(
            report["only_in_first"],
            serde_json::json!([{ "id": NOWHERE, "stripped": null }])
        );
        assert_eq!(
            report["only_in_second"],
            serde_json::json!([{ "id": 70110, "stripped": null }])
        );
    }
}

// SE-DIF-03
#[test]
fn two_versions_are_refused_naming_both() {
    let sandbox = Sandbox::new("diff_versions");
    sandbox.fixture(PesVersion::Pes21, "EDIT21");
    sandbox.fixture(PesVersion::Pes19, "EDIT19");
    sandbox
        .diff("EDIT21", "EDIT19", &[])
        .assert_refused(2, &["PES 2021", "PES 2019"]);
}

// SE-DIF-04
#[test]
fn every_player_at_minus_one_is_marked_and_the_appearance_id_is_no_row() {
    let sandbox = Sandbox::new("diff_stripped");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    sandbox.stripped_fixture("STRIPPED");
    let players = EditFile::load(&sandbox.path("EDIT"))
        .unwrap()
        .players()
        .len();

    let run = sandbox.diff("STRIPPED", "EDIT", &[]);
    assert_eq!(run.exit_code(), 0);
    let messages = run.messages();
    assert_eq!(messages.len(), players, "one mark per player, nothing else");
    assert!(
        messages
            .iter()
            .all(|line| line.starts_with("Info player_stripped (player=")
                && line.ends_with(", save=first)")),
        "{:?}",
        &messages[..3]
    );
    assert_eq!(
        messages[0],
        "Info player_stripped (player=70101, save=first)"
    );

    let run = sandbox.diff("STRIPPED", "STRIPPED", &[]);
    assert_eq!(
        run.messages()[0],
        "Info player_stripped (player=70101, save=both)"
    );
}

// SE-DIF-04
#[test]
fn a_stripped_player_only_in_one_save_is_reported_once() {
    let sandbox = Sandbox::new("diff_stripped_only_in");
    let mut a = EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
    let stripped = a.player_mut(70110).unwrap();
    stripped.id = NOWHERE;
    stripped.appearance_id = Some(STRIPPED_APPEARANCE_ID);
    fs::write(sandbox.path("A"), a.to_bytes(&TEST_SALT).unwrap()).unwrap();
    sandbox.fixture(PesVersion::Pes21, "B");

    let run = sandbox.diff("A", "B", &[]);
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec![
            format!("Info player_only_in (player={NOWHERE}, save=first)"),
            format!("Info player_stripped (player={NOWHERE}, save=first)"),
            "Info player_only_in (player=70110, save=second)".to_owned(),
        ]
    );

    let run = sandbox.diff("A", "B", &["--json"]);
    assert_eq!(run.exit_code(), 0);
    let stdout = run.stdout();
    let report: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("stdout does not parse as JSON: {error}\n{stdout}"));
    assert_eq!(report["players"], serde_json::json!([]));
    assert_eq!(
        report["only_in_first"],
        serde_json::json!([{ "id": NOWHERE, "stripped": "first" }])
    );
    assert_eq!(
        report["only_in_second"],
        serde_json::json!([{ "id": 70110, "stripped": null }])
    );

    let run = sandbox.diff("B", "A", &[]);
    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        vec![
            "Info player_only_in (player=70110, save=first)".to_owned(),
            format!("Info player_only_in (player={NOWHERE}, save=second)"),
            format!("Info player_stripped (player={NOWHERE}, save=second)"),
        ]
    );
}
