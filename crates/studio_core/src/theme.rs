//! The shared look (core plan, "Other UI components"): the fonts, the drafts'
//! style and the `theme` setting's application to the egui context. `install`
//! puts the one font set and the pinned style on a context: `run_gui` calls it
//! at window creation and every view test's harness calls it too, so a size a
//! test measures is the size a member sees. The spacing and both themes'
//! widget states are egui 0.36's, the look the drafts were reviewed in; the
//! rest of the presets stays egui's own `Visuals` until the curated one lands.

use crate::settings::Theme;
use crate::status::NoticeKind;

/// The success notices' bright green (the draft's `--success`). egui's `Visuals`
/// has a warning colour and an error colour but no success one, so the suite
/// carries its own.
pub const SUCCESS_FG: egui::Color32 = egui::Color32::from_rgb(0x4c, 0xd9, 0x64);

/// The success green on the light theme's panel: `SUCCESS_FG` there reads at a contrast of
/// 1.7 against egui's light panel, where this darker green reads at 5.6 (the light theme's
/// own link and warning colours are darker for the same reason).
pub const SUCCESS_FG_LIGHT: egui::Color32 = egui::Color32::from_rgb(0x13, 0x73, 0x33);

/// The colour a notice's text is drawn in: its kind's bright colour ("A notice
/// is seen, not found", core plan "Status bar").
pub fn notice_color(kind: NoticeKind, visuals: &egui::Visuals) -> egui::Color32 {
    match kind {
        NoticeKind::Success if visuals.dark_mode => SUCCESS_FG,
        NoticeKind::Success => SUCCESS_FG_LIGHT,
        NoticeKind::Info => visuals.hyperlink_color,
        NoticeKind::Warning => visuals.warn_fg_color,
    }
}

/// Applies the common `theme` setting to `ctx` through `Context::set_theme`: the setting,
/// never the system's theme, decides, so a member on a light desktop still gets the dark
/// default.
pub(crate) fn apply(ctx: &egui::Context, theme: Theme) {
    ctx.set_theme(match theme {
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
    });
}

/// Installs the suite's look on `ctx`: the fonts (`install_fonts`) and the
/// drafts' style. `run_gui` calls it at window creation and every view test's
/// harness calls it too, so a size a test measures is the size a member sees.
/// The spacing and both themes' widget states are egui 0.36's, the look the
/// drafts the GUI is built from were reviewed in: 0.37 moved them (the button
/// padding to 8 by 2, the controls to 20 high, the dark buttons to grey 50
/// with text at 215), so `install` pins them on both themes itself and an egui
/// upgrade cannot move the approved look (core plan "Other UI components", the
/// "Dark/light theme" row).
#[cfg(not(target_arch = "wasm32"))]
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        ctx.style_mut_of(theme, |style| {
            style.spacing.button_padding = egui::vec2(4.0, 1.0);
            style.spacing.interact_size = egui::vec2(40.0, 18.0);
            style.visuals.widgets = match theme {
                egui::Theme::Dark => widgets_dark(),
                egui::Theme::Light => widgets_light(),
            };
        });
    }
}

/// egui 0.36.1's `Widgets::dark()`, field for field: the drafts' stylesheet
/// draws widgets in these colors, and 0.37's `Widgets::dark()` moved them (the
/// buttons to grey 50 with text at 215, the radii to 4 to 6 points), so it
/// cannot supply them.
#[cfg(not(target_arch = "wasm32"))]
fn widgets_dark() -> egui::style::Widgets {
    egui::style::Widgets {
        noninteractive: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(27),
            bg_fill: egui::Color32::from_gray(27),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(140)),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
        inactive: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(60),
            bg_fill: egui::Color32::from_gray(60),
            bg_stroke: egui::Stroke::default(),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(180)),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
        hovered: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(70),
            bg_fill: egui::Color32::from_gray(70),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(150)),
            fg_stroke: egui::Stroke::new(1.5, egui::Color32::from_gray(240)),
            corner_radius: egui::CornerRadius::same(3),
            expansion: 0.0,
        },
        active: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(55),
            bg_fill: egui::Color32::from_gray(55),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
            fg_stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
        open: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(45),
            bg_fill: egui::Color32::from_gray(27),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(210)),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
    }
}

/// egui 0.36.1's `Widgets::light()`, field for field, for the same reason as
/// `widgets_dark`: 0.37's `Widgets::light()` rounded every state to 6 points,
/// which is not the look the drafts were reviewed in.
#[cfg(not(target_arch = "wasm32"))]
fn widgets_light() -> egui::style::Widgets {
    egui::style::Widgets {
        noninteractive: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(248),
            bg_fill: egui::Color32::from_gray(248),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(190)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
        inactive: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(230),
            bg_fill: egui::Color32::from_gray(230),
            bg_stroke: egui::Stroke::default(),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
        hovered: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(220),
            bg_fill: egui::Color32::from_gray(220),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(105)),
            fg_stroke: egui::Stroke::new(1.5, egui::Color32::BLACK),
            corner_radius: egui::CornerRadius::same(3),
            expansion: 0.0,
        },
        active: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(165),
            bg_fill: egui::Color32::from_gray(165),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::BLACK),
            fg_stroke: egui::Stroke::new(2.0, egui::Color32::BLACK),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
        open: egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_gray(220),
            bg_fill: egui::Color32::from_gray(220),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(160)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::BLACK),
            corner_radius: egui::CornerRadius::same(2),
            expansion: 0.0,
        },
    }
}

/// Registers the suite's fonts on `ctx`: the body font `"Ubuntu-Regular"`, the
/// file `resources/fonts/Ubuntu-Regular.ttf` bundles — the drafts the GUI is built
/// from are drawn at weight 400 and egui ships only the light weight, so the
/// family file replaces it (`resources/fonts/README.md`) — first in the
/// proportional family; the icon font `"icons"` (the subset
/// `resources/fonts/phosphor-subset.ttf` ships; `crate::icons` names its glyphs)
/// second — after the text font, which a member's text must resolve from first,
/// and ahead of egui's emoji fonts, so the icon codepoints come from it — and as
/// its own `icons` family. Monospace stays egui's Hack, its fallback the regular
/// weight too. Every build that has `studio_core` has egui's default fonts, through
/// its `eframe` dependency's `default_fonts`. Installed fonts resolve
/// before the system fallback eframe registers (`Context::set_fonts` replaces the
/// definitions only, not the context's font providers); `set_fonts` replaces the
/// whole font set, so the definitions start from egui's defaults and add ours.
/// Native only for now: Studio Web's frontend calls it when that exists.
#[cfg(not(target_arch = "wasm32"))]
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Ubuntu-Regular".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../resources/fonts/Ubuntu-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "icons".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../resources/fonts/phosphor-subset.ttf"
        ))),
    );
    // The regular weight takes the light one's place in every family, not only
    // the proportional one: monospace falls back to it for what Hack lacks, and
    // a family naming a `font_data` key that is not there panics in
    // `fonts_for_family`.
    fonts.font_data.remove("Ubuntu-Light");
    for name in fonts.families.values_mut().flatten() {
        if name == "Ubuntu-Light" {
            "Ubuntu-Regular".clone_into(name);
        }
    }
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
    fn each_notice_kind_gets_its_own_bright_colour() {
        for (visuals, success) in [
            (egui::Visuals::dark(), SUCCESS_FG),
            (egui::Visuals::light(), SUCCESS_FG_LIGHT),
        ] {
            assert_eq!(notice_color(NoticeKind::Success, &visuals), success);
            assert_eq!(
                notice_color(NoticeKind::Info, &visuals),
                visuals.hyperlink_color
            );
            assert_eq!(
                notice_color(NoticeKind::Warning, &visuals),
                visuals.warn_fg_color
            );
        }
    }

    #[test]
    fn install_pins_egui_0_36s_spacing_and_widget_states() {
        let ctx = egui::Context::default();
        install(&ctx);
        for (theme, inactive_fill, inactive_text) in
            [(egui::Theme::Dark, 60, 180), (egui::Theme::Light, 230, 60)]
        {
            let style = ctx.style_of(theme);
            assert_eq!(style.spacing.button_padding, egui::vec2(4.0, 1.0));
            assert_eq!(style.spacing.interact_size, egui::vec2(40.0, 18.0));
            let widgets = &style.visuals.widgets;
            assert_eq!(
                widgets.inactive.weak_bg_fill,
                egui::Color32::from_gray(inactive_fill)
            );
            assert_eq!(
                widgets.inactive.fg_stroke.color,
                egui::Color32::from_gray(inactive_text)
            );
            assert_eq!(widgets.hovered.corner_radius, egui::CornerRadius::same(3));
        }
    }

    #[test]
    fn installing_fonts_puts_ubuntu_regular_first_and_the_icons_second() {
        let ctx = egui::Context::default();
        install(&ctx);
        // `Context::fonts` is populated by the first pass.
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
        ctx.fonts(|fonts| {
            let definitions = fonts.definitions();
            assert!(definitions.font_data.contains_key("Ubuntu-Regular"));
            assert!(definitions.font_data.contains_key("icons"));
            assert!(
                !definitions.font_data.contains_key("Ubuntu-Light"),
                "egui's light weight is out: the drafts are drawn at weight 400"
            );
            let proportional = definitions
                .families
                .get(&egui::FontFamily::Proportional)
                .expect("the proportional family exists");
            assert_eq!(
                proportional.first().map(String::as_str),
                Some("Ubuntu-Regular"),
                "the body font resolves text first"
            );
            assert!(
                !proportional.iter().any(|name| name == "Ubuntu-Light"),
                "no light weight in the family"
            );
            let monospace = definitions
                .families
                .get(&egui::FontFamily::Monospace)
                .expect("the monospace family exists");
            assert_eq!(
                monospace.get(..2),
                Some(&["Hack".to_owned(), "Ubuntu-Regular".to_owned()][..]),
                "monospace keeps Hack first and falls back to the regular weight"
            );
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
        install(&ctx);
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
