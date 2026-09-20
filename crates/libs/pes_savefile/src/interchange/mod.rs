//! The interchange formats: files other tools and the game itself exchange
//! teams in. Each module reads its format into `model/` entries; the crypto
//! and layout tables live in `container`/`schema`.

/// PES's own in-game team export (`.ted` on 18-21, `TEXPORT00000000` on 15-17).
pub mod texport;

#[cfg(test)]
mod texport_golden;
