//! `PlayerEntry`: the version-independent player model. Fields a version lacks
//! are `Option` (the record has no run for them) or defaulted to 0/false; the
//! schema decides what is read and written, the model just holds the result.

use crate::codec::{self, CodecError};
use crate::schema::fields::PlayerField;

/// One player of the save: identity, abilities, positions, skills, motion, the
/// game's own edit flags and the appearance block (a separate record on PES
/// 15/16, part of the player record later).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerEntry {
    /// The player id (team id x 100 + slot for cup players).
    pub id: u32,
    /// Display name, colour codes included.
    pub name: String,
    /// Shirt name (single-byte text on disk).
    pub shirt_name: String,
    /// Nationality and physique basics.
    pub basic: PlayerBasics,
    /// The ability numbers plus form, injury and the version-gated ratings.
    pub stats: PlayerStats,
    /// Registered/playable positions and the playing style.
    pub positions: PlayerPositions,
    /// Skill and COM-playing-style flags.
    pub skills: PlayerSkills,
    /// Motion selections (hunching, arms, kick motions, celebrations).
    pub motion: PlayerMotion,
    /// The game's own "was edited" flags, one per editor section.
    pub edit_flags: PlayerEditFlags,
    /// Boots/gloves/base-copy ids, physique measurements and kit styling.
    pub appearance: PlayerAppearance,
}

/// Nationality and physique basics.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerBasics {
    /// Nationality id.
    pub nationality: u16,
    /// Age.
    pub age: u8,
    /// Height in cm.
    pub height: u8,
    /// Weight in kg.
    pub weight: u8,
}

/// The ability numbers. The `Option` fields exist only from the version on
/// that introduced them (`None` = this save's version has no such stat).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerStats {
    /// Attacking Prowess (Offensive Awareness).
    pub attacking_prowess: u8,
    /// Ball Control.
    pub ball_control: u8,
    /// Dribbling.
    pub dribbling: u8,
    /// Low Pass.
    pub low_pass: u8,
    /// Lofted Pass.
    pub lofted_pass: u8,
    /// Finishing.
    pub finishing: u8,
    /// Place Kicking.
    pub place_kicking: u8,
    /// Swerve (Curl).
    pub swerve: u8,
    /// Header (Heading).
    pub heading: u8,
    /// Defensive Prowess (Defensive Awareness).
    pub defensive_prowess: u8,
    /// Ball Winning (Tackling).
    pub ball_winning: u8,
    /// Kicking Power.
    pub kicking_power: u8,
    /// Speed.
    pub speed: u8,
    /// Explosive Power (Acceleration).
    pub explosive_power: u8,
    /// Body Control (Body Balance on PES 16).
    pub body_control: u8,
    /// Physical Contact (PES 17+).
    pub physical_contact: Option<u8>,
    /// Jump.
    pub jump: u8,
    /// Goalkeeping (GK Awareness).
    pub goalkeeping: u8,
    /// Catching (Saving on PES 15; folded into the GK set from PES 16).
    pub catching: Option<u8>,
    /// Clearing (PES 16+).
    pub clearing: Option<u8>,
    /// Reflexes (PES 16+).
    pub reflexes: Option<u8>,
    /// Coverage (PES 16+).
    pub coverage: Option<u8>,
    /// Stamina.
    pub stamina: u8,
    /// Weak Foot Usage.
    pub weak_foot_usage: u8,
    /// Weak Foot Accuracy.
    pub weak_foot_accuracy: u8,
    /// Form (Condition), 1 to 8 stored 0 to 7.
    pub form: u8,
    /// Injury Resistance.
    pub injury_resistance: u8,
    /// Star rating (PES 19+).
    pub star: Option<u8>,
    /// Tight Possession (PES 20+).
    pub tight_possession: Option<u8>,
    /// Aggression (PES 20+).
    pub aggression: Option<u8>,
    /// Playing attitude (PES 20+).
    pub playing_attitude: Option<u8>,
}

/// Registered/playable positions and the playing style.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerPositions {
    /// Registered position, GK 0 to CF 12.
    pub registered: u8,
    /// Playable position rating (0 none, 1 C, 2 B, 3 A), CF 0 to GK 12.
    pub playable: [u8; 13],
    /// Playing style, a version-specific index.
    pub playing_style: u8,
    /// Stronger foot (0 right, 1 left).
    pub stronger_foot: u8,
    /// Stronger hand (PES 20+).
    pub stronger_hand: Option<u8>,
}

/// Skill and COM-playing-style flags.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSkills {
    /// Player skills, 0 Scissors Feint to 40 Through Passing. Older versions
    /// carry fewer skills; the schema maps them into these canonical slots.
    pub skills: [bool; 41],
    /// COM playing styles, 0 Trickster to 6 Long Ranger.
    pub com_styles: [bool; 7],
}

// [bool; 41] is past the size std derives `Default` for.
impl Default for PlayerSkills {
    fn default() -> Self {
        Self {
            skills: [false; 41],
            com_styles: [false; 7],
        }
    }
}

/// Motion selections.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerMotion {
    /// Hunching while dribbling.
    pub hunching_dribbling: u8,
    /// Hunching while running.
    pub hunching_running: u8,
    /// Arm movement while dribbling.
    pub arm_movement_dribbling: u8,
    /// Arm movement while running.
    pub arm_movement_running: u8,
    /// Corner kick motion.
    pub corner_kick: u8,
    /// Free kick motion.
    pub free_kick: u8,
    /// Penalty kick motion.
    pub penalty_kick: u8,
    /// Dribbling motion (PES 20+).
    pub dribbling: Option<u8>,
    /// First goal celebration.
    pub goal_celebration_1: u8,
    /// Second goal celebration.
    pub goal_celebration_2: u8,
}

/// The game's own per-section "was edited" flags.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerEditFlags {
    /// The player was created or edited.
    pub player: bool,
    /// The basic settings were edited.
    pub basic_settings: bool,
    /// The registered position was edited.
    pub registered_position: bool,
    /// The playable positions were edited.
    pub playable_positions: bool,
    /// The abilities were edited.
    pub abilities: bool,
    /// The skills were edited.
    pub skills: bool,
    /// The playing style was edited.
    pub playing_style: bool,
    /// The COM playing styles were edited.
    pub com_styles: bool,
    /// The motions were edited.
    pub motion: bool,
    /// The player is a base copy of another.
    pub base_copy: bool,
    /// The face was edited.
    pub face: bool,
    /// The hairstyle was edited.
    pub hair: bool,
    /// The physique was edited.
    pub physique: bool,
    /// The strip style was edited.
    pub strip: bool,
}

/// Boots/gloves/base-copy ids, physique measurements and kit styling.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerAppearance {
    /// Boots model id.
    pub boots_id: u32,
    /// Goalkeeper gloves model id.
    pub gloves_id: u32,
    /// Base copy player id (the player's own id when unset).
    pub base_copy_id: u32,
    /// Neck length.
    pub neck_length: u8,
    /// Neck size.
    pub neck_size: u8,
    /// Shoulder height.
    pub shoulder_height: u8,
    /// Shoulder width.
    pub shoulder_width: u8,
    /// Chest measurement.
    pub chest: u8,
    /// Waist size.
    pub waist: u8,
    /// Arm size.
    pub arm_size: u8,
    /// Arm length.
    pub arm_length: u8,
    /// Thigh size.
    pub thigh: u8,
    /// Calf size.
    pub calf: u8,
    /// Leg length.
    pub leg_length: u8,
    /// Head length.
    pub head_length: u8,
    /// Head width.
    pub head_width: u8,
    /// Head depth.
    pub head_depth: u8,
    /// Left wrist tape colour.
    pub wrist_tape_color_left: u8,
    /// Right wrist tape colour.
    pub wrist_tape_color_right: u8,
    /// Wrist taping.
    pub wrist_taping: u8,
    /// Spectacles frame colour.
    pub spectacles_color: u8,
    /// Spectacles style.
    pub spectacles_style: u8,
    /// Sleeves.
    pub sleeves: u8,
    /// Long-sleeved inners.
    pub inners: u8,
    /// Sock length.
    pub socks: u8,
    /// Undershorts.
    pub undershorts: u8,
    /// Shirttail out.
    pub untucked: bool,
    /// Ankle taping.
    pub ankle_taping: bool,
    /// Player (outfield) gloves.
    pub player_gloves: bool,
    /// Player gloves colour.
    pub player_gloves_color: u8,
    /// Skin colour.
    pub skin_color: u8,
    /// Iris colour.
    pub iris_color: u8,
}

impl PlayerEntry {
    /// The model value a bit run carries, in codec field order.
    pub(crate) fn get(&self, field: PlayerField) -> Result<u32, CodecError> {
        let v = match field {
            PlayerField::Id => self.id,
            PlayerField::Nationality => u32::from(self.basic.nationality),
            PlayerField::Height => u32::from(self.basic.height),
            PlayerField::Weight => u32::from(self.basic.weight),
            PlayerField::GoalCelebration1 => u32::from(self.motion.goal_celebration_1),
            PlayerField::GoalCelebration2 => u32::from(self.motion.goal_celebration_2),
            PlayerField::AttackingProwess => u32::from(self.stats.attacking_prowess),
            PlayerField::DefensiveProwess => u32::from(self.stats.defensive_prowess),
            PlayerField::Goalkeeping => u32::from(self.stats.goalkeeping),
            PlayerField::Dribbling => u32::from(self.stats.dribbling),
            PlayerField::FreeKickMotion => u32::from(self.motion.free_kick),
            PlayerField::Finishing => u32::from(self.stats.finishing),
            PlayerField::LowPass => u32::from(self.stats.low_pass),
            PlayerField::LoftedPass => u32::from(self.stats.lofted_pass),
            PlayerField::Heading => u32::from(self.stats.heading),
            PlayerField::Form => u32::from(self.stats.form),
            PlayerField::EditedPlayer => u32::from(self.edit_flags.player),
            PlayerField::Swerve => u32::from(self.stats.swerve),
            PlayerField::Catching => {
                u32::from(self.stats.catching.ok_or_else(|| codec::missing(field))?)
            }
            PlayerField::Clearing => {
                u32::from(self.stats.clearing.ok_or_else(|| codec::missing(field))?)
            }
            PlayerField::Reflexes => {
                u32::from(self.stats.reflexes.ok_or_else(|| codec::missing(field))?)
            }
            PlayerField::InjuryResistance => u32::from(self.stats.injury_resistance),
            PlayerField::EditedBasicSettings => u32::from(self.edit_flags.basic_settings),
            PlayerField::BodyControl => u32::from(self.stats.body_control),
            PlayerField::PhysicalContact => u32::from(
                self.stats
                    .physical_contact
                    .ok_or_else(|| codec::missing(field))?,
            ),
            PlayerField::KickingPower => u32::from(self.stats.kicking_power),
            PlayerField::ExplosivePower => u32::from(self.stats.explosive_power),
            PlayerField::ArmMovementDribbling => u32::from(self.motion.arm_movement_dribbling),
            PlayerField::EditedRegisteredPosition => u32::from(self.edit_flags.registered_position),
            PlayerField::Age => u32::from(self.basic.age),
            PlayerField::RegisteredPosition => u32::from(self.positions.registered),
            PlayerField::PlayingStyle => u32::from(self.positions.playing_style),
            PlayerField::BallControl => u32::from(self.stats.ball_control),
            PlayerField::BallWinning => u32::from(self.stats.ball_winning),
            PlayerField::WeakFootAccuracy => u32::from(self.stats.weak_foot_accuracy),
            PlayerField::Jump => u32::from(self.stats.jump),
            PlayerField::ArmMovementRunning => u32::from(self.motion.arm_movement_running),
            PlayerField::CornerKickMotion => u32::from(self.motion.corner_kick),
            PlayerField::Coverage => {
                u32::from(self.stats.coverage.ok_or_else(|| codec::missing(field))?)
            }
            PlayerField::WeakFootUsage => u32::from(self.stats.weak_foot_usage),
            PlayerField::PlayablePosition(i) => u32::from(
                *self
                    .positions
                    .playable
                    .get(usize::from(i))
                    .ok_or_else(|| codec::index(field, i))?,
            ),
            PlayerField::HunchingDribbling => u32::from(self.motion.hunching_dribbling),
            PlayerField::HunchingRunning => u32::from(self.motion.hunching_running),
            PlayerField::PenaltyKickMotion => u32::from(self.motion.penalty_kick),
            PlayerField::PlaceKicking => u32::from(self.stats.place_kicking),
            PlayerField::EditedPlayablePositions => u32::from(self.edit_flags.playable_positions),
            PlayerField::EditedAbilities => u32::from(self.edit_flags.abilities),
            PlayerField::EditedSkills => u32::from(self.edit_flags.skills),
            PlayerField::Stamina => u32::from(self.stats.stamina),
            PlayerField::Speed => u32::from(self.stats.speed),
            PlayerField::EditedPlayingStyle => u32::from(self.edit_flags.playing_style),
            PlayerField::EditedComStyles => u32::from(self.edit_flags.com_styles),
            PlayerField::EditedMotion => u32::from(self.edit_flags.motion),
            PlayerField::BaseCopy => u32::from(self.edit_flags.base_copy),
            PlayerField::StrongerFoot => u32::from(self.positions.stronger_foot),
            PlayerField::ComStyle(i) => u32::from(
                *self
                    .skills
                    .com_styles
                    .get(usize::from(i))
                    .ok_or_else(|| codec::index(field, i))?,
            ),
            PlayerField::Skill(i) => u32::from(
                *self
                    .skills
                    .skills
                    .get(usize::from(i))
                    .ok_or_else(|| codec::index(field, i))?,
            ),
            PlayerField::Star => u32::from(self.stats.star.ok_or_else(|| codec::missing(field))?),
            PlayerField::DribblingMotion => {
                u32::from(self.motion.dribbling.ok_or_else(|| codec::missing(field))?)
            }
            PlayerField::TightPossession => u32::from(
                self.stats
                    .tight_possession
                    .ok_or_else(|| codec::missing(field))?,
            ),
            PlayerField::Aggression => {
                u32::from(self.stats.aggression.ok_or_else(|| codec::missing(field))?)
            }
            PlayerField::PlayingAttitude => u32::from(
                self.stats
                    .playing_attitude
                    .ok_or_else(|| codec::missing(field))?,
            ),
            PlayerField::StrongerHand => u32::from(
                self.positions
                    .stronger_hand
                    .ok_or_else(|| codec::missing(field))?,
            ),
            PlayerField::EditedFace => u32::from(self.edit_flags.face),
            PlayerField::EditedHair => u32::from(self.edit_flags.hair),
            PlayerField::EditedPhysique => u32::from(self.edit_flags.physique),
            PlayerField::EditedStrip => u32::from(self.edit_flags.strip),
            PlayerField::BootsId => self.appearance.boots_id,
            PlayerField::GlovesId => self.appearance.gloves_id,
            PlayerField::BaseCopyId => self.appearance.base_copy_id,
            PlayerField::NeckLength => u32::from(self.appearance.neck_length),
            PlayerField::NeckSize => u32::from(self.appearance.neck_size),
            PlayerField::ShoulderHeight => u32::from(self.appearance.shoulder_height),
            PlayerField::ShoulderWidth => u32::from(self.appearance.shoulder_width),
            PlayerField::Chest => u32::from(self.appearance.chest),
            PlayerField::Waist => u32::from(self.appearance.waist),
            PlayerField::ArmSize => u32::from(self.appearance.arm_size),
            PlayerField::ArmLength => u32::from(self.appearance.arm_length),
            PlayerField::Thigh => u32::from(self.appearance.thigh),
            PlayerField::Calf => u32::from(self.appearance.calf),
            PlayerField::LegLength => u32::from(self.appearance.leg_length),
            PlayerField::HeadLength => u32::from(self.appearance.head_length),
            PlayerField::HeadWidth => u32::from(self.appearance.head_width),
            PlayerField::HeadDepth => u32::from(self.appearance.head_depth),
            PlayerField::WristTapeColorLeft => u32::from(self.appearance.wrist_tape_color_left),
            PlayerField::WristTapeColorRight => u32::from(self.appearance.wrist_tape_color_right),
            PlayerField::WristTaping => u32::from(self.appearance.wrist_taping),
            PlayerField::SpectaclesColor => u32::from(self.appearance.spectacles_color),
            PlayerField::SpectaclesStyle => u32::from(self.appearance.spectacles_style),
            PlayerField::Sleeves => u32::from(self.appearance.sleeves),
            PlayerField::Inners => u32::from(self.appearance.inners),
            PlayerField::Socks => u32::from(self.appearance.socks),
            PlayerField::Undershorts => u32::from(self.appearance.undershorts),
            PlayerField::Untucked => u32::from(self.appearance.untucked),
            PlayerField::AnkleTaping => u32::from(self.appearance.ankle_taping),
            PlayerField::PlayerGloves => u32::from(self.appearance.player_gloves),
            PlayerField::PlayerGlovesColor => u32::from(self.appearance.player_gloves_color),
            PlayerField::SkinColor => u32::from(self.appearance.skin_color),
            PlayerField::IrisColor => u32::from(self.appearance.iris_color),
        };
        Ok(v)
    }

    /// Patches a bit-run value into the model, in codec field order. A gated
    /// field becomes `Some` because the schema that produced it has the run.
    pub(crate) fn set(&mut self, field: PlayerField, value: u32) -> Result<(), CodecError> {
        match field {
            PlayerField::Id => self.id = value,
            PlayerField::Nationality => self.basic.nationality = value as u16,
            PlayerField::Height => self.basic.height = value as u8,
            PlayerField::Weight => self.basic.weight = value as u8,
            PlayerField::GoalCelebration1 => self.motion.goal_celebration_1 = value as u8,
            PlayerField::GoalCelebration2 => self.motion.goal_celebration_2 = value as u8,
            PlayerField::AttackingProwess => self.stats.attacking_prowess = value as u8,
            PlayerField::DefensiveProwess => self.stats.defensive_prowess = value as u8,
            PlayerField::Goalkeeping => self.stats.goalkeeping = value as u8,
            PlayerField::Dribbling => self.stats.dribbling = value as u8,
            PlayerField::FreeKickMotion => self.motion.free_kick = value as u8,
            PlayerField::Finishing => self.stats.finishing = value as u8,
            PlayerField::LowPass => self.stats.low_pass = value as u8,
            PlayerField::LoftedPass => self.stats.lofted_pass = value as u8,
            PlayerField::Heading => self.stats.heading = value as u8,
            PlayerField::Form => self.stats.form = value as u8,
            PlayerField::EditedPlayer => self.edit_flags.player = value != 0,
            PlayerField::Swerve => self.stats.swerve = value as u8,
            PlayerField::Catching => self.stats.catching = Some(value as u8),
            PlayerField::Clearing => self.stats.clearing = Some(value as u8),
            PlayerField::Reflexes => self.stats.reflexes = Some(value as u8),
            PlayerField::InjuryResistance => self.stats.injury_resistance = value as u8,
            PlayerField::EditedBasicSettings => self.edit_flags.basic_settings = value != 0,
            PlayerField::BodyControl => self.stats.body_control = value as u8,
            PlayerField::PhysicalContact => self.stats.physical_contact = Some(value as u8),
            PlayerField::KickingPower => self.stats.kicking_power = value as u8,
            PlayerField::ExplosivePower => self.stats.explosive_power = value as u8,
            PlayerField::ArmMovementDribbling => self.motion.arm_movement_dribbling = value as u8,
            PlayerField::EditedRegisteredPosition => {
                self.edit_flags.registered_position = value != 0;
            }
            PlayerField::Age => self.basic.age = value as u8,
            PlayerField::RegisteredPosition => self.positions.registered = value as u8,
            PlayerField::PlayingStyle => self.positions.playing_style = value as u8,
            PlayerField::BallControl => self.stats.ball_control = value as u8,
            PlayerField::BallWinning => self.stats.ball_winning = value as u8,
            PlayerField::WeakFootAccuracy => self.stats.weak_foot_accuracy = value as u8,
            PlayerField::Jump => self.stats.jump = value as u8,
            PlayerField::ArmMovementRunning => self.motion.arm_movement_running = value as u8,
            PlayerField::CornerKickMotion => self.motion.corner_kick = value as u8,
            PlayerField::Coverage => self.stats.coverage = Some(value as u8),
            PlayerField::WeakFootUsage => self.stats.weak_foot_usage = value as u8,
            PlayerField::PlayablePosition(i) => {
                *self
                    .positions
                    .playable
                    .get_mut(usize::from(i))
                    .ok_or_else(|| codec::index(field, i))? = value as u8;
            }
            PlayerField::HunchingDribbling => self.motion.hunching_dribbling = value as u8,
            PlayerField::HunchingRunning => self.motion.hunching_running = value as u8,
            PlayerField::PenaltyKickMotion => self.motion.penalty_kick = value as u8,
            PlayerField::PlaceKicking => self.stats.place_kicking = value as u8,
            PlayerField::EditedPlayablePositions => {
                self.edit_flags.playable_positions = value != 0;
            }
            PlayerField::EditedAbilities => self.edit_flags.abilities = value != 0,
            PlayerField::EditedSkills => self.edit_flags.skills = value != 0,
            PlayerField::Stamina => self.stats.stamina = value as u8,
            PlayerField::Speed => self.stats.speed = value as u8,
            PlayerField::EditedPlayingStyle => self.edit_flags.playing_style = value != 0,
            PlayerField::EditedComStyles => self.edit_flags.com_styles = value != 0,
            PlayerField::EditedMotion => self.edit_flags.motion = value != 0,
            PlayerField::BaseCopy => self.edit_flags.base_copy = value != 0,
            PlayerField::StrongerFoot => self.positions.stronger_foot = value as u8,
            PlayerField::ComStyle(i) => {
                *self
                    .skills
                    .com_styles
                    .get_mut(usize::from(i))
                    .ok_or_else(|| codec::index(field, i))? = value != 0;
            }
            PlayerField::Skill(i) => {
                *self
                    .skills
                    .skills
                    .get_mut(usize::from(i))
                    .ok_or_else(|| codec::index(field, i))? = value != 0;
            }
            PlayerField::Star => self.stats.star = Some(value as u8),
            PlayerField::DribblingMotion => self.motion.dribbling = Some(value as u8),
            PlayerField::TightPossession => self.stats.tight_possession = Some(value as u8),
            PlayerField::Aggression => self.stats.aggression = Some(value as u8),
            PlayerField::PlayingAttitude => self.stats.playing_attitude = Some(value as u8),
            PlayerField::StrongerHand => self.positions.stronger_hand = Some(value as u8),
            PlayerField::EditedFace => self.edit_flags.face = value != 0,
            PlayerField::EditedHair => self.edit_flags.hair = value != 0,
            PlayerField::EditedPhysique => self.edit_flags.physique = value != 0,
            PlayerField::EditedStrip => self.edit_flags.strip = value != 0,
            PlayerField::BootsId => self.appearance.boots_id = value,
            PlayerField::GlovesId => self.appearance.gloves_id = value,
            PlayerField::BaseCopyId => self.appearance.base_copy_id = value,
            PlayerField::NeckLength => self.appearance.neck_length = value as u8,
            PlayerField::NeckSize => self.appearance.neck_size = value as u8,
            PlayerField::ShoulderHeight => self.appearance.shoulder_height = value as u8,
            PlayerField::ShoulderWidth => self.appearance.shoulder_width = value as u8,
            PlayerField::Chest => self.appearance.chest = value as u8,
            PlayerField::Waist => self.appearance.waist = value as u8,
            PlayerField::ArmSize => self.appearance.arm_size = value as u8,
            PlayerField::ArmLength => self.appearance.arm_length = value as u8,
            PlayerField::Thigh => self.appearance.thigh = value as u8,
            PlayerField::Calf => self.appearance.calf = value as u8,
            PlayerField::LegLength => self.appearance.leg_length = value as u8,
            PlayerField::HeadLength => self.appearance.head_length = value as u8,
            PlayerField::HeadWidth => self.appearance.head_width = value as u8,
            PlayerField::HeadDepth => self.appearance.head_depth = value as u8,
            PlayerField::WristTapeColorLeft => self.appearance.wrist_tape_color_left = value as u8,
            PlayerField::WristTapeColorRight => {
                self.appearance.wrist_tape_color_right = value as u8
            }
            PlayerField::WristTaping => self.appearance.wrist_taping = value as u8,
            PlayerField::SpectaclesColor => self.appearance.spectacles_color = value as u8,
            PlayerField::SpectaclesStyle => self.appearance.spectacles_style = value as u8,
            PlayerField::Sleeves => self.appearance.sleeves = value as u8,
            PlayerField::Inners => self.appearance.inners = value as u8,
            PlayerField::Socks => self.appearance.socks = value as u8,
            PlayerField::Undershorts => self.appearance.undershorts = value as u8,
            PlayerField::Untucked => self.appearance.untucked = value != 0,
            PlayerField::AnkleTaping => self.appearance.ankle_taping = value != 0,
            PlayerField::PlayerGloves => self.appearance.player_gloves = value != 0,
            PlayerField::PlayerGlovesColor => self.appearance.player_gloves_color = value as u8,
            PlayerField::SkinColor => self.appearance.skin_color = value as u8,
            PlayerField::IrisColor => self.appearance.iris_color = value as u8,
        }
        Ok(())
    }
}
