//! FTEX -> DDS, byte-identical to pes-file-tools' `ftexToDdsBuffer`.

use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::BinRead;
use flate2::read::ZlibDecoder;

use crate::dds::{DdsHeader, Dx10Header};
use crate::format::{mip_size, ChunkRecord, FtexHeader, FtexInfo, PixelFormat};
use crate::FtexError;

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

/// Converts a whole FTEX buffer to a DDS file, reproducing
/// `ftexToDdsBuffer`'s headers, frame order and per-frame padding exactly.
pub fn ftex_to_dds(ftex: &[u8]) -> Result<Vec<u8>, FtexError> {
    let header = read_header(ftex)?;
    let format = PixelFormat::from_id(header.pixel_format)
        .ok_or(FtexError::UnsupportedFormat(header.pixel_format))?;

    let mut dds_flags = 0x1 | 0x2 | 0x4 | 0x1000 | 0x20000;
    // capabilities, complex, mipmap — the cube-map branch in the reference
    // adds only bits that are already set here.
    let caps1 = 0x1000 | 0x8 | 0x400000;
    let mut caps2 = 0u32;

    let (image_count, dds_depth, ext_dimension, ext_flags) = if header.texture_type & 0x4 != 0 {
        // Cube map: six images, depth must be 1.
        if header.depth > 1 {
            return Err(FtexError::UnsupportedVariant("cube map with depth > 1"));
        }
        caps2 |= 0xfe00;
        (6u32, 1u32, 3u32, 0x4u32)
    } else if header.depth > 1 {
        // Volume texture.
        dds_flags |= 0x800000;
        caps2 |= 0x200000;
        (1, u32::from(header.depth), 4, 0)
    } else {
        (1, 1, 3, 0)
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
        )?;
        // Every frame is padded or truncated to its expected mip size.
        frame.resize(spec.expected_size, 0);
        frames.push(frame);
    }

    let (pitch_or_linear, format_flags, fourcc, rgb_bit_count, masks, dx10) =
        if format == PixelFormat::Argb8 {
            (
                4 * u32::from(header.width),
                0x41u32,
                [0u8; 4],
                32u32,
                [0x00ff0000, 0x0000ff00, 0x000000ff, 0xff000000],
                None,
            )
        } else {
            dds_flags |= 0x80000;
            let fourcc = format.fourcc().unwrap_or([0; 4]);
            (
                frames.first().map(Vec::len).unwrap_or(0) as u32,
                0x4u32,
                fourcc,
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
        height: u32::from(header.height),
        width: u32::from(header.width),
        pitch_or_linear_size: pitch_or_linear,
        depth: dds_depth,
        mipmap_count: u32::from(header.mipmap_count),
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

    let mut output = Vec::new();
    dds.write(&mut output)
        .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    if let Some(dx10) = dx10 {
        dx10.write(&mut output)
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
    }
    for frame in frames {
        output.extend_from_slice(&frame);
    }
    Ok(output)
}

fn read_header(ftex: &[u8]) -> Result<FtexHeader, FtexError> {
    if ftex.len() < 64 {
        return Err(FtexError::Truncated);
    }
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
fn read_frame(
    ftex: &[u8],
    offset: u32,
    chunk_count: u16,
    uncompressed_size: u32,
    compressed_size: u32,
) -> Result<Vec<u8>, FtexError> {
    let mut cursor = Cursor::new(ftex);
    cursor.seek(SeekFrom::Start(u64::from(offset)))?;

    if chunk_count == 0 {
        if compressed_size == 0 {
            return Ok(take(&mut cursor, ftex, uncompressed_size as usize)?);
        }
        let packed = take(&mut cursor, ftex, compressed_size as usize)?;
        return Ok(inflate(&packed)?);
    }

    let mut chunks = Vec::with_capacity(usize::from(chunk_count));
    for _ in 0..chunk_count {
        let record = ChunkRecord::read(&mut cursor).map_err(|_| FtexError::Truncated)?;
        chunks.push(record);
    }
    let mut frame = Vec::new();
    for chunk in &chunks {
        let at = offset
            .checked_add(chunk.offset & !(1 << 31))
            .ok_or(FtexError::Truncated)?;
        let mut chunk_cursor = Cursor::new(ftex);
        chunk_cursor.seek(SeekFrom::Start(u64::from(at)))?;
        let data = take(&mut chunk_cursor, ftex, usize::from(chunk.compressed_size))?;
        if chunk.compressed_size != chunk.uncompressed_size {
            frame.extend_from_slice(&inflate(&data)?);
        } else {
            frame.extend_from_slice(&data);
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

fn inflate(bytes: &[u8]) -> Result<Vec<u8>, FtexError> {
    let mut output = Vec::new();
    ZlibDecoder::new(bytes).read_to_end(&mut output)?;
    Ok(output)
}
