//! Raster sources -> [`Decoded`]: the `image` crate decodes the accepted
//! formats to straight-alpha RGBA8, one mip, no blocks.

use image::ImageFormat;

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

/// Decodes a raster file to a single-mip RGBA8 texture.
pub(crate) fn decode_raster(bytes: &[u8], format: SourceFormat) -> Result<Decoded, ConvertError> {
    let image_format = image_format(format).ok_or(ConvertError::Unsupported("raster format"))?;
    let image = image::load_from_memory_with_format(bytes, image_format)?.to_rgba8();
    let (width, height) = image.dimensions();
    Ok(Decoded {
        width,
        height,
        mips: vec![image.into_raw()],
        blocks: None,
    })
}
