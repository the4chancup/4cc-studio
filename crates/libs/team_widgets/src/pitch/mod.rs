//! The Tactics tab's pitch side (`plans/save_editor.md` "Tactics editor",
//! the lineup and pitch bullets): the draft's `pitchCanvas` and
//! `slotPopover` — the formation canvas, a marker per starter dragged to
//! his spot, dropped on to swap a bench player in, clicked for the slot's
//! details — plus `lineupList` and `benchList`, the starting eleven and
//! the bench, reordered and swapped by drag. The canvas is painted as PES
//! 2021's own formation screen (the plan's pitch bullet): the game
//! texture's turf bands, the screen's slot-to-spot mapping and markers of
//! portrait, badge and coloured name.

use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::tactics::{Formation, FormationSlot, TeamTactics};
use pes_version::PesVersion;

use crate::names::append_name;
use crate::tactics::{Assign, FieldedMark, OpenSlot, RosterRow, TacticsView, name_at, raw_name_at};
use crate::violations::{CROSS, MARK_GAP};

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

/// The playable positions grid's rows, the order a member reads a
/// formation in (the draft's `POS_ROWS`): the forwards left wing to right
/// wing, the attacking and wide midfielders, the central ones, the
/// defenders, GK alone below. The same rows, read back to front without
/// GK, are the formation name's lines (`formation_name`), so the grid and
/// the name cannot disagree.
pub const POSITION_ROWS: [&[&str]; 5] = [
    &["LWF", "CF", "SS", "RWF"],
    &["LMF", "AMF", "RMF"],
    &["CMF", "DMF"],
    &["LB", "CB", "RB"],
    &["GK"],
];

/// The payload a lineup row's drag carries and the pitch's markers drop
/// (the canvas lands in this module with it): `Starter` and `Marker`
/// index `TeamTactics::starting_eleven`, `Bench` `TeamTactics::bench_order`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineupDrag {
    /// A starting-eleven row.
    Starter(usize),
    /// A bench row.
    Bench(usize),
    /// A pitch marker: a lineup row treats it as its `Starter` index;
    /// another marker swaps their two slots in the shown formation, and
    /// the grass move (a row's release there never fires) is its alone.
    Marker(usize),
}

/// The canvas' size: the screen's own pitch drawn at its 720p capture's
/// scale (the widget allocates `PITCH_BOX`; the tests read this one).
#[cfg(test)]
pub(crate) const PITCH: egui::Vec2 = egui::vec2(320.0, 384.0);
/// The widget's whole extent: `PITCH` plus the draft `.pitch`'s 1 px
/// border a side (the plan's "386 points with its border").
pub const PITCH_BOX: egui::Vec2 = egui::vec2(322.0, 386.0);
/// The grid's extent in stored units.
const GRID_X: u8 = 104;
const GRID_Y: u8 = 48;

/// The badge centre's x for stored x 0 (fitted on the screen's capture).
const SPOT_X0: f32 = 12.8;
/// The badge centre's x pitch per stored x unit (fitted on the screen's
/// capture).
const SPOT_XS: f32 = 2.827;
/// The badge centre's y for stored y 0 (fitted on the screen's capture).
const SPOT_Y0: f32 = 363.8;
/// The badge centre's y pitch per stored y unit (fitted on the screen's
/// capture).
const SPOT_YS: f32 = 6.793;
/// The keeper's badge centre (the screen's keeper spot, on the goal
/// area's line), whatever his stored x and y.
const KEEPER_X: f32 = 160.0;
const KEEPER_Y: f32 = 358.5;

/// The badge and the portrait box's width.
const BADGE_W: f32 = 50.0;
/// The badge's height.
const BADGE_H: f32 = 14.0;
/// The portrait box's height.
const PORTRAIT_H: f32 = 50.0;
/// The marker box's reach above the badge centre: the portrait's 50 plus
/// the badge's 7 half.
const ABOVE_CENTRE: f32 = 57.0;
/// The marker box's reach below the badge centre: the name's bottom edge.
const BELOW_CENTRE: f32 = 20.0;
/// The name's centre under the badge centre's.
const NAME_DY: f32 = 13.0;

/// The turf texture's size: the game texture's own, drawn into the
/// smaller canvas.
const TURF_SIZE: [usize; 2] = [480, 576];
/// The turf's band count: dark at both ends.
const TURF_BANDS: usize = 11;
/// A band's dark and light base colours (`#2d571d`, `#35602a`).
const TURF_DARK: [u8; 3] = [0x2d, 0x57, 0x1d];
const TURF_LIGHT: [u8; 3] = [0x35, 0x60, 0x2a];

/// The lines' stroke: white at 20% opacity, 2 px wide.
const LINES: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(255, 255, 255, 51);
/// The badge's fill (`#242b2d`) and a join member's while join mode is
/// armed (`#005c80`).
const BADGE_FILL: egui::Color32 = egui::Color32::from_rgb(0x24, 0x2b, 0x2d);
const JOIN_FILL: egui::Color32 = egui::Color32::from_rgb(0x00, 0x5c, 0x80);
/// The position text's group colours: forwards `#ff2c77`, midfielders
/// `#57e42b`, defenders `#00d2ff`, the keeper `#ffde00`.
const FORWARD: egui::Color32 = egui::Color32::from_rgb(0xff, 0x2c, 0x77);
const MIDFIELDER: egui::Color32 = egui::Color32::from_rgb(0x57, 0xe4, 0x2b);
const DEFENDER: egui::Color32 = egui::Color32::from_rgb(0x00, 0xd2, 0xff);
const KEEPER: egui::Color32 = egui::Color32::from_rgb(0xff, 0xde, 0x00);
/// The corner brackets the screen's selection marks lend the editor's
/// states: the open popover's marker (`#14fbff`), every marker an armed
/// assignment mode can take (`#ff4e7f`).
const SELECTED_CORNERS: egui::Color32 = egui::Color32::from_rgb(0x14, 0xfb, 0xff);
const CANDIDATE_CORNERS: egui::Color32 = egui::Color32::from_rgb(0xff, 0x4e, 0x7f);

/// A wide name's scroll period in seconds (the game's measured 13.5).
const NAME_PERIOD: f32 = 13.5;
/// The scroll's speed, canvas px a second (the game's 40).
const NAME_SPEED: f32 = 40.0;
/// The gap between a scrolling name's end and its repeat, in px (the
/// game's 16).
const NAME_GAP: f32 = 16.0;

/// The formation name's size: its digits 15 points tall, the game's
/// label's (the plan's "The formation's name").
const LABEL_SIZE: f32 = 21.0;
/// The name's text origin, in from the canvas' left edge: the face's 2
/// point left bearing puts its ink 7 in, the game's.
const LABEL_LEFT: f32 = 5.0;
/// The name's baseline, up from the canvas' bottom edge (its digits sit
/// on it).
const LABEL_BOTTOM: f32 = 6.0;

/// A marker as a frame's pass sees it: its index in the shown formation,
/// the slot and its (clamped) badge centre.
#[derive(Clone, Copy)]
struct Spot {
    /// The slot's index in the formation's eleven.
    index: usize,
    /// The formation slot.
    slot: FormationSlot,
    /// The badge centre the marker draws and interacts at (the dragged
    /// one's is the clamped pointer instead).
    centre: egui::Pos2,
}

/// The formation selector of the formation bar (the draft's
/// `formationSel`, moved out of `pitch` for the bar over the pitch's and
/// the assignments' columns): the shown preset's three formations.
/// Writes `view.formation`, never the record.
pub fn formation_selector(ui: &mut egui::Ui, view: &mut TacticsView) {
    ui.horizontal(|ui| {
        for (index, label) in FORMATIONS.iter().enumerate() {
            ui.selectable_value(&mut view.formation, index, *label);
        }
    })
    .response
    .on_hover_text(
        "The game uses formations 2 and 3 only while the preset's fluid formation is on",
    );
}

/// The draft's `.pitch`: the canvas inside its 1 px border — a marker
/// per starter of the shown formation, dragged to his spot, dropped on
/// to swap, clicked for the slot popover — the formation's name in its
/// bottom-left corner, and the open popover beside its marker.
/// `formation_selector` heads it in the host's bar.
///
/// A marker carries `LineupDrag::Marker`, so a release over a lineup row
/// is that row's drop (a bench row swaps him out through `swap_in`, an
/// eleven row exchanges the two in the starting eleven); a marker is
/// itself a drop target — a `Bench` row on it is `swap_in`, a `Starter`
/// row exchanges the two in the starting eleven (every formation follows,
/// the game's select-one-then-the-other), another `Marker` exchanges the
/// two slots' position and spot in the shown formation alone (the game's
/// drag) — and a release on the grass writes its slot's snapped
/// coordinates, where a row's release does nothing. The keeper's marker
/// (position 0) is neither a drag source nor a marker's swap target; his
/// spot is fixed. `version` is the open save's (`swap_in`'s instruction
/// decode). Answers `true` only when `tactics` changed this frame.
pub fn pitch(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    version: PesVersion,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) -> bool {
    ui.push_id(id_salt, |ui| {
        // The answer is the deep compare, as `tactics_settings`' is.
        let before = tactics.clone();
        // The popover's slot is of the scope, preset and formation it
        // opened on; a change of any of them closes it.
        if view.popover.is_some_and(|open| {
            open.scope != ui.scope_id()
                || open.preset != view.preset
                || open.formation != view.formation
        }) {
            view.popover = None;
        }
        // The name scroll's clock: the time the pitch was first shown,
        // in the context's temp memory with the frame it was last drawn
        // on; a frame without it restarts it, and a frame can run the
        // pitch more than once (egui's discard pass), so the same frame
        // number keeps the clock.
        let clock_id = ui.scope_id().with("name clock");
        let now = ui.input(|i| i.time);
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let started = ui
            .ctx()
            .data_mut(|data| match data.get_temp::<(f64, u64)>(clock_id) {
                Some((started, last)) if frame_nr <= last + 1 => {
                    data.insert_temp(clock_id, (started, frame_nr));
                    started
                }
                _ => {
                    data.insert_temp(clock_id, (now, frame_nr));
                    now
                }
            });
        let seconds = (now - started) as f32;
        // The draft's `.pitch`: the canvas and its 1 px border a side
        // (`PITCH_BOX` is the widget's extent, `canvas` inside it). The
        // canvas' response is an `interact` on the already-allocated
        // rect, not a second allocation.
        let (area, _) = ui.allocate_exact_size(PITCH_BOX, egui::Sense::hover());
        let canvas = area.shrink(1.0);
        let canvas_response = ui.interact(
            canvas,
            ui.scope_id().with("pitch canvas"),
            egui::Sense::hover(),
        );
        canvas_response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Unknown, ui.is_enabled(), "pitch")
        });
        ui.painter().rect_stroke(
            area,
            0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let painter = ui.painter_at(canvas);
        paint_pitch(&painter, canvas);
        let players = tactics.presets[view.preset.min(2)].formations[view.formation.min(2)].players;
        let centres = players.map(|slot| clamp_centre(badge_centre(canvas.min, slot), canvas));
        let order = draw_order(&centres);
        // Interact in draw order first: egui gives an overlap to the
        // widget registered last, so the lowest marker (painted on top)
        // wins a press.
        let mut responses: [Option<egui::Response>; 11] = std::array::from_fn(|_| None);
        for index in order {
            let spot = Spot {
                index,
                slot: players[index],
                centre: centres[index],
            };
            responses[index] = Some(marker(ui, canvas, version, tactics, view, roster, spot));
        }
        // A drop hits the topmost marker first (the draft's `hitMarker`
        // checks back to front); the canvas' own is the grass.
        let mut dropped = None;
        for index in order.iter().rev() {
            if let Some(payload) = responses[*index]
                .as_ref()
                .and_then(|response| response.dnd_release_payload::<LineupDrag>())
            {
                dropped = Some((Some(*index), *payload));
                break;
            }
        }
        if dropped.is_none() {
            dropped = canvas_response
                .dnd_release_payload::<LineupDrag>()
                .map(|payload| (None, *payload));
        }
        match dropped {
            Some((Some(onto), LineupDrag::Bench(from))) => swap_in(version, tactics, onto, from),
            Some((Some(onto), LineupDrag::Starter(from))) if onto != from => {
                tactics.starting_eleven.swap(from, onto);
            }
            // A dragged marker's release reads `swap_target` — the same
            // pick the swap mark shows: over another outfield starter
            // the two slots exchange position and spot in the shown
            // formation alone, each player staying in his slot, as the
            // game's drag does (the starting-eleven exchange, which
            // reaches every formation, is the lineup rows'); over the
            // grass, his own spot or the keeper he moves to the snapped
            // drop point. A release egui also reports as his click (the
            // pointer moved within the click distance, though past his
            // edge) is that click, which `marker` served: no drop.
            Some((_, LineupDrag::Marker(from)))
                if !responses[from]
                    .as_ref()
                    .is_some_and(egui::Response::clicked) =>
            {
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    let formation =
                        &mut tactics.presets[view.preset.min(2)].formations[view.formation.min(2)];
                    match swap_target(&players, &centres, canvas, pos, from) {
                        Some(onto) => formation.players.swap(from, onto),
                        None => {
                            let (x, y) = snap(pos, canvas.min);
                            formation.players[from].x = x;
                            formation.players[from].y = y;
                        }
                    }
                }
            }
            // A row released on the grass or on his own marker, and a
            // marker's release that was his click, do nothing.
            Some(_) | None => {}
        }
        // The paint pass reads the written record: the spots are
        // recomputed after the drop is written, so a released marker
        // paints at his drop point on this very frame.
        let players = tactics.presets[view.preset.min(2)].formations[view.formation.min(2)].players;
        let centres = players.map(|slot| clamp_centre(badge_centre(canvas.min, slot), canvas));
        let order = draw_order(&centres);
        // Painted top of the canvas down (a lower marker covers the one
        // above it), the dragged one last — the keeper's `dragged` is
        // skipped: his slot is fixed, he is painted at his spot, never
        // at the pointer.
        let dragged = order.iter().copied().find(|index| {
            players[*index].position != 0
                && responses[*index]
                    .as_ref()
                    .expect("every marker interacted")
                    .dragged()
        });
        // The starter a release now would swap with — the same pick the
        // drop makes, so the mark and the release always agree. The
        // dragged marker sits on him, so it goes half transparent over
        // him and his corners are painted over both (the draft's
        // `drawPitch` target mark).
        let swap = dragged.and_then(|index| {
            ui.ctx()
                .pointer_interact_pos()
                .and_then(|pos| swap_target(&players, &centres, canvas, pos, index))
        });
        let mut still_for: Option<f32> = None;
        let mut record = |until: Option<f32>| {
            if let Some(until) = until {
                still_for = Some(still_for.map_or(until, |min: f32| min.min(until)));
            }
        };
        for index in order {
            if Some(index) != dragged {
                record(paint_marker(
                    &painter,
                    canvas,
                    tactics,
                    view,
                    roster,
                    Spot {
                        index,
                        slot: players[index],
                        centre: centres[index],
                    },
                    seconds,
                ));
            }
        }
        // The dragged marker's drawn place (the clamped pointer while
        // he is held): the cross pass reads it too.
        let dragged_at = dragged.map(|index| {
            ui.ctx()
                .pointer_interact_pos()
                .map_or(centres[index], |pos| clamp_centre(pos, canvas))
        });
        if let Some(index) = dragged {
            // He goes half transparent while he sits on a swap target.
            let mut painter = painter.clone();
            if swap.is_some() {
                painter.multiply_opacity(0.35);
            }
            record(paint_marker(
                &painter,
                canvas,
                tactics,
                view,
                roster,
                Spot {
                    index,
                    slot: players[index],
                    centre: dragged_at.expect("the dragged marker's centre"),
                },
                seconds,
            ));
        }
        paint_formation_name(
            &painter,
            canvas,
            &tactics.presets[view.preset.min(2)].formations[view.formation.min(2)],
        );
        // The swap mark over everything, as the screen's selection marks
        // a target.
        if let Some(target) = swap {
            paint_corners(
                &painter,
                portrait_rect(badge_rect(centres[target])),
                SELECTED_CORNERS,
            );
        }
        // Live AATF's `FieldedNotA` crosses (the plan's
        // field-a-violation-is-about sub-bullet's `FieldedNotA`
        // parenthesis; DECISIONS "Live AATF's pitch cross"): after every
        // marker and the swap mark, so a marker drawn later cannot
        // cover one — the `✖` right-aligned `MARK_GAP` left of the
        // starter's badge, centred on it, in the marks' highest
        // severity's colour, its tooltip their lines. Only while the
        // shown formation is one the shown preset fields.
        if tactics.presets[view.preset.min(2)]
            .fielded_formations()
            .contains(&view.formation.min(2))
        {
            for index in order {
                let marks: Vec<&FieldedMark> = roster
                    .get(usize::from(tactics.starting_eleven[index]))
                    .map(|row| {
                        row.fielded
                            .iter()
                            .filter(|mark| {
                                mark.preset == view.preset.min(2)
                                    && mark.position == players[index].position
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if marks.is_empty() {
                    continue;
                }
                let centre = if Some(index) == dragged {
                    dragged_at.expect("the dragged marker's centre")
                } else {
                    centres[index]
                };
                let badge = badge_rect(centre);
                let color = if marks.iter().any(|mark| mark.error) {
                    ui.visuals().error_fg_color
                } else {
                    ui.visuals().warn_fg_color
                };
                let galley = painter.layout_no_wrap(
                    CROSS.to_string(),
                    egui::TextStyle::Body.resolve(ui.style()),
                    color,
                );
                let pos = egui::pos2(
                    badge.left() - MARK_GAP - galley.size().x,
                    badge.center().y - galley.size().y / 2.0,
                );
                painter.galley(pos, galley.clone(), color);
                ui.interact(
                    egui::Rect::from_min_size(pos, galley.size()),
                    ui.scope_id().with(("live aatf pitch cross", index)),
                    egui::Sense::hover(),
                )
                .on_hover_text(
                    marks
                        .iter()
                        .map(|mark| mark.line.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
        }
        // The next name move asks the repaint (none when every name fits).
        if let Some(still_for) = still_for {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f32(still_for));
        }
        slot_popover(ui, canvas, tactics, view, roster);
        *tactics != before
    })
    .inner
}

/// A slot's badge centre on the canvas (the screen's own spot mapping,
/// the draft's `badgeAt` before its clamp): an outfield slot at
/// (`SPOT_X0` + `SPOT_XS` x, `SPOT_Y0` - `SPOT_YS` y), high y (the attack
/// end) at the top; the keeper at (`KEEPER_X`, `KEEPER_Y`), where the
/// screen draws him. `origin` is the canvas' top-left.
fn badge_centre(origin: egui::Pos2, slot: FormationSlot) -> egui::Pos2 {
    if slot.position == 0 {
        return origin + egui::vec2(KEEPER_X, KEEPER_Y);
    }
    origin
        + egui::vec2(
            SPOT_X0 + SPOT_XS * f32::from(slot.x),
            SPOT_Y0 - SPOT_YS * f32::from(slot.y),
        )
}

/// `pos` inside the canvas (from `origin`) snapped back through the
/// mapping to the stored grid (the draft's `snapXY`): the nearest unit,
/// clamped to the grid.
fn snap(pos: egui::Pos2, origin: egui::Pos2) -> (u8, u8) {
    let local = pos - origin;
    let x = ((local.x - SPOT_X0) / SPOT_XS)
        .round()
        .clamp(0.0, f32::from(GRID_X)) as u8;
    let y = ((SPOT_Y0 - local.y) / SPOT_YS)
        .round()
        .clamp(0.0, f32::from(GRID_Y)) as u8;
    (x, y)
}

/// `centre` clamped so the marker's whole box stays inside the canvas
/// (the draft's `clampBadge`).
fn clamp_centre(centre: egui::Pos2, canvas: egui::Rect) -> egui::Pos2 {
    egui::pos2(
        centre.x.clamp(
            canvas.left() + BADGE_W / 2.0,
            canvas.right() - BADGE_W / 2.0,
        ),
        centre
            .y
            .clamp(canvas.top() + ABOVE_CENTRE, canvas.bottom() - BELOW_CENTRE),
    )
}

/// The marker's whole box from the badge centre — the badge's width, the
/// portrait's top to the name's bottom — the centre clamped so the box
/// stays inside the canvas (the draft's `markerRect`).
fn marker_rect(centre: egui::Pos2, canvas: egui::Rect) -> egui::Rect {
    let centre = clamp_centre(centre, canvas);
    egui::Rect::from_min_max(
        centre - egui::vec2(BADGE_W / 2.0, ABOVE_CENTRE),
        centre + egui::vec2(BADGE_W / 2.0, BELOW_CENTRE),
    )
}

/// The order the screen draws markers in (the draft's `drawOrder`): top
/// of the canvas first, so a lower marker covers the one above it; ties
/// keep the slot order.
fn draw_order(centres: &[egui::Pos2; 11]) -> [usize; 11] {
    let mut order: [usize; 11] = std::array::from_fn(|index| index);
    order.sort_by(|a, b| centres[*a].y.total_cmp(&centres[*b].y).then(a.cmp(b)));
    order
}

/// The starter a release at `pos` would swap with (the draft's
/// `dropTarget`): the marker drawn on top among the others — `skip` is
/// the dragged one's own spot, never a target — unless it is the
/// keeper's (position 0), whose slot's position and spot are fixed, so
/// over him there is none. The swap mark and the drop both read it, so
/// they always agree.
fn swap_target(
    players: &[FormationSlot; 11],
    centres: &[egui::Pos2; 11],
    canvas: egui::Rect,
    pos: egui::Pos2,
    skip: usize,
) -> Option<usize> {
    draw_order(centres)
        .into_iter()
        .rev()
        .find(|&index| index != skip && marker_rect(centres[index], canvas).contains(pos))
        .filter(|&index| players[index].position != 0)
}

/// The badge's rect around its centre (50 x 14).
fn badge_rect(centre: egui::Pos2) -> egui::Rect {
    egui::Rect::from_center_size(centre, egui::vec2(BADGE_W, BADGE_H))
}

/// The portrait box standing on `badge`.
fn portrait_rect(badge: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(badge.left(), badge.top() - PORTRAIT_H),
        egui::vec2(BADGE_W, PORTRAIT_H),
    )
}

/// The screen's corner brackets around `rect` (the draft's `corners`):
/// a 9 px arm inward from each corner.
fn paint_corners(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(2.0, color);
    let arm = 9.0;
    let corner = |pos: egui::Pos2, dx: f32, dy: f32| {
        painter.line_segment([pos + egui::vec2(0.0, dy * arm), pos], stroke);
        painter.line_segment([pos, pos + egui::vec2(dx * arm, 0.0)], stroke);
    };
    corner(rect.left_top(), 1.0, 1.0);
    corner(rect.right_top(), -1.0, 1.0);
    corner(rect.left_bottom(), 1.0, -1.0);
    corner(rect.right_bottom(), -1.0, -1.0);
}

/// The formation's name in the canvas' bottom-left corner, as the game's
/// formation screen writes it (the draft's `FORMATION_LABEL`): white, in
/// the `label` family (the theme's Selawik Semibold) at `LABEL_SIZE`, its
/// text origin `LABEL_LEFT` in and its baseline `LABEL_BOTTOM` up. egui
/// places a galley by its top-left corner and its first glyph's origin
/// at the row's offset plus the glyph's own (the baseline, a font ascent
/// down), so the galley goes that far above and left of the wanted
/// origin.
fn paint_formation_name(painter: &egui::Painter, canvas: egui::Rect, formation: &Formation) {
    let galley = painter.layout_no_wrap(
        formation_name(formation),
        egui::FontId::new(LABEL_SIZE, egui::FontFamily::Name("label".into())),
        egui::Color32::WHITE,
    );
    let row = galley.rows.first().expect("a laid-out galley has a row");
    // An empty name (no outfield slot in a known position) draws nothing.
    let Some(glyph) = row.glyphs.first() else {
        return;
    };
    let origin = row.pos.to_vec2() + glyph.pos.to_vec2();
    let at = canvas.left_bottom() + egui::vec2(LABEL_LEFT, -LABEL_BOTTOM) - origin;
    painter.galley(at, galley, egui::Color32::WHITE);
}

/// A name no wider than the badge fits: it is centred and never moves
/// (the plan's "a name that fits never moves").
fn name_fits(width: f32) -> bool {
    width <= BADGE_W
}

/// The leftward offset of a name `width` wide `seconds` after the
/// scroll's clock started (the draft's `nameScroll`): it holds still
/// `NAME_PERIOD` less its `width + NAME_GAP` loop at `NAME_SPEED`, then
/// moves `NAME_SPEED` a second through that loop, ending where it began.
fn name_scroll(width: f32, seconds: f32) -> f32 {
    name_moving_for(width, seconds).max(0.0) * NAME_SPEED
}

/// How long until the name `width` wide next moves at `seconds`: 0 while
/// it moves.
fn name_still_for(width: f32, seconds: f32) -> f32 {
    (-name_moving_for(width, seconds)).max(0.0)
}

/// How long the name `width` wide has been moving at `seconds` in its
/// period, negative while it holds still (minus the time left before it
/// moves): the phase `name_scroll` and `name_still_for` both read.
fn name_moving_for(width: f32, seconds: f32) -> f32 {
    let moving = (width + NAME_GAP) / NAME_SPEED;
    seconds % NAME_PERIOD - (NAME_PERIOD - moving)
}

/// The position text's colour by the position's group, as the game's
/// (the draft's `POS_COLOUR` over `POSITIONS`' order): a position past
/// the table white, as `position_label` reads it "?".
fn position_colour(position: u8) -> egui::Color32 {
    match position {
        0 => KEEPER,
        1..=3 => DEFENDER,
        4..=8 => MIDFIELDER,
        9..=12 => FORWARD,
        _ => egui::Color32::WHITE,
    }
}

/// The turf image (the draft's `turf`): the game texture's size, eleven
/// horizontal bands dark at both ends, and the texture's grain as a
/// fixed-seed xorshift32 — deterministic, so the loaded texture is made
/// once per context.
fn turf_image() -> egui::ColorImage {
    let (width, height) = (TURF_SIZE[0], TURF_SIZE[1]);
    let mut pixels = Vec::with_capacity(width * height);
    // xorshift32 from a fixed seed: the same grain every run.
    let mut state = 0x9e37_79b9u32;
    for y in 0..height {
        let base = if (y * TURF_BANDS / height).is_multiple_of(2) {
            TURF_DARK
        } else {
            TURF_LIGHT
        };
        for _x in 0..width {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let noise = ((state as f32 / u32::MAX as f32 - 0.5) * 12.1).round() as i32;
            let channel = |base: u8| (i32::from(base) + noise).clamp(0, 255) as u8;
            pixels.push(egui::Color32::from_rgb(
                channel(base[0]),
                channel(base[1]),
                channel(base[2]),
            ));
        }
    }
    egui::ColorImage::new(TURF_SIZE, pixels)
}

/// The turf texture, loaded once per context under a fixed id.
fn turf(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::unique("pitch turf");
    if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return texture;
    }
    let texture = ctx.load_texture("pitch turf", turf_image(), egui::TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

/// The turf and the lines (the draft's `drawLines`): the halfway line,
/// the centre circle, and per end the penalty box, the goal area and the
/// penalty arc — no touchlines and no goal lines (the texture has none).
fn paint_pitch(painter: &egui::Painter, canvas: egui::Rect) {
    let stroke = egui::Stroke::new(2.0, LINES);
    painter.image(
        turf(painter.ctx()).id(),
        canvas,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    let at = |x: f32, y: f32| canvas.min + egui::vec2(x, y);
    painter.line_segment([at(0.0, 192.0), at(320.0, 192.0)], stroke);
    painter.circle_stroke(at(160.0, 192.0), 36.0, stroke);
    for top in [true, false] {
        // The end's goal line and the direction into the field.
        let (edge, dir) = if top { (0.0, 1.0) } else { (384.0, -1.0) };
        let line = |x: f32, depth: f32| at(x, edge + dir * depth);
        // The penalty box, 160 wide centred and 66 deep: its two sides
        // and its far edge (no goal line).
        for x in [80.0, 240.0] {
            painter.line_segment([line(x, 0.0), line(x, 66.0)], stroke);
        }
        painter.line_segment([line(80.0, 66.0), line(240.0, 66.0)], stroke);
        // The goal area, x 123 to 197 and 22 deep.
        for x in [123.0, 197.0] {
            painter.line_segment([line(x, 0.0), line(x, 22.0)], stroke);
        }
        painter.line_segment([line(123.0, 22.0), line(197.0, 22.0)], stroke);
        // The penalty arc: the centre circle's radius about the spot 44
        // from the goal line, only its part beyond the box's far edge
        // (where a point is more than 22 deep).
        let spot = line(160.0, 44.0);
        let half = (22.0f32 / 36.0).acos();
        let mid = if top {
            std::f32::consts::FRAC_PI_2
        } else {
            -std::f32::consts::FRAC_PI_2
        };
        let points: Vec<egui::Pos2> = (0..17)
            .map(|i| {
                let angle = mid - half + 2.0 * half * i as f32 / 16.0;
                spot + egui::vec2(36.0 * angle.cos(), 36.0 * angle.sin())
            })
            .collect();
        painter.line(points, stroke);
    }
}

/// One marker (the draft's chip grown to the screen's marker): the
/// slot's position over the starter's name, its AccessKit node a button
/// labeled with both. The keeper's slot (position 0) senses like the
/// others but sets no drag payload, so his marker never moves — and his
/// release is his click however far the pointer moved (the draft's
/// `d.gk`). A click serves the armed mode when one is armed, else opens
/// the slot's popover. The answer is the marker's
/// response, for the caller's drop check; `paint_marker` draws it.
fn marker(
    ui: &mut egui::Ui,
    canvas: egui::Rect,
    version: PesVersion,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
    spot: Spot,
) -> egui::Response {
    let keeper = spot.slot.position == 0;
    // The keeper senses drag like the others although he never moves:
    // egui's hit test answers a drag with the topmost drag-sensing
    // widget, so click-only would send a press on him to the marker
    // underneath.
    let response = ui.interact(
        marker_rect(spot.centre, canvas),
        ui.scope_id().with(("pitch marker", spot.index)),
        egui::Sense::click_and_drag(),
    );
    let name = name_at(roster, tactics.starting_eleven[spot.index]);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::Button,
            ui.is_enabled(),
            format!("{} {}", position_label(spot.slot.position), name),
        )
    });
    if !keeper {
        response.dnd_set_drag_payload(LineupDrag::Marker(spot.index));
    }
    // The keeper's release is his click however far the pointer moved:
    // his marker never moves (the draft's `d.gk` never sets `moved`), so
    // a dragged press on him is no drag.
    if response.clicked() || (keeper && response.drag_stopped()) {
        match view.assign {
            Some(assign) => assign.write(version, tactics, view, spot.index),
            None => {
                view.popover = Some(OpenSlot {
                    scope: ui.scope_id(),
                    preset: view.preset,
                    formation: view.formation,
                    slot: spot.index,
                    at: marker_rect(spot.centre, canvas).left_bottom() - canvas.min,
                });
            }
        }
    }
    response
}

/// One marker painted (the draft's `drawPitch` body): the portrait, the
/// badge with the position's abbreviation in its group's colour, the name
/// in its colour codes' colours (white where it has none), the captain's
/// mark when he is the starter, and the selection corners the screen's
/// marks lend the editor's states. `spot.centre` is the clamped badge
/// centre (the clamped pointer while dragged); `seconds` is the name
/// scroll's clock. Answers how long until the name next moves, `None`
/// when it fits and never moves.
fn paint_marker(
    painter: &egui::Painter,
    canvas: egui::Rect,
    tactics: &TeamTactics,
    view: &TacticsView,
    roster: &[RosterRow<'_>],
    spot: Spot,
    seconds: f32,
) -> Option<f32> {
    let roster_slot = tactics.starting_eleven[spot.index];
    let badge = badge_rect(spot.centre);
    let portrait = portrait_rect(badge);
    if let Some(texture) = roster
        .get(usize::from(roster_slot))
        .and_then(|row| row.portrait)
    {
        painter.image(
            texture,
            portrait,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
    let fill = if view.assign == Some(Assign::Join)
        && tactics.players_to_join_attack.contains(&roster_slot)
    {
        JOIN_FILL
    } else {
        BADGE_FILL
    };
    painter.rect_filled(badge, egui::CornerRadius::same(2), fill);
    painter.text(
        egui::pos2(badge.left() + 3.0, badge.center().y),
        egui::Align2::LEFT_CENTER,
        position_label(spot.slot.position),
        egui::FontId::proportional(12.0),
        position_colour(spot.slot.position),
    );
    // His medal tier's initial at the badge's right end, level with the
    // position text and in its font (the plan's "A starter's badge names
    // his medal tier"); a slot with no medal draws none.
    if let Some(medal) = roster
        .get(usize::from(roster_slot))
        .and_then(|row| row.medal)
    {
        painter.text(
            egui::pos2(badge.right() - 3.0, badge.center().y),
            egui::Align2::RIGHT_CENTER,
            medal.letter(),
            egui::FontId::proportional(12.0),
            medal.colour(),
        );
    }
    // The name in its colour codes' colours (white where it has none),
    // centred under the badge or cut at its right when wider.
    let mut job = egui::text::LayoutJob::default();
    append_name(&mut job, raw_name_at(roster, roster_slot));
    for section in &mut job.sections {
        section.format.font_id = egui::FontId::proportional(12.0);
        if section.format.color == egui::Color32::PLACEHOLDER {
            section.format.color = egui::Color32::WHITE;
        }
    }
    let galley = painter.layout_job(job);
    let size = galley.size();
    let wide = !name_fits(size.x);
    // Scroll names off: the scroll's resting phase — the name still at
    // the badge's left, cut to its width — and no repaint to ask for.
    let scroll = if wide && view.scroll_names {
        name_scroll(size.x, seconds)
    } else {
        0.0
    };
    let left = if wide {
        badge.left() - scroll
    } else {
        spot.centre.x - size.x / 2.0
    };
    let clip = egui::Rect::from_min_max(
        egui::pos2(badge.left(), canvas.top()),
        egui::pos2(badge.right(), canvas.bottom()),
    );
    let clipped = painter.with_clip_rect(clip);
    let top = spot.centre.y + NAME_DY - size.y / 2.0;
    clipped.galley(egui::pos2(left, top), galley.clone(), egui::Color32::WHITE);
    if wide {
        // Its repeat a loop to the right, so the start follows the end.
        clipped.galley(
            egui::pos2(left + size.x + NAME_GAP, top),
            galley,
            egui::Color32::WHITE,
        );
    }
    if roster_slot == tactics.set_pieces.captain {
        let mark = egui::Rect::from_min_max(
            egui::pos2(badge.left() + 1.0, badge.top() - 16.0),
            egui::pos2(badge.left() + 13.0, badge.top() - 5.0),
        );
        painter.rect_filled(mark, egui::CornerRadius::same(2), egui::Color32::WHITE);
        painter.text(
            mark.center(),
            egui::Align2::CENTER_CENTER,
            "C",
            egui::FontId::proportional(9.0),
            egui::Color32::BLACK,
        );
    }
    let corners = if view.popover.is_some_and(|open| open.slot == spot.index) {
        Some(SELECTED_CORNERS)
    } else if view.assign.is_some() {
        Some(CANDIDATE_CORNERS)
    } else {
        None
    };
    if let Some(color) = corners {
        paint_corners(painter, portrait, color);
    }
    (wide && view.scroll_names).then(|| name_still_for(size.x, seconds))
}

/// The formation's name as PES 2021's formation screen writes it in the
/// pitch's corner (`3-3-1-3`): the ten outfield slots counted by the row
/// of `POSITION_ROWS` their position belongs to, back to front, an empty
/// row left out. The keeper never counts, and the spot plays no part: an
/// AMF given CMF at his own spot turns `3-3-1-3` into `3-4-3`.
pub(crate) fn formation_name(formation: &Formation) -> String {
    // Every row but the last, GK's.
    POSITION_ROWS[..4]
        .iter()
        .rev()
        .map(|row| {
            formation
                .players
                .iter()
                .filter(|slot| row.contains(&position_label(slot.position)))
                .count()
        })
        .filter(|&count| count > 0)
        .map(|count| count.to_string())
        .collect::<Vec<_>>()
        .join("-")
}

/// The position byte's label (`POSITIONS`), "?" past the table as the
/// player list reads it.
pub(crate) fn position_label(position: u8) -> &'static str {
    POSITIONS.get(usize::from(position)).copied().unwrap_or("?")
}

#[cfg(test)]
mod tests;
