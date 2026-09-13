//! The kit-config values every kit of an FPC team carries.

use pes_version::PesVersion;

/// The kit-config values every kit of an FPC team carries, goalkeeper kit
/// included (`resources/FPC.wikitext` lines 14–20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KitFpcValues {
    /// Shirt model (FPC.wikitext line 17).
    pub shirt_model: u16,
    /// Shorts model (FPC.wikitext line 18).
    pub shorts_model: u16,
    /// Collar (FPC.wikitext line 19).
    pub collar: u16,
    /// Winter collar (FPC.wikitext line 20).
    pub winter_collar: u16,
}

/// The values for a version, `None` where FPC does not exist: PES 16/17 (the
/// 2024 system, FPC.wikitext line 9) and PES 19–21 return the values above;
/// PES 15 and 18 return `None`.
pub fn kit_values(version: PesVersion) -> Option<KitFpcValues> {
    match version {
        PesVersion::Pes16
        | PesVersion::Pes17
        | PesVersion::Pes19
        | PesVersion::Pes20
        | PesVersion::Pes21 => Some(KitFpcValues {
            shirt_model: 176,
            shorts_model: 16,
            collar: 105,
            winter_collar: 105,
        }),
        PesVersion::Pes15 | PesVersion::Pes18 => None,
    }
}
