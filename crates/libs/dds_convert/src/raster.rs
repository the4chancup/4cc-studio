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
    let image = image::load_from_memory_with_format(bytes, image_format)?.to_rgba8();
    let (width, height) = image.dimensions();
    let mut pixels = image.into_raw();
    if format == SourceFormat::Tiff && associated_alpha(bytes)? {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            let alpha = u32::from(pixel[3]);
            for channel in &mut pixel[..3] {
                // A zero alpha leaves the pixel as decoded.
                if let Some(straight) = (u32::from(*channel) * 255 + alpha / 2).checked_div(alpha) {
                    *channel = straight.min(255) as u8;
                }
            }
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
