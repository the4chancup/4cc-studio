//! What consumers edit: the version-independent model the codec fills. No I/O
//! and no version knowledge lives here; a field a version lacks is `None` or
//! defaulted, and the schema decides what is read and written.

/// The player model.
pub mod player;

pub use player::{
    PlayerAppearance, PlayerBasics, PlayerEditFlags, PlayerEntry, PlayerMotion, PlayerPositions,
    PlayerSkills, PlayerStats,
};
