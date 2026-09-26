//! Raster sources -> [`Decoded`]: the `image` crate decodes the accepted
//! formats to straight-alpha RGBA8, one mip, no blocks. A TIFF or TGA
//! declaring premultiplied alpha (TIFF ExtraSamples = 1, TGA extension
//! attributes type 4) is un-multiplied after the decode; a 16-bit TGA,
//! whose attribute bit `image` drops, is refused.

use std::io::Cursor;

use image::codecs::tga::TgaDecoder;
use image::{ColorType, DynamicImage, ExtendedColorType, ImageDecoder, ImageFormat};
use tiff::decoder::Decoder;
use tiff::tags::Tag;

use crate::{ConvertError, Decoded, SourceFormat};

/// The `image` crate's decoder id for an accepted raster format.
fn image_format(format: SourceFormat) -> Option<ImageFormat> {
    Some(match format {
        SourceFormat::Png => ImageFormat::Png,
        SourceFormat::Jpeg => ImageFormat::Jpeg,
        SourceFormat::Bmp => ImageFormat::Bmp,
        SourceFormat::WebP => ImageFormat::WebP,
        SourceFormat::Tga => ImageFormat::Tga,
        SourceFormat::Tiff => ImageFormat::Tiff,
        SourceFormat::Dds | SourceFormat::Ftex => return None,
    })
}

/// The `image` crate's decode of a raster buffer. TGA goes through its own
/// decoder first: the 16-bit primary layout whose attribute bit `image`
/// silently drops is refused, the interleaved storage `image` decodes in
/// order is refused (DirectXTexTGA.cpp:159-162), and the allocation limit
/// `ImageReader::decode` applies (image_reader_type.rs:314-320) is applied
/// to the declared size before the decode.
fn decode_image(bytes: &[u8], format: SourceFormat) -> Result<DynamicImage, ConvertError> {
    if format == SourceFormat::Tga {
        if bytes
            .get(17)
            .is_some_and(|descriptor| descriptor & 0xC0 != 0)
        {
            return Err(ConvertError::Unsupported("interleaved tga"));
        }
        let mut decoder = TgaDecoder::new(Cursor::new(bytes))?;
        if decoder.original_color_type() == ExtendedColorType::Rgb5x1 {
            return Err(ConvertError::Unsupported("16-bit tga: resave as 32-bit"));
        }
        let mut limits = image::Limits::default();
        limits.reserve(decoder.total_bytes())?;
        decoder.set_limits(limits)?;
        return Ok(DynamicImage::from_decoder(decoder)?);
    }
    let image_format = image_format(format).ok_or(ConvertError::Unsupported("raster format"))?;
    Ok(image::load_from_memory_with_format(bytes, image_format)?)
}

/// Whether a TIFF's first IFD declares ExtraSamples = 1 (associated —
/// premultiplied — alpha). An absent tag means straight alpha.
fn associated_alpha(bytes: &[u8]) -> Result<bool, ConvertError> {
    let mut decoder =
        Decoder::new(Cursor::new(bytes)).map_err(|_| ConvertError::Unsupported("tiff tags"))?;
    decoder
        .find_tag_unsigned_vec::<u16>(Tag::ExtraSamples)
        .map(|samples| samples.is_some_and(|s| s.first() == Some(&1)))
        .map_err(|_| ConvertError::Unsupported("tiff tags"))
}

/// Whether a TGA 2.0 file declares premultiplied alpha in its extension
/// area: the footer carries the `TRUEVISION-XFILE.` signature and an
/// extension offset whose 495-byte area ends with attributes type 4
/// (DirectXTex TGA.cpp:1346-1355).
fn tga_premultiplied(bytes: &[u8]) -> bool {
    let Some(footer) = bytes.last_chunk::<26>() else {
        return false;
    };
    if footer[8..26] != *b"TRUEVISION-XFILE.\0" {
        return false;
    }
    let extension = u32::from_le_bytes([footer[0], footer[1], footer[2], footer[3]]) as usize;
    if extension == 0 {
        return false;
    }
    let Some(area) = extension
        .checked_add(495)
        .and_then(|end| bytes.get(extension..end))
    else {
        return false;
    };
    u16::from_le_bytes([area[0], area[1]]) == 495 && area[494] == 4
}

/// Decodes a raster file to a single-mip RGBA8 texture.
pub(crate) fn decode_raster(bytes: &[u8], format: SourceFormat) -> Result<Decoded, ConvertError> {
    let image = decode_image(bytes, format)?;
    let (width, height) = (image.width(), image.height());
    let premultiplied = match format {
        SourceFormat::Tiff => associated_alpha(bytes)?,
        SourceFormat::Tga => tga_premultiplied(bytes),
        _ => false,
    };
    let has_alpha = image.color().has_alpha();
    let mut pixels = if premultiplied {
        unpremultiply(image)
    } else {
        image.to_rgba8().into_raw()
    };
    // texconv's default (no -tgazeroalpha) forces a TGA whose every alpha
    // sample is 0 to opaque (DirectXTexTGA.cpp:689-693,
    // texconv.cpp:2104-2106).
    if format == SourceFormat::Tga
        && has_alpha
        && pixels.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 0)
    {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
    }
    Ok(Decoded {
        width,
        height,
        mips: vec![pixels],
        blocks: None,
        authored_mips: false,
    })
}

/// Restores straight alpha in a premultiplied image: at the source's own
/// precision — f32 for float channels (quantizing would drop values below
/// the unit step), 16 bits for integer ones (an 8-bit reduction first would
/// lose low-alpha color; for 8-bit sources the 16-bit path gives the same
/// bytes as un-multiplying at 8 bits, checked over every (c, a) pair) —
/// then `image`'s own reduction to RGBA8.
fn unpremultiply(image: DynamicImage) -> Vec<u8> {
    if matches!(image.color(), ColorType::Rgb32F | ColorType::Rgba32F) {
        let mut rgba = image.to_rgba32f();
        for pixel in rgba.as_chunks_mut::<4>().0 {
            let alpha = pixel[3];
            if alpha > 0.0 {
                for channel in &mut pixel[..3] {
                    *channel = (*channel / alpha).clamp(0.0, 1.0);
                }
            }
        }
        return image::DynamicImage::ImageRgba32F(rgba)
            .to_rgba8()
            .into_raw();
    }
    let mut rgba = image.to_rgba16();
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = u64::from(pixel[3]);
        for channel in &mut pixel[..3] {
            // A zero alpha leaves the pixel as decoded.
            if let Some(straight) = (u64::from(*channel) * 65535 + alpha / 2).checked_div(alpha) {
                *channel = straight.min(65535) as u16;
            }
        }
    }
    image::DynamicImage::ImageRgba16(rgba).to_rgba8().into_raw()
}
