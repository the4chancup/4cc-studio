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
    let centres = centres_of(canvas, &formation.players);
    let from = marker_point(&nodes, &formation.players, index);
    // An off-grid drop, as every real one is: the release lands
    // `+vec2(1.6, -3.5)` past a free grid point — 0.57 and 0.52 of a
    // unit, so the written unit is the rounded (x + 1, y + 1), which a
    // `.floor()` snap would read (x, y). The release point itself stays
    // on the canvas and inside no other marker (another's would claim
    // the drop for a swap).
    let slot = formation.players[index];
    let (x, y, onto) = (0..=GRID_X)
        .flat_map(|x| (0..=GRID_Y).map(move |y| (x, y)))
        .map(|(x, y)| {
            (
                x,
                y,
                badge_centre(canvas.min, FormationSlot { position: 1, x, y })
                    + egui::vec2(1.6, -3.5),
            )
        })
        .find(|(x, y, point)| {
            *x < GRID_X
                && *y < GRID_Y
                && canvas.contains(*point)
                && (*x + 1 != slot.x || *y + 1 != slot.y)
                && centres.iter().enumerate().all(|(other, centre)| {
                    other == index || !marker_rect(*centre, canvas).contains(*point)
                })
        })
        .map(|(x, y, point)| (x + 1, y + 1, point))
        .expect("an off-grid point free of other markers");

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

    // A second drop below the half-mark on both axes: `+vec2(0.85,
    // -2.04)` past a free grid point is 0.30 of a unit each way, so the
    // written unit is that point's own (x, y), which a `.ceil()` snap
    // would read (x + 1, y + 1).
    let formation = tactics.presets[0].formations[0].clone();
    let centres = centres_of(canvas, &formation.players);
    let from = marker_point(&nodes, &formation.players, index);
    let slot = formation.players[index];
    let (x, y, onto) = (0..GRID_X)
        .flat_map(|x| (0..GRID_Y).map(move |y| (x, y)))
        .map(|(x, y)| {
            (
                x,
                y,
                badge_centre(canvas.min, FormationSlot { position: 1, x, y })
                    + egui::vec2(0.85, -2.04),
            )
        })
        .find(|(x, y, point)| {
            canvas.contains(*point)
                && (*x != slot.x || *y != slot.y)
                && centres
                    .iter()
                    .all(|centre| !marker_rect(*centre, canvas).contains(*point))
        })
        .expect("an off-grid point free of markers");

    // The pointer rests on his new spot a frame first: egui hit-tests a
    // press against the last frame's widgets, and the drop's frame
    // registered his marker at the spot he left.
    window.pitch(
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
        vec![egui::Event::PointerMoved(from)],
    );
    let changed = window.drag_pitch(
        from,
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the second drop changed the record");
    let mut expect = formation;
    expect.players[index].x = x;
    expect.players[index].y = y;
    assert_eq!(
        tactics.presets[0].formations[0], expect,
        "the point below the half-mark rounds down"
    );
}

// The decision "A pitch player dropped on another swaps the two slots in
// the shown formation, as the game's drag" (DECISIONS 2026-10-27): the
// two slots of preset 1's Kick-off exchange position, x and y, each
// player staying in his slot; the starting eleven, the preset's other
// two formations and presets 2 and 3 stay.
#[test]
fn a_marker_dropped_on_another_marker_swaps_the_two_slots_in_the_shown_formation() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[0].formations[0].clone();
    let (from, onto) = (3, 6);
    assert_ne!(
        formation.players[from], formation.players[onto],
        "premise: two different slots"
    );
    for (preset, formation) in [(0, 1), (0, 2), (1, 0), (2, 0)] {
        let players = before.presets[preset].formations[formation].players;
        assert_ne!(
            players[from], players[onto],
            "premise: a swap there too would show (preset {preset}, formation {formation})"
        );
    }
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    // `over` is inside `onto`'s box, free of every marker drawn after it
    // (`marker_point`), so the topmost-marker drop resolves to `onto`.
    let at = marker_point(&nodes, &formation.players, from);
    let over = marker_point(&nodes, &formation.players, onto);

    let changed = window.drag_pitch(
        at,
        over,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    let mut expect = before.clone();
    expect.presets[0].formations[0].players.swap(from, onto);
    assert_eq!(
        tactics, expect,
        "only Kick-off's two slots exchanged: the eleven, the other formations and presets stay"
    );
}

// Round S3's S3b-1: the swap writes the shown formation of the shown
// preset, not a fixed one: on preset 2's In-possession the two slots
// exchange there alone, preset 1's Kick-off (where a swap would show too)
// and every other formation unchanged.
#[test]
fn a_marker_dropped_on_another_swaps_in_the_shown_preset_and_formation() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView {
        preset: 1,
        formation: 1,
        ..TacticsView::default()
    };
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[1].formations[1].clone();
    let (from, onto) = (3, 6);
    assert_ne!(
        formation.players[from], formation.players[onto],
        "premise: two different slots in the shown formation"
    );
    let kick_off = before.presets[0].formations[0].players;
    assert_ne!(
        kick_off[from], kick_off[onto],
        "premise: a swap on preset 1's Kick-off would show"
    );
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let at = marker_point(&nodes, &formation.players, from);
    let over = marker_point(&nodes, &formation.players, onto);

    let changed = window.drag_pitch(
        at,
        over,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    let mut expect = before;
    expect.presets[1].formations[1].players.swap(from, onto);
    assert_eq!(
        tactics, expect,
        "only preset 2's In-possession two slots exchanged: the eleven and every other formation stay"
    );
}

// The keeper is never a swap target (the plan's pitch-drop bullet, his
// slot's position and spot being fixed): a marker held over him draws no
// swap mark, and released there moves to the snapped point as on the
// grass, the starting eleven and his slot unchanged.
#[test]
fn a_marker_released_on_the_keeper_moves_to_the_snapped_point() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let players = before.presets[0].formations[0].players;
    let keeper = players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let over = marker_point(&nodes, &players, keeper);
    // The dragged starter: an outfield one whose own box is off the hold,
    // the hold in no outfield marker's box but the keeper's.
    let index = (0..11)
        .find(|&index| {
            players[index].position != 0
                && !marker_rect(centres[index], canvas).contains(over)
                && snap(over, canvas.min) != (players[index].x, players[index].y)
        })
        .expect("an outfield starter off the keeper's marker");
    assert!(
        (0..11).all(|other| {
            other == keeper || !marker_rect(centres[other], canvas).contains(over)
        }),
        "premise: the hold is on the keeper's marker alone"
    );
    let from = marker_point(&nodes, &players, index);

    window.pitch(
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
        pointer_at(from, true),
    );
    let (_, shapes) = window.frame(vec![egui::Event::PointerMoved(over)], |ui| {
        pitch(
            ui,
            "pitch",
            PesVersion::Pes21,
            &mut tactics,
            &mut view,
            &roster,
        );
    });
    assert!(
        !shapes.iter().any(|clipped| matches!(
            &clipped.shape,
            egui::Shape::LineSegment { stroke, .. } if stroke.color == SELECTED_CORNERS
        )),
        "no swap mark over the keeper"
    );
    let (_, changed) = window.pitch(
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
        pointer_at(over, false),
    );
    assert!(changed, "the drop changed the record");
    let mut expect = before.clone();
    let (x, y) = snap(over, canvas.min);
    expect.presets[0].formations[0].players[index].x = x;
    expect.presets[0].formations[0].players[index].y = y;
    assert_eq!(
        tactics, expect,
        "he moved to the snapped point; the eleven and the keeper's slot stay"
    );
}

/// A marker `index` of preset 1's Kick-off whose box's left edge has
/// free grass `reach` points out: a press `inset` points inside that
/// edge, level with the box's centre, no marker drawn after his covering
/// it, and its release `reach` points left on no marker. `None` when no
/// outfield marker has one.
fn edge_press(
    canvas: egui::Rect,
    players: &[FormationSlot; 11],
    inset: f32,
    reach: f32,
) -> Option<(usize, egui::Pos2, egui::Pos2)> {
    let centres = centres_of(canvas, players);
    let order = draw_order(&centres);
    (0..11).find_map(|index| {
        let rect = marker_rect(centres[index], canvas);
        let press = egui::pos2(rect.left() + inset, rect.center().y);
        let release = press - egui::vec2(reach, 0.0);
        let after = draw_position(canvas, players, index);
        (players[index].position != 0
            && canvas.contains(release)
            && order[after + 1..]
                .iter()
                .all(|&other| !marker_rect(centres[other], canvas).contains(press))
            && centres
                .iter()
                .all(|centre| !marker_rect(*centre, canvas).contains(release)))
        .then_some((index, press, release))
    })
}

// Round S2's S2b-1: a release egui reports as the pressed marker's click
// (the pointer moved no farther than its click distance, 6) is a click,
// never a drop. egui marks the marker dragged once the pointer is past
// its hit radius (5) outside him, so a press 0.25 inside his edge
// released 5.5 off on the grass, a 5.75 move, is both his click and a
// drop: it opens his popover and writes nothing.
#[test]
fn a_release_within_the_click_distance_is_a_click_not_a_drop() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let players = before.presets[0].formations[0].players;
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let (index, press, release) =
        edge_press(canvas, &players, 0.25, 5.75).expect("a marker with grass at its left edge");
    assert_ne!(
        snap(release, canvas.min),
        (players[index].x, players[index].y),
        "premise: a drop there would move him"
    );

    let changed = window.drag_pitch(
        press,
        release,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(!changed, "a click writes nothing");
    assert_eq!(tactics, before, "his slot's x and y stay");
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(index),
        "the release is his click: the popover opens"
    );
}

// Round S3's S3b-2: the click guard covers the swap too. Kick-off's
// slot 1 sits a stored unit left of and 8 below slot 5, so his box,
// drawn after 5's, overlaps its lower part and reaches 2.8 points past
// its left edge: a press on 5 just above 1's top and a release 2.8
// points away inside 1's box (outside 5's, so egui marks 5 dragged) is
// 5's click, never a swap with 1.
#[test]
fn a_release_within_the_click_distance_over_a_neighbour_is_a_click() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let players = before.presets[0].formations[0].players;
    let (index, neighbour) = (5, 1);
    assert_ne!(
        players[index], players[neighbour],
        "premise: a swap of the two would show"
    );
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let own = marker_rect(centres[index], canvas);
    let other = marker_rect(centres[neighbour], canvas);
    let press = egui::pos2(own.left() + 1.0, other.top() - 1.0);
    let release = egui::pos2(own.left() - 1.0, other.top() + 1.0);
    let under = |point: egui::Pos2| -> Vec<usize> {
        (0..11)
            .filter(|&m| marker_rect(centres[m], canvas).contains(point))
            .collect()
    };
    assert_eq!(
        under(press),
        vec![index],
        "premise: the press is on his box alone"
    );
    assert_eq!(
        under(release),
        vec![neighbour],
        "premise: the release is on the neighbour's box alone"
    );
    assert!(
        (release - press).length() <= 4.0,
        "premise: the release is within the click distance"
    );

    let changed = window.drag_pitch(
        press,
        release,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(!changed, "a click writes nothing");
    assert_eq!(tactics, before, "no swap with the neighbour");
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(index),
        "the release is his click: the popover opens"
    );
}

// The same finding's other half: past the click distance the release is
// the drop, the snapped point written and no popover opened.
#[test]
fn a_release_past_the_click_distance_is_a_drop_not_a_click() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let players = before.presets[0].formations[0].players;
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let (index, press, release) =
        edge_press(canvas, &players, 1.0, 10.0).expect("a marker with grass at its left edge");
    let (x, y) = snap(release, canvas.min);
    assert_ne!(
        (x, y),
        (players[index].x, players[index].y),
        "premise: the drop moves him"
    );

    let changed = window.drag_pitch(
        press,
        release,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    let mut expect = before.clone();
    expect.presets[0].formations[0].players[index].x = x;
    expect.presets[0].formations[0].players[index].y = y;
    assert_eq!(tactics, expect, "the snapped point written");
    assert_eq!(view.popover, None, "no popover opened");
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
    let from = marker_point(&nodes, &formation.players, index);
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
    let onto = grass_point(canvas, &formation.players, egui::Rect::NOTHING);

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

// A starter released on his own marker moves too (the arm's "or on his
// own marker"): 4 units right still lands inside the marker box, where
// the drop is his own and the grass arm writes the snapped point.
#[test]
fn a_marker_released_on_his_own_marker_moves() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &formation.players);
    let order = draw_order(&centres);
    // An outfield slot whose x + 4 stays on the grid and whose drop
    // point covers no marker but his own (another's would claim the
    // release for a swap).
    let (index, onto) = (0..11)
        .filter(|&index| formation.players[index].position != 0)
        .filter_map(|index| {
            let slot = formation.players[index];
            let onto = badge_centre(canvas.min, slot) + egui::vec2(4.0 * SPOT_XS, 0.0);
            (slot.x <= 100
                && order.iter().all(|&other| {
                    other == index || !marker_rect(centres[other], canvas).contains(onto)
                }))
            .then_some((index, onto))
        })
        .next()
        .expect("some outfield slot can move 4 units right");
    let from = marker_point(&nodes, &formation.players, index);
    let slot = formation.players[index];
    assert!(
        marker_rect(centres[index], canvas).contains(onto),
        "premise: the release is still on his own marker"
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

// A press on the keeper where his marker covers another's is his (the
// plan's "a press or a drop hits the marker drawn on top" and "The
// goalkeeper's slot is fixed"): he senses click-and-drag like the
// others so egui's hit test gives him both hits — a press-drag-release
// on him moves neither and writes nothing; his release is his click.
#[test]
fn a_press_on_the_keeper_over_another_marker_moves_neither() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let keeper = tactics.presets[0].formations[0]
        .players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");
    // The first outfield slot moved so its marker box covers the
    // keeper's (badge centre about (159.8, 323.0) to his (160, 358.5)).
    let under = tactics.presets[0].formations[0]
        .players
        .iter()
        .position(|slot| slot.position != 0)
        .expect("an outfield slot");
    tactics.presets[0].formations[0].players[under].x = 52;
    tactics.presets[0].formations[0].players[under].y = 6;
    let before = tactics.clone();
    let players = tactics.presets[0].formations[0].players;

    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let point = canvas.min + egui::vec2(160.0, 330.0);
    assert!(
        marker_rect(centres[keeper], canvas).contains(point)
            && marker_rect(centres[under], canvas).contains(point),
        "premise: the point is inside both boxes"
    );
    assert!(
        draw_position(canvas, &players, keeper) > draw_position(canvas, &players, under),
        "premise: the keeper paints on top"
    );

    let changed = window.drag_pitch(
        point,
        grass_point(canvas, &players, egui::Rect::NOTHING),
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(!changed, "neither marker moves");
    assert_eq!(tactics, before, "and nothing wrote");
    // His marker never moves, so the release is his click: the popover.
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(keeper),
        "the release is the keeper's click"
    );
}

// The plan's "The goalkeeper's slot is fixed": his marker is no drag
// source, so a press-drag-release on it writes nothing — his release is
// his click, which opens the popover.
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
    let from = marker_point(&nodes, &formation.players, keeper);
    let onto = grass_point(canvas, &formation.players, egui::Rect::NOTHING);

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
    // His marker never moves, so the release is his click: the popover.
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(keeper),
        "the release is the keeper's click"
    );
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
    let formation = tactics.presets[0].formations[0].clone();
    let onto = marker_point(&nodes, &formation.players, index);

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
    let formation = tactics.presets[0].formations[0].clone();
    let onto = marker_point(&nodes, &formation.players, index);

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

// The drop reads the swap mark's pick: a starter's marker moved onto the
// dragged one's own spot is the release's target — the dragged one's own
// spot is never it (the draft's `hitMarker` `skip`), so a release on the
// overlap swaps the two slots instead of moving the dragged one.
#[test]
fn a_release_on_a_marker_under_the_dragged_ones_spot_swaps() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    // `other`'s marker is moved onto `index`'s own spot; `index` paints
    // after him (ties keep the slot order), so the press hits `index`.
    let (index, other) = (4, 2);
    let players = &mut tactics.presets[0].formations[0].players;
    assert!(
        players[index].position != 0 && players[other].position != 0,
        "premise: two outfield slots"
    );
    players[other].x = players[index].x;
    players[other].y = players[index].y;
    assert_ne!(
        players[other].position, players[index].position,
        "premise: the two slots differ in position, so their swap shows"
    );
    let before = tactics.clone();

    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let players = tactics.presets[0].formations[0].players;
    let centres = centres_of(canvas, &players);
    let order = draw_order(&centres);
    assert!(
        draw_position(canvas, &players, index) > draw_position(canvas, &players, other),
        "premise: `index` paints after `other` at their shared spot"
    );
    // The release 10 points right of their shared centre: past the click
    // distance (a drop, not a click) and still inside both boxes.
    let onto = centres[index] + egui::vec2(10.0, 0.0);
    let covering: Vec<usize> = order
        .iter()
        .rev()
        .copied()
        .filter(|&m| m != index && marker_rect(centres[m], canvas).contains(onto))
        .collect();
    assert_eq!(
        covering.first(),
        Some(&other),
        "premise: `other` is the topmost other marker under the release"
    );
    let from = marker_point(&nodes, &players, index);

    let changed = window.drag_pitch(
        from,
        onto,
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    let mut expect = before.clone();
    expect.presets[0].formations[0].players.swap(index, other);
    assert_eq!(
        tactics, expect,
        "the two slots exchanged, the starting eleven unchanged"
    );
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
