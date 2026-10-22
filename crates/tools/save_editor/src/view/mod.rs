//! The Save editor's window (`plans/save_editor.md` "Editing UI"): a toolbar over a
//! master-detail layout, the player list left (`player_list`) and the general info
//! strip right (`info_strip`) over the tab strip and the Abilities & Skills tab
//! (`abilities`). Render-only over the session's state: it opens a
//! save, filters and selects players, reorders a team's list by drag
//! (`Session::move_player`), edits the strip's fields through
//! `Session::edit_player`/`edit_team` (one undo entry per gesture), saves, and
//! shows a refusal in a modal.

mod abilities;
mod appearance;
mod info_strip;
mod menus;
mod player_list;
mod tactics;
mod team;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use pes_savefile::model::names::name_spans;
use pes_savefile::model::{PlayerEntry, TeamEntry};
use pes_version::Engine;
use studio_core::RunningPes;

use crate::session::Session;
use crate::session::selection::{self, TeamChoice};

/// The detail pane's drawn tabs (`plans/save_editor.md` "Editing UI", the
/// Tabs bullet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tab {
    /// Abilities & Skills.
    Abilities,
    /// Appearance (`appearance`).
    Appearance,
    /// The selected player's team (`team`).
    Team,
    /// The team's tactics (`tactics`).
    Tactics,
}

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
    /// The drawn tab.
    tab: Tab,
    /// The Tactics tab's on-screen state (`team_widgets`' `TacticsView`:
    /// the preset its groups edit); back to preset 1 on every open.
    tactics: team_widgets::tactics::TacticsView,
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
    /// "Edit aesthetics anyway" (`plans/save_editor.md` "Read-only
    /// aesthetics", the Unlock bullet): off at every start of the tool, and
    /// deliberately not reset when another save opens — a session switch,
    /// not a per-save one.
    edit_aesthetics: bool,
    /// The players the Strip again window lists (SE-GUI-20): the rows
    /// `Session::restored_players` named when the open save was a Fox one
    /// with some stripped and some restored, resolved once at open (a save
    /// with one player stripped by accident would otherwise look up every
    /// other player's team every frame); `None` while it is closed.
    strip_again: Option<Vec<StripRow>>,
    /// The Set boots and gloves dialog's state while it is open (the Team
    /// menu's `…` item); `None` while it is closed.
    boglo: Option<menus::Boglo>,
    /// The Set all stats / Bump all stats dialog's state while it is open
    /// (the Player menu's `…` items, and Ctrl+B); `None` while it is
    /// closed.
    stat_num: Option<menus::StatNum>,
    /// The Bump dialog's last N used in this window (its field's
    /// prefill), starting at 0.
    bump: i32,
    /// The Copy/Swap stats dialog's state while it is open (the Player
    /// menu's `…` items); `None` while it is closed.
    copy_swap: Option<menus::CopySwap>,
    /// The Fix database confirmation is open (the Savefile menu's item);
    /// `false` while it is closed.
    fix_database: bool,
    /// The hide IDs the common settings give (`fpc_hide_boots_id` and
    /// `fpc_hide_gloves_id`), refreshed from `ctx.common()` each `Tool::view`
    /// frame: every FPC path (Hide/Show body, Ctrl+I, the classifier's
    /// reading, the Team menu's Set/Remove) reads them, and an out-of-band
    /// pair disables those paths behind the error's text.
    pub(crate) hide_ids: Result<fpc::HideIds, fpc::HideIdsError>,
    /// Where the Appearance tab's Randomize draws a motion value in
    /// `1..=top` from: a field so the tests pin the draws; `new` seeds a
    /// xorshift from the wall clock (the picks need only be unpredictable
    /// to a member, not cryptographic).
    randomize_draw: Box<dyn FnMut(u8) -> u8>,
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

/// One row of the Strip again window, resolved at open (the draft's row:
/// the player's team, his id and his savefile name, colour codes included).
#[derive(Clone)]
pub(crate) struct StripRow {
    /// The player's team's name (`None` when he sits on no roster).
    team: Option<String>,
    /// The player id.
    id: u32,
    /// His savefile name, colour codes included.
    name: String,
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
            tab: Tab::Abilities,
            tactics: team_widgets::tactics::TacticsView::default(),
            refusal: None,
            rules: None,
            cards: None,
            generation: 0,
            edit_aesthetics: false,
            strip_again: None,
            boglo: None,
            stat_num: None,
            bump: 0,
            copy_swap: None,
            fix_database: false,
            hide_ids: Ok(fpc::HideIds::CONVENTION),
            randomize_draw: Box::new(appearance::seeded_draw()),
        }
    }

    /// Opens the save at `path`: on success the selector reads `ALL`, the
    /// filter is empty and the first listed player is selected; a file the
    /// save refuses is shown as a refusal and the open session is untouched.
    /// A Fox save with some appearance ids stripped and some set opens the
    /// Strip again window over the ones still set (SE-GUI-20); the
    /// aesthetics lock is deliberately not reset (`edit_aesthetics`).
    pub(crate) fn open(&mut self, path: &Path) {
        match Session::open(path) {
            Ok(session) => {
                self.team = TeamChoice::All;
                self.filter.clear();
                self.selected = selection::rows(session.file(), &self.team, "")
                    .first()
                    .map(|row| (row.team_id, row.slot));
                self.strip_again = match session.version().engine() {
                    Engine::Fox => {
                        let rows: Vec<StripRow> = session
                            .restored_players()
                            .iter()
                            .map(|player| {
                                let team = session.file().teams().iter().find(|team| {
                                    team.roster.iter().any(|slot| slot.player_id == player.id)
                                });
                                StripRow {
                                    team: team.map(|team| team.name.clone()),
                                    id: player.id,
                                    name: player.name.clone(),
                                }
                            })
                            .collect();
                        (!rows.is_empty()).then_some(rows)
                    }
                    Engine::PreFox => None,
                };
                self.session = Some(session);
                self.generation += 1;
                self.tactics = team_widgets::tactics::TacticsView::default();
                self.refusal = None;
                self.boglo = None;
                self.stat_num = None;
                self.copy_swap = None;
                self.fix_database = false;
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
            tab_strip(ui, &mut view.tab);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match view.tab {
                    Tab::Abilities => abilities::show(ui, view),
                    Tab::Appearance => appearance::show(ui, view),
                    Tab::Team => team::show(ui, view),
                    Tab::Tactics => tactics::show(ui, view),
                });
        });
    }
    refusal_dialog(ui, view);
    strip_again_dialog(ui, view);
    menus::boglo_dialog(ui, view);
    menus::stat_num_dialog(ui, view);
    menus::copy_swap_dialog(ui, view);
    menus::fix_database_dialog(ui, view);
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
    // Ctrl+F is Clear visual flags: an aesthetics write, so a no-op under
    // the lock, and a no-op while a widget holds the keyboard (`typing`).
    let clear_flags_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::F));
    // Ctrl+I is Toggle FPC settings, the same lock as Ctrl+F; Ctrl+B
    // opens the Bump dialog, a gameplay write the lock does not stop.
    let fpc_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::I));
    let bump_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::B));
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
        menus::player_menu(ui, view);
        menus::team_menu(ui, view);
        menus::savefile_menu(ui, view);
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
    if clear_flags_key
        && view.edit_aesthetics
        && let Some((team_id, _, _)) = selected_player(view)
        && let Some(session) = view.session.as_mut()
    {
        session.clear_visual_flags(team_id);
    }
    if fpc_key
        && view.edit_aesthetics
        && let Ok(ids) = view.hide_ids
        && let Some((_, _, player_id)) = selected_player(view)
        && let Some(session) = view.session.as_mut()
    {
        session.toggle_fpc(player_id, ids);
    }
    if bump_key
        && let Some((_, _, player_id)) = selected_player(view)
        && let Some(session) = view.session.as_mut()
    {
        view.stat_num = Some(menus::StatNum::bump(session, player_id, view.bump));
    }
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
/// bullet): Abilities & Skills, Appearance, Team and Tactics pick.
fn tab_strip(ui: &mut egui::Ui, tab: &mut Tab) {
    ui.horizontal(|ui| {
        for (which, label) in [
            (Tab::Abilities, "Abilities & Skills"),
            (Tab::Appearance, "Appearance"),
            (Tab::Team, "Team"),
            (Tab::Tactics, "Tactics"),
        ] {
            if ui
                .add(egui::Button::selectable(*tab == which, label))
                .clicked()
            {
                *tab = which;
            }
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

/// `edit_player` for the team's entry (the info strip's Number and
/// Captain, the Team tab's fields).
pub(crate) fn edit_team(
    session: &mut Session,
    id: u32,
    key: u64,
    edit: impl FnOnce(&mut TeamEntry),
) {
    let mut edited = session.file().team(id).expect("the selected team").clone();
    edit(&mut edited);
    session.edit_team(key, edited);
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

/// The Strip again window (SE-GUI-20, the draft's `drawStripAgain`): while
/// `view.strip_again` holds rows, a modal lists them — team name, player id
/// and the name as the player list colours it — with Strip again (one
/// sealed undo entry, `Session::strip`) and Close.
fn strip_again_dialog(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some(rows) = view.strip_again.clone() else {
        return;
    };
    let mut close = false;
    let response =
        egui::Modal::new(egui::Id::new("save_editor_strip_again")).show(ui.ctx(), |ui| {
            ui.heading("Appearance ids set on a stripped save");
            ui.label(format!(
                "{} player(s) have an appearance id set while the rest are stripped:",
                rows.len()
            ));
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for row in &rows {
                        ui.horizontal(|ui| {
                            if let Some(team) = &row.team {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(team).monospace().weak())
                                        .truncate(),
                                );
                            }
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(row.id.to_string()).monospace(),
                                )
                                .truncate(),
                            );
                            let mut name = egui::text::LayoutJob::default();
                            append_name(&mut name, &row.name);
                            ui.label(name);
                        });
                    }
                });
            ui.horizontal(|ui| {
                if ui.button("Strip again").clicked()
                    && let Some(session) = view.session.as_mut()
                {
                    session.strip(&rows.iter().map(|row| row.id).collect::<Vec<_>>());
                    close = true;
                }
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        });
    if close || response.should_close() {
        view.strip_again = None;
    }
}

/// The format of a plain run (no code, after a reset or a non-hex code):
/// `PLACEHOLDER`, which the widget replaces with its own text color —
/// `TextFormat::default()`'s gray is not it.
pub(crate) fn plain_format() -> egui::TextFormat {
    egui::TextFormat::simple(egui::FontId::default(), egui::Color32::PLACEHOLDER)
}

/// Appends `name`'s runs to `job`, each colored as `name_spans` gives it
/// (`None` runs the normal text color): how the player list and the Strip
/// again window draw a savefile name. Sections' byte ranges are relative
/// to `job`'s own text, so the spans append, never merge another job's.
pub(crate) fn append_name(job: &mut egui::text::LayoutJob, name: &str) {
    for (color, text) in name_spans(name) {
        let format = match color {
            Some([red, green, blue]) => egui::TextFormat::simple(
                egui::FontId::default(),
                egui::Color32::from_rgb(red, green, blue),
            ),
            None => plain_format(),
        };
        job.append(text, 0.0, format);
    }
}
