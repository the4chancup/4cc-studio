//! Aesthetics transplant: copy the appearance data of selected players from a
//! donor save into a target save of the same version (the scripts' four
//! selection forms kept as-is: single player id, `target:donor` id pairs,
//! whole team, team ranges).

use pes_version::PesVersion;

use crate::file::EditFile;
use crate::model::player::PlayerEntry;

/// The per-entry operation: `target` takes `donor`'s appearance block
/// (`appearance`, ingame-face run included, and the face/hair/physique/strip
/// edit flags); everything else, the id included, is the target's own.
pub fn transplant_player(target: &mut PlayerEntry, donor: &PlayerEntry) {
    target.appearance = donor.appearance.clone();
    target.edit_flags.face = donor.edit_flags.face;
    target.edit_flags.hair = donor.edit_flags.hair;
    target.edit_flags.physique = donor.edit_flags.physique;
    target.edit_flags.strip = donor.edit_flags.strip;
}

/// A transplant that could not run.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TransplantError {
    /// The two saves are not of the same game version.
    #[error("the saves are of different versions: {target:?} vs {donor:?}")]
    VersionMismatch {
        /// The target save's version.
        target: PesVersion,
        /// The donor save's version.
        donor: PesVersion,
    },
    /// A pair names a player the target save lacks.
    #[error("the target save has no player {0}")]
    TargetMissing(u32),
    /// A pair names a player the donor save lacks.
    #[error("the donor save has no player {0}")]
    DonorMissing(u32),
}

/// The save-level operation over `(target_id, donor_id)` pairs;
/// all-or-nothing: the version pair and every id are checked before the first
/// write.
pub fn transplant(
    target: &mut EditFile,
    donor: &EditFile,
    pairs: &[(u32, u32)],
) -> Result<(), TransplantError> {
    if target.version() != donor.version() {
        return Err(TransplantError::VersionMismatch {
            target: target.version(),
            donor: donor.version(),
        });
    }
    for &(target_id, donor_id) in pairs {
        if target.player(target_id).is_none() {
            return Err(TransplantError::TargetMissing(target_id));
        }
        if donor.player(donor_id).is_none() {
            return Err(TransplantError::DonorMissing(donor_id));
        }
    }
    for &(target_id, donor_id) in pairs {
        let source = donor.player(donor_id).expect("checked above");
        transplant_player(target.player_mut(target_id).expect("checked above"), source);
    }
    Ok(())
}

/// A selection argument that is not one of the scripts' four forms.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SelectionError {
    /// Not a number, or a pair/range with more than one separator.
    #[error("{0:?}: expected a player id, target:donor, a team id or a team range")]
    Malformed(String),
    /// A range whose ends are not teams (≥ 1000) or run backwards.
    #[error("{0:?}: a range's ends must be team ids below 1000, start to stop")]
    BadRange(String),
}

/// One selection argument in the scripts' four forms, expanded to `(target,
/// donor)` pairs: `70103` (one player, onto itself), `70103:70205`
/// (target:donor), `701` (a team: its slots 1-23, `team * 100 + 1..=23`, onto
/// themselves), `701-720` (a team range, likewise). A number below 1000 is a
/// team, as in the scripts.
pub fn parse_selection(arg: &str) -> Result<Vec<(u32, u32)>, SelectionError> {
    let number = |part: &str| {
        part.parse::<u32>()
            .map_err(|_| SelectionError::Malformed(arg.to_string()))
    };
    if arg.contains(':') {
        let parts: Vec<&str> = arg.split(':').collect();
        if parts.len() != 2 {
            return Err(SelectionError::Malformed(arg.to_string()));
        }
        return Ok(vec![(number(parts[0])?, number(parts[1])?)]);
    }
    if arg.contains('-') {
        let parts: Vec<&str> = arg.split('-').collect();
        if parts.len() != 2 {
            return Err(SelectionError::Malformed(arg.to_string()));
        }
        let (start, stop) = (number(parts[0])?, number(parts[1])?);
        if start >= 1000 || stop >= 1000 || start > stop {
            return Err(SelectionError::BadRange(arg.to_string()));
        }
        return Ok((start..=stop).flat_map(team_pairs).collect());
    }
    let n = number(arg)?;
    if n < 1000 {
        Ok(team_pairs(n).collect())
    } else {
        Ok(vec![(n, n)])
    }
}

/// Team `t`'s 23 player slots, each paired onto itself.
fn team_pairs(t: u32) -> impl Iterator<Item = (u32, u32)> {
    (1..=23).map(move |slot| (t * 100 + slot, t * 100 + slot))
}

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;

    use super::*;
    use crate::test_support::{open, test_salt};

    use SelectionError as E;

    #[test]
    fn parse_selection_expands_the_scripts_four_forms() {
        assert_eq!(parse_selection("70103"), Ok(vec![(70103, 70103)]));
        assert_eq!(parse_selection("70103:70205"), Ok(vec![(70103, 70205)]));

        let team = parse_selection("701").expect("a team");
        assert_eq!(team.len(), 23);
        assert_eq!(team[0], (70101, 70101));
        assert_eq!(team[22], (70123, 70123));

        let range = parse_selection("701-702").expect("a range");
        assert_eq!(range.len(), 46);
        assert_eq!(range[0], (70101, 70101));
        assert_eq!(range[22], (70123, 70123));
        assert_eq!(range[23], (70201, 70201));
        assert_eq!(range[45], (70223, 70223));

        assert_eq!(
            parse_selection("1000-1001"),
            Err(E::BadRange("1000-1001".to_string()))
        );
        assert_eq!(
            parse_selection("999-1000"),
            Err(E::BadRange("999-1000".to_string()))
        );
        assert_eq!(
            parse_selection("702-701"),
            Err(E::BadRange("702-701".to_string()))
        );
        assert_eq!(
            parse_selection("701-701").expect("a one-team range").len(),
            23
        );
        assert_eq!(parse_selection("1000"), Ok(vec![(1000, 1000)]));
        let team = parse_selection("999").expect("a team");
        assert_eq!(team.len(), 23);
        assert_eq!(team[0], (99901, 99901));
        assert_eq!(team[22], (99923, 99923));
        for bad in ["a", "1:2:3", "1-2-3", ""] {
            assert_eq!(
                parse_selection(bad),
                Err(E::Malformed(bad.to_string())),
                "{bad:?}"
            );
        }
    }

    /// `version`'s fixture as two `EditFile`s, `id` confirmed present in both.
    fn saves(version: PesVersion, id: u32) -> (EditFile, EditFile) {
        let (target, _) = open(version);
        let (donor, _) = open(version);
        assert!(target.player(id).is_some(), "{version:?} has {id}");
        assert!(donor.player(id).is_some());
        (target, donor)
    }

    #[test]
    fn transplant_moves_only_the_named_players() {
        for version in [PesVersion::Pes16, PesVersion::Pes17] {
            let (mut target, donor) = saves(version, 70205);
            assert!(target.player(70103).is_some(), "{version:?} has 70103");
            assert_ne!(
                target.player(70103).expect("in target").appearance,
                donor.player(70205).expect("in donor").appearance,
                "{version:?}: the pair discriminates the copy"
            );
            let before: Vec<PlayerEntry> = target.players().to_vec();
            let mut pairs = parse_selection("70103:70205").expect("pair");
            // Team 701 onto donor team 702: cross-id pairs, not self-pairs.
            pairs.extend(
                parse_selection("701")
                    .expect("team")
                    .into_iter()
                    .map(|(t, _)| (t, t + 100)),
            );
            transplant(&mut target, &donor, &pairs).expect("transplants");

            // Pairs apply in order; a later pair onto the same id wins
            // (the team range rewrites 70103 after the explicit pair).
            let mut expected = before.clone();
            for &(target_id, donor_id) in &pairs {
                let source = donor.player(donor_id).expect("in donor");
                let slot = expected
                    .iter_mut()
                    .find(|p| p.id == target_id)
                    .expect("in target");
                transplant_player(slot, source);
            }
            assert_eq!(target.players(), &expected[..], "{version:?}");
            let touched: Vec<u32> = pairs.iter().map(|(t, _)| *t).collect();
            assert_eq!(touched.len(), 24);
            target.to_bytes(&test_salt()).expect("the save writes");
        }
    }

    #[test]
    fn transplant_is_all_or_nothing() {
        let missing = 999999;
        let (mut target, donor) = saves(PesVersion::Pes16, 70205);
        assert!(target.player(70103).is_some());
        assert_ne!(
            target.player(70103).expect("in target").appearance,
            donor.player(70205).expect("in donor").appearance,
            "the first pair discriminates the copy"
        );
        assert!(target.player(missing).is_none());
        let before: Vec<PlayerEntry> = target.players().to_vec();

        // A valid first pair that would write, then an absent donor id:
        // nothing is written.
        let pairs = [(70103, 70205), (70104, missing)];
        assert_eq!(
            transplant(&mut target, &donor, &pairs),
            Err(TransplantError::DonorMissing(missing))
        );
        assert_eq!(target.players(), &before[..]);
        assert_eq!(
            target.player(70103).expect("in target"),
            before.iter().find(|p| p.id == 70103).expect("in before")
        );

        let pairs = [(missing, 70103)];
        assert_eq!(
            transplant(&mut target, &donor, &pairs),
            Err(TransplantError::TargetMissing(missing))
        );
        assert_eq!(target.players(), &before[..]);
    }

    #[test]
    fn transplant_refuses_a_version_mismatch() {
        let (mut target, _) = open(PesVersion::Pes16);
        let (donor, _) = open(PesVersion::Pes17);
        assert_eq!(
            transplant(&mut target, &donor, &[(70103, 70103)]),
            Err(TransplantError::VersionMismatch {
                target: PesVersion::Pes16,
                donor: PesVersion::Pes17,
            })
        );
    }

    #[test]
    fn transplant_player_moves_the_appearance_and_four_flags() {
        let (target, donor) = saves(PesVersion::Pes17, 70205);
        let target_p = target.player(70103).expect("in target");
        let donor_p = donor.player(70205).expect("in donor");
        assert_ne!(
            target_p.appearance, donor_p.appearance,
            "the pair discriminates the appearance copy"
        );

        // Flip the four flags on a copy of the donor, so the flag copy is
        // proven independent of the fixture's values.
        let mut flipped = donor_p.clone();
        flipped.edit_flags.face = !flipped.edit_flags.face;
        flipped.edit_flags.hair = !flipped.edit_flags.hair;
        flipped.edit_flags.physique = !flipped.edit_flags.physique;
        flipped.edit_flags.strip = !flipped.edit_flags.strip;

        let mut moved = target_p.clone();
        transplant_player(&mut moved, &flipped);
        let mut expected = target_p.clone();
        expected.appearance = flipped.appearance.clone();
        expected.edit_flags.face = flipped.edit_flags.face;
        expected.edit_flags.hair = flipped.edit_flags.hair;
        expected.edit_flags.physique = flipped.edit_flags.physique;
        expected.edit_flags.strip = flipped.edit_flags.strip;
        assert_eq!(moved, expected);
    }
}
