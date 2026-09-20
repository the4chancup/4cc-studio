//! Parity of `interchange::texport` with the reference editor's Texport read
//! walk, held as the literals measured on the real files (four PES 18, eleven
//! 19 and five 21 texports; the fixtures are one of each, whole): the key
//! index, where the tactics and player records start, the roster width, and
//! what the records say. The 15-17 half opens through the container: the
//! PES 17 `TEXPORT00000000`'s payload is the fixture, sliced as a save's is.

use std::io::Read;

use pes_version::PesVersion;

use crate::container::{MasterKey, SaveContainer, Scheme};
use crate::interchange::texport::{Texport, TexportError};
use crate::model::names::display_name;
use crate::test_support::test_salt;

/// One measured file.
struct Layout {
    version: PesVersion,
    bytes: &'static [u8],
    /// First key byte the reference XORs with.
    key_index: usize,
    /// The reference's `tacticsPos`.
    tactics_at: usize,
    /// The reference's `playersPos`.
    players_at: usize,
    /// Player record slots the file has room for (the roster width).
    width: usize,
    team: u32,
    team_name: &'static str,
    short_name: &'static str,
    /// Players actually present (roster slots filled).
    filled: usize,
    first_player: &'static str,
    first_boots: u32,
}

const LAYOUTS: [Layout; 3] = [
    Layout {
        version: PesVersion::Pes18,
        bytes: include_bytes!("../../tests/fixtures/pes18_texport.ted"),
        key_index: 0x12,
        tactics_at: 0x32C,
        players_at: 0x834,
        width: 32,
        team: 736,
        team_name: "/o/",
        short_name: "O",
        filled: 23,
        first_player: "BLACK ICE TREE",
        first_boots: 38,
    },
    Layout {
        version: PesVersion::Pes19,
        bytes: include_bytes!("../../tests/fixtures/pes19_texport.ted"),
        key_index: 0x13,
        tactics_at: 0x33C,
        players_at: 0x844,
        width: 40,
        team: 841,
        team_name: "/98hu/",
        short_name: "I1",
        filled: 23,
        first_player: "YOUR SKILL: A FAILURE",
        first_boots: 3601,
    },
    Layout {
        version: PesVersion::Pes21,
        bytes: include_bytes!("../../tests/fixtures/pes21_texport.ted"),
        key_index: 0x15,
        tactics_at: 0x410,
        players_at: 0x918,
        width: 40,
        team: 841,
        team_name: "/98hu/",
        short_name: "2HU",
        filled: 23,
        first_player: "YOUR SKILL: A FAILURE",
        first_boots: 3601,
    },
];

/// The reference's loop: the body from 0x50 XOR'd with the 32-byte key at
/// 0x30, starting at `key_index` and wrapping.
fn reference_decrypt(bytes: &[u8], key_index: usize) -> Vec<u8> {
    let key = &bytes[0x30..0x50];
    let mut out = bytes.to_vec();
    let mut k = key_index;
    for b in out.iter_mut().skip(0x50) {
        *b ^= key[k];
        k = (k + 1) % 0x20;
    }
    out
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes"))
}

#[test]
fn the_reference_walk_finds_the_records_at_the_literal_offsets() {
    for l in &LAYOUTS {
        let dec = reference_decrypt(l.bytes, l.key_index);
        assert_eq!(u32_at(&dec, l.tactics_at), l.team, "{:?} tactics id", l.version);
        let stride = (dec.len() - 12 - l.players_at) / l.width;
        assert_eq!(l.players_at + l.width * stride + 12, dec.len(), "{:?} width", l.version);
        let mut ids: Vec<u32> = (0..l.width)
            .map(|i| u32_at(&dec, l.players_at + i * stride))
            .collect();
        let empty = ids.split_off(l.filled);
        assert!(empty.iter().all(|&id| id == 0), "{:?} empty slots", l.version);
        let expected: Vec<u32> = (0..l.filled as u32).map(|i| l.team * 100 + 1 + i).collect();
        assert_eq!(ids, expected, "{:?} player ids", l.version);
    }
}

#[test]
fn from_bytes_decodes_what_the_reference_reads_and_detects_the_version() {
    for l in &LAYOUTS {
        let t = Texport::from_bytes(l.bytes, None).expect("opens");
        assert_eq!(t.version(), l.version);
        let team = t.team();
        assert_eq!(team.id, l.team);
        assert_eq!(display_name(&team.name), l.team_name);
        assert_eq!(team.short_name, l.short_name);
        assert_eq!(
            team.roster.iter().filter(|s| s.player_id != 0).count(),
            l.filled,
            "{:?} roster",
            l.version
        );
        let players = t.players();
        assert_eq!(players.len(), l.filled);
        for (i, p) in players.iter().enumerate() {
            assert_eq!(p.id, l.team * 100 + 1 + i as u32, "{:?} slot {i}", l.version);
            assert_eq!(p.id, team.roster[i].player_id, "{:?} roster slot {i}", l.version);
        }
        assert_eq!(display_name(&players[0].name), l.first_player);
        assert_eq!(players[0].appearance.boots_id, l.first_boots);
        // Every stat of every player is a real ability, not a misaligned read.
        for p in players {
            for v in [
                p.stats.attacking_prowess,
                p.stats.ball_control,
                p.stats.speed,
                p.stats.stamina,
                p.stats.goalkeeping,
            ] {
                assert!((40..=99).contains(&v), "{:?} player {} stat {v}", l.version, p.id);
            }
        }
        let explicit = Texport::from_bytes(l.bytes, Some(l.version)).expect("opens");
        assert_eq!(explicit.team(), t.team());
        assert_eq!(explicit.players(), t.players());
    }
}

#[test]
fn a_file_of_another_size_is_refused_for_the_named_version() {
    let err = Texport::from_bytes(LAYOUTS[1].bytes, Some(PesVersion::Pes18)).expect_err("refused");
    assert!(
        matches!(err, TexportError::WrongSize { version: PesVersion::Pes18, .. }),
        "{err:?}"
    );
    let err = Texport::from_bytes(&LAYOUTS[0].bytes[..0x50], None).expect_err("refused");
    assert!(matches!(err, TexportError::UnknownVersion), "{err:?}");
}

#[test]
fn round_trip_is_byte_identical_on_every_fixture() {
    for l in &LAYOUTS {
        let t = Texport::from_bytes(l.bytes, None).expect("opens");
        let out = t.to_bytes(&test_salt()).expect("encodes");
        assert_eq!(out, l.bytes, "{:?} round trip", l.version);
    }
}

/// The PES 17 texport as a container around the sliced payload (logo and
/// serial dropped, as the save fixtures are).
fn pes17_texport_bytes() -> Vec<u8> {
    let mut payload = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../../tests/fixtures/pes17_texport_payload.bin.zz")[..],
    )
    .read_to_end(&mut payload)
    .expect("inflates");
    let mut description = b"Team Export Data 03".to_vec();
    description.resize(384, 0);
    SaveContainer {
        scheme: Scheme::Keyed(MasterKey::Pes17),
        description,
        logo: Vec::new(),
        payload,
        identifier: vec![0; MasterKey::Pes17.header_size() - 80],
        serial: Vec::new(),
    }
    .to_bytes(&test_salt())
    .expect("container encodes")
}

#[test]
fn pes17_texport_opens_through_the_container_and_round_trips() {
    let bytes = pes17_texport_bytes();
    let t = Texport::from_bytes(&bytes, None).expect("opens");
    assert_eq!(t.version(), PesVersion::Pes17);
    // The reference reads the tactics record at 0x10330 and the players at 0x510840.
    assert_eq!(t.team().id, 736);
    assert_eq!(t.team().tactics.starting_eleven.len(), 11);
    let players = t.players();
    assert_eq!(players.len(), 23);
    assert_eq!(display_name(&players[0].name), "BLACK ICE TREE");
    assert_eq!(display_name(&players[1].name), "JAMES MAY");
    for (i, p) in players.iter().enumerate() {
        assert_eq!(p.id, 736 * 100 + 1 + i as u32);
    }
    assert_eq!(t.to_bytes(&test_salt()).expect("encodes"), bytes);
}
