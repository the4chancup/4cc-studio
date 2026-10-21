//! The undo and redo stacks (`plans/save_editor.md` "Editing UI", the Undo/redo
//! bullet): an undo entry is the before and after of each `PlayerEntry` or
//! `TeamEntry` it changed, one entry per gesture — a widget's changes merge
//! into the open entry while it stays unsealed.

use pes_savefile::EditFile;
use pes_savefile::model::{PlayerEntry, TeamEntry};

/// One model entry an edit changed, before and after. The entries are boxed: a
/// `TeamEntry` (its tactics included) is more than twice a `PlayerEntry`'s size, and
/// unboxed every change would carry the larger variant's room.
pub(crate) enum Change {
    /// A player's entry.
    Player {
        /// The entry before the edit.
        before: Box<PlayerEntry>,
        /// The entry after.
        after: Box<PlayerEntry>,
    },
    /// A team's entry.
    Team {
        /// The entry before the edit.
        before: Box<TeamEntry>,
        /// The entry after.
        after: Box<TeamEntry>,
    },
}

/// One undo step: the widget whose gesture made it (the merge key) and the
/// model entries it changed.
struct Entry {
    /// The host's widget id; only a change pushed under the same key merges
    /// into the open entry.
    key: u64,
    /// The entries the gesture changed so far.
    changes: Vec<Change>,
}

/// The session's undo and redo stacks. `undo.last()` is the open entry while
/// `open` holds: pushes under its key merge into it; `seal` or a push under
/// another key closes it.
pub(crate) struct UndoStack {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    open: bool,
}

impl Change {
    /// Whether the change does nothing.
    fn is_noop(&self) -> bool {
        match self {
            Change::Player { before, after } => before == after,
            Change::Team { before, after } => before == after,
        }
    }

    /// The same model entry another `Change` names: same kind (player or
    /// team), same id.
    fn same_entry(&self, other: &Change) -> bool {
        match (self, other) {
            (Change::Player { before: a, .. }, Change::Player { before: b, .. }) => a.id == b.id,
            (Change::Team { before: a, .. }, Change::Team { before: b, .. }) => a.id == b.id,
            (Change::Player { .. }, Change::Team { .. })
            | (Change::Team { .. }, Change::Player { .. }) => false,
        }
    }

    /// `self` retargeted to `other`'s after, keeping its own before (the
    /// merge's rule for a second change to one entry).
    fn merge_after(&mut self, other: Change) {
        match (self, other) {
            (Change::Player { after, .. }, Change::Player { after: new, .. }) => *after = new,
            (Change::Team { after, .. }, Change::Team { after: new, .. }) => *after = new,
            (Change::Player { .. }, Change::Team { .. })
            | (Change::Team { .. }, Change::Player { .. }) => {
                unreachable!("merge_after is called only under `same_entry`")
            }
        }
    }

    /// Writes `before`'s entry back into the file (`before` set) or
    /// `after`'s (cleared): undo and redo's halves of a restore. An entry
    /// the file lacks is left (the model never removes one).
    fn restore(&self, file: &mut EditFile, before: bool) {
        match self {
            Change::Player {
                before: old,
                after: new,
            } => {
                let value = if before { old } else { new };
                if let Some(player) = file.player_mut(value.id) {
                    *player = PlayerEntry::clone(value);
                }
            }
            Change::Team {
                before: old,
                after: new,
            } => {
                let value = if before { old } else { new };
                if let Some(team) = file.team_mut(value.id) {
                    *team = TeamEntry::clone(value);
                }
            }
        }
    }
}

impl UndoStack {
    /// An empty stack.
    pub(crate) fn new() -> UndoStack {
        UndoStack {
            undo: Vec::new(),
            redo: Vec::new(),
            open: false,
        }
    }

    /// Whether `undo` would do anything (the button's enabled state).
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether `redo` would do anything.
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Pushes `change` under `key`: a change equal to its own before is not
    /// pushed; a change under the open entry's key merges into it (an entry
    /// already holding that player or team keeps its `before` and takes the
    /// new `after`); any other push closes the open entry and starts one.
    /// A push clears the redo stack.
    pub(crate) fn push(&mut self, key: u64, change: Change) {
        if change.is_noop() {
            return;
        }
        self.redo.clear();
        match self.undo.last_mut() {
            Some(entry) if self.open && entry.key == key => {
                match entry
                    .changes
                    .iter_mut()
                    .find(|held| held.same_entry(&change))
                {
                    Some(held) => held.merge_after(change),
                    None => entry.changes.push(change),
                }
            }
            _ => {
                self.undo.push(Entry {
                    key,
                    changes: vec![change],
                });
                self.open = true;
            }
        }
    }

    /// Closes the open entry: the next push starts a fresh entry even under
    /// the same key (the gesture ended).
    pub(crate) fn seal(&mut self) {
        self.open = false;
    }

    /// Restores each change's `before` into the file and moves the newest
    /// entry to redo; `false` when the stack is empty.
    pub(crate) fn undo(&mut self, file: &mut EditFile) -> bool {
        self.seal();
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        for change in &entry.changes {
            change.restore(file, true);
        }
        self.redo.push(entry);
        true
    }

    /// Reapplies each change's `after` and moves the newest redo entry back;
    /// `false` when the redo stack is empty.
    pub(crate) fn redo(&mut self, file: &mut EditFile) -> bool {
        self.seal();
        let Some(entry) = self.redo.pop() else {
            return false;
        };
        for change in &entry.changes {
            change.restore(file, false);
        }
        self.undo.push(entry);
        self.open = false;
        true
    }
}

#[cfg(test)]
mod tests {
    use pes_savefile::fixture_save;
    use pes_savefile::model::player::PlayerBasics;
    use pes_version::PesVersion;

    use super::*;

    fn file() -> EditFile {
        EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).unwrap()
    }

    /// A `Change::Player` for `id` whose height goes `from` to `to`.
    fn change(id: u32, from: u8, to: u8) -> Change {
        let basic = |height| PlayerBasics {
            height,
            ..Default::default()
        };
        Change::Player {
            before: Box::new(PlayerEntry {
                id,
                basic: basic(from),
                ..Default::default()
            }),
            after: Box::new(PlayerEntry {
                id,
                basic: basic(to),
                ..Default::default()
            }),
        }
    }

    /// The height the file's player `id` holds.
    fn height(file: &EditFile, id: u32) -> u8 {
        file.player(id).unwrap().basic.height
    }

    #[test]
    fn pushes_under_one_key_merge_until_sealed() {
        let mut file = file();
        let mut stack = UndoStack::new();
        stack.push(1, change(70101, 10, 11));
        stack.push(1, change(70101, 11, 12));
        // One entry: undo restores the first push's before, not the last's.
        stack.undo(&mut file);
        assert_eq!(height(&file, 70101), 10);
        assert!(!stack.can_undo());

        stack.seal();
        stack.push(1, change(70101, 10, 11));
        stack.seal();
        stack.push(1, change(70101, 11, 12));
        // Two entries: the sealed first gesture is its own undo step.
        stack.undo(&mut file);
        assert_eq!(height(&file, 70101), 11);
        stack.undo(&mut file);
        assert_eq!(height(&file, 70101), 10);
    }

    #[test]
    fn a_push_under_another_key_starts_an_entry() {
        let mut file = file();
        let mut stack = UndoStack::new();
        stack.push(1, change(70101, 10, 11));
        stack.push(2, change(70102, 20, 21));
        stack.undo(&mut file);
        assert_eq!(height(&file, 70102), 20);
        stack.undo(&mut file);
        assert_eq!(height(&file, 70101), 10);
    }

    #[test]
    fn an_entry_merges_changes_to_different_players_under_one_key() {
        let mut file = file();
        let mut stack = UndoStack::new();
        stack.push(1, change(70101, 10, 11));
        stack.push(1, change(70102, 20, 21));
        // One entry holding both: undo restores both players at once.
        stack.undo(&mut file);
        assert_eq!(height(&file, 70101), 10);
        assert_eq!(height(&file, 70102), 20);
        assert!(!stack.can_undo());
    }

    #[test]
    fn a_push_clears_redo() {
        let mut file = file();
        let mut stack = UndoStack::new();
        stack.push(1, change(70101, 10, 11));
        stack.undo(&mut file);
        assert!(stack.can_redo());
        stack.push(2, change(70102, 20, 21));
        assert!(!stack.can_redo());
    }

    #[test]
    fn an_equal_after_pushes_nothing() {
        let mut stack = UndoStack::new();
        stack.push(1, change(70101, 10, 10));
        assert!(!stack.can_undo());
    }

    #[test]
    fn undo_then_redo_round_trips() {
        let mut file = file();
        let mut stack = UndoStack::new();
        let before = file.player(70101).unwrap().clone();
        file.player_mut(70101).unwrap().basic.height = 99;
        let after = file.player(70101).unwrap().clone();
        stack.push(
            1,
            Change::Player {
                before: Box::new(before),
                after: Box::new(after),
            },
        );
        stack.undo(&mut file);
        assert_ne!(height(&file, 70101), 99);
        stack.redo(&mut file);
        assert_eq!(height(&file, 70101), 99);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }
}
