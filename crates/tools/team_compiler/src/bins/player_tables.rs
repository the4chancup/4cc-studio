//! The Fox player tables (`team_compiler/pipeline.md` "Bins accumulation"): `BootsList.bin` and
//! `GloveList.bin`, which give a player his custom boots or gloves ID, and
//! `PlayerAppearance.bin`, every player's look. The game reads the highest-priority copy of
//! each table whole, so a compile writes the installed table with its compiled players' rows
//! changed, never a table of its own rows alone. None has a bundled base: a table no installed
//! CPK holds is not written.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::ensure;
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, tool_message};
use crate::paths;
use crate::plan::item_rows::{ItemRow, RowChange};

/// A `BootsList.bin` or `GloveList.bin` pair's size: the player id, then the item ID, each a
/// little-endian `u32`.
const PAIR: usize = 8;

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

    /// The table's file name, as findings name it.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ItemTable::Boots => "BootsList.bin",
            ItemTable::Gloves => "GloveList.bin",
        }
    }

    /// The table's path in a CPK.
    pub(crate) fn path(self) -> &'static str {
        match self {
            ItemTable::Boots => paths::BOOTS_LIST,
            ItemTable::Gloves => paths::GLOVE_LIST,
        }
    }
}

/// A `BootsList.bin` or `GloveList.bin` being edited: each player's item ID, by player id.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ItemList {
    /// The item ID of each player the table lists, by player id. A map, so a player id the
    /// installed table lists twice keeps its last pair (no table measured repeats one), and
    /// the pairs are written sorted by player id, as the game's own tables are.
    items: BTreeMap<u32, u32>,
}

impl ItemList {
    /// `bytes` as `table`'s (player id, item ID) pairs; a length that is not a whole number of
    /// 8-byte pairs is an error naming the table.
    pub(crate) fn read(table: ItemTable, bytes: &[u8]) -> anyhow::Result<ItemList> {
        ensure!(
            bytes.len().is_multiple_of(PAIR),
            "{} is {} bytes, not a whole number of {PAIR}-byte pairs",
            table.name(),
            bytes.len()
        );
        let (pairs, _) = bytes.as_chunks::<PAIR>();
        let items = pairs
            .iter()
            .map(|&[p0, p1, p2, p3, i0, i1, i2, i3]| {
                (
                    u32::from_le_bytes([p0, p1, p2, p3]),
                    u32::from_le_bytes([i0, i1, i2, i3]),
                )
            })
            .collect();
        Ok(ItemList { items })
    }

    /// Points `player_id` at `item_id`, replacing his pair or adding one.
    fn set(&mut self, player_id: u32, item_id: u32) {
        self.items.insert(player_id, item_id);
    }

    /// Removes `player_id`'s pair, when the table has one.
    fn remove(&mut self, player_id: u32) {
        self.items.remove(&player_id);
    }

    /// Applies `rows`' rows of `table`: each `Set` row whose task is among `committed` (the
    /// manifest positions of the batches the writer committed) is set, the others leaving the
    /// installed pair as it is, and each `Remove` row is removed.
    pub(crate) fn apply(
        &mut self,
        table: ItemTable,
        rows: &[ItemRow],
        committed: &BTreeSet<usize>,
    ) {
        for row in rows.iter().filter(|row| row.table == table) {
            match row.change {
                RowChange::Set { id, task } => {
                    if committed.contains(&task) {
                        self.set(row.player_id, id);
                    }
                }
                RowChange::Remove => self.remove(row.player_id),
            }
        }
    }

    /// The table's bytes: every pair, sorted by player id.
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.items
            .into_iter()
            .flat_map(|(player_id, item_id)| {
                player_id
                    .to_le_bytes()
                    .into_iter()
                    .chain(item_id.to_le_bytes())
            })
            .collect()
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
            ("table", table.name().to_owned()),
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
        ItemList::read(ItemTable::Boots, &bytes_of(pairs)).unwrap()
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
    fn a_list_is_a_whole_number_of_8_byte_pairs_and_the_error_names_it() {
        let error = ItemList::read(ItemTable::Boots, &[0; 12]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "BootsList.bin is 12 bytes, not a whole number of 8-byte pairs"
        );
        let error = ItemList::read(ItemTable::Gloves, &[0; 9]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "GloveList.bin is 9 bytes, not a whole number of 8-byte pairs"
        );
        assert_eq!(boots(&[]).into_bytes(), Vec::<u8>::new());
    }

    #[test]
    fn the_pairs_are_read_as_little_endian_player_id_then_item_id() {
        // Player 70201 with boots 11, then player 70205 with boots 625, each `u32` little-endian.
        let bytes = [
            0x39, 0x12, 0x01, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x3d, 0x12, 0x01, 0x00, 0x71, 0x02,
            0x00, 0x00,
        ];
        let list = ItemList::read(ItemTable::Boots, &bytes).unwrap();
        assert_eq!(list.items, BTreeMap::from([(70201, 11), (70205, 625)]));
        assert_eq!(list.into_bytes(), bytes);
    }

    #[test]
    fn set_replaces_a_player_s_pair_or_adds_one() {
        let mut list = boots(&[(70201, 11), (71405, 7)]);
        list.set(71405, 625);
        list.set(71406, 626);
        assert_eq!(
            list.into_bytes(),
            bytes_of(&[(70201, 11), (71405, 625), (71406, 626)])
        );
    }

    #[test]
    fn remove_takes_a_player_s_pair_out_and_leaves_a_list_without_one_as_it_is() {
        let mut list = boots(&[(70201, 11), (71405, 7)]);
        list.remove(71405);
        list.remove(71406);
        assert_eq!(list.into_bytes(), bytes_of(&[(70201, 11)]));
    }

    #[test]
    fn the_pairs_are_written_sorted_by_player_id_and_a_repeated_player_keeps_his_last() {
        let list = boots(&[(71405, 7), (70201, 11), (70707, 3), (71405, 9)]);
        assert_eq!(
            list.into_bytes(),
            bytes_of(&[(70201, 11), (70707, 3), (71405, 9)])
        );
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
        list.apply(ItemTable::Boots, &rows, &BTreeSet::from([1, 3]));
        assert_eq!(
            list.into_bytes(),
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
