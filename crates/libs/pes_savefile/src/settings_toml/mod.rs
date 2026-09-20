//! `PlayerSettings`: the model behind the export format's `settings.toml`
//! files — every authorable aesthetic setting of a player, as stored values
//! (`Option` throughout: `None` leaves the savefile field untouched). Parsing
//! and emitting the TOML text are a later slice; this is the model the codec
//! reads and writes through `from_player`/`apply`.

/// The settings.toml key table: `SettingKey` and its `KeySpec` columns.
pub(crate) mod keys;

#[cfg(test)]
mod tests;

use toml_edit::{DocumentMut, Item, Value};

use crate::codec::CodecError;
use crate::model::player::PlayerEntry;
#[cfg(test)]
use crate::schema::fields::PlayerField;
use crate::schema::ingame_face::IngameFaceField;

pub use keys::{KeySpec, Kind, SettingKey};

/// The player-name setting: `true` in TOML derives from the folder, a string
/// is written as is, absent leaves the savefile name untouched.
#[derive(Debug, Clone, PartialEq)]
pub enum NameSetting {
    /// Derive the name from the folder (`name = true`); `apply` does not
    /// resolve it — the caller (the Team compiler) does, since the folder
    /// rules are its.
    FromFolder,
    /// Write the string as is, colour codes included.
    Explicit(String),
}

/// The `[appearance]` table's stored values: every leaf `Option`, `None` =
/// leave the savefile value alone. Physique numbers are the stored value
/// (game number + 7), motion numbers are stored (1-based keys minus 1), label
/// keys are the index into the key's label list.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppearanceSettings {
    /// Skin colour.
    pub skin_color: Option<u8>,
    /// Iris colour.
    pub iris_color: Option<u8>,
    /// `[appearance.physique]`.
    pub physique: PhysiqueSettings,
    /// `[appearance.strip]`.
    pub strip: StripSettings,
    /// `[appearance.motion]`.
    pub motion: MotionSettings,
    /// `[appearance.face]`.
    pub face: FaceSettings,
}

/// The `[appearance.physique]` table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhysiqueSettings {
    /// Height in cm.
    pub height: Option<u8>,
    /// Weight in kg.
    pub weight: Option<u8>,
    /// Neck length, stored (game number + 7).
    pub neck_length: Option<u8>,
    /// Neck size, stored.
    pub neck_size: Option<u8>,
    /// Shoulder height, stored.
    pub shoulder_height: Option<u8>,
    /// Shoulder width, stored.
    pub shoulder_width: Option<u8>,
    /// Chest, stored.
    pub chest: Option<u8>,
    /// Waist, stored.
    pub waist: Option<u8>,
    /// Arm size, stored.
    pub arm_size: Option<u8>,
    /// Arm length, stored.
    pub arm_length: Option<u8>,
    /// Thigh, stored.
    pub thigh: Option<u8>,
    /// Calf, stored.
    pub calf: Option<u8>,
    /// Leg length, stored.
    pub leg_length: Option<u8>,
    /// Head length, stored.
    pub head_length: Option<u8>,
    /// Head width, stored.
    pub head_width: Option<u8>,
    /// Head depth, stored.
    pub head_depth: Option<u8>,
}

/// The `[appearance.strip]` table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StripSettings {
    /// Sleeves (label index).
    pub sleeves: Option<u8>,
    /// Inners (label index).
    pub inners: Option<u8>,
    /// Socks (label index).
    pub socks: Option<u8>,
    /// Undershorts (label index).
    pub undershorts: Option<u8>,
    /// Shirttail out.
    pub untucked: Option<bool>,
    /// Ankle taping.
    pub ankle_taping: Option<bool>,
    /// Wrist taping (label index).
    pub wrist_taping: Option<u8>,
    /// Left wrist tape colour.
    pub wrist_tape_color_left: Option<u8>,
    /// Right wrist tape colour.
    pub wrist_tape_color_right: Option<u8>,
    /// Spectacles style.
    pub spectacles: Option<u8>,
    /// Spectacles frame colour.
    pub spectacles_color: Option<u8>,
    /// Player (outfield) gloves.
    pub gloves: Option<bool>,
    /// Player gloves colour.
    pub gloves_color: Option<u8>,
}

/// The `[appearance.motion]` table, stored values (1-based keys minus 1).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionSettings {
    /// Hunching while dribbling, stored.
    pub hunching_dribbling: Option<u8>,
    /// Hunching while running, stored.
    pub hunching_running: Option<u8>,
    /// Arm movement while dribbling, stored.
    pub arm_movement_dribbling: Option<u8>,
    /// Arm movement while running, stored.
    pub arm_movement_running: Option<u8>,
    /// Corner kick motion, stored.
    pub corner_kick: Option<u8>,
    /// Free kick motion, stored.
    pub free_kick: Option<u8>,
    /// Penalty kick motion, stored.
    pub penalty_kick: Option<u8>,
    /// Dribbling motion (PES 20+).
    pub dribbling: Option<u8>,
    /// First goal celebration.
    pub goal_celebration_1: Option<u8>,
    /// Second goal celebration.
    pub goal_celebration_2: Option<u8>,
}

/// The `[appearance.face]` table: the eleven feature types of the
/// ingame-face run.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FaceSettings {
    /// Cheek type.
    pub cheek_type: Option<u8>,
    /// Forehead type.
    pub forehead_type: Option<u8>,
    /// Facial hair type.
    pub facial_hair_type: Option<u8>,
    /// Laughter lines type.
    pub laughter_lines_type: Option<u8>,
    /// Upper eyelid type.
    pub upper_eyelid_type: Option<u8>,
    /// Lower eyelid type.
    pub lower_eyelid_type: Option<u8>,
    /// Eyebrow type.
    pub eyebrow_type: Option<u8>,
    /// Neck line type.
    pub neck_line_type: Option<u8>,
    /// Nose type.
    pub nose_type: Option<u8>,
    /// Upper lip type.
    pub upper_lip_type: Option<u8>,
    /// Lower lip type.
    pub lower_lip_type: Option<u8>,
}

/// A settings.toml's player: the name plus the appearance tables.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerSettings {
    /// The name setting; `None` = never write a name.
    pub name: Option<NameSetting>,
    /// The `[appearance]` tables.
    pub appearance: AppearanceSettings,
}

/// A `PlayerSettings` could not be produced or applied.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// A set key has no field in this player's game version.
    #[error("{key}: this player's game version has no such setting")]
    NotInThisVersion {
        /// The TOML key.
        key: String,
    },
    /// A codec read or write failed (a player never read from a record has no
    /// ingame-face run to read or write through).
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// The text is not valid TOML.
    #[error("settings.toml is not valid TOML: {0}")]
    Toml(String),
    /// A key or table the settings schema does not know, by its dotted path
    /// (`appearance.boots_id`, `appearance.hair`, `stats`).
    #[error("{key}: unknown key")]
    UnknownKey {
        /// The dotted path.
        key: String,
    },
    /// The value is not the kind the key wants.
    #[error("{key}: expected {expected}")]
    WrongType {
        /// The dotted path (`name`, `appearance.strip.untucked`).
        key: String,
        /// What the key wants: "an integer", "a string", "true or false",
        /// "true or a string", "a table".
        expected: &'static str,
    },
    /// An integer outside the key's range.
    #[error("{key}: {value} is outside {range}")]
    OutOfRange {
        /// The dotted path.
        key: String,
        /// The value as written.
        value: i64,
        /// The range text ("-7 to 7", "1 to 10", "0 to 7").
        range: String,
    },
    /// A string that is none of the key's labels.
    #[error("{key}: unknown value {label:?}, expected one of {allowed}")]
    UnknownLabel {
        /// The dotted path.
        key: String,
        /// The label as written.
        label: String,
        /// The labels joined by ", ", each quoted.
        allowed: String,
    },
}

/// The stored value behind an appearance key (`bool` as 0/1), `None` when
/// unset. Free-standing so `team_toml`'s per-key gating can read one key.
pub(crate) fn get_appearance(appearance: &AppearanceSettings, key: SettingKey) -> Option<u8> {
    let a = appearance;
    match key {
        SettingKey::SkinColor => a.skin_color,
        SettingKey::IrisColor => a.iris_color,
        SettingKey::Height => a.physique.height,
        SettingKey::Weight => a.physique.weight,
        SettingKey::NeckLength => a.physique.neck_length,
        SettingKey::NeckSize => a.physique.neck_size,
        SettingKey::ShoulderHeight => a.physique.shoulder_height,
        SettingKey::ShoulderWidth => a.physique.shoulder_width,
        SettingKey::Chest => a.physique.chest,
        SettingKey::Waist => a.physique.waist,
        SettingKey::ArmSize => a.physique.arm_size,
        SettingKey::ArmLength => a.physique.arm_length,
        SettingKey::Thigh => a.physique.thigh,
        SettingKey::Calf => a.physique.calf,
        SettingKey::LegLength => a.physique.leg_length,
        SettingKey::HeadLength => a.physique.head_length,
        SettingKey::HeadWidth => a.physique.head_width,
        SettingKey::HeadDepth => a.physique.head_depth,
        SettingKey::Sleeves => a.strip.sleeves,
        SettingKey::Inners => a.strip.inners,
        SettingKey::Socks => a.strip.socks,
        SettingKey::Undershorts => a.strip.undershorts,
        SettingKey::Untucked => a.strip.untucked.map(u8::from),
        SettingKey::AnkleTaping => a.strip.ankle_taping.map(u8::from),
        SettingKey::WristTaping => a.strip.wrist_taping,
        SettingKey::WristTapeColorLeft => a.strip.wrist_tape_color_left,
        SettingKey::WristTapeColorRight => a.strip.wrist_tape_color_right,
        SettingKey::Spectacles => a.strip.spectacles,
        SettingKey::SpectaclesColor => a.strip.spectacles_color,
        SettingKey::Gloves => a.strip.gloves.map(u8::from),
        SettingKey::GlovesColor => a.strip.gloves_color,
        SettingKey::HunchingDribbling => a.motion.hunching_dribbling,
        SettingKey::HunchingRunning => a.motion.hunching_running,
        SettingKey::ArmMovementDribbling => a.motion.arm_movement_dribbling,
        SettingKey::ArmMovementRunning => a.motion.arm_movement_running,
        SettingKey::CornerKick => a.motion.corner_kick,
        SettingKey::FreeKick => a.motion.free_kick,
        SettingKey::PenaltyKick => a.motion.penalty_kick,
        SettingKey::Dribbling => a.motion.dribbling,
        SettingKey::GoalCelebration1 => a.motion.goal_celebration_1,
        SettingKey::GoalCelebration2 => a.motion.goal_celebration_2,
        SettingKey::CheekType => a.face.cheek_type,
        SettingKey::ForeheadType => a.face.forehead_type,
        SettingKey::FacialHairType => a.face.facial_hair_type,
        SettingKey::LaughterLinesType => a.face.laughter_lines_type,
        SettingKey::UpperEyelidType => a.face.upper_eyelid_type,
        SettingKey::LowerEyelidType => a.face.lower_eyelid_type,
        SettingKey::EyebrowType => a.face.eyebrow_type,
        SettingKey::NeckLineType => a.face.neck_line_type,
        SettingKey::NoseType => a.face.nose_type,
        SettingKey::UpperLipType => a.face.upper_lip_type,
        SettingKey::LowerLipType => a.face.lower_lip_type,
    }
}

/// Sets an appearance key's field to `value` (`None` clears it). Free-standing
/// so `team_toml`'s cross-version pre-adjust can drop a gated key.
pub(crate) fn set_appearance(
    appearance: &mut AppearanceSettings,
    key: SettingKey,
    value: Option<u8>,
) {
    let a = appearance;
    match key {
        SettingKey::SkinColor => a.skin_color = value,
        SettingKey::IrisColor => a.iris_color = value,
        SettingKey::Height => a.physique.height = value,
        SettingKey::Weight => a.physique.weight = value,
        SettingKey::NeckLength => a.physique.neck_length = value,
        SettingKey::NeckSize => a.physique.neck_size = value,
        SettingKey::ShoulderHeight => a.physique.shoulder_height = value,
        SettingKey::ShoulderWidth => a.physique.shoulder_width = value,
        SettingKey::Chest => a.physique.chest = value,
        SettingKey::Waist => a.physique.waist = value,
        SettingKey::ArmSize => a.physique.arm_size = value,
        SettingKey::ArmLength => a.physique.arm_length = value,
        SettingKey::Thigh => a.physique.thigh = value,
        SettingKey::Calf => a.physique.calf = value,
        SettingKey::LegLength => a.physique.leg_length = value,
        SettingKey::HeadLength => a.physique.head_length = value,
        SettingKey::HeadWidth => a.physique.head_width = value,
        SettingKey::HeadDepth => a.physique.head_depth = value,
        SettingKey::Sleeves => a.strip.sleeves = value,
        SettingKey::Inners => a.strip.inners = value,
        SettingKey::Socks => a.strip.socks = value,
        SettingKey::Undershorts => a.strip.undershorts = value,
        SettingKey::Untucked => a.strip.untucked = value.map(|v| v != 0),
        SettingKey::AnkleTaping => a.strip.ankle_taping = value.map(|v| v != 0),
        SettingKey::WristTaping => a.strip.wrist_taping = value,
        SettingKey::WristTapeColorLeft => a.strip.wrist_tape_color_left = value,
        SettingKey::WristTapeColorRight => a.strip.wrist_tape_color_right = value,
        SettingKey::Spectacles => a.strip.spectacles = value,
        SettingKey::SpectaclesColor => a.strip.spectacles_color = value,
        SettingKey::Gloves => a.strip.gloves = value.map(|v| v != 0),
        SettingKey::GlovesColor => a.strip.gloves_color = value,
        SettingKey::HunchingDribbling => a.motion.hunching_dribbling = value,
        SettingKey::HunchingRunning => a.motion.hunching_running = value,
        SettingKey::ArmMovementDribbling => a.motion.arm_movement_dribbling = value,
        SettingKey::ArmMovementRunning => a.motion.arm_movement_running = value,
        SettingKey::CornerKick => a.motion.corner_kick = value,
        SettingKey::FreeKick => a.motion.free_kick = value,
        SettingKey::PenaltyKick => a.motion.penalty_kick = value,
        SettingKey::Dribbling => a.motion.dribbling = value,
        SettingKey::GoalCelebration1 => a.motion.goal_celebration_1 = value,
        SettingKey::GoalCelebration2 => a.motion.goal_celebration_2 = value,
        SettingKey::CheekType => a.face.cheek_type = value,
        SettingKey::ForeheadType => a.face.forehead_type = value,
        SettingKey::FacialHairType => a.face.facial_hair_type = value,
        SettingKey::LaughterLinesType => a.face.laughter_lines_type = value,
        SettingKey::UpperEyelidType => a.face.upper_eyelid_type = value,
        SettingKey::LowerEyelidType => a.face.lower_eyelid_type = value,
        SettingKey::EyebrowType => a.face.eyebrow_type = value,
        SettingKey::NeckLineType => a.face.neck_line_type = value,
        SettingKey::NoseType => a.face.nose_type = value,
        SettingKey::UpperLipType => a.face.upper_lip_type = value,
        SettingKey::LowerLipType => a.face.lower_lip_type = value,
    }
}

/// The stored value behind `key` on `player`, `None` for a gated `Option`
/// the version lacks. `CodecError` from the ingame-face reads propagates.
fn read(player: &PlayerEntry, key: SettingKey) -> Result<Option<u8>, SettingsError> {
    let face = &player.appearance.ingame_face;
    let a = &player.appearance;
    let m = &player.motion;
    Ok(match key {
        SettingKey::SkinColor => Some(face.get(IngameFaceField::SkinColor)?),
        SettingKey::IrisColor => Some(face.get(IngameFaceField::IrisColor)?),
        SettingKey::Height => Some(player.basic.height),
        SettingKey::Weight => Some(player.basic.weight),
        SettingKey::NeckLength => Some(a.neck_length),
        SettingKey::NeckSize => Some(a.neck_size),
        SettingKey::ShoulderHeight => Some(a.shoulder_height),
        SettingKey::ShoulderWidth => Some(a.shoulder_width),
        SettingKey::Chest => Some(a.chest),
        SettingKey::Waist => Some(a.waist),
        SettingKey::ArmSize => Some(a.arm_size),
        SettingKey::ArmLength => Some(a.arm_length),
        SettingKey::Thigh => Some(a.thigh),
        SettingKey::Calf => Some(a.calf),
        SettingKey::LegLength => Some(a.leg_length),
        SettingKey::HeadLength => Some(a.head_length),
        SettingKey::HeadWidth => Some(a.head_width),
        SettingKey::HeadDepth => Some(a.head_depth),
        SettingKey::Sleeves => Some(a.sleeves),
        SettingKey::Inners => Some(a.inners),
        SettingKey::Socks => Some(a.socks),
        SettingKey::Undershorts => Some(a.undershorts),
        SettingKey::Untucked => Some(a.untucked.into()),
        SettingKey::AnkleTaping => Some(a.ankle_taping.into()),
        SettingKey::WristTaping => Some(a.wrist_taping),
        SettingKey::WristTapeColorLeft => Some(a.wrist_tape_color_left),
        SettingKey::WristTapeColorRight => Some(a.wrist_tape_color_right),
        SettingKey::Spectacles => Some(a.spectacles_style),
        SettingKey::SpectaclesColor => Some(a.spectacles_color),
        SettingKey::Gloves => Some(face.get(IngameFaceField::PlayerGloves)?),
        SettingKey::GlovesColor => Some(face.get(IngameFaceField::PlayerGlovesColor)?),
        SettingKey::HunchingDribbling => Some(m.hunching_dribbling),
        SettingKey::HunchingRunning => Some(m.hunching_running),
        SettingKey::ArmMovementDribbling => Some(m.arm_movement_dribbling),
        SettingKey::ArmMovementRunning => Some(m.arm_movement_running),
        SettingKey::CornerKick => Some(m.corner_kick),
        SettingKey::FreeKick => Some(m.free_kick),
        SettingKey::PenaltyKick => Some(m.penalty_kick),
        SettingKey::Dribbling => m.dribbling,
        SettingKey::GoalCelebration1 => Some(m.goal_celebration_1),
        SettingKey::GoalCelebration2 => Some(m.goal_celebration_2),
        SettingKey::CheekType => Some(face.get(IngameFaceField::CheekType)?),
        SettingKey::ForeheadType => Some(face.get(IngameFaceField::ForeheadType)?),
        SettingKey::FacialHairType => Some(face.get(IngameFaceField::FacialHairType)?),
        SettingKey::LaughterLinesType => Some(face.get(IngameFaceField::LaughterLinesType)?),
        SettingKey::UpperEyelidType => Some(face.get(IngameFaceField::UpperEyelidType)?),
        SettingKey::LowerEyelidType => Some(face.get(IngameFaceField::LowerEyelidType)?),
        SettingKey::EyebrowType => Some(face.get(IngameFaceField::EyebrowType)?),
        SettingKey::NeckLineType => Some(face.get(IngameFaceField::NeckLineType)?),
        SettingKey::NoseType => Some(face.get(IngameFaceField::NoseType)?),
        SettingKey::UpperLipType => Some(face.get(IngameFaceField::UpperLipType)?),
        SettingKey::LowerLipType => Some(face.get(IngameFaceField::LowerLipType)?),
    })
}

/// The player's appearance settings for Team TOML: every key `Some` with its
/// raw stored value — a save can hold values past the editor's range (the
/// field's bits are wider than the selectable range), and the interchange's
/// stored-ranges mode carries them verbatim.
pub(crate) fn appearance_from(player: &PlayerEntry) -> Result<AppearanceSettings, SettingsError> {
    let mut appearance = AppearanceSettings::default();
    for key in SettingKey::ALL {
        if let Some(value) = read(player, key)? {
            set_appearance(&mut appearance, key, Some(value));
        }
    }
    Ok(appearance)
}

impl PlayerSettings {
    /// The stored value behind a key (`bool` as 0/1), `None` when unset.
    pub fn get(&self, key: SettingKey) -> Option<u8> {
        get_appearance(&self.appearance, key)
    }

    /// Sets a stored value; `OutOfRange` when the key's kind cannot represent
    /// it, so a `PlayerSettings` built through `set` is always one the
    /// emitters can write.
    pub fn set(&mut self, key: SettingKey, value: u8) -> Result<(), SettingsError> {
        let spec = key.spec();
        if !spec.kind.accepts_stored(value) {
            return Err(SettingsError::OutOfRange {
                key: dotted(&spec),
                value: i64::from(value),
                range: range_text(spec.kind),
            });
        }
        set_appearance(&mut self.appearance, key, Some(value));
        Ok(())
    }

    /// Every key `Some` from the player; `name` is `Explicit` of the raw name,
    /// colour codes included. A stored value the key's kind cannot represent
    /// (a hostile save's out-of-range bits) is `OutOfRange`.
    pub fn from_player(player: &PlayerEntry) -> Result<Self, SettingsError> {
        let mut settings = PlayerSettings {
            name: Some(NameSetting::Explicit(player.name.clone())),
            ..PlayerSettings::default()
        };
        for key in SettingKey::ALL {
            if let Some(value) = read(player, key)? {
                settings.set(key, value)?;
            }
        }
        Ok(settings)
    }

    /// Writes the `Some` fields onto the player, all or nothing: on `Err` the
    /// player is unchanged. `NameSetting::FromFolder` is not applied: the
    /// caller (the Team compiler) resolves it to `Explicit` first, since the
    /// folder rules are its. A `Some` on a field this player's version lacks
    /// (`dribbling` before PES 20) is `SettingsError::NotInThisVersion`; a
    /// `Some` the key's kind cannot represent is `OutOfRange`, checked before
    /// any write.
    pub fn apply(&self, player: &mut PlayerEntry) -> Result<(), SettingsError> {
        for key in SettingKey::ALL {
            if let Some(value) = self.get(key) {
                let spec = key.spec();
                if !spec.kind.accepts_stored(value) {
                    return Err(SettingsError::OutOfRange {
                        key: dotted(&spec),
                        value: i64::from(value),
                        range: range_text(spec.kind),
                    });
                }
            }
        }
        let mut next = player.clone();
        if let Some(NameSetting::Explicit(name)) = &self.name {
            next.name = name.clone();
        }
        write_appearance(&self.appearance, &mut next)?;
        *player = next;
        Ok(())
    }
}

/// The write half of `PlayerSettings::apply`, without the kind-range
/// validation: `team_toml` applies a document's stored values as written.
/// Still all-or-nothing and still `NotInThisVersion` for a `Some` on a field
/// the player's version lacks.
pub(crate) fn apply_appearance(
    appearance: &AppearanceSettings,
    player: &mut PlayerEntry,
) -> Result<(), SettingsError> {
    let mut next = player.clone();
    write_appearance(appearance, &mut next)?;
    *player = next;
    Ok(())
}

/// Writes every `Some` key onto `next` (called on a scratch clone).
fn write_appearance(
    appearance: &AppearanceSettings,
    next: &mut PlayerEntry,
) -> Result<(), SettingsError> {
    let face = &mut next.appearance.ingame_face;
    for key in SettingKey::ALL {
        let Some(value) = get_appearance(appearance, key) else {
            continue;
        };
        match key {
            SettingKey::SkinColor => face.set(IngameFaceField::SkinColor, value)?,
            SettingKey::IrisColor => face.set(IngameFaceField::IrisColor, value)?,
            SettingKey::Height => next.basic.height = value,
            SettingKey::Weight => next.basic.weight = value,
            SettingKey::NeckLength => next.appearance.neck_length = value,
            SettingKey::NeckSize => next.appearance.neck_size = value,
            SettingKey::ShoulderHeight => next.appearance.shoulder_height = value,
            SettingKey::ShoulderWidth => next.appearance.shoulder_width = value,
            SettingKey::Chest => next.appearance.chest = value,
            SettingKey::Waist => next.appearance.waist = value,
            SettingKey::ArmSize => next.appearance.arm_size = value,
            SettingKey::ArmLength => next.appearance.arm_length = value,
            SettingKey::Thigh => next.appearance.thigh = value,
            SettingKey::Calf => next.appearance.calf = value,
            SettingKey::LegLength => next.appearance.leg_length = value,
            SettingKey::HeadLength => next.appearance.head_length = value,
            SettingKey::HeadWidth => next.appearance.head_width = value,
            SettingKey::HeadDepth => next.appearance.head_depth = value,
            SettingKey::Sleeves => next.appearance.sleeves = value,
            SettingKey::Inners => next.appearance.inners = value,
            SettingKey::Socks => next.appearance.socks = value,
            SettingKey::Undershorts => next.appearance.undershorts = value,
            SettingKey::Untucked => next.appearance.untucked = value != 0,
            SettingKey::AnkleTaping => next.appearance.ankle_taping = value != 0,
            SettingKey::WristTaping => next.appearance.wrist_taping = value,
            SettingKey::WristTapeColorLeft => next.appearance.wrist_tape_color_left = value,
            SettingKey::WristTapeColorRight => next.appearance.wrist_tape_color_right = value,
            SettingKey::Spectacles => next.appearance.spectacles_style = value,
            SettingKey::SpectaclesColor => next.appearance.spectacles_color = value,
            SettingKey::Gloves => face.set(IngameFaceField::PlayerGloves, value)?,
            SettingKey::GlovesColor => face.set(IngameFaceField::PlayerGlovesColor, value)?,
            SettingKey::HunchingDribbling => next.motion.hunching_dribbling = value,
            SettingKey::HunchingRunning => next.motion.hunching_running = value,
            SettingKey::ArmMovementDribbling => next.motion.arm_movement_dribbling = value,
            SettingKey::ArmMovementRunning => next.motion.arm_movement_running = value,
            SettingKey::CornerKick => next.motion.corner_kick = value,
            SettingKey::FreeKick => next.motion.free_kick = value,
            SettingKey::PenaltyKick => next.motion.penalty_kick = value,
            SettingKey::Dribbling => match next.motion.dribbling {
                Some(_) => next.motion.dribbling = Some(value),
                None => {
                    return Err(SettingsError::NotInThisVersion {
                        key: key.spec().name.to_string(),
                    });
                }
            },
            SettingKey::GoalCelebration1 => next.motion.goal_celebration_1 = value,
            SettingKey::GoalCelebration2 => next.motion.goal_celebration_2 = value,
            SettingKey::CheekType => face.set(IngameFaceField::CheekType, value)?,
            SettingKey::ForeheadType => face.set(IngameFaceField::ForeheadType, value)?,
            SettingKey::FacialHairType => face.set(IngameFaceField::FacialHairType, value)?,
            SettingKey::LaughterLinesType => face.set(IngameFaceField::LaughterLinesType, value)?,
            SettingKey::UpperEyelidType => face.set(IngameFaceField::UpperEyelidType, value)?,
            SettingKey::LowerEyelidType => face.set(IngameFaceField::LowerEyelidType, value)?,
            SettingKey::EyebrowType => face.set(IngameFaceField::EyebrowType, value)?,
            SettingKey::NeckLineType => face.set(IngameFaceField::NeckLineType, value)?,
            SettingKey::NoseType => face.set(IngameFaceField::NoseType, value)?,
            SettingKey::UpperLipType => face.set(IngameFaceField::UpperLipType, value)?,
            SettingKey::LowerLipType => face.set(IngameFaceField::LowerLipType, value)?,
        }
    }
    Ok(())
}

/// Who owns a savefile field: authored through `settings.toml`, derived by the
/// compiler, or gameplay data outside the file's scope. Test-side only so far:
/// the completeness test is its consumer.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ownership {
    /// An authorable `settings.toml` setting.
    Settings,
    /// Derived by the compiler (model ids, edit flags, base copy).
    CompilerOwned,
    /// Gameplay data the file does not carry.
    Gameplay,
}

/// The ownership of one player-record field.
#[cfg(test)]
pub(crate) fn ownership(field: PlayerField) -> Ownership {
    match field {
        PlayerField::BootsId
        | PlayerField::GlovesId
        | PlayerField::BaseCopyId
        | PlayerField::BaseCopy
        | PlayerField::EditedPlayer
        | PlayerField::EditedBasicSettings
        | PlayerField::EditedRegisteredPosition
        | PlayerField::EditedPlayablePositions
        | PlayerField::EditedAbilities
        | PlayerField::EditedSkills
        | PlayerField::EditedPlayingStyle
        | PlayerField::EditedComStyles
        | PlayerField::EditedMotion
        | PlayerField::EditedFace
        | PlayerField::EditedHair
        | PlayerField::EditedPhysique
        | PlayerField::EditedStrip => Ownership::CompilerOwned,
        PlayerField::Height
        | PlayerField::Weight
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
        | PlayerField::AnkleTaping
        | PlayerField::HunchingDribbling
        | PlayerField::HunchingRunning
        | PlayerField::ArmMovementDribbling
        | PlayerField::ArmMovementRunning
        | PlayerField::CornerKickMotion
        | PlayerField::FreeKickMotion
        | PlayerField::PenaltyKickMotion
        | PlayerField::DribblingMotion
        | PlayerField::GoalCelebration1
        | PlayerField::GoalCelebration2 => Ownership::Settings,
        PlayerField::Id
        | PlayerField::Nationality
        | PlayerField::Age
        | PlayerField::AttackingProwess
        | PlayerField::DefensiveProwess
        | PlayerField::Goalkeeping
        | PlayerField::Dribbling
        | PlayerField::Finishing
        | PlayerField::LowPass
        | PlayerField::LoftedPass
        | PlayerField::Heading
        | PlayerField::Form
        | PlayerField::Swerve
        | PlayerField::Catching
        | PlayerField::Clearing
        | PlayerField::Coverage
        | PlayerField::Reflexes
        | PlayerField::InjuryResistance
        | PlayerField::BodyControl
        | PlayerField::PhysicalContact
        | PlayerField::KickingPower
        | PlayerField::ExplosivePower
        | PlayerField::RegisteredPosition
        | PlayerField::PlayingStyle
        | PlayerField::BallControl
        | PlayerField::BallWinning
        | PlayerField::WeakFootAccuracy
        | PlayerField::WeakFootUsage
        | PlayerField::Jump
        | PlayerField::PlaceKicking
        | PlayerField::Stamina
        | PlayerField::Speed
        | PlayerField::StrongerFoot
        | PlayerField::StrongerHand
        | PlayerField::Star
        | PlayerField::TightPossession
        | PlayerField::Aggression
        | PlayerField::PlayingAttitude
        | PlayerField::PlayablePosition(_)
        | PlayerField::ComStyle(_)
        | PlayerField::Skill(_) => Ownership::Gameplay,
    }
}

/// The ownership of one ingame-face field: all are settings today, still an
/// exhaustive match so a new field fails to compile until classified.
#[cfg(test)]
pub(crate) fn face_ownership(field: IngameFaceField) -> Ownership {
    match field {
        IngameFaceField::PlayerGloves
        | IngameFaceField::PlayerGlovesColor
        | IngameFaceField::SkinColor
        | IngameFaceField::CheekType
        | IngameFaceField::ForeheadType
        | IngameFaceField::FacialHairType
        | IngameFaceField::LaughterLinesType
        | IngameFaceField::UpperEyelidType
        | IngameFaceField::LowerEyelidType
        | IngameFaceField::EyebrowType
        | IngameFaceField::NeckLineType
        | IngameFaceField::NoseType
        | IngameFaceField::UpperLipType
        | IngameFaceField::LowerLipType
        | IngameFaceField::IrisColor => Ownership::Settings,
    }
}

// The TOML half: `parse` reads a `settings.toml` text, `to_toml` emits the
// template (the plan's key-table block verbatim for a fully set settings),
// `update_toml` rewrites the `Some` values inside an existing document while
// preserving its comments and formatting.

/// The header of a generated `settings.toml`: the plan block's four comment
/// lines and the two-line `name` comment, verbatim.
const HEADER: &str = "\
# settings.toml, inside a player folder. Every key is optional: an absent key leaves
# that savefile setting untouched. A commented key shows what can be set and its range.
# The file never references models: what a player wears is decided by the folder
# contents (models present, link files pointing at shared folders).

# true = derive from the folder name (\"15 - Snuffy\" gives \"Snuffy\"; the whole folder
# name for players.txt-mapped folders); \"text\" = write as is; absent = leave untouched.
";

/// The dotted path of a key ("appearance.strip.sleeves").
fn dotted(spec: &keys::KeySpec) -> String {
    format!("{}.{}", spec.table, spec.name)
}

/// A key's TOML value for a stored `u8` (label string, signed physique
/// number, 1-based motion number, bool); `OutOfRange` for a value written
/// directly into a public field that the kind cannot represent.
fn toml_form(
    spec: &keys::KeySpec,
    stored: u8,
    stored_ranges: bool,
) -> Result<Value, SettingsError> {
    if !stored_ranges && !spec.kind.accepts_stored(stored) {
        return Err(SettingsError::OutOfRange {
            key: dotted(spec),
            value: i64::from(stored),
            range: range_text(spec.kind),
        });
    }
    Ok(match spec.kind {
        Kind::Number { .. } => i64::from(stored).into(),
        Kind::Signed7 => (i64::from(stored) - 7).into(),
        Kind::OneBased { .. } => (i64::from(stored) + 1).into(),
        Kind::Bool => (stored != 0).into(),
        Kind::Labels(labels) => labels[usize::from(stored)].into(),
    })
}

/// The range text a `Kind` reports (`"0 to 7"`, `"-7 to 7"`, `"1 to 10"`,
/// `"true, false"`, or the quoted label list).
fn range_text(kind: Kind) -> String {
    match kind {
        Kind::Number { min, max } => format!("{min} to {max}"),
        Kind::Signed7 => "-7 to 7".to_string(),
        Kind::OneBased { max } => format!("1 to {max}"),
        Kind::Bool => "true, false".to_string(),
        Kind::Labels(labels) => labels
            .iter()
            .map(|label| format!("\"{label}\""))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// The value an unset key's commented line shows.
fn neutral_text(kind: Kind) -> String {
    match kind {
        Kind::Number { .. } | Kind::Signed7 => "0".to_string(),
        Kind::OneBased { .. } => "1".to_string(),
        Kind::Bool => "false".to_string(),
        Kind::Labels(labels) => Value::from(labels[0]).to_string(),
    }
}

/// `body` padded so `#` starts at character 33 (1-based); a body longer than
/// the pad still gets one space before `#`.
fn padded(body: &str, comment: &str) -> String {
    let pad = 32_usize.saturating_sub(body.len()).max(1);
    format!("{body}{}# {comment}", " ".repeat(pad))
}

/// The key's dotted path under `root` (`root` = the appearance table's own
/// path: "appearance" in settings.toml, "players.03.appearance" in team.toml).
fn leaf_path(root: &str, spec: &keys::KeySpec) -> String {
    match spec.table {
        "appearance" => format!("{root}.{}", spec.name),
        _ => format!(
            "{root}.{}.{}",
            &spec.table["appearance.".len()..],
            spec.name
        ),
    }
}

/// The item at a key's `(table, name)` inside the `appearance` table, `None`
/// when absent; `path` is the appearance table's dotted path for error keys.
fn lookup<'a>(
    appearance: &'a dyn toml_edit::TableLike,
    spec: &keys::KeySpec,
    path: &str,
) -> Result<Option<&'a Item>, SettingsError> {
    let table = match spec.table {
        "appearance" => appearance,
        _ => {
            let sub = &spec.table["appearance.".len()..];
            let Some(item) = appearance.get(sub) else {
                return Ok(None);
            };
            match item.as_table_like() {
                Some(table) => table,
                None => {
                    return Err(SettingsError::WrongType {
                        key: format!("{path}.{sub}"),
                        expected: "a table",
                    });
                }
            }
        }
    };
    Ok(table.get(spec.name))
}

/// The stored `u8` behind one present item, range-checked per its `Kind`.
/// `stored_ranges` is `team_toml`'s mode: the editor range is documentation,
/// and any value the stored `u8` can hold is accepted (a real save's face
/// types and celebration numbers go past the selectable range).
fn value(dotted: &str, kind: Kind, item: &Item, stored_ranges: bool) -> Result<u8, SettingsError> {
    let wide = |value: i64| -> Result<u8, SettingsError> {
        u8::try_from(value).map_err(|_| SettingsError::OutOfRange {
            key: dotted.to_string(),
            value,
            range: "0 to 255".to_string(),
        })
    };
    let integer = |expected: &'static str| -> Result<i64, SettingsError> {
        item.as_value()
            .and_then(|v| v.as_integer())
            .ok_or_else(|| SettingsError::WrongType {
                key: dotted.to_string(),
                expected,
            })
    };
    match kind {
        Kind::Number { min, max } => {
            let value = integer("an integer")?;
            if stored_ranges {
                return wide(value);
            }
            if !(i64::from(min)..=i64::from(max)).contains(&value) {
                return Err(SettingsError::OutOfRange {
                    key: dotted.to_string(),
                    value,
                    range: range_text(kind),
                });
            }
            Ok(u8::try_from(value).expect("the range check bounds it"))
        }
        Kind::Signed7 => {
            let value = integer("an integer")?;
            if stored_ranges {
                return wide(value + 7);
            }
            if !(-7..=7).contains(&value) {
                return Err(SettingsError::OutOfRange {
                    key: dotted.to_string(),
                    value,
                    range: range_text(kind),
                });
            }
            Ok(u8::try_from(value + 7).expect("in 0..=14"))
        }
        Kind::OneBased { max } => {
            let value = integer("an integer")?;
            if stored_ranges {
                return wide(value - 1);
            }
            if !(1..=i64::from(max)).contains(&value) {
                return Err(SettingsError::OutOfRange {
                    key: dotted.to_string(),
                    value,
                    range: range_text(kind),
                });
            }
            Ok(u8::try_from(value - 1).expect("in 0..=max-1"))
        }
        Kind::Bool => item
            .as_value()
            .and_then(|v| v.as_bool())
            .map(u8::from)
            .ok_or_else(|| SettingsError::WrongType {
                key: dotted.to_string(),
                expected: "true or false",
            }),
        Kind::Labels(labels) => {
            let Some(text) = item.as_value().and_then(|v| v.as_str()) else {
                return Err(SettingsError::WrongType {
                    key: dotted.to_string(),
                    expected: "a string",
                });
            };
            labels
                .iter()
                .position(|label| *label == text)
                .map(|index| u8::try_from(index).expect("labels fit u8"))
                .ok_or_else(|| SettingsError::UnknownLabel {
                    key: dotted.to_string(),
                    label: text.to_string(),
                    allowed: range_text(kind),
                })
        }
    }
}

/// The spec's table path under the appearance root `root` ("appearance" →
/// `root`, "appearance.physique" → `root.physique`).
fn absolute(root: &str, table: &str) -> String {
    format!("{}{}", root, &table["appearance".len()..])
}

/// Everything the value walk did not consume is refused. A leaf is known
/// iff its dotted path is `name` or some key's; a table is known iff its
/// path is `appearance` or some key's table. The first unknown item is
/// `UnknownKey`; a table position holding a non-table is `WrongType`.
fn reject_unknown(document: &DocumentMut) -> Result<(), SettingsError> {
    for (name, item) in document.iter() {
        match name {
            "name" => {}
            "appearance" => reject_unknown_table(item, "appearance", "appearance")?,
            _ => {
                return Err(SettingsError::UnknownKey {
                    key: name.to_string(),
                });
            }
        }
    }
    Ok(())
}

/// The recursive half of [`reject_unknown`] and [`parse_appearance`]: `root`
/// is the appearance table's own dotted path, `path` the table being walked.
fn reject_unknown_table(item: &Item, root: &str, path: &str) -> Result<(), SettingsError> {
    let Some(table) = item.as_table_like() else {
        return Err(SettingsError::WrongType {
            key: path.to_string(),
            expected: "a table",
        });
    };
    for (leaf, sub) in table.iter() {
        let child = format!("{path}.{leaf}");
        if SettingKey::ALL
            .iter()
            .any(|key| absolute(root, key.spec().table) == child)
        {
            if !sub.is_table_like() {
                return Err(SettingsError::WrongType {
                    key: child,
                    expected: "a table",
                });
            }
            reject_unknown_table(sub, root, &child)?;
        } else if !SettingKey::ALL
            .iter()
            .any(|key| absolute(root, key.spec().table) == path && key.spec().name == leaf)
        {
            return Err(SettingsError::UnknownKey { key: child });
        }
    }
    Ok(())
}

/// The `[appearance]` table at `item` (dotted path `path`: "appearance" in a
/// settings.toml, "players.03.appearance" in a team.toml) parsed into stored
/// values. Shared with `team_toml` so the nested key handling has one home.
pub(crate) fn parse_appearance(
    item: &Item,
    path: &str,
    stored_ranges: bool,
) -> Result<AppearanceSettings, SettingsError> {
    let Some(appearance) = item.as_table_like() else {
        return Err(SettingsError::WrongType {
            key: path.to_string(),
            expected: "a table",
        });
    };
    let mut out = AppearanceSettings::default();
    for key in SettingKey::ALL {
        let spec = key.spec();
        let Some(item) = lookup(appearance, &spec, path)? else {
            continue;
        };
        let leaf = leaf_path(path, &spec);
        let stored = value(&leaf, spec.kind, item, stored_ranges)?;
        set_appearance(&mut out, key, Some(stored));
    }
    reject_unknown_table(item, path, path)?;
    Ok(out)
}

/// The `[prefix]` table plus its four subtables appended to `out`, in key
/// order: a `Some` emits its line with the key's comment, a `None` the
/// commented neutral line. Shared with `team_toml`'s `[players.NN.appearance]`.
pub(crate) fn emit_appearance(
    out: &mut String,
    prefix: &str,
    appearance: &AppearanceSettings,
    stored_ranges: bool,
) -> Result<(), SettingsError> {
    let mut table = "";
    for key in SettingKey::ALL {
        let spec = key.spec();
        if spec.table != table {
            out.push_str(&format!("\n[{}]\n", absolute(prefix, spec.table)));
            table = spec.table;
        }
        let stored = get_appearance(appearance, key);
        let body = match stored {
            Some(stored) => format!(
                "{} = {}",
                spec.name,
                toml_form(&spec, stored, stored_ranges)?
            ),
            None => format!("{} = {}", spec.name, neutral_text(spec.kind)),
        };
        let line = padded(&body, spec.comment);
        match stored {
            Some(_) => out.push_str(&line),
            None => out.push_str(&format!("# {line}")),
        }
        out.push('\n');
    }
    Ok(())
}

impl PlayerSettings {
    /// Reads a `settings.toml` text. Every key is optional; anything the key
    /// table does not know (a stray table, a compiler-owned field such as
    /// `boots_id`) is `UnknownKey`. The first error met, in `SettingKey::ALL`
    /// order then the unknown-key sweep, is the one reported.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let document: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| SettingsError::Toml(e.to_string()))?;
        let mut settings = PlayerSettings::default();
        if let Some(item) = document.get("name") {
            settings.name = Some(match item.as_value() {
                Some(Value::Boolean(b)) if *b.value() => NameSetting::FromFolder,
                Some(Value::String(s)) if s.value().contains('\0') => {
                    return Err(SettingsError::WrongType {
                        key: "name".to_string(),
                        expected: "a string without NUL",
                    });
                }
                Some(Value::String(s)) => NameSetting::Explicit(s.value().clone()),
                _ => {
                    return Err(SettingsError::WrongType {
                        key: "name".to_string(),
                        expected: "true or a string",
                    });
                }
            });
        }
        if let Some(item) = document.get("appearance") {
            settings.appearance = parse_appearance(item, "appearance", false)?;
        }
        reject_unknown(&document)?;
        Ok(settings)
    }

    /// The template `settings.toml`: the plan block verbatim for a settings
    /// where every key is `Some`; an unset key is its line commented out with
    /// the kind's neutral example value. `OutOfRange` for a value written
    /// directly into a public field that the key's kind cannot represent.
    pub fn to_toml(&self) -> Result<String, SettingsError> {
        let mut out = HEADER.to_string();
        match &self.name {
            Some(NameSetting::FromFolder) => out.push_str("name = true\n"),
            Some(NameSetting::Explicit(name)) => {
                out.push_str(&format!("name = {}\n", Value::from(name.as_str())));
            }
            None => out.push_str("# name = true\n"),
        }
        emit_appearance(&mut out, "appearance", &self.appearance, false)?;
        Ok(out)
    }

    /// Rewrites the `Some` values of an existing document, preserving its
    /// comments and formatting; `None` keys are left as the user wrote them
    /// (a commented key stays commented). Missing tables are created as
    /// standard tables; no comments are added. A table position
    /// (`appearance` or one of its four subtables) holding a non-table item
    /// is `WrongType`.
    pub fn update_toml(&self, document: &mut DocumentMut) -> Result<(), SettingsError> {
        fn set(item: &mut Item, value: Value) {
            let decor = item.as_value().map(|old| old.decor().clone());
            *item = Item::Value(value);
            if let (Some(decor), Item::Value(new)) = (decor, item) {
                *new.decor_mut() = decor;
            }
        }

        match &self.name {
            Some(NameSetting::FromFolder) => set(&mut document["name"], true.into()),
            Some(NameSetting::Explicit(name)) => {
                set(&mut document["name"], name.as_str().into());
            }
            None => {}
        }
        for key in SettingKey::ALL {
            let Some(stored) = self.get(key) else {
                continue;
            };
            let spec = key.spec();
            let mut item = document.as_item_mut();
            let mut path = String::new();
            for segment in spec.table.split('.') {
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(segment);
                let child = match item {
                    Item::Table(table) => {
                        if !table.contains_key(segment) {
                            table.insert(segment, Item::Table(toml_edit::Table::new()));
                        }
                        table.get_mut(segment).expect("just inserted")
                    }
                    Item::Value(Value::InlineTable(_)) => {
                        let child = &mut item[segment];
                        if matches!(child, Item::None) {
                            *child = Item::Value(Value::InlineTable(toml_edit::InlineTable::new()));
                        }
                        child
                    }
                    _ => {
                        return Err(SettingsError::WrongType {
                            key: path,
                            expected: "a table",
                        });
                    }
                };
                match child {
                    Item::Table(_) | Item::Value(Value::InlineTable(_)) => item = child,
                    _ => {
                        return Err(SettingsError::WrongType {
                            key: path,
                            expected: "a table",
                        });
                    }
                }
            }
            set(&mut item[spec.name], toml_form(&spec, stored, false)?);
        }
        Ok(())
    }
}
