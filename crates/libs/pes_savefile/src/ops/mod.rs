//! Save-to-save and preset operations over `model/` only, no bytes.

/// The aesthetics fingerprint: the normalized ingame-face run's hash.
pub mod fingerprint;
/// The FPC presets and interference inputs.
pub mod fpc;
/// Aesthetics transplant between two saves of the same version.
pub mod transplant;
/// Parity of transplant and fingerprint with the reference scripts' byte rules.
#[cfg(test)]
mod transplant_golden;
