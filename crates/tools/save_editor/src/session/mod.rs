//! The session: the save that is open, and the operations over it that the commands run (and
//! the window will, through its undo stack). egui-free, so the command line and the window
//! cannot diverge (`spec/save_editor.md` "Crate layout"). `pes_savefile` owns every byte and
//! every operation on the model; the session resolves ids against the open save and hands
//! the lib the entries it asks for.

use std::path::Path;

use anyhow::{Context, anyhow};
use pes_savefile::interchange::aesthetics_patch::{AestheticsPatch, PatchError, PatchSummary};
use pes_savefile::interchange::team_toml::{ImportNote, TeamToml, TeamTomlError};
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_savefile::model::{PlayerEntry, RosterSlot};
use pes_savefile::ops::transplant::{TransplantError, transplant};
use pes_savefile::{EditFile, SaveError};
use pes_version::PesVersion;

/// The open save.
pub(crate) struct Session {
    file: EditFile,
}

/// A player of a transplant pair whose appearance id is -1: his record is not what the game
/// shows (`spec/save_editor.md` "Stripped save"), so the output marks him.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stripped {
    /// The target: his record took the donor's fields, and the game does not read it.
    Target(u32),
    /// The donor: the fields copied are his record's, not the look the game shows for him.
    Donor(u32),
}

/// One player's contribution to `strip-and-seed`'s seed (`spec/save_editor.md` "Stripped
/// save"): his `PlayerAppearance.bin` row and his boots and gloves list values, as the
/// record holds them (a value past a key's range is carried as stored).
#[derive(Debug)]
pub(crate) struct SeedRow {
    /// The player's id.
    pub player: u32,
    /// His `PlayerAppearance.bin` row: the id little-endian, then the 56 bytes his record
    /// keeps after the base-copy id (`EditFile::appearance_row`).
    pub row: [u8; 60],
    /// His boots ID, the `BootsList.bin` pair's value little-endian.
    pub boots: u32,
    /// His gloves ID, the `GloveList.bin` pair's value little-endian.
    pub gloves: u32,
    /// The player his appearance block copies the face and hair from (`None` for a
    /// base-copy id of 0 or his own id, which both mean no base copy). The row carries
    /// no base-copy id, so once stripped he loses the copied face: `strip-and-seed`
    /// warns for each (`spec/save_editor.md` "Stripped save"). The record's face edit
    /// flag plays no part: in PES 21 Edit mode (2026-10-15) the game drew the copied
    /// face with the flag clear as with it set, and over a face model of the player's
    /// own.
    pub base_copy: Option<u32>,
}

impl Session {
    /// Opens the save at `path`. The error names the path and what `pes_savefile` refused.
    pub(crate) fn open(path: &Path) -> anyhow::Result<Session> {
        let file = EditFile::load(path)
            .with_context(|| format!("{}: cannot open the save", path.display()))?;
        Ok(Session { file })
    }

    /// The open save's game version.
    pub(crate) fn version(&self) -> PesVersion {
        self.file.version()
    }

    /// The open save, for the read-only operations over two saves (the comparator).
    pub(crate) fn file(&self) -> &EditFile {
        &self.file
    }

    /// The ids of the save's teams, in record order.
    pub(crate) fn team_ids(&self) -> impl Iterator<Item = u32> {
        self.file.teams().iter().map(|team| team.id)
    }

    /// The Team TOML text of the team with `id`: its entry, tactics and every rostered
    /// player with every field. An id the save lacks is an error naming it.
    pub(crate) fn export_team(&self, id: u32) -> anyhow::Result<String> {
        let team = self
            .file
            .team(id)
            .ok_or_else(|| anyhow!("the save has no team {id}"))?;
        // The whole pool, borrowed: `from_team` picks the roster's players out of it.
        let players: Vec<&PlayerEntry> = self.file.players().iter().collect();
        let document = TeamToml::from_team(self.version(), team, &players)
            .with_context(|| format!("team {id} cannot be exported"))?;
        document
            .to_toml()
            .with_context(|| format!("team {id} cannot be written as Team TOML"))
    }

    /// Imports `document` onto the team its `[team] id` names, with the lib's import rules
    /// (absent = untouched, cross-version conversion when the document's version differs),
    /// all or nothing. The notes are what was converted or dropped. A document with no id,
    /// one naming a team the save lacks, or one the lib refuses (a text that does not fit the
    /// version's field, named with the player it is for) is an error and the save is
    /// untouched.
    pub(crate) fn import_team(&mut self, document: &TeamToml) -> anyhow::Result<Vec<ImportNote>> {
        let id = document
            .team
            .id
            .ok_or_else(|| anyhow!("the Team TOML names no team: [team] id is required"))?;
        // Applied to a copy of the team, put back once every key applied: the lib takes the
        // team and the player pool as two borrows, which one `EditFile` cannot hand out at once.
        let mut team = self
            .file
            .team(id)
            .ok_or_else(|| anyhow!("the save has no team {id}"))?
            .clone();
        let version = self.version();
        let notes = document
            .apply(version, &mut team, self.file.players_mut())
            .map_err(|error| import_error(error, &team.roster))?;
        *self.file.team_mut(id).expect("found above") = team;
        Ok(notes)
    }

    /// Applies the compiler's patch: each listed player's motions and names, nothing of his
    /// appearance block. The summary lists the teams applied and the items the save lacked;
    /// a version or scheme mismatch, or a pre-Fox patch, is an error with the save untouched.
    pub(crate) fn apply_patch(
        &mut self,
        patch: &AestheticsPatch,
    ) -> Result<PatchSummary, PatchError> {
        patch.apply(&mut self.file)
    }

    /// Copies the donor's appearance block (its id aside) and the face, hair, physique and
    /// strip edit flags onto each pair's target, all or nothing: the version pair and every
    /// id are checked before the first write. The marks are the pairs' players at -1, in
    /// pair order, target before donor.
    pub(crate) fn transplant(
        &mut self,
        donor: &Session,
        pairs: &[(u32, u32)],
    ) -> Result<Vec<Stripped>, TransplantError> {
        transplant(&mut self.file, &donor.file, pairs)?;
        let mut marks = Vec::new();
        for &(target, source) in pairs {
            if is_stripped(&self.file, target) {
                marks.push(Stripped::Target(target));
            }
            if is_stripped(&donor.file, source) {
                marks.push(Stripped::Donor(source));
            }
        }
        Ok(marks)
    }

    /// The seed `strip-and-seed` writes into the Fox player tables: one `SeedRow` per
    /// player of the save whose appearance id is set, in record order. A player at -1 is
    /// left out: his record is not his look, so his rows stand (`spec/save_editor.md`
    /// "Stripped save"). The error is `appearance_row`'s, which a pre-Fox save refuses.
    pub(crate) fn seed(&self) -> Result<Vec<SeedRow>, SaveError> {
        let mut rows = Vec::new();
        for player in self.file.players() {
            if is_stripped(&self.file, player.id) {
                continue;
            }
            let copied = player.appearance.base_copy_id;
            rows.push(SeedRow {
                player: player.id,
                row: self.file.appearance_row(player.id)?,
                boots: player.appearance.boots_id,
                gloves: player.appearance.gloves_id,
                base_copy: (copied != 0 && copied != player.id).then_some(copied),
            });
        }
        Ok(rows)
    }

    /// The players `check-stripped` lists: an appearance id set while at least one other
    /// player of the save is at -1, in record order. A save with no stripped player lists
    /// nobody (nothing marked him a table-dressed save).
    pub(crate) fn restored_players(&self) -> Vec<&PlayerEntry> {
        if !self
            .file
            .players()
            .iter()
            .any(|player| player.appearance_id == Some(STRIPPED_APPEARANCE_ID))
        {
            return Vec::new();
        }
        self.file
            .players()
            .iter()
            .filter(|player| !is_stripped(&self.file, player.id))
            .collect()
    }

    /// Sets the appearance id of every player `ids` names to -1
    /// (`STRIPPED_APPEARANCE_ID`): the tables decide his look from then on.
    pub(crate) fn strip(&mut self, ids: &[u32]) {
        for player in self.file.players_mut() {
            if ids.contains(&player.id) {
                player.appearance_id = Some(STRIPPED_APPEARANCE_ID);
            }
        }
    }

    /// Writes the save to `out`, an existing file there first copied to `<out>.bak` (one
    /// level, as every save does); `out` may be the path the save was opened from.
    pub(crate) fn write(&self, out: &Path) -> anyhow::Result<()> {
        self.file
            .save(out)
            .with_context(|| format!("{}: cannot write the save", out.display()))
    }
}

/// Whether the player with `id` is at appearance id -1 (`STRIPPED_APPEARANCE_ID`): the
/// tables decide his look, not his record. A player the save lacks is not stripped.
pub(crate) fn is_stripped(file: &EditFile, id: u32) -> bool {
    file.player(id)
        .is_some_and(|player| player.appearance_id == Some(STRIPPED_APPEARANCE_ID))
}

/// The import error as the command reports it: a text that does not fit names the player
/// it is for (the roster slot's player id), since `players.NN` alone does not say who that
/// is in a save of thousands.
fn import_error(error: TeamTomlError, roster: &[RosterSlot]) -> anyhow::Error {
    if let TeamTomlError::TextTooLong { path, .. } = &error
        && let Some(player) = rostered_player(path, roster)
    {
        return anyhow!("player {player}: {error}");
    }
    error.into()
}

/// The player id of the roster slot a `players.NN…` key path names, when the slot is
/// rostered.
fn rostered_player(path: &str, roster: &[RosterSlot]) -> Option<u32> {
    let slot: usize = path.strip_prefix("players.")?.get(..2)?.parse().ok()?;
    let player = roster.get(slot.checked_sub(1)?)?.player_id;
    (player != 0).then_some(player)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_too_long_error_names_the_rostered_player_and_other_errors_pass_through() {
        let roster = [
            RosterSlot {
                player_id: 71401,
                number: 1,
            },
            RosterSlot {
                player_id: 0,
                number: 0,
            },
            RosterSlot {
                player_id: 71403,
                number: 3,
            },
        ];
        let too_long = |path: &str| TeamTomlError::TextTooLong {
            path: path.to_owned(),
            max: 46,
        };
        assert_eq!(
            import_error(too_long("players.03.name"), &roster).to_string(),
            "player 71403: players.03.name: does not fit the field (max 46)"
        );
        // An empty slot, a slot past the roster and a team text name no player.
        for path in ["players.02.name", "players.04.name", "team.name"] {
            assert_eq!(
                import_error(too_long(path), &roster).to_string(),
                format!("{path}: does not fit the field (max 46)"),
                "{path}"
            );
        }
        assert_eq!(
            import_error(TeamTomlError::NoSuchSlot { slot: 41 }, &roster).to_string(),
            "roster has no slot 41"
        );
    }
}
