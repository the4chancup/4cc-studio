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
use team_widgets::pitch::{self, FORMATIONS};
use team_widgets::tactics::{self, RosterRow};

use crate::session::Session;
use crate::view::{SaveView, edit_team, selected_player};

/// The tab for the selected player's team, below `info_strip` and the tab
/// strip.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, _, _)) = selected_player(view) else {
        return;
    };
    let SaveView {
        session,
        tactics,
        generation,
        focused,
        ..
    } = view;
    let Some(session) = session.as_mut() else {
        return;
    };
    let (version, team) = {
        let file = session.file();
        let Some(team) = file.team(team_id) else {
            return;
        };
        (file.version(), team.clone())
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
    let roster: Vec<RosterRow> = rows
        .iter()
        .map(|(name, raw_name, position)| RosterRow {
            name: name.as_str(),
            raw_name: raw_name.as_str(),
            registered_position: *position,
        })
        .collect();
    // Every widget id under the tab is the team's within this open (the
    // Team tab's rule): a combo's popup, a DragValue's in-progress edit
    // and the widget's state never cross teams nor saves.
    ui.push_id((*generation, team_id), |ui| {
        ui.horizontal(|ui| {
            tactics::preset_selector(ui, tactics);
            copy_swap_preset_menu(ui, session, team_id);
            copy_swap_formation_menu(ui, session, team_id, tactics.preset);
        });
        let mut edited = team.tactics.clone();
        // The draft's `note`: an armed mode's note spans both columns.
        tactics::assign_note(ui, tactics);
        // The draft's `taccols`: the pitch in the left column, the lists
        // at its right.
        let mut changed = [false; 3];
        ui.columns(2, |columns| {
            columns[0].vertical(|ui| {
                changed[0] = pitch::pitch(ui, "pitch", version, &mut edited, tactics, &roster);
            });
            columns[1].vertical(|ui| {
                changed[1] = pitch::lineup(ui, "lineup", version, &mut edited, tactics, &roster);
            });
        });
        changed[2] = tactics::tactics_settings(
            ui,
            "tactics settings",
            version,
            &mut edited,
            tactics,
            &roster,
        );
        // One undo key per widget call (the plan's "changes from one
        // widget merge into the open entry"): the pitch's drag never
        // merges into a settings click or a row drop.
        if let Some(widget) = ["pitch", "lineup", "settings"]
            .into_iter()
            .zip(changed)
            .find_map(|(widget, changed)| changed.then_some(widget))
        {
            // A gesture ends mid-frame when its widget loses the
            // keyboard (a click elsewhere surrenders it): the click's
            // write lands on this same frame under the same key, and
            // `view::show`'s seal runs only after the push. Seal here or
            // it merges into the ended gesture's entry.
            if focused.is_some() && *focused != ui.ctx().memory(|memory| memory.focused()) {
                session.seal();
            }
            edit_team(
                session,
                team_id,
                egui::Id::new(("tactics", team_id, widget)).value(),
                |team| team.tactics = edited,
            );
        }
    });
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
