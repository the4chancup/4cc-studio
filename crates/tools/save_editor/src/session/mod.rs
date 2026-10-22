//! The session: the save that is open, and the operations over it that the commands run (and
//! the window will, through its undo stack). egui-free, so the command line and the window
//! cannot diverge (`spec/save_editor.md` "Crate layout"). `pes_savefile` owns every byte and
//! every operation on the model; the session resolves ids against the open save and hands
//! the lib the entries it asks for.

pub(crate) mod rules;
pub(crate) mod selection;
pub(crate) mod undo;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, anyhow};
use pes_savefile::interchange::aesthetics_patch::{AestheticsPatch, PatchError, PatchSummary};
use pes_savefile::interchange::team_toml::{ImportNote, TeamToml};
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_savefile::model::{PlayerEntry, TeamEntry};
use pes_savefile::ops::transplant::{TransplantError, transplant};
use pes_savefile::schema::ingame_face::IngameFaceField;
use pes_savefile::{EditFile, SaveError};
use pes_version::PesVersion;
use studio_core::RunningPes;

use crate::session::undo::{Change, UndoStack};

/// The ticks `Session::revision` draws from, one per open and per change:
/// a revision never repeats, so a cache keyed on it (the view's badges) is
/// never stale across saves.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

/// The undo merge key `move_player`'s changes go under: a whole move is one
/// entry and never part of a widget's open gesture, so it needs no widget id
/// (the move seals its entry at the end regardless).
const MOVE_KEY: u64 = u64::MAX;

/// The undo merge key `strip`'s changes go under, `MOVE_KEY`'s twin: the whole
/// strip is one sealed entry, never part of a widget's open gesture.
const STRIP_KEY: u64 = u64::MAX - 1;

/// The fields of the in-game face run `player_export` carries (its appearance
/// section ends at `iris_col`; the face structure and the hair are not in the
/// struct): a move trades them with the record, the rest of the run stays with
/// the slot's id (`move_player`, SE-GUI-08).
const EXPORT_FACE_FIELDS: [IngameFaceField; 4] = [
    IngameFaceField::PlayerGloves,
    IngameFaceField::PlayerGlovesColor,
    IngameFaceField::SkinColor,
    IngameFaceField::IrisColor,
];

/// The open save: the file, the path it came from, the undo stack and the
/// baseline the unsaved-changes marks compare against (the file's players
/// and teams as opened or last saved).
pub(crate) struct Session {
    file: EditFile,
    /// The path `open` read, which `save` writes back to.
    path: PathBuf,
    /// Undo and redo (`plans/save_editor.md` "Editing UI", the Undo/redo bullet).
    undo: UndoStack,
    /// Every player by id, as opened or last saved.
    baseline_players: HashMap<u32, PlayerEntry>,
    /// Every team by id, as opened or last saved.
    baseline_teams: HashMap<u32, TeamEntry>,
    /// Whether the model differs from the baseline, recomputed by each
    /// mutating call (the toolbar's marker reads the flag, not a rescan).
    dirty: bool,
    /// The tick of the last change to the open file (`revision()`; every
    /// change takes a fresh one, so it never repeats).
    revision: u64,
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
        let (baseline_players, baseline_teams) = baselines(&file);
        Ok(Session {
            file,
            path: path.to_path_buf(),
            undo: UndoStack::new(),
            baseline_players,
            baseline_teams,
            dirty: false,
            revision: NEXT_REVISION.fetch_add(1, Ordering::Relaxed),
        })
    }

    /// The path the save was opened from, where Save writes it back.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The open save's game version.
    pub(crate) fn version(&self) -> PesVersion {
        self.file.version()
    }

    /// The open save, for the read-only operations over two saves (the comparator).
    pub(crate) fn file(&self) -> &EditFile {
        &self.file
    }

    /// The tick of the last change to the open file (`plans/save_editor.md`
    /// "Editing UI", the Cards left badges' cache): a fresh number every
    /// change — an edit pushed, an undo, a redo, an import, a patch, a
    /// transplant, a strip, a successful open — and never twice the same.
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    /// Marks the model changed: the next `revision`.
    fn touch(&mut self) {
        self.revision = NEXT_REVISION.fetch_add(1, Ordering::Relaxed);
    }

    /// The ids of the save's teams, in record order.
    pub(crate) fn team_ids(&self) -> impl Iterator<Item = u32> {
        self.file.teams().iter().map(|team| team.id)
    }

    /// Replaces the player `edited.id` names with `edited` and pushes the
    /// change under `key` (the widget's `egui::Id` value, the undo stack's
    /// merge key); `edited` equal to the stored entry pushes nothing.
    pub(crate) fn edit_player(&mut self, key: u64, edited: PlayerEntry) {
        let Some(entry) = self.file.player_mut(edited.id) else {
            return;
        };
        if *entry == edited {
            return;
        }
        let before = std::mem::replace(entry, edited.clone());
        self.undo.push(
            key,
            Change::Player {
                before: Box::new(before),
                after: Box::new(edited),
            },
        );
        self.touch();
        self.dirty = self.compute_dirty();
    }

    /// `edit_player` for a team entry (a roster slot's number, the captain).
    pub(crate) fn edit_team(&mut self, key: u64, edited: TeamEntry) {
        let Some(entry) = self.file.team_mut(edited.id) else {
            return;
        };
        if *entry == edited {
            return;
        }
        let before = std::mem::replace(entry, edited.clone());
        self.undo.push(
            key,
            Change::Team {
                before: Box::new(before),
                after: Box::new(edited),
            },
        );
        self.touch();
        self.dirty = self.compute_dirty();
    }

    /// The player-list drop (`plans/save_editor.md` "Editing UI", the Player list
    /// bullet): the record on roster slot `from` lands on `to`'s player id and the
    /// records between shift one filled slot along, each slot keeping what belongs
    /// to its id (`id`, `appearance_id`, `appearance.base_copy_id`, and the face and
    /// hair in the in-game face run: `moved_record`). The roster is never written,
    /// so its ids, the shirt numbers and the captain stay put (slot `NN` holds
    /// player `team id x 100 + NN` by 4cc rule). The whole move is one sealed undo entry. `from == to`, a team
    /// or player the save lacks, or an empty `from` or `to` slot does nothing; an
    /// empty slot between them is skipped over.
    pub(crate) fn move_player(&mut self, team_id: u32, from: u8, to: u8) {
        if from == to {
            return;
        }
        let Some(team) = self.file.team(team_id) else {
            return;
        };
        let (from, to) = (usize::from(from), usize::from(to));
        let Some(range) = team.roster.get(from.min(to)..=from.max(to)) else {
            return;
        };
        let lo = from.min(to);
        if range[from - lo].player_id == 0 || range[to - lo].player_id == 0 {
            return;
        }
        // The filled slots' ids in slot order: the sequence the move rotates.
        let filled: Vec<u32> = range
            .iter()
            .filter(|slot| slot.player_id != 0)
            .map(|slot| slot.player_id)
            .collect();
        let mut order = filled.clone();
        let taken = order.remove(
            filled
                .iter()
                .position(|id| *id == range[from - lo].player_id)
                .expect("`from` is filled"),
        );
        order.insert(
            filled
                .iter()
                .position(|id| *id == range[to - lo].player_id)
                .expect("`to` is filled"),
            taken,
        );
        // Each filled slot's player takes `order`'s record for it, keeping the
        // slot's own part of it (`moved_record`). Every before and after is
        // computed before the first write: a rostered id the save lacks leaves
        // it untouched.
        let mut moved = Vec::new();
        for (&slot_id, &record_id) in filled.iter().zip(&order) {
            if slot_id == record_id {
                continue;
            }
            let (Some(dest), Some(source)) =
                (self.file.player(slot_id), self.file.player(record_id))
            else {
                return;
            };
            moved.push((slot_id, dest.clone(), moved_record(source, dest)));
        }
        for (id, before, after) in moved {
            *self.file.player_mut(id).expect("resolved above") = after.clone();
            self.undo.push(
                MOVE_KEY,
                Change::Player {
                    before: Box::new(before),
                    after: Box::new(after),
                },
            );
        }
        // One entry for the whole move, sealed: the next edit does not merge.
        self.undo.seal();
        self.touch();
        self.dirty = self.compute_dirty();
    }

    /// Undoes the newest gesture; `false` when the stack is empty.
    pub(crate) fn undo(&mut self) -> bool {
        let undone = self.undo.undo(&mut self.file);
        if undone {
            self.touch();
            self.dirty = self.compute_dirty();
        }
        undone
    }

    /// Reapplies the newest undone gesture; `false` when the redo stack is
    /// empty.
    pub(crate) fn redo(&mut self) -> bool {
        let redone = self.undo.redo(&mut self.file);
        if redone {
            self.touch();
            self.dirty = self.compute_dirty();
        }
        redone
    }

    /// Whether `undo` would do anything (the Undo button's enabled state).
    pub(crate) fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    /// Whether `redo` would do anything.
    pub(crate) fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    /// Ends the open gesture: the next change starts a fresh undo entry.
    pub(crate) fn seal(&mut self) {
        self.undo.seal();
    }

    /// Whether the save differs from the file as opened or last saved (the
    /// plan's Unsaved changes bullet; the toolbar's marker). The flag is kept
    /// by `compute_dirty` at every mutating call, so a frame can ask it
    /// without rescanning the model.
    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// The deep compare behind `dirty`: any player or team unlike its
    /// baseline entry.
    fn compute_dirty(&self) -> bool {
        self.file
            .players()
            .iter()
            .any(|player| self.baseline_players.get(&player.id) != Some(player))
            || self
                .file
                .teams()
                .iter()
                .any(|team| self.baseline_teams.get(&team.id) != Some(team))
    }

    /// Whether the player of `team`'s roster `slot` (0-based) differs from
    /// the baseline: his `PlayerEntry` or his slot's shirt number (the dot's
    /// rule, `plans/save_editor.md` "Editing UI" Unsaved changes).
    pub(crate) fn player_dirty(&self, team_id: u32, slot: u8) -> bool {
        let Some(slot_entry) = self
            .file
            .team(team_id)
            .and_then(|team| team.roster.get(usize::from(slot)))
        else {
            return false;
        };
        let baseline_slot = self
            .baseline_teams
            .get(&team_id)
            .and_then(|team| team.roster.get(usize::from(slot)));
        if baseline_slot.is_none_or(|base| base != slot_entry) {
            return true;
        }
        self.file
            .player(slot_entry.player_id)
            .is_some_and(|player| self.baseline_players.get(&player.id) != Some(player))
    }

    /// Writes the save back to the path it was opened from, then makes the
    /// current model the baseline. Refused while the save's own PES runs (the
    /// game writes its save back when it closes, over ours); a game of
    /// another version does not block it.
    pub(crate) fn save(&mut self, running: &[RunningPes]) -> anyhow::Result<()> {
        refuse_while_running(running, self.version())?;
        self.write(&self.path)?;
        (self.baseline_players, self.baseline_teams) = baselines(&self.file);
        self.dirty = false;
        Ok(())
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
    /// one naming a team the save lacks, or one the lib refuses (a team name or short name
    /// that does not fit the version's field; a player's name or shirt name is cut to it with
    /// a warning instead) is an error and the save is untouched.
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
        let notes = document.apply(version, &mut team, self.file.players_mut())?;
        *self.file.team_mut(id).expect("found above") = team;
        self.touch();
        self.dirty = self.compute_dirty();
        Ok(notes)
    }

    /// Applies the compiler's patch: each listed player's motions and names, nothing of his
    /// appearance block. The summary lists the teams applied and the items the save lacked;
    /// a version or scheme mismatch, or a pre-Fox patch, is an error with the save untouched.
    pub(crate) fn apply_patch(
        &mut self,
        patch: &AestheticsPatch,
    ) -> Result<PatchSummary, PatchError> {
        let summary = patch.apply(&mut self.file)?;
        self.touch();
        self.dirty = self.compute_dirty();
        Ok(summary)
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
        self.touch();
        self.dirty = self.compute_dirty();
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
    /// (`STRIPPED_APPEARANCE_ID`): the tables decide his look from then on. The
    /// whole strip is one sealed undo entry (SE-GUI-20's Strip again), so one
    /// Undo restores every id; an id the save lacks, or a player already at -1,
    /// is left out.
    pub(crate) fn strip(&mut self, ids: &[u32]) {
        // The before/after pairs are computed before the first push, as in
        // `move_player`: the write and the undo entry are the same data.
        let mut changed = Vec::new();
        for player in self.file.players_mut() {
            if !ids.contains(&player.id) || player.appearance_id == Some(STRIPPED_APPEARANCE_ID) {
                continue;
            }
            let mut after = player.clone();
            after.appearance_id = Some(STRIPPED_APPEARANCE_ID);
            let before = std::mem::replace(player, after.clone());
            changed.push((before, after));
        }
        if changed.is_empty() {
            return;
        }
        for (before, after) in changed {
            self.undo.push(
                STRIP_KEY,
                Change::Player {
                    before: Box::new(before),
                    after: Box::new(after),
                },
            );
        }
        // One entry for the whole strip, sealed: the next edit does not merge.
        self.undo.seal();
        self.touch();
        self.dirty = self.compute_dirty();
    }

    /// Writes the save to `out`, an existing file there first copied to `<out>.bak` (one
    /// level, as every save does); `out` may be the path the save was opened from.
    pub(crate) fn write(&self, out: &Path) -> anyhow::Result<()> {
        self.file
            .save(out)
            .with_context(|| format!("{}: cannot write the save", out.display()))
    }
}

/// `source`'s record retargeted onto `dest`'s slot (`move_player`): the fields
/// `player_export` carries move with the record; the fields it does not carry
/// are `dest`'s — `id`, `appearance_id`, `base_copy_id`, and the in-game face
/// run apart from `EXPORT_FACE_FIELDS`, whose four the export holds and so
/// moves (its appearance section ends at `iris_col`; the face structure and
/// the hair are not in the struct).
fn moved_record(source: &PlayerEntry, dest: &PlayerEntry) -> PlayerEntry {
    let mut record = source.clone();
    record.id = dest.id;
    record.appearance_id = dest.appearance_id;
    record.appearance.base_copy_id = dest.appearance.base_copy_id;
    record.appearance.ingame_face = dest.appearance.ingame_face.clone();
    // A loaded save reads every player's whole run, and both layouts hold
    // the four fields, so neither call can fail on a record of the save.
    for field in EXPORT_FACE_FIELDS {
        let value = source
            .appearance
            .ingame_face
            .get(field)
            .expect("a loaded player's face run holds the exported fields");
        record
            .appearance
            .ingame_face
            .set(field, value)
            .expect("two players of one save share a face layout");
    }
    record
}

/// The save's players and teams by id: the baseline `is_dirty` and
/// `player_dirty` compare against, taken at `open` and after `save`.
fn baselines(file: &EditFile) -> (HashMap<u32, PlayerEntry>, HashMap<u32, TeamEntry>) {
    (
        file.players()
            .iter()
            .map(|player| (player.id, player.clone()))
            .collect(),
        file.teams()
            .iter()
            .map(|team| (team.id, team.clone()))
            .collect(),
    )
}

/// The refusal a writing operation makes while the PES of `version` runs: the
/// game writes its own save back when it closes, over the one just written.
/// A game of another version never writes this save, so it does not refuse
/// (the CLI's rule: `spec/save_editor.md` "CLI" SE-CLI-02).
pub(crate) fn refuse_while_running(
    running: &[RunningPes],
    version: PesVersion,
) -> anyhow::Result<()> {
    match running.iter().find(|game| game.version == version) {
        Some(game) => Err(anyhow!(
            "{} is running: it writes its own save back when it closes, over this one; \
             close the game first",
            game.version
        )),
        None => Ok(()),
    }
}

/// Whether the player with `id` is at appearance id -1 (`STRIPPED_APPEARANCE_ID`): the
/// tables decide his look, not his record. A player the save lacks is not stripped.
pub(crate) fn is_stripped(file: &EditFile, id: u32) -> bool {
    file.player(id)
        .is_some_and(|player| player.appearance_id == Some(STRIPPED_APPEARANCE_ID))
}

#[cfg(test)]
mod tests {
    use pes_savefile::model::ingame_face::IngameFace;
    use pes_savefile::schema::ingame_face::IngameFaceField;

    use super::*;

    /// Writes `version`'s fixture save under the test's scratch folder and
    /// opens it as a `Session`.
    fn open(version: PesVersion, test: &str) -> Session {
        let dir = crate::test_support::scratch(test);
        let path = dir.join("EDIT00000000");
        std::fs::write(&path, pes_savefile::fixture_save(version)).unwrap();
        Session::open(&path).unwrap()
    }

    /// `entry` without what a roster slot owns (`id`, `appearance_id`,
    /// `appearance.base_copy_id`, and the in-game face run apart from the
    /// exported fields, which `a_move_carries_only_the_exported_face_fields`
    /// checks on its own): the record a move carries.
    fn record(entry: &PlayerEntry) -> PlayerEntry {
        let mut record = entry.clone();
        record.id = 0;
        record.appearance_id = None;
        record.appearance.base_copy_id = 0;
        record.appearance.ingame_face = IngameFace::default();
        record
    }

    // plans/save_editor.md "Editing UI", the Player list bullet: a drop moves
    // the players' records between the ids, every `id`, `appearance_id` and
    // `base_copy_id` staying with its slot; the roster (and so the shirt
    // numbers) is never written, and the whole move is one undo entry.
    #[test]
    fn a_move_trades_the_records_between_the_ids_one_undo_entry() {
        let mut session = open(PesVersion::Pes21, "move");
        // The fixture's placeholders are identical records: distinguish the
        // three the move rotates, so the trades are not vacuous; a save makes
        // them the baseline, so `is_dirty` measures the move alone.
        session.file.player_mut(70101).unwrap().name = "FIRST".to_owned();
        session.file.player_mut(70102).unwrap().stats.speed = 71;
        session.file.player_mut(70103).unwrap().appearance.boots_id = 13;
        session.save(&[]).unwrap();
        let before: Vec<PlayerEntry> = [70101, 70102, 70103]
            .iter()
            .map(|id| session.file().player(*id).unwrap().clone())
            .collect();
        let roster_before = session.file().team(701).unwrap().roster.clone();
        assert_ne!(
            record(&before[0]),
            record(&before[1]),
            "premise: the records differ"
        );
        assert_ne!(
            record(&before[1]),
            record(&before[2]),
            "premise: the records differ"
        );

        session.move_player(701, 2, 0);

        // Slot 0's id is still 70101 and holds the record 70103 had; 70102
        // holds 70101's, 70103 70102's.
        let after: Vec<PlayerEntry> = [70101, 70102, 70103]
            .iter()
            .map(|id| session.file().player(*id).unwrap().clone())
            .collect();
        for (moved, held) in after.iter().zip([2usize, 0, 1]) {
            assert_eq!(
                record(moved),
                record(&before[held]),
                "the record player {} had",
                before[held].id
            );
        }
        for (moved, id) in after.iter().zip([70101u32, 70102, 70103]) {
            assert_eq!(moved.id, id, "the id stays with the slot");
            assert_eq!(moved.appearance_id, Some(id), "the appearance id too");
        }
        assert_eq!(
            after[0].appearance.base_copy_id, before[0].appearance.base_copy_id,
            "the base copy stays with the slot"
        );
        assert_eq!(
            after[1].appearance.base_copy_id, before[1].appearance.base_copy_id,
            "the base copy stays with the slot"
        );
        assert_eq!(
            after[2].appearance.base_copy_id, before[2].appearance.base_copy_id,
            "the base copy stays with the slot"
        );
        assert_eq!(
            session.file().team(701).unwrap().roster,
            roster_before,
            "the roster's ids and shirt numbers are unchanged"
        );
        assert!(session.is_dirty());
        assert!(session.player_dirty(701, 0), "the moved players are marked");

        // One undo restores all three entries exactly; one redo reapplies.
        assert!(session.undo());
        for (id, entry) in [70101u32, 70102, 70103].iter().zip(&before) {
            assert_eq!(session.file().player(*id).unwrap(), entry);
        }
        assert!(!session.can_undo(), "the whole move is one undo entry");
        assert!(session.redo());
        for (id, entry) in [70101u32, 70102, 70103].iter().zip(&after) {
            assert_eq!(session.file().player(*id).unwrap(), entry);
        }
    }

    // A move to itself, onto an empty slot, from an empty slot or past the
    // roster, and a team the save lacks: nothing changes, no undo entry.
    #[test]
    fn a_move_that_names_no_two_filled_slots_does_nothing() {
        let mut session = open(PesVersion::Pes21, "move_noop");
        let empty = session
            .file()
            .team(701)
            .unwrap()
            .roster
            .iter()
            .position(|slot| slot.player_id == 0)
            .map(|i| u8::try_from(i).unwrap())
            .expect("the roster has an empty slot");

        session.move_player(701, 0, 0);
        session.move_player(701, 0, empty);
        session.move_player(701, empty, 0);
        session.move_player(701, 0, 250);
        session.move_player(99999, 0, 1);

        assert!(!session.is_dirty());
        assert!(!session.can_undo());
    }

    // plans/save_editor.md "Editing UI", the Player list bullet: an empty
    // slot between the dragged and the target is skipped over — the records
    // rotate over the filled slots.
    #[test]
    fn a_move_skips_over_an_empty_slot() {
        let mut session = open(PesVersion::Pes21, "move_empty");
        // Distinguish the records (the fixture's placeholders are identical),
        // then empty out slot 1 (70102's record stays in the pool, unrostered).
        session.file.player_mut(70101).unwrap().name = "FIRST".to_owned();
        session.file.player_mut(70103).unwrap().name = "THIRD".to_owned();
        session.file.team_mut(701).unwrap().roster[1].player_id = 0;
        let before: Vec<PlayerEntry> = [70101, 70102, 70103]
            .iter()
            .map(|id| session.file().player(*id).unwrap().clone())
            .collect();
        assert_ne!(
            record(&before[0]),
            record(&before[2]),
            "premise: the records differ"
        );

        session.move_player(701, 0, 2);

        // Slot 0 (70101) holds 70103's record, slot 2 (70103) 70101's; the
        // skipped slot stays empty and 70102's record is untouched.
        assert_eq!(
            record(session.file().player(70101).unwrap()),
            record(&before[2])
        );
        assert_eq!(
            record(session.file().player(70103).unwrap()),
            record(&before[0])
        );
        assert_eq!(
            record(session.file().player(70102).unwrap()),
            record(&before[1])
        );
        assert_eq!(session.file().team(701).unwrap().roster[1].player_id, 0);
        assert_eq!(session.file().team(701).unwrap().roster[0].player_id, 70101);
        assert_eq!(session.file().team(701).unwrap().roster[2].player_id, 70103);
    }

    /// Writes `value` into `field` of `id`'s in-game face run.
    fn set_face(session: &mut Session, id: u32, field: IngameFaceField, value: u8) {
        session
            .file
            .player_mut(id)
            .unwrap()
            .appearance
            .ingame_face
            .set(field, value)
            .unwrap();
    }

    /// `field` of `id`'s in-game face run.
    fn get_face(session: &Session, id: u32, field: IngameFaceField) -> u8 {
        session
            .file()
            .player(id)
            .unwrap()
            .appearance
            .ingame_face
            .get(field)
            .unwrap()
    }

    // plans/save_editor.md "Editing UI", the Player list bullet: the move
    // carries the fields `player_export` carries — the player-gloves, skin
    // and iris fields inside the face run — and leaves the rest of the run
    // (the face structure and the hair) with the slot.
    #[test]
    fn a_move_carries_only_the_exported_face_fields() {
        face_fields_move(PesVersion::Pes21, "move_face21");
    }

    // The same on a version whose appearance block is a separate record
    // joined by id (PES 15/16, `appearance_id` `None`): it stays joined.
    #[test]
    fn a_move_carries_only_the_exported_face_fields_on_a_split_record() {
        face_fields_move(PesVersion::Pes16, "move_face16");
    }

    /// `marks`' (field, value on 70101, value on 70102) as the move leaves
    /// them in `session` (a `face_fields_move` assert, on the open session
    /// and again on the reopened save): the exported fields hold the
    /// source's value, the rest the slot's own.
    fn assert_face_move(session: &Session, marks: &[(IngameFaceField, u8, u8)]) {
        use IngameFaceField as F;
        for &(field, a, b) in marks {
            let carried = matches!(
                field,
                F::PlayerGloves | F::PlayerGlovesColor | F::SkinColor | F::IrisColor
            );
            // Slot 01 took 70102's record, slot 02 70101's: a carried field
            // holds the source's value, a kept one the slot's own.
            let (on_01, on_02) = if carried { (b, a) } else { (a, b) };
            assert_eq!(get_face(session, 70101, field), on_01, "{field:?}");
            assert_eq!(get_face(session, 70102, field), on_02, "{field:?}");
        }
    }

    /// The face-field half of `a_move_trades_the_records_between_the_ids` on
    /// `version`: 70102's record moved onto slot 01 trades the exported
    /// fields (gloves, skin, iris) and keeps each slot's own face and hair,
    /// in memory and as the file written by Save holds it.
    fn face_fields_move(version: PesVersion, test: &str) {
        use IngameFaceField as F;
        let mut session = open(version, test);
        // The fixture's runs are identical: mark each of the seven fields the
        // split decides, four carried by `player_export` and three that stay
        // (a hair field among them).
        let marks: [(F, u8, u8); 7] = [
            (F::Jawline, 5, 2),
            (F::NoseType, 7, 3),
            (F::HairStyle, 2, 5),
            (F::PlayerGloves, 1, 0),
            (F::PlayerGlovesColor, 5, 2),
            (F::SkinColor, 6, 2),
            (F::IrisColor, 9, 4),
        ];
        for &(field, a, b) in &marks {
            set_face(&mut session, 70101, field, a);
            set_face(&mut session, 70102, field, b);
        }
        for &(field, a, b) in &marks {
            assert_eq!(get_face(&session, 70101, field), a, "premise");
            assert_eq!(get_face(&session, 70102, field), b, "premise");
        }

        session.move_player(701, 1, 0);
        assert_face_move(&session, &marks);

        // Saved and reopened: the moved fields and the slots' faces are what
        // the file holds (SE-GUI-08's THEN after Save).
        session.save(&[]).unwrap();
        let reopened = Session::open(session.path()).unwrap();
        assert_face_move(&reopened, &marks);
    }

    // plans/save_editor.md "Editing UI", the Undo/redo bullet: one entry per
    // gesture — a second drag is its own entry, not a merge into the first's.
    #[test]
    fn two_moves_are_two_undo_steps() {
        let mut session = open(PesVersion::Pes21, "two_moves");
        session.file.player_mut(70101).unwrap().name = "FIRST".to_owned();
        session.file.player_mut(70102).unwrap().name = "SECOND".to_owned();
        session.file.player_mut(70103).unwrap().name = "THIRD".to_owned();

        session.move_player(701, 1, 0);
        session.move_player(701, 2, 1);
        assert_eq!(session.file().player(70101).unwrap().name, "SECOND");
        assert_eq!(session.file().player(70102).unwrap().name, "THIRD");
        assert_eq!(session.file().player(70103).unwrap().name, "FIRST");

        // One undo reverts only the second move; the next undoes the first.
        assert!(session.undo());
        assert_eq!(session.file().player(70101).unwrap().name, "SECOND");
        assert_eq!(session.file().player(70102).unwrap().name, "FIRST");
        assert_eq!(session.file().player(70103).unwrap().name, "THIRD");
        assert!(session.undo());
        assert_eq!(session.file().player(70101).unwrap().name, "FIRST");
        assert_eq!(session.file().player(70102).unwrap().name, "SECOND");
        assert_eq!(session.file().player(70103).unwrap().name, "THIRD");
        assert!(!session.can_undo());
    }

    #[test]
    fn refuse_while_running_refuses_only_the_saves_own_version() {
        assert!(refuse_while_running(&[], PesVersion::Pes21).is_ok());
        let game = |version| RunningPes { version, exe: None };
        // Another version's game never writes this save.
        assert!(refuse_while_running(&[game(PesVersion::Pes15)], PesVersion::Pes21).is_ok());
        let running = [game(PesVersion::Pes15), game(PesVersion::Pes21)];
        let error = refuse_while_running(&running, PesVersion::Pes21).unwrap_err();
        assert!(error.to_string().contains("PES 2021"), "{error}");
    }

    #[test]
    fn a_players_dot_covers_his_entry_and_his_roster_number() {
        let dir = crate::test_support::scratch("dirty");
        let path = dir.join("EDIT00000000");
        std::fs::write(&path, pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let mut session = Session::open(&path).unwrap();
        assert!(!session.is_dirty());
        assert!(!session.player_dirty(701, 0));

        // A player edit marks him.
        let mut edited = session.file().player(70101).unwrap().clone();
        edited.basic.height = edited.basic.height.wrapping_add(1);
        session.edit_player(1, edited);
        assert!(session.player_dirty(701, 0));
        assert!(session.is_dirty());

        // Undo clears his mark; editing his roster number marks him too.
        assert!(session.undo());
        assert!(!session.player_dirty(701, 0));
        let mut team = session.file().team(701).unwrap().clone();
        team.roster[0].number = team.roster[0].number.wrapping_add(1);
        session.edit_team(1, team);
        assert!(session.player_dirty(701, 0));

        // A roster number elsewhere marks nobody here.
        assert!(!session.player_dirty(701, 1));

        // Save makes the current model the baseline: clean again.
        session.save(&[]).unwrap();
        assert!(!session.is_dirty());
        assert!(!session.player_dirty(701, 0));
    }

    #[test]
    fn the_dirty_flag_and_the_redo_stack_track_edits_undos_redos_and_save() {
        let dir = crate::test_support::scratch("dirty_flag");
        let path = dir.join("EDIT00000000");
        std::fs::write(&path, pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let mut session = Session::open(&path).unwrap();
        assert!(!session.can_redo(), "nothing to redo on an open save");

        let mut edited = session.file().player(70101).unwrap().clone();
        edited.basic.height = edited.basic.height.wrapping_add(1);
        session.edit_player(1, edited);
        assert!(session.is_dirty(), "an edit marks the save");
        assert!(!session.can_redo(), "a new edit clears redo");

        assert!(session.undo());
        assert!(
            !session.is_dirty(),
            "undoing back to the baseline clears it"
        );
        assert!(session.can_redo(), "the undone edit is redoable");

        assert!(session.redo());
        assert!(session.is_dirty(), "redo marks it again");
        session.save(&[]).unwrap();
        assert!(!session.is_dirty(), "a save clears it");
    }

    // The mutators keep `dirty` beside `revision`: an import that changes a
    // player marks the save too (the window's unsaved-changes marker reads
    // `is_dirty`, and the interchange commands are the view's write paths).
    #[test]
    fn an_import_marks_the_save_dirty() {
        let dir = crate::test_support::scratch("import_dirty");
        let path = dir.join("EDIT00000000");
        std::fs::write(&path, pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let mut session = Session::open(&path).unwrap();
        assert!(!session.is_dirty());

        use pes_savefile::interchange::team_toml::{PlayerSection, TeamSection};
        let mut document = TeamToml {
            team: TeamSection {
                id: Some(701),
                ..Default::default()
            },
            ..Default::default()
        };
        document.players.insert(
            3,
            PlayerSection {
                name: Some("NEWNAME".to_string()),
                ..Default::default()
            },
        );
        session.import_team(&document).unwrap();
        assert!(session.is_dirty(), "the import changed a player");
    }

    // plans/save_editor.md "Editing UI", the Cards left bullet's cache: the
    // revision moves on every change to the open file (an edit pushed, an
    // undo, a redo) and not on a frame's seal.
    #[test]
    fn the_revision_moves_on_each_change_and_stays_put_on_a_frame() {
        let dir = crate::test_support::scratch("revision");
        let path = dir.join("EDIT00000000");
        std::fs::write(&path, pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let mut session = Session::open(&path).unwrap();
        let opened = session.revision();

        // The per-frame seal is not a change.
        session.seal();
        assert_eq!(session.revision(), opened, "a frame's seal moves nothing");

        let mut edited = session.file().player(70101).unwrap().clone();
        edited.basic.height = edited.basic.height.wrapping_add(1);
        session.edit_player(1, edited);
        let after_edit = session.revision();
        assert_ne!(after_edit, opened, "an edit takes the next tick");
        assert!(session.undo());
        let after_undo = session.revision();
        assert_ne!(after_undo, after_edit, "an undo takes the next tick");
        assert!(session.redo());
        assert_ne!(session.revision(), after_undo, "a redo takes the next tick");
    }

    // plans/save_editor.md "Stripped save" (the GUI parts): `strip` writes
    // every id as one sealed undo entry, so the Strip again button's whole
    // change is one Undo. An already-stripped player pushes nothing.
    #[test]
    fn a_strip_is_one_sealed_undo_entry() {
        let mut session = open(PesVersion::Pes21, "strip_undo");
        assert_eq!(
            session.file().player(70101).unwrap().appearance_id,
            Some(70101),
            "premise: the fixture's ids are set"
        );

        session.strip(&[70101, 70103]);

        for id in [70101u32, 70103] {
            assert_eq!(
                session.file().player(id).unwrap().appearance_id,
                Some(STRIPPED_APPEARANCE_ID),
                "{id} is stripped"
            );
        }
        assert!(session.is_dirty());

        // One undo restores both ids; the stack is then empty (the strip
        // sealed its entry, so no gesture could merge into it).
        assert!(session.undo());
        for id in [70101u32, 70103] {
            assert_eq!(
                session.file().player(id).unwrap().appearance_id,
                Some(id),
                "{id} is restored"
            );
        }
        assert!(!session.can_undo(), "the whole strip was one entry");
        assert!(session.redo());
        assert_eq!(
            session.file().player(70101).unwrap().appearance_id,
            Some(STRIPPED_APPEARANCE_ID)
        );

        // A second strip changes nobody: no entry, no dirt left once undone.
        session.undo();
        assert!(!session.is_dirty());
        session.strip(&[70101]);
        assert_eq!(
            session.file().player(70101).unwrap().appearance_id,
            Some(STRIPPED_APPEARANCE_ID)
        );
        session.undo();
        assert!(!session.is_dirty());
        session.strip(&[]);
        assert!(!session.can_undo(), "an empty strip pushes nothing");
    }
}
