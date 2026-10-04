//! The game's color bins as the compiler edits them (`team_compiler/pipeline.md` "Bins
//! accumulation"). Each holds one fixed-size record per team, in team ID order starting at team
//! 100, so team `id`'s record starts at `(id - 100) * record size`. A `TeamColor.bin` record is
//! 16 bytes: the `u16` team ID, the `u16` color count (4 in every record), then four colors of
//! three bytes each (R, G, B), the integers little-endian (`resources/bins/README.md`).

use anyhow::{Context, ensure};

use crate::templates;

/// One color as the bins hold it: red, green, blue.
pub(crate) type Rgb = [u8; 3];

/// The colors a kit's `colors.txt` gives: the kit's two menu colors.
pub(crate) const KIT_COLORS: usize = 2;

/// The colors the root `colors.txt` gives: a `TeamColor.bin` record holds four.
pub(crate) const TEAM_COLORS: usize = 4;

/// The team of each bin's first record.
const FIRST_TEAM: u16 = 100;

/// A `TeamColor.bin` record's header size: the team ID and the color count.
const TEAM_COLOR_HEADER: usize = 4;

/// A `TeamColor.bin` record's size, 16 bytes: its header, then `TEAM_COLORS` colors.
const TEAM_COLOR_RECORD: usize = TEAM_COLOR_HEADER + TEAM_COLORS * 3;

/// The color count every `TeamColor.bin` record's header gives.
const TEAM_COLOR_COUNT: u16 = 4;

/// A `TeamColor.bin` being edited.
pub(crate) struct TeamColorBin {
    /// The file's bytes, a whole number of records.
    bytes: Vec<u8>,
}

impl TeamColorBin {
    /// `bytes` as a bin; a length that is not a whole number of 16-byte records is an error,
    /// and so is a bin with more records than a `u16` has team IDs from 100.
    pub(crate) fn read(bytes: Vec<u8>) -> anyhow::Result<TeamColorBin> {
        ensure!(
            bytes.len().is_multiple_of(TEAM_COLOR_RECORD),
            "TeamColor.bin is {} bytes, not a whole number of {TEAM_COLOR_RECORD}-byte records",
            bytes.len()
        );
        let records = bytes.len() / TEAM_COLOR_RECORD;
        ensure!(
            records <= usize::from(u16::MAX - FIRST_TEAM) + 1,
            "TeamColor.bin holds {records} records, more than there are team IDs"
        );
        Ok(TeamColorBin { bytes })
    }

    /// Sets every record's header from its position (team ID `100 + index`, color count 4)
    /// and returns the IDs of the teams whose header was not that already, ascending.
    pub(crate) fn repair_headers(&mut self) -> Vec<u16> {
        let mut repaired = Vec::new();
        // `read` leaves no bytes past the last whole record.
        let (records, _) = self.bytes.as_chunks_mut::<TEAM_COLOR_RECORD>();
        for (index, record) in records.iter_mut().enumerate() {
            let team_id = u16::try_from(index)
                .ok()
                .and_then(|index| FIRST_TEAM.checked_add(index))
                .expect("`read` refuses a bin with more records than team IDs");
            let mut header = [0; TEAM_COLOR_HEADER];
            header[..2].copy_from_slice(&team_id.to_le_bytes());
            header[2..].copy_from_slice(&TEAM_COLOR_COUNT.to_le_bytes());
            if record[..TEAM_COLOR_HEADER] != header {
                record[..TEAM_COLOR_HEADER].copy_from_slice(&header);
                repaired.push(team_id);
            }
        }
        repaired
    }

    /// Writes `colors` over the first colors of `team_id`'s record; the colors it does not
    /// reach keep their bytes, and colors past the fourth have no slot and are not written. A
    /// team with no record is an error.
    pub(crate) fn set_colors(&mut self, team_id: u16, colors: &[Rgb]) -> anyhow::Result<()> {
        let start = team_id
            .checked_sub(FIRST_TEAM)
            .map(|index| usize::from(index) * TEAM_COLOR_RECORD);
        let record = start
            .and_then(|start| self.bytes.get_mut(start..start + TEAM_COLOR_RECORD))
            .with_context(|| format!("TeamColor.bin has no record for team {team_id}"))?;
        let (slots, _) = record[TEAM_COLOR_HEADER..].as_chunks_mut::<3>();
        for (slot, color) in slots.iter_mut().zip(colors) {
            *slot = *color;
        }
        Ok(())
    }

    /// The bin's bytes.
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// The bins a run builds on.
pub(crate) struct WorkingBins {
    /// `TeamColor.bin`.
    pub(crate) team_color: Vec<u8>,
}

impl WorkingBins {
    /// The bundled bases (`resources/bins/`), for a run with no installed bin to build on.
    pub(crate) fn bundled() -> WorkingBins {
        WorkingBins {
            team_color: templates::TEAM_COLOR.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two sound records, teams 100 and 101, each with its own four colors.
    fn two_records() -> Vec<u8> {
        let mut bytes = vec![0x64, 0x00, 0x04, 0x00];
        bytes.extend(1..=12);
        bytes.extend([0x65, 0x00, 0x04, 0x00]);
        bytes.extend(21..=32);
        bytes
    }

    #[test]
    fn a_bin_is_a_whole_number_of_records() {
        assert!(TeamColorBin::read(vec![0; 17]).is_err());
        assert!(TeamColorBin::read(vec![0; 32]).is_ok());
    }

    #[test]
    fn a_bin_holds_no_more_records_than_there_are_team_ids_from_100() {
        // Teams 100 to 65535: 65436 records.
        let most = 65_436 * TEAM_COLOR_RECORD;
        assert!(TeamColorBin::read(vec![0; most]).is_ok());
        let error = TeamColorBin::read(vec![0; most + TEAM_COLOR_RECORD])
            .err()
            .expect("one record too many");
        assert_eq!(
            error.to_string(),
            "TeamColor.bin holds 65437 records, more than there are team IDs"
        );
    }

    #[test]
    fn sound_headers_are_left_as_they_are() {
        let mut bin = TeamColorBin::read(two_records()).unwrap();
        assert_eq!(bin.repair_headers(), Vec::<u16>::new());
        assert_eq!(bin.into_bytes(), two_records());
    }

    #[test]
    fn a_header_overwritten_by_colors_is_set_from_its_position() {
        let mut bytes = two_records();
        bytes[16..20].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41]);
        let mut bin = TeamColorBin::read(bytes.clone()).unwrap();
        assert_eq!(bin.repair_headers(), [101]);
        let repaired = bin.into_bytes();
        assert_eq!(repaired[16..20], [0x65, 0x00, 0x04, 0x00]);
        assert_eq!(
            repaired[20..],
            bytes[20..],
            "the record's other bytes are kept"
        );
        assert_eq!(repaired[..16], bytes[..16], "the sound record is unchanged");
    }

    #[test]
    fn a_header_whose_count_is_not_four_is_repaired() {
        let mut bytes = two_records();
        bytes[2] = 3;
        let mut bin = TeamColorBin::read(bytes).unwrap();
        assert_eq!(bin.repair_headers(), [100]);
        assert_eq!(bin.into_bytes(), two_records());
    }

    #[test]
    fn two_colors_are_written_after_the_header_and_the_rest_of_the_record_is_kept() {
        let mut bin = TeamColorBin::read(two_records()).unwrap();
        bin.set_colors(101, &[[0xc1, 0x12, 0x00], [0x41, 0x41, 0x41]])
            .unwrap();
        let mut expected = two_records();
        expected[20..26].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
        assert_eq!(bin.into_bytes(), expected);
    }

    #[test]
    fn four_colors_fill_the_record() {
        let mut bin = TeamColorBin::read(two_records()).unwrap();
        bin.set_colors(100, &[[0xa1; 3], [0xa2; 3], [0xa3; 3], [0xa4; 3]])
            .unwrap();
        let bytes = bin.into_bytes();
        let mut record = vec![0x64, 0x00, 0x04, 0x00];
        for color in [0xa1, 0xa2, 0xa3, 0xa4] {
            record.extend([color; 3]);
        }
        assert_eq!(bytes[..16], record);
        assert_eq!(bytes[16..], two_records()[16..]);
    }

    #[test]
    fn a_team_with_no_record_is_an_error() {
        let mut bin = TeamColorBin::read(two_records()).unwrap();
        for team_id in [102, 99] {
            let error = bin.set_colors(team_id, &[[1, 2, 3]]).unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("TeamColor.bin has no record for team {team_id}")
            );
        }
        assert_eq!(bin.into_bytes(), two_records());
    }

    #[test]
    fn the_bundled_base_holds_teams_100_to_920_with_sound_headers() {
        let bins = WorkingBins::bundled();
        assert_eq!(bins.team_color.len(), 821 * TEAM_COLOR_RECORD);
        let mut bin = TeamColorBin::read(bins.team_color).unwrap();
        assert_eq!(bin.repair_headers(), Vec::<u16>::new());
    }
}
