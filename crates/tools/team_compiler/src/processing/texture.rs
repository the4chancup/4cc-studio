//! Texture conversion for the Fox engine, which reads FTEX.

use anyhow::{Context, bail};

/// The texture `name` as FTEX: an `.ftex` passes through as it is, a `.dds` is converted. Any
/// other image format cannot be compiled yet, and is refused by its name alone.
pub(super) fn to_ftex(name: &str, bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let extension = name.rsplit_once('.').map_or("", |(_, extension)| extension);
    if extension.eq_ignore_ascii_case("ftex") {
        return Ok(bytes);
    }
    if !extension.eq_ignore_ascii_case("dds") {
        bail!("{name}: cannot be compiled yet");
    }
    ftex::dds_to_ftex(&bytes, ftex::ColorSpace::Normal)
        .with_context(|| format!("{name}: cannot convert to FTEX"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn a_dds_is_converted_an_ftex_passes_through_and_nothing_else_is_compiled() {
        let dds = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Tracer/Kits/g1/kit.dds"),
        )
        .unwrap();
        let converted = to_ftex("kit.dds", dds.clone()).unwrap();
        assert_eq!(
            converted,
            ftex::dds_to_ftex(&dds, ftex::ColorSpace::Normal).unwrap()
        );
        assert_eq!(to_ftex("kit.FTEX", converted.clone()).unwrap(), converted);
        assert_eq!(to_ftex("kit.DDS", dds.clone()).unwrap(), converted);
        // DDS bytes under another extension are refused by name, never read as a DDS.
        let error = to_ftex("kit.png", dds).unwrap_err();
        assert_eq!(format!("{error}"), "kit.png: cannot be compiled yet");
        let error = to_ftex("kit.dds", b"not a DDS".to_vec()).unwrap_err();
        assert_eq!(format!("{error}"), "kit.dds: cannot convert to FTEX");
    }
}
