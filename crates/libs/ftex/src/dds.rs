//! The classic 128-byte DDS header and the 20-byte DX10 extension.

use binrw::{BinRead, BinWrite};

/// The 128-byte DDS header (little-endian), including magic.
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub(crate) struct DdsHeader {
    pub magic: [u8; 4],
    pub header_size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mipmap_count: u32,
    #[brw(pad_before = 44)]
    pub pixel_format_size: u32,
    pub format_flags: u32,
    pub fourcc: [u8; 4],
    pub rgb_bit_count: u32,
    pub r_mask: u32,
    pub g_mask: u32,
    pub b_mask: u32,
    pub a_mask: u32,
    pub capabilities1: u32,
    #[brw(pad_after = 12)]
    pub capabilities2: u32,
}

/// The 20-byte DX10 extension header that follows a `DX10` FourCC.
#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub(crate) struct Dx10Header {
    pub dxgi_format: u32,
    pub dimension: u32,
    pub misc_flags: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}
