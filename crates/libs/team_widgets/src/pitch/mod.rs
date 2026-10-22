//! The Tactics tab's pitch side (`plans/save_editor.md` "Tactics editor",
//! the lineup and pitch bullets): the draft's `pitchCanvas` and
//! `slotPopover` — the formation canvas, a marker per starter dragged to
//! his spot, dropped on to swap a bench player in, clicked for the slot's
//! details — plus `lineupList` and `benchList`, the starting eleven and
//! the bench, reordered and swapped by drag.

use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::tactics::{FormationSlot, TeamTactics};

use crate::tactics::{Assign, OpenSlot, RosterRow, TacticsView, name_at};

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
/// (the canvas lands in this module with it): `Starter` and `Marker`
/// index `TeamTactics::starting_eleven`, `Bench` `TeamTactics::bench_order`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineupDrag {
    /// A starting-eleven row.
    Starter(usize),
    /// A bench row.
    Bench(usize),
    /// A pitch marker: every drop target treats it as its `Starter` index
    /// — the grass move, a row's release never fires, is its alone.
    Marker(usize),
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

/// The draft's pitch column, the armed mode's note a host draws above it
/// excepted: the formation selector (the draft heads the pitch with it —
/// it writes `view.formation`, not the record), then the canvas — a
/// marker per starter of the shown formation, dragged to his spot,
/// dropped on to swap, clicked for the slot popover — and the open
/// popover beside its marker.
///
/// A marker carries `LineupDrag::Marker`, so a release over a lineup row
/// is that row's drop (a bench row swaps him out through `swap_in`, an
/// eleven row exchanges the two); a marker is itself a drop target — a
/// `Bench` row on it is `swap_in`, a `Starter` or another `Marker`
/// exchanges the two starters — and a release on the grass writes its
/// slot's snapped coordinates, where a row's release does nothing.
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
            Some((Some(onto), LineupDrag::Starter(from) | LineupDrag::Marker(from)))
                if onto != from =>
            {
                tactics.starting_eleven.swap(from, onto);
            }
            // A marker released on the grass (or on his own chip) moves
            // to the snapped drop point.
            Some((_, LineupDrag::Marker(from))) => {
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    let (x, y) = snap(pos, canvas.min);
                    let slot = &mut tactics.presets[view.preset.min(2)].formations
                        [view.formation.min(2)]
                    .players[from];
                    slot.x = x;
                    slot.y = y;
                }
            }
            // A row released on the grass or on his own marker does
            // nothing.
            Some(_) | None => {}
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
        response.dnd_set_drag_payload(LineupDrag::Marker(index));
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
mod tests;
