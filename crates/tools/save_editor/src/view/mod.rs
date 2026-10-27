//! The Save editor's window (`plans/save_editor.md` "Editing UI"): a toolbar over a
//! master-detail layout, the player list left (`player_list`, its team and
//! player rows, the quick actions under it) and, right, the view the selected
//! row names: the team view (`team`) on a team row, the player view — the
//! title (`info_strip`), the three columns (`abilities` with `info_strip`'s
//! General info and Other stats) and the Appearance section (`appearance`)
//! — on a player row. Render-only over
//! the session's state: it opens a
//! save, filters and selects players, reorders a team's list by drag
//! (`Session::move_player`), edits the strip's fields through
//! `Session::edit_player`/`edit_team` (one undo entry per gesture), saves, and
//! shows a refusal in a modal.

mod aatf_check;
mod aatf_panel;
mod abilities;
mod appearance;
mod comparator;
mod generate;
mod info_strip;
mod live_aatf;
mod live_marks;
mod menus;
mod player_list;
mod portraits;
pub(crate) mod prompt;
mod quick_actions;
mod tactics;
mod team;
mod terms;
#[cfg(test)]
mod tests;
mod worker;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use pes_savefile::model::{PlayerEntry, TeamEntry};
use pes_version::{Engine, PesVersion};
use studio_core::{Notice, RunningPes};
use team_widgets::names::append_name;

use crate::session::Session;
use crate::session::selection::{self, TeamChoice};

/// The list row the selection names (`plans/save_editor.md` "Editing UI",
/// the Player list bullet): a team by id, or a player by team and roster
/// slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Selection {
    /// A team's own row (the detail shows the team view).
    Team(u32),
    /// A player row: the team he is rostered on and his roster slot.
    Player(u32, u8),
}

/// The window's state: the open save and what the master-detail shows of it.
pub(crate) struct SaveView {
    /// The open save, if one was opened.
    session: Option<Session>,
    /// Which team the list shows.
    team: TeamChoice,
    /// The filter box's text.
    filter: String,
    /// The selected row.
    selected: Option<Selection>,
    /// The team view's Tactics state (`team_widgets`' `TacticsView`:
    /// the preset its panels edit, the formation its lists show, the
    /// armed assignment mode, the open slot popover and Scroll names);
    /// back to preset 1 and kick-off on every open.
    pub(crate) tactics: team_widgets::tactics::TacticsView,
    /// The widget holding the keyboard at the last frame's end. A
    /// widget's gesture ends when it loses the keyboard (a click
    /// elsewhere, a Tab): sealing then keeps two gestures' writes from
    /// merging into one undo entry (the plan's "one entry per gesture").
    /// A `None` to `Some` move never seals: a widget can take focus as
    /// its gesture starts (a DragValue's click), and that gesture stays
    /// one step.
    focused: Option<egui::Id>,
    /// The unsaved-changes prompt's state while it is open (SE-GUI-05):
    /// the pending action and the table's rows, computed once at open.
    pub(crate) prompt: Option<prompt::Prompt>,
    /// The running-game question a Save first asks while the save's own
    /// PES runs: the version of the game found (SE-GUI-21's window rule).
    edit_mode_question: Option<PesVersion>,
    /// The Discard-on-close answer's ask to close again, drained by
    /// `Tool::view` into `ctx.request_close()` (the shell owns the send).
    pub(crate) request_close: bool,
    /// The toolbar's Open dialog (`rfd`, stubbed in tests — the
    /// `open_save_dialog` pattern).
    open_dialog: comparator::OpenSaveDialog,
    /// A refusal to show until dismissed (an open the save refused).
    refusal: Option<Refusal>,
    /// The ruleset the Cards left badge uses, loaded on the first frame and again only
    /// when the file named differs.
    rules: Option<ActiveRules>,
    /// The AATF panel's state (`plans/save_editor.md` "Configurable AATF
    /// rules", the AATF panel bullet); closed at start.
    aatf: aatf_panel::AatfPanel,
    /// The comparator panel's state (`plans/save_editor.md` "Comparator
    /// (gameplay + aesthetics)"); closed at start.
    comparator: comparator::Comparator,
    /// The comparator's Pick dialog (`rfd`, stubbed in tests — the AATF
    /// panel's copy dialog pattern).
    open_save_dialog: comparator::OpenSaveDialog,
    /// The Generate dialog's state while it is open (the Player menu's
    /// `settings.toml` and in-game face items); `None` while it is closed.
    generate: Option<generate::Generate>,
    /// The Generate dialog's Pick-a-folder dialog (`rfd`, stubbed in
    /// tests — the same pattern as `open_save_dialog`).
    pick_folder_dialog: generate::PickFolderDialog,
    /// The status-bar notice the last action raised; `Tool::view` forwards
    /// it to the shell's `notify` after the frame.
    pub(crate) notice: Option<Notice>,
    /// The Live AATF toolbar toggle's state: `Tool::view` refreshes it
    /// from the `live_aatf` setting and writes the setting back when the
    /// toggle changes it; off in the tests' view-only frames.
    pub(crate) live_aatf: bool,
    /// The Live AATF dock's state (the Live AATF bullet's docked panel);
    /// drawn while `live_aatf` and a save is open.
    live: live_aatf::LiveAatf,
    /// The panel's Save a copy dialog (`rfd`, stubbed in tests — the
    /// Ruleset editor's picker pattern).
    aatf_copy_dialog: aatf_panel::SaveFileDialog,
    /// The comparator's Export dialog (`rfd`, stubbed in tests — the same
    /// dialog type as the AATF panel's).
    export_dialog: aatf_panel::SaveFileDialog,
    /// The "Cards left" badge as last computed and the `BadgeKey` it was
    /// computed for: `player_tier` then `card_limits` run again only when
    /// the key differs, not on every frame. `None` while the rules file
    /// does not load (the quick actions' red line says so; no badge is
    /// drawn then).
    cards: Option<(BadgeKey, OwnBadge)>,
    /// The pitch badges' medal letters per roster slot as last computed
    /// and the `MedalKey` they were computed for: `player_tier` runs
    /// again only when the key differs, not on every frame. A rules
    /// file that does not load is kept as a row of `None`s under the
    /// key, so no letter is drawn.
    medals: Option<(MedalKey, Vec<Option<team_widgets::tactics::Medal>>)>,
    /// The Tactics pitch's portraits, read from the selected game's
    /// `download/` CPKs off the UI thread (`select_portraits` points it at
    /// the PES folder each frame).
    portraits: portraits::Portraits,
    /// Which open this is: part of the pushed widget id, so egui widget
    /// state (a text field's undo history, a control's in-progress edit)
    /// never crosses into the next save's same player id.
    generation: u64,
    /// "Aesthetics from exports" (`plans/save_editor.md` "Read-only
    /// aesthetics"), refreshed from the setting each `Tool::view` frame as
    /// `live_aatf` is; off in the tests' view-only frames, which set it by
    /// hand as `edit_aesthetics`'s GIVEN.
    pub(crate) aesthetics_from_exports: bool,
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
    /// The quick actions' Set stats number (the draft's `set-stats`
    /// field), starting at 77.
    set_stats: u8,
    /// The Copy/Swap stats dialog's state while it is open (the Player
    /// menu's `…` items); `None` while it is closed.
    copy_swap: Option<menus::CopySwap>,
    /// The Fix database confirmation is open (the Savefile menu's item);
    /// `false` while it is closed.
    fix_database: bool,
    /// The Kit slot assignments dialog's team while it is open (the team
    /// view's `Kit slots…` button); `None` while it is closed.
    kit_slots: Option<u32>,
    /// The hide IDs the common settings give (`fpc_hide_boots_id` and
    /// `fpc_hide_gloves_id`), refreshed from `ctx.common()` each `Tool::view`
    /// frame: every FPC path (Hide/Show default body, Ctrl+I, the classifier's
    /// reading, the Team menu's Set/Remove) reads them, and an out-of-band
    /// pair disables those paths behind the error's text.
    pub(crate) hide_ids: Result<fpc::HideIds, fpc::HideIdsError>,
    /// Where the Appearance section's Randomize draws a motion value in
    /// `1..=top` from: a field so the tests pin the draws; `new` seeds a
    /// xorshift from the wall clock (the picks need only be unpredictable
    /// to a member, not cryptographic).
    randomize_draw: Box<dyn FnMut(u8) -> u8>,
}

/// The ruleset the Skills column's Cards left badge uses (`plans/save_editor.md`
/// "Editing UI", the Cards left bullet): the file it was loaded from and
/// the outcome as the badge's row draws it.
pub(crate) struct ActiveRules {
    /// The file `rules` was loaded from (`None`: the embedded official ruleset).
    file: Option<PathBuf>,
    /// The loaded rules or the load error's text (a file that does not
    /// load draws no badge; the quick actions' red line reports it). The
    /// `Arc` lets the AATF panel's worker check teams on the same loaded
    /// file the badge reads.
    rules: Result<Arc<aatf::Rules>, String>,
}

/// What the Skills column's Cards left badge was computed for: a change of
/// the open save (`Session::revision`), of the selection or of the rules
/// file recomputes it; anything else keeps a frame's `player_tier` and
/// `card_limits` cost off.
#[derive(PartialEq)]
pub(crate) struct BadgeKey {
    /// `Session::revision` when computed.
    revision: u64,
    /// The selected team.
    team: u32,
    /// The selected roster slot (0-based).
    slot: u8,
    /// The rules file the badge's ruleset came from (`None`: the official
    /// ruleset).
    rules_file: Option<PathBuf>,
}

/// What the pitch badges' medal letters were computed for: the Cards
/// left badge's `BadgeKey` minus the selection — a change of the open
/// save (`Session::revision`), of the team or of the rules file
/// recomputes them, and any other frame draws the kept ones.
#[derive(PartialEq)]
pub(crate) struct MedalKey {
    /// `Session::revision` when computed.
    revision: u64,
    /// The team the medals are for.
    team: u32,
    /// The rules file the medals' ruleset came from (`None`: the official
    /// ruleset).
    rules_file: Option<PathBuf>,
}

/// The badge `cards` holds for a `BadgeKey`, owned: what the rules read the
/// selected player as.
pub(crate) enum OwnBadge {
    /// The tier `Rules::player_tier` reads him as: its display label and
    /// his card economy under it.
    Tier {
        label: String,
        limits: aatf::CardLimits,
    },
    /// No tier's condition holds for him (`check_team`'s `NoTier`).
    NoTier,
    /// `player_tier` or `card_limits` failed (or the team could not be
    /// read), with the error's text: the row draws the red click line
    /// `TIER_CHECK_FAILED_LINE` in the badge's place, opening the error
    /// (a file that does not load draws nothing there — the quick
    /// actions' red line reports it). The AATF check never calls
    /// these two functions, so a member's logic edit that breaks only them
    /// would otherwise show nowhere. Kept under its key like the others, so
    /// the failing script does not run again on every frame.
    Failed(String),
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
            tactics: team_widgets::tactics::TacticsView::default(),
            focused: None,
            prompt: None,
            edit_mode_question: None,
            request_close: false,
            open_dialog: comparator::default_open_dialog(),
            refusal: None,
            rules: None,
            aatf: aatf_panel::AatfPanel::new(),
            comparator: comparator::Comparator::new(),
            open_save_dialog: comparator::default_open_dialog(),
            generate: None,
            pick_folder_dialog: generate::default_pick_folder(),
            notice: None,
            live_aatf: false,
            live: live_aatf::LiveAatf::new(),
            aatf_copy_dialog: aatf_panel::default_copy_dialog(),
            export_dialog: comparator::default_export_dialog(),
            cards: None,
            medals: None,
            portraits: portraits::Portraits::new(),
            generation: 0,
            aesthetics_from_exports: false,
            edit_aesthetics: false,
            strip_again: None,
            boglo: None,
            stat_num: None,
            bump: 0,
            set_stats: 77,
            copy_swap: None,
            fix_database: false,
            kit_slots: None,
            hide_ids: Ok(fpc::HideIds::CONVENTION),
            randomize_draw: Box::new(appearance::seeded_draw()),
        }
    }

    /// Points the pitch's portrait source at `pes_folder` for `version`: `Tool::view` calls it
    /// each frame with the sidebar's PES and the PES folder setting, as it refreshes `hide_ids`.
    pub(crate) fn select_portraits(
        &mut self,
        ctx: &egui::Context,
        pes_folder: &Path,
        version: PesVersion,
    ) {
        self.portraits.select(ctx, pes_folder, version);
    }

    /// The aesthetics lock (the section's lock bullet): on while the
    /// "Aesthetics from exports" setting is and the session's "Edit
    /// aesthetics anyway" switch is not. The one function the locked
    /// paths' guards read, so no site can forget the setting.
    pub(crate) fn aesthetics_locked(&self) -> bool {
        self.aesthetics_from_exports && !self.edit_aesthetics
    }

    /// Whether the open session holds unsaved changes (`Tool::hold_close`'s
    /// ask; `false` with none open).
    pub(crate) fn is_dirty(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.is_dirty())
    }

    /// The open save's path (`Tool::title_detail`'s, for the idle window
    /// title); `None` with none open.
    pub(crate) fn save_path(&self) -> Option<&Path> {
        self.session.as_ref().map(Session::path)
    }

    /// Opens the save at `path`: on success the selector reads `ALL`, the
    /// filter is empty and the first player row is selected (a team's own
    /// row is not a player); a file the
    /// save refuses is shown as a refusal and the open session is untouched.
    /// A Fox save with some appearance ids stripped and some set opens the
    /// Strip again window over the ones still set (SE-GUI-20); the
    /// aesthetics lock is deliberately not reset (`edit_aesthetics`).
    pub(crate) fn open(&mut self, path: &Path) {
        match Session::open(path) {
            Ok(session) => {
                self.team = TeamChoice::All;
                self.filter.clear();
                self.selected = selection::list(session.file(), &self.team, "")
                    .into_iter()
                    .find_map(|row| match row {
                        selection::ListRow::Player(row) => {
                            Some(Selection::Player(row.team_id, row.slot))
                        }
                        selection::ListRow::Team(_) => None,
                    });
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
                self.comparator.reset(session.version());
                self.session = Some(session);
                self.generation += 1;
                // A view setting, not the save's: `scroll_names` is the
                // member's, so it survives the open (the rest is the
                // save's: back to preset 1 and kick-off).
                let scroll_names = self.tactics.scroll_names;
                self.tactics = team_widgets::tactics::TacticsView::default();
                self.tactics.scroll_names = scroll_names;
                self.prompt = None;
                self.edit_mode_question = None;
                self.refusal = None;
                self.boglo = None;
                self.stat_num = None;
                self.copy_swap = None;
                self.generate = None;
                self.fix_database = false;
                self.kit_slots = None;
                self.aatf.reset();
                self.live.check.reset();
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
    set_surrender_on_press(ui.ctx());
    let file = rules_file.map(Path::to_path_buf);
    if view.rules.as_ref().is_none_or(|active| active.file != file) {
        view.rules = Some(ActiveRules {
            rules: crate::session::rules::load_rules(file.as_deref())
                .map(Arc::new)
                .map_err(|error| format!("{error:#}")),
            file,
        });
    }
    toolbar(ui, view, running);
    if view.session.is_none() {
        ui.weak("No save is open.");
    } else {
        // Before the left panel, so the dock spans the view's width under
        // the player list and the detail view (egui lays a panel out
        // against the space the earlier panels left).
        if view.live_aatf {
            live_aatf::show(ui, view);
        }
        let list_panel = egui::Panel::left("save_editor_players")
            // The draft's `.plist` width: 240 points, not resizable (the
            // plan's Player list bullet; the window scales instead of
            // narrowing under the design width). No bottom margin: the
            // quick actions' band is the panel's content's last item and
            // runs to the pane's bottom ("The view fills its pane").
            .resizable(false)
            .default_size(240.0)
            .frame(
                egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin {
                    left: 8,
                    right: 8,
                    top: 2,
                    bottom: 0,
                }),
            )
            .show(ui, |ui| player_list::show(ui, view));
        let list_right = list_panel.response.rect.max.x;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match view.selected {
                    Some(Selection::Team(_)) => team::show(ui, view),
                    Some(Selection::Player(..)) | None => {
                        // The player view: the title, the three columns,
                        // then the Appearance section, closed by default —
                        // the whole centred between the player list's right
                        // edge and the window's (`columns_width` is the last
                        // frame's measured width; a first frame has none
                        // and draws left-aligned once). The pad is the
                        // even share less the panel border already between
                        // the list's edge and the pane's content.
                        let width = abilities::columns_width(ui);
                        let window_right = ui.ctx().content_rect().max.x;
                        let gap = ((window_right - list_right - width) / 2.0).max(0.0);
                        let pad = (gap - (ui.cursor().min.x - list_right)).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_space(pad);
                            ui.vertical(|ui| {
                                // The stored total, or the pane's when it
                                // is smaller (the columns wrap inside).
                                ui.set_width(width.min(ui.available_width()));
                                info_strip::show(ui, view);
                                abilities::show(ui, view);
                                egui::CollapsingHeader::new("Appearance")
                                    .id_salt("appearance")
                                    // The section's content keeps the
                                    // view's width (the grids are sized
                                    // for it, the body's indent would push
                                    // them past it).
                                    .show_unindented(ui, |ui| appearance::show(ui, view));
                            });
                        });
                    }
                });
        });
    }
    strip_again_dialog(ui, view);
    team::kit_slots_dialog(ui, view);
    menus::boglo_dialog(ui, view);
    menus::stat_num_dialog(ui, view);
    menus::copy_swap_dialog(ui, view);
    menus::fix_database_dialog(ui, view);
    generate::dialog(ui, view);
    aatf_panel::show(ui, view);
    comparator::show(ui, view);
    prompt::show(ui, view, running);
    prompt::edit_mode_dialog(ui, view, running);
    // Last so the newest refusal, raised by a modal's own Save, sits
    // above the modal that stays up behind it.
    refusal_dialog(ui, view);
    // One undo entry per gesture: seal when the pointer is up and no
    // widget holds the keyboard, and when the widget that held it ends
    // the frame without it — a gesture ends when its widget loses the
    // keyboard. A `None` to `Some` move never seals (the `focused` doc).
    let (focused, settled) = edit_settled(ui, view.focused);
    if settled && let Some(session) = view.session.as_mut() {
        session.seal();
    }
    view.focused = focused;
}

/// The widget holding the keyboard this frame and whether the edit has
/// settled: the pointer up with no widget focused, or the widget that
/// held it having lost it. `show`'s seal condition, which the AATF
/// checks' start shares, so a check never runs over a mid-gesture file.
fn edit_settled(ui: &egui::Ui, last_focused: Option<egui::Id>) -> (Option<egui::Id>, bool) {
    let focused = ui.ctx().memory(|memory| memory.focused());
    let settled = (!ui.input(|input| input.pointer.any_down()) && focused.is_none())
        || (last_focused.is_some() && last_focused != focused);
    (focused, settled)
}

/// Whatever holds the keyboard gives it up. A run's start waits for
/// `edit_settled`, which a focus kept by the field or the button would
/// hold off, and a button activated without a pointer press (Tab + Space,
/// or AccessKit's Click — a `FAKE_PRIMARY_CLICKED` that moves no focus)
/// leaves the keyboard focus where it was.
pub(super) fn surrender_focus(ui: &egui::Ui) {
    if let Some(id) = ui.ctx().memory(|memory| memory.focused()) {
        ui.ctx().memory_mut(|memory| memory.surrender_focus(id));
    }
}

/// The editor's focus mode: a press on another widget ends the focused
/// widget's keyboard gesture on the press frame (egui's own default,
/// `Clicks`, waits for a completed click, so a drag elsewhere would keep
/// the field focused — its undo entries would merge and Ctrl+Z would
/// stay swallowed while `typing` holds). The option is the context's, so
/// once the editor has drawn it holds for every view, which is the
/// behaviour wanted there too; it moves to the shell's context setup
/// when the shell gets one (worklog 8.1).
fn set_surrender_on_press(ctx: &egui::Context) {
    ctx.memory_mut(|memory| {
        memory.options.input_options.surrender_focus_on = egui::SurrenderFocusOn::Presses;
    });
}

/// The toolbar: Open, Save, Undo, Redo and the unsaved-changes marker (the
/// draft's toolbar row). The open save's path rides the idle window title
/// (`Tool::title_detail`, gui.md "Window title"), not this row.
fn toolbar(ui: &mut egui::Ui, view: &mut SaveView, running: &dyn Fn() -> Vec<RunningPes>) {
    // The unsaved prompt and the running-game question hold every
    // shortcut dead while up (the prompt's own buttons are the way out).
    let modal_up = view.prompt.is_some() || view.edit_mode_question.is_some();
    // Ctrl+S saves even while a widget holds the keyboard; the other
    // shortcuts do nothing then (a text field's own undo is egui's).
    let save_key =
        !modal_up && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::S));
    let typing = ui.ctx().egui_wants_keyboard_input() || modal_up;
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
    // Ctrl+F is Clear visual flags on the selected row's team (a player's
    // or a team row's own): an aesthetics write, so a no-op under the
    // lock, and a no-op while a widget holds the keyboard (`typing`).
    let clear_flags_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::F));
    // Ctrl+I is Toggle FPC settings, the same lock as Ctrl+F; Ctrl+B
    // opens the Bump dialog, a gameplay write the lock does not stop.
    let fpc_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::I));
    let bump_key =
        !typing && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::B));
    ui.horizontal(|ui| {
        if ui.button("Open…").clicked() {
            // A dirty session prompts first (SE-GUI-05): the picker waits
            // for the answer.
            if view.is_dirty() {
                prompt::open(view, prompt::Pending::OpenPicker);
            } else if let Some(path) = (view.open_dialog)() {
                view.open(&path);
            }
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
        menus::panels_menu(ui, view);
        // The Live AATF toggle as a framed button (the plan's Live AATF
        // bullet): the play sign (`▶`) then the name — a check mark
        // beside it read as on while it was off — the selection fill
        // while on; a selectable label read as a label beside the menus.
        if ui
            .add(egui::Button::new("\u{25b6} Live AATF").selected(view.live_aatf))
            .on_hover_text(
                "Check the open team after every edit, in a panel at the bottom (the setting is \
                 remembered)",
            )
            .clicked()
        {
            view.live_aatf = !view.live_aatf;
        }
        // The save's path is in the window title (`title_detail`): beside the
        // unsaved mark it could wrap the toolbar onto a second row.
        if view.is_dirty() {
            ui.weak("\u{2022} unsaved changes");
        }
    });
    if clear_flags_key
        && !view.aesthetics_locked()
        && let Some(team_id) = selected_team_id(view)
        && let Some(session) = view.session.as_mut()
    {
        session.clear_visual_flags(team_id);
    }
    if fpc_key
        && !view.aesthetics_locked()
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

/// The window's Save (the toolbar's, Ctrl+S's and the prompt's): the
/// running-game question first while the save's own PES runs, then the
/// write. Answers whether the file was written — `false` while the
/// question waits or a write failed, so a prompt stays up.
pub(super) fn save(view: &mut SaveView, running: &dyn Fn() -> Vec<RunningPes>) -> bool {
    let Some(version) = view.session.as_ref().map(|session| session.version()) else {
        return false;
    };
    if running().iter().any(|game| game.version == version) {
        view.edit_mode_question = Some(version);
        return false;
    }
    write_now(view, running, false)
}

/// `session.save` and the refusal its error takes (the draft's
/// `saveFile`). `edit_mode_closed` is the member's answer the caller
/// holds: the toolbar's Save has none (the own-game check ran just
/// above), the question's Save has the confirmed `true`.
pub(super) fn write_now(
    view: &mut SaveView,
    running: &dyn Fn() -> Vec<RunningPes>,
    edit_mode_closed: bool,
) -> bool {
    let Some(session) = view.session.as_mut() else {
        return false;
    };
    match session.save(&running(), edit_mode_closed) {
        Ok(()) => {
            view.refusal = None;
            true
        }
        Err(error) => {
            view.refusal = Some(Refusal {
                title: "Save refused",
                message: format!("{error:#}"),
            });
            false
        }
    }
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

/// `edit_player` for the team's entry (General info's Shirt number and
/// Captain, the team view's fields, the Tactics editor's edited clone).
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

/// The line drawn where the ruleset's controls would stand when the active
/// rules file did not load: the AATF panel's list and the Live AATF dock
/// share the one wording.
pub(crate) fn rules_load_error(error: &str) -> String {
    format!("The AATF rules file could not be loaded: {error}")
}

/// The selected row's team id — a player row's team or a team row's own;
/// `None` with no row selected.
pub(crate) fn selected_team_id(view: &SaveView) -> Option<u32> {
    match view.selected? {
        Selection::Team(id) | Selection::Player(id, _) => Some(id),
    }
}

/// The selected player's `(team_id, roster slot, player_id)`; `None` when
/// the selection names no rostered player (a team row, a stale slot, an
/// empty save's list).
pub(crate) fn selected_player(view: &SaveView) -> Option<(u32, u8, u32)> {
    let Selection::Player(team_id, slot) = view.selected? else {
        return None;
    };
    let player_id = view.session.as_ref()?.slot_player(team_id, slot)?;
    Some((team_id, slot, player_id))
}

/// The selected row's `(team_id, save's version, team's entry cloned)`:
/// the team the row names — a player row's team or a team row's own;
/// `None` when the selection names no team or the entry is missing (the
/// team widgets' shared head).
pub(crate) fn selected_team(view: &SaveView) -> Option<(u32, PesVersion, TeamEntry)> {
    let team_id = match view.selected? {
        Selection::Team(team_id) | Selection::Player(team_id, _) => team_id,
    };
    let file = view.session.as_ref()?.file();
    let team = file.team(team_id)?.clone();
    Some((team_id, file.version(), team))
}

/// The refusal's dialog, while `view.refusal` holds one.
fn refusal_dialog(ui: &mut egui::Ui, view: &mut SaveView) {
    let mut close = false;
    if let Some(refusal) = view.refusal.as_ref() {
        let response =
            egui::Modal::new(egui::Id::unique("save_editor_refusal")).show(ui.ctx(), |ui| {
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
        egui::Modal::new(egui::Id::unique("save_editor_strip_again")).show(ui.ctx(), |ui| {
            ui.heading("Appearance ids set on a stripped save");
            ui.label(format!(
                "{} {} an appearance id set while the rest are stripped:",
                rows.len(),
                if rows.len() == 1 {
                    "player has"
                } else {
                    "players have"
                }
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
