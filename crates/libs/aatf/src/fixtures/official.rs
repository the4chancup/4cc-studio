//! The official ruleset's fixtures (Autumn 26). Roster slots name the
//! sample team's players: slot 2 and 10 its golds, 5, 9 and 20 its
//! silvers, 0 and 11 its goalkeepers (189 cm), 3, 4, 6, 15, 19 and 22 its
//! manlets (175 cm), 5, 8, 14, 17 and 20 its giants (194 cm).

use std::collections::BTreeMap;

use super::*;
use crate::ruleset::Card;
use crate::values::CardLimits;
use crate::violation::CheckError;

/// The sample team (legal under Spring 26: two golds, three silvers at 88,
/// goalkeepers at 74, Green quotas 5 giants, 6 tall, 6 mid, 6 manlets)
/// made legal under Autumn 26 by the smallest edits: one gold becomes a
/// silver and two silvers bronzes, the silver giant takes the Green
/// giants' -3, the goalkeepers rate 77, and one mid player becomes a sixth
/// giant (Autumn 26 quotas 6 giants, 6 tall, 5 mid, 6 manlets). The
/// Green system, both 189 cm goalkeepers counted as tall, the manlets'
/// weak foot and the silver giant's -3 are exercised by this team alone.
fn legal_records() -> Records {
    let mut records = Records::sample();
    // Premises on the sample as written.
    let dribbling = |records: &mut Records, slot: u8| records.player_mut(slot).stats.dribbling;
    assert_eq!(dribbling(&mut records, 2), 99);
    assert_eq!(dribbling(&mut records, 10), 99);
    assert_eq!(dribbling(&mut records, 5), 88);
    assert_eq!(dribbling(&mut records, 9), 88);
    assert_eq!(dribbling(&mut records, 20), 88);
    assert_eq!(dribbling(&mut records, 0), 74);
    assert_eq!(dribbling(&mut records, 11), 74);
    assert_eq!(records.player_mut(5).basic.height, 194);
    assert_eq!(records.player_mut(1).basic.height, 180);
    set_tier_record(records.player_mut(10), 92, 8, 3);
    set_tier_record(records.player_mut(5), 89, 8, 3);
    set_tier_record(records.player_mut(9), 86, 8, 3);
    set_tier_record(records.player_mut(20), 86, 8, 3);
    set_tier_record(records.player_mut(0), 77, 4, 1);
    set_tier_record(records.player_mut(11), 77, 4, 1);
    records.player_mut(1).basic.height = 194;
    records
}

/// `legal_records` viewed.
fn legal_team() -> TeamView {
    legal_records().view()
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
    for age in [15, 50] {
        assert_eq!(problems(&rules, &edited(3, |p| p.age = age)), vec![]);
    }
}

/// Slot 3 is 175 cm: `max(30, 175 - 129)..=175 - 81` is 46 to 94; at
/// 150 cm the height-derived bound (21 to 69) sits below the floor, so
/// `max(30, ..)..=69` is 30 to 69.
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
    // At 150 cm (still a manlet: the bracket's `up_to` is 175) the floor
    // is the lower bound: 29 is under it, 30 is legal.
    let under = edited(3, |p| {
        p.height = 150;
        p.weight = 29;
    });
    assert_eq!(
        problems(&rules, &under),
        vec![error(
            Some(3),
            ViolationKind::Weight {
                weight: 29,
                min: 30,
                max: 69
            }
        )]
    );
    let at_floor = edited(3, |p| {
        p.height = 150;
        p.weight = 30;
    });
    assert_eq!(problems(&rules, &at_floor), vec![]);
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

/// Slot 12 with every card removed and rated A at four more outfield
/// positions: five A positions where a regular is allowed one, so the
/// extra four deduct the paid limit to 3 - 4 = -1 and paid 0 is still
/// over it.
#[test]
fn an_overspend_in_a_positions_is_over_the_limit() {
    let rules = Rules::official();
    let team = edited(12, |p| {
        assert_eq!(p.cards.len(), 3);
        assert_eq!(p.ratings.iter().filter(|rating| **rating == 2).count(), 1);
        p.cards.clear();
        for name in ["DMF", "LMF", "AMF", "CF"] {
            p.ratings[usize::from(position(name).0)] = 2;
        }
    });
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(12), ViolationKind::Cards { paid: 0, limit: -1 })]
    );
    let limits = rules
        .card_limits(&team, team.player(12).expect("slot 12"), "regular")
        .expect("limits");
    assert_eq!(limits.limit, -1);
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

// ---- The specials, one fixture each

/// Slot 12 pays three cards, his limit: Malicia on top costs nothing, and
/// with a paid card on top of both he pays four, not five.
#[test]
fn malicia_is_a_free_card() {
    let rules = Rules::official();
    assert_eq!(
        problems(&rules, &edited(12, |p| p.cards.push(card("malicia")))),
        vec![]
    );
    let team = edited(12, |p| {
        p.cards.push(card("malicia"));
        p.cards.push(card("long_range_drive"));
    });
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(12), ViolationKind::Cards { paid: 4, limit: 3 })]
    );
}

/// Captaincy on slot 12 is a paid card, and free once he is the captain
/// (slot 10, the captain before, then pays his own within his limit).
#[test]
fn the_captains_captaincy_card_is_free() {
    let rules = Rules::official();
    assert_eq!(
        problems(&rules, &edited(12, |p| p.cards.push(card("captaincy")))),
        vec![error(Some(12), ViolationKind::Cards { paid: 4, limit: 3 })]
    );
    let mut team = edited(12, |p| p.cards.push(card("captaincy")));
    assert!(player(&team, 10).cards.contains(&card("captaincy")));
    team.captain = Some(12);
    assert_eq!(problems(&rules, &team), vec![]);
}

/// Slot 9, a bronze, registered GK with his A moved there, at a
/// goalkeeper's tall height (189, so that rule alone breaks).
#[test]
fn medals_cannot_be_goalkeepers() {
    let gk = position("GK");
    let ss = position("SS");
    let team = edited(9, |p| {
        assert_eq!(p.position, Some(ss));
        assert_eq!(p.height, 185);
        p.position = Some(gk);
        p.height = 189;
        p.ratings[usize::from(gk.0)] = 2;
        p.ratings[usize::from(ss.0)] = 0;
    });
    assert_eq!(
        problems(&Rules::official(), &team),
        vec![error(
            Some(9),
            special("Medals cannot be goalkeepers", "Medals cannot play as GK")
        )]
    );
}

/// Slot 0, a goalkeeper at 189 cm counted as tall by the legal team: at
/// 185 still tall, but in the range the special forbids; at 184 also in
/// no bracket, one tall player short.
#[test]
fn goalkeepers_at_189_count_as_tall_and_181_to_188_is_forbidden() {
    let rules = Rules::official();
    assert_eq!(player(&legal_team(), 0).height, 189);
    assert_eq!(
        problems(&rules, &edited(0, |p| p.height = 185)),
        vec![error(
            Some(0),
            special(
                "Tall goalkeepers are 189",
                "GKs in this bracket must be 189cm"
            )
        )]
    );
    assert_eq!(
        problems(&rules, &edited(0, |p| p.height = 184)),
        vec![
            error(Some(0), ViolationKind::Height { height: 184 }),
            error(
                Some(0),
                special(
                    "Tall goalkeepers are 189",
                    "GKs in this bracket must be 189cm"
                )
            ),
            quota("green", "tall", 5, 6),
        ]
    );
}

/// A goalkeeper at 194 is a warning, not an error (and a tall player
/// short, a giant over, in the brackets' order).
#[test]
fn goalkeepers_below_giant_is_a_warning() {
    assert_eq!(
        problems(&Rules::official(), &edited(0, |p| p.height = 194)),
        vec![
            warning(
                0,
                special(
                    "Goalkeepers below giant",
                    "GK heights cannot be 194cm or more"
                )
            ),
            quota("green", "tall", 5, 6),
            quota("green", "giant", 7, 6),
        ]
    );
}

/// The gold (slot 2, 185 cm) at 194: a warning, and the quotas.
#[test]
fn gold_below_giant_is_a_warning() {
    assert_eq!(
        problems(&Rules::official(), &edited(2, |p| p.height = 194)),
        vec![
            warning(
                2,
                special("Gold below giant", "Gold heights cannot be 194cm or more")
            ),
            quota("green", "tall", 5, 6),
            quota("green", "giant", 7, 6),
        ]
    );
}

/// `legal_team` under the Red system: no giant (the six 194 cm players
/// become one mid and five tall, the silver giant then rating 92), and
/// the six regular manlets at the Red manlet bonus, 82 (Red quotas 10
/// tall, 7 mid, 6 manlets).
fn red_legal_records() -> Records {
    let mut records = legal_records();
    for slot in [1, 8] {
        records.player_mut(slot).basic.height = 180;
    }
    for slot in [5, 20] {
        records.player_mut(slot).basic.height = 185;
    }
    // The two regular CBs fielded only at CB count 4 cm shorter: 189 is their tall.
    for slot in [14, 17] {
        records.player_mut(slot).basic.height = 189;
    }
    set_tier_record(records.player_mut(5), 92, 8, 3);
    for slot in [3, 4, 6, 15, 19, 22] {
        assert_eq!(records.player_mut(slot).basic.height, 175);
        set_tier_record(records.player_mut(slot), 82, 4, 1);
    }
    records
}

/// `red_legal_records` viewed.
fn red_legal_team() -> TeamView {
    red_legal_records().view()
}

/// `red_legal_team` with `edit` applied to the player at `slot`.
fn red_edited(slot: u8, edit: impl FnOnce(&mut PlayerView)) -> TeamView {
    let mut team = red_legal_team();
    edit(player_mut(&mut team, slot));
    team
}

#[test]
fn the_red_legal_team_reports_nothing() {
    assert_eq!(problems(&Rules::official(), &red_legal_team()), vec![]);
}

/// Under Red a regular manlet rates 82: one at 77 is off every rating
/// stat's target (17 of PES 15's 19 abilities; the prowess pair may sit
/// below). Under Green the same manlet at 77 is the legal team's.
#[test]
fn red_manlet_bonus() {
    let team = red_edited(3, |p| set_tier(p, 77, 4, 1));
    let found = problems(&Rules::official(), &team);
    assert_eq!(found.len(), 17, "{found:?}");
    assert!(
        found.iter().all(|v| matches!(
            v.kind,
            ViolationKind::Stat {
                value: 77,
                target: 82,
                at_most: false,
                ..
            }
        )),
        "{found:?}"
    );
}

/// Under Red a regular CB fielded only at CB counts 4 cm shorter at every
/// height, as `aatf.cpp` reads "Red CBs up to 189": slot 14 (a tall
/// substitute, so his registered position counts) is tall at 189 and in no
/// bracket at 185; a CB under Green at 189 is in no bracket; and slot 17, a
/// starting CB at 189, loses the allowance once one fielded formation moves
/// him to RB.
#[test]
fn red_cbs_count_four_cm_shorter() {
    let rules = Rules::official();
    let cb = position("CB");
    let red = red_legal_team();
    assert_eq!(player(&red, 14).position, Some(cb));
    assert_eq!(player(&red, 14).height, 189);
    assert!(!red.tactics.starting_eleven.contains(&14));
    assert_eq!(
        problems(&rules, &red_edited(14, |p| p.height = 185)),
        vec![
            error(Some(14), ViolationKind::Height { height: 185 }),
            error(
                None,
                ViolationKind::BracketQuota {
                    system: "red".to_string(),
                    bracket: "tall".to_string(),
                    count: 9,
                    quota: 10
                }
            ),
        ]
    );
    // A CB under Green: 189 is in no bracket.
    let mut green = legal_team();
    assert_eq!(player(&green, 8).height, 194);
    player_mut(&mut green, 8).height = 189;
    assert_eq!(
        problems(&rules, &green),
        vec![
            error(Some(8), ViolationKind::Height { height: 189 }),
            quota("green", "giant", 5, 6),
        ]
    );
    // A starting CB: fielded only at CB, then moved to RB in preset 1.
    let mut team = red_legal_team();
    assert_eq!(player(&team, 17).height, 189);
    assert!(team.fielded_only_at(17, cb));
    assert_eq!(problems(&rules, &team), vec![]);
    let index = team
        .tactics
        .starting_eleven
        .iter()
        .position(|slot| *slot == 17)
        .expect("slot 17 starts");
    team.tactics.presets[0].formations[0].players[index].position = position("RB").0;
    assert!(!team.fielded_only_at(17, cb));
    assert_eq!(
        problems(&rules, &team),
        vec![
            error(Some(17), ViolationKind::Height { height: 189 }),
            error(
                None,
                ViolationKind::BracketQuota {
                    system: "red".to_string(),
                    bracket: "tall".to_string(),
                    count: 9,
                    quota: 10
                }
            ),
        ]
    );
}

/// A manlet (slot 3, paid 3) gets one more card and one more A position,
/// and weak foot 4/4, which the legal team's manlets carry.
#[test]
fn manlets_get_a_card_an_a_position_and_four_four_weak_foot() {
    let rules = Rules::official();
    let manlet = player(&legal_team(), 3).clone();
    assert_eq!((manlet.height, manlet.cards.len()), (175, 3));
    assert_eq!((manlet.weak_foot_usage, manlet.weak_foot_accuracy), (4, 4));
    assert_eq!(
        problems(
            &rules,
            &edited(3, |p| p.cards.push(card("long_range_drive")))
        ),
        vec![]
    );
    let team = edited(3, |p| {
        p.cards.push(card("long_range_drive"));
        p.cards.push(card("knuckle_shot"));
    });
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(3), ViolationKind::Cards { paid: 5, limit: 4 })]
    );
    let cb = position("CB");
    assert_eq!(
        problems(&rules, &edited(3, |p| p.ratings[usize::from(cb.0)] = 2)),
        vec![]
    );
}

/// The legal team's silver giant (slot 5) rates 89; at 92 every ability
/// is off its lowered target, the prowess pair included (above it).
#[test]
fn green_silver_giants_rate_three_below() {
    let team = edited(5, |p| set_tier(p, 92, 8, 3));
    let found = problems(&Rules::official(), &team);
    assert_eq!(found.len(), 19, "{found:?}");
    assert!(
        found.iter().all(|v| matches!(
            v.kind,
            ViolationKind::Stat {
                value: 92,
                target: 89,
                ..
            }
        )),
        "{found:?}"
    );
}

/// Slot 9, a bronze at his limit with one COM style (free): a second A
/// position is the free one; with a second COM style the choice goes to
/// the COM style and the A position costs a card. Slot 10, a silver with
/// two COM styles (the choice taken), pays for a second A.
#[test]
fn medals_choose_a_free_a_position_or_a_free_com_style() {
    let rules = Rules::official();
    let cf = position("CF");
    let amf = position("AMF");
    let bronze = player(&legal_team(), 9).clone();
    assert_eq!(bronze.cards.len(), 7);
    assert_eq!(bronze.cards[6], card("speeding_bullet"));
    assert_eq!(
        problems(&rules, &edited(9, |p| p.ratings[usize::from(cf.0)] = 2)),
        vec![]
    );
    let team = edited(9, |p| {
        p.ratings[usize::from(cf.0)] = 2;
        p.cards.push(card("mazing_run"));
    });
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(9), ViolationKind::Cards { paid: 4, limit: 3 })]
    );
    let silver = player(&legal_team(), 10).clone();
    assert_eq!(silver.cards.len(), 9);
    let team = edited(10, |p| {
        p.ratings[usize::from(amf.0)] = 2;
        p.cards.push(card("knuckle_shot"));
    });
    assert_eq!(
        problems(&rules, &team),
        vec![error(Some(10), ViolationKind::Cards { paid: 5, limit: 4 })]
    );
}

// ---- Suggestions

/// Each listed allowance a player does not use up is one suggestion:
/// slot 12 holds no trick (two free); the gold (slot 2) pays four of six
/// and has weak foot usage 2 of 4, then, with a COM style dropped, one
/// of two, and with injury resistance 2, one short of 3 (no error: the
/// target is `at_most`).
#[test]
fn suggestions_name_each_unused_allowance() {
    let rules = Rules::official();
    let team = legal_team();
    assert_eq!(
        suggestions_for(&rules, &team, 12),
        vec![unused("tricks", 0, 2)]
    );
    assert_eq!(
        suggestions_for(&rules, &team, 2),
        vec![unused("cards", 4, 6), unused("weak_foot", 2, 4)]
    );
    let team = edited(2, |p| {
        assert_eq!(p.cards.pop(), Some(card("mazing_run")));
        p.injury_resistance = 2;
    });
    assert_eq!(problems(&rules, &team), vec![]);
    assert_eq!(
        suggestions_for(&rules, &team, 2),
        vec![
            unused("cards", 4, 6),
            unused("com", 1, 2),
            unused("weak_foot", 2, 4),
            unused("injury_resistance", 2, 3),
        ]
    );
}

// ---- tier_values, card_limits, apply_tier

/// For every tier of the official file, a player of that tier in the
/// legal team, his stats, form, injury resistance and weak foot broken,
/// is legal again after `apply_tier`: the silver giant under Green (89)
/// and a regular manlet under Red (82) included.
#[test]
fn apply_tier_then_check_reports_nothing_for_every_official_tier() {
    let rules = Rules::official();
    let tiers: Vec<&str> = rules
        .ruleset()
        .tiers
        .iter()
        .map(|tier| tier.id.as_str())
        .collect();
    assert_eq!(tiers, ["gold", "silver", "bronze", "goalkeeper", "regular"]);
    // (tier, slot, under the Red team rather than the Green one)
    for (tier, slot, red) in [
        ("gold", 2, false),
        ("silver", 10, false),
        ("silver", 5, false),
        ("bronze", 9, false),
        ("goalkeeper", 0, false),
        ("regular", 12, false),
        ("regular", 3, true),
    ] {
        let mut records = if red {
            red_legal_records()
        } else {
            legal_records()
        };
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

/// The targets carry the specials: the Green silver giant rates 89, a
/// manlet gets weak foot 4/4 and, under Red, 82; a regular who is no
/// manlet gets 2/2, form 4, injury resistance 1. An id no tier has is
/// refused before the script runs.
#[test]
fn tier_values_carry_the_specials() {
    let rules = Rules::official();
    let team = legal_team();
    let values = |team: &TeamView, slot: u8, tier: &str| {
        rules
            .tier_values(team, team.player(slot).expect("filled"), tier)
            .expect("values")
    };
    let giant = values(&team, 5, "silver");
    assert_eq!(giant.stats.len(), 25);
    assert!(giant.stats.values().all(|v| *v == 89), "{giant:?}");
    assert_eq!((giant.form, giant.injury_resistance), (8, 3));
    assert_eq!(
        giant.weak_foot,
        WeakFoot {
            usage: 4,
            accuracy: 4
        }
    );
    let manlet = values(&team, 3, "regular");
    assert!(manlet.stats.values().all(|v| *v == 77), "{manlet:?}");
    assert_eq!(
        manlet.weak_foot,
        WeakFoot {
            usage: 4,
            accuracy: 4
        }
    );
    let red = red_legal_team();
    let manlet = values(&red, 3, "regular");
    assert!(manlet.stats.values().all(|v| *v == 82), "{manlet:?}");
    let regular = values(&team, 12, "regular");
    assert_eq!(
        regular.weak_foot,
        WeakFoot {
            usage: 2,
            accuracy: 2
        }
    );
    assert_eq!((regular.form, regular.injury_resistance), (4, 1));
    assert_eq!(
        rules
            .tier_values(&team, team.player(3).expect("filled"), "platinum")
            .expect_err("no such tier"),
        CheckError::UnknownTier {
            tier: "platinum".to_string()
        }
    );
}

/// The card economy per player and tier: a regular's limit and free
/// tricks, a manlet's extra card and A position, the silver captain's
/// free Captaincy and the choice gone to his second COM style, and the
/// same player read as gold (free COM 2 by the tier, no option used).
#[test]
fn card_limits_name_the_economy() {
    let rules = Rules::official();
    let team = legal_team();
    let limits = |slot: u8, tier: &str| {
        rules
            .card_limits(&team, team.player(slot).expect("filled"), tier)
            .expect("limits")
    };
    assert_eq!(
        limits(12, "regular"),
        CardLimits {
            paid: 3,
            limit: 3,
            free: BTreeMap::from([("tricks".to_string(), 2)]),
            free_cards: vec![card("malicia")],
            required_cards: vec![],
            a_positions: 1,
            max_a_positions: None,
        }
    );
    let manlet = limits(3, "regular");
    assert_eq!((manlet.limit, manlet.a_positions), (4, 2));
    let captain = limits(10, "silver");
    assert_eq!(captain.limit, 5);
    assert_eq!(
        captain.free,
        BTreeMap::from([("com".to_string(), 2), ("tricks".to_string(), 3)])
    );
    assert_eq!(captain.free_cards, vec![card("malicia"), card("captaincy")]);
    let as_gold = limits(10, "gold");
    assert_eq!((as_gold.limit, as_gold.a_positions), (6, 1));
    assert_eq!(as_gold.free.get("com"), Some(&2));

    // The whole economy per tier: the paid limit, the A positions, the
    // free counts (the exact map, so a group named nowhere in it shows)
    // and `max_a_positions` unset, plus `tier_values`' weak foot.
    // Slot 0 for the goalkeeper, slot 9 for the bronze (a member taking
    // no option of the medals' choice); the other medals each carry a
    // share of the choice — slot 2's second A position takes the `a`
    // option (his `a_positions` reads the tier's 1 plus its share),
    // slot 5's the same, slot 10's second COM style takes the `com`
    // option (his `free` reads the tier's {tricks 3, com 1} plus its
    // share — the a_positions still the tier's own). Where a member's
    // share masks the file's value, the probe names it.
    assert_economy(
        &rules,
        &team,
        &[
            (2, "gold", 6, 2, &[("com", 2), ("tricks", 3)], (4, 4)),
            (5, "silver", 5, 2, &[("com", 1), ("tricks", 3)], (4, 4)),
            (10, "silver", 5, 1, &[("com", 2), ("tricks", 3)], (4, 4)),
            (9, "bronze", 4, 1, &[("com", 1), ("tricks", 3)], (4, 4)),
            (0, "goalkeeper", 2, 1, &[], (2, 2)),
            (12, "regular", 3, 1, &[("tricks", 2)], (2, 2)),
        ],
    );
    // The gold's `a_positions` masked by his option: the tier's own 1
    // reads on a member of another tier — the captain (a silver, one A
    // and two COM styles the gold's free two cover) read as gold.
    assert_eq!(limits(10, "gold").a_positions, 1);
}

/// `paid` counts the held cards no free group or free card covers:
/// slot 2, a gold holding nine cards — four tricks, three other skills
/// and two COM styles — under the gold's free three tricks and two COM
/// styles, so the four the frees leave are his paid ones. The badge a
/// card picker shows under a tier is `limit - paid`.
#[test]
fn card_limits_reports_the_paid_count() {
    let rules = Rules::official();
    let team = legal_team();
    let gold = player(&team, 2);
    assert_eq!(gold.cards.len(), 9, "premise: the gold holds nine cards");
    assert_eq!(
        gold.cards
            .iter()
            .filter(|held| {
                [
                    card("scissors_feint"),
                    card("marseille_turn"),
                    card("cut_behind_and_turn"),
                    card("scotch_move"),
                ]
                .contains(held)
            })
            .count(),
        4,
        "premise: four of the gold's nine cards are tricks"
    );
    let limits = rules.card_limits(&team, gold, "gold").expect("limits");
    assert_eq!(limits.free.get("com"), Some(&2));
    assert_eq!(limits.free.get("tricks"), Some(&3));
    assert_eq!(limits.paid, 4);
}

/// An allowance the file does not list is never suggested: a manlet
/// (slot 3) with one A of two, and a bronze (slot 9) taking no option of
/// the medals' choice, get no `a_positions` or `choices` suggestion.
#[test]
fn nothing_is_suggested_for_an_allowance_not_listed() {
    let rules = Rules::official();
    let team = legal_team();
    let official = rules.ruleset();
    assert!(!official.suggestions.iter().any(|s| s == "a_positions"));
    assert!(!official.suggestions.iter().any(|s| s == "choices"));
    assert_eq!(
        player(&team, 3).ratings.iter().filter(|r| **r == 2).count(),
        1
    );
    for slot in [3, 9] {
        assert!(
            suggestions_for(&rules, &team, slot)
                .iter()
                .all(|kind| matches!(
                    kind,
                    ViolationKind::Unused { allowance, .. }
                        if ["cards", "tricks", "com", "weak_foot", "injury_resistance"]
                            .contains(&allowance.as_str())
                )),
            "{slot}"
        );
    }
}
