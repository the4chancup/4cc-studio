//! The Save editor's window (`plans/save_editor.md` "Editing UI"): a toolbar over a
//! master-detail layout, the player list left (`player_list`) and the general info
//! strip right (`info_strip`). Render-only over the session's state; the undo stack
//! and the edits the strip's controls make arrive with the tabs' slice.

mod info_strip;
mod player_list;
#[cfg(test)]
mod tests;

use std::path::Path;

use studio_core::RunningPes;

use crate::session::Session;
use crate::session::selection::{self, TeamChoice};

/// The window's state: the open save and what the master-detail shows of it.
pub(crate) struct SaveView {
    /// The open save, if one was opened.
    session: Option<Session>,
    /// Which team the list shows.
    team: TeamChoice,
    /// The filter box's text.
    filter: String,
    /// The selected player, by team and roster slot.
    selected: Option<(u32, u8)>,
    /// A refusal to show until dismissed (an open the save refused).
    refusal: Option<Refusal>,
}

/// A refusal shown as a dialog: a title, the message and OK.
struct Refusal {
    /// The dialog's heading.
    title: &'static str,
    /// What was refused, as the operation reported it.
    message: String,
}

impl Default for SaveView {
    fn default() -> SaveView {
        SaveView::new()
    }
}

impl SaveView {
    /// The window as it starts: no save open.
    pub(crate) fn new() -> SaveView {
        SaveView {
            session: None,
            team: TeamChoice::All,
            filter: String::new(),
            selected: None,
            refusal: None,
        }
    }

    /// Opens the save at `path`: on success the selector reads `ALL`, the
    /// filter is empty and the first listed player is selected; a file the
    /// save refuses is shown as a refusal and the open session is untouched.
    pub(crate) fn open(&mut self, path: &Path) {
        match Session::open(path) {
            Ok(session) => {
                self.team = TeamChoice::All;
                self.filter.clear();
                self.selected = selection::rows(session.file(), &self.team, "")
                    .first()
                    .map(|row| (row.team_id, row.slot));
                self.session = Some(session);
                self.refusal = None;
            }
            Err(error) => {
                self.refusal = Some(Refusal {
                    title: "Cannot open the save",
                    message: format!("{error:#}"),
                });
            }
        }
    }
}

/// Shows the window for one frame. `running` is the running-PES probe the
/// save refusal checks (the Save button's slice wires it).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView, _running: &[RunningPes]) {
    ui.horizontal(|ui| {
        if ui.button("Open…").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_file()
        {
            view.open(&path);
        }
    });
    if view.session.is_none() {
        ui.weak("No save open.");
    } else {
        egui::Panel::left("save_editor_players")
            .resizable(true)
            .show(ui, |ui| player_list::show(ui, view));
        egui::CentralPanel::default().show(ui, |ui| info_strip::show(ui, view));
    }
    refusal_dialog(ui, view);
}

/// The refusal's dialog, while `view.refusal` holds one.
fn refusal_dialog(ui: &mut egui::Ui, view: &mut SaveView) {
    let mut close = false;
    if let Some(refusal) = view.refusal.as_ref() {
        let response =
            egui::Modal::new(egui::Id::new("save_editor_refusal")).show(ui.ctx(), |ui| {
                ui.heading(refusal.title);
                ui.label(&refusal.message);
                if ui.button("OK").clicked() {
                    close = true;
                }
            });
        close |= response.should_close();
    }
    if close {
        view.refusal = None;
    }
}
