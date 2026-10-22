//! The drag-and-drop tests: marker and row drops, the exchanges and
//! swaps, and the grass, own-chip and off-pitch releases.

use pes_savefile::model::instruction::Instruction;
use pes_savefile::schema::instruction;
use pes_version::PesVersion;

use super::*;
use crate::test_support::{Headless, fixture_rows, fixture_tactics, roster_rows};

// The draft's "Move on pitch": a marker released on the grass writes
// the snapped drop point to its slot's x and y, and only there.
#[test]
fn a_marker_dragged_to_grass_writes_the_snapped_point() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    let index = 4;
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let from = marker_centre(&nodes, formation.players[index]);
    let onto = grass_point(canvas, &formation.players);
    let (x, y) = snap(onto, canvas.min);
    let slot = formation.players[index];
    assert_ne!((slot.x, slot.y), (x, y), "premise: a real move");

    let changed = window.drag_pitch(
        from,
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    let mut expect = formation.clone();
    expect.players[index].x = x;
    expect.players[index].y = y;
    assert_eq!(
        tactics.presets[0].formations[0], expect,
        "only the dragged slot's x and y moved"
    );
}

// The decision "A pitch player dropped on another exchanges the two
// starters" (DECISIONS 2026-10-22): the two `starting_eleven` entries
// trade, the formation slots staying.
#[test]
fn a_marker_dropped_on_another_marker_exchanges_the_two_starters() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[0].formations[0].clone();
    let (from, onto) = (3, 6);
    let dragged = before.starting_eleven[from];
    let dropped = before.starting_eleven[onto];
    assert_ne!(dragged, dropped, "premise: two different slots");
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let at = marker_centre(&nodes, formation.players[from]);
    let over = marker_centre(&nodes, formation.players[onto]);
    assert!(
        formation.players[onto + 1..].iter().all(|slot| {
            !marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas).contains(over)
        }),
        "premise: no later marker covers the drop point"
    );

    let changed = window.drag_pitch(
        at,
        over,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    assert_eq!(tactics.starting_eleven[from], dropped);
    assert_eq!(tactics.starting_eleven[onto], dragged);
    assert_eq!(tactics.presets, before.presets, "the formation slots stay");
    assert_eq!(tactics.set_pieces, before.set_pieces, "no role moves");
    assert_eq!(
        tactics.players_to_join_attack, before.players_to_join_attack,
        "the join list stays"
    );
}

// The same decision's other half: a marker released off the pitch
// stays where he was.
#[test]
fn a_marker_released_off_the_pitch_changes_nothing() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[0].formations[0].clone();
    let index = 3;
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let from = marker_centre(&nodes, formation.players[index]);
    // Right of the canvas but inside the window.
    let onto = egui::pos2(canvas.max.x + 20.0, canvas.center().y);
    assert!(!canvas.contains(onto), "premise: a release off the pitch");
    assert!(onto.x < 600.0, "premise: still inside the window");

    let changed = window.drag_pitch(
        from,
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(!changed, "a release off the pitch writes nothing");
    assert_eq!(tactics, before, "and nothing moved");
}

// An eleven row released on the grass is no drop (the draft's canvas
// drop returns when it hits no marker): the grass move is
// `LineupDrag::Marker`'s alone.
#[test]
fn an_eleven_row_released_on_the_grass_changes_nothing() {
    let mut window = Headless::sized(600.0, 1100.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[0].formations[0].clone();
    let (nodes, _) =
        window.pitch_and_lineup(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let onto = grass_point(canvas, &formation.players);

    let changed = window.drag_pitch_and_lineup(
        row_centre(&nodes, 3),
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(!changed, "an eleven row on the grass is no drop");
    assert_eq!(tactics, before, "and nothing moved");
}

// A starter released on his own chip moves too (the arm's "or on his
// own chip"): 4 units right still lands inside the 42 px chip, where
// the drop is his own and the grass arm writes the snapped point.
#[test]
fn a_marker_released_on_his_own_chip_moves() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    // An outfield slot whose x + 4 stays on the grid and whose +12 px
    // drop point covers no other marker's chip.
    let (index, from, onto) = (0..11)
        .filter(|&index| formation.players[index].position != 0)
        .filter_map(|index| {
            let slot = formation.players[index];
            let centre = pitch_point(canvas.min, slot.x, slot.y);
            let onto = centre + egui::vec2(4.0 * UNIT.x, 0.0);
            (slot.x <= 100
                && formation.players.iter().enumerate().all(|(other, slot)| {
                    other == index
                        || !marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas)
                            .contains(onto)
                }))
            .then_some((index, centre, onto))
        })
        .next()
        .expect("some outfield slot can move 4 units right");
    let slot = formation.players[index];
    assert!(
        marker_rect(from, canvas).contains(onto),
        "premise: the release is still on his own chip"
    );

    let changed = window.drag_pitch(
        from,
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    let moved = tactics.presets[0].formations[0].players[index];
    assert_eq!(moved.x, slot.x + 4, "x grew by the 4 dragged units");
    assert_eq!(moved.y, slot.y, "y stayed");
}

// The plan's "The goalkeeper's slot is fixed": his marker clicks but
// is no drag source, so a press-drag-release on it writes nothing.
#[test]
fn the_keepers_marker_cannot_be_dragged() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    let keeper = formation
        .players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");
    let before = tactics.clone();
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let from = marker_centre(&nodes, formation.players[keeper]);
    let onto = grass_point(canvas, &formation.players);

    let changed = window.drag_pitch(
        from,
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(!changed, "the keeper's spot is fixed");
    assert_eq!(tactics, before, "and nothing wrote");
    assert_eq!(view.popover, None, "a dragged press is not a click");
}

// The widget's contract (the plan's "`libs/team_widgets`"): an idle
// frame and a hover over the canvas report no change and edit nothing.
#[test]
fn an_idle_frame_and_a_hover_change_nothing() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let (nodes, changed) =
        window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    assert!(!changed, "an idle frame reports no change");
    assert_eq!(tactics, before, "and edits nothing");

    let (_, changed) = window.pitch(
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
        vec![egui::Event::PointerMoved(canvas_rect(&nodes).center())],
    );
    assert!(!changed, "a hover reports no change");
    assert_eq!(tactics, before, "and edits nothing");
}

// The draft's canvas drop: a bench row released on a marker swaps him
// in through `swap_in` — the taker he replaces follows to the bench.
#[test]
fn a_bench_row_dropped_on_a_marker_swaps_him_in() {
    let mut window = Headless::sized(600.0, 1100.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let index = 5;
    let out = tactics.starting_eleven[index];
    let inn = tactics.bench_order[0];
    assert_ne!(out, inn, "premise: two different slots");
    tactics.set_pieces.penalty = out;
    let (nodes, _) =
        window.pitch_and_lineup(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let formation = tactics.presets[0].formations[0].clone();
    let onto = marker_rect(
        pitch_point(
            canvas.min,
            formation.players[index].x,
            formation.players[index].y,
        ),
        canvas,
    )
    .center();

    let changed = window.drag_pitch_and_lineup(
        row_centre(&nodes, 11),
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    assert_eq!(
        tactics.starting_eleven[index], inn,
        "the bench player starts"
    );
    assert_eq!(tactics.bench_order[0], out, "the starter sits");
    assert_eq!(
        tactics.set_pieces.penalty, inn,
        "the taker followed the swap"
    );
}

// An XI row released on a marker exchanges the two starters' slots —
// both stay on the pitch, so no role moves.
#[test]
fn a_starter_row_dropped_on_a_marker_exchanges_the_two() {
    let mut window = Headless::sized(600.0, 1100.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let index = 5;
    // A non-identity eleven: an arm writing indices would look the
    // same as an exchange on the fixture's [0, 1, 2, ...].
    tactics.starting_eleven.swap(2, 9);
    tactics.starting_eleven.swap(5, 8);
    let out = tactics.starting_eleven[2];
    let inn = tactics.starting_eleven[5];
    assert_ne!(out, inn, "premise: two different slots");
    let (nodes, _) =
        window.pitch_and_lineup(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let formation = tactics.presets[0].formations[0].clone();
    let onto = marker_rect(
        pitch_point(
            canvas.min,
            formation.players[index].x,
            formation.players[index].y,
        ),
        canvas,
    )
    .center();

    let changed = window.drag_pitch_and_lineup(
        row_centre(&nodes, 2),
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    assert_eq!(tactics.starting_eleven[2], inn);
    assert_eq!(tactics.starting_eleven[5], out);
}

// `swap_in`'s instruction gate: the outgoing starter's slot in a target
// byte repoints only where the version's decoded instruction takes one
// of ours (`takes_player`) — a byte under an instruction that takes none
// (as real saves hold) or a held Tight Marking (an opponent's) stays.
#[test]
fn a_swap_repoints_only_targets_of_instructions_that_take_a_player() {
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let out = tactics.starting_eleven[5];
    let inn = tactics.bench_order[0];
    assert_ne!(out, inn, "premise: two different slots");
    let encode = |inst| instruction::encode(PesVersion::Pes21, inst).expect("PES 21 stores it");
    let attack = tactics.presets[0]
        .attack_instructions
        .as_mut()
        .expect("PES 21 has instructions");
    attack[0].instruction = encode(Instruction::CounterTarget);
    attack[0].player = out;
    attack[1].instruction = encode(Instruction::TikiTaka);
    attack[1].player = out;
    let defence = tactics.presets[0]
        .defence_instructions
        .as_mut()
        .expect("PES 21 has instructions");
    defence[0].instruction = encode(Instruction::TightMarking);
    defence[0].player = out;

    swap_in(PesVersion::Pes21, &mut tactics, 5, 0);

    let attack = &tactics.presets[0]
        .attack_instructions
        .as_ref()
        .expect("instructions");
    assert_eq!(
        attack[0].player, inn,
        "a Counter Target's target follows the swap"
    );
    assert_eq!(
        attack[1].player, out,
        "Tiki-Taka takes no player: the byte stays"
    );
    let defence = &tactics.presets[0]
        .defence_instructions
        .as_ref()
        .expect("instructions");
    assert_eq!(
        defence[0].player, out,
        "a held Tight Marking marks an opponent: it stays"
    );
}
