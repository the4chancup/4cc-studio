//! Full Player Customization as data: the kit-config values an FPC team's
//! kits carry, the three player-appearance presets, and the interference
//! rules for settings that break or bend them. Pure data and rules; the
//! kit-config and savefile crates own the formats these values map onto.

mod interference;
mod kit;
mod player;

pub use interference::{Finding, Severity, StripStyle, check};
pub use kit::{KitFpcValues, kit_values};
pub use player::{
    Appearance, GK_GLOVES_RANGE, NONEXISTENT_BOOTS_ID, NONEXISTENT_GLOVES_ID, Preset, SkinColor,
    Sleeves, Socks, Tuck, preset,
};

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;

    use super::*;

    #[test]
    fn kit_values_per_version() {
        let expected = KitFpcValues {
            shirt_model: 176,
            shorts_model: 16,
            collar: 105,
            winter_collar: 105,
        };
        for version in [
            PesVersion::Pes16,
            PesVersion::Pes17,
            PesVersion::Pes19,
            PesVersion::Pes20,
            PesVersion::Pes21,
        ] {
            assert_eq!(kit_values(version), Some(expected));
        }
        assert_eq!(kit_values(PesVersion::Pes15), None);
        assert_eq!(kit_values(PesVersion::Pes18), None);
    }

    #[test]
    fn presets_match_the_text() {
        assert_eq!(
            preset(Preset::Hide),
            Appearance {
                sleeves: Sleeves::Long,
                tuck: Tuck::Tucked,
                socks: Socks::Short,
                boots_id: 55,
                gloves_id: 11,
                skin_color: SkinColor::Preset,
            }
        );
        assert_eq!(
            preset(Preset::Unhide),
            Appearance {
                sleeves: Sleeves::Short,
                tuck: Tuck::Untucked,
                socks: Socks::Standard,
                boots_id: 0,
                gloves_id: 0,
                skin_color: SkinColor::Preset,
            }
        );
        assert_eq!(
            preset(Preset::PartialHide),
            Appearance {
                skin_color: SkinColor::Custom,
                ..preset(Preset::Unhide)
            }
        );
    }

    #[test]
    fn each_finding_fires_on_exactly_its_condition() {
        let quiet = StripStyle::default();
        assert!(check(&quiet, true).is_empty());
        assert!(check(&quiet, false).is_empty());

        // Inners/undershorts/taping only matter on an FPC player.
        let mut style = StripStyle {
            inners_worn: true,
            undershorts_worn: true,
            wrist_taping: true,
            ..quiet
        };
        assert!(check(&style, false).is_empty());
        let codes: Vec<&str> = check(&style, true).iter().map(|f| f.code).collect();
        assert_eq!(
            codes,
            [
                "fpc_inners_break_hiding",
                "fpc_undershorts_break_hiding",
                "fpc_taping_shows_pieces"
            ]
        );

        // Ankle taping alone also trips the taping rule.
        style = StripStyle {
            ankle_taping: true,
            ..quiet
        };
        assert_eq!(check(&style, true)[0].code, "fpc_taping_shows_pieces");

        // Gloves checkbox + id 0 fires for any player; nonzero id does not.
        style = StripStyle {
            gloves_checkbox: true,
            gloves_id: 0,
            ..quiet
        };
        assert_eq!(
            check(&style, false)[0].code,
            "fpc_gloves_checkbox_winter_gloves"
        );
        style.gloves_id = 3;
        assert!(check(&style, false).is_empty());

        // Custom skin warns only on non-FPC players.
        style = StripStyle {
            skin_color: SkinColor::Custom,
            ..quiet
        };
        assert!(check(&style, true).is_empty());
        assert_eq!(check(&style, false)[0].code, "fpc_custom_skin_on_non_fpc");
    }

    #[test]
    fn severity_orders_info_before_warning_before_error() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert_eq!(
            check(
                &StripStyle {
                    inners_worn: true,
                    gloves_checkbox: true,
                    gloves_id: 0,
                    ..StripStyle::default()
                },
                true
            )
            .iter()
            .map(|f| f.severity)
            .collect::<Vec<_>>(),
            [Severity::Error, Severity::Warning]
        );
    }
}
