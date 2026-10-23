//! The Custom checks section (RE-GUI-13): the file's hand-written
//! checks, read-only in monospace.

/// The Custom checks form: the section as read, kept as it is on save.
pub(crate) fn show(ui: &mut egui::Ui, custom_checks: Option<&str>) {
    ui.heading("Custom checks");
    ui.weak("This code runs after the ruleset's own checks, and is kept as it is when you save.");
    let Some(checks) = custom_checks else {
        return;
    };
    ui.label(egui::RichText::new(checks).monospace());
}
