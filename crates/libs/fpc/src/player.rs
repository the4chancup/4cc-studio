//! The three FPC appearance presets, in this crate's own vocabulary rather
//! than savefile field names.

use std::ops::RangeInclusive;

/// Sleeve length (`FPC.wikitext` lines 27, 36, 45).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sleeves {
    /// Short sleeves.
    Short,
    /// Long sleeves.
    Long,
}

/// Whether the shirt is tucked (FPC.wikitext lines 28, 37, 46).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tuck {
    /// Untucked shirt.
    Untucked,
    /// Tucked shirt.
    Tucked,
}

/// Sock length (FPC.wikitext lines 29, 38, 47).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Socks {
    /// Short socks.
    Short,
    /// Standard socks.
    Standard,
    /// Long socks.
    Long,
}

/// Whether the skin color is a preset or Custom (FPC.wikitext line 57).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinColor {
    /// A preset skin color.
    #[default]
    Preset,
    /// Custom — hides the body but not the jersey.
    Custom,
}

/// The nonexistent boots ID the hide preset uses (FPC.wikitext line 30); a
/// custom model's own boots are substituted by the caller instead.
pub const NONEXISTENT_BOOTS_ID: u16 = 55;
/// The nonexistent gloves ID the hide preset uses (FPC.wikitext line 31); a
/// custom model's own gloves are substituted by the caller instead.
pub const NONEXISTENT_GLOVES_ID: u16 = 11;
/// The real gloves IDs goalkeepers may use when un-hidden (FPC.wikitext
/// line 40).
pub const GK_GLOVES_RANGE: RangeInclusive<u16> = 1..=10;

/// One of the three appearance presets. Boots/gloves ids are the nonexistent
/// ids the text prescribes; a custom model's own boots/gloves are the caller's
/// substitution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Appearance {
    /// Sleeve length.
    pub sleeves: Sleeves,
    /// Shirt tuck.
    pub tuck: Tuck,
    /// Sock length.
    pub socks: Socks,
    /// Boots ID.
    pub boots_id: u16,
    /// Gloves ID.
    pub gloves_id: u16,
    /// Skin color preset/custom.
    pub skin_color: SkinColor,
}

/// The preset kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Completely hide the player's default body (FPC.wikitext lines 24–31).
    Hide,
    /// Completely show the default body (lines 33–40).
    Unhide,
    /// Show the stock jersey but hide the skin (lines 42–48).
    PartialHide,
}

/// The appearance one preset prescribes.
pub fn preset(kind: Preset) -> Appearance {
    match kind {
        Preset::Hide => Appearance {
            sleeves: Sleeves::Long,
            tuck: Tuck::Tucked,
            socks: Socks::Short,
            boots_id: NONEXISTENT_BOOTS_ID,
            gloves_id: NONEXISTENT_GLOVES_ID,
            skin_color: SkinColor::Preset,
        },
        Preset::Unhide => Appearance {
            sleeves: Sleeves::Short,
            tuck: Tuck::Untucked,
            socks: Socks::Standard,
            boots_id: 0,
            gloves_id: 0,
            skin_color: SkinColor::Preset,
        },
        Preset::PartialHide => Appearance {
            sleeves: Sleeves::Short,
            tuck: Tuck::Untucked,
            socks: Socks::Standard,
            boots_id: 0,
            gloves_id: 0,
            skin_color: SkinColor::Custom,
        },
    }
}
