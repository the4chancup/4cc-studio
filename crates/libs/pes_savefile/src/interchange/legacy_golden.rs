//! Parity of `interchange::legacy` with the reference editor's struct dumps.
//! The `.4ccs` record is `player_export` in MSVC layout; its offsets are held
//! here as literals (from the ctypes mirror in
//! `scripts/provenance/fixtures/interchange_fixtures.py`) and read raw from
//! the real fixture, then compared with what `read_squad` decodes. The
//! `.4cct` fixture is synthesized from the reference's write walk over the
//! PES 19 fixture save's team 713, so its evidence is agreement with the
//! schema codec's tactics for that team.

use pes_version::PesVersion;

use crate::interchange::legacy::{LegacyError, read_squad, read_tactics};
use crate::interchange::team_toml::TeamToml;
use crate::model::instruction::Instruction;
use crate::schema::playstyle;
use crate::test_support::open;

const SQUAD: &[u8] = include_bytes!("../../tests/fixtures/pes19_squad.4ccs");
const NIGHTLY: &[u8] = include_bytes!("../../tests/fixtures/pes19_tactics.4cct");

/// `"21a"`/`"20a"` + two version digits.
const HEADER: usize = 5;
/// `sizeof(player_export)`.
const RECORD: usize = 356;
/// `uint16_t numbers[40]`.
const NUMBERS: usize = 80;

// `player_export` field offsets (MSVC, 4-byte alignment, one padding byte at 271).
const NATION: usize = 0;
const HEIGHT: usize = 4;
const ATK: usize = 8;
const GK: usize = 10;
const B_EDIT_PLAYER: usize = 18;
const REG_POS: usize = 32;
const PLAY_STYLE: usize = 33;
const PLAY_SKILL: usize = 82;
const NAME: usize = 124;
const SHIRT_NAME: usize = 246;
const BOOT_ID: usize = 272;
const COPY_ID: usize = 280;
const NECK_LEN: usize = 284;
const SKIN_COL: usize = 353;
const IRIS_COL: usize = 354;

/// What the three sampled records hold (values printed by the fixture script).
struct Sample {
    index: usize,
    name: &'static str,
    shirt: &'static str,
    height: u8,
    atk: u8,
    gk: u8,
    reg_pos: u8,
    play_style: u8,
    skills: &'static [usize],
    boots: u32,
    copy: u32,
    skin: u8,
    iris: u8,
}

const SAMPLES: [Sample; 3] = [
    Sample {
        index: 0,
        name: "SHOTABOT 9S",
        shirt: "NINE ES",
        height: 189,
        atk: 77,
        gk: 77,
        reg_pos: 0,
        play_style: 20,
        skills: &[20],
        boots: 403,
        copy: 71301,
        skin: 1,
        iris: 1,
    },
    Sample {
        index: 7,
        name: "LONELY GAY NEET",
        shirt: "ALL \"BI\" MYSELF",
        height: 185,
        atk: 88,
        gk: 88,
        reg_pos: 5,
        play_style: 12,
        skills: &[7, 11, 12, 13, 16],
        boots: 0,
        copy: 71308,
        skin: 1,
        iris: 1,
    },
    Sample {
        index: 22,
        name: "wtf im gransexual now??",
        shirt: "(YOU)SEXUAL",
        height: 180,
        atk: 77,
        gk: 77,
        reg_pos: 5,
        play_style: 11,
        skills: &[12, 13],
        boots: 407,
        copy: 71323,
        skin: 2,
        iris: 3,
    },
];

fn raw(i: usize) -> &'static [u8] {
    &SQUAD[HEADER + i * RECORD..HEADER + (i + 1) * RECORD]
}

fn u32_at(rec: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(rec[at..at + 4].try_into().expect("four bytes"))
}

fn wide_string(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16(&units).expect("UTF-16")
}

fn c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[..end].to_vec()).expect("ASCII")
}

#[test]
fn the_struct_dump_reads_at_the_literal_offsets() {
    assert_eq!(&SQUAD[..3], b"20a");
    assert_eq!(&SQUAD[3..5], b"19");
    assert_eq!(SQUAD.len(), HEADER + 23 * RECORD + NUMBERS, "no tactics block");
    for s in &SAMPLES {
        let rec = raw(s.index);
        assert_eq!(u32_at(rec, NATION), 231, "[{}] nation", s.index);
        assert_eq!(rec[HEIGHT], s.height, "[{}] height", s.index);
        assert_eq!(rec[ATK], s.atk, "[{}] atk", s.index);
        assert_eq!(rec[GK], s.gk, "[{}] gk", s.index);
        assert_eq!(rec[REG_POS], s.reg_pos, "[{}] reg_pos", s.index);
        assert_eq!(rec[PLAY_STYLE], s.play_style, "[{}] play_style", s.index);
        let skills: Vec<usize> = (0..41).filter(|&k| rec[PLAY_SKILL + k] != 0).collect();
        assert_eq!(skills, s.skills, "[{}] skills", s.index);
        assert_eq!(wide_string(&rec[NAME..NAME + 122]), s.name, "[{}] name", s.index);
        assert_eq!(c_string(&rec[SHIRT_NAME..SHIRT_NAME + 21]), s.shirt, "[{}] shirt", s.index);
        assert_eq!(u32_at(rec, BOOT_ID), s.boots, "[{}] boots", s.index);
        assert_eq!(u32_at(rec, COPY_ID), s.copy, "[{}] copy", s.index);
        assert_eq!(u32_at(rec, NECK_LEN), 7, "[{}] neck_len", s.index);
        assert_eq!(rec[SKIN_COL], s.skin, "[{}] skin", s.index);
        assert_eq!(rec[IRIS_COL], s.iris, "[{}] iris", s.index);
    }
    let numbers = &SQUAD[HEADER + 23 * RECORD..];
    for i in 0..40 {
        let n = u16::from_le_bytes([numbers[2 * i], numbers[2 * i + 1]]);
        assert_eq!(n, if i < 23 { i as u16 + 1 } else { 0 }, "number {i}");
    }
}

#[test]
fn read_squad_decodes_the_struct_dump() {
    let doc = read_squad(SQUAD).expect("a valid .4ccs");
    assert_eq!(doc.pes_version, Some(PesVersion::Pes19));
    assert_eq!(doc.players.len(), 23);
    assert_eq!(doc.players.keys().copied().collect::<Vec<u8>>(), (1..=23).collect::<Vec<u8>>());
    assert!(doc.tactics.starting_eleven.is_none(), "no tactics block in this file");
    assert!(doc.team.name.is_none(), "a .4ccs carries no team entry");
    for s in &SAMPLES {
        let p = &doc.players[&(s.index as u8 + 1)];
        let rec = raw(s.index);
        assert_eq!(p.name.as_deref(), Some(s.name));
        assert_eq!(p.shirt_name.as_deref(), Some(s.shirt));
        assert_eq!(p.number, Some(s.index as u16 + 1));
        assert_eq!(p.nationality, Some(231));
        assert_eq!(p.stats.attacking_prowess, Some(s.atk));
        assert_eq!(p.stats.goalkeeping, Some(s.gk));
        assert_eq!(p.positions.registered, Some(s.reg_pos));
        assert_eq!(
            p.positions.playing_style,
            Some(playstyle::decode(PesVersion::Pes19, s.play_style).expect("listed"))
        );
        let mut skills = [false; 41];
        for &k in s.skills {
            skills[k] = true;
        }
        assert_eq!(p.skills.skills, Some(skills));
        assert_eq!(p.edit_flags.player, Some(rec[B_EDIT_PLAYER] != 0));
        assert_eq!(p.boots_id, Some(s.boots));
        assert_eq!(p.base_copy_id, Some(s.copy));
        assert_eq!(p.appearance.skin_color, Some(s.skin));
        assert_eq!(p.appearance.iris_color, Some(s.iris));
        assert_eq!(p.appearance.physique.neck_length, Some(7));
        assert_eq!(p.appearance.physique.height, Some(s.height));
        assert!(p.ingame_face.is_none(), "the legacy record has no ingame-face run");
        // Fields PES 19 lacks are absent whatever the bytes say.
        assert!(p.stats.tight_possession.is_none());
        assert!(p.positions.stronger_hand.is_none());
    }
}

#[test]
fn read_tactics_equals_the_codec_for_the_same_team() {
    let doc = read_tactics(NIGHTLY).expect("a valid .4cct");
    assert_eq!(doc.pes_version, Some(PesVersion::Pes19));
    assert_eq!(doc.team.id, Some(713));
    assert!(doc.players.is_empty());

    let (file, _) = open(PesVersion::Pes19);
    let team = file.team(713).expect("team 713 is in the fixture save");
    let mut expected = TeamToml::from_team(PesVersion::Pes19, team, &[]).expect("dumps");
    // The nightly block has no captain.
    expected.tactics.set_pieces.captain = None;
    assert_eq!(doc.tactics, expected.tactics);

    // Literals from the fixture script, so the equality above is not vacuous.
    assert_eq!(doc.tactics.starting_eleven, Some([0, 3, 2, 4, 1, 5, 6, 7, 8, 9, 10]));
    assert_eq!(doc.tactics.set_pieces.penalty, Some(10));
    assert_eq!(doc.tactics.set_pieces.captain, None);
    let preset = &doc.tactics.presets[2];
    let instructions = preset.instructions.as_ref().expect("PES 19 has instructions");
    assert_eq!(instructions.defence[0].instruction, Instruction::SwarmTheBox);
    assert_eq!(instructions.defence[1].instruction, Instruction::Gegenpress);
    assert_eq!(instructions.attack[0].instruction, Instruction::Off);
    assert_eq!(preset.sliders.support_range, Some(3));
    assert_eq!(preset.style.buildup, Some(false));
    assert_eq!(preset.style.attacking_zone, Some(true));
}

#[test]
fn a_wrong_tag_or_version_is_refused() {
    let err = read_squad(NIGHTLY).expect_err("a .4cct is not a .4ccs");
    assert!(matches!(err, LegacyError::BadTag { .. }), "{err:?}");
    let err = read_tactics(SQUAD).expect_err("a .4ccs is not a .4cct");
    assert!(matches!(err, LegacyError::BadTag { .. }), "{err:?}");
    let mut bad = SQUAD.to_vec();
    bad[3..5].copy_from_slice(b"1x");
    let err = read_squad(&bad).expect_err("version digits");
    assert!(matches!(err, LegacyError::BadVersion(_)), "{err:?}");
    let err = read_squad(&SQUAD[..HEADER + RECORD]).expect_err("truncated");
    assert!(matches!(err, LegacyError::Truncated { .. }), "{err:?}");
}
