//! The shared look (core plan, "Other UI components"): the icon font, and the `theme`
//! setting's application to the egui context. The presets stay egui's own dark and light
//! `Visuals` until the curated one lands.

use crate::settings::Theme;

/// Applies the common `theme` setting to `ctx` through `Context::set_theme`: the setting,
/// never the system's theme, decides, so a member on a light desktop still gets the dark
/// default.
pub(crate) fn apply(ctx: &egui::Context, theme: Theme) {
    ctx.set_theme(match theme {
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
    });
}

/// Registers the icon font as `"icons"` (the subset `resources/fonts/phosphor-subset.ttf`
/// ships; `crate::icons` names its glyphs), so icon glyphs (`StudioTool::icon`,
/// `PlannedTool::icon`, the shell's own buttons) render inline inside ordinary labels.
/// It sits second in the proportional family — after the text font, which a member's text
/// must resolve from first, and ahead of egui's emoji fonts, so the icon codepoints come
/// from it — and as its own `icons` family. Installed fonts resolve before the system
/// fallback eframe registers (`Context::set_fonts` replaces the definitions only, not the
/// context's font providers). Called once at window creation; `set_fonts` replaces the
/// whole font set, so the definitions start from egui's defaults and add ours. Native
/// only for now: its one caller is `run_gui`; Studio Web's frontend calls it when that
/// exists.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "icons".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../resources/fonts/phosphor-subset.ttf"
        ))),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .expect("eframe's default fonts fill the proportional family")
        .insert(1, "icons".to_owned());
    fonts.families.insert(
        egui::FontFamily::Name("icons".into()),
        vec!["icons".to_owned()],
    );
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installing_fonts_registers_the_icon_font_second_in_proportional() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        // `Context::fonts` is populated by the first pass.
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
        ctx.fonts(|fonts| {
            let definitions = fonts.definitions();
            assert!(definitions.font_data.contains_key("icons"));
            let proportional = definitions
                .families
                .get(&egui::FontFamily::Proportional)
                .expect("the proportional family exists");
            assert_eq!(
                proportional.iter().position(|name| name == "icons"),
                Some(1),
                "icons is second: after the text font, ahead of egui's emoji fonts"
            );
            assert_eq!(
                definitions
                    .families
                    .get(&egui::FontFamily::Name("icons".into())),
                Some(&vec!["icons".to_owned()]),
                "icons is its own family too"
            );
        });
    }

    #[test]
    fn every_icon_constant_resolves_a_glyph_after_install() {
        const ALL: &[&str] = &[
            crate::icons::ARROW_FAT_UP,
            crate::icons::BUILDINGS,
            crate::icons::CROSSHAIR,
            crate::icons::DATABASE,
            crate::icons::DOWNLOAD_SIMPLE,
            crate::icons::FLAG,
            crate::icons::GEAR,
            crate::icons::MUSIC_NOTE,
            crate::icons::MUSIC_NOTES,
            crate::icons::NOTE_PENCIL,
            crate::icons::PLUS_CIRCLE,
            crate::icons::QUESTION,
            crate::icons::SCROLL,
            crate::icons::SMILEY,
            crate::icons::SOCCER_BALL,
            crate::icons::T_SHIRT,
            crate::icons::USERS_THREE,
            crate::icons::WARNING,
            crate::icons::X,
        ];
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
        ctx.fonts_mut(|fonts| {
            for name in ALL {
                let glyph = name.chars().next().expect("a constant is one glyph");
                assert!(
                    fonts.has_glyph(&egui::FontId::proportional(14.0), glyph),
                    "no font has U+{:04X}",
                    u32::from(glyph)
                );
            }
        });
    }
}
