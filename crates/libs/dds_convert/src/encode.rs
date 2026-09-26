//! [`Decoded`] -> finished container bytes: codec selection by target and
//! role, passthrough of compatible blocks, the DXT5nm channel layout for
//! normal maps, block encoding via `block_compression`, and the DDS/FTEX
//! container wrap.

use std::borrow::Cow;

use block_compression::encode::compress_rgba8;
use block_compression::{BC7Settings, CompressionVariant};
use pes_version::{Engine, PesVersion};

use crate::mips::{self, Mip};
use crate::{BlockCodec, ConvertError, Decoded, Target, TextureRole, alloc_len};

/// Emits the DDS or FTEX bytes for `decoded` on `target`, per the codec
/// rules of the conversion plan.
pub(crate) fn convert(decoded: &Decoded, target: Target) -> Result<Vec<u8>, ConvertError> {
    // Passthrough: blocks a target reads are kept and only the container
    // changes. The role narrows this: a normal-role source passes through
    // only as BC3 (taken to be laid out already) or, on PES 19-21, as BC5;
    // a normal-role BC7 or BC1 source is decoded and encoded to DXT5nm.
    if let Some(blocks) = &decoded.blocks
        && compatible(target.version, blocks.codec, target.role)
    {
        return container(
            target.version,
            pixel_format(blocks.codec),
            decoded.width,
            decoded.height,
            &blocks.mips,
        );
    }

    // Mips to emit: a source that carried its own mip chain keeps its level
    // count; a raster source gets the full chain generated.
    let mut emit: Vec<Mip> = Vec::new();
    if decoded.authored_mips {
        for (level, pixels) in decoded.mips.iter().enumerate() {
            emit.push(Mip {
                width: (decoded.width >> level).max(1),
                height: (decoded.height >> level).max(1),
                pixels: Cow::Borrowed(pixels.as_slice()),
            });
        }
    } else {
        emit.push(Mip {
            width: decoded.width,
            height: decoded.height,
            pixels: Cow::Borrowed(decoded.mips[0].as_slice()),
        });
        emit.extend(mips::generate(
            decoded.width,
            decoded.height,
            &decoded.mips[0],
        ));
    }

    let normal_layout = target.role == TextureRole::Normal || is_bc5_source(decoded);
    let codec = select_codec(target, normal_layout, &emit);
    let variant = compression_variant(codec, &emit);

    let mut blocks = Vec::with_capacity(emit.len());
    for mip in &emit {
        let swizzled;
        let rgba: &[u8] = if normal_layout {
            swizzled = normal_swizzle(mip, target.version.engine());
            &swizzled
        } else {
            &mip.pixels
        };
        blocks.push(encode_mip(variant, mip.width, mip.height, rgba)?);
    }
    container(
        target.version,
        pixel_format(codec),
        decoded.width,
        decoded.height,
        &blocks,
    )
}

/// Whether the target keeps this block codec without re-encoding: PES 15-18
/// read BC1..BC3, PES 19-21 also BC4, BC5 and BC7. A normal-role source
/// keeps only BC3 blocks (or BC5 on PES 19-21): anything else carries a
/// color layout the shader would read as a normal map.
fn compatible(version: PesVersion, codec: BlockCodec, role: TextureRole) -> bool {
    let version_reads = match codec {
        BlockCodec::Bc1 | BlockCodec::Bc2 | BlockCodec::Bc3 => true,
        BlockCodec::Bc4 | BlockCodec::Bc5 | BlockCodec::Bc7 => version >= PesVersion::Pes19,
    };
    let role_keeps = role == TextureRole::Color
        || codec == BlockCodec::Bc3
        || (codec == BlockCodec::Bc5 && version >= PesVersion::Pes19);
    version_reads && role_keeps
}

fn is_bc5_source(decoded: &Decoded) -> bool {
    matches!(
        decoded.blocks.as_ref().map(|blocks| blocks.codec),
        Some(BlockCodec::Bc5)
    )
}

/// The output codec when encoding is needed: normal maps are always BC3 in
/// the DXT5nm layout; color on PES 19-21 is BC7; on PES 15-18 fully opaque
/// color is BC1 and anything with alpha is BC3.
fn select_codec(target: Target, normal_layout: bool, emit: &[Mip]) -> BlockCodec {
    if normal_layout {
        return BlockCodec::Bc3;
    }
    if target.version >= PesVersion::Pes19 {
        return BlockCodec::Bc7;
    }
    if emit.iter().all(|mip| fully_opaque(&mip.pixels)) {
        return BlockCodec::Bc1;
    }
    BlockCodec::Bc3
}

fn fully_opaque(pixels: &[u8]) -> bool {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .all(|pixel| pixel[3] == 255)
}

/// The `block_compression` variant for an emitted codec. BC7 takes the
/// alpha preset only when an emitted mip actually carries alpha: on opaque
/// pixels both presets emit the same blocks (measured), and the opaque one
/// does less work (it skips mode 7 and the fourth channel).
fn compression_variant(codec: BlockCodec, emit: &[Mip]) -> CompressionVariant {
    match codec {
        BlockCodec::Bc1 => CompressionVariant::BC1,
        BlockCodec::Bc2 => CompressionVariant::BC2,
        BlockCodec::Bc3 => CompressionVariant::BC3,
        BlockCodec::Bc4 => CompressionVariant::BC4,
        BlockCodec::Bc5 => CompressionVariant::BC5,
        BlockCodec::Bc7 => {
            let settings = if emit.iter().all(|mip| fully_opaque(&mip.pixels)) {
                BC7Settings::opaque_basic()
            } else {
                BC7Settings::alpha_basic()
            };
            CompressionVariant::BC7(settings)
        }
    }
}

/// The FTEX pixel format a container header names for an emitted codec.
pub(crate) fn pixel_format(codec: BlockCodec) -> ftex::PixelFormat {
    match codec {
        BlockCodec::Bc1 => ftex::PixelFormat::Bc1,
        BlockCodec::Bc2 => ftex::PixelFormat::Bc2,
        BlockCodec::Bc3 => ftex::PixelFormat::Bc3,
        BlockCodec::Bc4 => ftex::PixelFormat::Bc4,
        BlockCodec::Bc5 => ftex::PixelFormat::Bc5,
        BlockCodec::Bc7 => ftex::PixelFormat::Bc7,
    }
}

/// The DXT5nm channel layout: X (decoded R) goes to alpha, Y (decoded G) to
/// green. pre-Fox files carry a grey Y in the color block (`R = B = Y`);
/// Fox files carry `R = 255`, `B = 0`.
fn normal_swizzle(mip: &Mip, engine: Engine) -> Vec<u8> {
    let mut rgba = mip.pixels.to_vec();
    for pixel in rgba.as_chunks_mut::<4>().0.iter_mut() {
        let x = pixel[0];
        let y = pixel[1];
        match engine {
            Engine::PreFox => {
                pixel[0] = y;
                pixel[2] = y;
            }
            Engine::Fox => {
                pixel[0] = 255;
                pixel[2] = 0;
            }
        }
        pixel[1] = y;
        pixel[3] = x;
    }
    rgba
}

/// Encodes one mip's RGBA8 pixels to blocks. The encoder requires
/// dimensions that are multiples of 4, so an unaligned mip is padded by
/// edge replication; the resulting block count is exactly what the logical
/// dimensions need, so nothing is trimmed. Dimensions whose padded product
/// cannot exist are `InvalidDecoded`, checked before the buffer is read.
fn encode_mip(
    variant: CompressionVariant,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<Vec<u8>, ConvertError> {
    let padded_width = width
        .div_ceil(4)
        .checked_mul(4)
        .ok_or(ConvertError::InvalidDecoded("dimensions"))?;
    let padded_height = height
        .div_ceil(4)
        .checked_mul(4)
        .ok_or(ConvertError::InvalidDecoded("dimensions"))?;
    let stride = padded_width
        .checked_mul(4)
        .ok_or(ConvertError::InvalidDecoded("dimensions"))?;
    if padded_width == width && padded_height == height {
        let mut blocks = vec![0u8; alloc_len(variant.blocks_byte_size(width, height) as u64)?];
        compress_rgba8(variant, rgba, &mut blocks, width, height, stride);
        return Ok(blocks);
    }
    let padded_len = u64::from(padded_width)
        .checked_mul(u64::from(padded_height))
        .and_then(|area| area.checked_mul(4))
        .ok_or(ConvertError::InvalidDecoded("dimensions"))
        .and_then(alloc_len)?;
    let mut padded = vec![0u8; padded_len];
    for y in 0..padded_height as usize {
        let source_y = y.min(height as usize - 1);
        for x in 0..padded_width as usize {
            let source_x = x.min(width as usize - 1);
            let from = (source_y * width as usize + source_x) * 4;
            let to = (y * padded_width as usize + x) * 4;
            padded[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
        }
    }
    let mut blocks =
        vec![0u8; alloc_len(variant.blocks_byte_size(padded_width, padded_height) as u64)?];
    compress_rgba8(
        variant,
        &padded,
        &mut blocks,
        padded_width,
        padded_height,
        stride,
    );
    Ok(blocks)
}

/// Wraps the emitted blocks in the container the version reads: a DDS for
/// PES 15-17, an FTEX for PES 18-21.
fn container(
    version: PesVersion,
    format: ftex::PixelFormat,
    width: u32,
    height: u32,
    blocks: &[Vec<u8>],
) -> Result<Vec<u8>, ConvertError> {
    let mut dds = ftex::dds::header_bytes(format, width, height, blocks.len() as u32);
    for mip in blocks {
        dds.extend_from_slice(mip);
    }
    if version.engine() == Engine::PreFox {
        return Ok(dds);
    }
    // why: the conversion plan's "FTEX texture type" bullet writes every Fox
    // output as 0x9, color and normal alike.
    Ok(ftex::dds_to_ftex(&dds, ftex::ColorSpace::Normal)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_mip_pads_an_unaligned_width() {
        // 6x4: the width alone pads to 8, so the encoded buffer is the
        // padded 8x4 block count.
        let blocks = encode_mip(CompressionVariant::BC1, 6, 4, &[0u8; 6 * 4 * 4]).unwrap();
        assert_eq!(blocks.len(), CompressionVariant::BC1.blocks_byte_size(8, 4));
    }

    #[test]
    fn encode_mip_reports_oversized_dimensions() {
        // u32::MAX - 2 wide pads past u32; the error comes before the
        // (deliberately tiny) pixel buffer is touched.
        assert!(matches!(
            encode_mip(CompressionVariant::BC1, u32::MAX - 2, 1, &[0u8; 4]),
            Err(ConvertError::InvalidDecoded("dimensions"))
        ));
    }
}
