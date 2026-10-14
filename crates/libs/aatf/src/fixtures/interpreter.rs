//! The generic interpreter's behavior on rulesets built for the test:
//! the paths no shipped ruleset's data exercises (the per-ruleset
//! fixtures cover the shipped files). The teams are the sample team or
//! the fixture saves' team 701, edited the smallest amount.

use super::*;
use crate::ruleset::Foot;

/// `text` assembled with the current logic.
fn rules(text: &str) -> Rules {
    Rules::from_data_block(text).expect("the data block loads")
}

/// The ability stat `name` names.
fn stat(name: &str) -> Stat {
    Stat::try_from(name.to_string()).expect("a stat name")
}

/// The sample team's view with `edit` applied to the player at `slot`.
fn view_edited(slot: u8, edit: impl FnOnce(&mut PlayerView)) -> TeamView {
    let mut team = TeamView::sample_team();
    edit(player_mut(&mut team, slot));
    team
}

/// The smallest ruleset one `forbid` special can probe a condition
/// through: every player is `regular`, so `when` alone decides whether
/// `label` is reported.
const PROBE: &str = "const RULESET = #{
    schema: 1, name: \"probe\", squad_size: 23,
    universal: #{ max_skill_cards: 10 },
    tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77 }],
    specials: [SPECIALS],
};";

/// `PROBE` with `specials` filled in.
fn probe(specials: &str) -> Rules {
    rules(&PROBE.replace("SPECIALS", specials))
}

/// A `forbid` probe: whether `when` holds for the player at `slot` shows
/// as a `Special` violation on him.
fn fires(rules: &Rules, team: &TeamView, slot: u8, label: &str) -> bool {
    problems(rules, team).iter().any(|v| {
        v.slot == Some(slot)
            && matches!(&v.kind, ViolationKind::Special { label: l, .. } if l == label)
    })
}

/// A special replacing `weak_foot` in part only (`weak_foot: #{ usage: 4 }`
/// keeps no `accuracy`) lists the usage suggestion without a `CheckError`:
/// the `suggestions` branch reads the limits through `num`, as the check does.
#[test]
fn a_partial_weak_foot_effect_reports_the_suggestion() {
    let rules = rules(
        "const RULESET = #{
            schema: 1, name: \"x\", squad_size: 23,
            universal: #{ max_skill_cards: 10 },
            tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77,
                        weak_foot: #{ usage: 2, accuracy: 2 } }],
            specials: [#{ label: \"four-four\", then: #{ weak_foot: #{ usage: 4 } } }],
            suggestions: [\"weak_foot\"],
        };",
    );
    let team = view_edited(3, |p| {
        p.weak_foot_usage = 2;
        p.weak_foot_accuracy = 2;
    });
    let found = rules.check_team(&team).expect("the check runs");
    assert!(
        found
            .iter()
            .any(|v| v.slot == Some(3) && v.kind == unused("weak_foot", 2, 4)),
        "{found:?}"
    );
}

/// `ranges`: each key's value within its inclusive range on the game's
/// scale — a stat through `player.stat`, the six `PlayerValue` names
/// through their accessors. A stat the version lacks fails its range.
#[test]
fn a_special_fires_through_ranges() {
    // A stat: slot 3's speed set to 90, in [90, 99], out at [91, 99].
    let team = view_edited(3, |p| p.abilities[usize::from(stat("speed").0)] = Some(90));
    let inside = probe(
        "#{ label: \"in\", when: #{ ranges: #{ speed: [90, 99] } }, then: #{ forbid: \"x\" } }",
    );
    assert!(fires(&inside, &team, 3, "in"));
    let below = probe(
        "#{ label: \"in\", when: #{ ranges: #{ speed: [91, 99] } }, then: #{ forbid: \"x\" } }",
    );
    assert!(!fires(&below, &team, 3, "in"));
    // `age` and `form` read the accessors, on the game's scale.
    let team = view_edited(3, |p| p.age = 20);
    let inside = probe(
        "#{ label: \"in\", when: #{ ranges: #{ age: [15, 20] } }, then: #{ forbid: \"x\" } }",
    );
    assert!(fires(&inside, &team, 3, "in"));
    let below = probe(
        "#{ label: \"in\", when: #{ ranges: #{ age: [15, 19] } }, then: #{ forbid: \"x\" } }",
    );
    assert!(!fires(&below, &team, 3, "in"));
    let team = view_edited(3, |p| p.form = 8);
    let inside =
        probe("#{ label: \"in\", when: #{ ranges: #{ form: [8, 8] } }, then: #{ forbid: \"x\" } }");
    assert!(fires(&inside, &team, 3, "in"));
    let below =
        probe("#{ label: \"in\", when: #{ ranges: #{ form: [1, 7] } }, then: #{ forbid: \"x\" } }");
    assert!(!fires(&below, &team, 3, "in"));
    // `physical_contact` is a stat the sample team's PES 15 lacks: `()`
    // fails the range for every player.
    let missing = probe(
        "#{ label: \"in\", when: #{ ranges: #{ physical_contact: [1, 99] } }, then: #{ forbid: \"x\" } }",
    );
    let found = problems(&missing, &TeamView::sample_team());
    assert!(
        !found
            .iter()
            .any(|v| matches!(v.kind, ViolationKind::Special { .. })),
        "{found:?}"
    );
}

/// `strong_foot` compares with `player.strong_foot`'s label; a player
/// whose byte is past 1 holds neither.
#[test]
fn a_special_fires_through_strong_foot() {
    let right =
        probe("#{ label: \"r\", when: #{ strong_foot: \"right\" }, then: #{ forbid: \"x\" } }");
    let left =
        probe("#{ label: \"l\", when: #{ strong_foot: \"left\" }, then: #{ forbid: \"x\" } }");
    let team = view_edited(3, |p| p.strong_foot = Some(Foot::Right));
    assert!(fires(&right, &team, 3, "r"));
    assert!(!fires(&left, &team, 3, "l"));
    let team = view_edited(3, |p| p.strong_foot = None);
    assert!(!fires(&right, &team, 3, "r"));
}

/// `any` holds when at least one entry does, and an additive effect of
/// its special applies once to a player who meets two of them.
#[test]
fn a_special_fires_through_any_and_applies_once() {
    let special = probe(
        "#{ label: \"any\", when: #{ any: [#{ captain: true }, #{ strong_foot: \"right\" }] },
           then: #{ forbid: \"x\" } }",
    );
    // Only the second entry holds: slot 3 is no captain (slot 10 is).
    let team = view_edited(3, |p| p.strong_foot = Some(Foot::Right));
    assert!(fires(&special, &team, 3, "any"));
    // Neither entry holds.
    let team = view_edited(3, |p| p.strong_foot = Some(Foot::Left));
    assert!(!fires(&special, &team, 3, "any"));
    // Both entries hold (a right-footed RB rated A there alone): the
    // `cards: 1` effect applies once — the limit rises by 1, not 2.
    let rules = rules(
        "const RULESET = #{
            schema: 1, name: \"x\", squad_size: 23,
            universal: #{ max_skill_cards: 10 },
            tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77,
                        cards: 3, a_positions: 1 }],
            specials: [#{ label: \"any\",
                when: #{ any: [#{ strong_foot: \"right\" }, #{ positions: [\"RB\"] }] },
                then: #{ cards: 1 } }],
        };",
    );
    let team = view_edited(3, |p| {
        assert_eq!(p.position.expect("a position").label(), "RB");
        assert_eq!(p.ratings.iter().filter(|r| **r == 2).count(), 1);
        p.strong_foot = Some(Foot::Right);
    });
    let limits = rules
        .card_limits(&team, team.player(3).expect("slot 3"), "regular")
        .expect("limits");
    assert_eq!(limits.limit, 4);
}

/// `form` and `injury_resistance` add to the tier's targets wherever
/// they are read: `tier_values`, the check and the suggestion.
#[test]
fn form_and_injury_resistance_effects_move_the_targets() {
    let rules = rules(
        "const RULESET = #{
            schema: 1, name: \"x\", squad_size: 23,
            universal: #{ max_skill_cards: 10 },
            tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77,
                        form: 4, injury_resistance: 1 }],
            specials: [#{ label: \"fit\", then: #{ form: 2, injury_resistance: 1 } }],
            suggestions: [\"injury_resistance\"],
        };",
    );
    // A regular at the tier's own form 4 and injury resistance 1 reads
    // targets of 6 and 2.
    let team = view_edited(3, |p| {
        p.form = 4;
        p.injury_resistance = 1;
    });
    let values = rules
        .tier_values(&team, team.player(3).expect("slot 3"), "regular")
        .expect("values");
    assert_eq!((values.form, values.injury_resistance), (6, 2));
    let found = problems(&rules, &team);
    assert!(
        found.iter().any(|v| v.slot == Some(3)
            && v.kind
                == ViolationKind::Form {
                    value: 4,
                    target: 6
                }),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|v| v.slot == Some(3)
                && v.kind == ViolationKind::InjuryResistance { value: 1, max: 2 }),
        "{found:?}"
    );
    // The suggestion reads the raised limit, not the tier's 1.
    assert!(
        rules
            .check_team(&team)
            .expect("the check runs")
            .iter()
            .any(|v| v.slot == Some(3) && v.kind == unused("injury_resistance", 1, 2)),
        "the suggestion sees the effect"
    );
}

/// A tier's `name_color`: its players' names must carry it, and a
/// player of a tier that sets none must carry none of the codes the
/// tiers set (the VTL11 rule). `apply_tier` writes or strips the
/// leading code so the checker accepts the result.
#[test]
fn a_tiers_name_color_is_checked_and_written() {
    let rules = rules(
        "const RULESET = #{
            schema: 1, name: \"x\", squad_size: 23,
            universal: #{ max_skill_cards: 10 },
            tiers: [
                #{ id: \"gold\", label: \"Gold\", counts_as: \"regular\",
                   when: #{ positions: [\"GK\"] },
                   rate: 77, name_color: \"cc9900ff\" },
                #{ id: \"silver\", label: \"Silver\", counts_as: \"regular\",
                   when: #{ positions: [\"CB\"] },
                   rate: 77, name_color: \"c0c0c0ff\" },
                #{ id: \"regular\", label: \"Regular\", rate: 77 },
            ],
        };",
    );
    let name_color_of = |team: &TeamView, slot: u8| {
        problems(&rules, team)
            .into_iter()
            .filter(|v| v.slot == Some(slot))
            .find_map(|v| match v.kind {
                ViolationKind::NameColor { .. } => Some(v.kind),
                _ => None,
            })
    };
    let name_colors = |team: &TeamView| {
        problems(&rules, team)
            .into_iter()
            .filter(|v| matches!(v.kind, ViolationKind::NameColor { .. }))
            .count()
    };
    // A gold (slot 0, registered GK — the sample holds a second at 11)
    // without a code, a regular (slot 3) carrying silver's, and a
    // regular (slot 4) carrying an unrelated one, which reports
    // nothing; the sample's CBs (8, 14, 17, 20) are the silvers.
    let mut team = TeamView::sample_team();
    player_mut(&mut team, 3).name_color = Some("c0c0c0ff".to_string());
    player_mut(&mut team, 4).name_color = Some("12345678".to_string());
    assert_eq!(
        name_color_of(&team, 0),
        Some(ViolationKind::NameColor {
            color: None,
            expected: Some("cc9900ff".to_string())
        })
    );
    assert_eq!(
        name_color_of(&team, 3),
        Some(ViolationKind::NameColor {
            color: Some("c0c0c0ff".to_string()),
            expected: None
        })
    );
    assert_eq!(name_color_of(&team, 4), None);
    for (slot, expected) in [
        (8, "c0c0c0ff"),
        (11, "cc9900ff"),
        (14, "c0c0c0ff"),
        (17, "c0c0c0ff"),
        (20, "c0c0c0ff"),
    ] {
        assert_eq!(
            name_color_of(&team, slot),
            Some(ViolationKind::NameColor {
                color: None,
                expected: Some(expected.to_string())
            }),
            "{slot}"
        );
    }
    // `apply_tier` then `check_team`: each medal name gains its tier's
    // code, the regular loses silver's, the unrelated code is kept —
    // and the checker accepts them all.
    let mut records = Records::sample();
    assert!(
        pes_savefile::model::names::name_color(&records.player_mut(0).name).is_none(),
        "the gold's name carries no leading code yet"
    );
    records.player_mut(3).name = "\u{11}cc0c0c0ffNAME".to_string();
    records.player_mut(4).name = "\u{11}c12345678NAME".to_string();
    for (slot, tier) in [
        (0u8, "gold"),
        (8, "silver"),
        (11, "gold"),
        (14, "silver"),
        (17, "silver"),
        (20, "silver"),
        (3, "regular"),
        (4, "regular"),
    ] {
        let view = records.view();
        let player = view.player(slot).expect("filled").clone();
        rules
            .apply_tier(&view, &player, tier, records.player_mut(slot))
            .expect("the tier applies");
    }
    assert_eq!(name_colors(&records.view()), 0);
    assert!(
        records.player_mut(0).name.starts_with("\u{11}ccc9900ff"),
        "{:?}",
        records.player_mut(0).name
    );
    assert!(
        records.player_mut(8).name.starts_with("\u{11}cc0c0c0ff"),
        "{:?}",
        records.player_mut(8).name
    );
    assert_eq!(records.player_mut(3).name, "NAME");
    assert_eq!(records.player_mut(4).name, "\u{11}c12345678NAME");
}
