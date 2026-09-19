//! Cross-version player conversion: rewrites `target`, a player read from a
//! `to`-version save, from `source`, read from a `from`-version save
//! (plan: `pes_savefile/operations.md`, "Cross-version player conversion").
//! Everything that translates one-to-one is copied through; what does not is
//! capped, dropped or filled from the target, and each such case is reported
//! as a `ConvertNote`, never silently. The version-dependent facts live in
//! `schema::playstyle` and `schema::limits`; the compile-policy rewrites the
//! converters also did are the Team compiler's, not this module's.

use pes_version::PesVersion;

use crate::codec::CodecError;
use crate::model::player::PlayerEntry;
use crate::model::playstyle::PlayStyle;
use crate::schema::fields::{PlayerField, PlayerText};
use crate::schema::ingame_face::IngameFaceField;
use crate::schema::limits::face_type_cap;
use crate::schema::{playstyle, schema_for};

/// What could not be carried one-to-one; the conversion still succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertNote {
    /// A face type above the target's cap was reset to 0 (the converters'
    /// default; not clamped).
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
    /// The source run is longer than the target's; its last `bytes` bytes
    /// were dropped (50 → 46).
    FaceRunTruncated {
        /// The number of source bytes that did not fit.
        bytes: usize,
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
        let value = cut(original, spec.len as usize - 1);
        if value.len() < original.len() {
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

    out.motion.hunching_dribbling = source.motion.hunching_dribbling;
    out.motion.hunching_running = source.motion.hunching_running;
    out.motion.arm_movement_dribbling = source.motion.arm_movement_dribbling;
    out.motion.arm_movement_running = source.motion.arm_movement_running;
    out.motion.corner_kick = source.motion.corner_kick;
    out.motion.free_kick = source.motion.free_kick;
    out.motion.penalty_kick = source.motion.penalty_kick;
    carry(&mut out.motion.dribbling, source.motion.dribbling);
    out.motion.goal_celebration_1 = source.motion.goal_celebration_1;
    out.motion.goal_celebration_2 = source.motion.goal_celebration_2;

    out.edit_flags.player = source.edit_flags.player;
    out.edit_flags.basic_settings = source.edit_flags.basic_settings;
    out.edit_flags.registered_position = source.edit_flags.registered_position;
    out.edit_flags.playable_positions = source.edit_flags.playable_positions;
    out.edit_flags.abilities = source.edit_flags.abilities;
    out.edit_flags.skills = source.edit_flags.skills;
    out.edit_flags.playing_style = source.edit_flags.playing_style;
    out.edit_flags.com_styles = source.edit_flags.com_styles;
    out.edit_flags.motion = source.edit_flags.motion;
    out.edit_flags.base_copy = source.edit_flags.base_copy;
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

    let dropped = out
        .appearance
        .ingame_face
        .copy_from(&source.appearance.ingame_face);
    if dropped > 0 {
        notes.push(ConvertNote::FaceRunTruncated { bytes: dropped });
    }
    for field in IngameFaceField::ALL {
        let Some(cap) = face_type_cap(to, field) else {
            continue;
        };
        let value = out.appearance.ingame_face.get(field)?;
        if value > cap {
            out.appearance.ingame_face.set(field, 0)?;
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
    use crate::schema::ingame_face::INGAME_FACE_FIELDS;
    use crate::test_support::{FIXTURES, count, find_player, payload, record};

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

    /// Whether run byte `i` holds none of the fifteen known fields' bits.
    fn outside_fields(i: usize) -> bool {
        let bit = i as u32 * 8;
        !INGAME_FACE_FIELDS
            .iter()
            .any(|spec| bit < spec.bit_offset + spec.bit_width && spec.bit_offset < bit + 8)
    }

    /// Every run byte outside the known fields is `expected`'s.
    fn assert_opaque_bytes(out: &PlayerEntry, expected: &PlayerEntry, upto: usize) {
        for (i, (&got, &want)) in out.appearance.ingame_face.bytes()[..upto]
            .iter()
            .zip(&expected.appearance.ingame_face.bytes()[..upto])
            .enumerate()
        {
            if outside_fields(i) {
                assert_eq!(got, want, "run byte {i}");
            }
        }
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
    /// hits the custom-skin rule.
    fn assert_face_rules(out: &PlayerEntry, source: &PlayerEntry, to: PesVersion) {
        for field in IngameFaceField::ALL {
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
        assert_opaque_bytes(&target, &source, 46);
        assert!(
            notes
                .iter()
                .all(|note| !matches!(note, N::FaceRunTruncated { .. })),
            "both runs are 50 bytes: {notes:?}"
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
        assert_opaque_bytes(&target, &source, 46);
        writes_ok(V::Pes21, &target);
    }

    #[test]
    fn the_run_truncates_on_15_and_keeps_the_target_tail_on_16() {
        let source16 = first(V::Pes16);
        let mut target15 = first(V::Pes15);
        let tail: Vec<u8> = target15.appearance.ingame_face.bytes()[46..].to_vec();
        assert!(tail.is_empty(), "PES 15's run is 46 bytes");
        let notes = convert_player(&source16, V::Pes16, &mut target15, V::Pes15).expect("converts");
        assert_eq!(
            notes
                .iter()
                .filter(|note| matches!(note, N::FaceRunTruncated { .. }))
                .collect::<Vec<_>>(),
            [&N::FaceRunTruncated { bytes: 4 }]
        );
        assert_eq!(target15.appearance.ingame_face.bytes().len(), 46);
        assert_opaque_bytes(&target15, &source16, 46);
        assert_face_rules(&target15, &source16, V::Pes15);

        let source15 = first(V::Pes15);
        let mut target16 = first(V::Pes16);
        let kept: Vec<u8> = target16.appearance.ingame_face.bytes()[46..].to_vec();
        let notes = convert_player(&source15, V::Pes15, &mut target16, V::Pes16).expect("converts");
        assert!(
            notes
                .iter()
                .all(|note| !matches!(note, N::FaceRunTruncated { .. }))
        );
        assert_eq!(
            target16.appearance.ingame_face.bytes()[46..],
            kept[..],
            "the longer target keeps its own tail"
        );
        assert_opaque_bytes(&target16, &source15, 46);
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

    #[test]
    fn skills_the_target_lacks_are_dropped_with_notes() {
        let mut source = first(V::Pes21);
        source.skills.skills = [true; 41];
        let schema15 = schema_for(V::Pes15).player;
        let missing: Vec<u8> = (0..41)
            .filter(|&i| !schema15.has(PlayerField::Skill(i)))
            .collect();
        assert!(!missing.is_empty(), "PES 15 lacks some skills");
        let mut target = first(V::Pes15);
        let notes = convert_player(&source, V::Pes21, &mut target, V::Pes15).expect("converts");
        let dropped: Vec<u8> = notes
            .iter()
            .filter_map(|note| {
                if let N::SkillDropped { index } = note {
                    Some(*index)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(dropped, missing);
        for &i in &missing {
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
    fn text_cuts_at_the_target_length_on_a_char_boundary() {
        let mut source = first(V::Pes19);
        source.name = "A".repeat(60);
        source.shirt_name = String::from("SHORT");
        let mut target = first(V::Pes16);
        let notes = convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.name.len(), 45);
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

        // Byte 45 lands inside the 3-byte '€': the cut backs off to 44.
        source.name = format!("{}€{}", "a".repeat(44), "b".repeat(10));
        let mut target = first(V::Pes16);
        convert_player(&source, V::Pes19, &mut target, V::Pes16).expect("converts");
        assert_eq!(target.name.len(), 44);
        assert!(target.name.is_char_boundary(target.name.len()));
        writes_ok(V::Pes16, &target);
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
