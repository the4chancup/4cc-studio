//! The shared look (core plan, "Other UI components"). Only the icon font lives here for now:
//! the curated `Visuals` presets arrive with the theme step of the GUI phase.

/// Registers the `egui-phosphor` regular font as a fallback for proportional text, so icon
/// glyphs (`StudioTool::icon`, `PlannedTool::icon`) render inline inside ordinary labels.
/// Called once at window creation; `Context::set_fonts` replaces the whole font set, so the
/// definitions start from egui's defaults and add ours. Native only for now: its one caller
/// is `run_gui`; Studio Web's frontend calls it when that exists.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installing_fonts_registers_phosphor_as_a_proportional_fallback() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        // `Context::fonts` is populated by the first pass.
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
        ctx.fonts(|fonts| {
            let definitions = fonts.definitions();
            assert!(definitions.font_data.contains_key("phosphor"));
            let proportional = definitions
                .families
                .get(&egui::FontFamily::Proportional)
                .expect("the proportional family exists");
            assert!(
                proportional.iter().any(|name| name == "phosphor"),
                "phosphor is a fallback of the proportional family"
            );
        });
    }
}
