//! Parity of `schema::instruction` with the reference editor's advanced-
//! instruction translation: the per-version stored values held as a literal
//! table (its `translate_adv_instruction` / `get_translated_adv_instruction`
//! pair, PES 17 storing the canonical value and 18-21 permuting the upper
//! half), checked both ways and over every tactics record of the fixtures.
//! The reference stops at 0x0F; the PES 20 and 21 fixture saves carry a
//! stored 0x10 (seven entries over seven teams), the instruction PES 2020
//! added, canonical 0x10 here.

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

/// The PES 18-21 stored value of each canonical instruction, from the
/// reference's 18+ branch (`0x08..=0x0C` shift up by two, the three PES 18
/// additions take `0x08`, `0x09` and `0x0F`).
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

const WITH_INSTRUCTIONS_18_PLUS: [PesVersion; 4] = [
    PesVersion::Pes18,
    PesVersion::Pes19,
    PesVersion::Pes20,
    PesVersion::Pes21,
];

/// The value only PES 20/21 store; 18/19 lists end at 0x0F.
const ANCHORING_20_PLUS: u8 = 0x10;

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
fn pes18_to_21_permute_the_upper_half_as_the_reference_does() {
    for version in WITH_INSTRUCTIONS_18_PLUS {
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
        let has_anchoring = matches!(version, PesVersion::Pes20 | PesVersion::Pes21);
        assert_eq!(
            decode(version, ANCHORING_20_PLUS).ok().flatten(),
            has_anchoring.then_some(Instruction::Anchoring),
            "{version:?} stored 0x10"
        );
        assert_eq!(
            encode(version, Instruction::Anchoring),
            has_anchoring.then_some(ANCHORING_20_PLUS),
            "{version:?} anchoring"
        );
        if !has_anchoring {
            assert!(matches!(
                decode(version, ANCHORING_20_PLUS),
                Err(CodecError::UnknownInstruction { .. })
            ));
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
        anchoring, 7,
        "the 20/21 fixtures' seven stored 0x10 entries decode"
    );
}
