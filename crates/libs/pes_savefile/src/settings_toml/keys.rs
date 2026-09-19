//! The settings.toml key table: one `SettingKey` per key of the aesthetics
//! export plan's "Player settings in exports" block, in the block's order,
//! with the block's tables, ranges, label lists and comments reproduced
//! literally.

#[cfg(test)]
use crate::schema::fields::PlayerField;
#[cfg(test)]
use crate::schema::ingame_face::IngameFaceField;

/// One TOML key of the table in the aesthetics export plan ("Player settings
/// in exports"), in the table's order. `ALL` lists them in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingKey {
    /// `[appearance] skin_color`
    SkinColor,
    /// `[appearance] iris_color`
    IrisColor,
    /// `[appearance.physique] height`
    Height,
    /// `[appearance.physique] weight`
    Weight,
    /// `[appearance.physique] neck_length`
    NeckLength,
    /// `[appearance.physique] neck_size`
    NeckSize,
    /// `[appearance.physique] shoulder_height`
    ShoulderHeight,
    /// `[appearance.physique] shoulder_width`
    ShoulderWidth,
    /// `[appearance.physique] chest`
    Chest,
    /// `[appearance.physique] waist`
    Waist,
    /// `[appearance.physique] arm_size`
    ArmSize,
    /// `[appearance.physique] arm_length`
    ArmLength,
    /// `[appearance.physique] thigh`
    Thigh,
    /// `[appearance.physique] calf`
    Calf,
    /// `[appearance.physique] leg_length`
    LegLength,
    /// `[appearance.physique] head_length`
    HeadLength,
    /// `[appearance.physique] head_width`
    HeadWidth,
    /// `[appearance.physique] head_depth`
    HeadDepth,
    /// `[appearance.strip] sleeves`
    Sleeves,
    /// `[appearance.strip] inners`
    Inners,
    /// `[appearance.strip] socks`
    Socks,
    /// `[appearance.strip] undershorts`
    Undershorts,
    /// `[appearance.strip] untucked`
    Untucked,
    /// `[appearance.strip] ankle_taping`
    AnkleTaping,
    /// `[appearance.strip] wrist_taping`
    WristTaping,
    /// `[appearance.strip] wrist_tape_color_left`
    WristTapeColorLeft,
    /// `[appearance.strip] wrist_tape_color_right`
    WristTapeColorRight,
    /// `[appearance.strip] spectacles`
    Spectacles,
    /// `[appearance.strip] spectacles_color`
    SpectaclesColor,
    /// `[appearance.strip] gloves`
    Gloves,
    /// `[appearance.strip] gloves_color`
    GlovesColor,
    /// `[appearance.motion] hunching_dribbling`
    HunchingDribbling,
    /// `[appearance.motion] hunching_running`
    HunchingRunning,
    /// `[appearance.motion] arm_movement_dribbling`
    ArmMovementDribbling,
    /// `[appearance.motion] arm_movement_running`
    ArmMovementRunning,
    /// `[appearance.motion] corner_kick`
    CornerKick,
    /// `[appearance.motion] free_kick`
    FreeKick,
    /// `[appearance.motion] penalty_kick`
    PenaltyKick,
    /// `[appearance.motion] dribbling` (PES 20+)
    Dribbling,
    /// `[appearance.motion] goal_celebration_1`
    GoalCelebration1,
    /// `[appearance.motion] goal_celebration_2`
    GoalCelebration2,
    /// `[appearance.face] cheek_type`
    CheekType,
    /// `[appearance.face] forehead_type`
    ForeheadType,
    /// `[appearance.face] facial_hair_type`
    FacialHairType,
    /// `[appearance.face] laughter_lines_type`
    LaughterLinesType,
    /// `[appearance.face] upper_eyelid_type`
    UpperEyelidType,
    /// `[appearance.face] lower_eyelid_type`
    LowerEyelidType,
    /// `[appearance.face] eyebrow_type`
    EyebrowType,
    /// `[appearance.face] neck_line_type`
    NeckLineType,
    /// `[appearance.face] nose_type`
    NoseType,
    /// `[appearance.face] upper_lip_type`
    UpperLipType,
    /// `[appearance.face] lower_lip_type`
    LowerLipType,
}

/// How a TOML value maps to the stored `u8`, and the widest accepted range
/// (a PES 16 file may legitimately be compiled for PES 21; the per-version
/// narrowing is the compiler's finding at compile time).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Stored as written: colours, celebrations, face types, height, weight.
    Number {
        /// Smallest accepted value.
        min: u8,
        /// Largest accepted value.
        max: u8,
    },
    /// Physique: TOML -7..=7, stored value + 7.
    Signed7,
    /// Motion: TOML 1..=max, stored value - 1.
    OneBased {
        /// Largest accepted value.
        max: u8,
    },
    /// Stored 0/1.
    Bool,
    /// Stored value = index into the list.
    Labels(&'static [&'static str]),
}

impl Kind {
    /// Whether a stored `u8` is one the kind can represent. A hostile save
    /// can hold a value outside it (a 2-bit label field reading 3); the
    /// parser never produces one and `from_player` refuses it.
    pub fn accepts_stored(self, stored: u8) -> bool {
        match self {
            Kind::Number { min, max } => (min..=max).contains(&stored),
            Kind::Signed7 => stored <= 14,
            Kind::OneBased { max } => stored < max,
            Kind::Bool => stored <= 1,
            Kind::Labels(labels) => usize::from(stored) < labels.len(),
        }
    }
}

/// The columns of one key-table row.
#[derive(Debug, Clone, Copy)]
pub struct KeySpec {
    /// The TOML table ("appearance", "appearance.physique", "appearance.strip",
    /// "appearance.motion", "appearance.face").
    pub table: &'static str,
    /// The TOML key name.
    pub name: &'static str,
    /// How the TOML value maps to the stored `u8`.
    pub kind: Kind,
    /// The range comment the template emits after the key, verbatim from the
    /// plan block.
    pub comment: &'static str,
}

/// Which savefile field a key writes. Test-side only so far: the completeness
/// test is its consumer until the parse/emit slice uses it.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Source {
    /// A `PlayerField` of the player or appearance record.
    Player(PlayerField),
    /// A bit run inside the ingame-face run.
    Face(IngameFaceField),
}

const PHYSIQUE: &str = "appearance.physique";
const STRIP: &str = "appearance.strip";
const MOTION: &str = "appearance.motion";
const FACE: &str = "appearance.face";
const APPEARANCE: &str = "appearance";
const SEVEN: &str = "-7 to 7";

impl SettingKey {
    /// Every key, in the plan table's order.
    pub const ALL: [SettingKey; 52] = [
        SettingKey::SkinColor,
        SettingKey::IrisColor,
        SettingKey::Height,
        SettingKey::Weight,
        SettingKey::NeckLength,
        SettingKey::NeckSize,
        SettingKey::ShoulderHeight,
        SettingKey::ShoulderWidth,
        SettingKey::Chest,
        SettingKey::Waist,
        SettingKey::ArmSize,
        SettingKey::ArmLength,
        SettingKey::Thigh,
        SettingKey::Calf,
        SettingKey::LegLength,
        SettingKey::HeadLength,
        SettingKey::HeadWidth,
        SettingKey::HeadDepth,
        SettingKey::Sleeves,
        SettingKey::Inners,
        SettingKey::Socks,
        SettingKey::Undershorts,
        SettingKey::Untucked,
        SettingKey::AnkleTaping,
        SettingKey::WristTaping,
        SettingKey::WristTapeColorLeft,
        SettingKey::WristTapeColorRight,
        SettingKey::Spectacles,
        SettingKey::SpectaclesColor,
        SettingKey::Gloves,
        SettingKey::GlovesColor,
        SettingKey::HunchingDribbling,
        SettingKey::HunchingRunning,
        SettingKey::ArmMovementDribbling,
        SettingKey::ArmMovementRunning,
        SettingKey::CornerKick,
        SettingKey::FreeKick,
        SettingKey::PenaltyKick,
        SettingKey::Dribbling,
        SettingKey::GoalCelebration1,
        SettingKey::GoalCelebration2,
        SettingKey::CheekType,
        SettingKey::ForeheadType,
        SettingKey::FacialHairType,
        SettingKey::LaughterLinesType,
        SettingKey::UpperEyelidType,
        SettingKey::LowerEyelidType,
        SettingKey::EyebrowType,
        SettingKey::NeckLineType,
        SettingKey::NoseType,
        SettingKey::UpperLipType,
        SettingKey::LowerLipType,
    ];

    /// The key's row in the plan table.
    pub fn spec(self) -> KeySpec {
        match self {
            SettingKey::SkinColor => KeySpec {
                table: APPEARANCE,
                name: "skin_color",
                kind: Kind::Number { min: 0, max: 7 },
                comment: "0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)",
            },
            SettingKey::IrisColor => KeySpec {
                table: APPEARANCE,
                name: "iris_color",
                kind: Kind::Number { min: 0, max: 10 },
                comment: "0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet",
            },
            SettingKey::Height => KeySpec {
                table: PHYSIQUE,
                name: "height",
                kind: Kind::Number { min: 0, max: 255 },
                comment: "cm",
            },
            SettingKey::Weight => KeySpec {
                table: PHYSIQUE,
                name: "weight",
                kind: Kind::Number { min: 0, max: 255 },
                comment: "kg",
            },
            SettingKey::NeckLength => physique("neck_length"),
            SettingKey::NeckSize => physique("neck_size"),
            SettingKey::ShoulderHeight => physique("shoulder_height"),
            SettingKey::ShoulderWidth => physique("shoulder_width"),
            SettingKey::Chest => physique("chest"),
            SettingKey::Waist => physique("waist"),
            SettingKey::ArmSize => physique("arm_size"),
            SettingKey::ArmLength => physique("arm_length"),
            SettingKey::Thigh => physique("thigh"),
            SettingKey::Calf => physique("calf"),
            SettingKey::LegLength => physique("leg_length"),
            SettingKey::HeadLength => physique("head_length"),
            SettingKey::HeadWidth => physique("head_width"),
            SettingKey::HeadDepth => physique("head_depth"),
            SettingKey::Sleeves => KeySpec {
                table: STRIP,
                name: "sleeves",
                kind: Kind::Labels(&["seasonal", "short", "long"]),
                comment: "\"seasonal\", \"short\", \"long\"",
            },
            SettingKey::Inners => KeySpec {
                table: STRIP,
                name: "inners",
                kind: Kind::Labels(&["off", "normal", "turtleneck"]),
                comment: "\"off\", \"normal\", \"turtleneck\"",
            },
            SettingKey::Socks => KeySpec {
                table: STRIP,
                name: "socks",
                kind: Kind::Labels(&["standard", "long", "short"]),
                comment: "\"standard\", \"long\", \"short\"",
            },
            SettingKey::Undershorts => KeySpec {
                table: STRIP,
                name: "undershorts",
                kind: Kind::Labels(&["off", "short", "winter_long", "short_winter_long"]),
                comment: "\"off\", \"short\", \"winter_long\" (none in summer, long in winter), \"short_winter_long\" (short in summer, long in winter)",
            },
            SettingKey::Untucked => bool_key("untucked", "true, false"),
            SettingKey::AnkleTaping => bool_key("ankle_taping", "true, false"),
            SettingKey::WristTaping => KeySpec {
                table: STRIP,
                name: "wrist_taping",
                kind: Kind::Labels(&["off", "right", "left", "both"]),
                comment: "\"off\", \"right\", \"left\", \"both\"",
            },
            SettingKey::WristTapeColorLeft => seven_color("wrist_tape_color_left"),
            SettingKey::WristTapeColorRight => seven_color("wrist_tape_color_right"),
            SettingKey::Spectacles => KeySpec {
                table: STRIP,
                name: "spectacles",
                kind: Kind::Number { min: 0, max: 7 },
                comment: "0 none, 1 rectangle rimless, 2 rectangle half frame, 3 rectangle full frame, 4 oval rimless, 5 oval half frame, 6 oval full frame, 7 round full frame",
            },
            SettingKey::SpectaclesColor => KeySpec {
                table: STRIP,
                name: "spectacles_color",
                kind: Kind::Number { min: 0, max: 7 },
                comment: "0 white, 1 black, 2 red, 3 blue, 4 yellow, 5 green, 6 pink, 7 turquoise",
            },
            SettingKey::Gloves => bool_key("gloves", "true, false (outfield player gloves)"),
            SettingKey::GlovesColor => seven_color("gloves_color"),
            SettingKey::HunchingDribbling => {
                motion_1based("hunching_dribbling", 5, "1 to 3 (PES 20 and 21: 1 to 5)")
            }
            SettingKey::HunchingRunning => {
                motion_1based("hunching_running", 5, "1 to 3 (PES 20 and 21: 1 to 5)")
            }
            SettingKey::ArmMovementDribbling => motion_1based(
                "arm_movement_dribbling",
                10,
                "1 to 8 (PES 20 and 21: 1 to 10)",
            ),
            SettingKey::ArmMovementRunning => motion_1based(
                "arm_movement_running",
                10,
                "1 to 8 (PES 20 and 21: 1 to 10)",
            ),
            SettingKey::CornerKick => {
                motion_1based("corner_kick", 10, "1 to 6 (PES 20 and 21: 1 to 10)")
            }
            SettingKey::FreeKick => {
                motion_1based("free_kick", 20, "1 to 16 (PES 20 and 21: 1 to 20)")
            }
            SettingKey::PenaltyKick => {
                motion_1based("penalty_kick", 7, "1 to 4 (PES 20 and 21: 1 to 7)")
            }
            SettingKey::Dribbling => KeySpec {
                table: MOTION,
                name: "dribbling",
                kind: Kind::Number { min: 0, max: 3 },
                comment: "0 to 3, PES 20 and 21 only",
            },
            SettingKey::GoalCelebration1 => celebration("goal_celebration_1"),
            SettingKey::GoalCelebration2 => celebration("goal_celebration_2"),
            SettingKey::CheekType => face("cheek_type", 3, "0 to 3"),
            SettingKey::ForeheadType => face("forehead_type", 5, "0 to 5"),
            SettingKey::FacialHairType => {
                face("facial_hair_type", 19, "0 to 12 (PES 20 and 21: 0 to 19)")
            }
            SettingKey::LaughterLinesType => face("laughter_lines_type", 4, "0 to 4"),
            SettingKey::UpperEyelidType => {
                face("upper_eyelid_type", 7, "0 to 6 (PES 20 and 21: 0 to 7)")
            }
            SettingKey::LowerEyelidType => {
                face("lower_eyelid_type", 6, "0 to 2 (PES 20 and 21: 0 to 6)")
            }
            SettingKey::EyebrowType => face("eyebrow_type", 7, "0 to 5 (PES 20 and 21: 0 to 7)"),
            SettingKey::NeckLineType => face("neck_line_type", 3, "0 to 2 (PES 20 and 21: 0 to 3)"),
            SettingKey::NoseType => face("nose_type", 7, "0 to 6 (PES 20 and 21: 0 to 7)"),
            SettingKey::UpperLipType => face("upper_lip_type", 4, "0 to 3 (PES 20 and 21: 0 to 4)"),
            SettingKey::LowerLipType => face("lower_lip_type", 4, "0 to 2 (PES 20 and 21: 0 to 4)"),
        }
    }

    /// The savefile field the key writes.
    #[cfg(test)]
    pub(crate) fn source(self) -> Source {
        match self {
            SettingKey::SkinColor => Source::Face(IngameFaceField::SkinColor),
            SettingKey::IrisColor => Source::Face(IngameFaceField::IrisColor),
            SettingKey::Height => Source::Player(PlayerField::Height),
            SettingKey::Weight => Source::Player(PlayerField::Weight),
            SettingKey::NeckLength => Source::Player(PlayerField::NeckLength),
            SettingKey::NeckSize => Source::Player(PlayerField::NeckSize),
            SettingKey::ShoulderHeight => Source::Player(PlayerField::ShoulderHeight),
            SettingKey::ShoulderWidth => Source::Player(PlayerField::ShoulderWidth),
            SettingKey::Chest => Source::Player(PlayerField::Chest),
            SettingKey::Waist => Source::Player(PlayerField::Waist),
            SettingKey::ArmSize => Source::Player(PlayerField::ArmSize),
            SettingKey::ArmLength => Source::Player(PlayerField::ArmLength),
            SettingKey::Thigh => Source::Player(PlayerField::Thigh),
            SettingKey::Calf => Source::Player(PlayerField::Calf),
            SettingKey::LegLength => Source::Player(PlayerField::LegLength),
            SettingKey::HeadLength => Source::Player(PlayerField::HeadLength),
            SettingKey::HeadWidth => Source::Player(PlayerField::HeadWidth),
            SettingKey::HeadDepth => Source::Player(PlayerField::HeadDepth),
            SettingKey::Sleeves => Source::Player(PlayerField::Sleeves),
            SettingKey::Inners => Source::Player(PlayerField::Inners),
            SettingKey::Socks => Source::Player(PlayerField::Socks),
            SettingKey::Undershorts => Source::Player(PlayerField::Undershorts),
            SettingKey::Untucked => Source::Player(PlayerField::Untucked),
            SettingKey::AnkleTaping => Source::Player(PlayerField::AnkleTaping),
            SettingKey::WristTaping => Source::Player(PlayerField::WristTaping),
            SettingKey::WristTapeColorLeft => Source::Player(PlayerField::WristTapeColorLeft),
            SettingKey::WristTapeColorRight => Source::Player(PlayerField::WristTapeColorRight),
            SettingKey::Spectacles => Source::Player(PlayerField::SpectaclesStyle),
            SettingKey::SpectaclesColor => Source::Player(PlayerField::SpectaclesColor),
            SettingKey::Gloves => Source::Face(IngameFaceField::PlayerGloves),
            SettingKey::GlovesColor => Source::Face(IngameFaceField::PlayerGlovesColor),
            SettingKey::HunchingDribbling => Source::Player(PlayerField::HunchingDribbling),
            SettingKey::HunchingRunning => Source::Player(PlayerField::HunchingRunning),
            SettingKey::ArmMovementDribbling => Source::Player(PlayerField::ArmMovementDribbling),
            SettingKey::ArmMovementRunning => Source::Player(PlayerField::ArmMovementRunning),
            SettingKey::CornerKick => Source::Player(PlayerField::CornerKickMotion),
            SettingKey::FreeKick => Source::Player(PlayerField::FreeKickMotion),
            SettingKey::PenaltyKick => Source::Player(PlayerField::PenaltyKickMotion),
            SettingKey::Dribbling => Source::Player(PlayerField::DribblingMotion),
            SettingKey::GoalCelebration1 => Source::Player(PlayerField::GoalCelebration1),
            SettingKey::GoalCelebration2 => Source::Player(PlayerField::GoalCelebration2),
            SettingKey::CheekType => Source::Face(IngameFaceField::CheekType),
            SettingKey::ForeheadType => Source::Face(IngameFaceField::ForeheadType),
            SettingKey::FacialHairType => Source::Face(IngameFaceField::FacialHairType),
            SettingKey::LaughterLinesType => Source::Face(IngameFaceField::LaughterLinesType),
            SettingKey::UpperEyelidType => Source::Face(IngameFaceField::UpperEyelidType),
            SettingKey::LowerEyelidType => Source::Face(IngameFaceField::LowerEyelidType),
            SettingKey::EyebrowType => Source::Face(IngameFaceField::EyebrowType),
            SettingKey::NeckLineType => Source::Face(IngameFaceField::NeckLineType),
            SettingKey::NoseType => Source::Face(IngameFaceField::NoseType),
            SettingKey::UpperLipType => Source::Face(IngameFaceField::UpperLipType),
            SettingKey::LowerLipType => Source::Face(IngameFaceField::LowerLipType),
        }
    }
}

fn physique(name: &'static str) -> KeySpec {
    KeySpec {
        table: PHYSIQUE,
        name,
        kind: Kind::Signed7,
        comment: SEVEN,
    }
}

fn bool_key(name: &'static str, comment: &'static str) -> KeySpec {
    KeySpec {
        table: STRIP,
        name,
        kind: Kind::Bool,
        comment,
    }
}

fn seven_color(name: &'static str) -> KeySpec {
    KeySpec {
        table: STRIP,
        name,
        kind: Kind::Number { min: 0, max: 7 },
        comment: "0 to 7",
    }
}

fn motion_1based(name: &'static str, max: u8, comment: &'static str) -> KeySpec {
    KeySpec {
        table: MOTION,
        name,
        kind: Kind::OneBased { max },
        comment,
    }
}

fn celebration(name: &'static str) -> KeySpec {
    KeySpec {
        table: MOTION,
        name,
        kind: Kind::Number { min: 0, max: 162 },
        comment: "0 none, 1 to 122 (PES 20 and 21: 1 to 162)",
    }
}

fn face(name: &'static str, max: u8, comment: &'static str) -> KeySpec {
    KeySpec {
        table: FACE,
        name,
        kind: Kind::Number { min: 0, max },
        comment,
    }
}
