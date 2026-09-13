//! Kit config codec: the 120-byte binary, its TOML form, validation, and the
//! FPC kit values (which live in the `fpc` crate).
//!
//! Decoding keeps every bit not yet decoded in [`KitConfig::unknown`] so real
//! configs round-trip bit-identically; the TOML form carries plain,
//! version-neutral values and the binary packing is applied on emission.

mod binary;
mod fpc;
mod model;
mod names;
mod toml_form;
mod validate;

pub use fpc::{apply_fpc, matches_fpc};
pub use model::{
    BackNumber, Badges, ChestNumber, Colors, FieldLimit, KitConfig, LongSleeves, NameShape,
    NameText, Numbers, Position, Rgb, Shirt, ShortSleeves, Shorts, ShortsNumber, Side,
    TEXTURE_NAME_FIELDS, field_limits,
};
pub use names::{TexturePresence, texture_names};
pub use validate::{Finding, OutOfRange, Severity, validate};

use pes_version::PesVersion;

impl KitConfig {
    /// The template every generated config starts from (the bundled template
    /// binary, decoded for PES 21 at first use).
    pub fn template() -> KitConfig {
        binary::template()
    }

    /// Decodes a kit config; WESYS-wrapped input is unwrapped first and the
    /// result must be exactly 120 bytes.
    pub fn decode(bytes: &[u8], version: PesVersion) -> Result<KitConfig, KitConfigError> {
        binary::decode(bytes, version)
    }

    /// Encodes the config, taking texture names from `source_texture_names`
    /// or zeros when `None`.
    pub fn encode(&self, version: PesVersion) -> [u8; 120] {
        binary::encode(self, version)
    }

    /// Encodes the config with the given five texture-name fields.
    pub fn encode_with_names(&self, version: PesVersion, names: &[[u8; 16]; 5]) -> [u8; 120] {
        binary::encode_with_names(self, version, names)
    }

    /// Builds a config from TOML text; missing sections and keys take the
    /// template's values.
    pub fn from_toml(text: &str) -> Result<KitConfig, KitConfigError> {
        toml_form::from_toml(text)
    }

    /// A fresh TOML document with the predefined per-field comments.
    pub fn to_toml(&self) -> String {
        toml_form::to_toml(self)
    }

    /// Sets every value in an existing document, preserving its comments and
    /// formatting.
    pub fn update_toml(&self, document: &mut toml_edit::DocumentMut) {
        toml_form::update_toml(self, document)
    }
}

/// Why binary or TOML input is not a kit config.
#[derive(Debug, thiserror::Error)]
pub enum KitConfigError {
    /// The unwrapped binary is not 120 bytes.
    #[error("kit config is {0} bytes, expected 120")]
    WrongLength(usize),
    /// The WESYS wrapper could not be unwrapped.
    #[error("wesys unwrap failed: {0}")]
    Wesys(#[from] wezlib::Error),
    /// The TOML text does not parse.
    #[error("invalid toml: {0}")]
    Toml(#[from] toml::de::Error),
    /// A key carries a value of the wrong type or outside its encoding.
    #[error("invalid value for {key}: {value}")]
    InvalidValue {
        /// The TOML key.
        key: &'static str,
        /// The offending value.
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE: &[u8] = include_bytes!("../tests/fixtures/template_XXX_DEF_xxx_realUni.bin");
    const GK: &[u8] = include_bytes!("../tests/fixtures/konami_pes21_100_DEF_GK1st_realUni.bin");
    const MODEL144: &[u8] =
        include_bytes!("../tests/fixtures/konami_pes21_model144_100_DEF_1st_realUni.bin");
    const REFEREE: &[u8] = include_bytes!("../tests/fixtures/konami_pes21_referee_EU_1.bin");
    const UNIPARAM: &[u8] =
        include_bytes!("../../uniparam/tests/fixtures/konami_pes21_UniformParameter.bin");

    const FIXTURES: [&[u8]; 4] = [TEMPLATE, GK, MODEL144, REFEREE];

    #[test]
    fn binary_round_trips_bit_identically() {
        for bytes in FIXTURES {
            let config = KitConfig::decode(bytes, PesVersion::Pes21).unwrap();
            assert_eq!(config.encode(PesVersion::Pes21), bytes);
        }
    }

    #[test]
    fn every_stock_config_round_trips_and_stays_in_range() {
        let container = uniparam::UniformParameter::read(UNIPARAM).unwrap();
        let mut checked = 0;
        for (name, content) in container.entries() {
            if content.len() != 120 {
                continue;
            }
            let config = KitConfig::decode(content, PesVersion::Pes21)
                .unwrap_or_else(|_| panic!("decode {name}"));
            assert_eq!(
                config.encode(PesVersion::Pes21),
                content,
                "round trip {name}"
            );
            checked += 1;

            assert!(
                matches!(config.shirt.model, 144 | 160 | 176),
                "shirt model {name}"
            );
            assert!(
                matches!(
                    config.shirt.long_sleeves,
                    LongSleeves::Normal | LongSleeves::UndershirtOnly
                ),
                "long sleeves {name}"
            );

            // Every stock config validates clean except the 13 referee ones
            // carrying short-sleeves value 3, which earn exactly one Info.
            let findings = validate(&config, PesVersion::Pes21);
            assert!(
                findings.is_empty()
                    || findings
                        == [Finding {
                            code: "kit_unknown_sleeve_value",
                            severity: Severity::Info,
                            context: None,
                        }],
                "findings for {name}: {findings:?}"
            );
        }
        assert_eq!(checked, 1372);
    }

    #[test]
    fn template_decodes_and_encodes_identically() {
        let template = KitConfig::template();
        assert_eq!(template.shirt.model, 176);
        assert_eq!(template.shirt.long_sleeves, LongSleeves::Normal);
        assert_eq!(template.shorts.model, 16);
        assert_eq!(template.shirt.collar, 105);
        assert_eq!(template.shirt.winter_collar, 105);
        assert_eq!(template.encode(PesVersion::Pes21), TEMPLATE);
    }

    #[test]
    fn toml_round_trip_carries_everything() {
        assert!(!KitConfig::template().to_toml().contains("[unknown]"));
        assert_eq!(KitConfig::from_toml("").unwrap(), KitConfig::template());
        for bytes in FIXTURES {
            let config = KitConfig::decode(bytes, PesVersion::Pes21).unwrap();
            let parsed = KitConfig::from_toml(&config.to_toml()).unwrap();
            assert_eq!(parsed, config);
        }
        // A minimal document fills the rest from the template.
        let minimal = KitConfig::from_toml("[shirt]\nmodel = 144\n").unwrap();
        let mut expected = KitConfig::template();
        expected.shirt.model = 144;
        assert_eq!(minimal, expected);
    }

    #[test]
    fn update_toml_preserves_comments() {
        let text = "[shirt]\nmodel = 144\n# my collar note\ncollar = 4 # inline\n";
        let mut document = text.parse::<toml_edit::DocumentMut>().unwrap();
        let mut config = KitConfig::template();
        config.shirt.model = 160;
        config.shirt.collar = 9;
        config.update_toml(&mut document);
        let out = document.to_string();
        assert!(out.contains("# my collar note"), "{out}");
        assert!(out.contains("collar = 9"), "{out}");
        assert!(out.contains("model = 160"), "{out}");
        // Everything set round-trips back to the same config.
        assert_eq!(KitConfig::from_toml(&out).unwrap(), config);
    }

    #[test]
    fn texture_names_derive_from_team_slot_and_presence() {
        let names = texture_names(
            701,
            "p1",
            TexturePresence {
                kit: true,
                back: true,
                ..TexturePresence::default()
            },
        );
        assert_eq!(&names[0][..7], b"u0701p1");
        assert_eq!(&names[1][..12], b"u0701p1_back");
        assert_eq!(names[2], [0u8; 16]);
        assert_eq!(names[3], [0u8; 16]);
        assert_eq!(names[4], [0u8; 16]);
    }

    #[test]
    fn validation_rules() {
        let mut config = KitConfig::template();
        config.shirt.short_sleeves = ShortSleeves::CutOut; // model 176
        assert!(
            validate(&config, PesVersion::Pes21)
                .iter()
                .any(|f| f.code == "kit_cut_out_requires_model_144_or_160"
                    && f.severity == Severity::Warning)
        );
        // Model 160 carries cut-out in stock data: no finding.
        config.shirt.model = 160;
        assert!(validate(&config, PesVersion::Pes21).is_empty());

        config = KitConfig::template();
        config.name.y = 30;
        // PES <= 20's Name Y field holds 0-16: one finding naming the field,
        // and the emitted bytes decode to the clamped 16.
        assert_eq!(
            validate(&config, PesVersion::Pes20),
            [Finding {
                code: "kit_value_out_of_range",
                severity: Severity::Warning,
                context: Some(OutOfRange {
                    field: "name.y",
                    value: 30,
                    max: 16,
                }),
            }]
        );
        let decoded =
            KitConfig::decode(&config.encode(PesVersion::Pes20), PesVersion::Pes20).unwrap();
        assert_eq!(decoded.name.y, 16);
        // PES 21's field holds 0-39: no finding, 30 emitted as is.
        assert!(validate(&config, PesVersion::Pes21).is_empty());
        let decoded =
            KitConfig::decode(&config.encode(PesVersion::Pes21), PesVersion::Pes21).unwrap();
        assert_eq!(decoded.name.y, 30);

        // Every packed field goes through the same table.
        let mut config = KitConfig::template();
        config.numbers.back.size = 255;
        assert!(validate(&config, PesVersion::Pes21).iter().any(|f| f.code
            == "kit_value_out_of_range"
            && f.context
                == Some(OutOfRange {
                    field: "number.back.size",
                    value: 255,
                    max: 15,
                })));
        let decoded =
            KitConfig::decode(&config.encode(PesVersion::Pes21), PesVersion::Pes21).unwrap();
        assert_eq!(decoded.numbers.back.size, 15);
    }

    #[test]
    fn pes15_pattern_remaps_and_warns() {
        let mut config = KitConfig::template();
        // The template preserves the byte's undecoded low nibble; drop it so
        // the emitted byte shows the pattern field alone. Its name.y = 30 is
        // out of range on PES 15, which is not what this test checks.
        config.unknown.remove(&0x24);
        config.name.y = 10;
        let pattern_byte = |config: &KitConfig, version: PesVersion| config.encode(version)[0x24];

        config.shirt.pattern = 6;
        assert_eq!(pattern_byte(&config, PesVersion::Pes15), 0x60);
        assert!(validate(&config, PesVersion::Pes15).is_empty());

        config.shirt.pattern = 12;
        assert_eq!(pattern_byte(&config, PesVersion::Pes15), 0xA0);
        assert!(validate(&config, PesVersion::Pes15).iter().any(|f| f.code
            == "kit_pattern_unsupported_pes15"
            && f.severity == Severity::Warning
            && f.context
                == Some(OutOfRange {
                    field: "shirt.pattern",
                    value: 12,
                    max: 11,
                })));

        config.shirt.pattern = 13;
        assert_eq!(pattern_byte(&config, PesVersion::Pes15), 0xB0);

        config.shirt.pattern = 14;
        assert_eq!(pattern_byte(&config, PesVersion::Pes15), 0xE0);
        assert!(
            validate(&config, PesVersion::Pes15)
                .iter()
                .any(|f| f.code == "kit_pattern_unsupported_pes15")
        );

        // Later versions read the full 4-bit field: no remap, no finding.
        config.shirt.pattern = 12;
        assert_eq!(pattern_byte(&config, PesVersion::Pes16), 0xC0);
        assert!(
            !validate(&config, PesVersion::Pes16)
                .iter()
                .any(|f| f.code == "kit_pattern_unsupported_pes15")
        );
    }

    #[test]
    fn wrong_typed_table_is_an_error() {
        assert!(matches!(
            KitConfig::from_toml("shirt = 144"),
            Err(KitConfigError::InvalidValue { key: "shirt", .. })
        ));
        assert!(matches!(
            KitConfig::from_toml("[number]\nback = 12"),
            Err(KitConfigError::InvalidValue {
                key: "number.back",
                ..
            })
        ));
    }

    #[test]
    fn non_ascii_hex_is_an_error_not_a_panic() {
        assert!(matches!(
            KitConfig::from_toml("[colors]\nshirt1 = \"#１２\""),
            Err(KitConfigError::InvalidValue { .. })
        ));
        assert!(matches!(
            KitConfig::from_toml("[source_texture_names]\nkit = \"１２３４５６７８１２01\""),
            Err(KitConfigError::InvalidValue { .. })
        ));
        let config = KitConfig::from_toml("[colors]\nshirt1 = \"#0a0B0c\"").unwrap();
        assert_eq!(config.colors.shirt1, Rgb(10, 11, 12));
    }

    #[test]
    fn fpc_apply_and_match() {
        let mut config = KitConfig::decode(GK, PesVersion::Pes21).unwrap();
        assert!(!matches_fpc(&config, PesVersion::Pes21));
        assert!(apply_fpc(&mut config, PesVersion::Pes21));
        assert!(matches_fpc(&config, PesVersion::Pes21));

        let mut config = KitConfig::template();
        assert!(!apply_fpc(&mut config, PesVersion::Pes18));
        assert!(!matches_fpc(&config, PesVersion::Pes18));
    }
}
