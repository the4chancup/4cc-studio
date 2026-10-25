//! The Tactics tab (`plans/save_editor.md` "Tactics editor"; the draft's
//! `tacticsTab`): `team_widgets`' pitch, lineup lists and tactics settings
//! over the selected player's team — the formation canvas with its markers
//! and slot popover, the starting eleven and the bench with swap-by-drag,
//! the preset selector, the Formation settings, Advanced instructions,
//! Player assignments and Automatic settings groups — plus the tab's own
//! Copy/swap preset and Copy/swap formation menus (the editor's alone,
//! the plan's placement rules; `session::tactics_ops`). The widgets edit a
//! clone of the team's tactics and report a change; the tab pushes the
//! clone back with `Session::edit_team` under one key per widget call
//! (pitch, lineup, settings), so a gesture's frames merge but different
//! widgets' edits never do (`seal` runs when the pointer is up and no
//! widget has focus).

use pes_savefile::model::names::display_name;
use pes_savefile::model::tactics::TeamTactics;
use pes_version::PesVersion;
use team_widgets::pitch::{self, FORMATIONS};
use team_widgets::tactics::{self, RosterRow};

use crate::session::Session;
use crate::view::{SaveView, edit_team, selected_team};

/// The lists column's width (the draft's `#tac-lists`): the Starting
/// eleven and Bench lists' fixed narrow column in both arrangements.
/// 200, not the draft's 240: at the default window's 992-point tab area
/// the settings column then gets 456 points, where the preset bar holds
/// one line and the whole column fits the pane; a long name is cut with
/// the full name on hover (`DECISIONS.md` 2026-10-25, "The Tactics
/// settings column fits the pane at 1280 × 800, the lists stay at 200").
pub(super) const LISTS_WIDTH: f32 = 200.0;
/// The settings column's break width (the draft's `#tac-settings` flex
/// base): beside the pitch and the lists on a wide pane, under them at
/// the pane's width on a narrow one.
pub(super) const SETTINGS_MIN: f32 = 360.0;

/// The tab for the selected player's team, in the strip's `Team` group.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, version, team)) = selected_team(view) else {
        return;
    };
    let SaveView {
        session,
        tactics,
        generation,
        focused,
        portraits,
        ..
    } = view;
    let Some(session) = session.as_mut() else {
        return;
    };
    // The roster as the widgets list it: the display names as the player
    // list shows them, the raw names for the colour-coded rows and the
    // registered positions, indexed by roster slot (an empty slot "" and
    // none).
    let rows: Vec<(String, String, Option<u8>)> = team
        .roster
        .iter()
        .map(|slot| {
            session.file().player(slot.player_id).map_or_else(
                || (String::new(), String::new(), None),
                |player| {
                    (
                        display_name(&player.name),
                        player.name.clone(),
                        Some(player.positions.registered),
                    )
                },
            )
        })
        .collect();
    // The roster's player ids (`None` an empty slot): the portrait source's ask.
    let player_ids: Vec<Option<u32>> = team
        .roster
        .iter()
        .map(|slot| {
            session
                .file()
                .player(slot.player_id)
                .map(|player| player.id)
        })
        .collect();
    let textures = portraits.textures(ui.ctx(), version, &player_ids);
    let roster: Vec<RosterRow> = rows
        .iter()
        .zip(textures)
        .map(|((name, raw_name, position), portrait)| RosterRow {
            name: name.as_str(),
            raw_name: raw_name.as_str(),
            registered_position: *position,
            portrait,
        })
        .collect();
    // Every widget id under the tab is the team's within this open (the
    // Team tab's rule): a combo's popup, a DragValue's in-progress edit
    // and the widget's state never cross teams nor saves.
    ui.push_id((*generation, team_id), |ui| {
        let mut edited = team.tactics.clone();
        let mut changed = [false; 3];
        // `three_columns`: the width the draft's `taccols` needs — the
        // pitch column's fixed `PITCH.x`, the lists' `LISTS_WIDTH` (the
        // draft's `#tac-lists`), the settings column's `SETTINGS_MIN`
        // (the draft's `#tac-settings` flex base) and the two
        // `item_spacing.x` gaps between them, computed from constants
        // (never measured back: `DECISIONS.md` 2026-10-24, rows picked by
        // width). Below it the pitch and the lists still sit side by
        // side and the settings column's content draws under them at the
        // pane's width.
        let gap = ui.spacing().item_spacing.x;
        let three_columns = pitch::PITCH.x + LISTS_WIDTH + SETTINGS_MIN + 2.0 * gap;
        // A side-by-side layout built like `ui.columns` but with the widths
        // the draft means: the pitch at its canvas' fixed width, the lists
        // at its right at their own fixed one. `ui.columns` splits the pane
        // evenly instead, and answers a column whose content is wider than
        // its half — the pitch at a narrow window — by widening the ui's
        // `max_rect`; the settings rows' wrap below then reads the
        // overflowed width as its bound.
        let top_left = ui.cursor().min;
        let right = ui.max_rect().right();
        let bottom = ui.max_rect().right_bottom().y;
        let pitch_rect =
            egui::Rect::from_min_max(top_left, egui::pos2(top_left.x + pitch::PITCH.x, bottom));
        let list_rect = egui::Rect::from_min_max(
            egui::pos2(pitch_rect.max.x + gap, top_left.y),
            egui::pos2(pitch_rect.max.x + gap + LISTS_WIDTH, bottom),
        );
        let mut pitch_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(pitch_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        let mut list_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(list_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        changed[0] = pitch::pitch(
            &mut pitch_ui,
            "pitch",
            version,
            &mut edited,
            tactics,
            &roster,
        );
        changed[1] = pitch::lineup(
            &mut list_ui,
            "lineup",
            version,
            &mut edited,
            tactics,
            &roster,
        );
        let block = pitch_ui
            .min_rect()
            .height()
            .max(list_ui.min_rect().height());
        // The settings column's `Ui` takes one explicit `scope_id` in
        // both arrangements, so its widgets' ids — the sliders' among
        // them — are equal either side of `three_columns`: an explicit
        // scope id makes the child's `unique_id` (what the auto widget
        // ids seed from) independent of the call counter, where a bare
        // `id_salt` would still read it.
        let settings_rect = if ui.available_width() >= three_columns {
            // The third column takes what the pitch and the lists leave.
            egui::Rect::from_min_max(
                egui::pos2(list_rect.max.x + gap, top_left.y),
                egui::pos2(right, bottom),
            )
        } else {
            // Below the columns' width it draws whole under them, one
            // item gap down as the next widget below them would sit.
            egui::Rect::from_min_max(
                egui::pos2(top_left.x, top_left.y + block + ui.spacing().item_spacing.y),
                egui::pos2(right, bottom),
            )
        };
        let mut settings_ui = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(ui.make_persistent_id("settings"))
                .max_rect(settings_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        changed[2] = settings_column(
            &mut settings_ui,
            session,
            team_id,
            version,
            tactics,
            &mut edited,
            &roster,
        );
        // One advance covering the columns' block (the tallest child).
        let height = [&pitch_ui, &list_ui, &settings_ui]
            .iter()
            .map(|child| child.min_rect().max.y)
            .fold(top_left.y, f32::max)
            - top_left.y;
        ui.advance_cursor_after_rect(egui::Rect::from_min_size(
            top_left,
            egui::vec2(right - top_left.x, height),
        ));
        // One undo key per widget call (the plan's "changes from one
        // widget merge into the open entry"): the pitch's drag never
        // merges into a settings click or a row drop.
        if let Some(widget) = ["pitch", "lineup", "settings"]
            .into_iter()
            .zip(changed)
            .find_map(|(widget, changed)| changed.then_some(widget))
        {
            // A gesture ends mid-frame when its widget loses the
            // keyboard (a press elsewhere surrenders it): the press's
            // write lands on this same frame under the same key, and
            // `view::show`'s seal runs only after the push. Seal here or
            // it merges into the ended gesture's entry, and record the
            // transition handled: the frame-end check must not seal this
            // gesture's own entry again.
            let now = ui.ctx().memory(|memory| memory.focused());
            if focused.is_some() && *focused != now {
                session.seal();
                *focused = now;
            }
            edit_team(
                session,
                team_id,
                egui::Id::unique(("tactics", team_id, widget)).value(),
                |team| team.tactics = edited,
            );
        }
    });
}

/// The settings column's whole content, in the draft's order (the
/// `tacbar` then `note` then the settings groups): the preset bar with
/// the selector and the two copy/swap menus, the armed mode's note, then
/// `tactics_settings`. On a wide pane it is the third column; below the
/// three-column width it draws under the pitch and the lists at the
/// pane's width, the preset bar first.
fn settings_column(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    version: PesVersion,
    view: &mut tactics::TacticsView,
    edited: &mut TeamTactics,
    roster: &[RosterRow<'_>],
) -> bool {
    ui.horizontal_wrapped(|ui| {
        tactics::preset_selector(ui, view);
        copy_swap_preset_menu(ui, session, team_id);
        copy_swap_formation_menu(ui, session, team_id, view.preset);
    });
    tactics::assign_note(ui, view);
    tactics::tactics_settings(ui, "tactics settings", version, edited, view, roster)
}

/// The "Copy/swap preset" menu (the draft's `presetMenu`): six copies,
/// then three swaps, each a sealed undo entry
/// (`Session::{copy_preset,swap_presets}`).
fn copy_swap_preset_menu(ui: &mut egui::Ui, session: &mut Session, team_id: u32) {
    ui.menu_button("Copy/swap preset", |ui| {
        for from in 0..3 {
            for to in 0..3 {
                if from != to
                    && ui
                        .button(format!("Copy preset {} to {}", from + 1, to + 1))
                        .clicked()
                {
                    session.copy_preset(team_id, from, to);
                    ui.close();
                }
            }
        }
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            if ui
                .button(format!("Swap presets {} and {}", a + 1, b + 1))
                .clicked()
            {
                session.swap_presets(team_id, a, b);
                ui.close();
            }
        }
    });
}

/// The "Copy/swap formation" menu (the draft's `formationMenu`) over the
/// preset on screen: six copies, then three swaps, each a sealed undo
/// entry (`Session::{copy_formation,swap_formations}`).
fn copy_swap_formation_menu(ui: &mut egui::Ui, session: &mut Session, team_id: u32, preset: usize) {
    ui.menu_button("Copy/swap formation", |ui| {
        for (from, &from_name) in FORMATIONS.iter().enumerate() {
            for (to, &to_name) in FORMATIONS.iter().enumerate() {
                if from != to
                    && ui
                        .button(format!("Copy {from_name} to {to_name}"))
                        .clicked()
                {
                    session.copy_formation(team_id, preset, from, to);
                    ui.close();
                }
            }
        }
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            if ui
                .button(format!("Swap {} and {}", FORMATIONS[a], FORMATIONS[b]))
                .clicked()
            {
                session.swap_formations(team_id, preset, a, b);
                ui.close();
            }
        }
    });
}
