//! The `.4ccm` music export format: a team's anthem, victory anthems, goalhorns,
//! chants and event clips, each song gated by conditions on the match state and
//! shaped by playback instructions.
//!
//! One typed model ([`MusicExport`]), one parser ([`parse`]) and one serializer
//! ([`write`]), shared by the music player and the music export editor. The
//! grammar, condition and instruction tables are in
//! `docs/plans/music_player.md` "The `.4ccm` format". Callers own file I/O:
//! the parser takes bytes and the file's stem (the team name when the file
//! names none), the serializer returns the text.

mod model;
mod parse;
mod tokens;
mod write;

pub use model::{
    Comparison, Condition, EndAction, EventClip, EventKind, Instruction, MatchType, MusicExport,
    PauseAction, Slot, SongEntry, StartOffset,
};
pub use parse::{ParseError, ParseErrorKind, Parsed, Warning, parse};
pub use write::write;
