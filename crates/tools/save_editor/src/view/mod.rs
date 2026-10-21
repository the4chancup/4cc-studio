//! The Save editor's window (`plans/save_editor.md` "Editing UI"): a toolbar over a
//! master-detail layout, the player list left (`player_list`) and the general info
//! strip right (`info_strip`). Render-only over the session's state: it opens a
//! save, filters and selects players, edits the strip's fields through
//! `Session::edit_player`/`edit_team` (one undo entry per gesture), saves, and
//! shows a refusal in a modal.

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

/// Shows the window for one frame. `running` is the running-PES probe,
/// called only when Save runs (a process snapshot is not a frame's cost).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView, running: &dyn Fn() -> Vec<RunningPes>) {
    toolbar(ui, view, running);
    if view.session.is_none() {
        ui.weak("No save open.");
    } else {
        egui::Panel::left("save_editor_players")
            .resizable(true)
            .show(ui, |ui| player_list::show(ui, view));
        egui::CentralPanel::default().show(ui, |ui| info_strip::show(ui, view));
    }
    refusal_dialog(ui, view);
    // One undo entry per gesture: seal when the pointer is up and no widget
    // holds the keyboard.
    if let Some(session) = view.session.as_mut()
        && !ui.input(|input| input.pointer.any_down())
        && !ui.ctx().egui_wants_keyboard_input()
    {
        session.seal();
    }
}

/// The toolbar: Open, Save, Undo, Redo and the open save's path with its
/// unsaved-changes marker (the draft's toolbar row).
fn toolbar(ui: &mut egui::Ui, view: &mut SaveView, running: &dyn Fn() -> Vec<RunningPes>) {
    // Ctrl+S saves even while a widget holds the keyboard; the other
    // shortcuts do nothing then (a text field's own undo is egui's).
    let save_key = ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::S));
    let typing = ui.ctx().egui_wants_keyboard_input();
    let undo_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Z));
    let redo_key = !typing
        && ui.input_mut(|input| {
            input.consume_key(egui::Modifiers::CTRL, egui::Key::Y)
                || input.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z)
        });
    ui.horizontal(|ui| {
        if ui.button("Open…").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_file()
        {
            view.open(&path);
        }
        let (can_undo, can_redo) = view
            .session
            .as_ref()
            .map(|session| (session.can_undo(), session.can_redo()))
            .unwrap_or((false, false));
        if (ui
            .add_enabled(can_undo, egui::Button::new("Undo"))
            .clicked()
            || undo_key)
            && let Some(session) = view.session.as_mut()
        {
            session.undo();
        }
        if (ui
            .add_enabled(can_redo, egui::Button::new("Redo"))
            .clicked()
            || redo_key)
            && let Some(session) = view.session.as_mut()
        {
            session.redo();
        }
        if ui
            .add_enabled(view.session.is_some(), egui::Button::new("Save"))
            .clicked()
            || save_key
        {
            save(view, running);
        }
        if let Some(session) = view.session.as_ref() {
            let path = session.path().display().to_string();
            let marker = if session.is_dirty() {
                "  \u{2022} unsaved changes"
            } else {
                ""
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("{path}{marker}"))
                        .monospace()
                        .weak(),
                )
                .truncate(),
            );
        }
    });
}

/// `session.save` and the modal its refusal takes (the draft's `saveFile`).
fn save(view: &mut SaveView, running: &dyn Fn() -> Vec<RunningPes>) {
    if let Some(session) = view.session.as_mut() {
        match session.save(&running()) {
            Ok(()) => view.refusal = None,
            Err(error) => {
                view.refusal = Some(Refusal {
                    title: "Save refused",
                    message: format!("{error:#}"),
                });
            }
        }
    }
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
