//! The canonical advanced-instruction enum: the sixteen values PES 18-21
//! share. The stored value is a per-version index (PES 15/16 have no
//! instruction table at all); `schema::instruction` maps it, this enum
//! carries no version knowledge.

/// One advanced instruction, in the canonical order (PES 17's stored order for
/// the first thirteen; the PES 18 additions at the end).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    /// No instruction.
    Off,
    /// Hug the Touchline.
    HugTheTouchline,
    /// False No. 9.
    FalseNo9,
    /// False Full-Backs.
    FalseFullBacks,
    /// Attacking Full-Backs.
    AttackingFullBacks,
    /// Wing Rotation.
    WingRotation,
    /// Tiki-Taka.
    TikiTaka,
    /// Centring Targets.
    CenteringTargets,
    /// Swarm the Box.
    SwarmTheBox,
    /// Deep Defensive Line.
    DeepDefensiveLine,
    /// Gegenpress.
    Gegenpress,
    /// Tight Marking.
    TightMarking,
    /// Counter Target.
    CounterTarget,
    /// Defensive (PES 18+).
    Defensive,
    /// False Winger (PES 18+).
    FalseWinger,
    /// Wing-Back (PES 18+).
    WingBack,
    /// Anchoring (PES 20/21 only).
    Anchoring,
}

impl Instruction {
    /// Every value, in the canonical order.
    pub const ALL: [Instruction; 17] = [
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

    /// Whether the instruction is given to one of the team's own players,
    /// picked with it: the instructions whose target-player field applies.
    pub fn takes_player(self) -> bool {
        matches!(self, Instruction::CounterTarget | Instruction::Defensive)
    }
}

impl Default for Instruction {
    /// The empty instruction (`Off` is the game's own "unset" value).
    fn default() -> Self {
        Instruction::Off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // plans/save_editor.md "Tactics editor", "What the controls write":
    // the target-player pick is enabled only for the instructions that
    // take one of ours, Counter Target and Defensive; here is the one
    // place the rule lives.
    #[test]
    fn only_counter_target_and_defensive_take_a_player() {
        for instruction in Instruction::ALL {
            assert_eq!(
                instruction.takes_player(),
                matches!(
                    instruction,
                    Instruction::CounterTarget | Instruction::Defensive
                ),
                "{instruction:?}"
            );
        }
    }
}
