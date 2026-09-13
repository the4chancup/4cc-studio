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
use crate::dds::{DdsHeader, DdsPixel, Dx10Header};
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

/// Maps the DDS pixel-format fields to an FTEX format, through the same
/// FourCC/DXGI/mask table `dds::read_layout` uses. A `DdsPixel::Uncompressed`
/// layout has no FTEX twin.
fn detect_format(
    format_flags: u32,
    header: &DdsHeader,
    cursor: &mut Cursor<&[u8]>,
) -> Result<PixelFormat, FtexError> {
    let pixel = if format_flags & 0x4 == 0 {
        crate::dds::uncompressed_pixel(header)?
    } else if &header.fourcc == b"DX10" {
        cursor.seek(SeekFrom::Start(128))?;
        let ext = Dx10Header::read(cursor).map_err(|_| FtexError::Truncated)?;
        crate::dds::dxgi_pixel(ext.dxgi_format)?
    } else {
        DdsPixel::Format(
            crate::dds::fourcc_format(&header.fourcc)
                .ok_or(FtexError::UnsupportedDds("unrecognized fourcc"))?,
        )
    };
    match pixel {
        DdsPixel::Format(format) => Ok(format),
        DdsPixel::Uncompressed { .. } => {
            Err(FtexError::UnsupportedDds("unrecognized uncompressed masks"))
        }
    }
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
        // Readers take `compressed == uncompressed` to mean a raw chunk, so a
        // zlib stream that happens to be exactly the piece's size would be
        // misread; store the piece itself instead (a 16-byte tail mip does
        // compress to 16 bytes).
        let stored = if packed.len() == piece.len() {
            piece
        } else {
            packed.as_slice()
        };
        header_buffer.extend_from_slice(&(stored.len() as u16).to_le_bytes());
        header_buffer.extend_from_slice(&(piece.len() as u16).to_le_bytes());
        header_buffer
            .extend_from_slice(&((chunk_buffer.len() + chunk_buffer_offset) as u32).to_le_bytes());
        chunk_buffer.extend_from_slice(stored);
    }

    let mut output = header_buffer;
    output.extend_from_slice(&chunk_buffer);
    output.resize(output.len() + (8 - output.len() % 8) % 8, 0);
    Ok((output, chunk_count as u16))
}
