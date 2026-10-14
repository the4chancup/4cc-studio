//! The three Fox database tables that dress a player, as `4cc_08_bins.cpk`
//! holds them: `PlayerAppearance.bin` (60-byte rows, the player id then the
//! 56 appearance bytes his record keeps after the appearance block's
//! base-copy id), `BootsList.bin` and `GloveList.bin` ((player id, item ID)
//! `u32` pairs). Sorted by player id with each id once is what every game
//! and cup copy of these tables measured holds (the twelve
//! `PlayerAppearance.bin` files of PES 17 to 20 and the cup's `4cc_01`),
//! which is why `to_bytes` sorts. The Team compiler's bins step and the
//! Save editor's strip and seed edit the same tables.

use std::collections::BTreeMap;

/// One of the three Fox player tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerTableFile {
    /// `PlayerAppearance.bin`: (player id, 56 appearance bytes) rows.
    PlayerAppearance,
    /// `BootsList.bin`: (player id, boots ID) pairs.
    BootsList,
    /// `GloveList.bin`: (player id, gloves ID) pairs.
    GloveList,
}

impl PlayerTableFile {
    /// The table's file name, as findings and errors name it.
    pub fn file_name(self) -> &'static str {
        match self {
            PlayerTableFile::PlayerAppearance => "PlayerAppearance.bin",
            PlayerTableFile::BootsList => "BootsList.bin",
            PlayerTableFile::GloveList => "GloveList.bin",
        }
    }

    /// The table's path in a CPK (no leading `/`).
    pub fn cpk_path(self) -> &'static str {
        match self {
            PlayerTableFile::PlayerAppearance => {
                "common/character0/model/character/appearance/PlayerAppearance.bin"
            }
            PlayerTableFile::BootsList => "common/character0/model/character/boots/BootsList.bin",
            PlayerTableFile::GloveList => "common/character0/model/character/glove/GloveList.bin",
        }
    }

    /// What the table calls a row in the read error: the item lists hold
    /// (player id, item ID) pairs, the appearance table rows.
    fn unit(self) -> &'static str {
        match self {
            PlayerTableFile::PlayerAppearance => "rows",
            PlayerTableFile::BootsList | PlayerTableFile::GloveList => "pairs",
        }
    }
}

/// `PlayerTable::read` could not read the bytes as the table.
#[derive(Debug, thiserror::Error)]
pub enum PlayerTableError {
    /// The bytes are not a whole number of the table's rows.
    #[error("{file} is {len} bytes, not a whole number of {row}-byte {unit}")]
    Length {
        /// The table's file name.
        file: &'static str,
        /// The bytes' length.
        len: usize,
        /// One row's length: the player id, then `VALUE` bytes.
        row: usize,
        /// What the table calls a row ("pairs" for the item lists, "rows"
        /// for `PlayerAppearance.bin`).
        unit: &'static str,
    },
}

/// One table: per player id, the `VALUE` bytes after the id (56 for
/// `PlayerAppearance.bin`, 4 for the boots and gloves lists, the item ID
/// little-endian), written sorted by id.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PlayerTable<const VALUE: usize> {
    /// The row of each player the table lists, by player id. A map, so a
    /// player id the file lists twice keeps its last row (no table measured
    /// repeats one), and the rows are written sorted by player id, as the
    /// games' and cups' copies of these tables are.
    rows: BTreeMap<u32, [u8; VALUE]>,
}

/// `PlayerAppearance.bin`: (player id, 56 appearance bytes) rows.
pub type AppearanceTable = PlayerTable<56>;
/// `BootsList.bin` or `GloveList.bin`: (player id, item ID) pairs.
pub type ItemList = PlayerTable<4>;

impl<const VALUE: usize> PlayerTable<VALUE> {
    /// An empty table: a table the CPK lacks.
    pub fn new() -> Self {
        PlayerTable {
            rows: BTreeMap::new(),
        }
    }

    /// `bytes` as `file`'s (player id, `VALUE` bytes) rows; a length that is
    /// not a whole number of `4 + VALUE`-byte rows is `Length`, naming the
    /// file.
    pub fn read(file: PlayerTableFile, bytes: &[u8]) -> Result<Self, PlayerTableError> {
        let row = 4 + VALUE;
        if !bytes.len().is_multiple_of(row) {
            return Err(PlayerTableError::Length {
                file: file.file_name(),
                len: bytes.len(),
                row,
                unit: file.unit(),
            });
        }
        let rows = bytes
            .chunks_exact(row)
            .map(|chunk| {
                (
                    u32::from_le_bytes(chunk[..4].try_into().expect("four bytes")),
                    chunk[4..].try_into().expect("VALUE bytes"),
                )
            })
            .collect();
        Ok(PlayerTable { rows })
    }

    /// Sets `player_id`'s row to `value`, replacing it or adding one.
    pub fn set(&mut self, player_id: u32, value: [u8; VALUE]) {
        self.rows.insert(player_id, value);
    }

    /// Removes `player_id`'s row, when the table has one.
    pub fn remove(&mut self, player_id: u32) {
        self.rows.remove(&player_id);
    }

    /// `player_id`'s row, when the table has one.
    pub fn get(&self, player_id: u32) -> Option<&[u8; VALUE]> {
        self.rows.get(&player_id)
    }

    /// The table's bytes: every row, sorted by player id.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.rows
            .iter()
            .flat_map(|(player_id, value)| player_id.to_le_bytes().into_iter().chain(*value))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `rows` (player id, value) as a table holds them, in the order given.
    fn bytes_of<const VALUE: usize>(rows: &[(u32, [u8; VALUE])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (player_id, value) in rows {
            bytes.extend(player_id.to_le_bytes());
            bytes.extend(*value);
        }
        bytes
    }

    #[test]
    fn a_sorted_table_round_trips_byte_for_byte() {
        for (file, bytes) in [
            (
                PlayerTableFile::BootsList,
                bytes_of(&[(70201, [11, 0, 0, 0]), (71405, [0x71, 0x02, 0x00, 0x00])]),
            ),
            (
                PlayerTableFile::PlayerAppearance,
                bytes_of(&[(70201, [7; 56]), (71405, [9; 56])]),
            ),
        ] {
            match file {
                PlayerTableFile::BootsList | PlayerTableFile::GloveList => {
                    let table = ItemList::read(file, &bytes).expect("reads");
                    assert_eq!(table.to_bytes(), bytes, "{}", file.file_name());
                }
                PlayerTableFile::PlayerAppearance => {
                    let table = AppearanceTable::read(file, &bytes).expect("reads");
                    assert_eq!(table.to_bytes(), bytes, "{}", file.file_name());
                }
            }
        }
    }

    #[test]
    fn an_unsorted_table_is_written_sorted_by_player_id() {
        let table = ItemList::read(
            PlayerTableFile::BootsList,
            &bytes_of(&[(71405, [7; 4]), (70201, [11; 4]), (70707, [3; 4])]),
        )
        .expect("reads");
        assert_eq!(
            table.to_bytes(),
            bytes_of(&[(70201, [11; 4]), (70707, [3; 4]), (71405, [7; 4])])
        );
    }

    #[test]
    fn a_repeated_player_id_keeps_his_last_row() {
        let table = ItemList::read(
            PlayerTableFile::BootsList,
            &bytes_of(&[(71405, [7; 4]), (70201, [11; 4]), (71405, [9; 4])]),
        )
        .expect("reads");
        assert_eq!(
            table.to_bytes(),
            bytes_of(&[(70201, [11; 4]), (71405, [9; 4])])
        );
    }

    #[test]
    fn set_replaces_a_players_row_or_adds_one_remove_removes_it_get_reads_it() {
        let mut table = ItemList::read(
            PlayerTableFile::BootsList,
            &bytes_of(&[(70201, [11; 4]), (71405, [7; 4])]),
        )
        .expect("reads");
        table.set(71405, [9; 4]);
        table.set(71406, [8; 4]);
        table.remove(70201);
        table.remove(71407);
        assert_eq!(table.get(71405), Some(&[9; 4]));
        assert_eq!(table.get(70201), None);
        assert_eq!(
            table.to_bytes(),
            bytes_of(&[(71405, [9; 4]), (71406, [8; 4])])
        );
        assert_eq!(ItemList::new().to_bytes(), Vec::<u8>::new());
    }

    #[test]
    fn a_length_that_is_not_whole_rows_is_an_error_naming_the_file() {
        let err = AppearanceTable::read(PlayerTableFile::PlayerAppearance, &[0; 61])
            .expect_err("61 bytes is not whole rows");
        assert_eq!(
            err.to_string(),
            "PlayerAppearance.bin is 61 bytes, not a whole number of 60-byte rows"
        );
        let err = ItemList::read(PlayerTableFile::BootsList, &[0; 9])
            .expect_err("9 bytes is not whole pairs");
        assert_eq!(
            err.to_string(),
            "BootsList.bin is 9 bytes, not a whole number of 8-byte pairs"
        );
        let err = ItemList::read(PlayerTableFile::GloveList, &[0; 12])
            .expect_err("12 bytes is not whole pairs");
        assert_eq!(
            err.to_string(),
            "GloveList.bin is 12 bytes, not a whole number of 8-byte pairs"
        );
    }

    #[test]
    fn each_files_cpk_path_is_the_tables_game_path() {
        assert_eq!(
            PlayerTableFile::PlayerAppearance.cpk_path(),
            "common/character0/model/character/appearance/PlayerAppearance.bin"
        );
        assert_eq!(
            PlayerTableFile::BootsList.cpk_path(),
            "common/character0/model/character/boots/BootsList.bin"
        );
        assert_eq!(
            PlayerTableFile::GloveList.cpk_path(),
            "common/character0/model/character/glove/GloveList.bin"
        );
    }
}
