//! FTEX/DDS shared records and the pixel-format table.

use binrw::{BinRead, BinWrite};

/// FTEX pixel format ids with their DDS mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 0, uncompressed A8R8G8B8 (legacy FourCC-less DDS).
    Argb8,
    /// 1, DXGI_FORMAT_R8_UNORM.
    R8,
    /// 2, BC1 (`DXT1`).
    Bc1,
    /// 3, BC2 (`DXT3`).
    Bc2,
    /// 4, BC3 (`DXT5`).
    Bc3,
    /// 8, BC4 (DXGI 80 / `ATI1`).
    Bc4,
    /// 9, BC5 (DXGI 83 / `ATI2` / `BC5U`).
    Bc5,
    /// 10, BC6H_UF16 (DXGI 95).
    Bc6h,
    /// 11, BC7 (DXGI 98).
    Bc7,
    /// 12, DXGI_FORMAT_R16G16B16A16_FLOAT (10).
    Rgba16F,
    /// 13, DXGI_FORMAT_R32G32B32A32_FLOAT (2).
    Rgba32F,
    /// 14, DXGI_FORMAT_R10G10B10A2_UNORM (24).
    Rgb10A2,
    /// 15, DXGI_FORMAT_R11G11B10_FLOAT (26).
    Rg11B10F,
}

impl PixelFormat {
    /// The id stored in the FTEX header.
    pub(crate) fn id(self) -> u16 {
        match self {
            PixelFormat::Argb8 => 0,
            PixelFormat::R8 => 1,
            PixelFormat::Bc1 => 2,
            PixelFormat::Bc2 => 3,
            PixelFormat::Bc3 => 4,
            PixelFormat::Bc4 => 8,
            PixelFormat::Bc5 => 9,
            PixelFormat::Bc6h => 10,
            PixelFormat::Bc7 => 11,
            PixelFormat::Rgba16F => 12,
            PixelFormat::Rgba32F => 13,
            PixelFormat::Rgb10A2 => 14,
            PixelFormat::Rg11B10F => 15,
        }
    }

    /// The format for an FTEX id, if known.
    pub(crate) fn from_id(id: u16) -> Option<Self> {
        Some(match id {
            0 => PixelFormat::Argb8,
            1 => PixelFormat::R8,
            2 => PixelFormat::Bc1,
            3 => PixelFormat::Bc2,
            4 => PixelFormat::Bc3,
            8 => PixelFormat::Bc4,
            9 => PixelFormat::Bc5,
            10 => PixelFormat::Bc6h,
            11 => PixelFormat::Bc7,
            12 => PixelFormat::Rgba16F,
            13 => PixelFormat::Rgba32F,
            14 => PixelFormat::Rgb10A2,
            15 => PixelFormat::Rg11B10F,
            _ => return None,
        })
    }

    /// (Pixels per block side, bytes per block) used by [`mip_size`]. A block
    /// side of 1 means uncompressed.
    pub(crate) fn block_size(self) -> (u32, u32) {
        match self {
            PixelFormat::Argb8 => (1, 4),
            PixelFormat::R8 => (1, 1),
            PixelFormat::Bc1 | PixelFormat::Bc4 => (4, 8),
            PixelFormat::Bc2
            | PixelFormat::Bc3
            | PixelFormat::Bc5
            | PixelFormat::Bc6h
            | PixelFormat::Bc7 => (4, 16),
            PixelFormat::Rgba16F => (1, 8),
            PixelFormat::Rgba32F => (1, 16),
            PixelFormat::Rgb10A2 | PixelFormat::Rg11B10F => (1, 4),
        }
    }

    /// The DXGI format number for the DX10 extension header, or `None` for the
    /// FourCC-only legacy formats (Argb8, Bc1, Bc2, Bc3).
    pub(crate) fn dxgi(self) -> Option<u32> {
        Some(match self {
            PixelFormat::Argb8 | PixelFormat::Bc1 | PixelFormat::Bc2 | PixelFormat::Bc3 => {
                return None;
            }
            PixelFormat::R8 => 61,
            PixelFormat::Bc4 => 80,
            PixelFormat::Bc5 => 83,
            PixelFormat::Bc6h => 95,
            PixelFormat::Bc7 => 98,
            PixelFormat::Rgba16F => 10,
            PixelFormat::Rgba32F => 2,
            PixelFormat::Rgb10A2 => 24,
            PixelFormat::Rg11B10F => 26,
        })
    }

    /// The DDS pixel-format FourCC: `DXT1`/`DXT3`/`DXT5` for the legacy BCn
    /// formats, `DX10` for the DXGI-mapped ones, `None` for Argb8 (which
    /// writes zero FourCC and mask fields instead).
    pub(crate) fn fourcc(self) -> Option<[u8; 4]> {
        Some(match self {
            PixelFormat::Argb8 => return None,
            PixelFormat::Bc1 => *b"DXT1",
            PixelFormat::Bc2 => *b"DXT3",
            PixelFormat::Bc3 => *b"DXT5",
            _ => *b"DX10",
        })
    }
}

/// The texture's intended color space, mapped onto the FTEX texture-type
/// field (`0x1` / `0x3` / `0x9`; cube maps OR in `0x4` separately).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// Texture type 0x1.
    Linear,
    /// Texture type 0x3.
    Srgb,
    /// Texture type 0x9, used for normal maps.
    Normal,
}

impl ColorSpace {
    pub(crate) fn texture_type(self) -> u32 {
        match self {
            ColorSpace::Linear => 0x1,
            ColorSpace::Srgb => 0x3,
            ColorSpace::Normal => 0x9,
        }
    }
}

/// What the FTEX header alone says about a texture: the fields texture checks
/// need without reading frame data.
#[derive(Debug, Clone, PartialEq)]
pub struct FtexInfo {
    /// Header version (2.03x, 2.04x).
    pub version: f32,
    /// Pixel format.
    pub format: PixelFormat,
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// Depth (>1 = volume texture).
    pub depth: u16,
    /// Mipmap levels per image.
    pub mipmaps: u8,
    /// Raw texture-type field (color space plus the 0x4 cube-map bit).
    pub texture_type: u32,
    /// Whether the 0x4 texture-type bit is set (six images of mipmaps).
    pub is_cube_map: bool,
}

/// Bytes one mip level of one image occupies in the DDS stream
/// (block-rounded, at least one block per side). Dimensions shifted down
/// past level 31 read as 0 and clamp to 1, the same result the reference's
/// `x // 2**j` produces.
pub fn mip_size(format: PixelFormat, width: u32, height: u32, depth: u32, level: u32) -> usize {
    let (block_pixels, block_bytes) = format.block_size();
    let w = width.checked_shr(level).unwrap_or(0).max(1);
    let h = height.checked_shr(level).unwrap_or(0).max(1);
    let d = depth.checked_shr(level).unwrap_or(0).max(1);
    let blocks_w = w.div_ceil(block_pixels);
    let blocks_h = h.div_ceil(block_pixels);
    // A size that does not fit usize cannot be held in memory, so the slice
    // lookup it bounds fails as `Truncated`.
    usize::try_from(
        u64::from(blocks_w) * u64::from(blocks_h) * u64::from(d) * u64::from(block_bytes),
    )
    .unwrap_or(usize::MAX)
}

/// The 64-byte FTEX header (little-endian).
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub(crate) struct FtexHeader {
    pub magic: [u8; 4],
    pub version: f32,
    pub pixel_format: u16,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub mipmap_count: u8,
    pub nrt: u8,
    pub flags: u16,
    pub unknown1: u32,
    pub unknown2: u32,
    pub texture_type: u32,
    pub ftexs_count: u8,
    pub unknown3: u8,
    #[brw(pad_before = 14)]
    pub hash1: [u8; 8],
    pub hash2: [u8; 8],
}

/// One 16-byte mip record per frame.
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub(crate) struct MipRecord {
    pub offset: u32,
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub index: u8,
    pub ftexs_number: u8,
    pub chunk_count: u16,
}

/// One 8-byte chunk record inside a chunked frame.
#[derive(Debug, BinRead)]
#[brw(little)]
pub(crate) struct ChunkRecord {
    pub compressed_size: u16,
    pub uncompressed_size: u16,
    /// Offset relative to the frame start; bit 31 is ignored on read.
    pub offset: u32,
}
