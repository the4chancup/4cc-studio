//! The toolbar's ruleset dropdown (the draft's `rulesetPicker`): the AATF
//! rulesets built into the binary by description, in the templates'
//! order, then under a separator the imported ones by description. A pick
//! writes `view.aatf_ruleset`, which `Tool::view` writes back to the
//! setting; the next frame's `show` resolves it, and the caches keyed to
//! the ruleset follow.

use std::sync::OnceLock;

use crate::view::SaveView;

/// The dropdown's tooltip (the draft's).
const TOOLTIP: &str = "The AATF rules the quick actions, the card counts and Live AATF use";

/// The built-in rulesets' stems and descriptions, in the templates' order,
/// read once: `aatf::templates` parses every embedded block, and the list
/// never changes while the binary runs.
fn builtins() -> &'static [(&'static str, String)] {
    static BUILTINS: OnceLock<Vec<(&'static str, String)>> = OnceLock::new();
    BUILTINS.get_or_init(|| {
        aatf::templates()
            .into_iter()
            .map(|template| (template.stem, template.ruleset.name))
            .collect()
    })
}

/// The dropdown for one frame: the active ruleset's description closed,
/// as wide as its widest row so a pick never resizes the toolbar.
pub(super) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let active = view
        .rules
        .as_ref()
        .expect("`show` resolves the ruleset before the toolbar draws");
    let key = active.key.clone();
    let selected = active.rules.ruleset().name.clone();
    let font = egui::TextStyle::Button.resolve(ui.style());
    let widest = builtins()
        .iter()
        .map(|(_, name)| name.as_str())
        .chain(
            view.imported
                .iter()
                .map(|import| import.rules.ruleset().name.as_str()),
        )
        .map(|name| {
            ui.painter()
                .layout_no_wrap(name.to_owned(), font.clone(), egui::Color32::WHITE)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
    // The closed combo's chrome around its text: the button's padding on
    // both sides, the gap and the arrow.
    let chrome = {
        let spacing = ui.spacing();
        2.0 * spacing.button_padding.x + spacing.icon_spacing + spacing.icon_width
    };
    let mut picked = None;
    egui::ComboBox::from_id_salt("save_editor_ruleset")
        .selected_text(selected)
        .width(widest + chrome)
        .show_ui(ui, |ui| {
            for (stem, name) in builtins() {
                if ui.selectable_label(key == *stem, name).clicked() {
                    picked = Some((*stem).to_owned());
                }
            }
            if !view.imported.is_empty() {
                ui.separator();
            }
            for import in &view.imported {
                if ui
                    .selectable_label(key == import.file_name, &import.rules.ruleset().name)
                    .clicked()
                {
                    picked = Some(import.file_name.clone());
                }
            }
        })
        .response
        .on_hover_text(TOOLTIP);
    if let Some(picked) = picked {
        view.aatf_ruleset = picked;
    }
}
