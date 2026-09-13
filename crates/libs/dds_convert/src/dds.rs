//! DDS and FTEX sources -> [`Decoded`]: container walk via `ftex::dds`,
//! block decode via `block_compression`, mask decode for uncompressed
//! layouts.

use block_compression::CompressionVariant;
use block_compression::decode::decompress_blocks_as_rgba8;
use ftex::PixelFormat;
use ftex::dds::{DdsLayout, DdsPixel, read_layout};

use crate::{BlockCodec, Blocks, ConvertError, Decoded};

/// Decodes an FTEX source by converting it to DDS first; the DDS path then
/// applies unchanged.
pub(crate) fn decode_ftex(bytes: &[u8]) -> Result<Decoded, ConvertError> {
    decode_dds(&ftex::ftex_to_dds(bytes)?)
}

/// Decodes a DDS source (already unwrapped by the caller): every mip the
/// file carries, plus the raw blocks when the codec is a block format a
/// target might keep.
pub(crate) fn decode_dds(dds: &[u8]) -> Result<Decoded, ConvertError> {
    let layout = read_layout(dds)?;
    let codec = block_codec(&layout);
    let mut mips = Vec::with_capacity(layout.mipmaps as usize);
    let mut blocks = Vec::with_capacity(layout.mipmaps as usize);
    let mut offset = layout.data_offset;
    for level in 0..layout.mipmaps {
        let width = (layout.width >> level).max(1);
        let height = (layout.height >> level).max(1);
        let size = mip_size(&layout, width, height);
        let data = dds
            .get(offset..offset + size)
            .ok_or(ConvertError::Truncated)?;
        mips.push(decode_mip(&layout, codec, width, height, data)?);
        blocks.push(data.to_vec());
        offset += size;
    }

    Ok(Decoded {
        width: layout.width,
        height: layout.height,
        mips,
        blocks: codec.map(|(codec, _)| Blocks {
            codec,
            mips: blocks,
        }),
    })
}

/// Bytes one mip level occupies in the DDS stream.
fn mip_size(layout: &DdsLayout, width: u32, height: u32) -> usize {
    match layout.pixel {
        DdsPixel::Format(format) => ftex::mip_size(format, width, height, 1, 0),
        DdsPixel::Uncompressed { bit_count, .. } => (width * height * bit_count / 8) as usize,
    }
}

/// The block codec behind the layout's pixel format, when it is one this
/// crate keeps or emits, paired with its `block_compression` variant.
fn block_codec(layout: &DdsLayout) -> Option<(BlockCodec, CompressionVariant)> {
    let format = match layout.pixel {
        DdsPixel::Format(format) => format,
        DdsPixel::Uncompressed { .. } => return None,
    };
    Some(match format {
        PixelFormat::Bc1 => (BlockCodec::Bc1, CompressionVariant::BC1),
        PixelFormat::Bc2 => (BlockCodec::Bc2, CompressionVariant::BC2),
        PixelFormat::Bc3 => (BlockCodec::Bc3, CompressionVariant::BC3),
        PixelFormat::Bc4 => (BlockCodec::Bc4, CompressionVariant::BC4),
        PixelFormat::Bc5 => (BlockCodec::Bc5, CompressionVariant::BC5),
        PixelFormat::Bc7 => (
            BlockCodec::Bc7,
            CompressionVariant::BC7(block_compression::BC7Settings::alpha_basic()),
        ),
        _ => return None,
    })
}

/// One mip level to straight-alpha RGBA8.
fn decode_mip(
    layout: &DdsLayout,
    codec: Option<(BlockCodec, CompressionVariant)>,
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<u8>, ConvertError> {
    match layout.pixel {
        DdsPixel::Format(PixelFormat::Argb8) => decode_uncompressed(
            width,
            height,
            32,
            [0x00ff0000, 0x0000ff00, 0x000000ff, 0xff000000],
            data,
        ),
        DdsPixel::Format(PixelFormat::R8) => {
            decode_uncompressed(width, height, 8, [0xff, 0, 0, 0], data)
        }
        DdsPixel::Uncompressed {
            bit_count,
            r_mask,
            g_mask,
            b_mask,
            a_mask,
        } => decode_uncompressed(
            width,
            height,
            bit_count,
            [r_mask, g_mask, b_mask, a_mask],
            data,
        ),
        DdsPixel::Format(_) => {
            let (_, variant) = codec.ok_or(ConvertError::Unsupported("pixel format"))?;
            Ok(decompress_mip(variant, width, height, data))
        }
    }
}

/// Decompresses one mip's blocks. The decoder writes whole 4x4 blocks, so a
/// mip smaller than 4 on a side is decoded at its block-rounded size and
/// cropped; rows beyond the mip's own pitch are dropped.
fn decompress_mip(variant: CompressionVariant, width: u32, height: u32, blocks: &[u8]) -> Vec<u8> {
    let padded_width = width.div_ceil(4) * 4;
    let padded_height = height.div_ceil(4) * 4;
    let mut padded = vec![0u8; (padded_width * padded_height * 4) as usize];
    decompress_blocks_as_rgba8(variant, padded_width, padded_height, blocks, &mut padded);

    // The decoder truncates the interpolated index values; the reference
    // decoder rounds them to nearest, so the alpha-indexed channels (BC3
    // alpha, the BC4/BC5 data channels) are recomputed from the endpoints.
    fix_interpolated_channels(variant, padded_width, padded_height, blocks, &mut padded);

    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height as usize {
        let from = y * padded_width as usize * 4;
        let to = y * width as usize * 4;
        rgba[to..to + width as usize * 4].copy_from_slice(&padded[from..from + width as usize * 4]);
    }
    // The block decoder leaves missing channels zero; the reference decoder
    // reports the one/two-channel formats as opaque (alpha 255).
    if matches!(variant, CompressionVariant::BC4 | CompressionVariant::BC5) {
        for pixel in rgba.as_chunks_mut::<4>().0.iter_mut() {
            pixel[3] = 255;
        }
    }
    rgba
}

/// Rewrites the alpha-indexed channels of `padded` with rounded
/// interpolation: BC3's alpha and BC4/BC5's data channels use the 8-byte
/// endpoint+3-bit-index format, whose interpolated entries are
/// `(e0*(n-i) + e1*i + n/2) / n`.
fn fix_interpolated_channels(
    variant: CompressionVariant,
    padded_width: u32,
    padded_height: u32,
    blocks: &[u8],
    padded: &mut [u8],
) {
    // (block byte size, alpha-block offsets within the block, output channel
    // per alpha block)
    let (block_bytes, alpha_blocks): (usize, &[(usize, usize)]) = match variant {
        CompressionVariant::BC3 => (16, &[(0, 3)]),
        CompressionVariant::BC4 => (8, &[(0, 0)]),
        CompressionVariant::BC5 => (16, &[(0, 0), (8, 1)]),
        _ => return,
    };
    let blocks_x = padded_width / 4;
    let blocks_y = padded_height / 4;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block = &blocks[((by * blocks_x + bx) as usize * block_bytes)..];
            for &(alpha_offset, channel) in alpha_blocks {
                let alpha_block = &block[alpha_offset..alpha_offset + 8];
                let ramp = alpha_ramp(alpha_block[0], alpha_block[1]);
                let mut packed = [0u8; 8];
                packed[..6].copy_from_slice(&alpha_block[2..8]);
                let indices = u64::from_le_bytes(packed);
                for index in 0..16u32 {
                    let x = bx * 4 + index % 4;
                    let y = by * 4 + index / 4;
                    let at = ((y * padded_width + x) * 4) as usize + channel;
                    padded[at] = ramp[((indices >> (3 * index)) & 7) as usize];
                }
            }
        }
    }
}

/// The 8-entry value ramp of an alpha-indexed block: endpoints then the
/// interpolated entries, rounded to nearest as the reference decoder does.
fn alpha_ramp(a0: u8, a1: u8) -> [u8; 8] {
    let mut ramp = [a0, a1, 0, 0, 0, 0, 0, 0];
    if a0 > a1 {
        for i in 1..7u32 {
            ramp[1 + i as usize] = ((u32::from(a0) * (7 - i) + u32::from(a1) * i + 3) / 7) as u8;
        }
    } else {
        for i in 1..5u32 {
            ramp[1 + i as usize] = ((u32::from(a0) * (5 - i) + u32::from(a1) * i + 2) / 5) as u8;
        }
        ramp[6] = 0;
        ramp[7] = 255;
    }
    ramp
}

/// Expands one uncompressed mip to RGBA8 through its channel masks. Every
/// mask must be a byte-aligned 8-bit field; anything else is rejected. A
/// zero alpha mask yields 255.
fn decode_uncompressed(
    width: u32,
    height: u32,
    bit_count: u32,
    masks: [u32; 4],
    data: &[u8],
) -> Result<Vec<u8>, ConvertError> {
    let bytes_per_pixel = match bit_count {
        8 | 16 | 24 | 32 => (bit_count / 8) as usize,
        _ => return Err(ConvertError::Unsupported("pixel bit count")),
    };
    let mut channels = [None; 4];
    for (channel, mask) in masks.iter().enumerate() {
        if *mask == 0 {
            continue;
        }
        let shift = mask.trailing_zeros();
        if *mask != 0xff << shift || shift % 8 != 0 || shift >= bit_count {
            return Err(ConvertError::Unsupported("pixel masks"));
        }
        channels[channel] = Some(shift / 8);
    }

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for pixel in data.chunks_exact(bytes_per_pixel) {
        for (channel, rgba_channel) in channels.iter().enumerate() {
            let value = match rgba_channel {
                Some(byte) if *byte < bytes_per_pixel as u32 => pixel[*byte as usize],
                _ if channel == 3 => 255,
                _ => 0,
            };
            rgba.push(value);
        }
    }
    Ok(rgba)
}
