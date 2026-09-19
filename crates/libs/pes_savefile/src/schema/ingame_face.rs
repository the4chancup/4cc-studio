//! The known bit runs inside the ingame-face run: the fifteen the legacy tools
//! read or write, as `FieldSpec<IngameFaceField>` rows (the one place these
//! offsets appear). The offsets are the appearance block's bytes 22 onward,
//! relative to the run's first byte, and come from the two legacy converters'
//! write walks and the reference save editor's reads. Hand-written, not
//! generated: the run is opaque bytes no read walk decodes as a whole.

use super::FieldSpec;

/// A known bit run inside the ingame-face run; the fifteen the legacy tools
/// read or write. Offsets are relative to the run's first byte (byte 22 of the
/// appearance block).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngameFaceField {
    /// Player (outfield) gloves, byte 0 bit 0.
    PlayerGloves,
    /// Player gloves colour, byte 0 bit 1.
    PlayerGlovesColor,
    /// Skin colour, byte 23 bit 0.
    SkinColor,
    /// Cheek type, byte 23 bit 3.
    CheekType,
    /// Forehead type, byte 24 bit 0.
    ForeheadType,
    /// Facial hair type, byte 24 bit 3.
    FacialHairType,
    /// Laughter lines type, byte 25 bit 0.
    LaughterLinesType,
    /// Upper eyelid type, byte 25 bit 3.
    UpperEyelidType,
    /// Lower eyelid type, byte 26 bit 0.
    LowerEyelidType,
    /// Eyebrow type, byte 28 bit 0.
    EyebrowType,
    /// Neck line type, byte 28 bit 5.
    NeckLineType,
    /// Nose type, byte 30 bit 0.
    NoseType,
    /// Upper lip type, byte 31 bit 0.
    UpperLipType,
    /// Lower lip type, byte 31 bit 3.
    LowerLipType,
    /// Iris colour, byte 42 bit 0.
    IrisColor,
}

impl IngameFaceField {
    /// Every variant, so tests can walk the closed set.
    pub const ALL: [IngameFaceField; 15] = [
        IngameFaceField::PlayerGloves,
        IngameFaceField::PlayerGlovesColor,
        IngameFaceField::SkinColor,
        IngameFaceField::CheekType,
        IngameFaceField::ForeheadType,
        IngameFaceField::FacialHairType,
        IngameFaceField::LaughterLinesType,
        IngameFaceField::UpperEyelidType,
        IngameFaceField::LowerEyelidType,
        IngameFaceField::EyebrowType,
        IngameFaceField::NeckLineType,
        IngameFaceField::NoseType,
        IngameFaceField::UpperLipType,
        IngameFaceField::LowerLipType,
        IngameFaceField::IrisColor,
    ];
}

/// The fifteen known fields of the ingame-face run, identical on every version
/// because the appearance block's internal layout is.
pub const INGAME_FACE_FIELDS: &[FieldSpec<IngameFaceField>] = &[
    FieldSpec {
        field: IngameFaceField::PlayerGloves,
        bit_offset: 0,
        bit_width: 1,
    },
    FieldSpec {
        field: IngameFaceField::PlayerGlovesColor,
        bit_offset: 1,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::SkinColor,
        bit_offset: 184,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::CheekType,
        bit_offset: 187,
        bit_width: 5,
    },
    FieldSpec {
        field: IngameFaceField::ForeheadType,
        bit_offset: 192,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::FacialHairType,
        bit_offset: 195,
        bit_width: 5,
    },
    FieldSpec {
        field: IngameFaceField::LaughterLinesType,
        bit_offset: 200,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::UpperEyelidType,
        bit_offset: 203,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::LowerEyelidType,
        bit_offset: 208,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::EyebrowType,
        bit_offset: 224,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::NeckLineType,
        bit_offset: 229,
        bit_width: 2,
    },
    FieldSpec {
        field: IngameFaceField::NoseType,
        bit_offset: 240,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::UpperLipType,
        bit_offset: 248,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::LowerLipType,
        bit_offset: 251,
        bit_width: 3,
    },
    FieldSpec {
        field: IngameFaceField::IrisColor,
        bit_offset: 336,
        bit_width: 4,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_covers_every_variant_exactly_once() {
        assert_eq!(INGAME_FACE_FIELDS.len(), IngameFaceField::ALL.len());
        for field in IngameFaceField::ALL {
            assert_eq!(
                INGAME_FACE_FIELDS
                    .iter()
                    .filter(|s| s.field == field)
                    .count(),
                1,
                "{field:?} has not exactly one row"
            );
        }
        for spec in INGAME_FACE_FIELDS {
            assert!(
                IngameFaceField::ALL.contains(&spec.field),
                "{:?} is a row outside ALL",
                spec.field
            );
        }
    }

    #[test]
    fn the_fifteen_runs_do_not_overlap_and_fit_the_shortest_run() {
        let mut runs: Vec<(u32, u32)> = INGAME_FACE_FIELDS
            .iter()
            .map(|f| (f.bit_offset, f.bit_offset + f.bit_width))
            .collect();
        runs.sort_unstable();
        for pair in runs.windows(2) {
            assert!(pair[0].1 <= pair[1].0, "runs {pair:?} overlap");
        }
        for &(start, end) in &runs {
            assert!(end <= 46 * 8, "run {start}..{end} past the PES 15 run");
        }
    }
}
