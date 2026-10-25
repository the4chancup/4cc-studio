//! Fixture helpers shared by the crate's test modules: inflated payloads,
//! section records, and player lookup. The payload plumbing itself is
//! `fixtures`, shared with the `test-support` feature.

use pes_version::PesVersion;

use crate::codec::{read_player, read_player_into};
use crate::model::player::PlayerEntry;
use crate::schema::{SectionLayout, VersionSchema};

pub(crate) use crate::fixtures::{fixture_container, payload, test_salt};

/// The versions a committed payload fixture exists for.
pub(crate) const FIXTURES: [PesVersion; 7] = [
    PesVersion::Pes15,
    PesVersion::Pes16,
    PesVersion::Pes17,
    PesVersion::Pes18,
    PesVersion::Pes19,
    PesVersion::Pes20,
    PesVersion::Pes21,
];

/// The record count of a section.
pub(crate) fn count(payload: &[u8], section: &SectionLayout) -> usize {
    u16::from_le_bytes(
        payload[section.count_offset..section.count_offset + 2]
            .try_into()
            .expect("u16"),
    ) as usize
}

/// Record `i` of a section.
pub(crate) fn record<'a>(
    payload: &'a [u8],
    section: &SectionLayout,
    size: usize,
    i: usize,
) -> &'a [u8] {
    &payload[section.offset + i * size..section.offset + (i + 1) * size]
}

/// The PES 15/16 appearance record of a decoded player id.
pub(crate) fn appearance_for(payload: &[u8], schema: &VersionSchema, player: &mut PlayerEntry) {
    if let Some((section, appearance)) = &schema.appearance {
        for i in 0..count(payload, section) {
            let rec = record(payload, section, appearance.size, i);
            if u32::from_le_bytes(rec[..4].try_into().expect("id")) == player.id {
                read_player_into(player, rec, appearance).expect("appearance decodes");
                return;
            }
        }
        panic!("no appearance record for player {}", player.id);
    }
}

/// The decoded player with `id` (appearance fields included on 15/16).
pub(crate) fn find_player(payload: &[u8], schema: &VersionSchema, id: u32) -> PlayerEntry {
    for i in 0..count(payload, &schema.players) {
        let rec = record(payload, &schema.players, schema.player.size, i);
        let mut player = read_player(rec, schema.player).expect("decode");
        if player.id == id {
            appearance_for(payload, schema, &mut player);
            return player;
        }
    }
    panic!("player {id} not in the save");
}

/// `target` takes `donor`'s appearance block (`appearance`, ingame-face run
/// included, and the face/hair/physique/strip edit flags); everything else,
/// the id and the appearance id included, stays the target's own. The
/// reference scripts' slice (`ops/appearance_golden.rs` pins it), and the
/// yardstick the comparator's aesthetics scope is tested against.
pub(crate) fn copy_appearance(target: &mut PlayerEntry, donor: &PlayerEntry) {
    target.appearance = donor.appearance.clone();
    target.edit_flags.face = donor.edit_flags.face;
    target.edit_flags.hair = donor.edit_flags.hair;
    target.edit_flags.physique = donor.edit_flags.physique;
    target.edit_flags.strip = donor.edit_flags.strip;
}

/// A fixture as an `EditFile`, plus the container bytes it decoded from.
pub(crate) fn open(version: PesVersion) -> (crate::file::EditFile, Vec<u8>) {
    let container = fixture_container(version);
    let bytes = container.to_bytes(&test_salt()).expect("container encodes");
    let file = crate::file::EditFile::from_bytes(&bytes).expect("the save opens");
    (file, bytes)
}
