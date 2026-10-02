//! Texture conversion for the Fox engine, which reads FTEX.

use anyhow::Context;

use crate::plan::subset::TextureFormat;

/// The texture file `name`, in `format`, as FTEX: an FTEX passes through as it is, a DDS is
/// converted.
pub(super) fn to_ftex(
    format: TextureFormat,
    name: &str,
    bytes: Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    match format {
        TextureFormat::Ftex => Ok(bytes),
        TextureFormat::Dds => ftex::dds_to_ftex(&bytes, ftex::ColorSpace::Normal)
            .with_context(|| format!("{name}: cannot convert to FTEX")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn a_dds_is_converted_and_an_ftex_passes_through() {
        let dds = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Tracer/Kits/g1/kit.dds"),
        )
        .unwrap();
        let converted = to_ftex(TextureFormat::Dds, "kit.dds", dds.clone()).unwrap();
        assert_eq!(
            converted,
            ftex::dds_to_ftex(&dds, ftex::ColorSpace::Normal).unwrap()
        );
        assert_eq!(
            to_ftex(TextureFormat::Ftex, "kit.ftex", converted.clone()).unwrap(),
            converted
        );
        let error = to_ftex(TextureFormat::Dds, "kit.dds", b"not a DDS".to_vec()).unwrap_err();
        assert_eq!(format!("{error}"), "kit.dds: cannot convert to FTEX");
    }
}
