//! DBC7's fixtures. The sample team on PES 17 made legal under DBC7:
//! every slot's height from a table per system of the page (Height, Manlet,
//! Universal, GigaHeight with its 50 cm pool), the tiers written through
//! `Rules::apply_tier` — the golds, silvers and regulars of one medal
//! system each. Roster slots name the sample team's players: 2 and 10
//! rated 99, 5, 9 and 20 rated 88, 0 and 11 its goalkeepers at 189, which
//! the GK allowance counts as 185 in every system.

use super::*;

/// The DBC7 data block, a test ruleset.
const DBC7: &str = include_str!("../../rulesets/dbc7.rhai");

/// DBC7's rules loaded.
fn rules() -> Rules {
    Rules::from_data_block(DBC7).expect("DBC7 loads")
}

// The height tables, one per system of the page, each slot's height by
// roster slot. The goalkeepers stay 189 (counted as 185).

/// Height: 6x 195 (5, 8, 14, 17, 20, 21), 6x 185 (2, 9, 10, 16 and the
/// keepers), 5x 180, 6x 175.
const HEIGHT: [u8; 23] = [
    189, 180, 185, 175, 175, 195, 175, 180, 195, 185, 185, 189, 180, 180, 195, 175, 185, 195, 180,
    175, 195, 195, 175,
];

/// Manlet: the Height table with 5, 8, 14 and 17 to the talls (10x 185)
/// and 20 and 21 to the mids (7x 180), the same 6x 175.
const MANLET: [u8; 23] = [
    189, 180, 185, 175, 175, 185, 175, 180, 185, 185, 185, 189, 180, 180, 185, 175, 185, 185, 180,
    175, 180, 180, 175,
];

/// Universal: 23x 185 (the keepers counted).
const UNIVERSAL: [u8; 23] = [
    189, 185, 185, 185, 185, 185, 185, 185, 185, 185, 185, 189, 185, 185, 185, 185, 185, 185, 185,
    185, 185, 185, 185,
];

/// GigaHeight: 7x 185 (2, 5, 9, 10, 16 and the keepers), 10x 180, 6x 175.
const GIGA: [u8; 23] = [
    189, 180, 185, 175, 175, 185, 175, 180, 180, 185, 185, 189, 180, 180, 180, 175, 185, 180, 180,
    175, 180, 180, 175,
];

/// The page's two-210 variant: 2x 210 (2, 16), 5x 185 (5, 9, 10 and the
/// keepers), 10x 180, 6x 175 — the 210s take tall slots and draw 25 each,
/// exactly the 50 cm pool.
const GIGA_210: [u8; 23] = [
    189, 180, 210, 175, 175, 185, 175, 180, 180, 185, 185, 189, 180, 180, 180, 175, 210, 180, 180,
    175, 180, 180, 175,
];

/// The Manlet table with a silver manlet: slot 9 at 175, slot 20 back to
/// the talls and slot 4 to the mids.
const MANLET_BONUS: [u8; 23] = [
    189, 180, 185, 175, 180, 185, 175, 180, 185, 175, 185, 189, 180, 180, 185, 175, 185, 185, 180,
    175, 185, 180, 175,
];

// The medal systems' slot assignments: the natural golds 2 and 10, the
// natural silvers 5, 9 and 20, each system's extras named.

const GGG: &[(u8, &str)] = &[(2, "gold"), (5, "gold"), (10, "gold")];
const GGSS: &[(u8, &str)] = &[(2, "gold"), (10, "gold"), (5, "silver"), (9, "silver")];
const GSSSS: &[(u8, &str)] = &[
    (2, "gold"),
    (10, "silver"),
    (5, "silver"),
    (9, "silver"),
    (20, "silver"),
];
const SSSSSS: &[(u8, &str)] = &[
    (2, "silver"),
    (5, "silver"),
    (9, "silver"),
    (10, "silver"),
    (16, "silver"),
    (20, "silver"),
];

/// `records` (the sample team on PES 17) made legal under DBC7: every
/// slot's height from `heights`, then the tiers the sample does not
/// already hold written through `Rules::apply_tier` — `medals`' slots to
/// their tiers, `regular` to the sample's rated players no medal table
/// counts (every other slot already rates no more than 77, the at_most
/// ceiling).
fn legalize(mut records: Records, heights: &[u8; 23], medals: &[(u8, &str)]) -> Records {
    // Premises on the sample as written.
    let dribbling = |records: &mut Records, slot: u8| records.player_mut(slot).stats.dribbling;
    assert_eq!(dribbling(&mut records, 2), 99);
    assert_eq!(dribbling(&mut records, 10), 99);
    assert_eq!(dribbling(&mut records, 5), 88);
    assert_eq!(dribbling(&mut records, 9), 88);
    assert_eq!(dribbling(&mut records, 20), 88);
    assert_eq!(dribbling(&mut records, 0), 74);
    assert_eq!(dribbling(&mut records, 11), 74);
    assert_eq!(records.player_mut(0).basic.height, 189);
    assert_eq!(records.player_mut(11).basic.height, 189);
    {
        let view = records.view();
        assert_eq!(player(&view, 0).position, Some(position("GK")));
        assert_eq!(player(&view, 11).position, Some(position("GK")));
    }
    for (slot, height) in heights.iter().enumerate() {
        records
            .player_mut(u8::try_from(slot).expect("the table has 23"))
            .basic
            .height = *height;
    }
    let mut tiers: Vec<(u8, &str)> = medals.to_vec();
    for slot in [2u8, 5, 9, 10, 20] {
        if !tiers.iter().any(|(s, _)| *s == slot) {
            tiers.push((slot, "regular"));
        }
    }
    let rules = rules();
    for (slot, tier) in tiers {
        let view = records.view();
        let player = view.player(slot).expect("a filled slot").clone();
        rules
            .apply_tier(&view, &player, tier, records.player_mut(slot))
            .expect("the tier applies");
    }
    records
}

/// `legalize` on the sample team's PES 17 build.
fn legal_records(heights: &[u8; 23], medals: &[(u8, &str)]) -> Records {
    legalize(Records::sample_on(PesVersion::Pes17), heights, medals)
}

/// `legal_records` viewed.
fn legal_team(heights: &[u8; 23], medals: &[(u8, &str)]) -> TeamView {
    legal_records(heights, medals).view()
}

/// The legal team under `heights` and `medals`, `edit` applied at `slot`.
fn edited(
    heights: &[u8; 23],
    medals: &[(u8, &str)],
    slot: u8,
    edit: impl FnOnce(&mut PlayerView),
) -> TeamView {
    let mut team = legal_team(heights, medals);
    edit(player_mut(&mut team, slot));
    team
}

/// Every height system of the page and every medal system reports
/// nothing: Height, Manlet, Universal and GigaHeight's own variant (2x
/// 210, 5x 185, 10x 180, 6x 175, drawing exactly the 50) on the GGSS
/// medals, and each medal system on the Giga heights.
#[test]
fn the_legal_teams_report_nothing() {
    let rules = rules();
    assert_eq!(rules.ruleset().name, "DBC7");
    for (name, heights) in [
        ("Height", &HEIGHT),
        ("Manlet", &MANLET),
        ("Universal", &UNIVERSAL),
        ("GigaHeight 2x210", &GIGA_210),
    ] {
        assert_eq!(
            problems(&rules, &legal_team(heights, GGSS)),
            vec![],
            "{name}"
        );
    }
    for (name, medals) in [
        ("GGG", GGG),
        ("GGSS", GGSS),
        ("GSSSS", GSSSS),
        ("SSSSSS", SSSSSS),
    ] {
        assert_eq!(
            problems(&rules, &legal_team(&GIGA, medals)),
            vec![],
            "{name}"
        );
    }
}

/// A registered goalkeeper at 189 counts as 185 in every system: under
/// Universal (23 talls) and under GigaHeight's two-210 variant, where the
/// rest of the team draws exactly the 50 and he draws nothing (his 4 cm
/// are the special's allowance, not pool).
#[test]
fn a_keeper_at_189_counts_as_185_in_every_system() {
    let rules = rules();
    for (name, heights) in [("Universal", &UNIVERSAL), ("GigaHeight 2x210", &GIGA_210)] {
        let team = legal_team(heights, GGSS);
        assert_eq!(player(&team, 0).position, Some(position("GK")), "{name}");
        assert_eq!(player(&team, 0).height, 189, "{name}");
        assert_eq!(player(&team, 11).position, Some(position("GK")), "{name}");
        assert_eq!(player(&team, 11).height, 189, "{name}");
        assert_eq!(problems(&rules, &team), vec![], "{name}");
    }
}

/// Under the Manlet system a 175 regular may carry 82 and a 175 silver 93,
/// or anything up to it: the bonus is a one-option choice, taken when a
/// stat needs it. The
/// team's counts (10x 185, 7x 180, 6x 175) also fill GigaHeight's quotas
/// (three 185s in mid slots, drawing 15), so the systems' order is what
/// reads it as Manlet, and only Manlet gives the bonus.
/// The same regular with one stat at 83 is a `Stat` violation at target
/// 82; under the Height system (no bonus) the 82s are `Stat` violations
/// at target 77 — PES 17's twenty-three ability stats.
#[test]
fn the_manlet_bonus_raises_the_175_targets() {
    let rules = rules();
    let mut team = legal_team(&MANLET_BONUS, GGSS);
    assert_eq!(player(&team, 3).height, 175);
    assert_eq!(player(&team, 9).height, 175);
    // The bonus taken whole: the regular (slot 3) at 82, the silver (9) at 93.
    set_tier(player_mut(&mut team, 3), 82, 8, 3);
    set_tier(player_mut(&mut team, 9), 93, 8, 3);
    assert_eq!(problems(&rules, &team), vec![]);
    // The bonus taken in part: a silver at 92 still reads as a silver, since
    // recognition sees the unbuffed 88 (three Manlet teams of the
    // maintainer's PES 2017 save carry one; a bonus every 175 must take
    // read them as regulars over 82).
    set_tier(player_mut(&mut team, 9), 92, 8, 3);
    assert_eq!(problems(&rules, &team), vec![]);
    // One stat over: dribbling 83 against the bonused target 82.
    let dribbling = stat("dribbling");
    player_mut(&mut team, 3).abilities[usize::from(dribbling.0)] = Some(83);
    assert_eq!(
        problems(&rules, &team),
        vec![error(
            Some(3),
            ViolationKind::Stat {
                stat: dribbling,
                value: 83,
                target: 82,
                at_most: true,
            }
        )]
    );
    // The same 82s under Height: no bonus, target 77 on each stat.
    let team = edited(&HEIGHT, GGSS, 3, |p| set_tier(p, 82, 8, 3));
    let found = problems(&rules, &team);
    assert_eq!(found.len(), 23, "{found:?}");
    assert!(
        found.iter().all(|v| v.slot == Some(3)
            && matches!(
                &v.kind,
                ViolationKind::Stat {
                    value: 82,
                    target: 77,
                    at_most: true,
                    ..
                }
            )),
        "{found:?}"
    );
}

/// One centimetre over the pool: the two-210 variant draws exactly 50; a
/// mid at 181 draws 51 — one `ExtraHeight` and nothing else (every
/// bracket still fills).
#[test]
fn a_centimetre_over_the_pool_is_extra_height() {
    let rules = rules();
    let team = edited(&GIGA_210, GGSS, 8, |p| p.height = 181);
    assert_eq!(
        problems(&rules, &team),
        vec![error(
            None,
            ViolationKind::ExtraHeight {
                used: 51,
                total: 50
            }
        )]
    );
}

/// A player over the pool's `max` takes no bracket: the `Height`
/// violation and the `BracketQuota` his tall slot leaves (the other 210
/// still draws his 25).
#[test]
fn a_player_over_the_max_is_a_height_violation() {
    let rules = rules();
    let team = edited(&GIGA_210, GGSS, 2, |p| p.height = 211);
    assert_eq!(
        problems(&rules, &team),
        vec![
            error(Some(2), ViolationKind::Height { height: 211 }),
            quota("giga", "tall", 6, 7),
        ]
    );
}

/// A player below every quota bracket takes none: a manlet at 174 is a
/// `Height` violation and the manlet quota comes up one short.
#[test]
fn a_player_below_the_brackets_is_a_height_violation() {
    let rules = rules();
    let team = edited(&GIGA, GGSS, 3, |p| p.height = 174);
    assert_eq!(
        problems(&rules, &team),
        vec![
            error(Some(3), ViolationKind::Height { height: 174 }),
            quota("giga", "manlet", 5, 6),
        ]
    );
}

/// The pool's tallest-first placement: 8x 185, 9x 180, 6x 175 — the
/// eighth 185 (the keepers counted) takes a mid slot and draws 5, inside
/// the 50.
#[test]
fn a_tall_player_fills_a_mid_slot() {
    let rules = rules();
    let team = edited(&GIGA, GGSS, 8, |p| p.height = 185);
    assert_eq!(problems(&rules, &team), vec![]);
}

/// A medal count no system has: three golds and a silver match none —
/// one `MedalSystem` on the team, and the regulars' count left alone (19
/// is 23 less the four named-tier players), so no `TierCount`.
#[test]
fn a_medal_count_no_system_has_is_one_violation() {
    let rules = rules();
    let medals: &[(u8, &str)] = &[(2, "gold"), (5, "gold"), (10, "gold"), (9, "silver")];
    assert_eq!(
        problems(&rules, &legal_team(&GIGA, medals)),
        vec![error(
            None,
            ViolationKind::MedalSystem {
                systems: ["GGG", "GGSS", "GSSSS", "SSSSSS"]
                    .map(String::from)
                    .to_vec(),
            }
        )]
    );
}

/// Any stat above its target is a violation (the rules page's cap,
/// stricter than the checker's warning threshold): a regular with one
/// stat at 78, one `Stat` at target 77.
#[test]
fn a_stat_one_over_the_target_is_a_violation() {
    let rules = rules();
    let dribbling = stat("dribbling");
    let team = edited(&GIGA, GGSS, 12, |p| {
        p.abilities[usize::from(dribbling.0)] = Some(78)
    });
    assert_eq!(
        problems(&rules, &team),
        vec![error(
            Some(12),
            ViolationKind::Stat {
                stat: dribbling,
                value: 78,
                target: 77,
                at_most: true,
            }
        )]
    );
}
