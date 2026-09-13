//! DDS -> FTEX in the layout PES accepts from the 4cc compilers: 16 KiB
//! chunks zlib-compressed at level 3, 8-byte padded chunk areas, mip records
//! with ftexs 0, and a header with nrt 0x02, flags 0x11, zero hashes. The
//! deflate bytes are miniz_oxide's rather than zlib's, so the output is
//! not byte-identical to an archive-shipped one; parity is verified by converting back.

use std::io::{Cursor, Seek, SeekFrom, Write};

use binrw::{BinRead, BinWrite};
use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::FtexError;
use crate::dds::{DdsHeader, Dx10Header};
use crate::format::{ColorSpace, FtexHeader, MipRecord, PixelFormat, mip_size};

const CHUNK_SIZE: usize = 1 << 14;

/// Converts a DDS buffer to FTEX. `color_space` selects the texture-type
/// field (Linear 0x1, Srgb 0x3, Normal 0x9); the cube-map bit is added from
/// the DDS capabilities.
pub fn dds_to_ftex(dds: &[u8], color_space: ColorSpace) -> Result<Vec<u8>, FtexError> {
    if dds.len() < 128 {
        return Err(FtexError::Truncated);
    }
    let mut cursor = Cursor::new(dds);
    let dds_header = DdsHeader::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
    if dds_header.magic != *b"DDS " {
        return Err(FtexError::BadMagic);
    }
    if dds_header.header_size != 124 {
        return Err(FtexError::UnsupportedDds("header size != 124"));
    }

    let mipmap_count = if dds_header.capabilities1 & 0x400000 != 0 && dds_header.mipmap_count > 1 {
        dds_header.mipmap_count
    } else {
        1
    };

    let is_cube_map = if dds_header.capabilities2 & 0x200 != 0 {
        if dds_header.capabilities2 & 0xfe00 != 0xfe00 {
            return Err(FtexError::UnsupportedDds("incomplete cube map"));
        }
        true
    } else {
        false
    };
    let image_count = if is_cube_map { 6u32 } else { 1 };
    let depth = if dds_header.capabilities2 & 0x200000 != 0 {
        dds_header.depth
    } else {
        1
    };
    if is_cube_map && depth > 1 {
        return Err(FtexError::UnsupportedDds("cube map with volume depth"));
    }

    let format = detect_format(dds_header.format_flags, &dds_header, &mut cursor)?;

    let mut texture_type = color_space.texture_type();
    if is_cube_map {
        texture_type |= 0x4;
    }

    let version = if format.id() > 4 { 2.04f32 } else { 2.03f32 };

    // Frames: one mip per image, in image-then-mip order, each chunk-encoded.
    let mut frame_buffer = Vec::new();
    let mut records = Vec::new();
    for _ in 0..image_count {
        for level in 0..mipmap_count {
            let length = mip_size(format, dds_header.width, dds_header.height, depth, level);
            let start = cursor.position() as usize;
            let frame = dds.get(start..start + length).ok_or(FtexError::Truncated)?;
            let (encoded, chunk_count) = encode_image(frame)?;
            cursor.set_position((start + length) as u64);
            records.push(MipRecord {
                offset: 0, // filled below
                uncompressed_size: length as u32,
                compressed_size: encoded.len() as u32,
                index: level as u8,
                ftexs_number: 0,
                chunk_count,
            });
            frame_buffer.extend_from_slice(&encoded);
        }
    }

    let frame_buffer_offset = 64u32 + records.len() as u32 * 16;
    let mut mip_writer = Cursor::new(Vec::new());
    let mut relative = 0u32;
    for record in &mut records {
        record.offset = frame_buffer_offset + relative;
        relative += record.compressed_size;
        record
            .write(&mut mip_writer)
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    }
    let mip_buffer = mip_writer.into_inner();

    let header = FtexHeader {
        magic: *b"FTEX",
        version,
        pixel_format: format.id(),
        width: dds_header.width as u16,
        height: dds_header.height as u16,
        depth: depth as u16,
        mipmap_count: mipmap_count as u8,
        nrt: 0x02,
        flags: 0x11,
        unknown1: 1,
        unknown2: 0,
        texture_type,
        ftexs_count: 0,
        unknown3: 0,
        hash1: [0; 8],
        hash2: [0; 8],
    };

    let mut writer = Cursor::new(Vec::new());
    header
        .write(&mut writer)
        .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    let mut output = writer.into_inner();
    output.extend_from_slice(&mip_buffer);
    output.extend_from_slice(&frame_buffer);
    Ok(output)
}

/// Maps the DDS pixel-format fields to an FTEX format, with the same
/// acceptance rules the 4cc compilers have always applied.
fn detect_format(
    format_flags: u32,
    header: &DdsHeader,
    cursor: &mut Cursor<&[u8]>,
) -> Result<PixelFormat, FtexError> {
    if format_flags & 0x4 == 0 {
        // No FourCC: match the uncompressed masks.
        if format_flags & 0x40 != 0
            && format_flags & 0x1 != 0
            && header.r_mask == 0x00ff0000
            && header.g_mask == 0x0000ff00
            && header.b_mask == 0x000000ff
            && header.a_mask == 0xff000000
        {
            return Ok(PixelFormat::Argb8);
        }
        if format_flags & 0x20000 != 0
            && header.r_mask == 0xff
            && header.g_mask == 0
            && header.b_mask == 0
            && header.a_mask == 0
        {
            return Ok(PixelFormat::R8);
        }
        return Err(FtexError::UnsupportedDds("unrecognized uncompressed masks"));
    }

    let format = match &header.fourcc {
        b"DX10" => {
            cursor.seek(SeekFrom::Start(128))?;
            let ext = Dx10Header::read(cursor).map_err(|_| FtexError::Truncated)?;
            dxgi_to_format(ext.dxgi_format)
                .ok_or(FtexError::UnsupportedDds("unrecognized dxgi format"))?
        }
        b"8888" => PixelFormat::Argb8,
        b"DXT1" => PixelFormat::Bc1,
        b"DXT3" => PixelFormat::Bc2,
        b"DXT5" => PixelFormat::Bc3,
        b"ATI1" => PixelFormat::Bc4,
        b"ATI2" | b"BC5U" => PixelFormat::Bc5,
        _ => return Err(FtexError::UnsupportedDds("unrecognized fourcc")),
    };
    Ok(format)
}

/// The DX10 DXGI-format -> FTEX-format mapping; note 87 (B8G8R8A8) maps to
/// Argb8.
fn dxgi_to_format(dxgi: u32) -> Option<PixelFormat> {
    Some(match dxgi {
        87 => PixelFormat::Argb8,
        61 => PixelFormat::R8,
        71 => PixelFormat::Bc1,
        74 => PixelFormat::Bc2,
        77 => PixelFormat::Bc3,
        80 => PixelFormat::Bc4,
        83 => PixelFormat::Bc5,
        95 => PixelFormat::Bc6h,
        98 => PixelFormat::Bc7,
        10 => PixelFormat::Rgba16F,
        1 => PixelFormat::Rgba32F,
        24 => PixelFormat::Rgb10A2,
        26 => PixelFormat::Rg11B10F,
        _ => return None,
    })
}

/// Chunk-encodes one frame: `chunk_count` 8-byte records followed by zlib
/// streams of 16 KiB pieces, the whole area padded to 8.
fn encode_image(data: &[u8]) -> Result<(Vec<u8>, u16), FtexError> {
    let chunk_count = data.len().div_ceil(CHUNK_SIZE);
    let chunk_buffer_offset = chunk_count * 8;

    let mut header_buffer = Vec::new();
    let mut chunk_buffer = Vec::new();
    for piece in data.chunks(CHUNK_SIZE) {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(3));
        encoder.write_all(piece)?;
        let packed = encoder.finish()?;
        header_buffer.extend_from_slice(&(packed.len() as u16).to_le_bytes());
        header_buffer.extend_from_slice(&(piece.len() as u16).to_le_bytes());
        header_buffer
            .extend_from_slice(&((chunk_buffer.len() + chunk_buffer_offset) as u32).to_le_bytes());
        chunk_buffer.extend_from_slice(&packed);
    }

    let mut output = header_buffer;
    output.extend_from_slice(&chunk_buffer);
    output.resize(output.len() + (8 - output.len() % 8) % 8, 0);
    Ok((output, chunk_count as u16))
}
