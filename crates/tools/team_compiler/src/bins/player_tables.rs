//! The Fox player tables (`spec/team_compiler/pipeline.md` "Bins accumulation"): `BootsList.bin` and
//! `GloveList.bin`, which give a player his custom boots or gloves ID, and
//! `PlayerAppearance.bin`, every player's look. The game reads the highest-priority copy of
//! each table whole, so a compile writes the installed table with its compiled players' rows
//! changed, never a table of its own rows alone. None has a bundled base: a table no installed
//! CPK holds is not written.
//!
//! The tables' codec is `pes_savefile::player_tables`; what is the tool's
//! stays here: applying a plan's `ItemRow`s and the `player_table_missing`
//! finding.

use std::collections::BTreeSet;

use anyhow::ensure;
use pes_savefile::player_tables::{ItemList, PlayerTableFile};
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, tool_message};
use crate::plan::item_rows::{ItemRow, RowChange};

/// A `PlayerAppearance.bin` row's size: the player id, then 56 appearance bytes.
const APPEARANCE_ROW: usize = 60;

/// One of the two tables pointing a player at his own boots or gloves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemTable {
    /// `BootsList.bin`: (player id, boots ID).
    Boots,
    /// `GloveList.bin`: (player id, gloves ID).
    Gloves,
}

impl ItemTable {
    /// Both tables, in the order the CPK holds them.
    pub(crate) const ALL: [ItemTable; 2] = [ItemTable::Boots, ItemTable::Gloves];

    /// The table's `PlayerTableFile`, the codec's name for it.
    pub(crate) fn file(self) -> PlayerTableFile {
        match self {
            ItemTable::Boots => PlayerTableFile::BootsList,
            ItemTable::Gloves => PlayerTableFile::GloveList,
        }
    }
}

/// Applies `rows`' rows of `table` to `list`: each `Set` row whose task is among `committed` (the
/// manifest positions of the batches the writer committed) is set, the others leaving the
/// installed pair as it is, and each `Remove` row is removed.
pub(crate) fn apply(
    list: &mut ItemList,
    table: ItemTable,
    rows: &[ItemRow],
    committed: &BTreeSet<usize>,
) {
    for row in rows.iter().filter(|row| row.table == table) {
        match row.change {
            RowChange::Set { id, task } => {
                if committed.contains(&task) {
                    list.set(row.player_id, id.to_le_bytes());
                }
            }
            RowChange::Remove => list.remove(row.player_id),
        }
    }
}

/// The `player_table_missing` finding for `table` when no installed CPK holds it: naming it
/// and counting `rows`' `Set` rows of it whose task is among `committed`, the rows the CPK
/// leaves out; none when there are no such rows (a `Remove` row has nothing to leave out).
pub(crate) fn table_missing(
    table: ItemTable,
    rows: &[ItemRow],
    committed: &BTreeSet<usize>,
) -> Option<Message> {
    let left_out = rows
        .iter()
        .filter(|row| row.table == table)
        .filter(|row| match row.change {
            RowChange::Set { task, .. } => committed.contains(&task),
            RowChange::Remove => false,
        })
        .count();
    if left_out == 0 {
        return None;
    }
    Some(tool_message(
        Code::PlayerTableMissing,
        Scope::Run,
        Disposition::Keep,
        vec![
            ("table", table.file().file_name().to_owned()),
            ("rows", left_out.to_string()),
        ],
    ))
}

/// `bytes` as a `PlayerAppearance.bin`, which passes through unchanged: a length that is not a
/// whole number of 60-byte rows is an error. No row is read.
pub(crate) fn read_player_appearance(bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    ensure!(
        bytes.len().is_multiple_of(APPEARANCE_ROW),
        "PlayerAppearance.bin is {} bytes, not a whole number of {APPEARANCE_ROW}-byte rows",
        bytes.len()
    );
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pairs `pairs` (player id, item ID) as a table holds them, in the order given.
    fn bytes_of(pairs: &[(u32, u32)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (player_id, item_id) in pairs {
            bytes.extend(player_id.to_le_bytes());
            bytes.extend(item_id.to_le_bytes());
        }
        bytes
    }

    /// The boots list holding `pairs`.
    fn boots(pairs: &[(u32, u32)]) -> ItemList {
        ItemList::read(ItemTable::Boots.file(), &bytes_of(pairs)).unwrap()
    }

    /// The row of `table` for `player_id` making `change`.
    fn row(table: ItemTable, player_id: u32, change: RowChange) -> ItemRow {
        ItemRow {
            table,
            player_id,
            change,
        }
    }

    #[test]
    fn a_player_appearance_bin_is_whole_60_byte_rows_kept_as_they_are() {
        let rows: Vec<u8> = (0..120).collect();
        assert_eq!(read_player_appearance(rows.clone()).unwrap(), rows);
        let error = read_player_appearance(vec![0; 61]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "PlayerAppearance.bin is 61 bytes, not a whole number of 60-byte rows"
        );
    }

    #[test]
    fn a_committed_set_row_is_set_an_uncommitted_one_kept_and_a_remove_row_removed() {
        let mut list = boots(&[(70201, 11), (71405, 7), (71406, 9), (71407, 8)]);
        let rows = [
            row(ItemTable::Boots, 71405, RowChange::Set { id: 625, task: 1 }),
            // Its task failed: the installed pair stays.
            row(ItemTable::Boots, 71407, RowChange::Set { id: 627, task: 2 }),
            row(ItemTable::Boots, 71406, RowChange::Remove),
            row(ItemTable::Boots, 71408, RowChange::Set { id: 628, task: 3 }),
            // Another table's rows change nothing here.
            row(ItemTable::Gloves, 70201, RowChange::Remove),
            row(
                ItemTable::Gloves,
                71409,
                RowChange::Set { id: 629, task: 1 },
            ),
        ];
        apply(&mut list, ItemTable::Boots, &rows, &BTreeSet::from([1, 3]));
        assert_eq!(
            list.to_bytes(),
            bytes_of(&[(70201, 11), (71405, 625), (71407, 8), (71408, 628)])
        );
    }

    #[test]
    fn a_missing_table_counts_its_committed_set_rows_and_says_nothing_without_one() {
        let rows = [
            row(ItemTable::Boots, 71405, RowChange::Set { id: 625, task: 1 }),
            row(ItemTable::Boots, 71407, RowChange::Set { id: 627, task: 2 }),
            row(ItemTable::Boots, 71408, RowChange::Set { id: 628, task: 3 }),
            row(ItemTable::Boots, 71406, RowChange::Remove),
            row(ItemTable::Gloves, 71406, RowChange::Remove),
        ];
        let committed = BTreeSet::from([1, 3]);
        assert_eq!(
            table_missing(ItemTable::Boots, &rows, &committed),
            Some(tool_message(
                Code::PlayerTableMissing,
                Scope::Run,
                Disposition::Keep,
                vec![
                    ("table", "BootsList.bin".to_owned()),
                    ("rows", "2".to_owned())
                ],
            ))
        );
        assert_eq!(
            table_missing(ItemTable::Boots, &rows, &BTreeSet::new()),
            None,
            "no Set row committed"
        );
        assert_eq!(
            table_missing(ItemTable::Gloves, &rows, &committed),
            None,
            "only a Remove row"
        );
        let message = table_missing(ItemTable::Boots, &rows, &committed).unwrap();
        assert_eq!(message.severity, studio_core::Severity::Warning);
    }
}
