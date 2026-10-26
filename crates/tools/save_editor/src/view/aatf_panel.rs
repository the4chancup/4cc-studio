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

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use pes_savefile::EditFile;

use crate::session::rules;
use crate::view::aatf_check::{self, CheckKey, TeamsCheck};
use crate::view::{ActiveRules, Refusal, SaveView, edit_settled, rules_load_error};

/// Which teams the panel checks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AatfMode {
    /// The selected row's team, else the selector's.
    Current,
    /// The teams ticked in the panel's list.
    Select,
}

/// A Save-file dialog: the suggested file name to the picked path
/// (`rfd` in the window, a stub in the tests — a native dialog cannot be
/// driven headless; the Ruleset editor's `SaveDialog` pattern). The
/// panel's Save a copy and the comparator's Export share the type.
pub(crate) type SaveFileDialog = Box<dyn FnMut(&str) -> Option<PathBuf>>;

/// The `rfd` Save dialog (the Ruleset editor's `default_save_dialog`
/// pattern): the suggested name comes from the caller.
#[cfg(not(test))]
pub(crate) fn default_copy_dialog() -> SaveFileDialog {
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
pub(crate) fn default_copy_dialog() -> SaveFileDialog {
    Box::new(|_| panic!("a test reached a real file dialog: stub aatf_copy_dialog"))
}

/// The panel's state on `SaveView` (`view.aatf`): open or not, the mode,
/// the picked ids, the suggestions toggle, the teams' check, the last
/// validation and the copy's path.
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
    /// The picked teams' check (the worker check `aatf_check` shares with
    /// the Live AATF dock).
    pub(super) check: TeamsCheck,
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
            check: TeamsCheck::new(),
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
        self.check.reset();
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
    let (Some(session), Some(active)) = (session.as_ref(), rules.as_ref()) else {
        return;
    };
    let file = session.file();
    let (_, settled) = edit_settled(ui, *focused);
    // The teams to check: the selected row's team, else the selector's;
    // in Select mode the ticked ids, already in id order.
    let mut no_teams = None;
    let ids: Vec<u32> = match aatf.mode {
        AatfMode::Current => match aatf_check::current_team(*team, *selected) {
            Some(id) => vec![id],
            None => {
                no_teams = Some("No team is open.");
                Vec::new()
            }
        },
        AatfMode::Select => {
            if aatf.picked.is_empty() {
                no_teams = Some("No teams are picked.");
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
    let checking = aatf_check::update_check(
        &mut aatf.check,
        ui.ctx(),
        file,
        loaded.as_ref(),
        key,
        settled,
    );
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
            if checking {
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
                    } else if let Some((result_key, checks)) = &aatf.check.result {
                        clicked = aatf_check::results(
                            ui,
                            file,
                            "aatf",
                            result_key,
                            checks,
                            aatf.show_suggestions,
                            Some("No violations."),
                        );
                    }
                    clicked
                })
                .inner;
            if let Some(player) = clicked {
                aatf_check::select_player(player, selected, team, filter);
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
    copy_dialog: &mut SaveFileDialog,
    refusal: &mut Option<Refusal>,
    file: &EditFile,
) {
    ui.horizontal_wrapped(|ui| {
        if let Ok(rules) = &active.rules {
            ui.weak(format!("Ruleset {} ", rules.ruleset().name));
        }
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
        .on_hover_text(
            "Check the rules file in use for errors, each reported with its line or setting",
        )
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
            ui.weak(format!("There are no findings in {name}."));
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
