//! Fixture helpers shared by the crate's test modules: inflated payloads,
//! section records, and player lookup.

use std::io::Read;

use pes_version::PesVersion;

use crate::codec::{read_player, read_player_into};
use crate::model::player::PlayerEntry;
use crate::schema::{SectionLayout, VersionSchema};

/// The inflated payload of one version's fixture.
pub(crate) fn payload(version: PesVersion) -> Vec<u8> {
    let bytes: &[u8] = match version {
        PesVersion::Pes15 => include_bytes!("../tests/fixtures/pes15_payload.bin.zz"),
        PesVersion::Pes16 => include_bytes!("../tests/fixtures/pes16_payload.bin.zz"),
        PesVersion::Pes17 => include_bytes!("../tests/fixtures/pes17_payload.bin.zz"),
        PesVersion::Pes18 => include_bytes!("../tests/fixtures/pes18_payload.bin.zz"),
        PesVersion::Pes19 => include_bytes!("../tests/fixtures/pes19_payload.bin.zz"),
        PesVersion::Pes20 => include_bytes!("../tests/fixtures/pes20_payload.bin.zz"),
        PesVersion::Pes21 => include_bytes!("../tests/fixtures/pes21_payload.bin.zz"),
    };
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut out)
        .expect("fixture payload inflates");
    out
}

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

/// The master key matching a fixture version.
fn key(version: PesVersion) -> crate::container::MasterKey {
    use crate::container::MasterKey;
    match version {
        PesVersion::Pes15 => panic!("PES 15 is not keyed"),
        PesVersion::Pes16 => MasterKey::Pes16,
        PesVersion::Pes17 => MasterKey::Pes17,
        PesVersion::Pes18 => MasterKey::Pes18,
        PesVersion::Pes19 => MasterKey::Pes19,
        PesVersion::Pes20 => MasterKey::Pes20,
        PesVersion::Pes21 => MasterKey::Pes21,
    }
}

/// A `SaveContainer` around a fixture's inflated payload.
pub(crate) fn fixture_container(version: PesVersion) -> crate::container::SaveContainer {
    use crate::container::{SaveContainer, Scheme};
    let mut description = b"Edit Data".to_vec();
    description.resize(384, 0);
    match version {
        PesVersion::Pes15 => SaveContainer {
            scheme: Scheme::Pes15,
            description,
            logo: Vec::new(),
            payload: payload(version),
            identifier: Vec::new(),
            serial: Vec::new(),
        },
        _ => {
            let key = key(version);
            SaveContainer {
                scheme: Scheme::Keyed(key),
                description,
                logo: Vec::new(),
                payload: payload(version),
                identifier: vec![0; key.header_size() - 80],
                serial: Vec::new(),
            }
        }
    }
}

/// The salt the tests write with.
pub(crate) fn test_salt() -> [u8; 320] {
    std::array::from_fn(|i| (i % 251) as u8 + 1)
}

/// A fixture as an `EditFile`, plus the container bytes it decoded from.
pub(crate) fn open(version: PesVersion) -> (crate::file::EditFile, Vec<u8>) {
    let container = fixture_container(version);
    let bytes = container.to_bytes(&test_salt()).expect("container encodes");
    let file = crate::file::EditFile::from_bytes(&bytes).expect("the save opens");
    (file, bytes)
}
