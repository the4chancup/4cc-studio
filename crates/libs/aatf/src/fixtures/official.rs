//! The official ruleset's fixtures (Autumn 26). Roster slots name the
//! sample team's players: slot 2 and 10 its golds, 5, 9 and 20 its
//! silvers, 0 and 11 its goalkeepers (189 cm), 3, 4, 6, 15, 19 and 22 its
//! manlets (175 cm), 5, 8, 14, 17 and 20 its giants (194 cm).

use super::*;
use crate::ruleset::{Card, Position, Stat};

/// The ability stat `name` names.
fn stat(name: &str) -> Stat {
    Stat::try_from(name.to_string()).expect("a stat name")
}

/// The position `name` names.
fn position(name: &str) -> Position {
    Position::try_from(name.to_string()).expect("a position name")
}

/// The card `name` names.
fn card(name: &str) -> Card {
    Card::try_from(name.to_string()).expect("a card name")
}

/// The sample team (legal under Spring 26: two golds, three silvers at 88,
/// goalkeepers at 74, Green quotas 5 giants, 6 tall, 6 mid, 6 manlets)
/// made legal under Autumn 26 by the smallest edits: one gold becomes a
/// silver and two silvers bronzes, the silver giant takes the Green
/// giants' -3, the goalkeepers rate 77, and one mid player becomes a sixth
/// giant (Autumn 26 quotas 6 giants, 6 tall, 5 mid, 6 manlets). The
/// Green system, both 189 cm goalkeepers counted as tall, the manlets'
/// weak foot and the silver giant's -3 are exercised by this team alone.
fn legal_team() -> TeamView {
    let mut team = TeamView::sample_team();
    // Premises on the sample as written.
    let dribbling = stat("dribbling");
    let value = |team: &TeamView, slot: u8| player(team, slot).abilities[usize::from(dribbling.0)];
    assert_eq!(value(&team, 2), Some(99));
    assert_eq!(value(&team, 10), Some(99));
    assert_eq!(value(&team, 5), Some(88));
    assert_eq!(value(&team, 9), Some(88));
    assert_eq!(value(&team, 20), Some(88));
    assert_eq!(value(&team, 0), Some(74));
    assert_eq!(value(&team, 11), Some(74));
    assert_eq!(player(&team, 5).height, 194);
    assert_eq!(player(&team, 1).height, 180);
    set_tier(player_mut(&mut team, 10), 92, 8, 3);
    set_tier(player_mut(&mut team, 5), 89, 8, 3);
    set_tier(player_mut(&mut team, 9), 86, 8, 3);
    set_tier(player_mut(&mut team, 20), 86, 8, 3);
    set_tier(player_mut(&mut team, 0), 77, 4, 1);
    set_tier(player_mut(&mut team, 11), 77, 4, 1);
    player_mut(&mut team, 1).height = 194;
    team
}

/// `legal_team` with `edit` applied to the player at `slot`.
fn edited(slot: u8, edit: impl FnOnce(&mut PlayerView)) -> TeamView {
    let mut team = legal_team();
    edit(player_mut(&mut team, slot));
    team
}

#[test]
fn the_legal_team_reports_nothing() {
    assert_eq!(problems(&Rules::official(), &legal_team()), vec![]);
}

/// Slot 3 (a regular RB, A at RB alone) rated C at RB.
#[test]
fn registered_position_not_a() {
    let rules = Rules::official();
    let rb = position("RB");
    assert_eq!(player(&legal_team(), 3).ratings[usize::from(rb.0)], 2);
    let team = edited(3, |p| p.ratings[usize::from(rb.0)] = 0);
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(3), ViolationKind::RegisteredNotA)]
    );
}

/// Slot 3 given a B at CB.
#[test]
fn b_rating() {
    let cb = position("CB");
    let team = edited(3, |p| p.ratings[usize::from(cb.0)] = 1);
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![error(Some(3), ViolationKind::BRating)]
    );
}

/// Slot 12 (a regular CMF holding two skills and one COM style, so his
/// three paid cards sit at the limit) with one card dropped, which costs
/// nothing, then given an A at GK: the GK rating is his second A. Without
/// the dropped card the extra A would also cost a card.
#[test]
fn gk_second_a() {
    let rules = Rules::official();
    let gk = position("GK");
    let dropped = edited(12, |p| {
        assert_eq!(p.cards.len(), 3);
        p.cards.pop();
    });
    assert_eq!(problems(&rules, &dropped), vec![]);
    let mut team = dropped;
    player_mut(&mut team, 12).ratings[usize::from(gk.0)] = 2;
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(12), ViolationKind::GkSecondA)]
    );
}

#[test]
fn age_outside_the_bounds() {
    let rules = Rules::official();
    for age in [14, 51] {
        let team = edited(3, |p| p.age = age);
        assert_eq!(
            problems(&rules, &team),
            vec![error(
                Some(3),
                ViolationKind::Age {
                    age,
                    min: 15,
                    max: 50
                }
            )],
            "{age}"
        );
    }
    assert_eq!(problems(&rules, &edited(3, |p| p.age = 50)), vec![]);
}

/// Slot 3 is 175 cm: `max(30, 175 - 129)..=175 - 81` is 46 to 94.
#[test]
fn weight_outside_the_rule() {
    let rules = Rules::official();
    for weight in [45, 95] {
        let team = edited(3, |p| p.weight = weight);
        assert_eq!(
            problems(&rules, &team),
            vec![error(
                Some(3),
                ViolationKind::Weight {
                    weight,
                    min: 46,
                    max: 94
                }
            )],
            "{weight}"
        );
    }
    assert_eq!(problems(&rules, &edited(3, |p| p.weight = 46)), vec![]);
}

/// A registered position past the labels, and a playing style the
/// version's table lacks: each `()` in the script.
#[test]
fn position_and_playing_style_ranges() {
    let rules = Rules::official();
    assert_eq!(
        problems(&rules, &edited(3, |p| p.position = None)),
        vec![error(Some(3), ViolationKind::PositionRange)]
    );
    assert_eq!(
        problems(&rules, &edited(3, |p| p.playing_style = None)),
        vec![error(Some(3), ViolationKind::PlayingStyleRange)]
    );
}

/// A rating stat is met exactly: dribbling 78 on a regular is off target
/// 77 (and reads as a regular still, bronze's 86 being above 78).
#[test]
fn a_rating_stat_off_its_target() {
    let dribbling = stat("dribbling");
    let team = edited(3, |p| p.abilities[usize::from(dribbling.0)] = Some(78));
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![error(
            Some(3),
            ViolationKind::Stat {
                stat: dribbling,
                value: 78,
                target: 77,
                at_most: false
            }
        )]
    );
}

/// An `at_most` stat may sit below its target: attacking prowess 70 on a
/// regular is legal, 78 is not.
#[test]
fn an_at_most_stat_may_sit_below() {
    let rules = Rules::official();
    let prowess = stat("attacking_prowess");
    let index = usize::from(prowess.0);
    assert_eq!(
        problems(&rules, &edited(3, |p| p.abilities[index] = Some(70))),
        vec![]
    );
    assert_eq!(
        problems(&rules, &edited(3, |p| p.abilities[index] = Some(78))),
        vec![error(
            Some(3),
            ViolationKind::Stat {
                stat: prowess,
                value: 78,
                target: 77,
                at_most: true
            }
        )]
    );
}

/// Form is met exactly: a regular at 5 or 3 is off target 4.
#[test]
fn form_off_its_target() {
    let rules = Rules::official();
    for form in [5, 3] {
        assert_eq!(
            problems(&rules, &edited(3, |p| p.form = form)),
            vec![error(
                Some(3),
                ViolationKind::Form {
                    value: form,
                    target: 4
                }
            )],
            "{form}"
        );
    }
}

/// Injury resistance is at most the target: a regular at 2 is over 1.
#[test]
fn injury_resistance_over_the_target() {
    assert_eq!(
        problems(&Rules::official(), &edited(3, |p| p.injury_resistance = 2)),
        vec![error(
            Some(3),
            ViolationKind::InjuryResistance { value: 2, max: 1 }
        )]
    );
}

/// Slot 12, a regular who is no manlet, is limited to weak foot 2/2.
#[test]
fn weak_foot_over_the_limits() {
    let rules = Rules::official();
    assert_eq!(player(&legal_team(), 12).height, 180);
    assert_eq!(
        problems(&rules, &edited(12, |p| p.weak_foot_usage = 3)),
        vec![error(
            Some(12),
            ViolationKind::WeakFootUsage { value: 3, max: 2 }
        )]
    );
    assert_eq!(
        problems(&rules, &edited(12, |p| p.weak_foot_accuracy = 3)),
        vec![error(
            Some(12),
            ViolationKind::WeakFootAccuracy { value: 3, max: 2 }
        )]
    );
}

/// Slot 12 pays three cards, a regular's limit: a fourth skill is over
/// it, a trick (two free for a regular) is not, and an extra A position
/// costs a card.
#[test]
fn paid_cards_over_the_limit() {
    let rules = Rules::official();
    assert_eq!(player(&legal_team(), 12).cards.len(), 3);
    assert_eq!(
        problems(
            &rules,
            &edited(12, |p| p.cards.push(card("long_range_drive")))
        ),
        vec![error(Some(12), ViolationKind::Cards { paid: 4, limit: 3 })]
    );
    assert_eq!(
        problems(
            &rules,
            &edited(12, |p| p.cards.push(card("scissors_feint")))
        ),
        vec![]
    );
    let cf = position("CF");
    assert_eq!(
        problems(&rules, &edited(12, |p| p.ratings[usize::from(cf.0)] = 2)),
        vec![error(Some(12), ViolationKind::Cards { paid: 3, limit: 2 })]
    );
}

/// Eleven skills on slot 12 (a regular who is no manlet, so no bonus
/// card), none of them a trick: the game's cap of ten, over the paid limit
/// as well (the cap is never reached under it in this file).
#[test]
fn skill_cards_over_the_games_cap() {
    let team = edited(12, |p| {
        p.cards = [7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18]
            .into_iter()
            .map(Card::Skill)
            .collect();
    });
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![
            error(Some(12), ViolationKind::Cards { paid: 11, limit: 3 }),
            error(Some(12), ViolationKind::SkillCardCap { count: 11, max: 10 }),
        ]
    );
}

/// Slot 12 (180 cm) at 182 cm, in no bracket: the player's height, and
/// the mid quota he left. (A manlet grown instead would also lose his
/// weak-foot allowance: the test uses a mid player so one rule breaks.)
#[test]
fn a_height_in_no_bracket() {
    assert_eq!(
        problems(&Rules::official(), &edited(12, |p| p.height = 182)),
        vec![
            error(Some(12), ViolationKind::Height { height: 182 }),
            error(
                None,
                ViolationKind::BracketQuota {
                    system: "green".to_string(),
                    bracket: "mid".to_string(),
                    count: 4,
                    quota: 5
                }
            ),
        ]
    );
}

/// The gold made a silver: no gold, three silvers.
#[test]
fn tier_counts_off() {
    let team = edited(2, |p| set_tier(p, 92, 8, 3));
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![
            error(
                None,
                ViolationKind::TierCount {
                    tier: "gold".to_string(),
                    count: 0,
                    expected: 1
                }
            ),
            error(
                None,
                ViolationKind::TierCount {
                    tier: "silver".to_string(),
                    count: 3,
                    expected: 2
                }
            ),
        ]
    );
}

/// Slot 12, a mid player, shrunk to a manlet: one manlet over, one mid
/// short, in the brackets' order. (His weak foot 2/2 and three paid cards
/// stay within a manlet's larger allowances.)
#[test]
fn bracket_quotas_off() {
    assert_eq!(
        problems(&Rules::official(), &edited(12, |p| p.height = 175)),
        vec![
            error(
                None,
                ViolationKind::BracketQuota {
                    system: "green".to_string(),
                    bracket: "manlet".to_string(),
                    count: 7,
                    quota: 6
                }
            ),
            error(
                None,
                ViolationKind::BracketQuota {
                    system: "green".to_string(),
                    bracket: "mid".to_string(),
                    count: 4,
                    quota: 5
                }
            ),
        ]
    );
}

/// No captain set. Slot 10's Captaincy card is then paid, within his
/// silver limit still.
#[test]
fn no_captain() {
    let mut team = legal_team();
    assert_eq!(team.captain, Some(10));
    team.captain = None;
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![error(None, ViolationKind::Captain)]
    );
}

/// Both goalkeepers off the roster: no goalkeeper, two regulars short,
/// two tall players short.
#[test]
fn no_goalkeeper() {
    let mut team = legal_team();
    let gk = position("GK");
    team.players.retain(|p| p.position != Some(gk));
    assert_eq!(team.players.len(), 21);
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![
            error(
                None,
                ViolationKind::TierCount {
                    tier: "regular".to_string(),
                    count: 16,
                    expected: 18
                }
            ),
            error(
                None,
                ViolationKind::BracketQuota {
                    system: "green".to_string(),
                    bracket: "tall".to_string(),
                    count: 4,
                    quota: 6
                }
            ),
            error(None, ViolationKind::Goalkeepers { count: 0, min: 1 }),
        ]
    );
}

/// A player below every tier's expected rating reads as the last tier
/// whose condition holds, the regular, and is reported against it: one
/// `Stat` per ability PES 15 has (19) but the two `at_most` prowess
/// stats, which may sit below.
#[test]
fn a_player_below_every_tier_reads_as_the_last_fitting_tier() {
    let team = edited(3, |p| set_tier(p, 70, 4, 1));
    let found = problems(&Rules::official(), &team);
    assert_eq!(player(&team, 3).abilities.iter().flatten().count(), 19);
    assert_eq!(found.len(), 17, "{found:?}");
    assert!(
        found.iter().all(|v| matches!(
            v.kind,
            ViolationKind::Stat {
                value: 70,
                target: 77,
                at_most: false,
                ..
            }
        )),
        "{found:?}"
    );
}
