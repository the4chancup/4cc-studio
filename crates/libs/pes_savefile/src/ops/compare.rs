//! The comparator: every difference between two players of the same version
//! (the compare script's fields plus the reference editor's full walk, so
//! each reference is a subset of these rows), and the save-level diff pairing
//! players by id. `ops/` reads `model/` only; the field list is the schema's.

use std::collections::{HashMap, HashSet};

use pes_version::PesVersion;

use crate::codec::{self, CodecError};
use crate::file::EditFile;
use crate::model::player::PlayerEntry;
use crate::ops::fingerprint::{FaceHash, face_hash};
use crate::schema::fields::{PlayerField, PlayerText};
use crate::schema::ingame_face::{INGAME_FACE_FIELDS, IngameFaceField};
use crate::schema::{RecordSchema, VersionSchema};

/// Every difference between two players of the same version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerDiff {
    /// A bit-run field of the record.
    Field {
        /// The schema field that differs.
        field: PlayerField,
        /// `a`'s stored value.
        old: u32,
        /// `b`'s stored value.
        new: u32,
    },
    /// The name or shirt name.
    Text {
        /// The text field that differs.
        text: PlayerText,
        /// `a`'s text.
        old: String,
        /// `b`'s text.
        new: String,
    },
    /// A known bit run of the ingame-face block.
    Face {
        /// The ingame-face field that differs.
        field: IngameFaceField,
        /// `a`'s value.
        old: u8,
        /// `b`'s value.
        new: u8,
    },
    /// Bits of the run no `IngameFaceField` names differ; the hashes are the
    /// reference's fingerprints of the two runs.
    FaceRun {
        /// `a`'s fingerprint.
        old: FaceHash,
        /// `b`'s fingerprint.
        new: FaceHash,
    },
}

/// Whether a diff row is an aesthetics change or a gameplay one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffScope {
    /// A gameplay field.
    Gameplay,
    /// An aesthetics field: what `transplant_player` moves.
    Aesthetics,
}

impl PlayerDiff {
    /// Aesthetics is what `transplant_player` moves: the appearance-block
    /// fields (the four appearance edit flags, boots/gloves/base-copy IDs,
    /// physique, strip), every `Face` and `FaceRun` row. Everything else
    /// (motion and the other edit flags included) is gameplay.
    pub fn scope(&self) -> DiffScope {
        use PlayerField as F;
        let field = match self {
            PlayerDiff::Field { field, .. } => *field,
            PlayerDiff::Text { .. } => return DiffScope::Gameplay,
            PlayerDiff::Face { .. } | PlayerDiff::FaceRun { .. } => {
                return DiffScope::Aesthetics;
            }
        };
        match field {
            F::EditedFace
            | F::EditedHair
            | F::EditedPhysique
            | F::EditedStrip
            | F::BootsId
            | F::GlovesId
            | F::BaseCopyId
            | F::NeckLength
            | F::NeckSize
            | F::ShoulderHeight
            | F::ShoulderWidth
            | F::Chest
            | F::Waist
            | F::ArmSize
            | F::ArmLength
            | F::Thigh
            | F::Calf
            | F::LegLength
            | F::HeadLength
            | F::HeadWidth
            | F::HeadDepth
            | F::WristTapeColorLeft
            | F::WristTapeColorRight
            | F::WristTaping
            | F::SpectaclesColor
            | F::SpectaclesStyle
            | F::Sleeves
            | F::Inners
            | F::Socks
            | F::Undershorts
            | F::Untucked
            | F::AnkleTaping => DiffScope::Aesthetics,
            F::Id
            | F::Nationality
            | F::Height
            | F::Weight
            | F::GoalCelebration1
            | F::GoalCelebration2
            | F::AttackingProwess
            | F::DefensiveProwess
            | F::Goalkeeping
            | F::Dribbling
            | F::FreeKickMotion
            | F::Finishing
            | F::LowPass
            | F::LoftedPass
            | F::Heading
            | F::Form
            | F::EditedPlayer
            | F::Swerve
            | F::Catching
            | F::Clearing
            | F::Reflexes
            | F::InjuryResistance
            | F::EditedBasicSettings
            | F::BodyControl
            | F::PhysicalContact
            | F::KickingPower
            | F::ExplosivePower
            | F::ArmMovementDribbling
            | F::EditedRegisteredPosition
            | F::Age
            | F::RegisteredPosition
            | F::PlayingStyle
            | F::BallControl
            | F::BallWinning
            | F::WeakFootAccuracy
            | F::Jump
            | F::ArmMovementRunning
            | F::CornerKickMotion
            | F::Coverage
            | F::WeakFootUsage
            | F::PlayablePosition(_)
            | F::HunchingDribbling
            | F::HunchingRunning
            | F::PenaltyKickMotion
            | F::PlaceKicking
            | F::EditedPlayablePositions
            | F::EditedAbilities
            | F::EditedSkills
            | F::Stamina
            | F::Speed
            | F::EditedPlayingStyle
            | F::EditedComStyles
            | F::EditedMotion
            | F::BaseCopy
            | F::StrongerFoot
            | F::ComStyle(_)
            | F::Skill(_)
            | F::Star
            | F::DribblingMotion
            | F::TightPossession
            | F::Aggression
            | F::PlayingAttitude
            | F::StrongerHand => DiffScope::Gameplay,
        }
    }
}

/// `Field` rows of one record schema, in schema order. `seen`, when given,
/// is a record whose fields are skipped (the 15/16 appearance record
/// repeats `Id`).
fn record_diffs(
    a: &PlayerEntry,
    b: &PlayerEntry,
    schema: &RecordSchema<PlayerField, PlayerText>,
    seen: Option<&RecordSchema<PlayerField, PlayerText>>,
    diffs: &mut Vec<PlayerDiff>,
) -> Result<(), CodecError> {
    for (field, _, _) in codec::runs(schema) {
        if seen.is_some_and(|seen| seen.has(field)) {
            continue;
        }
        let (old, new) = (a.get(field)?, b.get(field)?);
        if old != new {
            diffs.push(PlayerDiff::Field { field, old, new });
        }
    }
    Ok(())
}

/// Every difference between two players of the same version: the schema's
/// stored fields (the player record's, plus the 15/16 appearance record's)
/// through the model, the two texts, the known ingame-face bits, and the
/// run's undecoded bits as one row. `Err` only when a gated field the schema
/// stores is `None` on either side (an entry not read from a save of this
/// version).
pub fn compare_players(
    a: &PlayerEntry,
    b: &PlayerEntry,
    schema: &VersionSchema,
) -> Result<Vec<PlayerDiff>, CodecError> {
    let mut diffs = Vec::new();
    let (fa, fb) = (&a.appearance.ingame_face, &b.appearance.ingame_face);
    if fa.bytes().len() != fb.bytes().len() || fa.bytes().is_empty() {
        // A run that does not reach a field is `NoIngameFaceRun` on that side;
        // an empty run is an entry never read from a record.
        return Err(CodecError::NoIngameFaceRun {
            field: String::from("ingame_face"),
        });
    }
    record_diffs(a, b, schema.player, None, &mut diffs)?;
    if let Some((_, appearance)) = &schema.appearance {
        record_diffs(a, b, appearance, Some(schema.player), &mut diffs)?;
    }
    for (text, old, new) in [
        (PlayerText::Name, &a.name, &b.name),
        (PlayerText::ShirtName, &a.shirt_name, &b.shirt_name),
    ] {
        if old != new {
            diffs.push(PlayerDiff::Text {
                text,
                old: old.clone(),
                new: new.clone(),
            });
        }
    }
    for spec in INGAME_FACE_FIELDS {
        let (old, new) = (fa.get(spec.field)?, fb.get(spec.field)?);
        if old != new {
            diffs.push(PlayerDiff::Face {
                field: spec.field,
                old,
                new,
            });
        }
    }
    if fa.undecoded() != fb.undecoded() {
        diffs.push(PlayerDiff::FaceRun {
            old: face_hash(fa),
            new: face_hash(fb),
        });
    }
    Ok(diffs)
}

/// The player-level diff of two saves, players paired by id (the compare
/// script's pairing; the reference editor pairs by roster slot, which reports
/// a reshuffle as edits).
#[derive(Debug, Default)]
pub struct SaveDiff {
    /// Players in both saves with at least one difference, in `a`'s record
    /// order.
    pub players: Vec<(u32, Vec<PlayerDiff>)>,
    /// Ids `a` has and `b` lacks, in `a`'s record order.
    pub only_in_a: Vec<u32>,
    /// Ids `b` has and `a` lacks, in `b`'s record order.
    pub only_in_b: Vec<u32>,
}

/// A save-level compare that could not run.
#[derive(Debug, thiserror::Error)]
pub enum CompareError {
    /// The two saves are not of the same game version.
    #[error("the saves are of different versions: {a:?} vs {b:?}")]
    VersionMismatch {
        /// `a`'s version.
        a: PesVersion,
        /// `b`'s version.
        b: PesVersion,
    },
    /// A field the schema stores had no value on one side.
    #[error(transparent)]
    Codec(#[from] CodecError),
}

/// Two saves, players paired by id.
pub fn compare(a: &EditFile, b: &EditFile) -> Result<SaveDiff, CompareError> {
    if a.version() != b.version() {
        return Err(CompareError::VersionMismatch {
            a: a.version(),
            b: b.version(),
        });
    }
    let schema = crate::schema::schema_for(a.version());
    // `or_insert` keeps the first record of a duplicate id, as `player` does.
    let b_by_id: HashMap<u32, &PlayerEntry> =
        b.players().iter().fold(HashMap::new(), |mut map, p| {
            map.entry(p.id).or_insert(p);
            map
        });
    let mut diff = SaveDiff::default();
    let mut a_ids = HashSet::new();
    for pa in a.players() {
        a_ids.insert(pa.id);
        match b_by_id.get(&pa.id) {
            Some(pb) => {
                let rows = compare_players(pa, pb, schema)?;
                if !rows.is_empty() {
                    diff.players.push((pa.id, rows));
                }
            }
            None => diff.only_in_a.push(pa.id),
        }
    }
    diff.only_in_b = b
        .players()
        .iter()
        .filter(|p| !a_ids.contains(&p.id))
        .map(|p| p.id)
        .collect();
    Ok(diff)
}

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;

    use super::*;
    use crate::model::ingame_face::IngameFace;
    use crate::schema::schema_for;
    use crate::test_support::{FIXTURES, find_player, open, payload};

    fn player(version: PesVersion, id: u32) -> PlayerEntry {
        find_player(&payload(version), schema_for(version), id)
    }

    /// `(byte, bit)` of the run covered by no known field.
    fn an_undecoded_bit(len: usize) -> (usize, u32) {
        for byte in 0..len {
            for bit in 0..8u32 {
                let at = byte as u32 * 8 + bit;
                let covered = INGAME_FACE_FIELDS
                    .iter()
                    .any(|s| at >= s.bit_offset && at < s.bit_offset + s.bit_width);
                if !covered {
                    return (byte, bit);
                }
            }
        }
        panic!("the {len}-byte run has no undecoded bit");
    }

    #[test]
    fn a_player_equals_its_own_copy_on_every_version() {
        for version in FIXTURES {
            let p = player(version, 70101);
            assert!(
                compare_players(&p, &p, schema_for(version))
                    .expect("compares")
                    .is_empty(),
                "{version:?}"
            );
        }
    }

    #[test]
    fn each_single_edit_is_one_row_of_the_right_scope() {
        let a = player(PesVersion::Pes21, 70101);
        let schema = schema_for(PesVersion::Pes21);

        let mut b = a.clone();
        b.stats.speed += 1;
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Field {
                field: PlayerField::Speed,
                old: u32::from(a.stats.speed),
                new: u32::from(b.stats.speed),
            }]
        );
        assert_eq!(diffs[0].scope(), DiffScope::Gameplay);

        let mut b = a.clone();
        b.name.push('X');
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Text {
                text: PlayerText::Name,
                old: a.name.clone(),
                new: b.name.clone(),
            }]
        );
        assert_eq!(diffs[0].scope(), DiffScope::Gameplay);

        let mut b = a.clone();
        b.shirt_name.push('X');
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Text {
                text: PlayerText::ShirtName,
                old: a.shirt_name.clone(),
                new: b.shirt_name.clone(),
            }]
        );

        let mut b = a.clone();
        b.appearance.sleeves = (b.appearance.sleeves + 1) % 3;
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Field {
                field: PlayerField::Sleeves,
                old: u32::from(a.appearance.sleeves),
                new: u32::from(b.appearance.sleeves),
            }]
        );
        assert_eq!(diffs[0].scope(), DiffScope::Aesthetics);
    }

    #[test]
    fn a_face_field_is_one_face_row_an_unknown_bit_is_one_run_row() {
        let a = player(PesVersion::Pes21, 70101);
        let schema = schema_for(PesVersion::Pes21);
        let face = &a.appearance.ingame_face;

        // A known field: one Face row, no run row.
        let mut b = a.clone();
        let nose = face.get(IngameFaceField::NoseType).expect("covered");
        b.appearance
            .ingame_face
            .set(IngameFaceField::NoseType, nose ^ 1)
            .expect("fits");
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Face {
                field: IngameFaceField::NoseType,
                old: nose,
                new: nose ^ 1,
            }]
        );

        // A bit no field covers: one FaceRun row with the two fingerprints.
        let (byte, bit) = an_undecoded_bit(face.bytes().len());
        let mut run = face.bytes().to_vec();
        run[byte] ^= 1 << bit;
        let mut b = a.clone();
        b.appearance.ingame_face = IngameFace::from_bytes(run);
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::FaceRun {
                old: face_hash(face),
                new: face_hash(&b.appearance.ingame_face),
            }]
        );
        assert_ne!(face_hash(face), face_hash(&b.appearance.ingame_face));

        // The skin bit is a known field: a Face row, and masked out of the
        // run row either way.
        let mut b = a.clone();
        let skin = face.get(IngameFaceField::SkinColor).expect("covered");
        b.appearance
            .ingame_face
            .set(IngameFaceField::SkinColor, skin ^ 1)
            .expect("fits");
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Face {
                field: IngameFaceField::SkinColor,
                old: skin,
                new: skin ^ 1,
            }]
        );
    }

    #[test]
    fn a_field_is_aesthetics_exactly_when_transplant_moves_it() {
        let schema = schema_for(PesVersion::Pes21);
        let a = player(PesVersion::Pes21, 70101);
        for (field, _, width) in crate::codec::runs(schema.player) {
            let old = a.get(field).expect("covered");
            let new = if old == 0 { 1 } else { 0 };
            assert!(width == 32 || new < (1u32 << width), "{field:?}");
            let mut b = a.clone();
            b.set(field, new).expect("fits");
            let diffs = compare_players(&a, &b, schema).expect("compares");
            assert_eq!(diffs, [PlayerDiff::Field { field, old, new }], "{field:?}");
            let mut c = a.clone();
            crate::ops::transplant::transplant_player(&mut c, &b);
            let moved = c.get(field).expect("covered") == new;
            assert_eq!(
                diffs[0].scope() == DiffScope::Aesthetics,
                moved,
                "{field:?}"
            );
        }

        // The texts are gameplay.
        let mut b = a.clone();
        b.name.push('X');
        b.shirt_name.push('X');
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(diffs.len(), 2, "{diffs:?}");
        for diff in &diffs {
            assert_eq!(diff.scope(), DiffScope::Gameplay, "{diff:?}");
        }
    }

    #[test]
    fn a_face_field_and_an_undecoded_bit_are_two_rows() {
        let a = player(PesVersion::Pes21, 70101);
        let schema = schema_for(PesVersion::Pes21);
        let face = &a.appearance.ingame_face;
        let nose = face.get(IngameFaceField::NoseType).expect("covered");
        let (byte, bit) = an_undecoded_bit(face.bytes().len());

        let mut b = a.clone();
        let mut run = face.bytes().to_vec();
        run[byte] ^= 1 << bit;
        b.appearance.ingame_face = IngameFace::from_bytes(run);
        b.appearance
            .ingame_face
            .set(IngameFaceField::NoseType, nose ^ 1)
            .expect("fits");

        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [
                PlayerDiff::Face {
                    field: IngameFaceField::NoseType,
                    old: nose,
                    new: nose ^ 1,
                },
                PlayerDiff::FaceRun {
                    old: face_hash(face),
                    new: face_hash(&b.appearance.ingame_face),
                },
            ]
        );
    }

    #[test]
    fn the_appearance_records_id_is_not_a_second_id_row() {
        let schema = schema_for(PesVersion::Pes16);
        let a = player(PesVersion::Pes16, 70101);
        let b = player(PesVersion::Pes16, 70202);
        assert_ne!(a.id, b.id);
        let ids: Vec<PlayerDiff> = compare_players(&a, &b, schema)
            .expect("compares")
            .into_iter()
            .filter(|d| {
                matches!(
                    d,
                    PlayerDiff::Field {
                        field: PlayerField::Id,
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(ids.len(), 1, "{ids:?}");
    }

    #[test]
    fn the_appearance_record_is_walked_on_pes16() {
        let a = player(PesVersion::Pes16, 70101);
        let schema = schema_for(PesVersion::Pes16);
        assert!(schema.appearance.is_some());
        let mut b = a.clone();
        b.appearance.boots_id = b.appearance.boots_id.wrapping_add(1) % 0x4000;
        let diffs = compare_players(&a, &b, schema).expect("compares");
        assert_eq!(
            diffs,
            [PlayerDiff::Field {
                field: PlayerField::BootsId,
                old: a.appearance.boots_id,
                new: b.appearance.boots_id,
            }]
        );
    }

    #[test]
    fn compare_pairs_by_id_and_reports_unmatched() {
        let (a, _) = open(PesVersion::Pes16);
        let (b, _) = open(PesVersion::Pes16);
        let diff = compare(&a, &b).expect("same save");
        assert!(diff.players.is_empty());
        assert!(diff.only_in_a.is_empty());
        assert!(diff.only_in_b.is_empty());

        let mut b = b;
        b.player_mut(70103).expect("in b").stats.speed += 1;
        let renamed = 70104;
        assert!(b.player(renamed).is_some());
        let new_id = 999999;
        assert!(a.player(new_id).is_none());
        b.player_mut(renamed).expect("in b").id = new_id;

        let diff = compare(&a, &b).expect("compares");
        assert_eq!(diff.players.len(), 1);
        assert_eq!(diff.players[0].0, 70103);
        assert_eq!(
            diff.players[0].1,
            [PlayerDiff::Field {
                field: PlayerField::Speed,
                old: a.player(70103).expect("in a").stats.speed.into(),
                new: b.player(70103).expect("in b").stats.speed.into(),
            }]
        );
        assert_eq!(diff.only_in_a, [renamed]);
        assert_eq!(diff.only_in_b, [new_id]);
    }

    #[test]
    fn compare_refuses_a_version_mismatch() {
        let (a, _) = open(PesVersion::Pes16);
        let (b, _) = open(PesVersion::Pes17);
        assert!(matches!(
            compare(&a, &b),
            Err(CompareError::VersionMismatch {
                a: PesVersion::Pes16,
                b: PesVersion::Pes17,
            })
        ));
    }

    #[test]
    fn an_entry_never_read_has_no_face_run_to_compare() {
        let schema = schema_for(PesVersion::Pes21);
        let fixture = player(PesVersion::Pes21, 70101);
        assert!(matches!(
            compare_players(&PlayerEntry::default(), &fixture, schema),
            Err(CodecError::NoIngameFaceRun { .. })
        ));
        assert!(matches!(
            compare_players(&PlayerEntry::default(), &PlayerEntry::default(), schema),
            Err(CodecError::NoIngameFaceRun { .. })
        ));
    }
}
