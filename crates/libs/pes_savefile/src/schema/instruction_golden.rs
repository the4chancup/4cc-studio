//! Parity of `schema::instruction` with the reference editor's advanced-
//! instruction translation for PES 17-19: the per-version stored values held
//! as a literal table (its `translate_adv_instruction` /
//! `get_translated_adv_instruction` pair, PES 17 storing the canonical value
//! and 18/19 permuting the upper half), checked both ways and over every
//! tactics record of the fixtures. On 20/21 the reference applies its 18
//! table and is wrong: the expected values there are the ones measured
//! in-game on PES 21, the 18/19 list with Anchoring inserted at 0x0A and
//! 18/19's 0x0A-0x0F moved to 0x0B-0x10.

use pes_version::PesVersion;

use crate::codec::CodecError;
use crate::model::instruction::Instruction;
use crate::schema::instruction::{decode, encode};
use crate::schema::schema_for;
use crate::test_support::{FIXTURES, open};

/// The canonical order, index = the PES 17 stored value up to 0x0C.
const CANONICAL: [Instruction; 17] = [
    Instruction::Off,
    Instruction::HugTheTouchline,
    Instruction::FalseNo9,
    Instruction::FalseFullBacks,
    Instruction::AttackingFullBacks,
    Instruction::WingRotation,
    Instruction::TikiTaka,
    Instruction::CenteringTargets,
    Instruction::SwarmTheBox,
    Instruction::DeepDefensiveLine,
    Instruction::Gegenpress,
    Instruction::TightMarking,
    Instruction::CounterTarget,
    Instruction::Defensive,
    Instruction::FalseWinger,
    Instruction::WingBack,
    Instruction::Anchoring,
];

/// The PES 18/19 stored value of each canonical instruction, from the
/// reference's 18+ branch (`0x08..=0x0C` shift up by two, the three PES 18
/// additions take `0x08`, `0x09` and `0x0F`); PES 20/21 permute again,
/// `STORED_20_PLUS`.
const STORED_18_PLUS: [(Instruction, u8); 16] = [
    (Instruction::Off, 0x00),
    (Instruction::HugTheTouchline, 0x01),
    (Instruction::FalseNo9, 0x02),
    (Instruction::FalseFullBacks, 0x03),
    (Instruction::AttackingFullBacks, 0x04),
    (Instruction::WingRotation, 0x05),
    (Instruction::TikiTaka, 0x06),
    (Instruction::CenteringTargets, 0x07),
    (Instruction::Defensive, 0x08),
    (Instruction::FalseWinger, 0x09),
    (Instruction::SwarmTheBox, 0x0A),
    (Instruction::DeepDefensiveLine, 0x0B),
    (Instruction::Gegenpress, 0x0C),
    (Instruction::TightMarking, 0x0D),
    (Instruction::CounterTarget, 0x0E),
    (Instruction::WingBack, 0x0F),
];

/// The PES 20/21 stored value of each canonical instruction: the 18/19
/// list with Anchoring inserted at 0x0A, measured in-game on PES 21 (the
/// reference applies its 18 table and names the six moved values wrong).
const STORED_20_PLUS: [(Instruction, u8); 17] = [
    (Instruction::Off, 0x00),
    (Instruction::HugTheTouchline, 0x01),
    (Instruction::FalseNo9, 0x02),
    (Instruction::FalseFullBacks, 0x03),
    (Instruction::AttackingFullBacks, 0x04),
    (Instruction::WingRotation, 0x05),
    (Instruction::TikiTaka, 0x06),
    (Instruction::CenteringTargets, 0x07),
    (Instruction::Defensive, 0x08),
    (Instruction::FalseWinger, 0x09),
    (Instruction::Anchoring, 0x0A),
    (Instruction::SwarmTheBox, 0x0B),
    (Instruction::DeepDefensiveLine, 0x0C),
    (Instruction::Gegenpress, 0x0D),
    (Instruction::TightMarking, 0x0E),
    (Instruction::CounterTarget, 0x0F),
    (Instruction::WingBack, 0x10),
];

/// The versions the reference's 18+ table holds for.
const WITH_INSTRUCTIONS_18_19: [PesVersion; 2] = [PesVersion::Pes18, PesVersion::Pes19];

/// The versions that insert Anchoring at 0x0A.
const WITH_INSTRUCTIONS_20_21: [PesVersion; 2] = [PesVersion::Pes20, PesVersion::Pes21];

/// The attack instructions of PES 21's Edit-mode Game Plan screen.
const ATTACK_INSTRUCTIONS: [Instruction; 10] = [
    Instruction::HugTheTouchline,
    Instruction::FalseNo9,
    Instruction::FalseFullBacks,
    Instruction::AttackingFullBacks,
    Instruction::WingRotation,
    Instruction::TikiTaka,
    Instruction::CenteringTargets,
    Instruction::Defensive,
    Instruction::FalseWinger,
    Instruction::Anchoring,
];

/// The defence instructions of the same screen.
const DEFENCE_INSTRUCTIONS: [Instruction; 6] = [
    Instruction::SwarmTheBox,
    Instruction::DeepDefensiveLine,
    Instruction::Gegenpress,
    Instruction::TightMarking,
    Instruction::CounterTarget,
    Instruction::WingBack,
];

#[test]
fn pes17_stores_the_canonical_value_and_lacks_the_three_pes18_additions() {
    for (value, instruction) in CANONICAL.iter().enumerate().take(0x0D) {
        let value = u8::try_from(value).expect("fits");
        assert_eq!(
            decode(PesVersion::Pes17, value).expect("listed"),
            Some(*instruction)
        );
        assert_eq!(encode(PesVersion::Pes17, *instruction), Some(value));
    }
    for value in 0x0D..=0xFF_u8 {
        assert!(
            matches!(
                decode(PesVersion::Pes17, value),
                Err(CodecError::UnknownInstruction {
                    version: PesVersion::Pes17,
                    value: v
                }) if v == value
            ),
            "PES 17 stored {value:#x}"
        );
    }
    for instruction in [
        Instruction::Defensive,
        Instruction::FalseWinger,
        Instruction::WingBack,
        Instruction::Anchoring,
    ] {
        assert_eq!(
            encode(PesVersion::Pes17, instruction),
            None,
            "{instruction:?}"
        );
    }
}

#[test]
fn pes18_and_19_permute_the_upper_half_as_the_reference_does() {
    for version in WITH_INSTRUCTIONS_18_19 {
        for (instruction, stored) in STORED_18_PLUS {
            assert_eq!(
                decode(version, stored).expect("listed"),
                Some(instruction),
                "{version:?} stored {stored:#x}"
            );
            assert_eq!(
                encode(version, instruction),
                Some(stored),
                "{version:?} {instruction:?}"
            );
        }
        assert_eq!(
            encode(version, Instruction::Anchoring),
            None,
            "{version:?} anchoring"
        );
        for value in 0x10..=0xFF_u8 {
            assert!(
                matches!(
                    decode(version, value),
                    Err(CodecError::UnknownInstruction { .. })
                ),
                "{version:?} stored {value:#x}"
            );
        }
    }
}

#[test]
fn pes20_and_21_insert_anchoring_at_0x0a() {
    for version in WITH_INSTRUCTIONS_20_21 {
        for (instruction, stored) in STORED_20_PLUS {
            assert_eq!(
                decode(version, stored).expect("listed"),
                Some(instruction),
                "{version:?} stored {stored:#x}"
            );
            assert_eq!(
                encode(version, instruction),
                Some(stored),
                "{version:?} {instruction:?}"
            );
        }
        for value in 0x11..=0xFF_u8 {
            assert!(
                matches!(
                    decode(version, value),
                    Err(CodecError::UnknownInstruction { .. })
                ),
                "{version:?} stored {value:#x}"
            );
        }
    }
}

/// The five instructions set in-game on PES 21 (Edit > Teams > Game Plan,
/// saved) and read back from the save: Anchoring takes 0x0A, the 18/19
/// values after it shifting up one.
#[test]
fn pes20_and_21_decode_the_values_measured_in_game() {
    for version in WITH_INSTRUCTIONS_20_21 {
        for (stored, instruction) in [
            (0x09, Instruction::FalseWinger),
            (0x0A, Instruction::Anchoring),
            (0x0B, Instruction::SwarmTheBox),
            (0x0C, Instruction::DeepDefensiveLine),
            (0x0F, Instruction::CounterTarget),
        ] {
            assert_eq!(
                decode(version, stored).expect("listed"),
                Some(instruction),
                "{version:?} stored {stored:#x}"
            );
        }
    }
}

/// The set instructions of every fixture sit on the side PES 21's Game Plan
/// screen lists them under (a table that swaps an attack name into a defence
/// slot's value is caught here, where a round trip is not).
#[test]
fn every_fixture_instruction_sits_on_its_side() {
    let mut attack = 0usize;
    let mut defence = 0usize;
    for version in FIXTURES {
        let (file, _) = open(version);
        for team in file.teams() {
            for (preset, preset_tactics) in team.tactics.presets.iter().enumerate() {
                for (side, entries, listed) in [
                    (
                        "attack",
                        &preset_tactics.attack_instructions,
                        &ATTACK_INSTRUCTIONS[..],
                    ),
                    (
                        "defence",
                        &preset_tactics.defence_instructions,
                        &DEFENCE_INSTRUCTIONS[..],
                    ),
                ] {
                    for entry in entries.iter().flatten() {
                        let canonical = decode(version, entry.instruction)
                            .unwrap_or_else(|e| {
                                panic!(
                                    "{version:?} team {} preset {} {side} stored {:#x}: {e}",
                                    team.id,
                                    preset + 1,
                                    entry.instruction
                                )
                            })
                            .expect("a version with instructions decodes to Some");
                        if canonical == Instruction::Off {
                            continue;
                        }
                        assert!(
                            listed.contains(&canonical),
                            "{version:?} team {} preset {} {side} stored {:#x} \
                             decodes to {canonical:?}, not on the {side} list",
                            team.id,
                            preset + 1,
                            entry.instruction
                        );
                        if side == "attack" {
                            attack += 1;
                        } else {
                            defence += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(attack >= 20, "only {attack} set attack instructions");
    assert!(defence >= 20, "only {defence} set defence instructions");
}

#[test]
fn pes15_and_16_have_no_instructions() {
    for version in [PesVersion::Pes15, PesVersion::Pes16] {
        for value in 0..=0xFF_u8 {
            assert_eq!(decode(version, value).expect("no list, no error"), None);
        }
        for instruction in CANONICAL {
            assert_eq!(encode(version, instruction), None);
        }
    }
}

#[test]
fn every_fixture_instruction_decodes_and_re_encodes_to_itself() {
    let mut seen = 0usize;
    let mut anchoring = 0usize;
    let mut stored_0x10 = 0usize;
    for version in FIXTURES {
        let (file, _) = open(version);
        let has_instructions = schema_for(version).tactic.instructions.is_some();
        for team in file.teams() {
            for preset in &team.tactics.presets {
                let sides = [&preset.attack_instructions, &preset.defence_instructions];
                for side in sides {
                    assert_eq!(
                        side.is_some(),
                        has_instructions,
                        "{version:?} team {}",
                        team.id
                    );
                    for entry in side.iter().flatten() {
                        let canonical = decode(version, entry.instruction)
                            .unwrap_or_else(|e| {
                                panic!(
                                    "{version:?} team {} stored {:#x}: {e}",
                                    team.id, entry.instruction
                                )
                            })
                            .expect("a version with instructions decodes to Some");
                        assert_eq!(encode(version, canonical), Some(entry.instruction));
                        if canonical != Instruction::Off {
                            seen += 1;
                        }
                        if canonical == Instruction::Anchoring {
                            anchoring += 1;
                        }
                        if entry.instruction == 0x10 {
                            assert_eq!(
                                canonical,
                                Instruction::WingBack,
                                "{version:?} team {} stored 0x10",
                                team.id
                            );
                            stored_0x10 += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(
        seen >= 50,
        "the fixtures carry {seen} set instructions; the check is not vacuous"
    );
    assert_eq!(
        anchoring, 24,
        "the 20/21 fixtures' stored 0x0A entries decode to Anchoring"
    );
    assert_eq!(
        stored_0x10, 7,
        "the seven stored 0x10 entries decode to WingBack"
    );
}
