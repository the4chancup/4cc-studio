//! The per-version caps on the ingame-face type fields
//! (`plans/pes_savefile/operations.md`, "Cross-version player conversion",
//! whose table has PES 15's column). A field
//! that is not a face type (a slider, a colour, the hair, gloves, skin, iris)
//! has no cap: `None`.

use pes_version::PesVersion;

use crate::schema::ingame_face::IngameFaceField;

use IngameFaceField as F;

/// The highest value `field` takes on `version`, `None` for the fields that
/// are not face types. For `NoseType` the cap is on the nose type, which PES
/// 15 codes with the nose angle (`IngameFace::nose`), not on the raw field.
pub fn face_type_cap(version: PesVersion, field: IngameFaceField) -> Option<u8> {
    // (PES 15, PES 16-19, PES 20-21): the plan's table. PES 15's column is
    // its editor's range joined with the values the game's own faces use.
    let (pes15, older, newer) = match field {
        F::PlayerGloves
        | F::PlayerGlovesColor
        | F::Jawline
        | F::JawHeight
        | F::Pes15JawDepth
        | F::Cheekbones
        | F::CheekboneHeight
        | F::Pes15CheekboneDepth
        | F::BrowWidth
        | F::InnerEyebrowHeight
        | F::EyebrowDepth
        | F::TempleWidth
        | F::OuterEyebrowHeight
        | F::Pes15EyebrowDepth
        | F::PupilSize
        | F::EyeDepth
        | F::InnerEyePosition
        | F::InnerEyeHeight
        | F::OuterEyePosition
        | F::EyeCornerHeight
        | F::UpperEyelidWidthInner
        | F::UpperEyelidHeightInner
        | F::UpperEyelidWidthOuter
        | F::UpperEyelidHeightOuter
        | F::Pes15BottomEyelidPosition
        | F::BottomEyelidHeight
        | F::HorizontalEyePosition
        | F::EyeHeight
        | F::NoseHeight
        | F::NoseTipDepth
        | F::NostrilWidth
        | F::Pes15NasalBridgePosition
        | F::NoseDepth
        | F::NoseWidth
        | F::Pes15NasalRootHeight
        | F::MouthHeight
        | F::MouthDepth
        | F::LipSize
        | F::MouthCornerHeight
        | F::LipWidth
        | F::Pes15MouthEdgePosition
        | F::ChinWidth
        | F::ChinHeight
        | F::ChinDepth
        | F::SkinColor
        | F::EarLength
        | F::EarWidth
        | F::EarAngle
        | F::FaceSize
        | F::FaceHeight
        | F::HairColorRed
        | F::HairColorGreen
        | F::HairColorBlue
        | F::Pes15HairVariation
        | F::HairStyle
        | F::HairCroppedStyle
        | F::EyebrowColorRed
        | F::EyebrowStyle
        | F::EyebrowColorGreen
        | F::EyebrowDensity
        | F::EyebrowColorBlue
        | F::EyebrowThickness
        | F::Pes15HairLength
        | F::HairLength
        | F::Pes15HairType
        | F::HairWaveLevel
        | F::Pes15EyeWhite
        | F::HairAccessory
        | F::HairAccessoryColor
        | F::FacialHairColorRed
        | F::FacialHairColorGreen
        | F::FacialHairColorBlue
        | F::FacialHairThickness
        | F::IrisColor
        | F::HairVariation
        | F::HairColorPreset
        | F::HairFrontStyle
        | F::HairParted
        | F::HairSideBackStyle
        | F::Hairline
        | F::HairForeheadWidth => return None,
        F::CheekType => (14, 3, 3),
        F::ForeheadType => (6, 5, 5),
        F::FacialHairType => (29, 12, 19),
        F::LaughterLinesType => (6, 4, 4),
        F::UpperEyelidType => (14, 6, 7),
        F::LowerEyelidType => (14, 2, 6),
        F::EyebrowType => (17, 5, 7),
        F::NeckLineType => (5, 2, 3),
        F::NoseType => (11, 6, 7),
        F::UpperLipType => (7, 3, 4),
        F::LowerLipType => (7, 2, 4),
    };
    match version {
        PesVersion::Pes15 => Some(pes15),
        PesVersion::Pes16 | PesVersion::Pes17 | PesVersion::Pes18 | PesVersion::Pes19 => {
            Some(older)
        }
        PesVersion::Pes20 | PesVersion::Pes21 => Some(newer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use PesVersion as V;

    /// The eleven face-type fields, in `IngameFaceField::ALL` order.
    const TYPES: [IngameFaceField; 11] = [
        F::CheekType,
        F::ForeheadType,
        F::FacialHairType,
        F::LaughterLinesType,
        F::UpperEyelidType,
        F::LowerEyelidType,
        F::EyebrowType,
        F::NeckLineType,
        F::NoseType,
        F::UpperLipType,
        F::LowerLipType,
    ];

    #[test]
    fn the_non_type_fields_have_no_cap_the_eleven_types_do() {
        for version in PesVersion::ALL {
            for field in IngameFaceField::ALL {
                let cap = face_type_cap(version, field);
                if TYPES.contains(&field) {
                    assert!(cap.is_some(), "{version:?} {field:?}");
                } else {
                    assert_eq!(cap, None, "{version:?} {field:?}");
                }
            }
        }
    }

    /// The plan's table, as literals, in `TYPES` order: PES 15's measured
    /// caps, the 19 → 16 converter's (PES 16 column) and the 16 → 21
    /// converter's (PES 21 column).
    const GOLDEN: [(u8, u8, u8); 11] = [
        (14, 3, 3),
        (6, 5, 5),
        (29, 12, 19),
        (6, 4, 4),
        (14, 6, 7),
        (14, 2, 6),
        (17, 5, 7),
        (5, 2, 3),
        (11, 6, 7),
        (7, 3, 4),
        (7, 2, 4),
    ];

    #[test]
    fn the_caps_are_the_plan_s_values() {
        for (field, (pes15, older, newer)) in TYPES.into_iter().zip(GOLDEN) {
            assert_eq!(
                face_type_cap(V::Pes15, field),
                Some(pes15),
                "Pes15 {field:?}"
            );
            for version in [V::Pes16, V::Pes17, V::Pes18, V::Pes19] {
                assert_eq!(
                    face_type_cap(version, field),
                    Some(older),
                    "{version:?} {field:?}"
                );
            }
            for version in [V::Pes20, V::Pes21] {
                assert_eq!(
                    face_type_cap(version, field),
                    Some(newer),
                    "{version:?} {field:?}"
                );
            }
        }
    }

    #[test]
    fn the_newer_caps_never_narrow() {
        for field in TYPES {
            let older = face_type_cap(PesVersion::Pes19, field).expect("a type");
            let newer = face_type_cap(PesVersion::Pes20, field).expect("a type");
            assert!(newer >= older, "{field:?}: {older} -> {newer}");
        }
    }
}
