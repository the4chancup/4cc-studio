//! The start page (the draft's `startPage`, as a page rather than a
//! window): the whole view with no document (`page`), a modal over one
//! from File's "New from template…" (`modal`).

use std::path::PathBuf;
use std::sync::OnceLock;

use aatf::Template;
use studio_core::ToolContext;

use crate::settings::RulesetEditorSettings;
use crate::view::{Pending, RulesetView, file_label, guard};

/// The templates, parsed once (`aatf::templates` compiles each embedded
/// block; the list never changes).
pub(crate) fn templates() -> &'static [Template] {
    static TEMPLATES: OnceLock<Vec<Template>> = OnceLock::new();
    TEMPLATES.get_or_init(aatf::templates)
}

/// The whole view when no document is open.
pub(crate) fn page(
    ui: &mut egui::Ui,
    view: &mut RulesetView,
    ctx: &ToolContext,
    settings: &RulesetEditorSettings,
) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            ui.heading("Ruleset editor");
            body(ui, view, ctx, settings);
        });
}

/// The start page over the document (File's "New from template…"), with
/// a Close button back to it.
pub(crate) fn modal(
    ui: &mut egui::Ui,
    view: &mut RulesetView,
    ctx: &ToolContext,
    settings: &RulesetEditorSettings,
) {
    let mut close = false;
    let response = egui::Modal::new(egui::Id::unique("ruleset_start")).show(ui.ctx(), |ui| {
        ui.heading("Ruleset editor");
        body(ui, view, ctx, settings);
        if ui.button("Close").clicked() {
            close = true;
        }
    });
    if close || response.should_close() {
        view.start_open = false;
    }
}

/// The start page's entries (RE-GUI-01): New from template in the
/// embedded rulesets' order then Blank; Random, drawn disabled until
/// its build; Open file…; the recent files, newest first, the existing
/// ones only; and the last open's refusal when there was one.
fn body(
    ui: &mut egui::Ui,
    view: &mut RulesetView,
    ctx: &ToolContext,
    settings: &RulesetEditorSettings,
) {
    ui.strong("New from template");
    for (index, template) in templates().iter().enumerate() {
        if ui.button(button_label(template)).clicked() {
            guard(view, ctx, settings, Pending::Template(index));
        }
    }
    if ui
        .button("Blank: one tier, no brackets, the universal rules on")
        .clicked()
    {
        guard(view, ctx, settings, Pending::Blank);
    }
    ui.separator();
    ui.horizontal(|ui| {
        ui.strong("Random");
        for preset in ["Sensible", "Weird", "Crazy"] {
            ui.add_enabled(false, egui::Button::new(preset))
                .on_disabled_hover_text("The generator comes in a later build");
        }
    });
    ui.separator();
    if ui.button("Open file…").clicked() {
        guard(view, ctx, settings, Pending::OpenFile);
    }
    ui.separator();
    ui.strong("Recent files");
    let recent: Vec<PathBuf> = settings
        .recent_files
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect();
    if recent.is_empty() {
        ui.weak("none");
    }
    for path in recent {
        if ui
            .button(file_label(&path))
            .on_hover_text(path.display().to_string())
            .clicked()
        {
            guard(view, ctx, settings, Pending::OpenPath(path));
        }
    }
    // The last open's refusal (RE-FILE-03): the file's name, then one
    // line per finding as `validate` prints them.
    if let Some((name, lines)) = &view.open_refusal {
        ui.separator();
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("Could not open {name}:"),
        );
        for line in lines {
            ui.colored_label(ui.visuals().error_fg_color, line);
        }
    }
}

/// A template button's label (the draft's): the two that need a word
/// about them get it.
fn button_label(template: &Template) -> String {
    match template.ruleset.name.as_str() {
        "Autumn 26" => "Autumn 26 (the official rules)".to_string(),
        "DBC7" => "DBC7 (Divegrass Before Christmas 7)".to_string(),
        name => name.to_string(),
    }
}
