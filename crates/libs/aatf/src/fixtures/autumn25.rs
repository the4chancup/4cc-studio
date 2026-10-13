//! Autumn 25's fixtures: Spring 26 with two silvers and goalkeepers at
//! 77, so the sample team (three silvers, goalkeepers at 74) breaks
//! exactly those two values.

use super::*;
use crate::ruleset::Position;

/// The Autumn 25 data block (the 2025 Autumn and 2026 Winter cups), a
/// test ruleset.
const AUTUMN25: &str = include_str!("../../rulesets/autumn25.rhai");

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
