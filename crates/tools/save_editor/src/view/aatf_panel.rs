//! The floating AATF panel (`plans/save_editor.md` "Configurable AATF
//! rules", the AATF panel bullet; the draft's `aatfPanel`): an
//! `egui::Window` listing the active ruleset's violations for the current
//! team or a picked set, grouped per player through `team_widgets`'
//! `violations_list`, a row's click selecting the player. The check runs
//! on a worker thread — a whole-save pick is seconds of script — started
//! when the session's revision, the team ids or the rules file change and
//! the edit has settled (`edit_settled`, the same condition the undo
//! stack's seal uses); until the reply lands the last result stays,
//! marked `Checking…`. Validate rules reports the lib's load-time
//! validation of the active file; Save a copy writes the embedded
//! official file out.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use crossbeam_channel::Receiver;
use pes_savefile::EditFile;
use pes_savefile::model::PlayerEntry;
use team_widgets::violations::{ListedPlayer, SelectedPlayer, TeamViolations, violations_list};

use crate::session::rules::{self, TeamCheck};
use crate::session::selection::TeamChoice;
use crate::view::{ActiveRules, Refusal, SaveView, edit_settled, rules_load_error};

/// Which teams the panel checks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AatfMode {
    /// The selector's team (with ALL, the selected player's).
    Current,
    /// The teams ticked in the panel's list.
    Select,
}

/// The Save a copy dialog: the suggested file name to the picked path
/// (`rfd` in the window, a stub in the tests — a native dialog cannot be
/// driven headless; the Ruleset editor's `SaveDialog` pattern).
pub(crate) type SaveCopyDialog = Box<dyn FnMut(&str) -> Option<PathBuf>>;

/// The `rfd` Save dialog (the Ruleset editor's `default_save_dialog`
/// pattern): the suggested name comes from the caller.
#[cfg(not(test))]
pub(crate) fn default_copy_dialog() -> SaveCopyDialog {
    Box::new(|name| {
        rfd::FileDialog::new()
            .add_filter("AATF rules", &["rhai"])
            .set_file_name(name)
            .save_file()
    })
}

/// `default_copy_dialog` under `cargo test`: a test reaching a real file
/// dialog panics; tests stub `view.aatf_copy_dialog`.
#[cfg(test)]
pub(crate) fn default_copy_dialog() -> SaveCopyDialog {
    Box::new(|_| panic!("a test reached a real file dialog: stub aatf_copy_dialog"))
}

/// What a check ran for: a change of the open save, of the ids or of the
/// rules file recomputes it.
#[derive(Clone, PartialEq)]
pub(super) struct CheckKey {
    /// `Session::revision` when the views were built.
    pub(super) revision: u64,
    /// The teams checked, in id order.
    pub(super) ids: Vec<u32>,
    /// The rules file the check's ruleset came from (`None`: the embedded
    /// official ruleset).
    pub(super) rules_file: Option<PathBuf>,
}

/// A check in flight: the key it runs for and its reply channel.
pub(super) struct PendingCheck {
    /// The key the worker's `check_teams` runs for.
    pub(super) key: CheckKey,
    /// The worker's reply: its key and the checks, in `ids`' order.
    pub(super) replies: Receiver<(CheckKey, Vec<TeamCheck>)>,
}

/// The panel's state on `SaveView` (`view.aatf`): open or not, the mode,
/// the picked ids, the suggestions toggle, the last result with its key,
/// the pending check's receiver, the last validation and the copy's path.
pub(super) struct AatfPanel {
    /// Whether the window is drawn (the Panels menu's item and the quick
    /// actions' `AATF…` button open it).
    pub(super) open: bool,
    /// Current team or Select teams.
    pub(super) mode: AatfMode,
    /// The ids ticked in Select mode.
    pub(super) picked: BTreeSet<u32>,
    /// The Show suggestions checkbox (off).
    pub(super) show_suggestions: bool,
    /// The last check's key and its per-team results.
    pub(super) result: Option<(CheckKey, Vec<TeamCheck>)>,
    /// The check in flight, if one is.
    pub(super) pending: Option<PendingCheck>,
    /// The last Validate rules answer: the file's label and its finding
    /// lines.
    validation: Option<(String, Vec<String>)>,
    /// The copy's written path, for the `Wrote <path>` line.
    wrote: Option<PathBuf>,
}

impl AatfPanel {
    /// The panel closed, on Current team.
    pub(super) fn new() -> AatfPanel {
        AatfPanel {
            open: false,
            mode: AatfMode::Current,
            picked: BTreeSet::new(),
            show_suggestions: false,
            result: None,
            pending: None,
            validation: None,
            wrote: None,
        }
    }

    /// Another save opened (`SaveView::open`): the picks, results,
    /// pending check, validation and copy are of the old save's teams;
    /// open, the mode and Show suggestions are the member's view choices
    /// and stay.
    pub(super) fn reset(&mut self) {
        self.picked.clear();
        self.result = None;
        self.pending = None;
        self.validation = None;
        self.wrote = None;
    }
}

/// The window for one frame, while `view.aatf.open`. The reply is polled
/// before a new check is decided on, so the frame that starts a check
/// draws `Checking…` over the previous result. A closed panel starts no
/// check.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let SaveView {
        aatf,
        session,
        team,
        filter,
        selected,
        rules,
        refusal,
        aatf_copy_dialog,
        focused,
        ..
    } = view;
    if !aatf.open {
        return;
    }
    let reply = aatf
        .pending
        .as_ref()
        .map(|pending| pending.replies.try_recv());
    match reply {
        Some(Ok((key, checks))) => {
            aatf.result = Some((key, checks));
            aatf.pending = None;
        }
        // A worker that died without replying leaves no result to wait
        // for: drop the pending so the spawn below starts a fresh check.
        Some(Err(crossbeam_channel::TryRecvError::Disconnected)) => {
            log::warn!("the AATF check's worker ended without a reply");
            aatf.pending = None;
        }
        _ => {}
    }
    let (Some(session), Some(active)) = (session.as_ref(), rules.as_ref()) else {
        return;
    };
    let file = session.file();
    let (_, settled) = edit_settled(ui, *focused);
    // The teams to check: the selector's, or with ALL the selected
    // player's (the team the strip and tabs are editing); in Select mode
    // the ticked ids, already in id order.
    let mut no_teams = None;
    let ids: Vec<u32> = match aatf.mode {
        AatfMode::Current => match *team {
            TeamChoice::Team(id) => vec![id],
            TeamChoice::All => match *selected {
                Some((team_id, _)) => vec![team_id],
                None => {
                    no_teams = Some("No team open.");
                    Vec::new()
                }
            },
        },
        AatfMode::Select => {
            if aatf.picked.is_empty() {
                no_teams = Some("No team picked.");
            }
            aatf.picked.iter().copied().collect()
        }
    };
    let (loaded, load_error) = match &active.rules {
        Ok(rules) => (Some(Arc::clone(rules)), None),
        Err(error) => (None, Some(error.clone())),
    };
    let key = CheckKey {
        revision: session.revision(),
        ids,
        rules_file: active.file.clone(),
    };
    // A check is due when the key moved on since the last result (a rules
    // file that does not load never starts one: its error is the list).
    let due = !key.ids.is_empty()
        && loaded.is_some()
        && aatf.result.as_ref().is_none_or(|(held, _)| *held != key);
    let pending_for_key = aatf
        .pending
        .as_ref()
        .is_some_and(|pending| pending.key == key);
    if due
        && settled
        && !pending_for_key
        && let Some(rules) = &loaded
    {
        aatf.pending = Some(spawn_check(ui.ctx(), file, rules, key.clone()));
    }
    // The ruleset's name on a loaded file, else the file's path: only a
    // rules file fails to load (`Rules::official` panics rather than err).
    let ruleset_label = match &active.rules {
        Ok(rules) => rules.ruleset().name.clone(),
        Err(_) => active
            .file
            .as_ref()
            .expect("only a rules file fails to load")
            .display()
            .to_string(),
    };
    let mut open = aatf.open;
    egui::Window::new(format!("AATF violations ({ruleset_label})"))
        .id(egui::Id::unique("save_editor_aatf_panel"))
        .default_width(620.0)
        .collapsible(false)
        .resizable(true)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            rows(ui, aatf, active, aatf_copy_dialog, refusal, file);
            // Above the scroll, so a member scrolled deep in the list
            // still sees that the rows are the previous result's.
            if aatf.pending.is_some() || due {
                ui.weak("Checking\u{2026}");
            }
            // The results fill the rest of the window in their own
            // scroll: a whole-save pick, or one team's suggestions, runs
            // taller than it. The salt keeps its state apart from the
            // picker's: both default to this `Ui`'s one `scroll_area`
            // id, and the shorter area would clamp the other's offset.
            let clicked = egui::ScrollArea::vertical()
                .id_salt("save_editor_aatf_results")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut clicked = None;
                    if let Some(line) = no_teams {
                        ui.weak(line);
                    } else if let Some(error) = &load_error {
                        ui.weak(rules_load_error(error));
                    } else if let Some((result_key, checks)) = &aatf.result {
                        clicked = results(ui, file, result_key, checks, aatf.show_suggestions);
                    }
                    clicked
                })
                .inner;
            if let Some(player) = clicked {
                *selected = Some((player.team_id, player.slot));
                if *team != TeamChoice::All {
                    *team = TeamChoice::Team(player.team_id);
                }
                // As `open` clears it: the row the click selects would
                // be hidden by a filter matching nothing.
                filter.clear();
            }
        });
    aatf.open = open;
}

/// The widget rows above the results: the ruleset label, the mode
/// radios, the suggestions toggle and Save a copy (row 1), Validate
/// rules (row 2), then the last validation and `Wrote` lines, and in
/// Select mode the team checkboxes in the selector's order.
fn rows(
    ui: &mut egui::Ui,
    aatf: &mut AatfPanel,
    active: &ActiveRules,
    copy_dialog: &mut SaveCopyDialog,
    refusal: &mut Option<Refusal>,
    file: &EditFile,
) {
    ui.horizontal_wrapped(|ui| {
        match &active.rules {
            Ok(rules) => ui.weak(format!("Ruleset {} ", rules.ruleset().name)),
            Err(_) => ui.weak("Ruleset "),
        };
        ui.radio_value(&mut aatf.mode, AatfMode::Current, "Current team");
        ui.radio_value(&mut aatf.mode, AatfMode::Select, "Select teams");
        ui.checkbox(&mut aatf.show_suggestions, "Show suggestions");
        if ui
            .button("Save a copy")
            .on_hover_text("Write the built-in official ruleset to a file")
            .clicked()
            && let Some(path) = copy_dialog(aatf::OFFICIAL_FILE_NAME)
        {
            // As `write_rosters_tsv` writes its file: the error names the path.
            match std::fs::write(&path, aatf::official_file())
                .with_context(|| path.display().to_string())
            {
                Ok(()) => aatf.wrote = Some(path),
                Err(error) => {
                    *refusal = Some(Refusal {
                        title: "Cannot save the copy",
                        message: format!("{error:#}"),
                    });
                }
            }
        }
    });
    if ui
        .button("Validate rules")
        .on_hover_text("Check the rules file in use for errors, each with its line or setting")
        .clicked()
    {
        let name = active
            .file
            .as_ref()
            .map(|file| file.display().to_string())
            .unwrap_or_else(|| "the official ruleset".to_string());
        aatf.validation = Some((name, rules::validate(active.file.as_deref())));
    }
    if let Some((name, findings)) = &aatf.validation {
        if findings.is_empty() {
            ui.weak(format!("{name}: no findings"));
        } else {
            ui.weak(format!("{name}:"));
            for line in findings {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    egui::RichText::new(line).monospace(),
                );
            }
        }
    }
    if let Some(path) = &aatf.wrote {
        ui.weak(format!("Wrote {}", path.display()));
    }
    if aatf.mode == AatfMode::Select {
        egui::ScrollArea::vertical()
            .id_salt("save_editor_aatf_picker")
            .max_height(180.0)
            .show(ui, |ui| {
                for entry in file.teams() {
                    let mut on = aatf.picked.contains(&entry.id);
                    if ui
                        .checkbox(&mut on, format!("{} {}", entry.id, entry.name))
                        .changed()
                    {
                        if on {
                            aatf.picked.insert(entry.id);
                        } else {
                            aatf.picked.remove(&entry.id);
                        }
                    }
                }
            });
    }
}

/// The check's results: one error-coloured line per team whose check
/// failed (as `check-aatf` reports it), `No violations.` when no team
/// carries an error or a warning, then the `violations_list` itself.
/// Returns the player whose row was clicked this frame.
fn results(
    ui: &mut egui::Ui,
    file: &EditFile,
    key: &CheckKey,
    checks: &[TeamCheck],
    show_suggestions: bool,
) -> Option<SelectedPlayer> {
    // Player ids are stable inside an open file: one map serves the
    // whole roster scan, as `selection::rows` builds its own (its
    // construction lives there inline).
    let players: HashMap<u32, &PlayerEntry> = file
        .players()
        .iter()
        .map(|player| (player.id, player))
        .collect();
    // One `ListedPlayer` list per checked team, for the group headers;
    // `teams` borrows it below, so it is filled first.
    let mut listed: Vec<Vec<ListedPlayer>> = Vec::with_capacity(key.ids.len());
    for (id, check) in key.ids.iter().zip(checks) {
        listed.push(if check.is_ok() {
            listed_players(file, &players, *id)
        } else {
            Vec::new()
        });
    }
    let mut teams = Vec::new();
    for (i, (id, check)) in key.ids.iter().zip(checks).enumerate() {
        let name = file.team(*id).map(|team| team.name.as_str()).unwrap_or("?");
        match check {
            Err(error) => {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("Team {id} {name}: the AATF check failed: {error}"),
                );
            }
            Ok(violations) => teams.push(TeamViolations {
                team_id: *id,
                team_name: name,
                players: &listed[i],
                violations,
            }),
        }
    }
    let clean = checks
        .iter()
        .filter_map(|check| check.as_ref().ok())
        .flatten()
        .all(|violation| {
            !matches!(
                violation.severity,
                aatf::Severity::Error | aatf::Severity::Warning
            )
        });
    // Only when every check succeeded: beside a failed team's line it
    // would read as clearing that team, whose violations are unknown.
    if clean && checks.iter().all(Result::is_ok) {
        ui.weak("No violations.");
    }
    violations_list(ui, "aatf", &teams, show_suggestions)
}

/// Team `id`'s rostered players as the list's `ListedPlayer`s, in slot
/// order, their names read through `players`.
fn listed_players<'a>(
    file: &'a EditFile,
    players: &HashMap<u32, &'a PlayerEntry>,
    id: u32,
) -> Vec<ListedPlayer<'a>> {
    let Some(team) = file.team(id) else {
        return Vec::new();
    };
    team.roster
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.player_id != 0)
        .map(|(slot, roster)| ListedPlayer {
            slot: u8::try_from(slot).expect("a roster slot"),
            id: roster.player_id,
            number: roster.number,
            name: players
                .get(&roster.player_id)
                .map(|player| player.name.as_str())
                .unwrap_or("?"),
        })
        .collect()
}

/// One check's worker: `key`'s teams viewed on the caller's thread (the
/// `EditFile` lives there), then `check_teams` on the shared ruleset off
/// it, the key riding the reply so the panel knows what landed.
fn spawn_check(
    ctx: &egui::Context,
    file: &EditFile,
    rules: &Arc<aatf::Rules>,
    key: CheckKey,
) -> PendingCheck {
    let views = rules::team_views(file, &key.ids);
    let (sender, replies) = crossbeam_channel::unbounded();
    let rules = Arc::clone(rules);
    let ctx = ctx.clone();
    let reply_key = key.clone();
    std::thread::spawn(move || {
        let checks = rules::check_teams(&rules, &views);
        // A newer check can replace the receiver mid-flight; that send's
        // failure is the superseded thread ending, not an error.
        if sender.send((reply_key, checks)).is_err() {
            log::debug!("the AATF panel dropped a superseded check");
        }
        ctx.request_repaint();
    });
    PendingCheck { key, replies }
}
