//! The per-version caps on the ingame-face type fields (plan:
//! `pes_savefile/operations.md`, "Cross-version player conversion"). A field
//! that is not a face type (gloves, skin, iris) has no cap: `None`.

use pes_version::PesVersion;

use crate::schema::ingame_face::IngameFaceField;

use IngameFaceField as F;

/// The highest value `field` takes on `version`, `None` for the fields that
/// are not face types (`PlayerGloves`, `PlayerGlovesColor`, `SkinColor`,
/// `IrisColor`).
pub fn face_type_cap(version: PesVersion, field: IngameFaceField) -> Option<u8> {
    // (PES 15-19, PES 20-21): the plan's table.
    let (older, newer) = match field {
        F::PlayerGloves | F::PlayerGlovesColor | F::SkinColor | F::IrisColor => return None,
        F::CheekType => (3, 3),
        F::ForeheadType => (5, 5),
        F::FacialHairType => (12, 19),
        F::LaughterLinesType => (4, 4),
        F::UpperEyelidType => (6, 7),
        F::LowerEyelidType => (2, 6),
        F::EyebrowType => (5, 7),
        F::NeckLineType => (2, 3),
        F::NoseType => (6, 7),
        F::UpperLipType => (3, 4),
        F::LowerLipType => (2, 4),
    };
    match version {
        PesVersion::Pes15
        | PesVersion::Pes16
        | PesVersion::Pes17
        | PesVersion::Pes18
        | PesVersion::Pes19 => Some(older),
        PesVersion::Pes20 | PesVersion::Pes21 => Some(newer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_four_non_type_fields_have_no_cap_the_eleven_types_do() {
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

    #[test]
    fn the_newer_caps_never_narrow() {
        for field in TYPES {
            let older = face_type_cap(PesVersion::Pes19, field).expect("a type");
            let newer = face_type_cap(PesVersion::Pes20, field).expect("a type");
            assert!(newer >= older, "{field:?}: {older} -> {newer}");
        }
    }
}
