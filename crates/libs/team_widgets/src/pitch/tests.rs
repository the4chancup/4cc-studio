use egui::accesskit::{self, Role};
use pes_version::PesVersion;

use super::*;
use crate::tactics;
use crate::test_support::{
    Headless, centre, controls_right_of, entry, entry_id, fixture_rows, fixture_tactics,
    pointer_at, roster_rows,
};

impl Headless {
    /// One frame of `lineup` over `tactics`, `view` and `roster`: the
    /// frame's AccessKit nodes and whether it changed `tactics`.
    pub(crate) fn lineup(
        &mut self,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame(events, |ui| {
            changed = Some(super::lineup(ui, "lineup", tactics, view, roster));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// A drag from `from` to `onto` over the widget (press, move,
    /// release): whether the drop changed `tactics`.
    pub(crate) fn drag_lineup(
        &mut self,
        from: egui::Pos2,
        onto: egui::Pos2,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> bool {
        self.lineup(tactics, view, roster, pointer_at(from, true));
        self.lineup(tactics, view, roster, vec![egui::Event::PointerMoved(onto)]);
        let (_, changed) = self.lineup(tactics, view, roster, pointer_at(onto, false));
        changed
    }

    /// One frame of `pitch` over `tactics`, `view` and `roster`: the
    /// frame's AccessKit nodes and whether it changed `tactics`. The
    /// armed mode's note heads the frame, as the host draws it.
    pub(crate) fn pitch(
        &mut self,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame(events, |ui| {
            tactics::assign_note(ui, view);
            changed = Some(super::pitch(ui, "pitch", tactics, view, roster));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// `pitch` keeping the nodes' AccessKit ids (for
    /// `AccessKitActionRequest` events on a combo's popup items).
    fn pitch_ids(
        &mut self,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> Vec<(accesskit::NodeId, accesskit::Node)> {
        let (nodes, _) = self.frame_at_ids(None, vec![], |ui| {
            tactics::assign_note(ui, view);
            super::pitch(ui, "pitch", tactics, view, roster);
        });
        nodes
    }

    /// An AccessKit `Click` action on `id`: a click delivered as the
    /// accessibility event, for widgets a pointer click does not
    /// reach (a combo's popup items).
    fn action_click_pitch(
        &mut self,
        id: accesskit::NodeId,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.pitch(
            tactics,
            view,
            roster,
            vec![egui::Event::AccessKitActionRequest(
                accesskit::ActionRequest {
                    action: accesskit::Action::Click,
                    target_tree: accesskit::TreeId::ROOT,
                    target_node: id,
                    data: None,
                },
            )],
        )
    }

    /// One frame of `pitch` keeping the painted shapes: the paint
    /// assertions read the markers' fills, strokes and text anchors.
    fn pitch_paint(
        &mut self,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        self.frame(vec![], |ui| {
            tactics::assign_note(ui, view);
            super::pitch(ui, "pitch", tactics, view, roster);
        })
    }

    /// A click at `pos` over `pitch` (press on one frame, release on
    /// the next): the release frame's nodes and change.
    pub(crate) fn click_pitch(
        &mut self,
        pos: egui::Pos2,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.pitch(tactics, view, roster, pointer_at(pos, true));
        self.pitch(tactics, view, roster, pointer_at(pos, false))
    }

    /// A drag from `from` to `onto` over `pitch` (press, move,
    /// release): whether the drop changed `tactics`.
    fn drag_pitch(
        &mut self,
        from: egui::Pos2,
        onto: egui::Pos2,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> bool {
        self.pitch(tactics, view, roster, pointer_at(from, true));
        self.pitch(tactics, view, roster, vec![egui::Event::PointerMoved(onto)]);
        let (_, changed) = self.pitch(tactics, view, roster, pointer_at(onto, false));
        changed
    }

    /// One frame of `pitch` over the lineup lists, as the host's two
    /// columns draw them side by side: either's change counts.
    pub(crate) fn pitch_and_lineup(
        &mut self,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = false;
        let (nodes, _) = self.frame(events, |ui| {
            tactics::assign_note(ui, view);
            changed = super::pitch(ui, "pitch", tactics, view, roster);
            changed |= super::lineup(ui, "lineup", tactics, view, roster);
        });
        (nodes, changed)
    }

    /// A drag from `from` to `onto` over the pitch and the lists
    /// (press, move, release): whether the drop changed `tactics`.
    pub(crate) fn drag_pitch_and_lineup(
        &mut self,
        from: egui::Pos2,
        onto: egui::Pos2,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> bool {
        self.pitch_and_lineup(tactics, view, roster, pointer_at(from, true));
        self.pitch_and_lineup(tactics, view, roster, vec![egui::Event::PointerMoved(onto)]);
        let (_, changed) = self.pitch_and_lineup(tactics, view, roster, pointer_at(onto, false));
        changed
    }

    /// One frame of `tactics_settings` (the role chips' home for the
    /// assign-mode tests): its nodes and whether it changed `tactics`.
    fn settings(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame(events, |ui| {
            changed = Some(tactics::tactics_settings(
                ui, "tactics", version, tactics, view, roster,
            ));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// A click at `pos` over `tactics_settings` (press on one frame,
    /// release on the next): the release frame's nodes and change.
    fn click_settings(
        &mut self,
        pos: egui::Pos2,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.settings(version, tactics, view, roster, pointer_at(pos, true));
        self.settings(version, tactics, view, roster, pointer_at(pos, false))
    }

    /// The tooltip drawn while the pointer rests at `pos` containing
    /// `needle` (bounded; needs `tooltip_delay` at 0). The pointer
    /// first moves away for a frame: egui refuses a tooltip while the
    /// last click is more recent than the last movement.
    pub(crate) fn pitch_tooltip(
        &mut self,
        pos: egui::Pos2,
        needle: &str,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> Option<String> {
        self.pitch(
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(egui::Pos2::ZERO)],
        );
        self.pitch(tactics, view, roster, vec![egui::Event::PointerMoved(pos)]);
        for _ in 0..90 {
            let (nodes, _) = self.pitch(tactics, view, roster, vec![]);
            if let Some(value) = nodes.iter().find_map(|node| {
                node.value()
                    .filter(|value| value.contains(needle))
                    .map(str::to_owned)
            }) {
                return Some(value);
            }
        }
        None
    }
}

/// The pitch canvas' rect (the "pitch" node's).
pub(crate) fn canvas_rect(nodes: &[accesskit::Node]) -> egui::Rect {
    let node = nodes
        .iter()
        .find(|node| node.role() == Role::Unknown && node.label() == Some("pitch"))
        .expect("the pitch canvas");
    let bounds = node.bounds().expect("canvas bounds");
    egui::Rect::from_min_max(
        egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
        egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
    )
}

/// The centre of `slot`'s marker as drawn on `nodes`' canvas.
pub(crate) fn marker_centre(nodes: &[accesskit::Node], slot: FormationSlot) -> egui::Pos2 {
    let canvas = canvas_rect(nodes);
    marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas).center()
}

/// A point on `canvas` no marker of `players` covers: a grass drop
/// (the first free of the grid scan).
fn grass_point(canvas: egui::Rect, players: &[FormationSlot; 11]) -> egui::Pos2 {
    let free = |point: egui::Pos2| {
        players.iter().all(|slot| {
            !marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas).contains(point)
        })
    };
    (0..=GRID_X)
        .flat_map(|x| (0..=GRID_Y).map(move |y| (x, y)))
        .map(|(x, y)| pitch_point(canvas.min, x, y))
        .find(|point| free(*point))
        .expect("some grid point is free of markers")
}

/// The frame's Label nodes whose text is `text`, in (y, x) order.
fn label_nodes<'a>(nodes: &'a [accesskit::Node], text: &str) -> Vec<&'a accesskit::Node> {
    let mut found: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::Label && node.value() == Some(text))
        .collect();
    found.sort_by(|a, b| {
        let key = |node: &&accesskit::Node| {
            let bounds = node.bounds().expect("bounds");
            (bounds.y0, bounds.x0)
        };
        key(a).partial_cmp(&key(b)).expect("ordered bounds")
    });
    found
}

/// The lineup rows' label triples (≡ handle, position, name) in draw
/// order: the 11 starting-eleven rows, then the bench rows. Rows are
/// found by index, not by name — the fixture's PES 21 players all
/// read "PLACEHOLDER".
pub(crate) fn lineup_rows(nodes: &[accesskit::Node]) -> Vec<Vec<&accesskit::Node>> {
    label_nodes(nodes, "≡")
        .iter()
        .map(|handle| {
            let bounds = handle.bounds().expect("handle bounds");
            let mut row: Vec<&accesskit::Node> = nodes
                .iter()
                .filter(|node| {
                    node.role() == Role::Label
                        && node
                            .bounds()
                            .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0)
                })
                .collect();
            row.sort_by(|a, b| {
                let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
                x(a).partial_cmp(&x(b)).expect("ordered bounds")
            });
            row
        })
        .collect()
}

/// The centre of lineup `row`'s name label (the `index`th of
/// `lineup_rows`: 0 to 10 the eleven, 11 on the bench), where a drag
/// lands.
pub(crate) fn row_centre(nodes: &[accesskit::Node], index: usize) -> egui::Pos2 {
    let rows = lineup_rows(nodes);
    centre(
        rows.get(index)
            .and_then(|row| row.last())
            .copied()
            .unwrap_or_else(|| panic!("no lineup row {index}")),
    )
}

/// The position column's text of lineup `row`.
pub(crate) fn position_of_row(nodes: &[accesskit::Node], index: usize) -> String {
    let rows = lineup_rows(nodes);
    rows.get(index)
        .and_then(|row| row.get(1))
        .and_then(|node| node.value())
        .unwrap_or_else(|| panic!("no lineup row {index}"))
        .to_owned()
}

// The draft's `snapXY`: the nearest stored unit, clamped to the grid.
#[test]
fn snap_rounds_to_the_nearest_unit_and_clamps_to_the_grid() {
    let origin = egui::pos2(40.0, 80.0);
    assert_eq!(snap(origin, origin), (0, 48), "the canvas' own corner");
    assert_eq!(snap(origin + PITCH, origin), (104, 0), "the far corner");
    assert_eq!(
        snap(
            origin + egui::vec2(MARGIN.x + 1.4 * UNIT.x, MARGIN.y),
            origin
        ),
        (1, 48),
        "1.4 units right rounds to 1"
    );
    assert_eq!(
        snap(origin + PITCH + egui::vec2(50.0, 50.0), origin),
        (104, 0),
        "a point past the canvas clamps"
    );
    assert_eq!(
        snap(origin - egui::vec2(50.0, 50.0), origin),
        (0, 48),
        "a point before it clamps the same way"
    );
    assert_eq!(
        snap(origin + MARGIN + egui::vec2(0.0, 10.0 * UNIT.y), origin),
        (0, 38),
        "ten units down the grid is y 38"
    );
}

// The draft's `pitchX`/`pitchY` and chip clamp: a stored (x, y) lands
// `UNIT` a unit inside the margin, high y on top; the chip centres on
// the point and clamps 2 px inside the canvas.
#[test]
fn markers_sit_on_the_grid_and_clamp_inside_the_canvas() {
    let origin = egui::pos2(10.0, 20.0);
    let canvas = egui::Rect::from_min_size(origin, PITCH);
    assert_eq!(
        pitch_point(origin, 0, 48),
        egui::pos2(14.0, 22.0),
        "the grid's own corner"
    );
    assert_eq!(
        pitch_point(origin, 52, 24),
        egui::pos2(170.0, 190.0),
        "the grid's centre"
    );
    assert_eq!(
        pitch_point(origin, 104, 0),
        egui::pos2(326.0, 358.0),
        "the far corner"
    );
    assert_eq!(
        marker_rect(egui::pos2(170.0, 190.0), canvas),
        egui::Rect::from_min_size(egui::pos2(149.0, 177.0), CHIP),
        "the chip centres on the point"
    );
    assert_eq!(
        marker_rect(egui::pos2(14.0, 22.0), canvas).min,
        egui::pos2(12.0, 22.0),
        "the top-left clamp"
    );
    assert_eq!(
        marker_rect(egui::pos2(326.0, 358.0), canvas).min,
        egui::pos2(286.0, 330.0),
        "the bottom-right clamp"
    );
}

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
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let from = marker_centre(&nodes, formation.players[index]);
    let onto = grass_point(canvas, &formation.players);
    let (x, y) = snap(onto, canvas.min);
    let slot = formation.players[index];
    assert_ne!((slot.x, slot.y), (x, y), "premise: a real move");

    let changed = window.drag_pitch(from, onto, &mut tactics, &mut view, &roster);
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
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let at = marker_centre(&nodes, formation.players[from]);
    let over = marker_centre(&nodes, formation.players[onto]);
    assert!(
        formation.players[onto + 1..].iter().all(|slot| {
            !marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas).contains(over)
        }),
        "premise: no later marker covers the drop point"
    );

    let changed = window.drag_pitch(at, over, &mut tactics, &mut view, &roster);
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
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let from = marker_centre(&nodes, formation.players[index]);
    // Right of the canvas but inside the window.
    let onto = egui::pos2(canvas.max.x + 20.0, canvas.center().y);
    assert!(!canvas.contains(onto), "premise: a release off the pitch");
    assert!(onto.x < 600.0, "premise: still inside the window");

    let changed = window.drag_pitch(from, onto, &mut tactics, &mut view, &roster);
    assert!(!changed, "a release off the pitch writes nothing");
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
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
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

    let changed = window.drag_pitch(from, onto, &mut tactics, &mut view, &roster);
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
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let from = marker_centre(&nodes, formation.players[keeper]);
    let onto = grass_point(canvas, &formation.players);

    let changed = window.drag_pitch(from, onto, &mut tactics, &mut view, &roster);
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
    let (nodes, changed) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    assert!(!changed, "an idle frame reports no change");
    assert_eq!(tactics, before, "and edits nothing");

    let (_, changed) = window.pitch(
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
    let (nodes, _) = window.pitch_and_lineup(&mut tactics, &mut view, &roster, vec![]);
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
    let (nodes, _) = window.pitch_and_lineup(&mut tactics, &mut view, &roster, vec![]);
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
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the drop changed the record");
    assert_eq!(tactics.starting_eleven[2], inn);
    assert_eq!(tactics.starting_eleven[5], out);
}

// The draft's role palette: arm a chip and the note asks for its pitch
// player; the click writes his roster slot to the taker and disarms.
#[test]
fn a_role_chip_then_a_marker_click_writes_the_taker_and_disarms() {
    // The window is wide enough to hold the whole chips row (the
    // settings pane's content is wider than the pitch column).
    let mut window = Headless::sized(1400.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let index = 7;
    let starter = tactics.starting_eleven[index];
    assert_ne!(
        tactics.set_pieces.penalty, starter,
        "premise: the marker's player is not already the taker"
    );

    // Arm the Penalty kick chip.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Penalty kick:"))
        })
        .expect("the penalty chip");
    let (_, changed) =
        window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert!(!changed, "arming writes no record");
    assert_eq!(view.assign, Some(Assign::Role(5)), "the mode is armed");

    // The pitch draws the note; the marker click writes the taker.
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Click the pitch player for penalty kick.")),
        "the armed note is drawn"
    );
    entry(&nodes, Role::Button, "Cancel");
    let canvas = canvas_rect(&nodes);
    let formation = tactics.presets[0].formations[0].clone();
    let at = marker_rect(
        pitch_point(
            canvas.min,
            formation.players[index].x,
            formation.players[index].y,
        ),
        canvas,
    )
    .center();
    let (_, changed) = window.click_pitch(at, &mut tactics, &mut view, &roster);
    assert!(changed, "the assignment writes the record");
    assert_eq!(
        tactics.set_pieces.penalty, starter,
        "the marker's player takes it"
    );
    assert_eq!(view.assign, None, "and the mode disarms");
}

// The join pick toggles members until Done: three clicks fill the
// three entries, a fourth is ignored, and clicking a member removes
// him, packing the list (the plan's "Players to join the attack stay
// packed").
#[test]
fn join_clicks_fill_three_then_toggle_members_packed() {
    let mut window = Headless::sized(900.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    tactics.players_to_join_attack = [u8::MAX; 3];
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);

    // Arm the join chip.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Join attack:"))
        })
        .expect("the join chip");
    window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert_eq!(view.assign, Some(Assign::Join), "the mode is armed");

    // Three marker clicks fill the list, packed.
    let starters = tactics.starting_eleven;
    let click_marker =
        |window: &mut Headless, tactics: &mut TeamTactics, view: &mut TacticsView, index: usize| {
            let (nodes, _) = window.pitch(tactics, view, &roster, vec![]);
            let canvas = canvas_rect(&nodes);
            let slot = tactics.presets[0].formations[0].players[index];
            let at = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas).center();
            window.click_pitch(at, tactics, view, &roster)
        };
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 4);
    assert!(changed, "the first click wrote");
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 5);
    assert!(changed, "the second click wrote");
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 6);
    assert!(changed, "the third click wrote");
    assert_eq!(
        tactics.players_to_join_attack,
        [starters[4], starters[5], starters[6]],
        "three clicks fill three"
    );
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 7);
    assert!(!changed, "a fourth is ignored");
    assert_eq!(
        tactics.players_to_join_attack,
        [starters[4], starters[5], starters[6]],
        "and the list is as it was"
    );
    // Clicking the first member removes him; the list packs.
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 4);
    assert!(changed, "the removal wrote");
    assert_eq!(
        tactics.players_to_join_attack,
        [starters[5], starters[6], u8::MAX],
        "the first member left and the list packed"
    );
    assert_eq!(view.assign, Some(Assign::Join), "join stays armed");

    // Done disarms.
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let done = entry(&nodes, Role::Button, "Done");
    window.click_pitch(centre(done), &mut tactics, &mut view, &roster);
    assert_eq!(view.assign, None, "Done disarms the mode");
}

// `paint_pitch`'s lines, read off the painted shapes in canvas-
// relative coordinates: the grid, the two boxes and six-yard boxes
// stroked in LINES, the halfway line and the centre circle.
#[test]
fn the_painted_lines_are_the_pitches_markings() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let at = |x: f32, y: f32| canvas.min + egui::vec2(x, y);
    let mut rects: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if shape.stroke.color == LINES => Some(shape.rect),
            _ => None,
        })
        .collect();
    rects.sort_by(|a, b| {
        (a.min.y, a.min.x)
            .partial_cmp(&(b.min.y, b.min.x))
            .expect("ordered bounds")
    });
    assert_eq!(
        rects,
        [
            egui::Rect::from_min_size(at(4.0, 2.0), egui::vec2(312.0, 336.0)),
            egui::Rect::from_min_size(at(80.0, 2.0), egui::vec2(160.0, 52.0)),
            egui::Rect::from_min_size(at(122.0, 2.0), egui::vec2(76.0, 18.0)),
            egui::Rect::from_min_size(at(80.0, 286.0), egui::vec2(160.0, 52.0)),
            egui::Rect::from_min_size(at(122.0, 320.0), egui::vec2(76.0, 18.0)),
        ],
        "the grid, the boxes and the six-yard boxes"
    );
    let halfway: Vec<[egui::Pos2; 2]> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { points, stroke } if stroke.color == LINES => Some(*points),
            _ => None,
        })
        .collect();
    assert_eq!(
        halfway,
        [[at(4.0, 170.0), at(316.0, 170.0)]],
        "the halfway line"
    );
    let circles: Vec<(egui::Pos2, f32)> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Circle(shape) if shape.stroke.color == LINES => {
                Some((shape.center, shape.radius))
            }
            _ => None,
        })
        .collect();
    assert_eq!(circles, [(at(160.0, 170.0), 30.0)], "the centre circle");
}

// `marker`'s armed fill: under join mode a member's chip paints
// JOIN_FILL, a non-member's CHIP_FILL, the keeper's KEEPER_FILL; with
// nothing armed the member's is CHIP_FILL again.
#[test]
fn the_join_modes_members_paint_in_join_fill() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    tactics.players_to_join_attack = [tactics.starting_eleven[3], u8::MAX, u8::MAX];
    view.assign = Some(Assign::Join);
    let players = tactics.presets[0].formations[0].players;
    let keeper = players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");
    assert!(keeper != 3 && keeper != 4, "premise: 3 and 4 are outfield");

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let fill_of = |shapes: &[egui::epaint::ClippedShape], canvas: egui::Rect, index: usize| {
        let slot = players[index];
        let rect = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas);
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect == rect => Some(shape.fill),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted rect for marker {index}"))
    };
    assert_eq!(fill_of(&shapes, canvas, 3), JOIN_FILL, "the member's chip");
    assert_eq!(
        fill_of(&shapes, canvas, 4),
        CHIP_FILL,
        "a non-member's chip"
    );
    assert_eq!(
        fill_of(&shapes, canvas, keeper),
        KEEPER_FILL,
        "the keeper's chip"
    );

    view.assign = None;
    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    assert_eq!(
        fill_of(&shapes, canvas, 3),
        CHIP_FILL,
        "unarmed, the member's chip is the ordinary fill"
    );
}

// `marker`'s open stroke: the marker whose popover is open strokes
// OPEN_STROKE, an unopened one CHIP_STROKE.
#[test]
fn the_open_popovers_marker_strokes_orange() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;
    let index = 3;

    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let at = marker_centre(&nodes, players[index]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(index),
        "the popover is slot 3's"
    );

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let stroke_of = |index: usize| {
        let slot = players[index];
        let rect = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas);
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect == rect => Some(shape.stroke.color),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted rect for marker {index}"))
    };
    assert_eq!(stroke_of(3), OPEN_STROKE, "the open slot's marker");
    assert_eq!(stroke_of(4), CHIP_STROKE, "an unopened one");
}

// `marker`'s texts: the position label centred on `rect.top() + 7`,
// the name cut to 8 characters on `rect.top() + 19`.
#[test]
fn the_markers_texts_anchor_inside_the_chip() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;
    let index = 4;
    let slot = players[index];
    let name: String = name_at(&roster, tactics.starting_eleven[index])
        .chars()
        .take(8)
        .collect();

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let rect = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas);
    let text_at = |pos: egui::Pos2| {
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(shape)
                    if shape.visual_bounding_rect().center().distance(pos) <= 1.0 =>
                {
                    Some(shape.galley.job.text.as_str())
                }
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted text at {pos:?}"))
    };
    assert_eq!(
        text_at(egui::pos2(rect.center().x, rect.top() + 7.0)),
        position_label(slot.position),
        "the position label"
    );
    assert_eq!(
        text_at(egui::pos2(rect.center().x, rect.top() + 19.0)),
        name,
        "the name"
    );
}

// The popover's slot is of the preset it opened on: a preset switch
// closes it (a formation switch is
// `the_popover_closes_when_the_formation_changes`'s case).
#[test]
fn a_preset_switch_closes_the_popover() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    let index = 3;

    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let at = marker_centre(&nodes, formation.players[index]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    entry(&nodes, Role::Button, "Close");

    view.preset = 1;
    window.pitch(&mut tactics, &mut view, &roster, vec![]);
    assert_eq!(view.popover, None, "a preset switch closes the popover");
}

// The popover's Position combo writes the picked position byte to
// the shown formation's slot alone.
#[test]
fn the_popovers_position_pick_writes_the_slot() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[0].formations[0].clone();
    let index = 3;
    let cf = POSITIONS
        .iter()
        .position(|label| *label == "CF")
        .expect("CF is a position") as u8;
    assert_ne!(
        formation.players[index].position, cf,
        "premise: a real change"
    );

    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let at = marker_centre(&nodes, formation.players[index]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let combo = controls_right_of(&nodes, "Position", Role::ComboBox)
        .into_iter()
        .next()
        .expect("the position combo");
    window.click_pitch(centre(combo), &mut tactics, &mut view, &roster);
    // The item pick goes through AccessKit, as the instruction
    // combos' do: a pointer click does not reach a combo's popup.
    let ids = window.pitch_ids(&mut tactics, &mut view, &roster);
    let item = entry_id(&ids, Role::Button, "CF");
    let (_, changed) = window.action_click_pitch(item, &mut tactics, &mut view, &roster);
    assert!(changed, "the pick changed the record");
    assert_eq!(
        tactics.presets[0].formations[0].players[index].position, cf,
        "the pick wrote CF"
    );
    assert_eq!(
        tactics.presets[0].formations[1..],
        before.presets[0].formations[1..],
        "the other two formations untouched"
    );
}
