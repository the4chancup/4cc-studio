//! What consumers edit: the version-independent model the codec fills. No I/O
//! and no version knowledge lives here; a field a version lacks is `None` or
//! defaulted, and the schema decides what is read and written.

/// The ingame-face run: opaque bytes with typed accessors.
pub mod ingame_face;
/// Save-name helpers: colour-code stripping for display.
pub mod names;
/// The player model.
pub mod player;
/// The tactics model.
pub mod tactics;
/// The team model.
pub mod team;

pub use ingame_face::IngameFace;
pub use player::{
    PlayerAppearance, PlayerBasics, PlayerEditFlags, PlayerEntry, PlayerMotion, PlayerPositions,
    PlayerSkills, PlayerStats,
};
pub use tactics::{
    AdvancedInstruction, Formation, FormationSlot, SetPieceTakers, TacticSliders, TacticStyle,
    TacticsPreset, TeamAutoFlags, TeamTactics,
};
pub use team::{KitSlot, RosterSlot, TeamColor, TeamEditFlags, TeamEntry};
