//! A settings row (core plan, "Settings menu"; the draft `docs/prototypes/index.html`'s
//! `sRow`): the label cell, its tooltip leading with the settings key, and the widget
//! cell. `path_row` is the one row every section's path setting draws — the common
//! folders' and a tool's file alike — so no tool's path commits differently from the
//! common ones: the text edit commits on Enter or focus loss, never per keystroke, and
//! the Browse button opens `rfd`'s native picker of the row's kind, a pick committing
//! at once.

use std::path::PathBuf;

/// What `path_row`'s Browse button opens (gui.md "Settings menu"): the native picker
/// of the setting's kind, a folder or a file filtered to the type the tool reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    /// A folder (`rfd`'s `pick_folder`).
    Folder,
    /// A file of `filter`'s `extensions` (`rfd`'s `pick_file` behind an `add_filter`).
    File {
        /// The filter's name as the dialog shows it.
        filter: &'static str,
        /// The file endings the picker accepts, without the dot (`4ccrules`).
        extensions: &'static [&'static str],
    },
}

/// A row's left cell: the plain label, its tooltip the settings key then the field's
/// doc comment first sentence (the draft's `sRow`), so a member who edits
/// `settings.toml` by hand still finds it.
pub(crate) fn row_label(ui: &mut egui::Ui, label: &str, key: &str, tip: &str) -> egui::Response {
    ui.label(label).on_hover_text(format!("{key}: {tip}"))
}

/// A path row: the edit binds `buffer` and commits it to `current` when it loses focus
/// or Enter is pressed, returning whether the commit changed the setting; the Browse
/// button after the edit opens `kind`'s native picker and a pick sets the buffer and
/// the setting at once (a commit). While the row has no focus the buffer follows
/// `current`, so a change made outside the menu (the notice's "Fix PES path") still
/// shows.
pub fn path_row(
    ui: &mut egui::Ui,
    label: &str,
    key: &str,
    tip: &str,
    buffer: &mut String,
    current: &mut String,
    _kind: PathKind,
) -> bool {
    let label = row_label(ui, label, key, tip);
    let (response, picked) = ui
        .horizontal(|ui| {
            let response = ui
                .add(egui::TextEdit::singleline(buffer))
                .labelled_by(label.id);
            // `rfd`'s pickers are native-only, so the wasm row is the edit
            // alone (`_kind`: `browse` exists only off wasm).
            #[cfg(not(target_arch = "wasm32"))]
            let picked = browse(ui, _kind);
            #[cfg(target_arch = "wasm32")]
            let picked: Option<PathBuf> = None;
            (response, picked)
        })
        .inner;
    ui.end_row();
    if let Some(path) = picked {
        *buffer = path.display().to_string();
        *current = buffer.clone();
        return true;
    }
    if response.lost_focus() {
        if buffer == current {
            return false;
        }
        *current = buffer.clone();
        return true;
    }
    // The buffer follows `current` only while the edit lacks focus in
    // egui's memory: `Response::has_focus` is also false while the
    // window is unfocused, so resetting on it would drop a half-typed
    // path on an Alt+Tab.
    if !ui.ctx().memory(|memory| memory.has_focus(response.id)) {
        *buffer = current.clone();
    }
    false
}

/// The Browse button and the native picker it opens, `kind`'s: `rfd`'s blocking
/// folder dialog or file dialog filtered to `extensions`; `Some` only on a pick.
#[cfg(not(target_arch = "wasm32"))]
fn browse(ui: &mut egui::Ui, kind: PathKind) -> Option<PathBuf> {
    if !ui.button("Browse").clicked() {
        return None;
    }
    match kind {
        PathKind::Folder => rfd::FileDialog::new().pick_folder(),
        PathKind::File { filter, extensions } => rfd::FileDialog::new()
            .add_filter(filter, extensions)
            .pick_file(),
    }
}
