//! The Fox `PlayerAppearance.bin` row: the 60-byte database row a Fox game
//! dresses a stripped player (appearance id -1) with — the player id, then
//! the 56 appearance bytes of his player record starting just after the
//! appearance block's base-copy id — built from a `PlayerSettings`.

use pes_version::PesVersion;

use super::{PlayerSettings, SettingsError};

use crate::codec::{read_player, write_player};
use crate::schema::schema_for;

/// The row's 56 appearance bytes' offset inside the version's player record:
/// right where the appearance block's base-copy id ends (record bytes 128 to
/// 184 on PES 18/19, 252 to 308 on PES 20/21; `model.md` "The Fox database
/// row": the games' own tables hold the same bytes at the same place in the
/// same 72-byte block). `None` on a pre-Fox version: its appearance travels
/// in the savefile (PES 17's own table reads a row at this place in this
/// block, but its compile writes the savefile instead).
fn row_offset(version: PesVersion) -> Option<usize> {
    match version {
        PesVersion::Pes18 | PesVersion::Pes19 => Some(128),
        PesVersion::Pes20 | PesVersion::Pes21 => Some(252),
        PesVersion::Pes15 | PesVersion::Pes16 | PesVersion::Pes17 => None,
    }
}

impl PlayerSettings {
    /// The player's `PlayerAppearance.bin` row for a Fox version: his id
    /// little-endian, then the 56 appearance bytes his player record holds
    /// just after the appearance block's base-copy id. The row is what
    /// `apply`'s appearance write puts onto a record of all zero bytes, so
    /// absent keys land at their `KeySpec` defaults and the run's
    /// measured-unused bits at the games' values. None of `apply`'s TOML-surface checks is the
    /// row's: the names are never read (an overlong one is no error here),
    /// and a stored value past a key's kind range but inside its field's
    /// bits — a save's own data, which strip-and-seed must reproduce — is
    /// written as is. A pre-Fox version has no row (`NoAppearanceRow`).
    pub fn appearance_row(
        &self,
        player_id: u32,
        version: PesVersion,
    ) -> Result<[u8; 60], SettingsError> {
        let offset = row_offset(version).ok_or(SettingsError::NoAppearanceRow { version })?;
        let schema = schema_for(version);
        let mut record = vec![0u8; schema.player.size];
        let mut player = read_player(&record, schema.player)?;
        self.write_appearance_defaults(&mut player, version)?;
        write_player(&player, &mut record, schema.player)?;
        let mut row = [0u8; 60];
        row[..4].copy_from_slice(&player_id.to_le_bytes());
        row[4..].copy_from_slice(&record[offset..offset + 56]);
        Ok(row)
    }
}
