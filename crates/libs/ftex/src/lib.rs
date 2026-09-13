//! Fox Engine FTEX texture container: FTEX <-> DDS.
//!
//! [`ftex_to_dds`] produces the DDS layout the 4cc compilers have always
//! emitted for an FTEX, byte for byte; [`dds_to_ftex`] produces the FTEX layout
//! they emitted, except that the zlib streams are miniz_oxide's, so round-trip
//! parity is verified by converting back to DDS.

/// The DDS container knowledge (header records, layout walk, header writer)
/// shared with crates that read or rebuild DDS files.
pub mod dds;
mod format;
mod from_dds;
mod to_dds;

pub use format::{ColorSpace, FtexInfo, PixelFormat, mip_size};
pub use from_dds::dds_to_ftex;
pub use to_dds::{ftex_to_dds, info};

/// Why an FTEX or DDS buffer could not be converted.
#[derive(Debug, thiserror::Error)]
pub enum FtexError {
    /// The buffer ends before a structure that extends past it.
    #[error("buffer is truncated")]
    Truncated,
    /// Wrong `FTEX`/`DDS ` magic.
    #[error("invalid magic")]
    BadMagic,
    /// FTEX version outside the accepted 2.025..=2.045 range.
    #[error("unsupported ftex version {0}")]
    UnsupportedVersion(f32),
    /// A shape of the format this crate does not read (names it).
    #[error("unsupported ftex variant: {0}")]
    UnsupportedVariant(&'static str),
    /// Unknown FTEX pixel-format id.
    #[error("unsupported ftex pixel format {0}")]
    UnsupportedFormat(u16),
    /// A DDS feature this crate cannot map to FTEX (names it).
    #[error("unsupported dds: {0}")]
    UnsupportedDds(&'static str),
    /// Zlib or I/O failure while (de)compressing.
    #[error("zlib error: {0}")]
    Zlib(#[from] std::io::Error),
    /// A mip record's index did not match its expected mip level.
    #[error("expected mip {expected}, found {found}")]
    UnexpectedMipmap {
        /// The mip level the record should describe.
        expected: u8,
        /// The mip index the record carries.
        found: u8,
    },
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    const BC7: &[u8] = include_bytes!("../tests/fixtures/konami_bc7_nb00b.ftex");
    const BC1: &[u8] = include_bytes!("../tests/fixtures/konami_bc1_bibs_metalness.ftex");
    const BC1_DDS: &[u8] = include_bytes!("../tests/fixtures/konami_bc1_bibs_metalness.dds");
    const BC3_NRM: &[u8] = include_bytes!("../tests/fixtures/konami_bc3_dummy_nrm.ftex");
    const BC3_NRM_DDS: &[u8] = include_bytes!("../tests/fixtures/konami_bc3_dummy_nrm.dds");
    const RGBA32: &[u8] = include_bytes!("../tests/fixtures/konami_rgba32_gr_dither_nrt.ftex");
    const RGBA32_DDS: &[u8] = include_bytes!("../tests/fixtures/konami_rgba32_gr_dither_nrt.dds");
    const CUBE: &[u8] =
        include_bytes!("../tests/fixtures/konami_bc1_cubemap_default_reflection.ftex");
    const CUBE_DDS: &[u8] =
        include_bytes!("../tests/fixtures/konami_bc1_cubemap_default_reflection.dds");
    const LOGO: &[u8] = include_bytes!("../tests/fixtures/konami_bc3_1x1_cup_logo.ftex");
    const LOGO_DDS: &[u8] = include_bytes!("../tests/fixtures/konami_bc3_1x1_cup_logo.dds");
    const PFT_FROM_DDS: &[u8] = include_bytes!("../tests/fixtures/pft_from_dummy_nrm_dds.ftex");

    #[test]
    fn info_matches_the_readme_table() {
        let cases = [
            (BC7, PixelFormat::Bc7, 1024, 128, 9, 0x1, 2.040, false),
            (BC1, PixelFormat::Bc1, 16, 16, 3, 0x1, 2.030, false),
            (BC3_NRM, PixelFormat::Bc3, 128, 128, 6, 0x9, 2.040, false),
            (RGBA32, PixelFormat::Argb8, 8, 8, 2, 0x1, 2.030, false),
            (CUBE, PixelFormat::Bc1, 16, 16, 3, 0x7, 2.040, true),
            (LOGO, PixelFormat::Bc3, 1, 1, 1, 0x3, 2.040, false),
        ];
        for (bytes, format, width, height, mips, ty, version, cube) in cases {
            let info = info(bytes).unwrap();
            assert_eq!(info.format, format);
            assert_eq!(info.width, width);
            assert_eq!(info.height, height);
            assert_eq!(info.mipmaps, mips);
            assert_eq!(info.texture_type, ty);
            assert_eq!(info.is_cube_map, cube);
            assert!((info.version - version).abs() < 0.0005, "{}", info.version);
        }
    }

    #[test]
    fn ftex_to_dds_matches_committed_files() {
        for (ftex, dds) in [
            (BC1, BC1_DDS),
            (BC3_NRM, BC3_NRM_DDS),
            (RGBA32, RGBA32_DDS),
            (CUBE, CUBE_DDS),
            (LOGO, LOGO_DDS),
        ] {
            assert_eq!(ftex_to_dds(ftex).unwrap(), dds);
        }
    }

    #[test]
    fn bc7_converts_to_expected_dds() {
        let dds = ftex_to_dds(BC7).unwrap();
        assert_eq!(dds.len(), 174980);
        let digest: [u8; 32] = Sha256::digest(&dds).into();
        assert_eq!(
            hex(&digest),
            "e1853fe7888013ce91b659b2c34d45dc28fef8880386df3a2f89a71af22fcdeb"
        );
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn dds_round_trips_through_ftex() {
        for (dds, space) in [
            (BC1_DDS, ColorSpace::Linear),
            (BC3_NRM_DDS, ColorSpace::Normal),
            (RGBA32_DDS, ColorSpace::Linear),
            (CUBE_DDS, ColorSpace::Linear),
            (LOGO_DDS, ColorSpace::Srgb),
        ] {
            let ftex = dds_to_ftex(dds, space).unwrap();
            assert_eq!(ftex_to_dds(&ftex).unwrap(), dds);
        }
    }

    #[test]
    fn dds_to_ftex_matches_reference_header_and_mip_fields() {
        let ours = dds_to_ftex(BC3_NRM_DDS, ColorSpace::Normal).unwrap();
        assert_eq!(&ours[..64], &PFT_FROM_DDS[..64]);

        // Same mip count -> same number of 16-byte records.
        let mip_count = usize::from(info(PFT_FROM_DDS).unwrap().mipmaps);
        for i in 0..mip_count {
            let ours_rec = &ours[64 + 16 * i..64 + 16 * i + 16];
            let pft_rec = &PFT_FROM_DDS[64 + 16 * i..64 + 16 * i + 16];
            // Fields: uncompressed size, index, ftexs, chunk count must match;
            // offset and compressed size may differ.
            assert_eq!(
                &ours_rec[4..8],
                &pft_rec[4..8],
                "uncompressed size, mip {i}"
            );
            assert_eq!(
                &ours_rec[12..16],
                &pft_rec[12..16],
                "index/ftexs/chunks, mip {i}"
            );
        }
        assert_eq!(ftex_to_dds(&ours).unwrap(), BC3_NRM_DDS);
    }

    #[test]
    fn malformed_inputs_error() {
        let mut bad_magic = BC1.to_vec();
        bad_magic[0] = b'X';
        assert!(matches!(ftex_to_dds(&bad_magic), Err(FtexError::BadMagic)));

        let mut bad_version = BC1.to_vec();
        bad_version[4..8].copy_from_slice(&2.05f32.to_le_bytes());
        assert!(matches!(
            ftex_to_dds(&bad_version),
            Err(FtexError::UnsupportedVersion(v)) if (v - 2.05).abs() < 0.001
        ));

        // Header valid but the mip table runs past the end.
        assert!(matches!(ftex_to_dds(&BC1[..70]), Err(FtexError::Truncated)));

        let mut bad_dds = BC1_DDS.to_vec();
        bad_dds[84..88].copy_from_slice(b"NOPE");
        assert!(matches!(
            dds_to_ftex(&bad_dds, ColorSpace::Linear),
            Err(FtexError::UnsupportedDds(_))
        ));
    }

    #[test]
    fn read_layout_reports_dims_and_rejects_cube() {
        let layout = dds::read_layout(BC1_DDS).unwrap();
        assert_eq!(layout.pixel, dds::DdsPixel::Format(PixelFormat::Bc1));
        assert_eq!((layout.width, layout.height), (16, 16));
        assert_eq!(layout.mipmaps, 3);
        assert_eq!(layout.data_offset, 128);

        assert!(matches!(
            dds::read_layout(CUBE_DDS),
            Err(FtexError::UnsupportedDds("cube map"))
        ));
        assert_eq!(dds::read_layout(BC1_DDS).unwrap().row_pitch, None);
    }

    /// DX10 headers: arrays, the cube flag, signed and alpha-less formats.
    #[test]
    fn read_layout_dx10_extension_rules() {
        let dx10 = ftex_to_dds(BC7).unwrap();
        assert_eq!(&dx10[84..88], b"DX10");
        let layout = dds::read_layout(&dx10).unwrap();
        assert_eq!(layout.pixel, dds::DdsPixel::Format(PixelFormat::Bc7));
        assert_eq!(layout.data_offset, 148);

        let mut array = dx10.clone();
        array[140..144].copy_from_slice(&2u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&array),
            Err(FtexError::UnsupportedDds("texture array"))
        ));
        assert!(matches!(
            dds_to_ftex(&array, ColorSpace::Linear),
            Err(FtexError::UnsupportedDds("texture array"))
        ));

        let mut cube_flag = dx10.clone();
        cube_flag[136..140].copy_from_slice(&0x4u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&cube_flag),
            Err(FtexError::UnsupportedDds("cube map"))
        ));

        let mut signed = dx10.clone();
        signed[128..132].copy_from_slice(&84u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&signed),
            Err(FtexError::UnsupportedDds("signed block format"))
        ));

        let mut bgrx = dx10.clone();
        bgrx[128..132].copy_from_slice(&88u32.to_le_bytes());
        assert_eq!(
            dds::read_layout(&bgrx).unwrap().pixel,
            dds::DdsPixel::Uncompressed {
                bit_count: 32,
                r_mask: 0xff0000,
                g_mask: 0xff00,
                b_mask: 0xff,
                a_mask: 0,
            }
        );
    }
}
