//! The grass-drop position rule's tests: the real formations the
//! rule's edges were measured on with the dragged players' overalls by
//! the editor's formula on their records, and synthetic
//! overalls for the pick's own parts (the free-candidate skip, the
//! tie, the keep and the missing row).

use super::*;
use crate::pitch::drop_rule::dropped_position;

/// `dropped_position` at each `(x, y)` of `drops` answered by its
/// expected position, one assertion per drop named `case (x, y)`.
fn check(
    formation: &Formation,
    slot: usize,
    overalls: &[u8; 13],
    case: &str,
    drops: &[(u8, u8, &str)],
) {
    for &(x, y, expected) in drops {
        assert_eq!(
            dropped_position(formation, slot, x, y, Some(overalls)),
            position_byte(expected),
            "{case} ({x}, {y}): {expected} expected",
        );
    }
}

// Team 701's preset 1 formation 1 (PES 2017): the dragged slot and his
// thirteen overalls (GK to CF; every one the game showed equals them),
// then each drop and its position, on either side of each edge the
// game's drag panel was seen to cross.
#[test]
fn the_drop_rule_on_701s_preset_1_formation_1() {
    let formation = formation_701_1();
    check(
        &formation,
        2,
        &[62, 83, 73, 73, 73, 69, 68, 68, 71, 68, 68, 70, 70],
        "uv_up",
        &[
            (40, 15, "CB"),
            (40, 16, "DMF"),
            (40, 25, "DMF"),
            (40, 26, "AMF"),
            (40, 34, "AMF"),
            (40, 35, "SS"),
        ],
    );
    check(
        &formation,
        9,
        &[80, 90, 94, 94, 93, 89, 87, 87, 91, 88, 88, 89, 101],
        "bl_down",
        &[
            (38, 40, "CF"),
            (38, 39, "SS"),
            (38, 35, "SS"),
            (38, 34, "AMF"),
        ],
    );
    check(
        &formation,
        3,
        &[62, 70, 73, 85, 73, 69, 68, 68, 71, 68, 68, 70, 70],
        "rb_left",
        &[(84, 10, "RB"), (83, 10, "CB")],
    );
    check(
        &formation,
        4,
        &[62, 70, 85, 73, 73, 69, 68, 68, 71, 68, 68, 70, 70],
        "lb_right",
        &[(20, 10, "LB"), (21, 10, "CB")],
    );
    check(
        &formation,
        9,
        &[80, 90, 94, 94, 93, 89, 87, 87, 91, 88, 88, 89, 101],
        "bl_left",
        &[(21, 42, "CF"), (20, 42, "SS")],
    );
    check(
        &formation,
        5,
        &[73, 82, 85, 85, 96, 81, 79, 79, 83, 80, 80, 82, 82],
        "dmf_down",
        &[(52, 16, "DMF"), (52, 15, "CB")],
    );
    check(
        &formation,
        8,
        &[80, 87, 92, 92, 92, 89, 87, 87, 101, 87, 87, 94, 88],
        "amf_left",
        &[(21, 34, "AMF"), (20, 34, "CMF")],
    );
    check(
        &formation,
        5,
        &[73, 82, 85, 85, 96, 81, 79, 79, 83, 80, 80, 82, 82],
        "gk_box",
        &[(54, 2, "DMF")],
    );
}

// Team 718's preset 1 formations 1 and 3 (PES 2021): the dragged slot
// and his thirteen overalls (GK to CF; within 1 of those the game
// showed), then each drop and its position. Near a teammate's marker
// the game showed that teammate's position instead (the swap the drop
// on a marker makes), so `cmf_left`'s (31, 26) read LMF there.
#[test]
fn the_drop_rule_on_718s_preset_1_formations() {
    let party: [u8; 13] = [76, 85, 88, 88, 88, 84, 95, 82, 85, 86, 82, 84, 84];
    let zone: [u8; 13] = [73, 82, 85, 85, 85, 93, 79, 79, 83, 80, 80, 82, 82];
    let formation = formation_718_1();
    check(
        &formation,
        8,
        &party,
        "lmf_up",
        &[(20, 34, "LMF"), (20, 35, "LWF")],
    );
    check(
        &formation,
        8,
        &party,
        "lmf_right",
        &[(20, 26, "LMF"), (21, 26, "AMF"), (40, 26, "AMF")],
    );
    check(
        &formation,
        2,
        &zone,
        "cmf_left",
        &[(31, 26, "CMF"), (20, 26, "CMF"), (5, 26, "CMF")],
    );
    let formation = formation_718_3();
    check(
        &formation,
        8,
        &party,
        "lmf3_right",
        &[(21, 26, "AMF"), (31, 26, "AMF")],
    );
}

// Slot 2's CB dropped at (40, 20): the point's default is CMF and its
// candidate DMF, and his CB is neither — with both rated 80 the
// default stands; at 81 the candidate wins.
#[test]
fn the_default_wins_a_tie() {
    let formation = formation_701_1();
    let mut overalls = [70; 13];
    overalls[usize::from(position_byte("CMF"))] = 80;
    overalls[usize::from(position_byte("DMF"))] = 80;
    assert_eq!(
        dropped_position(&formation, 2, 40, 20, Some(&overalls)),
        position_byte("CMF"),
        "a tie keeps the band's default"
    );
    overalls[usize::from(position_byte("DMF"))] = 81;
    assert_eq!(
        dropped_position(&formation, 2, 40, 20, Some(&overalls)),
        position_byte("DMF"),
        "a higher-rated candidate wins"
    );
}

// Slot 3's DMF dropped at (90, 20): RMF's area holds the point but
// slot 7 is the formation's RMF, so the candidate is none and the
// default CMF is taken — with slot 7 turned CMF, RMF is free and wins.
#[test]
fn a_wide_candidate_a_teammate_holds_is_skipped() {
    let mut formation = formation_718_1();
    let mut overalls = [70; 13];
    overalls[usize::from(position_byte("RMF"))] = 90;
    overalls[usize::from(position_byte("CMF"))] = 80;
    assert_eq!(
        dropped_position(&formation, 3, 90, 20, Some(&overalls)),
        position_byte("CMF"),
        "the held RMF is skipped: the default stands"
    );
    formation.players[7].position = position_byte("CMF");
    assert_eq!(
        dropped_position(&formation, 3, 90, 20, Some(&overalls)),
        position_byte("RMF"),
        "the RMF slot freed, the higher-rated candidate wins"
    );
}

// Slot 5's DMF dropped at (40, 20): the point's candidate is his own
// DMF, so he keeps it although the default CMF rates higher.
#[test]
fn the_current_position_is_kept_where_it_is_a_candidate() {
    let formation = formation_701_1();
    let mut overalls = [70; 13];
    overalls[usize::from(position_byte("CMF"))] = 80;
    assert_eq!(
        dropped_position(&formation, 5, 40, 20, Some(&overalls)),
        position_byte("DMF"),
        "his own DMF is the point's candidate"
    );
}

// A slot whose row reads no overalls keeps his position: no overall,
// no pick.
#[test]
fn no_overalls_keep_the_position() {
    let formation = formation_701_1();
    assert_eq!(
        dropped_position(&formation, 2, 40, 20, None),
        position_byte("CB"),
        "no overalls: his CB is kept"
    );
}
