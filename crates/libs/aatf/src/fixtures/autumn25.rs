//! Autumn 25's fixtures: Spring 26 with two silvers and goalkeepers at
//! 77, so the sample team (three silvers, goalkeepers at 74) breaks
//! exactly those two values.

use super::*;
use crate::ruleset::{Card, Position};
use pes_savefile::interchange::team_toml::labels::PLAYABLE_POSITIONS;

/// The Autumn 25 data block (the 2025 Autumn and 2026 Winter cups), a
/// test ruleset.
const AUTUMN25: &str = include_str!("../../rulesets/autumn25.rhai");

/// The card `name` names.
fn card(name: &str) -> Card {
    Card::try_from(name.to_string()).expect("a card name")
}

/// The position `name` names.
fn position(name: &str) -> Position {
    Position::try_from(name.to_string()).expect("a position name")
}

/// The index in `entry`'s `playable` table (the save's order, not the
/// position table's) of position `name`.
fn playable_at(name: &str) -> usize {
    PLAYABLE_POSITIONS
        .iter()
        .position(|index| *index == usize::from(position(name).0))
        .expect("a position")
}

/// The sample team's records made legal under Autumn 25, the same edits
/// `the_sample_team_made_legal_reports_nothing` makes on the view: the
/// goalkeepers at the regulars' 77, and the third silver (slot 20) a
/// regular — weak foot accuracy 2, one COM style and one A position
/// dropped.
fn legal_records() -> Records {
    let mut records = Records::sample();
    set_tier_record(records.player_mut(0), 77, 4, 1);
    set_tier_record(records.player_mut(11), 77, 4, 1);
    let entry = records.player_mut(20);
    set_tier_record(entry, 77, 4, 1);
    entry.stats.weak_foot_accuracy = 1;
    let Card::ComStyle(long_ball_expert) = card("long_ball_expert") else {
        unreachable!("a COM style name")
    };
    entry.skills.com_styles[usize::from(long_ball_expert)] = false;
    entry.positions.playable[playable_at("DMF")] = 0;
    records
}

/// The sample team under Autumn 25: a silver too many (so a regular
/// short), and each goalkeeper at 74 reads as a regular (no tier's 77
/// fits, the last tier whose condition holds is the regular) with every
/// rating stat off 77.
#[test]
fn the_sample_team_breaks_the_two_values_that_differ() {
    let rules = Rules::from_data_block(AUTUMN25).expect("Autumn 25 loads");
    assert_eq!(rules.ruleset().name, "Autumn 25");
    let team = TeamView::sample_team();
    let found = problems(&rules, &team);
    for (tier, count, expected) in [("silver", 3, 2), ("regular", 18, 19)] {
        let off = error(
            None,
            ViolationKind::TierCount {
                tier: tier.to_string(),
                count,
                expected,
            },
        );
        assert!(found.contains(&off), "{tier}: {found:?}");
    }
    let goalkeeper_stats: Vec<&Violation> = found
        .iter()
        .filter(|v| {
            matches!(
                v.kind,
                ViolationKind::Stat {
                    value: 74,
                    target: 77,
                    at_most: false,
                    ..
                }
            ) && (v.slot == Some(0) || v.slot == Some(11))
        })
        .collect();
    assert_eq!(goalkeeper_stats.len(), 34, "{found:?}");
    assert_eq!(found.len(), 36, "{found:?}");
}

/// Both goalkeepers at 77 and one silver (slot 20, a CB with two A
/// positions, three skills and a COM style) made a regular, down to a
/// regular's weak foot 2/2, one A position and three paid cards: legal.
#[test]
fn the_sample_team_made_legal_reports_nothing() {
    let rules = Rules::from_data_block(AUTUMN25).expect("Autumn 25 loads");
    let mut team = TeamView::sample_team();
    set_tier(player_mut(&mut team, 0), 77, 4, 1);
    set_tier(player_mut(&mut team, 11), 77, 4, 1);
    let silver = player_mut(&mut team, 20);
    set_tier(silver, 77, 4, 1);
    assert_eq!(silver.weak_foot_accuracy, 4);
    silver.weak_foot_accuracy = 2;
    assert_eq!(silver.cards.len(), 4);
    silver.cards.pop();
    assert_eq!(silver.ratings.iter().filter(|r| **r == 2).count(), 2);
    let dmf = Position::try_from("DMF".to_string()).expect("a position");
    assert_eq!(silver.ratings[usize::from(dmf.0)], 2);
    silver.ratings[usize::from(dmf.0)] = 0;
    assert_eq!(problems(&rules, &team), vec![]);
}

/// For every tier of the file, a player of that tier in the made-legal
/// team — his stats, form, injury resistance and weak foot broken — is
/// legal again after `apply_tier`.
#[test]
fn apply_tier_then_check_reports_nothing_for_every_tier() {
    let rules = Rules::from_data_block(AUTUMN25).expect("Autumn 25 loads");
    let tiers: Vec<&str> = rules
        .ruleset()
        .tiers
        .iter()
        .map(|tier| tier.id.as_str())
        .collect();
    assert_eq!(tiers, ["gold", "silver", "goalkeeper", "regular"]);
    for (tier, slot) in [
        ("gold", 2),
        ("silver", 5),
        ("goalkeeper", 0),
        ("regular", 12),
    ] {
        let mut records = legal_records();
        let entry = records.player_mut(slot);
        set_tier_record(entry, 55, 2, 3);
        entry.stats.weak_foot_usage = 0;
        entry.stats.weak_foot_accuracy = 0;
        let broken = records.view();
        assert!(!problems(&rules, &broken).is_empty(), "{tier} {slot}");
        let player = broken.player(slot).expect("filled").clone();
        rules
            .apply_tier(&broken, &player, tier, records.player_mut(slot))
            .expect("the tier applies");
        assert_eq!(problems(&rules, &records.view()), vec![], "{tier} {slot}");
    }
}
