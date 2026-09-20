//! The per-version advanced-instruction mappings (plan:
//! `pes_savefile/model.md`, "Version quirks"). PES 15/16 have no instruction
//! table (`decode` is `Ok(None)` for every value, `encode` `None`). PES 17's
//! stored order IS the canonical order for the thirteen it has; PES 18-21
//! keep 0x00-0x07 and permute the rest into 0x08-0x0F.

use pes_version::PesVersion;

use crate::codec::CodecError;
use crate::model::instruction::Instruction;

use Instruction as I;

/// PES 18/19: the stored value indexes this list (the reference editor's
/// table verbatim — it ends at 0x0F).
const STORED_18: [Instruction; 16] = [
    I::Off,
    I::HugTheTouchline,
    I::FalseNo9,
    I::FalseFullBacks,
    I::AttackingFullBacks,
    I::WingRotation,
    I::TikiTaka,
    I::CenteringTargets,
    I::Defensive,
    I::FalseWinger,
    I::SwarmTheBox,
    I::DeepDefensiveLine,
    I::Gegenpress,
    I::TightMarking,
    I::CounterTarget,
    I::WingBack,
];

/// PES 20/21: the 18/19 list plus `Anchoring` at 0x10, the instruction PES
/// 2020 added — measured on the 20/21 fixture saves (the reference's table
/// ends at 0x0F); the name is the game's, unverified against the saves' edit
/// screen.
const STORED_20: [Instruction; 17] = [
    I::Off,
    I::HugTheTouchline,
    I::FalseNo9,
    I::FalseFullBacks,
    I::AttackingFullBacks,
    I::WingRotation,
    I::TikiTaka,
    I::CenteringTargets,
    I::Defensive,
    I::FalseWinger,
    I::SwarmTheBox,
    I::DeepDefensiveLine,
    I::Gegenpress,
    I::TightMarking,
    I::CounterTarget,
    I::WingBack,
    I::Anchoring,
];

/// The stored-value list `version` uses, `None` on PES 15/16 (no table).
fn stored(version: PesVersion) -> Option<&'static [Instruction]> {
    match version {
        PesVersion::Pes15 | PesVersion::Pes16 => None,
        PesVersion::Pes17 => Some(&Instruction::ALL[..13]),
        PesVersion::Pes18 | PesVersion::Pes19 => Some(&STORED_18),
        PesVersion::Pes20 | PesVersion::Pes21 => Some(&STORED_20),
    }
}

/// The instruction stored as `value` in `version`'s table: `Ok(None)` when the
/// version has no instruction table, `Err(UnknownInstruction)` for a value its
/// table does not list.
pub fn decode(version: PesVersion, value: u8) -> Result<Option<Instruction>, CodecError> {
    let Some(table) = stored(version) else {
        return Ok(None);
    };
    match table.get(usize::from(value)) {
        Some(instruction) => Ok(Some(*instruction)),
        None => Err(CodecError::UnknownInstruction { version, value }),
    }
}

/// The stored value `instruction` has in `version`'s table, `None` when the
/// version has no table or no such instruction.
pub fn encode(version: PesVersion, instruction: Instruction) -> Option<u8> {
    stored(version)?
        .iter()
        .position(|i| *i == instruction)
        .map(|index| u8::try_from(index).expect("the tables fit u8"))
}
