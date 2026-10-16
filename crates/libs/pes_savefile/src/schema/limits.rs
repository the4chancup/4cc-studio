//! The per-version caps on the ingame-face type fields
//! (`spec/pes_savefile/operations.md`, "Cross-version player conversion",
//! whose table has PES 15's column) and the per-version tops of the motion
//! fields (`spec/aesthetics_export/settings_toml.md` "Player settings in
//! exports", the measured table). A field
//! that is not a face type (a slider, a colour, the hair, gloves, skin, iris)
//! has no cap, and a player field that is not a measured motion has no top:
//! `None`.

use pes_version::PesVersion;

use crate::schema::fields::PlayerField;
use crate::schema::ingame_face::IngameFaceField;

use IngameFaceField as F;

/// The highest value `field` takes on `version`, `None` for the fields that
/// are not face types. For `NoseType` the cap is on the nose type, which PES
/// 15 codes with the nose angle (`IngameFace::nose`), not on the raw field.
pub fn face_type_cap(version: PesVersion, field: IngameFaceField) -> Option<u8> {
    // (PES 15, PES 16-18, PES 19, PES 20-21): the plan's table. PES 15's
    // column is its editor's range joined with the values the game's own
    // faces use; PES 19's cheek and forehead are measured in its editor,
    // its other nine types assumed equal to 16's.
    let (pes15, older, pes19, newer) = match field {
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
        F::CheekType => (14, 3, 5, 3),
        F::ForeheadType => (6, 5, 6, 5),
        F::FacialHairType => (29, 12, 12, 19),
        F::LaughterLinesType => (6, 4, 4, 4),
        F::UpperEyelidType => (14, 6, 6, 7),
        F::LowerEyelidType => (14, 2, 2, 6),
        F::EyebrowType => (17, 5, 5, 7),
        F::NeckLineType => (5, 2, 2, 3),
        F::NoseType => (11, 6, 6, 7),
        F::UpperLipType => (7, 3, 3, 4),
        F::LowerLipType => (7, 2, 2, 4),
    };
    match version {
        PesVersion::Pes15 => Some(pes15),
        PesVersion::Pes16 | PesVersion::Pes17 | PesVersion::Pes18 => Some(older),
        PesVersion::Pes19 => Some(pes19),
        PesVersion::Pes20 | PesVersion::Pes21 => Some(newer),
    }
}

/// The highest stored value of motion field `field` on `version`, `None` for the fields that
/// are not motions with a measured range (`Dribbling` included). The 1-based motions store
/// one below what the editor shows; the goal celebrations are stored as shown, 0 being OFF.
pub fn motion_top(version: PesVersion, field: PlayerField) -> Option<u8> {
    // (PES 15, PES 16, PES 17, PES 18, PES 19, PES 20, PES 21): the plan's
    // measured table, in stored values.
    let (pes15, pes16, pes17, pes18, pes19, pes20, pes21) = match field {
        PlayerField::HunchingDribbling | PlayerField::HunchingRunning => (2, 2, 2, 2, 2, 4, 4),
        PlayerField::ArmMovementDribbling | PlayerField::ArmMovementRunning => {
            (4, 7, 7, 7, 7, 9, 9)
        }
        PlayerField::CornerKickMotion => (5, 5, 5, 5, 5, 9, 9),
        PlayerField::FreeKickMotion => (15, 15, 15, 15, 15, 19, 19),
        PlayerField::PenaltyKickMotion => (3, 3, 3, 3, 3, 6, 6),
        PlayerField::GoalCelebration1 | PlayerField::GoalCelebration2 => {
            (112, 122, 122, 127, 140, 152, 162)
        }
        PlayerField::Id
        | PlayerField::Nationality
        | PlayerField::Height
        | PlayerField::Weight
        | PlayerField::AttackingProwess
        | PlayerField::DefensiveProwess
        | PlayerField::Goalkeeping
        | PlayerField::Dribbling
        | PlayerField::Finishing
        | PlayerField::LowPass
        | PlayerField::LoftedPass
        | PlayerField::Heading
        | PlayerField::Form
        | PlayerField::EditedPlayer
        | PlayerField::Swerve
        | PlayerField::Catching
        | PlayerField::Clearing
        | PlayerField::Reflexes
        | PlayerField::InjuryResistance
        | PlayerField::EditedBasicSettings
        | PlayerField::BodyControl
        | PlayerField::PhysicalContact
        | PlayerField::KickingPower
        | PlayerField::ExplosivePower
        | PlayerField::EditedRegisteredPosition
        | PlayerField::Age
        | PlayerField::RegisteredPosition
        | PlayerField::PlayingStyle
        | PlayerField::BallControl
        | PlayerField::BallWinning
        | PlayerField::WeakFootAccuracy
        | PlayerField::Jump
        | PlayerField::Coverage
        | PlayerField::WeakFootUsage
        | PlayerField::PlayablePosition(_)
        | PlayerField::PlaceKicking
        | PlayerField::EditedPlayablePositions
        | PlayerField::EditedAbilities
        | PlayerField::EditedSkills
        | PlayerField::Stamina
        | PlayerField::Speed
        | PlayerField::EditedPlayingStyle
        | PlayerField::EditedComStyles
        | PlayerField::EditedMotion
        | PlayerField::BaseCopy
        | PlayerField::StrongerFoot
        | PlayerField::ComStyle(_)
        | PlayerField::Skill(_)
        | PlayerField::Star
        | PlayerField::DribblingMotion
        | PlayerField::TightPossession
        | PlayerField::Aggression
        | PlayerField::PlayingAttitude
        | PlayerField::StrongerHand
        | PlayerField::AppearanceId
        | PlayerField::EditedFace
        | PlayerField::EditedHair
        | PlayerField::EditedPhysique
        | PlayerField::EditedStrip
        | PlayerField::BootsId
        | PlayerField::GlovesId
        | PlayerField::BaseCopyId
        | PlayerField::NeckLength
        | PlayerField::NeckSize
        | PlayerField::ShoulderHeight
        | PlayerField::ShoulderWidth
        | PlayerField::Chest
        | PlayerField::Waist
        | PlayerField::ArmSize
        | PlayerField::ArmLength
        | PlayerField::Thigh
        | PlayerField::Calf
        | PlayerField::LegLength
        | PlayerField::HeadLength
        | PlayerField::HeadWidth
        | PlayerField::HeadDepth
        | PlayerField::WristTapeColorLeft
        | PlayerField::WristTapeColorRight
        | PlayerField::WristTaping
        | PlayerField::SpectaclesColor
        | PlayerField::SpectaclesStyle
        | PlayerField::Sleeves
        | PlayerField::Inners
        | PlayerField::Socks
        | PlayerField::Undershorts
        | PlayerField::Untucked
        | PlayerField::AnkleTaping => return None,
    };
    match version {
        PesVersion::Pes15 => Some(pes15),
        PesVersion::Pes16 => Some(pes16),
        PesVersion::Pes17 => Some(pes17),
        PesVersion::Pes18 => Some(pes18),
        PesVersion::Pes19 => Some(pes19),
        PesVersion::Pes20 => Some(pes20),
        PesVersion::Pes21 => Some(pes21),
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
    /// caps, PES 16 to 18's, PES 19's own (cheek and forehead measured in
    /// its editor) and PES 20-21's.
    const GOLDEN: [(u8, u8, u8, u8); 11] = [
        (14, 3, 5, 3),
        (6, 5, 6, 5),
        (29, 12, 12, 19),
        (6, 4, 4, 4),
        (14, 6, 6, 7),
        (14, 2, 2, 6),
        (17, 5, 5, 7),
        (5, 2, 2, 3),
        (11, 6, 6, 7),
        (7, 3, 3, 4),
        (7, 2, 2, 4),
    ];

    #[test]
    fn the_caps_are_the_plan_s_values() {
        for (field, (pes15, older, pes19, newer)) in TYPES.into_iter().zip(GOLDEN) {
            assert_eq!(
                face_type_cap(V::Pes15, field),
                Some(pes15),
                "Pes15 {field:?}"
            );
            for version in [V::Pes16, V::Pes17, V::Pes18] {
                assert_eq!(
                    face_type_cap(version, field),
                    Some(older),
                    "{version:?} {field:?}"
                );
            }
            assert_eq!(
                face_type_cap(V::Pes19, field),
                Some(pes19),
                "Pes19 {field:?}"
            );
            for version in [V::Pes20, V::Pes21] {
                assert_eq!(
                    face_type_cap(version, field),
                    Some(newer),
                    "{version:?} {field:?}"
                );
            }
        }
    }

    /// PES 19's own column: its editor offers cheek to 5 and forehead to 6,
    /// wider than both neighbours'.
    #[test]
    fn pes19s_measured_cheek_and_forehead() {
        assert_eq!(face_type_cap(V::Pes19, F::CheekType), Some(5));
        assert_eq!(face_type_cap(V::Pes19, F::ForeheadType), Some(6));
        assert_eq!(face_type_cap(V::Pes18, F::CheekType), Some(3));
        assert_eq!(face_type_cap(V::Pes18, F::ForeheadType), Some(5));
    }

    /// The nine motion fields a measured range was taken for, in
    /// `settings.toml`'s `[appearance.motion]` order (`dribbling`, the same
    /// 0 to 3 on both versions that hold it, is not one).
    const MOTIONS: [PlayerField; 9] = [
        PlayerField::HunchingDribbling,
        PlayerField::HunchingRunning,
        PlayerField::ArmMovementDribbling,
        PlayerField::ArmMovementRunning,
        PlayerField::CornerKickMotion,
        PlayerField::FreeKickMotion,
        PlayerField::PenaltyKickMotion,
        PlayerField::GoalCelebration1,
        PlayerField::GoalCelebration2,
    ];

    /// The plan's measured tops as the editor shows them
    /// (`spec/aesthetics_export/settings_toml.md` "Player settings in
    /// exports"), one row per `MOTIONS` field, one column per version of
    /// `PesVersion::ALL`.
    const SHOWN_TOPS: [[u8; 7]; 9] = [
        [3, 3, 3, 3, 3, 5, 5],
        [3, 3, 3, 3, 3, 5, 5],
        [5, 8, 8, 8, 8, 10, 10],
        [5, 8, 8, 8, 8, 10, 10],
        [6, 6, 6, 6, 6, 10, 10],
        [16, 16, 16, 16, 16, 20, 20],
        [4, 4, 4, 4, 4, 7, 7],
        [112, 122, 122, 127, 140, 152, 162],
        [112, 122, 122, 127, 140, 152, 162],
    ];

    #[test]
    fn the_motion_tops_are_the_plan_s_table_in_stored_values() {
        for (field, shown) in MOTIONS.into_iter().zip(SHOWN_TOPS) {
            // The seven 1-based motions show stored + 1; the two
            // celebrations are stored as shown.
            let one_based = !matches!(
                field,
                PlayerField::GoalCelebration1 | PlayerField::GoalCelebration2
            );
            for (version, top) in PesVersion::ALL.into_iter().zip(shown) {
                let stored = if one_based { top - 1 } else { top };
                assert_eq!(
                    motion_top(version, field),
                    Some(stored),
                    "{version:?} {field:?}"
                );
            }
        }
    }

    /// PES 19 is wider than 20 and 21 on the two types measured in its
    /// editor (cheek, forehead), so the never-narrow check runs 18 to 20.
    #[test]
    fn the_newer_caps_never_narrow() {
        for field in TYPES {
            let older = face_type_cap(PesVersion::Pes18, field).expect("a type");
            let newer = face_type_cap(PesVersion::Pes20, field).expect("a type");
            assert!(newer >= older, "{field:?}: {older} -> {newer}");
        }
    }

    #[test]
    fn an_unmeasured_motion_and_a_non_motion_have_no_top() {
        for version in PesVersion::ALL {
            for field in [
                PlayerField::DribblingMotion,
                PlayerField::Dribbling,
                PlayerField::Height,
            ] {
                assert_eq!(motion_top(version, field), None, "{version:?} {field:?}");
            }
        }
    }
}
