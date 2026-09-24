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
    let mut cursor = Cursor::new(dds);
    let dds_header = DdsHeader::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
    if dds_header.magic != *b"DDS " {
        return Err(FtexError::BadMagic);
    }
    if dds_header.header_size != 124 {
        return Err(FtexError::UnsupportedDds("header size != 124"));
    }

    let mipmap_count = crate::dds::mip_count(&dds_header);

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

    // The FTEX header's narrower fields, checked before the mip loop reads any
    // dimension: a size past them is an error, not a wrapped header.
    let field = |what: &'static str, value: usize| FtexError::HeaderFieldOverflow { what, value };
    let header_width =
        u16::try_from(dds_header.width).map_err(|_| field("width", dds_header.width as usize))?;
    let header_height = u16::try_from(dds_header.height)
        .map_err(|_| field("height", dds_header.height as usize))?;
    let header_depth = u16::try_from(depth).map_err(|_| field("depth", depth as usize))?;
    let header_mipmap_count =
        u8::try_from(mipmap_count).map_err(|_| field("mipmap_count", mipmap_count as usize))?;

    // Frames: one mip per image, in image-then-mip order, each chunk-encoded.
    let mut frame_buffer = Vec::new();
    let mut records = Vec::new();
    for _ in 0..image_count {
        for level in 0..mipmap_count {
            let length = mip_size(format, dds_header.width, dds_header.height, depth, level);
            let start = cursor.position() as usize;
            let end = start.checked_add(length).ok_or(FtexError::Truncated)?;
            let frame = dds.get(start..end).ok_or(FtexError::Truncated)?;
            let (encoded, chunk_count) = encode_image(frame)?;
            cursor.set_position(end as u64);
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
        width: header_width,
        height: header_height,
        depth: header_depth,
        mipmap_count: header_mipmap_count,
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
        crate::dds::single_image(&ext)?;
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
    let chunk_count = u16::try_from(chunk_count).map_err(|_| FtexError::HeaderFieldOverflow {
        what: "chunk count",
        value: chunk_count,
    })?;
    let chunk_buffer_offset = usize::from(chunk_count) * 8;

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
        let stored_len = u16::try_from(stored.len())
            .expect("a 16 KiB piece's zlib stream is at most 16395 bytes");
        let piece_len = u16::try_from(piece.len()).expect("a piece is at most 16 KiB");
        let chunk_offset = u32::try_from(chunk_buffer.len() + chunk_buffer_offset)
            .expect("65535 chunks of at most 16395 bytes plus their records stay under u32::MAX");
        header_buffer.extend_from_slice(&stored_len.to_le_bytes());
        header_buffer.extend_from_slice(&piece_len.to_le_bytes());
        header_buffer.extend_from_slice(&chunk_offset.to_le_bytes());
        chunk_buffer.extend_from_slice(stored);
    }

    let mut output = header_buffer;
    output.extend_from_slice(&chunk_buffer);
    output.resize(output.len() + (8 - output.len() % 8) % 8, 0);
    Ok((output, chunk_count))
}
