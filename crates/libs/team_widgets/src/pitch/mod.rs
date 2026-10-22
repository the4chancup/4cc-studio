//! The Tactics tab's pitch side (`plans/save_editor.md` "Tactics editor",
//! the lineup and pitch bullets): the draft's `pitchCanvas` and
//! `slotPopover` — the formation canvas, a marker per starter dragged to
//! his spot, dropped on to swap a bench player in, clicked for the slot's
//! details — plus `lineupList` and `benchList`, the starting eleven and
//! the bench, reordered and swapped by drag.

use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::tactics::{FormationSlot, TeamTactics};

use crate::tactics::{self, Assign, OpenSlot, RosterRow, TacticsView, name_at};

/// The "Starting eleven" and "Bench" lists, their rows and `swap_in`.
mod lineup;
/// The slot popover a marker click opens beside its marker.
mod popover;

pub use lineup::lineup;
use lineup::swap_in;
use popover::slot_popover;

/// The formations' names, in the record's order (the draft's
/// `FORMATIONS`).
pub const FORMATIONS: [&str; 3] = ["Kick-off", "In-possession", "Out-of-possession"];

/// The payload a lineup row's drag carries and the pitch's markers drop
/// (the canvas lands in this module with it): which list the row is in
/// and its index in it, into `TeamTactics::starting_eleven` or
/// `TeamTactics::bench_order`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineupDrag {
    /// A starting-eleven row.
    Starter(usize),
    /// A bench row.
    Bench(usize),
}

/// The canvas' size (the draft's `PITCH`): the record's grid — x 0 to
/// `GRID_X`, y 0 to `GRID_Y`, the record's whole units — drawn `UNIT` a
/// unit inside a `MARGIN` margin, high y (the attack end) at the top.
const PITCH: egui::Vec2 = egui::vec2(320.0, 340.0);
/// The grid's extent in stored units.
const GRID_X: u8 = 104;
const GRID_Y: u8 = 48;
/// The margin the grid leaves inside the canvas.
const MARGIN: egui::Vec2 = egui::vec2(4.0, 2.0);
/// A stored unit's pitch, x and y.
const UNIT: egui::Vec2 = egui::vec2(3.0, 7.0);
/// A marker chip's size (the draft's 42 by 26).
const CHIP: egui::Vec2 = egui::vec2(42.0, 26.0);

/// The turf's fill (the draft's `#173517`).
const TURF: egui::Color32 = egui::Color32::from_rgb(0x17, 0x35, 0x17);
/// The lines' stroke (`#4a7a4a`).
const LINES: egui::Color32 = egui::Color32::from_rgb(0x4a, 0x7a, 0x4a);
/// A marker's fill (`#3c3c3c`), the keeper's (`#5a4a1e`) and a join
/// member's while join mode is armed (`#005c80`).
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgb(0x3c, 0x3c, 0x3c);
const KEEPER_FILL: egui::Color32 = egui::Color32::from_rgb(0x5a, 0x4a, 0x1e);
const JOIN_FILL: egui::Color32 = egui::Color32::from_rgb(0x00, 0x5c, 0x80);
/// A marker's stroke (`#6e6e6e`), the armed mode's (`#c0deff`) and the
/// open popover's (`#ff8f00`, which wins over the armed one).
const CHIP_STROKE: egui::Color32 = egui::Color32::from_rgb(0x6e, 0x6e, 0x6e);
const ARMED_STROKE: egui::Color32 = egui::Color32::from_rgb(0xc0, 0xde, 0xff);
const OPEN_STROKE: egui::Color32 = egui::Color32::from_rgb(0xff, 0x8f, 0x00);
/// The marker's texts: the position (`#dcdcdc`) over the name (`#8c8c8c`).
const CHIP_POSITION: egui::Color32 = egui::Color32::from_rgb(0xdc, 0xdc, 0xdc);
const CHIP_NAME: egui::Color32 = egui::Color32::from_rgb(0x8c, 0x8c, 0x8c);

/// The draft's pitch column: the armed mode's note while one is armed,
/// the formation selector (the draft heads the pitch with it — it writes
/// `view.formation`, not the record), then the canvas — a marker per
/// starter of the shown formation, dragged to his spot, dropped on to
/// swap, clicked for the slot popover — and the open popover beside its
/// marker.
///
/// A marker carries `LineupDrag::Starter`, so a release over a lineup row
/// is that row's drop (a bench row swaps him out through `swap_in`, a
/// starter row exchanges the two); a marker is itself a drop target — a
/// `Bench` row on it is `swap_in`, a `Starter` exchanges the two starters
/// — and a release on the grass writes the slot's snapped coordinates.
/// The keeper's marker (position 0) is not a drag source; its spot is
/// fixed. Answers `true` only when `tactics` changed this frame.
pub fn pitch(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) -> bool {
    ui.push_id(id_salt, |ui| {
        // The answer is the deep compare, as `tactics_settings`' is.
        let before = tactics.clone();
        tactics::assign_note(ui, view);
        ui.horizontal(|ui| {
            for (index, label) in FORMATIONS.iter().enumerate() {
                ui.selectable_value(&mut view.formation, index, *label);
            }
        })
        .response
        .on_hover_text(
            "The game uses formations 2 and 3 only while the preset's fluid formation is on",
        );
        // The popover's slot is of the scope, preset and formation it
        // opened on; a change of any of them closes it.
        if view.popover.is_some_and(|open| {
            open.scope != ui.id() || open.preset != view.preset || open.formation != view.formation
        }) {
            view.popover = None;
        }
        let (canvas, canvas_response) = ui.allocate_exact_size(PITCH, egui::Sense::hover());
        canvas_response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Other, ui.is_enabled(), "pitch")
        });
        let painter = ui.painter_at(canvas);
        paint_pitch(&painter, canvas);
        let players = tactics.presets[view.preset.min(2)].formations[view.formation.min(2)].players;
        let mut responses = Vec::with_capacity(players.len());
        for (index, slot) in players.iter().enumerate() {
            responses.push(marker(ui, canvas, tactics, view, roster, index, *slot));
        }
        // A drop hits the topmost marker first (the draft's `hitMarker`
        // checks back to front); the canvas' own is the grass.
        let mut dropped = None;
        for (index, response) in responses.iter().enumerate().rev() {
            if let Some(payload) = response.dnd_release_payload::<LineupDrag>() {
                dropped = Some((Some(index), *payload));
                break;
            }
        }
        if dropped.is_none() {
            dropped = canvas_response
                .dnd_release_payload::<LineupDrag>()
                .map(|payload| (None, *payload));
        }
        match dropped {
            Some((Some(onto), LineupDrag::Bench(from))) => swap_in(tactics, onto, from),
            Some((Some(onto), LineupDrag::Starter(from))) if onto != from => {
                tactics.starting_eleven.swap(from, onto);
            }
            // A starter released on the grass (or on his own chip) moves
            // to the snapped drop point.
            Some((_, LineupDrag::Starter(from))) => {
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    let (x, y) = snap(pos, canvas.min);
                    let slot = &mut tactics.presets[view.preset.min(2)].formations
                        [view.formation.min(2)]
                    .players[from];
                    slot.x = x;
                    slot.y = y;
                }
            }
            // A bench row on the grass does nothing.
            Some((None, LineupDrag::Bench(_))) | None => {}
        }
        slot_popover(ui, tactics, view, roster, canvas);
        *tactics != before
    })
    .inner
}

/// The pitch point of grid (`x`, `y`), the draft's `pitchX`/`pitchY`;
/// `origin` is the canvas' top-left.
fn pitch_point(origin: egui::Pos2, x: u8, y: u8) -> egui::Pos2 {
    origin
        + MARGIN
        + egui::vec2(
            f32::from(x) * UNIT.x,
            (f32::from(GRID_Y) - f32::from(y)) * UNIT.y,
        )
}

/// `pos` inside the canvas (from `origin`) snapped to the stored grid,
/// the draft's `snapXY`: the nearest unit, clamped to the grid.
fn snap(pos: egui::Pos2, origin: egui::Pos2) -> (u8, u8) {
    let local = pos - origin - MARGIN;
    let x = (local.x / UNIT.x).round().clamp(0.0, f32::from(GRID_X)) as u8;
    let y = (f32::from(GRID_Y) - (local.y / UNIT.y).round()).clamp(0.0, f32::from(GRID_Y)) as u8;
    (x, y)
}

/// The chip's rect centred on `center`, kept 2 px inside the canvas (the
/// draft's clamp: it stops short of the far edge by its own size).
fn marker_rect(center: egui::Pos2, canvas: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(
            (center.x - CHIP.x / 2.0).clamp(canvas.left() + 2.0, canvas.right() - CHIP.x - 2.0),
            (center.y - CHIP.y / 2.0).clamp(canvas.top() + 2.0, canvas.bottom() - CHIP.y - 4.0),
        ),
        CHIP,
    )
}

/// The turf and the lines (the draft's `drawPitch` background): the
/// touchlines, the halfway line, the centre circle and the two boxes.
fn paint_pitch(painter: &egui::Painter, canvas: egui::Rect) {
    let stroke = egui::Stroke::new(1.0, LINES);
    painter.rect_filled(canvas, 0.0, TURF);
    let grid = egui::Rect::from_min_size(
        canvas.min + MARGIN,
        egui::vec2(f32::from(GRID_X) * UNIT.x, f32::from(GRID_Y) * UNIT.y),
    );
    painter.rect_stroke(grid, 0.0, stroke, egui::StrokeKind::Inside);
    painter.line_segment(
        [
            egui::pos2(grid.left(), grid.center().y),
            egui::pos2(grid.right(), grid.center().y),
        ],
        stroke,
    );
    painter.circle_stroke(grid.center(), 30.0, stroke);
    for top in [true, false] {
        let box_top = if top {
            grid.top()
        } else {
            grid.bottom() - 52.0
        };
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(grid.left() + 76.0, box_top),
                egui::vec2(160.0, 52.0),
            ),
            0.0,
            stroke,
            egui::StrokeKind::Inside,
        );
        let six_top = if top {
            grid.top()
        } else {
            grid.bottom() - 18.0
        };
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(grid.left() + 118.0, six_top),
                egui::vec2(76.0, 18.0),
            ),
            0.0,
            stroke,
            egui::StrokeKind::Inside,
        );
    }
}

/// One marker (the draft's chip): the slot's position over the starter's
/// name cut to 8 characters, its AccessKit node a button labeled with
/// both; painted at the pointer while dragged. The keeper's slot
/// (position 0) is not a drag source but still clicks. A click serves the
/// armed mode when one is armed, else opens the slot's popover. The
/// answer is the marker's response, for the caller's drop check.
fn marker(
    ui: &mut egui::Ui,
    canvas: egui::Rect,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
    index: usize,
    slot: FormationSlot,
) -> egui::Response {
    let keeper = slot.position == 0;
    let center = pitch_point(canvas.min, slot.x, slot.y);
    let response = ui.interact(
        marker_rect(center, canvas),
        ui.id().with(("pitch marker", index)),
        if keeper {
            egui::Sense::click()
        } else {
            egui::Sense::click_and_drag()
        },
    );
    let name = name_at(roster, tactics.starting_eleven[index]);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            format!("{} {}", position_label(slot.position), name),
        )
    });
    if !keeper {
        response.dnd_set_drag_payload(LineupDrag::Starter(index));
    }
    if response.clicked() {
        match view.assign {
            Some(assign) => assign.write(tactics, view, index),
            None => {
                view.popover = Some(OpenSlot {
                    scope: ui.id(),
                    preset: view.preset,
                    formation: view.formation,
                    slot: index,
                });
            }
        }
    }
    let at = if response.dragged() {
        ui.ctx().pointer_interact_pos().unwrap_or(center)
    } else {
        center
    };
    let rect = marker_rect(at, canvas);
    let fill = if view.assign == Some(Assign::Join)
        && tactics
            .players_to_join_attack
            .contains(&tactics.starting_eleven[index])
    {
        JOIN_FILL
    } else if keeper {
        KEEPER_FILL
    } else {
        CHIP_FILL
    };
    let stroke = if view.popover.is_some_and(|open| open.slot == index) {
        OPEN_STROKE
    } else if view.assign.is_some() {
        ARMED_STROKE
    } else {
        CHIP_STROKE
    };
    let painter = ui.painter_at(canvas);
    painter.rect(
        rect,
        egui::CornerRadius::same(3),
        fill,
        egui::Stroke::new(1.0, stroke),
        egui::StrokeKind::Inside,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 7.0),
        egui::Align2::CENTER_CENTER,
        position_label(slot.position),
        egui::FontId::proportional(10.0),
        CHIP_POSITION,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 19.0),
        egui::Align2::CENTER_CENTER,
        name.chars().take(8).collect::<String>(),
        egui::FontId::proportional(9.0),
        CHIP_NAME,
    );
    response
}

/// The position byte's label (`POSITIONS`), "?" past the table as the
/// player list reads it.
fn position_label(position: u8) -> &'static str {
    POSITIONS.get(usize::from(position)).copied().unwrap_or("?")
}

#[cfg(test)]
mod tests {
    use egui::accesskit::{self, Role};
    use pes_version::PesVersion;

    use super::*;
    use crate::test_support::{
        Headless, centre, entry, fixture_rows, fixture_tactics, pointer_at, roster_rows,
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
        /// frame's AccessKit nodes and whether it changed `tactics`.
        pub(crate) fn pitch(
            &mut self,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            roster: &[RosterRow<'_>],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, bool) {
            let mut changed = None;
            let (nodes, _) = self.frame(events, |ui| {
                changed = Some(super::pitch(ui, "pitch", tactics, view, roster));
            });
            (nodes, changed.expect("the widget ran"))
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
            let (_, changed) =
                self.pitch_and_lineup(tactics, view, roster, pointer_at(onto, false));
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
        assert_eq!(tactics.starting_eleven[2], 5);
        assert_eq!(tactics.starting_eleven[5], 2);
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
        let click_marker = |window: &mut Headless,
                            tactics: &mut TeamTactics,
                            view: &mut TacticsView,
                            index: usize| {
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
}
