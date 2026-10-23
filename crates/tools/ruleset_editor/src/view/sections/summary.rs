//! The Summary section (RE-SUM-01): the ruleset as readable rules text
//! (`crate::summary`), and Copy putting it on the clipboard.

use aatf::Ruleset;

use super::Section;

/// The Summary form: the rules text in monospace, and the **Copy**
/// button (`ui.ctx().copy_text`, for posting the rules).
pub(crate) fn show(ui: &mut egui::Ui, ruleset: &Ruleset) {
    ui.heading(Section::Summary.label());
    let text = crate::summary::summary(ruleset);
    ui.horizontal(|ui| {
        if ui
            .button("Copy")
            .on_hover_text("Copies the rules as Markdown, for posting them")
            .clicked()
        {
            ui.ctx().copy_text(text.clone());
        }
        ui.weak("The ruleset as rules text, written from the data.");
    });
    ui.label(egui::RichText::new(text).monospace());
}
