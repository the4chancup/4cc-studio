use egui::accesskit::{self, Role};
use pes_version::PesVersion;

use super::*;
use crate::tactics;
use crate::test_support::{Headless, centre, pointer_at};

/// The drag-and-drop and swap tests.
mod drops;
/// The geometry and painted-output tests.
mod paint;
/// The slot popover and armed assign modes' tests.
mod popover;

impl Headless {
    /// One frame of `lineup` over `tactics`, `view` and `roster`: the
    /// frame's AccessKit nodes and whether it changed `tactics`.
    pub(crate) fn lineup(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame(events, |ui| {
            changed = Some(super::lineup(ui, "lineup", version, tactics, view, roster));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// A drag from `from` to `onto` over the widget (press, move,
    /// release): whether the drop changed `tactics`.
    pub(crate) fn drag_lineup(
        &mut self,
        from: egui::Pos2,
        onto: egui::Pos2,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> bool {
        self.lineup(version, tactics, view, roster, pointer_at(from, true));
        self.lineup(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(onto)],
        );
        let (_, changed) = self.lineup(version, tactics, view, roster, pointer_at(onto, false));
        changed
    }

    /// One frame of `pitch` over `tactics`, `view` and `roster`: the
    /// frame's AccessKit nodes and whether it changed `tactics`. The
    /// armed mode's note heads the frame, as the host draws it.
    pub(crate) fn pitch(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame(events, |ui| {
            tactics::assign_note(ui, view);
            changed = Some(super::pitch(ui, "pitch", version, tactics, view, roster));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// `pitch` keeping the nodes' AccessKit ids (for
    /// `AccessKitActionRequest` events on a combo's popup items).
    fn pitch_ids(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> Vec<(accesskit::NodeId, accesskit::Node)> {
        let (nodes, _) = self.frame_at_ids(None, vec![], |ui| {
            tactics::assign_note(ui, view);
            super::pitch(ui, "pitch", version, tactics, view, roster);
        });
        nodes
    }

    /// An AccessKit `Click` action on `id`: a click delivered as the
    /// accessibility event, for widgets a pointer click does not
    /// reach (a combo's popup items).
    fn action_click_pitch(
        &mut self,
        id: accesskit::NodeId,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.pitch(
            version,
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

    /// One frame of `lineup` keeping the painted shapes: the name
    /// assertions read the rows' text runs.
    fn lineup_paint(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        self.frame(vec![], |ui| {
            super::lineup(ui, "lineup", version, tactics, view, roster);
        })
    }

    /// One frame of `pitch` keeping the painted shapes: the paint
    /// assertions read the markers' fills, strokes and text anchors.
    fn pitch_paint(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        self.frame(vec![], |ui| {
            tactics::assign_note(ui, view);
            super::pitch(ui, "pitch", version, tactics, view, roster);
        })
    }

    /// A click at `pos` over `pitch` (press on one frame, release on
    /// the next): the release frame's nodes and change.
    pub(crate) fn click_pitch(
        &mut self,
        pos: egui::Pos2,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.pitch(version, tactics, view, roster, pointer_at(pos, true));
        self.pitch(version, tactics, view, roster, pointer_at(pos, false))
    }

    /// A drag from `from` to `onto` over `pitch` (press, move,
    /// release): whether the drop changed `tactics`.
    fn drag_pitch(
        &mut self,
        from: egui::Pos2,
        onto: egui::Pos2,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> bool {
        self.pitch(version, tactics, view, roster, pointer_at(from, true));
        self.pitch(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(onto)],
        );
        let (_, changed) = self.pitch(version, tactics, view, roster, pointer_at(onto, false));
        changed
    }

    /// One frame of `pitch` over the lineup lists, as the host's two
    /// columns draw them side by side: either's change counts.
    pub(crate) fn pitch_and_lineup(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = false;
        let (nodes, _) = self.frame(events, |ui| {
            tactics::assign_note(ui, view);
            changed = super::pitch(ui, "pitch", version, tactics, view, roster);
            changed |= super::lineup(ui, "lineup", version, tactics, view, roster);
        });
        (nodes, changed)
    }

    /// A drag from `from` to `onto` over the pitch and the lists
    /// (press, move, release): whether the drop changed `tactics`.
    pub(crate) fn drag_pitch_and_lineup(
        &mut self,
        from: egui::Pos2,
        onto: egui::Pos2,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> bool {
        self.pitch_and_lineup(version, tactics, view, roster, pointer_at(from, true));
        self.pitch_and_lineup(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(onto)],
        );
        let (_, changed) =
            self.pitch_and_lineup(version, tactics, view, roster, pointer_at(onto, false));
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
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> Option<String> {
        self.pitch(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(egui::Pos2::ZERO)],
        );
        self.pitch(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(pos)],
        );
        for _ in 0..90 {
            let (nodes, _) = self.pitch(version, tactics, view, roster, vec![]);
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
