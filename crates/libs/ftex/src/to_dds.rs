//! FTEX -> DDS: one frame per mip per image, DDS header rebuilt from the FTEX
//! header, byte-identical to the conversion the 4cc compilers have shipped.

use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::BinRead;
use flate2::read::ZlibDecoder;

use crate::FtexError;
use crate::dds::build_header;
use crate::format::{ChunkRecord, FtexHeader, FtexInfo, MipRecord, PixelFormat, mip_size};

/// Reads only the header and reports what texture checks ask for.
pub fn info(ftex: &[u8]) -> Result<FtexInfo, FtexError> {
    let header = read_header(ftex)?;
    Ok(FtexInfo {
        version: header.version,
        format: PixelFormat::from_id(header.pixel_format)
            .ok_or(FtexError::UnsupportedFormat(header.pixel_format))?,
        width: header.width,
        height: header.height,
        depth: header.depth,
        mipmaps: header.mipmap_count,
        texture_type: header.texture_type,
        is_cube_map: header.texture_type & 0x4 != 0,
    })
}

/// Converts a whole FTEX buffer to a DDS file: headers, frame order and
/// per-frame padding exactly as the 4cc compilers have always emitted them.
pub fn ftex_to_dds(ftex: &[u8]) -> Result<Vec<u8>, FtexError> {
    let header = read_header(ftex)?;
    let format = PixelFormat::from_id(header.pixel_format)
        .ok_or(FtexError::UnsupportedFormat(header.pixel_format))?;

    let (image_count, dds_depth, cube) = if header.texture_type & 0x4 != 0 {
        // Cube map: six images, depth must be 1.
        if header.depth > 1 {
            return Err(FtexError::UnsupportedVariant("cube map with depth > 1"));
        }
        (6u32, 1u32, true)
    } else {
        // A depth above 1 is a volume texture; `build_header` writes the
        // volume fields from it.
        (1, u32::from(header.depth.max(1)), false)
    };

    let mut cursor = Cursor::new(ftex);
    cursor.set_position(64);

    // A frame is one mip level of one image; cube maps carry 6*mips frames.
    struct FrameSpec {
        offset: u32,
        chunk_count: u16,
        uncompressed_size: u32,
        compressed_size: u32,
        expected_size: usize,
    }
    let mut specs = Vec::new();
    for _ in 0..image_count {
        for level in 0..u32::from(header.mipmap_count) {
            let record = MipRecord::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
            if record.index != level as u8 {
                return Err(FtexError::UnexpectedMipmap {
                    expected: level as u8,
                    found: record.index,
                });
            }
            specs.push(FrameSpec {
                offset: record.offset,
                chunk_count: record.chunk_count,
                uncompressed_size: record.uncompressed_size,
                compressed_size: record.compressed_size,
                expected_size: mip_size(
                    format,
                    u32::from(header.width),
                    u32::from(header.height),
                    dds_depth,
                    level,
                ),
            });
        }
    }

    let mut frames = Vec::with_capacity(specs.len());
    for spec in &specs {
        let mut frame = read_frame(
            ftex,
            spec.offset,
            spec.chunk_count,
            spec.uncompressed_size,
            spec.compressed_size,
            spec.expected_size,
        )?;
        // Every frame is padded or truncated to its expected mip size.
        frame.resize(spec.expected_size, 0);
        frames.push(frame);
    }

    let mut output = build_header(
        format,
        u32::from(header.width),
        u32::from(header.height),
        u32::from(header.mipmap_count),
        dds_depth,
        cube,
    );
    for frame in frames {
        output.extend_from_slice(&frame);
    }
    Ok(output)
}

fn read_header(ftex: &[u8]) -> Result<FtexHeader, FtexError> {
    let header = FtexHeader::read(&mut Cursor::new(ftex)).map_err(|_| FtexError::Truncated)?;
    if header.magic != *b"FTEX" {
        return Err(FtexError::BadMagic);
    }
    if !(2.025..=2.045).contains(&header.version) {
        return Err(FtexError::UnsupportedVersion(header.version));
    }
    if header.ftexs_count > 0 {
        return Err(FtexError::UnsupportedVariant("ftexs count > 0"));
    }
    if header.mipmap_count == 0 {
        return Err(FtexError::UnsupportedVariant("no mipmaps"));
    }
    Ok(header)
}

/// Reads one frame: a raw block, a single zlib stream, or a chunked stream
/// whose records carry (compressed, uncompressed, frame-relative offset).
/// No read produces more than the frame's `expected_size` bytes: chunks
/// past a full frame are never consulted and inflation stops at the bytes
/// still needed.
fn read_frame(
    ftex: &[u8],
    offset: u32,
    chunk_count: u16,
    uncompressed_size: u32,
    compressed_size: u32,
    expected_size: usize,
) -> Result<Vec<u8>, FtexError> {
    let mut cursor = Cursor::new(ftex);
    cursor.seek(SeekFrom::Start(u64::from(offset)))?;

    if chunk_count == 0 {
        if compressed_size == 0 {
            return take(&mut cursor, ftex, uncompressed_size as usize);
        }
        let packed = take(&mut cursor, ftex, compressed_size as usize)?;
        return inflate(&packed, expected_size);
    }

    let mut chunks = Vec::with_capacity(usize::from(chunk_count));
    for _ in 0..chunk_count {
        let record = ChunkRecord::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
        chunks.push(record);
    }
    let mut frame = Vec::new();
    for chunk in &chunks {
        if frame.len() >= expected_size {
            break;
        }
        let at = offset
            .checked_add(chunk.offset & !(1 << 31))
            .ok_or(FtexError::Truncated)?;
        let mut chunk_cursor = Cursor::new(ftex);
        chunk_cursor.seek(SeekFrom::Start(u64::from(at)))?;
        let data = take(&mut chunk_cursor, ftex, usize::from(chunk.compressed_size))?;
        let remaining = expected_size - frame.len();
        if chunk.compressed_size != chunk.uncompressed_size {
            frame.extend_from_slice(&inflate(&data, remaining)?);
        } else {
            frame.extend_from_slice(&data[..data.len().min(remaining)]);
        }
    }
    Ok(frame)
}

fn take(cursor: &mut Cursor<&[u8]>, whole: &[u8], len: usize) -> Result<Vec<u8>, FtexError> {
    let start = cursor.position() as usize;
    let end = start.checked_add(len).ok_or(FtexError::Truncated)?;
    let slice = whole.get(start..end).ok_or(FtexError::Truncated)?;
    cursor.set_position(end as u64);
    Ok(slice.to_vec())
}

/// Inflates at most `limit` bytes; a stream that would produce more is
/// read only that far.
fn inflate(bytes: &[u8], limit: usize) -> Result<Vec<u8>, FtexError> {
    let mut output = Vec::new();
    ZlibDecoder::new(bytes)
        .take(limit as u64)
        .read_to_end(&mut output)?;
    Ok(output)
}
