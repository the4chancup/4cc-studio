//! The data-block preview: the ruleset's `to_data_block()` in monospace,
//! read-only, the lines the
//! last edit changed highlighted (the draft's `refresh`).

use crate::view::RulesetView;

/// The right `egui::Panel`, drawn only while **Show preview** is on.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut RulesetView) {
    let Some(document) = view.document.as_ref() else {
        return;
    };
    ui.horizontal(|ui| {
        ui.strong("Rules file: data block");
        ui.weak("(read-only)");
    });
    egui::ScrollArea::both()
        .id_salt("ruleset_preview")
        .auto_shrink(false)
        .show(ui, |ui| {
            for line in document.ruleset.to_data_block().lines() {
                let text = egui::RichText::new(line).monospace();
                let text = if view.changed.contains(line) {
                    text.background_color(ui.visuals().selection.bg_fill)
                } else {
                    text
                };
                ui.add(egui::Label::new(text).selectable(false));
            }
        });
}
