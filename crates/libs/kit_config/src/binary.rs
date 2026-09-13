//! The 120-byte kit-config binary layout (kit_config_editor plan, "The
//! format"). Decode keeps every bit not yet decoded in
//! `KitConfig::unknown` (per byte offset, known bits zeroed); encode writes
//! the known fields and ORs the remainders back, so real configs round-trip
//! bit-identically. Values wider than their field are clamped to the
//! version's `field_limits` maximum on encode; `validate` reports them with
//! the same table.

use std::collections::BTreeMap;

use pes_version::PesVersion;

use crate::KitConfigError;
use crate::model::*;

/// The bundled template every generated config starts from.
const TEMPLATE_BYTES: &[u8; 120] =
    include_bytes!("../tests/fixtures/template_XXX_DEF_xxx_realUni.bin");

/// The template config, decoded for PES 21 on first use.
pub fn template() -> KitConfig {
    use std::sync::OnceLock;
    static TEMPLATE: OnceLock<KitConfig> = OnceLock::new();
    TEMPLATE
        .get_or_init(|| {
            decode(TEMPLATE_BYTES, PesVersion::Pes21).expect("bundled template must decode")
        })
        .clone()
}

/// Decodes a kit config; WESYS-wrapped input is unwrapped first, the result
/// must be exactly 120 bytes.
pub fn decode(bytes: &[u8], version: PesVersion) -> Result<KitConfig, KitConfigError> {
    let bytes = wezlib::decompress_if_wrapped(bytes)?;
    let bytes: &[u8] = bytes.as_ref();
    if bytes.len() != 120 {
        return Err(KitConfigError::WrongLength(bytes.len()));
    }

    let mut unknown = BTreeMap::new();
    let mut keep_unknown = |offset: u8, byte: u8, mask: u8| {
        let remainder = byte & mask;
        if remainder != 0 {
            unknown.insert(offset, remainder);
        }
    };

    let short_sleeves = match bytes[0x00] & 0x3 {
        1 => ShortSleeves::Normal,
        2 => ShortSleeves::CutOut,
        other => ShortSleeves::Raw(other),
    };
    keep_unknown(0x00, bytes[0x00], 0xFC);

    let long_sleeves = match bytes[0x02] {
        0x3E => LongSleeves::Normal,
        0xBB => LongSleeves::UndershirtOnly,
        other => LongSleeves::Raw(other),
    };

    keep_unknown(0x13, bytes[0x13], 0xFF);

    // Shorts number: Y at 0x16[1-4], X = 0x17[0-1] << 2 | 0x16[6-7],
    // size 0x17[3-6], side 0x17[7].
    keep_unknown(0x16, bytes[0x16], 0x21);
    let shorts_number = ShortsNumber {
        y: (bytes[0x16] >> 1) & 0xF,
        x: ((bytes[0x17] & 0x3) << 2) | (bytes[0x16] >> 6),
        size: (bytes[0x17] >> 3) & 0xF,
        side: if bytes[0x17] & 0x80 != 0 {
            Side::Right
        } else {
            Side::Left
        },
    };
    keep_unknown(0x17, bytes[0x17], 0x04);

    // Back number: Y 0x18[0-4], size = 0x19[0-1] << 2 | 0x18[6-7],
    // spacing 0x19[4-5].
    keep_unknown(0x18, bytes[0x18], 0x20);
    let back_number = BackNumber {
        y: bytes[0x18] & 0x1F,
        size: ((bytes[0x19] & 0x3) << 2) | (bytes[0x18] >> 6),
        spacing: (bytes[0x19] >> 4) & 0x3,
    };
    keep_unknown(0x19, bytes[0x19], 0xCC);

    // Chest number: Y 0x1A[0-3], X 0x1A[4-7], size 0x1B[0-3], tight 0x1B[7].
    let chest_number = ChestNumber {
        y: bytes[0x1A] & 0xF,
        x: bytes[0x1A] >> 4,
        size: bytes[0x1B] & 0xF,
    };
    let tight = bytes[0x1B] & 0x80 != 0;
    keep_unknown(0x1B, bytes[0x1B], 0x70);

    // Name Y spans 0x1C/0x1D and differs by version: 5 bits on PES <= 20,
    // 6 bits on PES 21 (which claims 0x1C bit 3 as data).
    let (name_y, unknown_1c_mask) = if version >= PesVersion::Pes21 {
        (
            ((bytes[0x1D] & 0x1) << 5) | ((bytes[0x1C] >> 3) & 0x1F),
            0x07u8,
        )
    } else {
        (((bytes[0x1D] & 0x1) << 4) | (bytes[0x1C] >> 4), 0x0Fu8)
    };
    keep_unknown(0x1C, bytes[0x1C], unknown_1c_mask);
    let name = NameText {
        show: bytes[0x1E] & 0x1 == 0,
        shape: match bytes[0x1D] >> 6 {
            1 => NameShape::LightCurve,
            2 => NameShape::MediumCurve,
            3 => NameShape::ExtremeCurve,
            _ => NameShape::Straight,
        },
        y: name_y,
        size: (bytes[0x1D] >> 1) & 0x1F,
    };

    // Sleeve badges.
    keep_unknown(0x1E, bytes[0x1E], 0x02);
    keep_unknown(0x1F, bytes[0x1F], 0x08);
    keep_unknown(0x20, bytes[0x20], 0x20);
    keep_unknown(0x21, bytes[0x21], 0x80);
    keep_unknown(0x23, bytes[0x23], 0xFE);
    let badges = Badges {
        left_short: Position {
            x: (bytes[0x1E] >> 2) & 0xF,
            y: ((bytes[0x1F] & 0x7) << 2) | (bytes[0x1E] >> 6),
        },
        right_short: Position {
            x: bytes[0x1F] >> 4,
            y: bytes[0x20] & 0x1F,
        },
        left_long: Position {
            x: ((bytes[0x21] & 0x3) << 2) | (bytes[0x20] >> 6),
            y: (bytes[0x21] >> 2) & 0x1F,
        },
        right_long: Position {
            x: bytes[0x22] & 0xF,
            y: ((bytes[0x23] & 0x1) << 4) | (bytes[0x22] >> 4),
        },
    };

    keep_unknown(0x24, bytes[0x24], 0x0F);
    keep_unknown(0x25, bytes[0x25], 0xFF);
    keep_unknown(0x26, bytes[0x26], 0xFF);
    keep_unknown(0x27, bytes[0x27], 0xFF);

    let mut names = [[0u8; 16]; 5];
    for (i, name_field) in names.iter_mut().enumerate() {
        name_field.copy_from_slice(&bytes[0x28 + 16 * i..0x38 + 16 * i]);
    }

    Ok(KitConfig {
        shirt: Shirt {
            model: bytes[0x01],
            collar: bytes[0x14],
            winter_collar: bytes[0x15],
            tight,
            pattern: bytes[0x24] >> 4,
            long_sleeves,
            short_sleeves,
        },
        shorts: Shorts { model: bytes[0x03] },
        colors: Colors {
            shirt1: Rgb(bytes[0x04], bytes[0x05], bytes[0x06]),
            shirt2: Rgb(bytes[0x07], bytes[0x08], bytes[0x09]),
            undershirt: Rgb(bytes[0x0A], bytes[0x0B], bytes[0x0C]),
            shorts: Rgb(bytes[0x0D], bytes[0x0E], bytes[0x0F]),
            socks: Rgb(bytes[0x10], bytes[0x11], bytes[0x12]),
        },
        name,
        numbers: Numbers {
            back: back_number,
            chest: chest_number,
            shorts: shorts_number,
        },
        badges,
        unknown,
        source_texture_names: Some(names),
    })
}

/// Encodes the config, taking texture names from `source_texture_names` or
/// zeros when `None`.
pub fn encode(config: &KitConfig, version: PesVersion) -> [u8; 120] {
    let zeros = [[0u8; 16]; 5];
    let names = config.source_texture_names.as_ref().unwrap_or(&zeros);
    encode_with_names(config, version, names)
}

/// Encodes the config with the given five texture-name fields.
pub fn encode_with_names(
    config: &KitConfig,
    version: PesVersion,
    names: &[[u8; 16]; 5],
) -> [u8; 120] {
    // One table drives the clamp and `validate`'s finding; every field named
    // below is a member of `field_limits`, so `unwrap_or(u8::MAX)` never
    // applies.
    let limits = field_limits(version);
    let limit = |field: &str| -> u8 {
        limits
            .iter()
            .find(|limit| limit.field == field)
            .map_or(u8::MAX, |limit| limit.max)
    };
    let mut bytes = [0u8; 120];
    let apply_unknown = |bytes: &mut [u8; 120], offset: usize, mask: u8| {
        if let Some(remainder) = config.unknown.get(&(offset as u8)) {
            bytes[offset] |= remainder & mask;
        }
    };

    bytes[0x00] = match config.shirt.short_sleeves {
        ShortSleeves::Normal => 1,
        ShortSleeves::CutOut => 2,
        ShortSleeves::Raw(value) => value & 0x3,
    };
    apply_unknown(&mut bytes, 0x00, 0xFC);

    bytes[0x01] = config.shirt.model;
    bytes[0x02] = match config.shirt.long_sleeves {
        LongSleeves::Normal => 0x3E,
        LongSleeves::UndershirtOnly => 0xBB,
        LongSleeves::Raw(value) => value,
    };
    bytes[0x03] = config.shorts.model;

    for (offset, rgb) in [
        (0x04, config.colors.shirt1),
        (0x07, config.colors.shirt2),
        (0x0A, config.colors.undershirt),
        (0x0D, config.colors.shorts),
        (0x10, config.colors.socks),
    ] {
        bytes[offset] = rgb.0;
        bytes[offset + 1] = rgb.1;
        bytes[offset + 2] = rgb.2;
    }

    apply_unknown(&mut bytes, 0x13, 0xFF);
    bytes[0x14] = config.shirt.collar;
    bytes[0x15] = config.shirt.winter_collar;

    let shorts_x = config.numbers.shorts.x.min(limit("number.shorts.x"));
    bytes[0x16] =
        (config.numbers.shorts.y.min(limit("number.shorts.y")) << 1) | ((shorts_x & 0x3) << 6);
    bytes[0x17] = ((shorts_x >> 2) & 0x3)
        | (config.numbers.shorts.size.min(limit("number.shorts.size")) << 3)
        | match config.numbers.shorts.side {
            Side::Left => 0,
            Side::Right => 0x80,
        };
    apply_unknown(&mut bytes, 0x16, 0x21);
    apply_unknown(&mut bytes, 0x17, 0x04);

    let back_size = config.numbers.back.size.min(limit("number.back.size"));
    bytes[0x18] = config.numbers.back.y.min(limit("number.back.y")) | ((back_size & 0x3) << 6);
    bytes[0x19] = ((back_size >> 2) & 0x3)
        | (config
            .numbers
            .back
            .spacing
            .min(limit("number.back.spacing"))
            << 4);
    apply_unknown(&mut bytes, 0x18, 0x20);
    apply_unknown(&mut bytes, 0x19, 0xCC);

    bytes[0x1A] = config.numbers.chest.y.min(limit("number.chest.y"))
        | (config.numbers.chest.x.min(limit("number.chest.x")) << 4);
    bytes[0x1B] = config.numbers.chest.size.min(limit("number.chest.size"))
        | (if config.shirt.tight { 0x80 } else { 0 });
    apply_unknown(&mut bytes, 0x1B, 0x70);

    // The game bounds Name Y at 0-16 on PES <= 20 and 0-39 on PES 21
    // (`limit("name.y")`), inside a 5-bit / 6-bit field.
    let name_y = config.name.y.min(limit("name.y"));
    let name_size = config.name.size.min(limit("name.size"));
    if version >= PesVersion::Pes21 {
        bytes[0x1C] = (name_y & 0x1F) << 3;
        bytes[0x1D] =
            ((name_y >> 5) & 0x1) | (name_size << 1) | (name_shape_bits(config.name.shape) << 6);
        apply_unknown(&mut bytes, 0x1C, 0x07);
    } else {
        bytes[0x1C] = (name_y & 0xF) << 4;
        bytes[0x1D] =
            ((name_y >> 4) & 0x1) | (name_size << 1) | (name_shape_bits(config.name.shape) << 6);
        apply_unknown(&mut bytes, 0x1C, 0x0F);
    }

    let left_short_y = config.badges.left_short.y.min(limit("badge.left_short.y"));
    bytes[0x1E] = (if config.name.show { 0 } else { 1 })
        | (config.badges.left_short.x.min(limit("badge.left_short.x")) << 2)
        | ((left_short_y & 0x3) << 6);
    bytes[0x1F] = ((left_short_y >> 2) & 0x7)
        | (config
            .badges
            .right_short
            .x
            .min(limit("badge.right_short.x"))
            << 4);
    let left_long_x = config.badges.left_long.x.min(limit("badge.left_long.x"));
    bytes[0x20] = config
        .badges
        .right_short
        .y
        .min(limit("badge.right_short.y"))
        | ((left_long_x & 0x3) << 6);
    bytes[0x21] = ((left_long_x >> 2) & 0x3)
        | (config.badges.left_long.y.min(limit("badge.left_long.y")) << 2);
    bytes[0x22] = config.badges.right_long.x.min(limit("badge.right_long.x"))
        | ((config.badges.right_long.y.min(limit("badge.right_long.y")) & 0xF) << 4);
    bytes[0x23] = (config.badges.right_long.y.min(limit("badge.right_long.y")) >> 4) & 0x1;
    apply_unknown(&mut bytes, 0x1E, 0x02);
    apply_unknown(&mut bytes, 0x1F, 0x08);
    apply_unknown(&mut bytes, 0x20, 0x20);
    apply_unknown(&mut bytes, 0x21, 0x80);
    apply_unknown(&mut bytes, 0x23, 0xFE);

    bytes[0x24] = config.shirt.pattern.min(limit("shirt.pattern")) << 4;
    apply_unknown(&mut bytes, 0x24, 0x0F);
    if version == PesVersion::Pes15 && bytes[0x24] >> 5 == 0b110 {
        // PES 15 reads only bits 5-7 of the pattern byte (a 3-bit index,
        // 0-5 valid) where later versions read bits 4-7; 4-bit values 12-13
        // (3-bit 6, which PES 15 has no pattern for) map to 10-11 (3-bit 5),
        // and 14-15 are emitted as is.
        bytes[0x24] = (bytes[0x24] & 0x1F) | (0b101 << 5);
    }
    apply_unknown(&mut bytes, 0x25, 0xFF);
    apply_unknown(&mut bytes, 0x26, 0xFF);
    apply_unknown(&mut bytes, 0x27, 0xFF);

    for (i, name) in names.iter().enumerate() {
        bytes[0x28 + 16 * i..0x38 + 16 * i].copy_from_slice(name);
    }

    bytes
}

fn name_shape_bits(shape: NameShape) -> u8 {
    match shape {
        NameShape::Straight => 0,
        NameShape::LightCurve => 1,
        NameShape::MediumCurve => 2,
        NameShape::ExtremeCurve => 3,
    }
}
