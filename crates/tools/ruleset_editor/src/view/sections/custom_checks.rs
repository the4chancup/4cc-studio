//! The Custom checks section (RE-GUI-13): the file's hand-written
//! checks, read-only in monospace.

use super::Section;

/// The Custom checks form: the section's code as read, kept as it is on
/// save.
pub(crate) fn show(ui: &mut egui::Ui, custom_checks: Option<&str>) {
    ui.heading(Section::CustomChecks.label());
    ui.weak("This code runs after the ruleset's own checks, and is kept as it is when you save.");
    let Some(checks) = custom_checks else {
        return;
    };
    // `checks` is the section as read, its `==== CUSTOM CHECKS` banner
    // line included so the save writes it back byte for byte; the code
    // alone is what the member reads (the draft shows it alone).
    let code = checks
        .split_once('\n')
        .map(|(_, code)| code)
        .unwrap_or(checks);
    ui.label(egui::RichText::new(code).monospace());
}
