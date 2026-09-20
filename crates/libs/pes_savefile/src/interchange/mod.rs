//! The interchange formats: files other tools and the game itself exchange
//! teams in. Each module reads its format into `model/` entries; the crypto
//! and layout tables live in `container`/`schema`.

/// The legacy 4ccEditor formats (`.4ccs`, `.4cct`), read-only.
pub mod legacy;
/// `team.toml`: the full-fidelity team interchange format.
pub mod team_toml;
/// PES's own in-game team export (`.ted` on 18-21, `TEXPORT00000000` on 15-17).
pub mod texport;

#[cfg(test)]
mod legacy_golden;
#[cfg(test)]
mod texport_golden;
