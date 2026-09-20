//! The legacy 4ccEditor formats, read-only: `.4ccs` squad dumps and `.4cct`
//! "nightly" tactics files. Both decode into a [`TeamToml`] so that
//! [`TeamToml::apply`] is the only code that writes interchange data into a
//! save.
//!
//! The `.4ccs` record is the MSVC layout of the reference editor's
//! `player_export` struct — a foreign tool's in-memory layout, not a save
//! record, so its offsets live here as `const`s and not in `schema/`. They are
//! mirrored from the ctypes table in
//! `scripts/provenance/fixtures/interchange_fixtures.py`; the field names in
//! the comments are that struct's.

use std::collections::HashSet;

use pes_version::PesVersion;

use crate::codec::CodecError;
use crate::interchange::team_toml::player_keys::PlayerKey;
use crate::interchange::team_toml::{
    AutoSection, InstructionsSection, PlayerSection, SetPiecesSection, SlidersSection,
    StyleSection, TacticsSection, TeamToml, TeamTomlError,
};
use crate::model::instruction::Instruction;
use crate::model::tactics::FormationSlot;
use crate::schema::fields::PlayerField;
use crate::schema::{playstyle, schema_for};
use crate::settings_toml::keys::{SettingKey, Source};
use crate::settings_toml::set_appearance;

/// What a legacy file cannot be read as.
#[derive(Debug, thiserror::Error)]
pub enum LegacyError {
    /// The file's tag is not the format's.
    #[error("expected tag {expected:?}, found {found:?}")]
    BadTag {
        /// The tag the format carries.
        expected: &'static str,
        /// What the file starts with.
        found: String,
    },
    /// The two version digits are not a supported PES version.
    #[error("{0:?} is not a PES version this crate reads")]
    BadVersion(String),
    /// The 8-byte team id field is not ASCII decimal.
    #[error("team id {0:?} is not ASCII decimal")]
    BadTeamId(String),
    /// The file ends mid-structure.
    #[error("file is {len} bytes; {needed} are needed")]
    Truncated {
        /// The smallest size that would parse.
        needed: usize,
        /// The file's size.
        len: usize,
    },
    /// A stored playing-style byte the exporting version's list does not hold.
    #[error("playing style {value} is not in {version:?}'s list")]
    UnknownPlayingStyle {
        /// The exporting version.
        version: PesVersion,
        /// The stored byte.
        value: u8,
    },
    /// An instruction byte outside the canonical table.
    #[error("instruction byte {0} is not a canonical instruction")]
    BadInstruction(u8),
    /// A field value the field cannot hold.
    #[error(transparent)]
    Value(#[from] TeamTomlError),
    /// A name/shirt-name field that does not decode.
    #[error("text does not decode: {0}")]
    Text(String),
}

/// Tag + two version digits.
const HEADER: usize = 5;
/// `sizeof(player_export)`.
const RECORD: usize = 356;
/// The `[u16; 40]` shirt-number block after the records.
const NUMBERS: usize = 80;
/// The tactics block both formats carry.
const TACTICS: usize = 405;

// `player_export` field offsets (MSVC, 4-byte alignment, `bool` one byte,
// `wchar_t` two; one padding byte at 271 — mirrored from the fixture script's
// ctypes table).
/// player_export.nation
const NATION: usize = 0;
/// player_export.height
const HEIGHT: usize = 4;
/// player_export.weight
const WEIGHT: usize = 5;
/// player_export.gc1
const GC1: usize = 6;
/// player_export.gc2
const GC2: usize = 7;
/// player_export.mo_fk
const MO_FK: usize = 12;
/// player_export.b_edit_player
const B_EDIT_PLAYER: usize = 18;
/// player_export.b_edit_basicset
const B_EDIT_BASICSET: usize = 24;
/// player_export.mo_armd
const MO_ARMD: usize = 29;
/// player_export.b_edit_regpos
const B_EDIT_REGPOS: usize = 30;
/// player_export.age
const AGE: usize = 31;
/// player_export.reg_pos
const REG_POS: usize = 32;
/// player_export.play_style
const PLAY_STYLE: usize = 33;
/// player_export.mo_armr
const MO_ARMR: usize = 38;
/// player_export.mo_ck
const MO_CK: usize = 39;
/// player_export.play_pos[13]
const PLAY_POS: usize = 42;
/// player_export.mo_hunchd
const MO_HUNCHD: usize = 55;
/// player_export.mo_hunchr
const MO_HUNCHR: usize = 56;
/// player_export.mo_pk
const MO_PK: usize = 57;
/// player_export.mo_drib
const MO_DRIB: usize = 60;
/// player_export.b_edit_playpos
const B_EDIT_PLAYPOS: usize = 64;
/// player_export.b_edit_ability
const B_EDIT_ABILITY: usize = 65;
/// player_export.b_edit_skill
const B_EDIT_SKILL: usize = 66;
/// player_export.b_edit_style
const B_EDIT_STYLE: usize = 69;
/// player_export.b_edit_com
const B_EDIT_COM: usize = 70;
/// player_export.b_edit_motion
const B_EDIT_MOTION: usize = 71;
/// player_export.b_base_copy
const B_BASE_COPY: usize = 72;
/// player_export.strong_foot
const STRONG_FOOT: usize = 73;
/// player_export.strong_hand
const STRONG_HAND: usize = 74;
/// player_export.com_style[7]
const COM_STYLE: usize = 75;
/// player_export.play_skill[41]
const PLAY_SKILL: usize = 82;
/// player_export.name — wchar_t[61], UTF-16LE, NUL-terminated
const NAME: usize = 124;
/// player_export.name's byte length
const NAME_LEN: usize = 122;
/// player_export.shirt_name — char[21], single-byte, NUL-terminated
const SHIRT_NAME: usize = 246;
/// player_export.shirt_name's byte length
const SHIRT_NAME_LEN: usize = 21;
/// player_export.b_edit_face
const B_EDIT_FACE: usize = 267;
/// player_export.b_edit_hair
const B_EDIT_HAIR: usize = 268;
/// player_export.b_edit_phys
const B_EDIT_PHYS: usize = 269;
/// player_export.b_edit_strip
const B_EDIT_STRIP: usize = 270;
/// player_export.boot_id
const BOOT_ID: usize = 272;
/// player_export.glove_id
const GLOVE_ID: usize = 276;
/// player_export.copy_id
const COPY_ID: usize = 280;
/// player_export.neck_len … player_export.head_dep — thirteen i32s
const PHYSIQUE: usize = 284;
/// player_export.wrist_col_l
const WRIST_COL_L: usize = 340;
/// player_export.wrist_col_r
const WRIST_COL_R: usize = 341;
/// player_export.wrist_tape
const WRIST_TAPE: usize = 342;
/// player_export.spec_col
const SPEC_COL: usize = 343;
/// player_export.spec_style
const SPEC_STYLE: usize = 344;
/// player_export.sleeve
const SLEEVE: usize = 345;
/// player_export.inners
const INNERS: usize = 346;
/// player_export.socks
const SOCKS: usize = 347;
/// player_export.undershorts
const UNDERSHORTS: usize = 348;
/// player_export.untucked
const UNTUCKED: usize = 349;
/// player_export.ankle_tape
const ANKLE_TAPE: usize = 350;
/// player_export.gloves
const GLOVES: usize = 351;
/// player_export.gloves_col
const GLOVES_COL: usize = 352;
/// player_export.skin_col
const SKIN_COL: usize = 353;
/// player_export.iris_col
const IRIS_COL: usize = 354;

/// The byte stats and positions scalars that map one-to-one onto `PlayerKey`s,
/// `(offset, key)`. Every one is gated on the exporting version storing the
/// field and range-checked by `PlayerKey::set_section`.
const U8_SCALARS: &[(usize, PlayerKey)] = &[
    (8, PlayerKey::AttackingProwess),       // player_export.atk
    (9, PlayerKey::DefensiveProwess),       // player_export.def
    (10, PlayerKey::Goalkeeping),           // player_export.gk
    (11, PlayerKey::Dribbling),             // player_export.drib
    (13, PlayerKey::Finishing),             // player_export.finish
    (14, PlayerKey::LowPass),               // player_export.lowpass
    (15, PlayerKey::LoftedPass),            // player_export.loftpass
    (16, PlayerKey::Heading),               // player_export.header
    (17, PlayerKey::Form),                  // player_export.form
    (19, PlayerKey::Swerve),                // player_export.swerve
    (20, PlayerKey::Catching),              // player_export.catching
    (21, PlayerKey::Clearing),              // player_export.clearing
    (22, PlayerKey::Reflexes),              // player_export.reflex
    (23, PlayerKey::InjuryResistance),      // player_export.injury
    (25, PlayerKey::BodyControl),           // player_export.body_ctrl
    (26, PlayerKey::PhysicalContact),       // player_export.phys_cont
    (27, PlayerKey::KickingPower),          // player_export.kick_pwr
    (28, PlayerKey::ExplosivePower),        // player_export.exp_pwr
    (AGE, PlayerKey::Age),                  // player_export.age
    (REG_POS, PlayerKey::Registered),       // player_export.reg_pos
    (34, PlayerKey::BallControl),           // player_export.ball_ctrl
    (35, PlayerKey::BallWinning),           // player_export.ball_win
    (36, PlayerKey::WeakFootAccuracy),      // player_export.weak_acc
    (37, PlayerKey::Jump),                  // player_export.jump
    (40, PlayerKey::Coverage),              // player_export.cover
    (41, PlayerKey::WeakFootUsage),         // player_export.weak_use
    (58, PlayerKey::PlaceKicking),          // player_export.place_kick
    (59, PlayerKey::Star),                  // player_export.star
    (61, PlayerKey::TightPossession),       // player_export.tight_pos
    (62, PlayerKey::Aggression),            // player_export.aggres
    (63, PlayerKey::PlayingAttitude),       // player_export.play_attit
    (67, PlayerKey::Stamina),               // player_export.stamina
    (68, PlayerKey::Speed),                 // player_export.speed
    (STRONG_FOOT, PlayerKey::StrongerFoot), // player_export.strong_foot
    (STRONG_HAND, PlayerKey::StrongerHand), // player_export.strong_hand
];

/// The `bool` edit flags, `(offset, key)`.
const EDIT_FLAGS: &[(usize, PlayerKey)] = &[
    (B_EDIT_PLAYER, PlayerKey::EditedPlayer),
    (B_EDIT_BASICSET, PlayerKey::EditedBasicSettings),
    (B_EDIT_REGPOS, PlayerKey::EditedRegisteredPosition),
    (B_EDIT_PLAYPOS, PlayerKey::EditedPlayablePositions),
    (B_EDIT_ABILITY, PlayerKey::EditedAbilities),
    (B_EDIT_SKILL, PlayerKey::EditedSkills),
    (B_EDIT_STYLE, PlayerKey::EditedPlayingStyle),
    (B_EDIT_COM, PlayerKey::EditedComStyles),
    (B_EDIT_MOTION, PlayerKey::EditedMotion),
    (B_BASE_COPY, PlayerKey::EditedBaseCopy),
    (B_EDIT_FACE, PlayerKey::EditedFace),
    (B_EDIT_HAIR, PlayerKey::EditedHair),
    (B_EDIT_PHYS, PlayerKey::EditedPhysique),
    (B_EDIT_STRIP, PlayerKey::EditedStrip),
];

/// The `u32` ids, `(offset, key)`.
const U32_SCALARS: &[(usize, PlayerKey)] = &[
    (NATION, PlayerKey::Nationality),
    (BOOT_ID, PlayerKey::BootsId),
    (GLOVE_ID, PlayerKey::GlovesId),
    (COPY_ID, PlayerKey::BaseCopyId),
];

/// `u8` bytes that land in `appearance` (physique `height`/`weight`, motion,
/// strip and the face-run colours), `(offset, key)`.
const APPEARANCE_U8: &[(usize, SettingKey)] = &[
    (HEIGHT, SettingKey::Height),
    (WEIGHT, SettingKey::Weight),
    (GC1, SettingKey::GoalCelebration1),
    (GC2, SettingKey::GoalCelebration2),
    (MO_FK, SettingKey::FreeKick),
    (MO_ARMD, SettingKey::ArmMovementDribbling),
    (MO_ARMR, SettingKey::ArmMovementRunning),
    (MO_CK, SettingKey::CornerKick),
    (MO_HUNCHD, SettingKey::HunchingDribbling),
    (MO_HUNCHR, SettingKey::HunchingRunning),
    (MO_PK, SettingKey::PenaltyKick),
    (MO_DRIB, SettingKey::Dribbling),
    (WRIST_COL_L, SettingKey::WristTapeColorLeft),
    (WRIST_COL_R, SettingKey::WristTapeColorRight),
    (WRIST_TAPE, SettingKey::WristTaping),
    (SPEC_COL, SettingKey::SpectaclesColor),
    (SPEC_STYLE, SettingKey::Spectacles),
    (SLEEVE, SettingKey::Sleeves),
    (INNERS, SettingKey::Inners),
    (SOCKS, SettingKey::Socks),
    (UNDERSHORTS, SettingKey::Undershorts),
    (GLOVES_COL, SettingKey::GlovesColor),
    (SKIN_COL, SettingKey::SkinColor),
    (IRIS_COL, SettingKey::IrisColor),
];

/// `bool` bytes that land in `appearance.strip`.
const APPEARANCE_BOOL: &[(usize, SettingKey)] = &[
    (UNTUCKED, SettingKey::Untucked),
    (ANKLE_TAPE, SettingKey::AnkleTaping),
    (GLOVES, SettingKey::Gloves),
];

/// The thirteen `i32` physique fields, in struct order from `PHYSIQUE`.
const PHYSIQUE_KEYS: &[SettingKey] = &[
    SettingKey::NeckLength,
    SettingKey::NeckSize,
    SettingKey::ShoulderHeight,
    SettingKey::ShoulderWidth,
    SettingKey::Chest,
    SettingKey::Waist,
    SettingKey::ArmSize,
    SettingKey::ArmLength,
    SettingKey::Thigh,
    SettingKey::Calf,
    SettingKey::LegLength,
    SettingKey::HeadLength,
    SettingKey::HeadWidth,
    SettingKey::HeadDepth,
];

fn u32_at(record: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(record[at..at + 4].try_into().expect("four bytes"))
}

fn i32_at(record: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(record[at..at + 4].try_into().expect("four bytes"))
}

/// `wchar_t[61]`, UTF-16LE, NUL-terminated.
fn wide_string(bytes: &[u8]) -> Result<String, LegacyError> {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16(&units).map_err(|e| LegacyError::Text(e.to_string()))
}

/// `char[21]`, the codec's single-byte text (byte b is char U+00b),
/// NUL-terminated.
fn single_byte_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| char::from(b)).collect()
}

/// The two version digits at `bytes[3..5]` → `PesVersion`; anything that is
/// not `15`..=`21` is `BadVersion`.
fn version_digits(bytes: &[u8]) -> Result<PesVersion, LegacyError> {
    let digits = bytes
        .get(3..5)
        .and_then(|d| std::str::from_utf8(d).ok())
        .unwrap_or("??");
    let number = digits
        .parse::<u16>()
        .map_err(|_| LegacyError::BadVersion(digits.to_string()))?;
    PesVersion::from_number(number).ok_or_else(|| LegacyError::BadVersion(digits.to_string()))
}

/// One 356-byte `player_export` record → a `PlayerSection`. Fields the
/// exporting version does not store stay `None` whatever the bytes hold — the
/// writer fills some of them with stale values.
fn read_player(
    version: PesVersion,
    fields: &HashSet<PlayerField>,
    record: &[u8],
    number: Option<u16>,
) -> Result<PlayerSection, LegacyError> {
    debug_assert_eq!(record.len(), RECORD);
    let mut section = PlayerSection {
        name: Some(wide_string(&record[NAME..NAME + NAME_LEN])?),
        shirt_name: Some(single_byte_string(
            &record[SHIRT_NAME..SHIRT_NAME + SHIRT_NAME_LEN],
        )),
        number,
        ..PlayerSection::default()
    };
    for &(offset, key) in U8_SCALARS.iter().chain(EDIT_FLAGS) {
        if fields.contains(&key.spec().field) {
            key.set_section(&mut section, u32::from(record[offset]))?;
        }
    }
    for &(offset, key) in U32_SCALARS {
        if fields.contains(&key.spec().field) {
            key.set_section(&mut section, u32_at(record, offset))?;
        }
    }
    // The editor's `play_pos` is the save's playable array verbatim — CF 0 to
    // GK 12, the model's order, no remap.
    section.positions.playable = Some(
        record[PLAY_POS..PLAY_POS + 13]
            .try_into()
            .expect("thirteen bytes"),
    );
    section.positions.playing_style = Some(
        playstyle::decode(version, record[PLAY_STYLE]).map_err(|e| match e {
            CodecError::UnknownPlayingStyle { version, value } => {
                LegacyError::UnknownPlayingStyle { version, value }
            }
            other => LegacyError::from(TeamTomlError::from(other)),
        })?,
    );
    let mut skills = [false; 41];
    for (set, &byte) in skills.iter_mut().zip(&record[PLAY_SKILL..PLAY_SKILL + 41]) {
        *set = byte != 0;
    }
    section.skills.skills = Some(skills);
    let mut com_styles = [false; 7];
    for (set, &byte) in com_styles.iter_mut().zip(&record[COM_STYLE..COM_STYLE + 7]) {
        *set = byte != 0;
    }
    section.skills.com_styles = Some(com_styles);
    for &(offset, key) in APPEARANCE_U8 {
        put_appearance(&mut section, fields, key, u32::from(record[offset]))?;
    }
    for &(offset, key) in APPEARANCE_BOOL {
        put_appearance(&mut section, fields, key, u32::from(record[offset] != 0))?;
    }
    for (i, &key) in PHYSIQUE_KEYS.iter().enumerate() {
        let value = i32_at(record, PHYSIQUE + 4 * i);
        let value = u8::try_from(value).map_err(|_| TeamTomlError::OutOfRange {
            key: format!("appearance key {}", key.spec().name),
            value: i64::from(value),
            range: "0 to 255".to_string(),
        })?;
        put_appearance(&mut section, fields, key, u32::from(value))?;
    }
    Ok(section)
}

/// Write `value` into `section.appearance` under `key` when the exporting
/// version stores the key's field (`Source::Face` fields live in the ingame
/// face run, which every version has).
fn put_appearance(
    section: &mut PlayerSection,
    fields: &HashSet<PlayerField>,
    key: SettingKey,
    value: u32,
) -> Result<(), LegacyError> {
    let stored = match key.source() {
        Source::Player(field) => fields.contains(&field),
        Source::Face(_) => true,
    };
    if stored {
        let value = u8::try_from(value).map_err(|_| TeamTomlError::OutOfRange {
            key: key.spec().name.to_string(),
            value: i64::from(value),
            range: "0 to 255".to_string(),
        })?;
        set_appearance(&mut section.appearance, key, Some(value));
    }
    Ok(())
}

/// The 405-byte tactics block both formats carry, into `tactics`. The byte
/// order is the reference's `save_tactical_data` write walk.
fn tactics_block(block: &[u8]) -> Result<TacticsSection, LegacyError> {
    debug_assert_eq!(block.len(), TACTICS);
    let mut pos = 0;
    let mut tactics = TacticsSection::default();
    for preset in &mut tactics.presets {
        for formation in &mut preset.formations {
            let positions = &block[pos..pos + 11];
            let mut slots = [FormationSlot::default(); 11];
            pos += 11;
            for (k, slot) in slots.iter_mut().enumerate() {
                // The block is y, then x.
                *slot = FormationSlot {
                    position: positions[k],
                    y: block[pos],
                    x: block[pos + 1],
                };
                pos += 2;
            }
            *formation = Some(slots);
        }
        let style = &block[pos..pos + 7];
        preset.style = StyleSection {
            attacking_style: Some(style[0] != 0),
            buildup: Some(style[1] != 0),
            attacking_zone: Some(style[2] != 0),
            positioning: Some(style[3] != 0),
            defensive_style: Some(style[4] != 0),
            containment_area: Some(style[5] != 0),
            pressure: Some(style[6] != 0),
            ..StyleSection::default()
        };
        pos += 7;
        // Two attack then two defence instructions, canonical byte + player.
        let mut instructions = InstructionsSection::default();
        for entry in instructions
            .attack
            .iter_mut()
            .chain(&mut instructions.defence)
        {
            let byte = block[pos];
            entry.instruction = *Instruction::ALL
                .get(usize::from(byte))
                .ok_or(LegacyError::BadInstruction(byte))?;
            entry.player = block[pos + 1];
            pos += 2;
        }
        preset.instructions = Some(instructions);
        let sliders = &block[pos..pos + 5];
        preset.sliders = SlidersSection {
            support_range: Some(sliders[0]),
            numbers_in_attack: Some(sliders[1]),
            defensive_line: Some(sliders[2]),
            compactness: Some(sliders[3]),
            numbers_in_defence: Some(sliders[4]),
        };
        pos += 5;
        preset.style.fluid = Some(block[pos] != 0);
        pos += 1;
    }
    tactics.starting_eleven = Some(block[pos..pos + 11].try_into().expect("eleven"));
    pos += 11;
    tactics.bench_order = Some(block[pos..pos + 21].try_into().expect("twenty-one"));
    pos += 21;
    // FK long, FK short, FK 2, CK left, CK right, PK; no captain in the block.
    let takers = &block[pos..pos + 6];
    tactics.set_pieces = SetPiecesSection {
        free_kick_long: Some(takers[0]),
        free_kick_short: Some(takers[1]),
        free_kick_second: Some(takers[2]),
        corner_left: Some(takers[3]),
        corner_right: Some(takers[4]),
        penalty: Some(takers[5]),
        captain: None,
    };
    pos += 6;
    tactics.players_to_join_attack = Some(block[pos..pos + 3].try_into().expect("three"));
    pos += 3;
    tactics.auto = AutoSection {
        substitution: Some(block[pos]),
        offside_trap: Some(block[pos + 1] != 0),
        preset_change: Some(block[pos + 2] != 0),
        attack_defence_levels: Some(block[pos + 3] != 0),
    };
    pos += 4;
    debug_assert_eq!(pos, TACTICS);
    Ok(tactics)
}

/// A `.4ccs` squad file: `"20a"`/`"21a"`, two version digits, `n` player
/// records, the shirt-number block, an optional tactics block.
pub fn read_squad(bytes: &[u8]) -> Result<TeamToml, LegacyError> {
    if bytes.len() < HEADER + NUMBERS {
        return Err(LegacyError::Truncated {
            needed: HEADER + NUMBERS,
            len: bytes.len(),
        });
    }
    let tag = &bytes[..3];
    if tag != b"21a" && tag != b"20a" {
        return Err(LegacyError::BadTag {
            expected: "21a",
            found: String::from_utf8_lossy(tag).into_owned(),
        });
    }
    let version = version_digits(bytes)?;
    // Layout: header | n * RECORD | numbers | optional TACTICS. The block is
    // longer than a record, so test it by subtraction, not by remainder.
    let body = bytes.len() - HEADER - NUMBERS;
    let (players, block) = if body.is_multiple_of(RECORD) {
        (body / RECORD, None)
    } else if body >= TACTICS && (body - TACTICS).is_multiple_of(RECORD) {
        (
            (body - TACTICS) / RECORD,
            Some(&bytes[bytes.len() - TACTICS..]),
        )
    } else {
        // Either a truncated tactics block or a truncated record.
        let rem = body % RECORD;
        let needed = bytes.len() - rem + if rem <= TACTICS { TACTICS } else { RECORD };
        return Err(LegacyError::Truncated {
            needed,
            len: bytes.len(),
        });
    };
    let fields = schema_for(version).player_field_set();
    let numbers = &bytes[HEADER + players * RECORD..HEADER + players * RECORD + NUMBERS];
    let mut out = TeamToml {
        pes_version: Some(version),
        ..TeamToml::default()
    };
    for i in 0..players {
        let record = &bytes[HEADER + i * RECORD..HEADER + (i + 1) * RECORD];
        // The numbers block holds 40 slots; a file carrying more records has
        // no number for the rest.
        let number = (i < 40).then(|| u16::from_le_bytes([numbers[2 * i], numbers[2 * i + 1]]));
        let section = read_player(version, &fields, record, number)?;
        out.players.insert(
            u8::try_from(i + 1).expect("a roster slot index fits u8"),
            section,
        );
    }
    if let Some(block) = block {
        out.tactics = tactics_block(block)?;
    }
    Ok(out)
}

/// A `.4cct` "nightly" tactics file: `"001"`, two version digits, an 8-byte
/// ASCII team id, the 405-byte block.
pub fn read_tactics(bytes: &[u8]) -> Result<TeamToml, LegacyError> {
    const LENGTH: usize = 13 + TACTICS;
    if bytes.get(..3) != Some(b"001".as_slice()) {
        return Err(LegacyError::BadTag {
            expected: "001",
            found: String::from_utf8_lossy(bytes.get(..3).unwrap_or(b"")).into_owned(),
        });
    }
    if bytes.len() != LENGTH {
        return Err(LegacyError::Truncated {
            needed: LENGTH,
            len: bytes.len(),
        });
    }
    let version = version_digits(bytes)?;
    let id_bytes = &bytes[5..13];
    let end = id_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(id_bytes.len());
    let id_text = std::str::from_utf8(&id_bytes[..end]).unwrap_or("?");
    let team_id = id_text
        .parse::<u32>()
        .map_err(|_| LegacyError::BadTeamId(id_text.to_string()))?;
    let mut out = TeamToml {
        pes_version: Some(version),
        ..TeamToml::default()
    };
    out.team.id = Some(team_id);
    out.tactics = tactics_block(&bytes[13..])?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::open;

    const SQUAD: &[u8] = include_bytes!("../../tests/fixtures/pes19_squad.4ccs");
    const NIGHTLY: &[u8] = include_bytes!("../../tests/fixtures/pes19_tactics.4cct");

    /// A `.4ccs` with the tactics block appended carries both halves.
    #[test]
    fn a_squad_with_a_tactics_block_decodes_both() {
        let mut combined = SQUAD.to_vec();
        combined.extend_from_slice(&NIGHTLY[13..]);
        let doc = read_squad(&combined).expect("a .4ccs with tactics");
        assert_eq!(doc.players, read_squad(SQUAD).expect("squad").players);
        assert_eq!(doc.tactics, read_tactics(NIGHTLY).expect("tactics").tactics);
    }

    /// `"20a"` and `"21a"` are the same record; the tag is the only difference.
    #[test]
    fn both_squad_tags_decode_identically() {
        let mut tagged = SQUAD.to_vec();
        tagged[..3].copy_from_slice(b"21a");
        assert_eq!(
            read_squad(&tagged).expect("21a"),
            read_squad(SQUAD).expect("20a")
        );
    }

    /// A PES 16 header gates off the fields 16 does not store and decodes the
    /// playing style with 16's list.
    #[test]
    fn a_pes16_header_gates_the_newer_fields() {
        let mut v16 = SQUAD.to_vec();
        v16[3..5].copy_from_slice(b"16");
        // If any record's style byte is unlisted in 16, the whole file is
        // UnknownPlayingStyle — check that case honestly.
        let all_decode = (0..23).all(|i| {
            playstyle::decode(PesVersion::Pes16, v16[HEADER + i * RECORD + PLAY_STYLE]).is_ok()
        });
        if !all_decode {
            assert!(matches!(
                read_squad(&v16),
                Err(LegacyError::UnknownPlayingStyle { .. })
            ));
            return;
        }
        let doc = read_squad(&v16).expect("decodes under 16");
        for section in doc.players.values() {
            assert!(section.stats.physical_contact.is_none());
            assert!(section.stats.star.is_none());
            assert!(section.stats.tight_possession.is_none());
            assert!(section.positions.stronger_hand.is_none());
            assert!(section.appearance.motion.dribbling.is_none());
        }
        let raw = v16[HEADER + PLAY_STYLE];
        assert_eq!(
            doc.players[&1].positions.playing_style,
            playstyle::decode(PesVersion::Pes16, raw).ok()
        );
    }

    /// A 404-byte block is `Truncated`, not a short parse.
    #[test]
    fn a_short_tactics_block_is_truncated() {
        let err = read_tactics(&NIGHTLY[..NIGHTLY.len() - 1]).expect_err("one byte short");
        assert!(matches!(err, LegacyError::Truncated { .. }), "{err:?}");
    }

    /// An instruction byte the canonical table does not hold is BadInstruction.
    #[test]
    fn an_out_of_table_instruction_is_an_error() {
        // First instruction byte of preset 1: 3 formations * 33 + 7 style.
        let mut bad = NIGHTLY.to_vec();
        bad[13 + 106] = 0x20;
        assert!(matches!(
            read_tactics(&bad),
            Err(LegacyError::BadInstruction(0x20))
        ));
    }

    /// The end-to-end path: a `.4ccs` applies onto the fixture save's own team
    /// through `TeamToml::apply` and writes what the file says.
    #[test]
    fn a_squad_applies_to_the_fixture_team() {
        let doc = read_squad(SQUAD).expect("a valid .4ccs");
        let (file, _) = open(PesVersion::Pes19);
        let team = file.team(713).expect("team 713");
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        let notes = doc
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("same-version apply");
        assert!(notes.is_empty(), "{notes:?}");
        // Player 1's literals from the golden's sample table.
        let first = players.iter().find(|p| p.id == 71301).expect("applied");
        assert_eq!(first.name, "SHOTABOT 9S");
        assert_eq!(first.shirt_name, "NINE ES");
        assert_eq!(first.stats.attacking_prowess, 77);
        assert_eq!(first.stats.goalkeeping, 77);
        assert_eq!(first.positions.registered, 0);
        assert_eq!(first.basic.height, 189);
        assert_eq!(
            first
                .appearance
                .ingame_face
                .get(crate::schema::ingame_face::IngameFaceField::SkinColor)
                .ok(),
            Some(1)
        );
        let last = players.iter().find(|p| p.id == 71323).expect("applied");
        assert_eq!(last.name, "wtf im gransexual now??");
        assert_eq!(last.appearance.boots_id, 407);
        assert!(last.skills.skills[12] && last.skills.skills[13]);
    }
}
