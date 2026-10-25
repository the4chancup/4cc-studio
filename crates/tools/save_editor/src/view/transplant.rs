//! The aesthetics transplant panel (`plans/save_editor.md` "Aesthetics
//! transplant"; the draft's `transplantPanel`): an `egui::Window` copying a
//! picked donor save's looks onto the players the selection names — the
//! CLI's `transplant` in the window, the aesthetics diff of the result
//! previewed on a worker before Apply (a wrong team id shows as an
//! unexpected list of players, not as damage found later). The selection
//! text is the CLI's `--select`: player ids, `target:donor` pairs, teams,
//! team ranges; an edit of it withdraws the preview until Preview is
//! pressed again. The preview's key is the open save's and the donor's
//! `Session::revision` plus the selection text, so after Apply — itself
//! `Session::transplant`, one sealed undo entry, written only on Save — the
//! armed preview recomputes and reads `No player of the selection changes.`
//! Under the read-only lock Apply is disabled with the ownership line
//! (SE-GUI-22); the two-pane browser is a later slice's.

use std::path::PathBuf;

use pes_savefile::EditFile;
use pes_savefile::ops::compare::DiffScope;
use pes_savefile::ops::transplant::{parse_selection, transplant};
use pes_version::PesVersion;

use crate::diff;
use crate::messages::{stripped_mark, version_mismatch_sentence, when_text};
use crate::session::{Session, Stripped, is_stripped};
use crate::view::appearance::ownership_tip;
use crate::view::comparator::{self, Line};
use crate::view::worker::{Pending, WorkerResult};
use crate::view::{Refusal, SaveView, aatf_check, edit_settled, surrender_focus};

/// The panel's state on `SaveView` (`view.transplant`): open or not, the
/// picked donor save, the last pick's refusal, the selection text, whether
/// Preview armed the worker, the last Apply's mark and the preview's worker
/// state.
pub(super) struct Transplant {
    /// Whether the window is drawn (the Panels menu's `Aesthetics
    /// transplant…` item).
    pub(super) open: bool,
    /// The picked donor save: its path and its session, opened through
    /// `Session::open` so a refusal is the toolbar's own (`<path>: cannot
    /// open the save`).
    pub(super) donor: Option<(PathBuf, Session)>,
    /// The last pick's refusal — a save of another version or one that did
    /// not open — drawn error-coloured under the donor row.
    refused: Option<String>,
    /// The selection text (the CLI's `--select` text).
    pub(super) selection: String,
    /// Whether Preview was pressed for the current selection: while set,
    /// the preview re-runs whenever the key moves (an edit, Apply). An
    /// edit of the text clears it.
    armed: bool,
    /// The revision `Session::transplant` left the save at: the `Applied;
    /// unsaved until Save.` line draws while it is still current and the
    /// session dirty — an Undo, a Redo, a later edit or a Save ends it.
    applied: Option<u64>,
    /// The preview: the last result with the key it ran for and the run in
    /// flight (`WorkerResult`'s cycle, the comparator's own).
    pub(super) preview: WorkerResult<PreviewKey, Result<Preview, String>>,
}

/// What a preview ran for: a change of the open save (`Session::revision`),
/// a new donor or a new selection text recomputes it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PreviewKey {
    /// `Session::revision` of the open save when the run was due.
    pub(super) revision: u64,
    /// The donor session's `Session::revision` — a re-pick of the same
    /// path is a fresh session, so the file's own changes recompute.
    pub(super) donor: u64,
    /// The selection text the run's pairs come from.
    pub(super) selection: String,
}

/// One preview's result: the selection's pairs for Apply, the
/// aesthetics-diff line list (the comparator's `lines` over the open save
/// against a copy with the pairs applied) and the stripped marks' warning
/// lines.
pub(super) struct Preview {
    /// The selection's `(target, donor)` pairs, in selection order.
    pairs: Vec<(u32, u32)>,
    /// The line list `comparator::lines` built for the result's diff.
    pub(super) lines: Vec<Line>,
    /// One warning line per stripped pair player (the
    /// `transplant_target_stripped` / `transplant_donor_stripped` texts).
    marks: Vec<String>,
}

impl Transplant {
    /// The panel closed, nothing picked.
    pub(super) fn new() -> Transplant {
        Transplant {
            open: false,
            donor: None,
            refused: None,
            selection: String::new(),
            armed: false,
            applied: None,
            preview: WorkerResult::new(),
        }
    }

    /// Another save opened (`SaveView::open`): the donor stays unless the
    /// new save is of another version, refused as a pick is (the
    /// comparator's `reset` rule); the selection text stays, the preview
    /// and the applied mark go.
    pub(super) fn reset(&mut self, open_version: PesVersion) {
        self.preview.reset();
        self.armed = false;
        self.applied = None;
        self.refused = None;
        let donor_version = self.donor.as_ref().map(|(_, donor)| donor.version());
        if let Some(donor) = donor_version
            && donor != open_version
        {
            self.donor = None;
            self.refused = Some(mismatch_refusal(open_version, donor));
        }
    }
}

/// The pick's refusal for a donor of another version (the CLI's own
/// sentence, the base first as `transplant` prints it).
fn mismatch_refusal(open: PesVersion, donor: PesVersion) -> String {
    version_mismatch_sentence("The base save", open, "the donor", donor)
}

/// The window for one frame, while `view.transplant.open` and a save is
/// open: the donor row (Pick, Apply), the refusal, the selection row, the
/// `Previewing…` mark and the preview's lines. The reply is polled before a
/// new run is decided on, as the comparator's is; a closed panel starts no
/// run.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let SaveView {
        transplant,
        session,
        team,
        filter,
        selected,
        refusal,
        open_save_dialog,
        edit_aesthetics,
        focused,
        ..
    } = view;
    if !transplant.open {
        return;
    }
    let Some(session) = session.as_mut() else {
        return;
    };
    let (_, settled) = edit_settled(ui, *focused);
    // A run is due while armed when the key moved on since the last result
    // (a donor is always of the open save's version: the pick and `reset`
    // refuse the rest).
    let key = transplant.donor.as_ref().map(|(_, donor)| PreviewKey {
        revision: session.revision(),
        donor: donor.revision(),
        selection: transplant.selection.clone(),
    });
    let previewing = if let Some(key) = &key {
        let donor = transplant.donor.as_ref().map(|(_, donor)| donor.file());
        transplant
            .preview
            .update(key, transplant.armed, settled, |key| {
                spawn_preview(
                    ui.ctx(),
                    donor.expect("the key names a donor"),
                    session.file(),
                    key,
                )
            })
    } else {
        false
    };
    let mut open = transplant.open;
    egui::Window::new("Aesthetics transplant")
        .id(egui::Id::unique("save_editor_transplant"))
        .default_width(620.0)
        .collapsible(false)
        .resizable(true)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            // One line: the label, the path truncated to the room the
            // buttons leave, the buttons at the row's right — a real
            // save's path once wrapped the buttons onto a line of their
            // own.
            ui.horizontal_top(|ui| {
                ui.weak("Donor save ");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    // Apply, on the donor row, with a donor: the shown preview
                    // must be for the current selection, open save and donor
                    // and hold at least one row; the lock's hover is the
                    // ownership line (SE-GUI-22).
                    if transplant.donor.is_some() {
                        let current = transplant
                            .preview
                            .result
                            .as_ref()
                            .is_some_and(|(held, _)| Some(held) == key.as_ref());
                        let has_row =
                            transplant
                                .preview
                                .result
                                .as_ref()
                                .is_some_and(|(_, result)| {
                                    result.as_ref().is_ok_and(|preview| {
                                        preview
                                            .lines
                                            .iter()
                                            .any(|line| matches!(line, Line::Row { .. }))
                                    })
                                });
                        let apply = *edit_aesthetics && current && has_row;
                        let tip = if *edit_aesthetics {
                            "Write the donor's appearance onto the selection, one undoable edit"
                        } else {
                            ownership_tip(session.version())
                        };
                        let response = ui.add_enabled(apply, egui::Button::new("Apply"));
                        let response = if apply {
                            response.on_hover_text(tip)
                        } else {
                            response.on_disabled_hover_text(tip)
                        };
                        if response.clicked() {
                            // Apply's own re-preview waits for `edit_settled`
                            // just as a started run does.
                            surrender_focus(ui);
                            if let Some((_, donor)) = &transplant.donor
                                && let Some((_, Ok(preview))) = &transplant.preview.result
                            {
                                let pairs = preview.pairs.clone();
                                match session.transplant(donor, &pairs) {
                                    Ok(_) => transplant.applied = Some(session.revision()),
                                    Err(error) => {
                                        *refusal = Some(Refusal {
                                            title: "Cannot transplant",
                                            message: format!("{error}"),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    if ui
                        .button("Pick\u{2026}")
                        .on_hover_text("The save the looks are copied from")
                        .clicked()
                    {
                        surrender_focus(ui);
                        if let Some(path) = open_save_dialog() {
                            // The held preview is of the old donor, the applied
                            // mark of an old write.
                            transplant.preview.reset();
                            transplant.armed = false;
                            transplant.applied = None;
                            match Session::open(&path) {
                                Ok(donor) if donor.version() == session.version() => {
                                    transplant.donor = Some((path, donor));
                                    transplant.refused = None;
                                }
                                Ok(donor) => {
                                    transplant.donor = None;
                                    transplant.refused =
                                        Some(mismatch_refusal(session.version(), donor.version()));
                                }
                                Err(error) => {
                                    transplant.donor = None;
                                    transplant.refused = Some(format!("{error:#}"));
                                }
                            }
                        }
                    }
                    match &transplant.donor {
                        Some((path, _)) => {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(path.display().to_string()).monospace(),
                                )
                                .truncate(),
                            );
                        }
                        None => {
                            ui.weak("(none)");
                        }
                    }
                });
            });
            if let Some(refused) = &transplant.refused {
                ui.colored_label(ui.visuals().error_fg_color, refused.as_str());
            }
            if transplant.donor.is_none() {
                if transplant.refused.is_none() {
                    ui.weak("Pick a donor save of the same version.");
                }
                return;
            }
            ui.horizontal_wrapped(|ui| {
                ui.weak("Selection ");
                if ui
                    .add(egui::TextEdit::singleline(&mut transplant.selection).desired_width(340.0))
                    .changed()
                {
                    // The held preview is of the old text: withdraw it
                    // until Preview is pressed again.
                    transplant.armed = false;
                    transplant.preview.reset();
                }
                if ui
                    .button("Preview")
                    .on_hover_text("The aesthetics diff of the result")
                    .clicked()
                {
                    surrender_focus(ui);
                    transplant.armed = true;
                    transplant.applied = None;
                }
            });
            ui.add(egui::Label::new(
                egui::RichText::new("Player ids, target:donor pairs, teams, team ranges.")
                    .weak()
                    .small(),
            ));
            // Above the list, as the comparator's mark: the rows below are
            // the previous result's while a run is in flight.
            if previewing {
                ui.weak("Previewing\u{2026}");
            }
            match &transplant.preview.result {
                Some((_, Ok(preview))) => {
                    for mark in &preview.marks {
                        ui.colored_label(ui.visuals().warn_fg_color, mark);
                    }
                    if preview.lines.is_empty() {
                        if preview.marks.is_empty() {
                            ui.weak("No player of the selection changes.");
                        }
                    } else {
                        let rows = preview
                            .lines
                            .iter()
                            .filter(|line| matches!(line, Line::Row { .. }))
                            .count();
                        ui.weak(format!("{rows} changed field(s)"));
                        let clicked = egui::ScrollArea::vertical()
                            .id_salt("save_editor_transplant_rows")
                            .auto_shrink([false, false])
                            .show_rows(
                                ui,
                                ui.text_style_height(&egui::TextStyle::Body),
                                preview.lines.len(),
                                |ui, range| {
                                    let mut clicked = None;
                                    for i in range {
                                        if let Some(player) =
                                            comparator::line(ui, &preview.lines[i])
                                        {
                                            clicked = Some(player);
                                        }
                                    }
                                    clicked
                                },
                            )
                            .inner;
                        if let Some(player) = clicked {
                            aatf_check::select_player(player, selected, team, filter);
                        }
                    }
                }
                Some((_, Err(error))) => {
                    ui.colored_label(ui.visuals().error_fg_color, error.as_str());
                }
                None => {}
            }
            if transplant
                .applied
                .is_some_and(|at| at == session.revision() && session.is_dirty())
            {
                ui.weak("Applied; unsaved until Save.");
            }
        });
    transplant.open = open;
}

/// One preview's worker: both `EditFile`s cloned for the thread (the
/// comparator's `spawn_compare` shape — the open save's clone costs ~4 ms
/// on the PES 21 fixture, nothing against the parse-and-diff run), then
/// the selection parsed, the pairs applied to the open save's clone and
/// the aesthetics diff of the two, the key riding the reply so the caller
/// knows what landed.
fn spawn_preview(
    ctx: &egui::Context,
    donor: &EditFile,
    open: &EditFile,
    key: &PreviewKey,
) -> Pending<PreviewKey, Result<Preview, String>> {
    let (donor, open) = (donor.clone(), open.clone());
    let selection = key.selection.clone();
    let (sender, replies) = crossbeam_channel::unbounded();
    let reply_key = key.clone();
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let result = preview(&donor, &open, &selection);
        // A newer preview can replace the receiver mid-flight; that
        // send's failure is the superseded thread ending, not an error.
        if sender.send((reply_key, result)).is_err() {
            log::debug!("the transplant panel's caller dropped a superseded preview");
        }
        ctx.request_repaint();
    });
    Pending {
        key: key.clone(),
        replies,
    }
}

/// One preview run: the selection text parsed item by item (empty items
/// skipped), the pairs applied to a copy of `open`, then the aesthetics
/// rows of the before/after diff in the comparator's `lines` order — a row
/// reads `<label> <the target's value now> → <the donor's>` — and one
/// warning line per stripped pair player. A parse error or a refused
/// transplant (a target or donor id a save lacks) is the error line.
fn preview(donor: &EditFile, open: &EditFile, selection: &str) -> Result<Preview, String> {
    let mut pairs = Vec::new();
    for item in selection.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        match parse_selection(item) {
            Ok(more) => pairs.extend(more),
            Err(error) => return Err(error.to_string()),
        }
    }
    let mut after = open.clone();
    transplant(&mut after, donor, &pairs).map_err(|error| error.to_string())?;
    let mut report = diff::build(open, &after, Some(DiffScope::Aesthetics))
        .map_err(|error| error.to_string())?;
    // A head's stripped mark is the comparator's, naming which of two saves
    // a player is at -1 in; here both sides are the open save, before and
    // after, and the warning lines already name a stripped pair player. A
    // player whose only entry was that mark has nothing to preview.
    report.players.retain_mut(|player| {
        player.stripped = None;
        !player.rows.is_empty()
    });
    let lines = comparator::lines(&report, open, &after);
    let mut marks = Vec::new();
    for &(target, source) in &pairs {
        if is_stripped(&after, target) {
            marks.push(mark_line(Stripped::Target(target)));
        }
        if is_stripped(donor, source) {
            marks.push(mark_line(Stripped::Donor(source)));
        }
    }
    Ok(Preview {
        pairs,
        lines,
        marks,
    })
}

/// A mark's warning line: the `transplant_target_stripped` /
/// `transplant_donor_stripped` catalog text, the player named.
fn mark_line(mark: Stripped) -> String {
    let id = match mark {
        Stripped::Target(id) | Stripped::Donor(id) => id,
    };
    format!("Player {id}: {}", when_text(&stripped_mark(mark)))
}

#[cfg(test)]
mod tests {
    use pes_savefile::fixture_save;
    use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;

    use super::*;

    // A stripped target whose look the donor changes (his boots id here):
    // his head lists his rows but no ", stripped in both" — the
    // comparator's mark names which of two saves a player is at -1 in, and
    // here both sides are the open save; the warning lines name him.
    #[test]
    fn a_stripped_target_with_changes_gets_a_mark_free_head() {
        let mut open = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).expect("loads");
        open.player_mut(70103)
            .expect("70103 is a player")
            .appearance_id = Some(STRIPPED_APPEARANCE_ID);
        let mut donor = open.clone();
        donor
            .player_mut(70103)
            .expect("70103 is a donor player")
            .appearance
            .boots_id = 37;

        let preview = preview(&donor, &open, "70103").expect("the preview runs");
        assert!(
            preview
                .lines
                .iter()
                .any(|line| matches!(line, Line::Head { .. })),
            "70103's head: {:?}",
            preview.lines.len()
        );
        for line in &preview.lines {
            let text = match line {
                Line::Heading(text) => text,
                Line::Head { text, .. } | Line::Row { text, .. } => text,
            };
            assert!(!text.contains("stripped"), "no mark in {text:?}");
        }
        assert_eq!(preview.marks.len(), 2, "the target's mark and the donor's");
        assert!(preview.marks[0].starts_with("Player 70103: "));
    }

    // A pair whose target is at -1 and whose look the donor already shares
    // has no row: the warning line names him, and no head reads the
    // comparator's "stripped in both", which would mean nothing with one
    // save before and after.
    #[test]
    fn a_stripped_target_with_no_change_is_a_warning_line_alone() {
        let mut open = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).expect("loads");
        open.player_mut(70103)
            .expect("70103 is a player")
            .appearance_id = Some(STRIPPED_APPEARANCE_ID);
        let donor = open.clone();

        let preview = preview(&donor, &open, "70103").expect("the preview runs");
        assert!(preview.lines.is_empty(), "no head, no row");
        assert_eq!(preview.marks.len(), 2, "the target's mark and the donor's");
        assert!(preview.marks[0].starts_with("Player 70103: "));
    }
}
