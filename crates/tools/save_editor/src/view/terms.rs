//! The tabs' terms for a `diff` row (`plans/save_editor.md` "Comparator
//! (gameplay + aesthetics)", the row bullet): a field a tab draws reads
//! under the tab's own label, its old and new values as that tab shows
//! them, so a label or scale changed in a tab changes the row with it. A
//! field no tab draws keeps the report's name and values
//! (`messages::diff_fields`).

use pes_savefile::interchange::team_toml::labels::{
    ABILITY_STATS, COM_STYLES, PLAYABLE_POSITIONS, POSITIONS, SKILLS, title_case,
};
use pes_savefile::ops::compare::PlayerDiff;
use pes_savefile::schema::fields::{ABILITY_FIELDS, PlayerField, PlayerText};
use pes_savefile::schema::ingame_face::IngameFaceField;
use pes_savefile::settings_toml::{Kind, SettingKey};
use pes_version::PesVersion;
use team_widgets::cards;

use crate::messages::diff_fields;
use crate::view::appearance::{self, fields};
use crate::view::{abilities, info_strip};

/// A `diff` row's terms: the field's label and its two values as the tab
/// that owns the field shows them.
pub(super) struct Term {
    /// The field's label: the tab's own where a tab draws the field.
    pub(super) label: String,
    /// `old` as the tab shows it.
    pub(super) old: String,
    /// `new` the same way.
    pub(super) new: String,
}

/// `diff`'s row in the tabs' terms.
pub(super) fn row_term(version: PesVersion, diff: &PlayerDiff) -> Term {
    match diff {
        PlayerDiff::Field { .. } => player_field_term(version, diff),
        PlayerDiff::Face { .. } => face_field_term(version, diff),
        PlayerDiff::Text { text, old, new } => match text {
            PlayerText::Name => Term {
                label: info_strip::NAME_LABEL.to_owned(),
                old: info_strip::shown_name(old),
                new: info_strip::shown_name(new),
            },
            PlayerText::ShirtName => Term {
                label: info_strip::SHIRT_LABEL.to_owned(),
                old: old.clone(),
                new: new.clone(),
            },
        },
        // The run's fingerprint row is drawn by no tab.
        PlayerDiff::FaceRun { .. } => report_term(diff),
    }
}

/// The report's own terms for `diff`: the field's schema name and its
/// stored values — the row for a field no tab draws.
fn report_term(diff: &PlayerDiff) -> Term {
    let (label, old, new) = diff_fields(diff);
    Term { label, old, new }
}

/// A `Field` row's term: the tabs' own tables first (`SMALL_STATS`,
/// `MOTIONS`, `PHYSIQUE`, `STRIP_ENUMS`, the ability stats), then the
/// fields the tabs draw one-off, then the report's terms for a field no
/// tab draws — the `_` arm IS the rule for those, not an oversight.
fn player_field_term(version: PesVersion, diff: &PlayerDiff) -> Term {
    let PlayerDiff::Field { field, old, new } = *diff else {
        unreachable!("player_field_term gets a Field row")
    };
    if let Some(small) = abilities::SMALL_STATS
        .iter()
        .find(|stat| stat.field == field)
    {
        let shift = u32::from(small.shift);
        return numbers(small.label, old + shift, new + shift);
    }
    if let Some(index) = ABILITY_FIELDS.iter().position(|ability| *ability == field) {
        return numbers(title_case(ABILITY_STATS[index]), old, new);
    }
    if let Some(motion) = fields::MOTIONS.iter().find(|row| row.field == field) {
        let shift = u32::from(motion.one_based);
        return numbers(motion.label, old + shift, new + shift);
    }
    if let Some((label, key, ..)) = fields::PHYSIQUE
        .iter()
        .find(|(_, key, ..)| key.player_field() == Some(field))
    {
        let Kind::Signed { limit } = key.spec().kind else {
            unreachable!("the physique keys are Signed");
        };
        let limit = i64::from(limit);
        return Term {
            label: (*label).to_owned(),
            old: (i64::from(old) - limit).to_string(),
            new: (i64::from(new) - limit).to_string(),
        };
    }
    if let Some((label, key, ..)) = fields::STRIP_ENUMS
        .iter()
        .find(|(_, key, ..)| key.player_field() == Some(field))
    {
        return term(
            *label,
            fields::option_name(*key, stored_u8(old)),
            fields::option_name(*key, stored_u8(new)),
        );
    }
    match field {
        PlayerField::Height => numbers(info_strip::HEIGHT_LABEL, old, new),
        PlayerField::Weight => numbers(info_strip::WEIGHT_LABEL, old, new),
        PlayerField::Age => numbers(info_strip::AGE_LABEL, old, new),
        PlayerField::Nationality => numbers(info_strip::NATION_LABEL, old, new),
        PlayerField::RegisteredPosition => term(
            abilities::REGISTERED_POSITION_LABEL,
            abilities::position_label(stored_u8(old)),
            abilities::position_label(stored_u8(new)),
        ),
        PlayerField::PlayingStyle => term(
            abilities::PLAYING_STYLE_LABEL,
            cards::playing_style_text(version, stored_u8(old)),
            cards::playing_style_text(version, stored_u8(new)),
        ),
        PlayerField::StrongerFoot => term(
            abilities::STRONGER_FOOT_LABEL,
            abilities::foot_label(stored_u8(old)),
            abilities::foot_label(stored_u8(new)),
        ),
        PlayerField::StrongerHand => term(
            abilities::STRONGER_HAND_LABEL,
            abilities::foot_label(stored_u8(old)),
            abilities::foot_label(stored_u8(new)),
        ),
        PlayerField::PlayablePosition(index) => term(
            // The group's heading then the row's own label: a bare position
            // name would read as a registered position's value.
            format!(
                "{} {}",
                abilities::PLAYABLE_POSITIONS_LABEL,
                POSITIONS[PLAYABLE_POSITIONS[usize::from(index)]]
            ),
            rating_label(old),
            rating_label(new),
        ),
        PlayerField::Skill(index) => term(
            title_case(SKILLS[usize::from(index)]),
            on_off(old),
            on_off(new),
        ),
        PlayerField::ComStyle(index) => term(
            title_case(COM_STYLES[usize::from(index)]),
            on_off(old),
            on_off(new),
        ),
        PlayerField::DribblingMotion => numbers(appearance::DRIBBLING_MOTION_LABEL, old, new),
        PlayerField::BootsId => numbers(appearance::BOOTS_ID_LABEL, old, new),
        PlayerField::GlovesId => numbers(appearance::GLOVES_ID_LABEL, old, new),
        PlayerField::BaseCopyId => numbers(appearance::COPY_FROM_ID_LABEL, old, new),
        PlayerField::WristTaping => keyed(field, appearance::WRIST_TAPING_LABEL, old, new),
        PlayerField::SpectaclesStyle => keyed(field, appearance::SPECTACLES_LABEL, old, new),
        PlayerField::Untucked => flag(appearance::SHIRTTAIL_OUT_LABEL, old, new),
        PlayerField::AnkleTaping => flag(appearance::ANKLE_TAPING_LABEL, old, new),
        PlayerField::EditedFace => flag(appearance::FACE_LABEL, old, new),
        PlayerField::EditedHair => flag(appearance::HAIR_LABEL, old, new),
        PlayerField::EditedPhysique => flag(appearance::PHYSIQUE_LABEL, old, new),
        PlayerField::EditedStrip => flag(appearance::STRIP_LABEL, old, new),
        PlayerField::EditedMotion => flag(appearance::MOTION_LABEL, old, new),
        PlayerField::BaseCopy => flag(appearance::BASE_COPY_LABEL, old, new),
        PlayerField::WristTapeColorLeft
        | PlayerField::WristTapeColorRight
        | PlayerField::SpectaclesColor => {
            let key = key_writing(|key| key.player_field() == Some(field))
                .expect("the strip palettes have settings keys");
            let palette = fields::palette(version, key);
            Term {
                label: palette.label.to_owned(),
                old: palette.shown(stored_u8(old)),
                new: palette.shown(stored_u8(new)),
            }
        }
        // The rest is drawn by no tab (the ids and gameplay edit flags
        // among them): the report's terms.
        _ => report_term(diff),
    }
}

/// A `Face` row's term: the checkbox and the palettes are drawn, the rest
/// of the run's fields are not (the report's terms).
fn face_field_term(version: PesVersion, diff: &PlayerDiff) -> Term {
    let PlayerDiff::Face { field, old, new } = *diff else {
        unreachable!("face_field_term gets a Face row")
    };
    match field {
        IngameFaceField::PlayerGloves => Term {
            label: appearance::PLAYER_GLOVES_LABEL.to_owned(),
            old: on_off(u32::from(old)).to_owned(),
            new: on_off(u32::from(new)).to_owned(),
        },
        IngameFaceField::SkinColor
        | IngameFaceField::IrisColor
        | IngameFaceField::PlayerGlovesColor => {
            let key = key_writing(|key| key.face_field() == Some(field))
                .expect("the colour fields have settings keys");
            let palette = fields::palette(version, key);
            Term {
                label: palette.label.to_owned(),
                old: palette.shown(old),
                new: palette.shown(new),
            }
        }
        _ => report_term(diff),
    }
}

/// A `SettingKey`-keyed field's term: the combo's selected text at the
/// stored index (`fields::option_name`).
fn keyed(field: PlayerField, label: &'static str, old: u32, new: u32) -> Term {
    let key = key_writing(|key| key.player_field() == Some(field))
        .expect("the keyed fields have settings keys");
    term(
        label,
        fields::option_name(key, stored_u8(old)),
        fields::option_name(key, stored_u8(new)),
    )
}

/// A term over shown texts.
fn term(label: impl Into<String>, old: impl Into<String>, new: impl Into<String>) -> Term {
    Term {
        label: label.into(),
        old: old.into(),
        new: new.into(),
    }
}

/// A term over stored numbers shown as stored (`old`/`new` unchanged).
fn numbers(label: impl Into<String>, old: u32, new: u32) -> Term {
    term(label, old.to_string(), new.to_string())
}

/// A checkbox field's term: a set bit reads "on" (the draft's `T_flag`).
fn flag(label: &'static str, old: u32, new: u32) -> Term {
    term(label, on_off(old), on_off(new))
}

/// `stored` narrowed to the byte a byte-sized control stores (the
/// combos, the palettes, and the position, style, foot and hand pickers).
fn stored_u8(stored: u32) -> u8 {
    u8::try_from(stored).expect("a byte-sized control's field is at most eight bits")
}

/// `stored`'s rating letter; a stored 3, which selects none of the three.
fn rating_label(stored: u32) -> &'static str {
    abilities::RATING_LABELS
        .iter()
        .find(|(_, rating)| u32::from(*rating) == stored)
        .map_or("none", |(label, _)| *label)
}

/// `stored`'s checkbox reading: a set bit reads "on".
fn on_off(stored: u32) -> &'static str {
    if stored == 0 { "off" } else { "on" }
}

/// The `SettingKey` that `writes` (`keys.rs`' `player_field` or
/// `face_field` compared with a field): the key table's own binding, so a
/// key remapped to another field remaps here.
fn key_writing(writes: impl Fn(SettingKey) -> bool) -> Option<SettingKey> {
    SettingKey::ALL.iter().copied().find(|key| writes(*key))
}

#[cfg(test)]
mod tests {
    use pes_savefile::ops::fingerprint::FaceHash;
    use pes_savefile::{EditFile, fixture_save};

    use super::*;

    /// A player of the PES 17 fixture's team 734 ("Muh Spooks"), for real
    /// stored values.
    fn muh_spooks() -> pes_savefile::model::PlayerEntry {
        EditFile::from_bytes(&fixture_save(PesVersion::Pes17))
            .expect("the fixture loads")
            .player(73401)
            .expect("73401 is a player of team 734")
            .clone()
    }

    fn field(field: PlayerField, old: u32, new: u32) -> PlayerDiff {
        PlayerDiff::Field { field, old, new }
    }

    fn face_row(field: IngameFaceField, old: u8, new: u8) -> PlayerDiff {
        PlayerDiff::Face { field, old, new }
    }

    /// `term` reads (label, old, new) — the literals the tab draws, never
    /// the consts under test.
    fn assert_term(term: &Term, label: &str, old: &str, new: &str) {
        assert_eq!(
            (term.label.as_str(), term.old.as_str(), term.new.as_str()),
            (label, old, new)
        );
    }

    /// The info strip's fields: its labels and the stored value as is, the
    /// name with its `\x11` spelled as the strip spells it.
    #[test]
    fn the_info_strips_fields() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert_eq!(
            (
                player.basic.height,
                player.basic.age,
                player.basic.nationality
            ),
            (185, 20, 210),
            "premise: 73401's fixture basics"
        );
        assert_term(
            &row_term(version, &field(PlayerField::Height, 185, 190)),
            "Height",
            "185",
            "190",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Weight, 85, 80)),
            "Weight",
            "85",
            "80",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Age, 20, 21)),
            "Age",
            "20",
            "21",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Nationality, 210, 0)),
            "Nation",
            "210",
            "0",
        );
        assert_eq!(player.name, "Muh Spooks", "premise: 73401's name");
        assert_term(
            &row_term(
                version,
                &PlayerDiff::Text {
                    text: PlayerText::Name,
                    old: "Muh \x11Spooks".to_owned(),
                    new: player.name.clone(),
                },
            ),
            "Name",
            "Muh \\x11Spooks",
            "Muh Spooks",
        );
        assert_eq!(player.shirt_name, "SEXY MAX", "premise: 73401's shirt");
        assert_term(
            &row_term(
                version,
                &PlayerDiff::Text {
                    text: PlayerText::ShirtName,
                    old: player.shirt_name.clone(),
                    new: "NO NAME".to_owned(),
                },
            ),
            "Shirt",
            "SEXY MAX",
            "NO NAME",
        );
    }

    /// The first row's combos: the position and style names and the foot's
    /// label as the tab shows them.
    #[test]
    fn the_first_rows_fields() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert_eq!(player.positions.registered, 1, "premise: a CB");
        assert_term(
            &row_term(version, &field(PlayerField::RegisteredPosition, 1, 0)),
            "Registered position",
            "CB",
            "GK",
        );
        // PES 17's stored 15 is the Build Up of the version's own list.
        assert_eq!(player.positions.playing_style, 15, "premise");
        assert_term(
            &row_term(version, &field(PlayerField::PlayingStyle, 15, 1)),
            "Playing style",
            "Build Up",
            "Goal Poacher",
        );
        // The style is the version's: PES 21's stored 15 is another one.
        assert_term(
            &row_term(PesVersion::Pes21, &field(PlayerField::PlayingStyle, 15, 1)),
            "Playing style",
            "Offensive Fullback",
            "Goal Poacher",
        );
        // A stored value PES 17's list cannot decode shows as its number,
        // as the picker shows it.
        assert_term(
            &row_term(version, &field(PlayerField::PlayingStyle, 15, 22)),
            "Playing style",
            "Build Up",
            "22",
        );
        assert_eq!(player.positions.stronger_foot, 1, "premise: left-footed");
        assert_term(
            &row_term(version, &field(PlayerField::StrongerFoot, 1, 0)),
            "Stronger foot",
            "Left",
            "Right",
        );
        assert_term(
            &row_term(PesVersion::Pes21, &field(PlayerField::StrongerHand, 0, 1)),
            "Stronger hand",
            "Right",
            "Left",
        );
    }

    /// The small stats' +1 shift, and the ability stats' title-cased table
    /// names with the stored number.
    #[test]
    fn the_small_stats_and_abilities_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert_eq!(player.stats.weak_foot_usage, 1, "premise");
        // A weak foot usage stored 2 shows 3.
        assert_term(
            &row_term(version, &field(PlayerField::WeakFootUsage, 1, 2)),
            "Weak foot usage",
            "2",
            "3",
        );
        assert_term(
            &row_term(version, &field(PlayerField::WeakFootAccuracy, 3, 0)),
            "Weak foot accuracy",
            "4",
            "1",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Form, 3, 7)),
            "Form",
            "4",
            "8",
        );
        assert_term(
            &row_term(version, &field(PlayerField::InjuryResistance, 0, 2)),
            "Injury resistance",
            "1",
            "3",
        );
        // Star and playing attitude (PES 19/20+) show as stored.
        assert_term(
            &row_term(PesVersion::Pes21, &field(PlayerField::Star, 0, 7)),
            "Star",
            "0",
            "7",
        );
        assert_term(
            &row_term(
                PesVersion::Pes21,
                &field(PlayerField::PlayingAttitude, 3, 0),
            ),
            "Playing attitude",
            "3",
            "0",
        );
        assert_eq!(player.stats.speed, 77, "premise");
        assert_term(
            &row_term(version, &field(PlayerField::Speed, 77, 40)),
            "Speed",
            "77",
            "40",
        );
        assert_term(
            &row_term(version, &field(PlayerField::AttackingProwess, 90, 99)),
            "Attacking Prowess",
            "90",
            "99",
        );
    }

    /// The playable positions: "Playable position <POS>" with the rating's
    /// letter; a stored 3, which selects none of the three.
    #[test]
    fn the_playable_positions_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        // 73401's only A rating sits at playable index 9, the CB.
        assert_eq!(player.positions.playable[9], 2, "premise");
        assert_term(
            &row_term(version, &field(PlayerField::PlayablePosition(9), 2, 0)),
            "Playable positions CB",
            "A",
            "C",
        );
        assert_term(
            &row_term(version, &field(PlayerField::PlayablePosition(0), 3, 1)),
            "Playable positions CF",
            "none",
            "B",
        );
    }

    /// Skills and COM styles read "on"/"off" under the card's own name.
    #[test]
    fn the_skills_and_com_styles_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert!(
            !player.skills.skills[0] && !player.skills.com_styles[6],
            "premise"
        );
        assert_term(
            &row_term(version, &field(PlayerField::Skill(0), 0, 1)),
            "Scissors Feint",
            "off",
            "on",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Skill(40), 1, 0)),
            "Through Passing",
            "on",
            "off",
        );
        assert_term(
            &row_term(version, &field(PlayerField::ComStyle(6), 0, 1)),
            "Long Ranger",
            "off",
            "on",
        );
    }

    /// The 1-based motions show one above the stored, the celebrations and
    /// the dribbling motion as stored.
    #[test]
    fn the_motion_fields_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert_eq!(player.motion.free_kick, 0, "premise");
        // A motion stored 14 shows 15.
        assert_term(
            &row_term(version, &field(PlayerField::FreeKickMotion, 0, 14)),
            "Free kick",
            "1",
            "15",
        );
        assert_term(
            &row_term(version, &field(PlayerField::HunchingDribbling, 0, 2)),
            "Hunching (dribbling)",
            "1",
            "3",
        );
        assert_term(
            &row_term(version, &field(PlayerField::ArmMovementRunning, 1, 0)),
            "Arm movement (running)",
            "2",
            "1",
        );
        assert_term(
            &row_term(version, &field(PlayerField::CornerKickMotion, 0, 5)),
            "Corner kick",
            "1",
            "6",
        );
        assert_term(
            &row_term(version, &field(PlayerField::PenaltyKickMotion, 2, 0)),
            "Penalty kick",
            "3",
            "1",
        );
        assert_eq!(player.motion.goal_celebration_1, 1, "premise");
        assert_term(
            &row_term(version, &field(PlayerField::GoalCelebration1, 1, 0)),
            "Goal celebration 1",
            "1",
            "0",
        );
        assert_term(
            &row_term(version, &field(PlayerField::GoalCelebration2, 0, 7)),
            "Goal celebration 2",
            "0",
            "7",
        );
        assert_term(
            &row_term(
                PesVersion::Pes21,
                &field(PlayerField::DribblingMotion, 3, 0),
            ),
            "Dribbling motion",
            "3",
            "0",
        );
    }

    /// The physique fields on their Signed scale: stored - 7.
    #[test]
    fn the_physique_fields_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert_eq!(player.appearance.neck_length, 6, "premise");
        // A physique stored 7 shows 0, 6 shows -1.
        assert_term(
            &row_term(version, &field(PlayerField::NeckLength, 6, 7)),
            "Neck length",
            "-1",
            "0",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Waist, 14, 0)),
            "Waist",
            "7",
            "-7",
        );
    }

    /// The strip's fields: the combos' picked texts, the checkboxes'
    /// "on"/"off", the palettes' names and numbers, the ids as stored.
    #[test]
    fn the_strip_fields_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert_eq!(
            (
                player.appearance.sleeves,
                player.appearance.socks,
                player.appearance.undershorts
            ),
            (1, 1, 2),
            "premise"
        );
        assert_term(
            &row_term(version, &field(PlayerField::Sleeves, 1, 0)),
            "Sleeves",
            "Short",
            "Seasonal",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Inners, 0, 2)),
            "Inners",
            "Off",
            "Turtleneck",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Socks, 1, 2)),
            "Socks",
            "Long",
            "Short",
        );
        assert_term(
            &row_term(version, &field(PlayerField::Undershorts, 2, 0)),
            "Undershorts",
            "Winter Long",
            "Off",
        );
        assert_term(
            &row_term(version, &field(PlayerField::WristTaping, 0, 3)),
            "Wrist taping",
            "Off",
            "Both",
        );
        assert_term(
            &row_term(version, &field(PlayerField::SpectaclesStyle, 0, 7)),
            "Spectacles",
            "None",
            "Round full frame",
        );
        assert!(
            player.appearance.untucked && !player.appearance.ankle_taping,
            "premise"
        );
        assert_term(
            &row_term(version, &field(PlayerField::Untucked, 1, 0)),
            "Shirttail out",
            "on",
            "off",
        );
        assert_term(
            &row_term(version, &field(PlayerField::AnkleTaping, 0, 1)),
            "Ankle taping",
            "off",
            "on",
        );
        assert_eq!(
            (
                player.appearance.wrist_tape_color_left,
                player.appearance.spectacles_color
            ),
            (0, 0),
            "premise"
        );
        assert_term(
            &row_term(version, &field(PlayerField::WristTapeColorLeft, 0, 4)),
            "Wrist tape left",
            "0",
            "4",
        );
        assert_term(
            &row_term(version, &field(PlayerField::WristTapeColorRight, 4, 0)),
            "Wrist tape right",
            "4",
            "0",
        );
        assert_term(
            &row_term(version, &field(PlayerField::SpectaclesColor, 0, 3)),
            "Spectacles colour",
            "White",
            "Blue",
        );
        assert_eq!(
            (
                player.appearance.boots_id,
                player.appearance.gloves_id,
                player.appearance.base_copy_id
            ),
            (0, 0, 0),
            "premise"
        );
        assert_term(
            &row_term(version, &field(PlayerField::BootsId, 0, 3601)),
            "Boots ID",
            "0",
            "3601",
        );
        assert_term(
            &row_term(version, &field(PlayerField::GlovesId, 0, 627)),
            "Gloves ID",
            "0",
            "627",
        );
        assert_term(
            &row_term(version, &field(PlayerField::BaseCopyId, 0, 70101)),
            "Copy from ID",
            "0",
            "70101",
        );
    }

    /// The edit flags' checkboxes read "on"/"off" under the tab's labels.
    #[test]
    fn the_edit_flags_rows() {
        let version = PesVersion::Pes17;
        let player = muh_spooks();
        assert!(
            !player.edit_flags.face && player.edit_flags.base_copy == Some(false),
            "premise"
        );
        for (flag_field, label) in [
            (PlayerField::EditedFace, "Face"),
            (PlayerField::EditedHair, "Hair"),
            (PlayerField::EditedPhysique, "Physique"),
            (PlayerField::EditedStrip, "Strip"),
            (PlayerField::EditedMotion, "Motion"),
            (PlayerField::BaseCopy, "Base copy"),
        ] {
            assert_term(
                &row_term(version, &field(flag_field, 0, 1)),
                label,
                "off",
                "on",
            );
        }
    }

    /// The ingame-face fields the tab draws: the gloves checkbox and the
    /// colour palettes. The Custom skin is pre-Fox only — on Fox a stored
    /// 7 draws its number.
    #[test]
    fn the_face_fields_rows() {
        let player = muh_spooks();
        let get = |field| {
            player
                .appearance
                .ingame_face
                .get(field)
                .expect("the run holds the field")
        };
        assert_eq!(
            (
                get(IngameFaceField::SkinColor),
                get(IngameFaceField::IrisColor),
                get(IngameFaceField::PlayerGloves)
            ),
            (1, 2, 0),
            "premise"
        );
        assert_term(
            &row_term(
                PesVersion::Pes17,
                &face_row(IngameFaceField::SkinColor, 1, 7),
            ),
            "Skin",
            "Light",
            "Custom",
        );
        assert_term(
            &row_term(
                PesVersion::Pes21,
                &face_row(IngameFaceField::SkinColor, 1, 7),
            ),
            "Skin",
            "Light",
            "7",
        );
        assert_term(
            &row_term(
                PesVersion::Pes17,
                &face_row(IngameFaceField::IrisColor, 2, 10),
            ),
            "Iris",
            "Brown",
            "Violet",
        );
        assert_term(
            &row_term(
                PesVersion::Pes17,
                &face_row(IngameFaceField::PlayerGlovesColor, 0, 6),
            ),
            "Gloves colour",
            "0",
            "6",
        );
        assert_term(
            &row_term(
                PesVersion::Pes17,
                &face_row(IngameFaceField::PlayerGloves, 0, 1),
            ),
            "Player gloves",
            "off",
            "on",
        );
    }

    /// A field no tab draws (a face field, the gameplay edit flags, the
    /// run's fingerprint) keeps the report's name and values.
    #[test]
    fn fields_no_tab_draws_keep_the_reports_terms() {
        let version = PesVersion::Pes17;
        assert_term(
            &row_term(version, &face_row(IngameFaceField::NoseHeight, 3, 5)),
            "NoseHeight",
            "3",
            "5",
        );
        assert_term(
            &row_term(version, &field(PlayerField::EditedPlayer, 0, 1)),
            "EditedPlayer",
            "0",
            "1",
        );
        assert_term(
            &row_term(
                version,
                &PlayerDiff::FaceRun {
                    old: FaceHash([0; 4]),
                    new: FaceHash([0xff; 4]),
                },
            ),
            "face_run",
            "00000000",
            "ffffffff",
        );
    }

    /// Every table-driven tab row has a term: a field added to a tab
    /// without one falls back to the report's schema name, which is not
    /// its label — a drift this test fails on.
    #[test]
    fn every_table_driven_field_has_a_term() {
        let version = PesVersion::Pes17;
        for stat in abilities::SMALL_STATS {
            let term = row_term(version, &field(stat.field, 0, 0));
            assert_eq!(term.label, stat.label, "{:?}", stat.field);
        }
        for motion in fields::MOTIONS {
            let term = row_term(version, &field(motion.field, 0, 0));
            assert_eq!(term.label, motion.label, "{:?}", motion.field);
        }
        for (label, key, ..) in fields::PHYSIQUE {
            let field = key
                .player_field()
                .expect("a physique key's is a player field");
            let term = row_term(version, &self::field(field, 0, 0));
            assert_eq!(term.label, label, "{field:?}");
        }
        for (label, key, ..) in fields::STRIP_ENUMS {
            let field = key.player_field().expect("a strip key's is a player field");
            let term = row_term(version, &self::field(field, 0, 0));
            assert_eq!(term.label, label, "{field:?}");
        }
    }
}
