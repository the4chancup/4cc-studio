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
use pes_savefile::{EditFile, SaveError};
use pes_version::PesVersion;
use studio_core::RunningPes;

use crate::session::undo::{Change, UndoStack};

/// The ticks `Session::revision` draws from, one per open and per change:
/// a revision never repeats, so a cache keyed on it (the view's badges) is
/// never stale across saves.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

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
    /// (`STRIPPED_APPEARANCE_ID`): the tables decide his look from then on.
    pub(crate) fn strip(&mut self, ids: &[u32]) {
        for player in self.file.players_mut() {
            if ids.contains(&player.id) {
                player.appearance_id = Some(STRIPPED_APPEARANCE_ID);
            }
        }
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
    use super::*;

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
        let dir = std::env::temp_dir().join("save_editor_session_dirty");
        std::fs::create_dir_all(&dir).unwrap();
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
        let dir = std::env::temp_dir().join("save_editor_session_dirty_flag");
        std::fs::create_dir_all(&dir).unwrap();
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
        let dir = std::env::temp_dir().join("save_editor_session_import_dirty");
        std::fs::create_dir_all(&dir).unwrap();
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
        let dir = std::env::temp_dir().join("save_editor_session_revision");
        std::fs::create_dir_all(&dir).unwrap();
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
}
