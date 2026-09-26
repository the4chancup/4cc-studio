//! Raster sources -> [`Decoded`]: the `image` crate decodes the accepted
//! formats to straight-alpha RGBA8, one mip, no blocks. A TIFF whose first
//! IFD declares ExtraSamples = 1 stores premultiplied alpha and is
//! un-multiplied after the decode.

use std::io::Cursor;

use image::ImageFormat;
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

/// Decodes a raster file to a single-mip RGBA8 texture.
pub(crate) fn decode_raster(bytes: &[u8], format: SourceFormat) -> Result<Decoded, ConvertError> {
    let image_format = image_format(format).ok_or(ConvertError::Unsupported("raster format"))?;
    let image = image::load_from_memory_with_format(bytes, image_format)?;
    let (width, height) = (image.width(), image.height());
    let pixels = if format == SourceFormat::Tiff && associated_alpha(bytes)? {
        unpremultiply(image)
    } else {
        image.to_rgba8().into_raw()
    };
    Ok(Decoded {
        width,
        height,
        mips: vec![pixels],
        blocks: None,
        authored_mips: false,
    })
}

/// Restores straight alpha in a premultiplied image at 16-bit precision
/// (an 8-bit reduction first would drop low-alpha color), then reduces it
/// with `image`'s own conversion. For 8-bit sources this gives the same
/// bytes as un-multiplying at 8 bits (checked over every (c, a) pair), so
/// one path serves both.
fn unpremultiply(image: image::DynamicImage) -> Vec<u8> {
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
