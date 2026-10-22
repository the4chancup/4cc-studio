//! `team_widgets`: the egui widgets the Save editor and the Team creator both
//! draw over `pes_savefile`'s model types (`plans/save_editor.md`
//! "`libs/team_widgets`"). A widget takes `&mut` model, the version's schema
//! for gating and the AATF rules where it shows limits; it draws and it
//! returns what changed, so the host decides what an edit means. It never
//! sees a session, an undo stack, a file, or a savefile byte.

/// The card pickers: the skills grid and COM playing styles with a
/// remaining-count badge per ruleset tier, and the playing-style combo.
pub mod cards;
/// The tactics editor's settings half: preset selector, sliders, style
/// pairs, advanced instructions and automatic settings.
pub mod tactics;
/// The headless egui harness the tests share.
#[cfg(test)]
mod test_support;
/// The AATF results list: a team's violations grouped per player, a row's
/// click reporting the player to select.
pub mod violations;
