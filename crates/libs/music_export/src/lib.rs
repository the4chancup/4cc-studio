//! The `.4ccm` music export format: a team's anthem, victory anthems, goalhorns,
//! chants and event clips, each song gated by conditions on the match state and
//! shaped by playback instructions.
//!
//! One typed model ([`MusicExport`]), one parser ([`parse`]) and one serializer
//! ([`write`]), shared by the music player and the music export editor, plus
//! what the player runs a match on: the match state ([`MatchState`]), the
//! condition evaluator ([`Verdict`]), each button's song selection
//! ([`TeamLists`]) and the on-disk lookup of an export's songs
//! (`resolve_export`, native only). The grammar, condition and instruction
//! tables and every type's shape are in
//! `docs/plans/music_player.md` "The `.4ccm` format". Callers own file I/O:
//! the parser takes bytes and the file's stem (the team name when the file
//! names none), the serializer returns the text.

mod eval;
mod model;
mod parse;
#[cfg(not(target_arch = "wasm32"))]
mod resolve;
mod select;
mod state;
mod tokens;
mod write;

pub use eval::Verdict;
pub use model::{
    Comparison, Condition, EndAction, EventClip, EventKind, Instruction, MatchType, MusicExport,
    PauseAction, Slot, SongEntry, StartOffset,
};
pub use parse::{ParseError, ParseErrorKind, Parsed, Warning, parse};
#[cfg(not(target_arch = "wasm32"))]
pub use resolve::{Resolution, resolve_export, resolve_song};
pub use select::{EntryId, Loaded, Press, TeamLists};
pub use state::{MatchState, Side, TeamState};
pub use write::write;
