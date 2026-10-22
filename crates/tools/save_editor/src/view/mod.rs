//! The Save editor's window (`plans/save_editor.md` "Editing UI"): a toolbar over a
//! master-detail layout, the player list left (`player_list`) and the general info
//! strip right (`info_strip`) over the tab strip and the Abilities & Skills tab
//! (`abilities`). Render-only over the session's state: it opens a
//! save, filters and selects players, reorders a team's list by drag
//! (`Session::move_player`), edits the strip's fields through
//! `Session::edit_player`/`edit_team` (one undo entry per gesture), saves, and
//! shows a refusal in a modal.

mod abilities;
mod info_strip;
mod player_list;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use pes_savefile::model::PlayerEntry;
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
    /// The ruleset the badges use, loaded on the first frame and again only
    /// when the file named differs.
    rules: Option<ActiveRules>,
    /// The "Cards left" badges as last computed and the `BadgeKey` they
    /// were computed for: `card_limits` runs again only when the key
    /// differs, not on every frame.
    cards: Option<(BadgeKey, Vec<(String, aatf::CardLimits)>)>,
    /// Which open this is: part of the pushed widget id, so egui widget
    /// state (a text field's undo history, a control's in-progress edit)
    /// never crosses into the next save's same player id.
    generation: u64,
}

/// The ruleset the Abilities tab's badges use (`plans/save_editor.md`
/// "Editing UI", the Cards left bullet): the file it was loaded from and
/// the outcome as the badges' row draws it.
pub(crate) struct ActiveRules {
    /// The file `rules` was loaded from (`None`: the embedded official ruleset).
    file: Option<PathBuf>,
    /// The loaded rules or the load error's text (drawn instead of the badges).
    rules: Result<aatf::Rules, String>,
}

/// What the Abilities tab's badges were computed for: a change of the open
/// save (`Session::revision`), of the selection or of the rules file
/// recomputes them; anything else keeps a frame's `card_limits` cost off.
#[derive(PartialEq)]
pub(crate) struct BadgeKey {
    /// `Session::revision` when computed.
    revision: u64,
    /// The selected team.
    team: u32,
    /// The selected roster slot (0-based).
    slot: u8,
    /// The rules file the badges' ruleset came from (`None`: the official
    /// ruleset).
    rules_file: Option<PathBuf>,
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
            rules: None,
            cards: None,
            generation: 0,
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
                self.generation += 1;
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
/// `rules_file` is the AATF rules file the `aatf_rules` setting resolved
/// to; the ruleset loads on the first frame and again only when it differs.
pub(crate) fn show(
    ui: &mut egui::Ui,
    view: &mut SaveView,
    running: &dyn Fn() -> Vec<RunningPes>,
    rules_file: Option<&Path>,
) {
    let file = rules_file.map(Path::to_path_buf);
    if view.rules.as_ref().is_none_or(|active| active.file != file) {
        view.rules = Some(ActiveRules {
            rules: crate::session::rules::load_rules(file.as_deref())
                .map_err(|error| format!("{error:#}")),
            file,
        });
    }
    toolbar(ui, view, running);
    if view.session.is_none() {
        ui.weak("No save open.");
    } else {
        egui::Panel::left("save_editor_players")
            .resizable(true)
            .show(ui, |ui| player_list::show(ui, view));
        egui::CentralPanel::default().show(ui, |ui| {
            info_strip::show(ui, view);
            tab_strip(ui);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| abilities::show(ui, view));
        });
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
    // Ctrl+Shift+Z first: `consume_key(Ctrl, Z)` matches it too, so the more
    // specific shortcut has to be tested before the plainer one.
    let redo_key = !typing
        && ui.input_mut(|input| {
            input.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z)
                || input.consume_key(egui::Modifiers::CTRL, egui::Key::Y)
        });
    let undo_key = !typing
        && !redo_key
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Z));
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

/// The detail pane's tabs (`plans/save_editor.md` "Editing UI", the Tabs
/// bullet): `Abilities & Skills` selected; the other three drawn disabled
/// until their slices land. No tab enum yet: there is one tab to show.
fn tab_strip(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(egui::Button::selectable(true, "Abilities & Skills"));
        for label in ["Appearance", "Team", "Tactics"] {
            ui.add_enabled(false, egui::Button::selectable(false, label))
                .on_disabled_hover_text("Not available yet");
        }
    });
}

/// Clones the player's entry, applies `edit` and pushes it under `key` (the
/// widget's `egui::Id` value, the undo stack's merge key).
pub(crate) fn edit_player(
    session: &mut Session,
    id: u32,
    key: u64,
    edit: impl FnOnce(&mut PlayerEntry),
) {
    let mut edited = session
        .file()
        .player(id)
        .expect("the selected player")
        .clone();
    edit(&mut edited);
    session.edit_player(key, edited);
}

/// The selected player's `(team_id, roster slot, player_id)`; `None` when
/// the selection names no rostered player (a stale slot, an empty save's
/// list).
pub(crate) fn selected_player(view: &SaveView) -> Option<(u32, u8, u32)> {
    let (team_id, slot) = view.selected?;
    let file = &view.session.as_ref()?.file();
    let player_id = file.team(team_id)?.roster.get(usize::from(slot))?.player_id;
    file.player(player_id)?;
    Some((team_id, slot, player_id))
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
