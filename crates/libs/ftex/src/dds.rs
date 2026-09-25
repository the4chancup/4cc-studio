//! The classic 128-byte DDS header, the 20-byte DX10 extension, and the
//! FourCC/DXGI/channel-mask table that maps a DDS pixel format onto the
//! formats FTEX knows. Shared by both conversion directions and by crates
//! that only need to walk a DDS's mips.

use std::io::Cursor;

use binrw::{BinRead, BinWrite};

use crate::FtexError;
use crate::format::{PixelFormat, mip_size};

/// The 128-byte DDS header (little-endian), including magic.
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub(crate) struct DdsHeader {
    /// `b"DDS "`.
    pub magic: [u8; 4],
    /// Always 124.
    pub header_size: u32,
    /// `DDSD_*` capability flags.
    pub flags: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image width in pixels.
    pub width: u32,
    /// Pitch for uncompressed formats, level-0 byte size for compressed ones.
    pub pitch_or_linear_size: u32,
    /// Volume depth (1 for a plain 2D texture).
    pub depth: u32,
    /// Number of mip levels the file carries.
    pub mipmap_count: u32,
    /// Always 32.
    #[brw(pad_before = 44)]
    pub pixel_format_size: u32,
    /// `DDPF_*` flags: 0x4 = FourCC, 0x40 = RGB, 0x1 = alpha pixels, 0x20000 = luminance.
    pub format_flags: u32,
    /// The FourCC code (`DXT1`, `DX10`, ...), zero when `format_flags` has no 0x4.
    pub fourcc: [u8; 4],
    /// Bits per pixel for uncompressed formats.
    pub rgb_bit_count: u32,
    /// Red channel mask.
    pub r_mask: u32,
    /// Green channel mask.
    pub g_mask: u32,
    /// Blue channel mask.
    pub b_mask: u32,
    /// Alpha channel mask (0 means no alpha).
    pub a_mask: u32,
    /// `DDSCAPS_*`: 0x1000 texture, 0x8 complex, 0x400000 mipmap.
    pub capabilities1: u32,
    /// `DDSCAPS2_*`: 0x200 cube map, 0xfe00 the six faces, 0x200000 volume.
    #[brw(pad_after = 12)]
    pub capabilities2: u32,
}

/// The 20-byte DX10 extension header that follows a `DX10` FourCC.
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub(crate) struct Dx10Header {
    /// The `DXGI_FORMAT` number.
    pub dxgi_format: u32,
    /// `D3D10_RESOURCE_DIMENSION` (3 = 2D, 4 = 3D/volume).
    pub dimension: u32,
    /// 0x4 marks a cube map.
    pub misc_flags: u32,
    /// Array size; 1 for every texture this crate reads.
    pub array_size: u32,
    /// Alpha mode flags.
    pub misc_flags2: u32,
}

/// How the pixels of a DDS are stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdsPixel {
    /// A format FTEX also knows (BC1..BC7, `Argb8` = B,G,R,A byte order, `R8`,
    /// and the float formats).
    Format(PixelFormat),
    /// Any other byte-aligned uncompressed layout, described by its channel
    /// masks. DXGI 28/29 (R8G8B8A8) arrives as `bit_count: 32`,
    /// `r: 0xff, g: 0xff00, b: 0xff0000, a: 0xff000000`.
    Uncompressed {
        /// Bits per pixel.
        bit_count: u32,
        /// Red channel mask.
        r_mask: u32,
        /// Green channel mask.
        g_mask: u32,
        /// Blue channel mask.
        b_mask: u32,
        /// Alpha channel mask (0 means the pixels are opaque).
        a_mask: u32,
    },
}

impl DdsPixel {
    /// Bytes one tightly packed row of `width` pixels occupies in the
    /// stream, for the layouts that store rows (`Uncompressed`, `Argb8`,
    /// `R8`). `None` for the block and float formats, whose stream size is
    /// counted in blocks, not rows.
    pub fn row_bytes(self, width: u32) -> Option<u64> {
        Some(match self {
            DdsPixel::Uncompressed { bit_count, .. } => u64::from(width) * u64::from(bit_count) / 8,
            DdsPixel::Format(PixelFormat::Argb8) => u64::from(width) * 4,
            DdsPixel::Format(PixelFormat::R8) => u64::from(width),
            DdsPixel::Format(_) => return None,
        })
    }
}

/// The layout of a 2D DDS: enough to walk its mip chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DdsLayout {
    /// The pixel format.
    pub pixel: DdsPixel,
    /// Level-0 width in pixels.
    pub width: u32,
    /// Level-0 height in pixels.
    pub height: u32,
    /// Number of mip levels the file carries (at least 1).
    pub mipmaps: u32,
    /// Offset of the first mip's data (128, or 148 with a DX10 extension).
    pub data_offset: usize,
    /// The level-0 row pitch the header declares for an uncompressed layout
    /// (`DDSD_PITCH`), when it is wider than the tightly packed row: some
    /// exporters pad rows to 4 bytes. `None` means rows are tightly packed.
    pub row_pitch: Option<u32>,
}

/// The `DDSCAPS2` bit that marks a cube map.
const CAPS2_CUBE: u32 = 0x200;
/// The `DDSCAPS2` bit that marks a volume texture.
const CAPS2_VOLUME: u32 = 0x200000;
/// The `DDSD` flag saying `pitch_or_linear_size` is a row pitch.
pub(crate) const DDSD_PITCH: u32 = 0x8;
/// The DX10 `misc_flags` bit that marks a cube map.
const DX10_MISC_CUBE: u32 = 0x4;

/// Parses the DDS header(s) of a 2D texture. Cube maps and volume textures
/// are `FtexError::UnsupportedDds("cube map")` / `("volume texture")`. A
/// missing mipmap flag or a count of 0 means 1 mip. sRGB DXGI ids map to
/// their UNORM twins. A zero dimension, and a mip count the dimensions
/// cannot halve into, are refused: D3D rejects both at texture creation.
pub fn read_layout(dds: &[u8]) -> Result<DdsLayout, FtexError> {
    let mut cursor = Cursor::new(dds);
    let header = DdsHeader::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
    if header.magic != *b"DDS " {
        return Err(FtexError::BadMagic);
    }
    if header.header_size != 124 {
        return Err(FtexError::UnsupportedDds("header size != 124"));
    }
    if header.capabilities2 & CAPS2_CUBE != 0 {
        return Err(FtexError::UnsupportedDds("cube map"));
    }
    if header.capabilities2 & CAPS2_VOLUME != 0 {
        return Err(FtexError::UnsupportedDds("volume texture"));
    }
    if header.width == 0 || header.height == 0 {
        return Err(FtexError::UnsupportedDds("zero dimension"));
    }
    let mipmaps = mip_count(&header);
    // A mip halves the larger side; the count that bottoms out at 1x1 is
    // log2(max side) + 1 = 32 - leading_zeros.
    if mipmaps > 32 - header.width.max(header.height).leading_zeros() {
        return Err(FtexError::UnsupportedDds("mip count past the dimensions"));
    }

    let mut data_offset = 128usize;
    let pixel = if header.format_flags & 0x4 == 0 {
        uncompressed_pixel(&header)?
    } else {
        match &header.fourcc {
            b"DX10" => {
                let ext = Dx10Header::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
                if ext.dimension == 4 || header.depth > 1 {
                    return Err(FtexError::UnsupportedDds("volume texture"));
                }
                if ext.misc_flags & DX10_MISC_CUBE != 0 {
                    return Err(FtexError::UnsupportedDds("cube map"));
                }
                // The low three bits of misc_flags2 are the alpha mode;
                // 2 (premultiplied) has no straight-alpha decode.
                if ext.misc_flags2 & 0x7 == 2 {
                    return Err(FtexError::UnsupportedDds("premultiplied alpha"));
                }
                single_image(&ext)?;
                data_offset = 148;
                dxgi_pixel(ext.dxgi_format)?
            }
            _ => DdsPixel::Format(
                fourcc_format(&header.fourcc)
                    .ok_or(FtexError::UnsupportedDds("unrecognized fourcc"))?,
            ),
        }
    };

    // The declared pitch counts only when it is wider than a tight row.
    let row_pitch = pixel
        .row_bytes(header.width)
        .filter(|tight| {
            header.flags & DDSD_PITCH != 0 && u64::from(header.pitch_or_linear_size) > *tight
        })
        .map(|_| header.pitch_or_linear_size);
    Ok(DdsLayout {
        pixel,
        width: header.width,
        height: header.height,
        mipmaps,
        data_offset,
        row_pitch,
    })
}

/// The mip count a header declares: the `DDSCAPS_MIPMAP` bit gates the
/// count field, so without it the texture has one level whatever the field
/// says.
pub(crate) fn mip_count(header: &DdsHeader) -> u32 {
    if header.capabilities1 & 0x400000 != 0 {
        header.mipmap_count.max(1)
    } else {
        1
    }
}

/// Rejects a DX10 header that describes more than one image: texture arrays
/// have no FTEX form and nothing in an export is one.
pub(crate) fn single_image(ext: &Dx10Header) -> Result<(), FtexError> {
    if ext.array_size > 1 {
        return Err(FtexError::UnsupportedDds("texture array"));
    }
    Ok(())
}

/// The uncompressed (no FourCC) pixel description: the mask layouts FTEX
/// knows become `DdsPixel::Format`, everything else stays masks.
pub(crate) fn uncompressed_pixel(header: &DdsHeader) -> Result<DdsPixel, FtexError> {
    if header.format_flags & 0x40 != 0
        && header.format_flags & 0x1 != 0
        && header.r_mask == 0x00ff0000
        && header.g_mask == 0x0000ff00
        && header.b_mask == 0x000000ff
        && header.a_mask == 0xff000000
    {
        return Ok(DdsPixel::Format(PixelFormat::Argb8));
    }
    if header.format_flags & 0x20000 != 0
        && header.r_mask == 0xff
        && header.g_mask == 0
        && header.b_mask == 0
        && header.a_mask == 0
    {
        return Ok(DdsPixel::Format(PixelFormat::R8));
    }
    Ok(DdsPixel::Uncompressed {
        bit_count: header.rgb_bit_count,
        r_mask: header.r_mask,
        g_mask: header.g_mask,
        b_mask: header.b_mask,
        a_mask: header.a_mask,
    })
}

/// The legacy FourCC -> pixel-format table (DX10 handled by its caller).
pub(crate) fn fourcc_format(fourcc: &[u8; 4]) -> Option<PixelFormat> {
    Some(match fourcc {
        b"8888" => PixelFormat::Argb8,
        b"DXT1" => PixelFormat::Bc1,
        b"DXT3" => PixelFormat::Bc2,
        b"DXT5" => PixelFormat::Bc3,
        b"ATI1" => PixelFormat::Bc4,
        b"ATI2" | b"BC5U" => PixelFormat::Bc5,
        _ => return None,
    })
}

/// The DX10 DXGI-format -> pixel table; sRGB ids map to their UNORM twins
/// and 87 (B8G8R8A8) maps to Argb8, as FTEX stores it. DXGI 28/29
/// (R8G8B8A8) and 88/92 (B8G8R8X8, whose fourth byte is not alpha) have no
/// FTEX twin and stay masks. The signed block formats (81 BC4_SNORM, 84
/// BC5_SNORM, 96 BC6H_SF16) are refused: their blocks mean different values,
/// so relabelling them unsigned would silently change the texture.
pub(crate) fn dxgi_pixel(dxgi: u32) -> Result<DdsPixel, FtexError> {
    let format = match dxgi {
        87 | 91 => PixelFormat::Argb8,
        61 => PixelFormat::R8,
        71 | 72 => PixelFormat::Bc1,
        74 | 75 => PixelFormat::Bc2,
        77 | 78 => PixelFormat::Bc3,
        80 => PixelFormat::Bc4,
        83 => PixelFormat::Bc5,
        95 => PixelFormat::Bc6h,
        98 | 99 => PixelFormat::Bc7,
        81 | 84 | 96 => return Err(FtexError::UnsupportedDds("signed block format")),
        88 | 92 => {
            return Ok(DdsPixel::Uncompressed {
                bit_count: 32,
                r_mask: 0xff0000,
                g_mask: 0xff00,
                b_mask: 0xff,
                a_mask: 0,
            });
        }
        10 => PixelFormat::Rgba16F,
        2 => PixelFormat::Rgba32F,
        24 => PixelFormat::Rgb10A2,
        26 => PixelFormat::Rg11B10F,
        28 | 29 => {
            return Ok(DdsPixel::Uncompressed {
                bit_count: 32,
                r_mask: 0xff,
                g_mask: 0xff00,
                b_mask: 0xff0000,
                a_mask: 0xff000000,
            });
        }
        _ => return Err(FtexError::UnsupportedDds("unrecognized dxgi format")),
    };
    Ok(DdsPixel::Format(format))
}

/// Builds the DDS header bytes `ftex_to_dds` emits: the 128-byte header plus
/// the DX10 extension for the DXGI-mapped formats. `cube` writes the six-face
/// capabilities and the DX10 cube flag; `depth` above 1 writes the volume
/// fields.
pub(crate) fn build_header(
    format: PixelFormat,
    width: u32,
    height: u32,
    mipmaps: u32,
    depth: u32,
    cube: bool,
) -> Vec<u8> {
    let mut dds_flags = 0x1 | 0x2 | 0x4 | 0x1000 | 0x20000;
    // capabilities, complex, mipmap; a cube map adds no further caps1 bits.
    let caps1 = 0x1000 | 0x8 | 0x400000;
    let mut caps2 = 0u32;
    let (ext_dimension, ext_flags) = if cube {
        caps2 |= 0xfe00;
        (3u32, 0x4u32)
    } else if depth > 1 {
        dds_flags |= 0x800000;
        caps2 |= 0x200000;
        (4, 0)
    } else {
        (3, 0)
    };

    let (pitch_or_linear, format_flags, fourcc, rgb_bit_count, masks, dx10) =
        if format == PixelFormat::Argb8 {
            dds_flags |= 0x8; // pitch
            (
                4 * width,
                0x41u32,
                [0u8; 4],
                32u32,
                [0x00ff0000, 0x0000ff00, 0x000000ff, 0xff000000],
                None,
            )
        } else {
            dds_flags |= 0x80000;
            (
                mip_size(format, width, height, depth, 0) as u32,
                0x4u32,
                format.fourcc().unwrap_or([0; 4]),
                0u32,
                [0u32; 4],
                format.dxgi().map(|dxgi| Dx10Header {
                    dxgi_format: dxgi,
                    dimension: ext_dimension,
                    misc_flags: ext_flags,
                    array_size: 1,
                    misc_flags2: 0,
                }),
            )
        };

    let dds = DdsHeader {
        magic: *b"DDS ",
        header_size: 124,
        flags: dds_flags,
        height,
        width,
        pitch_or_linear_size: pitch_or_linear,
        depth,
        mipmap_count: mipmaps,
        pixel_format_size: 32,
        format_flags,
        fourcc,
        rgb_bit_count,
        r_mask: masks[0],
        g_mask: masks[1],
        b_mask: masks[2],
        a_mask: masks[3],
        capabilities1: caps1,
        capabilities2: caps2,
    };

    let mut writer = Cursor::new(Vec::new());
    dds.write(&mut writer)
        .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    if let Some(dx10) = dx10 {
        dx10.write(&mut writer)
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    }
    writer.into_inner()
}

/// The header `ftex_to_dds` writes for a 2D, depth-1, non-cube texture:
/// legacy FourCC headers for BC1/BC2/BC3 and Argb8, DX10 for the rest.
pub fn header_bytes(format: PixelFormat, width: u32, height: u32, mipmaps: u32) -> Vec<u8> {
    build_header(format, width, height, mipmaps, 1, false)
}
