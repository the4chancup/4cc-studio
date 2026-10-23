//! The icon glyphs the shell and the tools draw: sidebar rail entries, the Help and
//! Settings buttons, the status bar's condition and dismiss icons. Each constant is the
//! glyph's one-character string, at the codepoint `resources/fonts/phosphor-subset.ttf`
//! keeps it at (that file's `README.md` lists the set and how the subset was cut from
//! Phosphor Icons). The font is the interim one: the suite's own icon font, drawn per
//! the core plan's "Styling is code, not CSS", replaces it in worklog step 8.14, and
//! these constants stay.

/// The Export upgrader's sidebar glyph.
pub const ARROW_FAT_UP: &str = "\u{E52E}";
/// The Stadium compiler's sidebar glyph.
pub const BUILDINGS: &str = "\u{E102}";
/// The Match tracker's sidebar glyph.
pub const CROSSHAIR: &str = "\u{E1D6}";
/// The DB generator's sidebar glyph.
pub const DATABASE: &str = "\u{E1DE}";
/// The status bar's update-available condition.
pub const DOWNLOAD_SIMPLE: &str = "\u{E20C}";
/// The Refs arranger's sidebar glyph.
pub const FLAG: &str = "\u{E244}";
/// The sidebar's Settings button.
pub const GEAR: &str = "\u{E270}";
/// The Music player's sidebar glyph.
pub const MUSIC_NOTE: &str = "\u{E33C}";
/// The Music export editor's sidebar glyph.
pub const MUSIC_NOTES: &str = "\u{E340}";
/// The Save editor's sidebar glyph.
pub const NOTE_PENCIL: &str = "\u{E34C}";
/// The Team creator's sidebar glyph.
pub const PLUS_CIRCLE: &str = "\u{E3D6}";
/// The sidebar's Help button.
pub const QUESTION: &str = "\u{E3E8}";
/// The Ruleset editor's sidebar glyph.
pub const SCROLL: &str = "\u{EB7A}";
/// The Player aesthetics editor's sidebar glyph.
pub const SMILEY: &str = "\u{E436}";
/// The Balls compiler's sidebar glyph.
pub const SOCCER_BALL: &str = "\u{E716}";
/// The Kit config editor's sidebar glyph.
pub const T_SHIRT: &str = "\u{E670}";
/// The Team compiler's sidebar glyph.
pub const USERS_THREE: &str = "\u{E68E}";
/// The status bar's read-only-data-folder condition.
pub const WARNING: &str = "\u{E4E0}";
/// The status bar notice's dismiss button.
pub const X: &str = "\u{E4F6}";
