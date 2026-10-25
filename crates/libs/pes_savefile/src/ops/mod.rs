//! Save-to-save and preset operations over `model/` only, no bytes.

/// Parity of the appearance block and the fingerprint with the reference
/// scripts' byte rules.
#[cfg(test)]
mod appearance_golden;
/// The comparator: every stored-field difference between two same-version
/// players or saves.
pub mod compare;
/// Parity of compare with the compare script's diff outcome.
#[cfg(test)]
mod compare_golden;
/// The aesthetics fingerprint: the normalized ingame-face run's hash.
pub mod fingerprint;
/// The FPC presets and interference inputs.
pub mod fpc;
