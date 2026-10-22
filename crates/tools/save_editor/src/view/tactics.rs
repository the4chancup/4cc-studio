//! The Tactics tab (`plans/save_editor.md` "Tactics editor"; the draft's
//! `tacticsTab`): `team_widgets`' tactics settings over the selected
//! player's team — the preset selector, the Formation settings, Advanced
//! instructions and Automatic settings groups — plus the tab's own
//! Copy/swap preset and Copy/swap formation menus (the editor's alone,
//! the plan's placement rules; `session::tactics_ops`). The pitch and the
//! lineup land with the next part. The widget edits a clone of the team's
//! tactics and reports a change; the tab pushes the clone back with
//! `Session::edit_team` under one key per team and tab, so a slider drag
//! is one undo entry (`seal` runs when the pointer is up and no widget
//! has focus).

use pes_savefile::model::names::display_name;
use team_widgets::tactics::tactics_settings;

use crate::session::Session;
use crate::view::{SaveView, edit_team, selected_player};

/// The formations' names, in the record's order.
const FORMATIONS: [&str; 3] = ["Kick-off", "In-possession", "Out-of-possession"];

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
    // The target pickers' labels: the roster's display names as the
    // player list shows them, indexed by roster slot ("" an empty one).
    let names: Vec<String> = team
        .roster
        .iter()
        .map(|slot| {
            session
                .file()
                .player(slot.player_id)
                .map(|player| display_name(&player.name))
                .unwrap_or_default()
        })
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    // Every widget id under the tab is the team's within this open (the
    // Team tab's rule): a combo's popup, a DragValue's in-progress edit
    // and the widget's state never cross teams nor saves.
    ui.push_id((*generation, team_id), |ui| {
        ui.horizontal(|ui| {
            copy_swap_preset_menu(ui, session, team_id);
            copy_swap_formation_menu(ui, session, team_id, tactics.preset);
        });
        let mut edited = team.tactics.clone();
        if tactics_settings(
            ui,
            "tactics settings",
            version,
            &mut edited,
            tactics,
            &names,
        ) {
            edit_team(
                session,
                team_id,
                egui::Id::new(("tactics", team_id)).value(),
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
