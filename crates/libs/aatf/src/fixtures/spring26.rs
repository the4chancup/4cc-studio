//! Spring 26's fixtures: the sample team is a real team built under this
//! ruleset (team 714 of the 2026 Summer Cup save), so it is the
//! known-legal team as it stands.

use super::*;

/// The Spring 26 data block (the ruleset of the 2026 Spring and Summer
/// cups), a template.
const SPRING26: &str = include_str!("../../rulesets/spring26.rhai");

/// The sample team as written reports nothing: two golds at 99, three
/// silvers at 88, goalkeepers at 74, Green with 5 giants.
#[test]
fn the_sample_team_is_legal_under_the_ruleset_it_was_built_for() {
    let rules = Rules::from_data_block(SPRING26).expect("Spring 26 loads");
    assert_eq!(rules.ruleset().name, "Spring 26");
    let team = TeamView::sample_team();
    assert_eq!(player(&team, 0).abilities[2], Some(74));
    assert_eq!(problems(&rules, &team), vec![]);
}

/// The Spring 26 values the official file changed: a silver at 92 is off
/// every rating stat's target (88), and a medal's weak foot usage 3 is
/// over 2.
#[test]
fn a_silver_at_the_official_rate_and_a_medals_weak_foot_usage_are_off() {
    let rules = Rules::from_data_block(SPRING26).expect("Spring 26 loads");
    let mut team = TeamView::sample_team();
    set_tier(player_mut(&mut team, 5), 92, 8, 3);
    let found = problems(&rules, &team);
    // 17 rating stats of PES 15's 19 abilities, the prowess pair above 88 too: 19.
    assert_eq!(found.len(), 19, "{found:?}");
    assert!(
        found.iter().all(|v| matches!(
            v.kind,
            ViolationKind::Stat {
                value: 92,
                target: 88,
                ..
            }
        )),
        "{found:?}"
    );
    let mut team = TeamView::sample_team();
    assert_eq!(player(&team, 5).weak_foot_usage, 2);
    player_mut(&mut team, 5).weak_foot_usage = 3;
    assert_eq!(
        problems(&rules, &team),
        vec![error(
            Some(5),
            ViolationKind::WeakFootUsage { value: 3, max: 2 }
        )]
    );
}
