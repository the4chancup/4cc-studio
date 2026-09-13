//! The `KitConfig` struct and its field enums (kit_config_editor plan,
//! "The format").

use std::collections::BTreeMap;

/// A 120-byte kit config's decoded fields plus everything not yet decoded.
#[derive(Debug, Clone, PartialEq)]
pub struct KitConfig {
    /// Shirt block.
    pub shirt: Shirt,
    /// Shorts model.
    pub shorts: Shorts,
    /// The five kit colors.
    pub colors: Colors,
    /// Shirt name text.
    pub name: NameText,
    /// The three printed numbers.
    pub numbers: Numbers,
    /// The four sleeve-badge positions.
    pub badges: Badges,
    /// Undecoded bits per byte offset, known bits zeroed. Only offsets whose
    /// remainder is nonzero are present.
    pub unknown: BTreeMap<u8, u8>,
    /// The five raw 16-byte texture names (kit, back, chest, leg, name) as
    /// read; `None` for a config built from TOML without
    /// `[source_texture_names]`.
    pub source_texture_names: Option<[[u8; 16]; 5]>,
}

/// The shirt fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shirt {
    /// Literal model byte: 144, 160 or 176, or any other value read.
    pub model: u8,
    /// Collar id, 1-107.
    pub collar: u8,
    /// Winter collar id, 1-107.
    pub winter_collar: u8,
    /// Tight shirt (models 144/160 only).
    pub tight: bool,
    /// Shirt pattern, 0-13.
    pub pattern: u8,
    /// Long-sleeves type.
    pub long_sleeves: LongSleeves,
    /// Short-sleeves type.
    pub short_sleeves: ShortSleeves,
}

/// Long-sleeves type (byte 0x02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongSleeves {
    /// 0x3E, the combo entry and usual value.
    Normal,
    /// 0xBB, undershirt only (shirt model 144 only).
    UndershirtOnly,
    /// Any other byte, preserved verbatim.
    Raw(u8),
}

/// Short-sleeves type (byte 0x00 bits 0-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortSleeves {
    /// 1, normal.
    Normal,
    /// 2, cut-out (shirt model 144 only).
    CutOut,
    /// 0 or 3, preserved verbatim.
    Raw(u8),
}

/// The shorts model byte (0x03), 0-17.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shorts {
    /// Shorts model.
    pub model: u8,
}

/// An RGB triplet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// The config's five kit colors (bytes 0x04-0x12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    /// Shirt color 1.
    pub shirt1: Rgb,
    /// Shirt color 2.
    pub shirt2: Rgb,
    /// Undershirt color.
    pub undershirt: Rgb,
    /// Shorts color.
    pub shorts: Rgb,
    /// Socks color.
    pub socks: Rgb,
}

/// Shirt name text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameText {
    /// Whether the name is shown (0 = hidden).
    pub show: bool,
    /// The name's arc shape.
    pub shape: NameShape,
    /// Vertical position (0-16; 0-39 on PES 21).
    pub y: u8,
    /// Size, 0-20.
    pub size: u8,
}

/// The name arc shape (byte 0x1D bits 6-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameShape {
    /// 0.
    Straight,
    /// 1.
    LightCurve,
    /// 2.
    MediumCurve,
    /// 3.
    ExtremeCurve,
}

/// The three printed numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Numbers {
    /// Back number.
    pub back: BackNumber,
    /// Chest number.
    pub chest: ChestNumber,
    /// Shorts number.
    pub shorts: ShortsNumber,
}

/// Back number placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackNumber {
    /// Y, 0-29.
    pub y: u8,
    /// Size, 0-13.
    pub size: u8,
    /// Spacing, 0-2.
    pub spacing: u8,
}

/// Chest number placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChestNumber {
    /// X, 0-14.
    pub x: u8,
    /// Y, 0-7.
    pub y: u8,
    /// Size, 0-15.
    pub size: u8,
}

/// Shorts number placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortsNumber {
    /// Left or right leg.
    pub side: Side,
    /// X, 0-14.
    pub x: u8,
    /// Y, 0-14.
    pub y: u8,
    /// Size, 0-13.
    pub size: u8,
}

/// Which leg the shorts number sits on (byte 0x17 bit 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Left.
    Left,
    /// Right.
    Right,
}

/// The four sleeve-badge positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badges {
    /// Badge on the right short sleeve.
    pub right_short: Position,
    /// Badge on the left short sleeve.
    pub left_short: Position,
    /// Badge on the right long sleeve.
    pub right_long: Position,
    /// Badge on the left long sleeve.
    pub left_long: Position,
}

/// A badge position (x 0-14, y 0-31).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Horizontal offset.
    pub x: u8,
    /// Vertical offset.
    pub y: u8,
}

/// The order of the five 16-byte texture-name fields and their TOML keys.
pub const TEXTURE_NAME_FIELDS: [&str; 5] = ["kit", "back", "chest", "leg", "name"];
