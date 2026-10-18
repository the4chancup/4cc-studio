//! Cross-version player conversion: rewrites `target`, a player read from a
//! `to`-version save, from `source`, read from a `from`-version save
//! (plan: `spec/pes_savefile/operations.md`, "Cross-version player conversion").
//! Everything that translates one-to-one is copied through; what does not is
//! capped, dropped or filled from the target, and each such case is reported
//! as a `ConvertNote`, never silently. The ingame-face run is copied whole
//! within one of its two layouts (`FaceLayout`) and field by field between
//! them, the target's own hair fields standing. The version-dependent facts
//! live in `schema::playstyle` and `schema::limits`; compile-policy rewrites
//! beyond a straight carry are the Team compiler's, not this module's.

use pes_version::PesVersion;

use crate::codec::CodecError;
use crate::model::ingame_face::IngameFace;
use crate::model::player::PlayerEntry;
#[cfg(test)]
use crate::model::player::STRIPPED_APPEARANCE_ID;
use crate::model::playstyle::PlayStyle;
use crate::schema::fields::{PlayerField, PlayerText};
use crate::schema::ingame_face::{FaceLayout, IngameFaceField};
use crate::schema::limits::{face_type_cap, motion_top};
use crate::schema::{playstyle, schema_for};

/// What could not be carried one-to-one; the conversion still succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertNote {
    /// A face type above the target's cap was reset to 0 rather than clamped
    /// to the cap.
    FaceTypeReset {
        /// The capped field.
        field: IngameFaceField,
        /// The value the source carried.
        value: u8,
    },
    /// Skin colour 7 (custom skin) reset to 1: `from` or `to` has no custom
    /// skin.
    CustomSkinReset,
    /// The playing style has no value in `to`; set to `None` (0).
    PlayingStyleDropped {
        /// The style that does not translate.
        style: PlayStyle,
    },
    /// A set skill `to` has no bit for was cleared.
    SkillDropped {
        /// The cleared skill's canonical index.
        index: u8,
    },
    /// The name or shirt name was cut to the target field's length (at a
    /// char boundary).
    TextTruncated {
        /// The text that was cut.
        text: PlayerText,
    },
    /// PES 15's nose angle has no field on PES 16 to 21: a non-zero one was
    /// dropped, the nose type written alone.
    NoseAngleDropped {
        /// The angle the source carried, -1 or 1.
        angle: i8,
    },
    /// A motion above the target's top was written as the top, which the target's editor
    /// shows for it anyway.
    MotionCapped {
        /// The capped field.
        field: PlayerField,
        /// The value the source carried (stored).
        value: u8,
    },
}

/// A conversion that could not run.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// The source or the target has no ingame-face run (an entry never read
    /// from a record).
    #[error("the source or the target carries no ingame-face run")]
    NoIngameFaceRun,
    /// The source's stored playing style is not a value of `from`'s list, or
    /// an ingame-face field could not be read or written.
    #[error(transparent)]
    Codec(#[from] CodecError),
}

/// `s` cut to at most `max` bytes at a char boundary.
fn cut(s: &str, max: usize) -> String {
    if s.len() <= max {
        return String::from(s);
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    String::from(&s[..end])
}

/// Overwrites `target` only when both sides carry the field (a gated `Option`
/// the source's version lacks leaves the target's `None` standing).
fn carry<T: Copy>(target: &mut Option<T>, source: Option<T>) {
    if let (Some(slot), Some(value)) = (target.as_mut(), source) {
        *slot = value;
    }
}

/// Writes `source`'s ingame-face run, read from a `from`-version save, onto
/// `target`'s, read from a `to`-version save. Within one layout the run is
/// the source's whole. Between PES 15's layout and PES 16 to 21's, each
/// field both hold is read from the source and written on the target, whose
/// own hair fields and unused bits stand as the template player holds them:
/// the two layouts store different hair fields at the same bytes, so a byte
/// copy would read one version's hair as the other's. The nose type is
/// written alone, at angle 0: the angle PES 15 codes with it has no field on
/// 16 to 21 (a non-zero one is noted). A 16 to 21 nose type past 21 has no
/// PES 15 coding (3 × type + the angle's code in six bits) and is refused as
/// `ValueTooWide` rather than written wrong; no game writes one, the widest
/// cap being 7.
fn carry_face(
    target: &mut IngameFace,
    source: &IngameFace,
    from: PesVersion,
    to: PesVersion,
    notes: &mut Vec<ConvertNote>,
) -> Result<(), CodecError> {
    if FaceLayout::of(from) == FaceLayout::of(to) {
        *target = source.clone();
        return Ok(());
    }
    for field in IngameFaceField::ALL {
        if field.layout().is_some() || field == IngameFaceField::NoseType {
            continue;
        }
        target.set(field, source.get(field)?)?;
    }
    let (nose_type, angle) = source.nose()?;
    target.set_nose(nose_type, 0)?;
    if angle != 0 {
        notes.push(ConvertNote::NoseAngleDropped { angle });
    }
    Ok(())
}

/// Rewrites `target`, a player read from a `to`-version save, from `source`,
/// read from a `from`-version save. All-or-nothing: on `Err` the target is
/// unchanged. The target keeps its own `id` (it is the slot being filled).
pub fn convert_player(
    source: &PlayerEntry,
    from: PesVersion,
    target: &mut PlayerEntry,
    to: PesVersion,
) -> Result<Vec<ConvertNote>, ConvertError> {
    if source.appearance.ingame_face.bytes().is_empty()
        || target.appearance.ingame_face.bytes().is_empty()
    {
        return Err(ConvertError::NoIngameFaceRun);
    }
    let style = playstyle::decode(from, source.positions.playing_style)?;
    let schema = schema_for(to).player;
    let mut notes = Vec::new();
    let mut out = target.clone();

    for spec in schema.texts {
        let (slot, original) = match spec.text {
            PlayerText::Name => (&mut out.name, &source.name),
            PlayerText::ShirtName => (&mut out.shirt_name, &source.shirt_name),
        };
        // `Name` is UTF-8 (cut at a byte boundary); `ShirtName` is single-byte
        // text on disk, one char per stored byte (cut at a char count).
        let value = match spec.text {
            PlayerText::Name => cut(original, spec.len as usize),
            PlayerText::ShirtName => original.chars().take(spec.len as usize).collect::<String>(),
        };
        if value != *original {
            notes.push(ConvertNote::TextTruncated { text: spec.text });
        }
        *slot = value;
    }

    out.basic.nationality = source.basic.nationality;
    out.basic.age = source.basic.age;
    out.basic.height = source.basic.height;
    out.basic.weight = source.basic.weight;

    out.stats.attacking_prowess = source.stats.attacking_prowess;
    out.stats.ball_control = source.stats.ball_control;
    out.stats.dribbling = source.stats.dribbling;
    out.stats.low_pass = source.stats.low_pass;
    out.stats.lofted_pass = source.stats.lofted_pass;
    out.stats.finishing = source.stats.finishing;
    out.stats.place_kicking = source.stats.place_kicking;
    out.stats.swerve = source.stats.swerve;
    out.stats.heading = source.stats.heading;
    out.stats.defensive_prowess = source.stats.defensive_prowess;
    out.stats.ball_winning = source.stats.ball_winning;
    out.stats.kicking_power = source.stats.kicking_power;
    out.stats.speed = source.stats.speed;
    out.stats.explosive_power = source.stats.explosive_power;
    out.stats.body_control = source.stats.body_control;
    out.stats.jump = source.stats.jump;
    out.stats.goalkeeping = source.stats.goalkeeping;
    out.stats.stamina = source.stats.stamina;
    out.stats.weak_foot_usage = source.stats.weak_foot_usage;
    out.stats.weak_foot_accuracy = source.stats.weak_foot_accuracy;
    out.stats.form = source.stats.form;
    out.stats.injury_resistance = source.stats.injury_resistance;
    carry(
        &mut out.stats.physical_contact,
        source.stats.physical_contact,
    );
    carry(&mut out.stats.catching, source.stats.catching);
    carry(&mut out.stats.clearing, source.stats.clearing);
    carry(&mut out.stats.reflexes, source.stats.reflexes);
    carry(&mut out.stats.coverage, source.stats.coverage);
    carry(&mut out.stats.star, source.stats.star);
    carry(
        &mut out.stats.tight_possession,
        source.stats.tight_possession,
    );
    carry(&mut out.stats.aggression, source.stats.aggression);
    carry(
        &mut out.stats.playing_attitude,
        source.stats.playing_attitude,
    );

    out.positions.registered = source.positions.registered;
    out.positions.playable = source.positions.playable;
    out.positions.stronger_foot = source.positions.stronger_foot;
    carry(
        &mut out.positions.stronger_hand,
        source.positions.stronger_hand,
    );
    out.positions.playing_style = match playstyle::encode(to, style) {
        Some(index) => index,
        None => {
            notes.push(ConvertNote::PlayingStyleDropped { style });
            0
        }
    };

    out.skills.com_styles = source.skills.com_styles;
    out.skills.skills = source.skills.skills;
    for i in 0..41u8 {
        if !schema.has(PlayerField::Skill(i)) && out.skills.skills[usize::from(i)] {
            out.skills.skills[usize::from(i)] = false;
            notes.push(ConvertNote::SkillDropped { index: i });
        }
    }

    // The nine measured motions copy capped at `to`'s top: the target's
    // editor shows a past-top value as the top anyway, and a copied one can
    // be rewritten to the list's first entry on the player's next in-game
    // save (the plan's "Cross-version player conversion").
    for field in [
        PlayerField::HunchingDribbling,
        PlayerField::HunchingRunning,
        PlayerField::ArmMovementDribbling,
        PlayerField::ArmMovementRunning,
        PlayerField::CornerKickMotion,
        PlayerField::FreeKickMotion,
        PlayerField::PenaltyKickMotion,
        PlayerField::GoalCelebration1,
        PlayerField::GoalCelebration2,
    ] {
        let value = u8::try_from(source.get(field)?).expect("a motion field is a byte");
        let top = motion_top(to, field).expect("a motion field has a top");
        out.set(field, u32::from(value.min(top)))?;
        if value > top {
            notes.push(ConvertNote::MotionCapped { field, value });
        }
    }
    carry(&mut out.motion.dribbling, source.motion.dribbling);

    out.edit_flags.player = source.edit_flags.player;
    out.edit_flags.basic_settings = source.edit_flags.basic_settings;
    out.edit_flags.registered_position = source.edit_flags.registered_position;
    out.edit_flags.playable_positions = source.edit_flags.playable_positions;
    out.edit_flags.abilities = source.edit_flags.abilities;
    out.edit_flags.skills = source.edit_flags.skills;
    out.edit_flags.playing_style = source.edit_flags.playing_style;
    out.edit_flags.com_styles = source.edit_flags.com_styles;
    out.edit_flags.motion = source.edit_flags.motion;
    carry(&mut out.edit_flags.base_copy, source.edit_flags.base_copy);
    out.edit_flags.face = source.edit_flags.face;
    out.edit_flags.hair = source.edit_flags.hair;
    out.edit_flags.physique = source.edit_flags.physique;
    out.edit_flags.strip = source.edit_flags.strip;

    out.appearance.boots_id = source.appearance.boots_id;
    out.appearance.gloves_id = source.appearance.gloves_id;
    out.appearance.base_copy_id = source.appearance.base_copy_id;
    out.appearance.neck_length = source.appearance.neck_length;
    out.appearance.neck_size = source.appearance.neck_size;
    out.appearance.shoulder_height = source.appearance.shoulder_height;
    out.appearance.shoulder_width = source.appearance.shoulder_width;
    out.appearance.chest = source.appearance.chest;
    out.appearance.waist = source.appearance.waist;
    out.appearance.arm_size = source.appearance.arm_size;
    out.appearance.arm_length = source.appearance.arm_length;
    out.appearance.thigh = source.appearance.thigh;
    out.appearance.calf = source.appearance.calf;
    out.appearance.leg_length = source.appearance.leg_length;
    out.appearance.head_length = source.appearance.head_length;
    out.appearance.head_width = source.appearance.head_width;
    out.appearance.head_depth = source.appearance.head_depth;
    out.appearance.wrist_tape_color_left = source.appearance.wrist_tape_color_left;
    out.appearance.wrist_tape_color_right = source.appearance.wrist_tape_color_right;
    out.appearance.wrist_taping = source.appearance.wrist_taping;
    out.appearance.spectacles_color = source.appearance.spectacles_color;
    out.appearance.spectacles_style = source.appearance.spectacles_style;
    out.appearance.sleeves = source.appearance.sleeves;
    out.appearance.inners = source.appearance.inners;
    out.appearance.socks = source.appearance.socks;
    out.appearance.undershorts = source.appearance.undershorts;
    out.appearance.untucked = source.appearance.untucked;
    out.appearance.ankle_taping = source.appearance.ankle_taping;

    carry_face(
        &mut out.appearance.ingame_face,
        &source.appearance.ingame_face,
        from,
        to,
        &mut notes,
    )?;
    for field in IngameFaceField::ALL {
        let Some(cap) = face_type_cap(to, field) else {
            continue;
        };
        let face = &mut out.appearance.ingame_face;
        // The cap is on the nose type, not on PES 15's field, which codes
        // the angle with it; a reset keeps the angle.
        let nose = field == IngameFaceField::NoseType;
        let (value, angle) = if nose {
            face.nose()?
        } else {
            (face.get(field)?, 0)
        };
        if value > cap {
            if nose {
                face.set_nose(0, angle)?;
            } else {
                face.set(field, 0)?;
            }
            notes.push(ConvertNote::FaceTypeReset { field, value });
        }
    }
    if out.appearance.ingame_face.get(IngameFaceField::SkinColor)? == 7
        && !(fpc::custom_skin_available(from) && fpc::custom_skin_available(to))
    {
        out.appearance
            .ingame_face
            .set(IngameFaceField::SkinColor, 1)?;
        notes.push(ConvertNote::CustomSkinReset);
    }

    *target = out;
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;

    use super::*;
    use crate::codec::{read_player, write_player};
    use crate::model::ingame_face::{IngameFace, in_layout};
    use crate::schema::ingame_face::FaceLayout;
    use crate::settings_toml::PlayerSettings;
    use crate::test_support::{FIXTURES, appearance_for, count, find_player, payload, record};

    use ConvertNote as N;
    use IngameFaceField as F;
    use PesVersion as V;

    /// The first player of `version`'s fixture (appearance record included on
    /// 15/16).
    fn first(version: PesVersion) -> PlayerEntry {
        let payload = payload(version);
        let schema = schema_for(version);
        let first = read_player(
            record(&payload, &schema.players, schema.player.size, 0),
            schema.player,
        )
        .expect("decode");
        find_player(&payload, schema, first.id)
    }

    /// `player` writes into copies of `version`'s player record (and, on
    /// 15/16, its appearance record): the conversion output is encodable.
    fn writes_ok(version: PesVersion, player: &PlayerEntry) {
        let payload = payload(version);
        let schema = schema_for(version);
        let mut rec = record(&payload, &schema.players, schema.player.size, 0).to_vec();
        write_player(player, &mut rec, schema.player).expect("player record writes");
        if let Some((section, appearance)) = &schema.appearance {
            let rec = (0..count(&payload, section))
                .map(|i| record(&payload, section, appearance.size, i))
                .find(|rec| u32::from_le_bytes(rec[..4].try_into().expect("id")) == player.id)
                .expect("the target's appearance record");
            let mut rec = rec.to_vec();
            write_player(player, &mut rec, appearance).expect("appearance record writes");
        }
    }

    /// The fields both layouts hold, the nose aside (re-coded, not copied).
    fn shared_fields() -> impl Iterator<Item = IngameFaceField> {
        IngameFaceField::ALL
            .into_iter()
            .filter(|field| field.layout().is_none() && *field != F::NoseType)
    }

    /// The first player of `version`'s fixture whose `hair` field is set and
    /// whose run differs from `template`'s in at least five shared fields: a
    /// face whose conversion is visible, with a hair byte a byte copy would
    /// misread.
    fn cup_player(
        version: PesVersion,
        template: &IngameFace,
        hair: IngameFaceField,
    ) -> PlayerEntry {
        let payload = payload(version);
        let schema = schema_for(version);
        for i in 0..count(&payload, &schema.players) {
            let rec = record(&payload, &schema.players, schema.player.size, i);
            let mut player = read_player(rec, schema.player).expect("decode");
            appearance_for(&payload, schema, &mut player);
            let face = &player.appearance.ingame_face;
            let differing = shared_fields()
                .filter(|&field| {
                    face.get(field).expect("shared") != template.get(field).expect("shared")
                })
                .count();
            if face.get(hair).expect("held") != 0 && differing >= 5 {
                return player;
            }
        }
        panic!("no {version:?} fixture player with {hair:?} set differs in five shared fields");
    }

    /// The cross-layout rules: every shared field is the source's (the cap
    /// and the skin rule applied), the nose type the source's, capped, at
    /// angle 0, and the target layout's own hair fields and unused bits the
    /// template's.
    fn assert_face_converted(
        out: &PlayerEntry,
        source: &PlayerEntry,
        template: &IngameFace,
        to: PesVersion,
    ) {
        assert_face_rules(out, source, to);
        let face = &out.appearance.ingame_face;
        let (source_type, _) = source.appearance.ingame_face.nose().expect("reads");
        let cap = face_type_cap(to, F::NoseType).expect("a type");
        let want = if source_type > cap { 0 } else { source_type };
        assert_eq!(face.nose().expect("reads"), (want, 0), "{to:?} nose");
        let own = IngameFaceField::ALL
            .into_iter()
            .filter(|field| field.layout() == Some(FaceLayout::of(to)));
        for field in own {
            assert_eq!(
                face.get(field).expect("held"),
                template.get(field).expect("held"),
                "{to:?} {field:?} is the template's"
            );
        }
        assert_eq!(
            face.unused(),
            template.unused(),
            "{to:?} unused bits are the template's"
        );
    }

    /// The strip/physique copy-through fields of `out` equal `source`'s.
    fn assert_appearance_carried(out: &PlayerEntry, source: &PlayerEntry) {
        let (o, s) = (&out.appearance, &source.appearance);
        assert_eq!(o.boots_id, s.boots_id, "boots_id");
        assert_eq!(o.gloves_id, s.gloves_id, "gloves_id");
        assert_eq!(o.base_copy_id, s.base_copy_id, "base_copy_id");
        assert_eq!(o.neck_length, s.neck_length, "neck_length");
        assert_eq!(o.neck_size, s.neck_size, "neck_size");
        assert_eq!(o.shoulder_height, s.shoulder_height, "shoulder_height");
        assert_eq!(o.shoulder_width, s.shoulder_width, "shoulder_width");
        assert_eq!(o.chest, s.chest, "chest");
        assert_eq!(o.waist, s.waist, "waist");
        assert_eq!(o.arm_size, s.arm_size, "arm_size");
        assert_eq!(o.arm_length, s.arm_length, "arm_length");
        assert_eq!(o.thigh, s.thigh, "thigh");
        assert_eq!(o.calf, s.calf, "calf");
        assert_eq!(o.leg_length, s.leg_length, "leg_length");
        assert_eq!(o.head_length, s.head_length, "head_length");
        assert_eq!(o.head_width, s.head_width, "head_width");
        assert_eq!(o.head_depth, s.head_depth, "head_depth");
        assert_eq!(o.wrist_tape_color_left, s.wrist_tape_color_left);
        assert_eq!(o.wrist_tape_color_right, s.wrist_tape_color_right);
        assert_eq!(o.wrist_taping, s.wrist_taping, "wrist_taping");
        assert_eq!(o.spectacles_color, s.spectacles_color, "spectacles_color");
        assert_eq!(o.spectacles_style, s.spectacles_style, "spectacles_style");
        assert_eq!(o.sleeves, s.sleeves, "sleeves");
        assert_eq!(o.inners, s.inners, "inners");
        assert_eq!(o.socks, s.socks, "socks");
        assert_eq!(o.undershorts, s.undershorts, "undershorts");
        assert_eq!(o.untucked, s.untucked, "untucked");
        assert_eq!(o.ankle_taping, s.ankle_taping, "ankle_taping");
    }

    /// Each face type is `min`-capped against `to`; the skin carries unless 7
    /// hits the custom-skin rule. Only the fields both runs hold: a layout's
    /// own hair fields are not carried. The nose is capped on the target's
    /// nose type, PES 15's angle aside.
    fn assert_face_rules(out: &PlayerEntry, source: &PlayerEntry, to: PesVersion) {
        let (face, from) = (&out.appearance.ingame_face, &source.appearance.ingame_face);
        let cap = face_type_cap(to, F::NoseType).expect("a type");
        assert!(face.nose().expect("reads").0 <= cap, "{to:?} nose type");
        for field in IngameFaceField::ALL {
            let held = |run: &IngameFace| in_layout(field.layout(), run.layout());
            if field == F::NoseType || !held(face) || !held(from) {
                continue;
            }
            let got = out.appearance.ingame_face.get(field).expect("covered");
            let src = source.appearance.ingame_face.get(field).expect("covered");
            let want = match face_type_cap(to, field) {
                Some(cap) if src > cap => 0,
                Some(_) => src,
                None if field == F::SkinColor && src == 7 => {
                    if fpc::custom_skin_available(to) {
                        7
                    } else {
                        1
                    }
                }
                None => src,
            };
            assert_eq!(got, want, "{to:?} {field:?}");
        }
    }

    #[test]
    fn a_pes19_player_converts_into_a_pes16_template() {
        let source = first(V::Pes19);
        let mut target = first(V::Pes16);
        let before = target.clone();
        let schema = schema_for(V::Pes16).player;
        assert_eq!(
            schema.texts.iter().map(|spec| spec.len).collect::<Vec<_>>(),
            [46, 16]
        );
        let notes = convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.id, before.id, "the slot's id stands");
        assert_eq!(target.name, source.name, "a 19 name fits 46 bytes");
        assert_eq!(target.shirt_name, source.shirt_name);
        assert_appearance_carried(&target, &source);
        assert_face_rules(&target, &source, V::Pes16);
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::NoseAngleDropped { .. })),
            "one layout has no angle to drop: {notes:?}"
        );
        writes_ok(V::Pes16, &target);
    }

    #[test]
    fn a_pes16_player_converts_into_a_pes21_template() {
        let source = first(V::Pes16);
        let mut target = first(V::Pes21);
        convert_player(&source, V::Pes16, &mut target, V::Pes21).expect("converts");
        assert_eq!(target.name, source.name);
        assert_eq!(target.shirt_name, source.shirt_name);
        assert_appearance_carried(&target, &source);
        assert_face_rules(&target, &source, V::Pes21);
        writes_ok(V::Pes21, &target);
    }

    /// Within one layout the run is the source's, byte for byte (PES 18 to
    /// 21: no cap narrows and neither has a custom skin, so nothing resets).
    #[test]
    fn a_conversion_within_one_layout_copies_the_run_whole() {
        let source = first(V::Pes18);
        let mut target = first(V::Pes21);
        assert_ne!(
            target.appearance.ingame_face.bytes(),
            source.appearance.ingame_face.bytes(),
            "the two runs differ"
        );
        convert_player(&source, V::Pes18, &mut target, V::Pes21).expect("converts");
        assert_eq!(
            target.appearance.ingame_face.bytes(),
            source.appearance.ingame_face.bytes()
        );
        writes_ok(V::Pes21, &target);
    }

    /// A PES 15 cup player converted to PES 21 keeps his face and gets the
    /// template's hair; a PES 21 one converted to PES 15 the reverse, at
    /// nose angle 0.
    #[test]
    fn a_face_converts_field_by_field_between_the_layouts() {
        // The search lands on player 70101 of the PES 15 fixture, 22 shared
        // fields apart from the PES 21 template.
        let template21 = first(V::Pes21);
        let source15 = cup_player(
            V::Pes15,
            &template21.appearance.ingame_face,
            F::Pes15HairVariation,
        );
        // Premise: byte 34 is the variation on 15 and the hairstyle and
        // cropped style on 21, and the two differ, so a byte copy would show.
        assert_ne!(
            source15.appearance.ingame_face.bytes()[34],
            template21.appearance.ingame_face.bytes()[34]
        );
        let mut target = template21.clone();
        let notes = convert_player(&source15, V::Pes15, &mut target, V::Pes21).expect("converts");
        assert_face_converted(
            &target,
            &source15,
            &template21.appearance.ingame_face,
            V::Pes21,
        );
        let (_, angle) = source15.appearance.ingame_face.nose().expect("reads");
        assert_eq!(
            notes.contains(&N::NoseAngleDropped { angle }),
            angle != 0,
            "a non-zero angle is noted: {notes:?}"
        );
        writes_ok(V::Pes21, &target);

        // The search lands on player 70101 of the PES 21 fixture, 22 shared
        // fields apart from the PES 15 template.
        let template15 = first(V::Pes15);
        let source21 = cup_player(V::Pes21, &template15.appearance.ingame_face, F::HairStyle);
        assert_ne!(
            source21.appearance.ingame_face.bytes()[34],
            template15.appearance.ingame_face.bytes()[34]
        );
        let mut target = template15.clone();
        let notes = convert_player(&source21, V::Pes21, &mut target, V::Pes15).expect("converts");
        assert_face_converted(
            &target,
            &source21,
            &template15.appearance.ingame_face,
            V::Pes15,
        );
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::NoseAngleDropped { .. })),
            "PES 21 has no angle to drop: {notes:?}"
        );
        writes_ok(V::Pes15, &target);
    }

    /// The nose is capped on its type, PES 15's angle kept: a PES 15 field of
    /// 40 is type 13, angle -1 (40 = 3 × 13 + 1), past PES 15's cap of 11,
    /// and resets to type 0 with that angle within the layout. Into PES 16
    /// the angle has no field and is noted; the type, past 16's cap of 6,
    /// resets too.
    #[test]
    fn a_nose_type_above_the_cap_resets_and_keeps_the_angle() {
        let mut source = first(V::Pes15);
        source
            .appearance
            .ingame_face
            .set(F::NoseType, 40)
            .expect("fits the field");
        assert_eq!(
            source.appearance.ingame_face.nose().expect("reads"),
            (13, -1)
        );
        let mut target = first(V::Pes15);
        let notes = convert_player(&source, V::Pes15, &mut target, V::Pes15).expect("converts");
        assert_eq!(
            target.appearance.ingame_face.nose().expect("reads"),
            (0, -1)
        );
        assert!(notes.contains(&N::FaceTypeReset {
            field: F::NoseType,
            value: 13,
        }));

        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes15, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.appearance.ingame_face.nose().expect("reads"), (0, 0));
        assert!(notes.contains(&N::FaceTypeReset {
            field: F::NoseType,
            value: 13,
        }));
        assert!(notes.contains(&N::NoseAngleDropped { angle: -1 }));
    }

    /// A 16 to 21 nose type past 21 has no PES 15 coding (3 × type + the
    /// angle's code in six bits): refused, the target unchanged.
    #[test]
    fn a_nose_type_with_no_pes15_coding_is_refused() {
        let mut source = first(V::Pes16);
        source
            .appearance
            .ingame_face
            .set(F::NoseType, 22)
            .expect("fits the field");
        let mut target = first(V::Pes15);
        let before = target.clone();
        assert!(matches!(
            convert_player(&source, V::Pes16, &mut target, V::Pes15),
            Err(ConvertError::Codec(CodecError::ValueTooWide { .. }))
        ));
        assert_eq!(target, before, "all-or-nothing");

        source
            .appearance
            .ingame_face
            .set(F::NoseType, 21)
            .expect("fits the field");
        let notes = convert_player(&source, V::Pes16, &mut target, V::Pes15).expect("converts");
        assert_eq!(target.appearance.ingame_face.nose().expect("reads"), (0, 0));
        assert!(notes.contains(&N::FaceTypeReset {
            field: F::NoseType,
            value: 21,
        }));
    }

    #[test]
    fn a_type_above_the_target_cap_resets_to_zero_with_a_note() {
        let mut source = first(V::Pes21);
        source
            .appearance
            .ingame_face
            .set(F::FacialHairType, 19)
            .expect("fits PES 21's width");
        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes21, &mut target, V::Pes16).expect("converts");
        assert_eq!(
            target
                .appearance
                .ingame_face
                .get(F::FacialHairType)
                .expect("covered"),
            0
        );
        assert!(notes.contains(&N::FaceTypeReset {
            field: F::FacialHairType,
            value: 19,
        }));

        let mut target21 = first(V::Pes21);
        let notes = convert_player(&source, V::Pes21, &mut target21, V::Pes21).expect("converts");
        assert_eq!(
            target21
                .appearance
                .ingame_face
                .get(F::FacialHairType)
                .expect("covered"),
            19
        );
        assert!(!notes.iter().any(|note| matches!(
            note,
            N::FaceTypeReset {
                field: F::FacialHairType,
                ..
            }
        )));
    }

    #[test]
    fn custom_skin_survives_only_between_versions_that_have_it() {
        let mut source = first(V::Pes16);
        source
            .appearance
            .ingame_face
            .set(F::SkinColor, 7)
            .expect("fits");
        let mut target17 = first(V::Pes17);
        let notes = convert_player(&source, V::Pes16, &mut target17, V::Pes17).expect("converts");
        assert_eq!(
            target17
                .appearance
                .ingame_face
                .get(F::SkinColor)
                .expect("covered"),
            7
        );
        assert!(!notes.contains(&N::CustomSkinReset));

        let mut target18 = first(V::Pes18);
        let notes = convert_player(&source, V::Pes16, &mut target18, V::Pes18).expect("converts");
        assert_eq!(
            target18
                .appearance
                .ingame_face
                .get(F::SkinColor)
                .expect("covered"),
            1
        );
        assert!(notes.contains(&N::CustomSkinReset));

        let mut source19 = first(V::Pes19);
        source19
            .appearance
            .ingame_face
            .set(F::SkinColor, 7)
            .expect("fits");
        let mut target16 = first(V::Pes16);
        let notes = convert_player(&source19, V::Pes19, &mut target16, V::Pes16).expect("converts");
        assert_eq!(
            target16
                .appearance
                .ingame_face
                .get(F::SkinColor)
                .expect("covered"),
            1
        );
        assert!(notes.contains(&N::CustomSkinReset));
    }

    #[test]
    fn the_playing_style_translates_drops_and_refuses() {
        let mut source = first(V::Pes19);
        source.positions.playing_style = 7; // RoamingFlank, absent on 16
        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.positions.playing_style, 0);
        assert!(notes.contains(&N::PlayingStyleDropped {
            style: PlayStyle::RoamingFlank,
        }));

        let mut source = first(V::Pes16);
        source.positions.playing_style = 17; // OffensiveGoalkeeper
        let mut target = first(V::Pes19);
        let notes = convert_player(&source, V::Pes16, &mut target, V::Pes19).expect("converts");
        assert_eq!(target.positions.playing_style, 20);
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::PlayingStyleDropped { .. }))
        );

        let mut source = first(V::Pes16);
        source.positions.playing_style = 16; // the hole
        let mut target = first(V::Pes19);
        let before = target.clone();
        assert!(matches!(
            convert_player(&source, V::Pes16, &mut target, V::Pes19),
            Err(ConvertError::Codec(CodecError::UnknownPlayingStyle { .. }))
        ));
        assert_eq!(target, before, "all-or-nothing");
    }

    /// The `SkillDropped` indices of a conversion's notes.
    fn dropped(notes: &[ConvertNote]) -> Vec<u8> {
        notes
            .iter()
            .filter_map(|note| {
                if let N::SkillDropped { index } = note {
                    Some(*index)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn skills_the_target_lacks_are_dropped_with_notes() {
        // PES 15's skill rows (schema/pes15.rs): 0-5, 7-9, 11-15, 18-20,
        // 22-23, 25-27; every other index drops.
        const PES15_DROPS: &[u8] = &[
            6, 10, 16, 17, 21, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ];
        // PES 16's skills are one array of 28 (schema/pes16.rs `Skill` spec).
        const PES16_DROPS: &[u8] = &[28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40];

        let mut source = first(V::Pes21);
        source.skills.skills = [true; 41];
        let mut target = first(V::Pes15);
        let notes = convert_player(&source, V::Pes21, &mut target, V::Pes15).expect("converts");
        assert_eq!(dropped(&notes), PES15_DROPS);
        for i in 0..41u8 {
            assert_eq!(
                target.skills.skills[usize::from(i)],
                !PES15_DROPS.contains(&i),
                "skill {i}"
            );
        }

        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes21, &mut target, V::Pes16).expect("converts");
        assert_eq!(dropped(&notes), PES16_DROPS);
        for i in 0..28usize {
            assert!(target.skills.skills[i], "skill {i} survives the array path");
        }
        for &i in PES16_DROPS {
            assert!(!target.skills.skills[usize::from(i)]);
        }

        let mut target21 = first(V::Pes21);
        let notes = convert_player(&source, V::Pes21, &mut target21, V::Pes21).expect("converts");
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::SkillDropped { .. })),
            "PES 21 has every skill"
        );
    }

    #[test]
    fn gated_fields_carry_only_where_the_target_has_them() {
        let source = first(V::Pes16);
        assert_eq!(source.stats.star, None, "PES 16 has no star");
        let mut target = first(V::Pes19);
        let kept = target.stats.star;
        assert!(kept.is_some(), "the PES 19 template carries a star");
        convert_player(&source, V::Pes16, &mut target, V::Pes19).expect("converts");
        assert_eq!(target.stats.star, kept, "the template's star stands");

        let source = first(V::Pes19);
        assert!(source.stats.star.is_some());
        let mut target = first(V::Pes16);
        convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.stats.star, None);

        let mut source = first(V::Pes19);
        source.stats.star = Some(4);
        let mut target = first(V::Pes21);
        target.stats.star = Some(1);
        convert_player(&source, V::Pes19, &mut target, V::Pes21).expect("converts");
        assert_eq!(target.stats.star, Some(4), "both sides have it: carried");
    }

    #[test]
    fn the_base_copy_flag_carries_only_between_versions_that_have_it() {
        let source = first(V::Pes19);
        let mut target = first(V::Pes18);
        target.edit_flags.base_copy = Some(true);
        convert_player(&source, V::Pes19, &mut target, V::Pes18).expect("converts");
        assert_eq!(
            target.edit_flags.base_copy,
            Some(true),
            "PES 19 has no flag: the template's stands"
        );

        for (from, to) in [(true, false), (false, true)] {
            let mut source = first(V::Pes18);
            source.edit_flags.base_copy = Some(from);
            let mut target = first(V::Pes18);
            target.edit_flags.base_copy = Some(to);
            convert_player(&source, V::Pes18, &mut target, V::Pes18).expect("converts");
            assert_eq!(
                target.edit_flags.base_copy,
                Some(from),
                "both sides have it: carried"
            );
        }
    }

    #[test]
    fn an_entry_without_a_run_is_refused() {
        let mut target = first(V::Pes16);
        let before = target.clone();
        assert!(matches!(
            convert_player(&PlayerEntry::default(), V::Pes19, &mut target, V::Pes16),
            Err(ConvertError::NoIngameFaceRun)
        ));
        assert_eq!(target, before);

        let source = first(V::Pes16);
        let mut empty = PlayerEntry::default();
        assert!(matches!(
            convert_player(&source, V::Pes16, &mut empty, V::Pes19),
            Err(ConvertError::NoIngameFaceRun)
        ));
        assert_eq!(empty, PlayerEntry::default());
    }

    #[test]
    fn a_full_field_name_carries_whole_with_no_note() {
        // PES 17 and 18 name fields are both 46 bytes: a name that fills its
        // field (the census's no-NUL shape) converts untouched.
        let mut source = first(V::Pes17);
        source.name = "a".repeat(46);
        let mut target = first(V::Pes18);
        let notes = convert_player(&source, V::Pes17, &mut target, V::Pes18).expect("converts");
        assert_eq!(target.name, source.name);
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::TextTruncated { .. })),
            "{notes:?}"
        );
        writes_ok(V::Pes18, &target);
    }

    #[test]
    fn text_cuts_at_the_target_length_on_a_char_boundary() {
        let mut source = first(V::Pes19);
        source.name = "A".repeat(60);
        source.shirt_name = String::from("SHORT");
        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.name.len(), 46);
        assert_eq!(
            notes
                .iter()
                .filter(|note| matches!(note, N::TextTruncated { .. }))
                .collect::<Vec<_>>(),
            [&N::TextTruncated {
                text: PlayerText::Name,
            }]
        );
        writes_ok(V::Pes16, &target);

        // The cut at byte 46 lands inside the 3-byte '€': it backs off to 44.
        source.name = format!("{}€{}", "a".repeat(44), "b".repeat(10));
        let mut target = first(V::Pes16);
        convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.name.len(), 44);
        assert!(target.name.is_char_boundary(target.name.len()));
        writes_ok(V::Pes16, &target);
    }

    /// A motion past the target's measured top is written as the top and
    /// noted; a target at least as wide keeps the value with no note.
    #[test]
    fn a_motion_above_the_target_top_is_capped_and_noted() {
        // Every motion at PES 20's top (stored; the 1-based tops minus 1,
        // the celebrations as shown).
        let mut source = first(V::Pes20);
        source.motion.hunching_dribbling = 4;
        source.motion.hunching_running = 4;
        source.motion.arm_movement_dribbling = 9;
        source.motion.arm_movement_running = 9;
        source.motion.corner_kick = 9;
        source.motion.free_kick = 19;
        source.motion.penalty_kick = 6;
        source.motion.dribbling = Some(3);
        source.motion.goal_celebration_1 = 152;
        source.motion.goal_celebration_2 = 152;

        let mut target = first(V::Pes15);
        let notes = convert_player(&source, V::Pes20, &mut target, V::Pes15).expect("converts");
        let motion = &target.motion;
        assert_eq!(
            (
                motion.hunching_dribbling,
                motion.hunching_running,
                motion.arm_movement_dribbling,
                motion.arm_movement_running,
                motion.corner_kick,
                motion.free_kick,
                motion.penalty_kick,
                motion.goal_celebration_1,
                motion.goal_celebration_2,
            ),
            (2, 2, 4, 4, 5, 15, 3, 112, 112),
            "PES 15's tops, stored"
        );
        assert_eq!(motion.dribbling, None, "PES 15 has no dribbling field");
        let capped: Vec<(PlayerField, u8)> = notes
            .iter()
            .filter_map(|note| {
                if let N::MotionCapped { field, value } = note {
                    Some((*field, *value))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            capped,
            [
                (PlayerField::HunchingDribbling, 4),
                (PlayerField::HunchingRunning, 4),
                (PlayerField::ArmMovementDribbling, 9),
                (PlayerField::ArmMovementRunning, 9),
                (PlayerField::CornerKickMotion, 9),
                (PlayerField::FreeKickMotion, 19),
                (PlayerField::PenaltyKickMotion, 6),
                (PlayerField::GoalCelebration1, 152),
                (PlayerField::GoalCelebration2, 152),
            ],
            "the nine motions noted with the source's stored values"
        );
        writes_ok(V::Pes15, &target);

        let mut target21 = first(V::Pes21);
        let notes = convert_player(&source, V::Pes20, &mut target21, V::Pes21).expect("converts");
        assert_eq!(target21.motion, source.motion, "unchanged");
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::MotionCapped { .. })),
            "{notes:?}"
        );
    }

    /// PES 19's editor offers cheek types to 5 and forehead types to 6: a
    /// player carrying them reads into a `settings.toml` without a refusal,
    /// keeps them on a PES 19 target, keeps them on PES 18 too (its
    /// unmeasured caps are the fields' bits), and resets to 0 into PES 16,
    /// whose measured caps hold.
    #[test]
    fn pes19s_wider_cheek_and_forehead_types() {
        let mut source = first(V::Pes19);
        source
            .appearance
            .ingame_face
            .set(F::CheekType, 5)
            .expect("fits the field");
        source
            .appearance
            .ingame_face
            .set(F::ForeheadType, 6)
            .expect("fits the field");
        PlayerSettings::from_player(&source).expect("reads without a refusal");

        let mut target19 = first(V::Pes19);
        let notes = convert_player(&source, V::Pes19, &mut target19, V::Pes19).expect("converts");
        assert_eq!(
            target19
                .appearance
                .ingame_face
                .get(F::CheekType)
                .expect("covered"),
            5
        );
        assert_eq!(
            target19
                .appearance
                .ingame_face
                .get(F::ForeheadType)
                .expect("covered"),
            6
        );
        assert!(
            !notes.iter().any(|note| matches!(
                note,
                N::FaceTypeReset {
                    field: F::CheekType | F::ForeheadType,
                    ..
                }
            )),
            "{notes:?}"
        );

        let mut target18 = first(V::Pes18);
        let notes = convert_player(&source, V::Pes19, &mut target18, V::Pes18).expect("converts");
        assert_eq!(
            target18
                .appearance
                .ingame_face
                .get(F::CheekType)
                .expect("covered"),
            5,
            "PES 18's unmeasured cheek cap is the field's bits"
        );
        assert_eq!(
            target18
                .appearance
                .ingame_face
                .get(F::ForeheadType)
                .expect("covered"),
            6,
            "PES 18's unmeasured forehead cap is the field's bits"
        );
        assert!(
            !notes
                .iter()
                .any(|note| matches!(note, N::FaceTypeReset { .. })),
            "{notes:?}"
        );

        let mut target16 = first(V::Pes16);
        let notes = convert_player(&source, V::Pes19, &mut target16, V::Pes16).expect("converts");
        assert_eq!(
            target16
                .appearance
                .ingame_face
                .get(F::CheekType)
                .expect("covered"),
            0
        );
        assert!(notes.contains(&N::FaceTypeReset {
            field: F::CheekType,
            value: 5,
        }));
        assert_eq!(
            target16
                .appearance
                .ingame_face
                .get(F::ForeheadType)
                .expect("covered"),
            0
        );
        assert!(notes.contains(&N::FaceTypeReset {
            field: F::ForeheadType,
            value: 6,
        }));
    }

    #[test]
    fn the_shirt_name_counts_chars_not_utf8_bytes() {
        // ShirtName is single-byte text on disk: 15 'é' is a 15-byte payload.
        let mut source = first(V::Pes16);
        source.shirt_name = "é".repeat(15);
        let mut target = first(V::Pes19);
        let notes = convert_player(&source, V::Pes16, &mut target, V::Pes19).expect("converts");
        assert_eq!(target.shirt_name, "é".repeat(15));
        assert!(!notes.iter().any(|note| matches!(
            note,
            N::TextTruncated {
                text: PlayerText::ShirtName,
            }
        )));

        source.shirt_name = "é".repeat(17);
        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes16, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.shirt_name, "é".repeat(16));
        assert!(notes.contains(&N::TextTruncated {
            text: PlayerText::ShirtName,
        }));
        writes_ok(V::Pes16, &target);
    }

    /// The appearance id is the slot's, not the source's: the conversion
    /// leaves the target's alone, a -1 and a 15/16 `None` included.
    #[test]
    fn the_appearance_id_stays_the_targets() {
        let mut source = first(V::Pes19);
        source.appearance_id = Some(0);
        let mut target = first(V::Pes21);
        let kept = target.appearance_id;
        assert_eq!(kept, Some(target.id));
        assert_ne!(Some(0), kept, "the value discriminates a carry");
        convert_player(&source, V::Pes19, &mut target, V::Pes21).expect("converts");
        assert_eq!(target.appearance_id, kept, "the target's stands");

        let mut target = first(V::Pes21);
        target.appearance_id = Some(STRIPPED_APPEARANCE_ID);
        convert_player(&source, V::Pes19, &mut target, V::Pes21).expect("converts");
        assert_eq!(
            target.appearance_id,
            Some(STRIPPED_APPEARANCE_ID),
            "a -1 stands too"
        );

        let mut target = first(V::Pes16);
        assert_eq!(target.appearance_id, None);
        convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.appearance_id, None, "15/16 has no such field");
    }

    #[test]
    fn every_fixture_pair_converts_and_writes() {
        for &from in &FIXTURES {
            let source = first(from);
            for &to in &FIXTURES {
                let mut target = first(to);
                convert_player(&source, from, &mut target, to)
                    .unwrap_or_else(|e| panic!("{from:?} -> {to:?}: {e}"));
                writes_ok(to, &target);
            }
        }
    }
}
