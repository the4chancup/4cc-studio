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
    /// A header field cannot hold the value the input describes (a DDS
    /// dimension or mip count past FTEX's narrower fields, a frame needing
    /// more chunks than the u16 record field holds).
    #[error("header field overflow: {what} is {value}")]
    HeaderFieldOverflow {
        /// Which field overflowed.
        what: &'static str,
        /// The value that did not fit.
        value: usize,
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
    const VOLUME: &[u8] = include_bytes!("../tests/fixtures/pft_volume_rgba16f.ftex");
    const VOLUME_DDS: &[u8] = include_bytes!("../tests/fixtures/pft_volume_rgba16f.dds");
    const RAW_FRAMES: &[u8] = include_bytes!("../tests/fixtures/handbuilt_raw_frames.ftex");
    const SINGLE_ZLIB: &[u8] = include_bytes!("../tests/fixtures/handbuilt_single_zlib.ftex");

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

        // Buffers that end inside a header are truncated in both directions.
        assert!(matches!(ftex_to_dds(&BC1[..63]), Err(FtexError::Truncated)));
        assert!(matches!(
            dds_to_ftex(&BC1_DDS[..100], ColorSpace::Linear),
            Err(FtexError::Truncated)
        ));
        // A DX10 extension cut short: at 140 bytes only 12 of its 20 bytes
        // are present.
        let dx10 = ftex_to_dds(BC7).unwrap();
        assert!(matches!(
            dds::read_layout(&dx10[..140]),
            Err(FtexError::Truncated)
        ));

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

    #[test]
    fn mip_size_is_block_rounded_and_scales_with_depth() {
        // BC1: 4x4 blocks of 8 bytes. 4x4 -> 8; 8x8 -> 2*2*8 = 32; 5x5 is
        // block-rounded to 2x2 blocks -> 32; depth 2 doubles -> 16.
        assert_eq!(mip_size(PixelFormat::Bc1, 4, 4, 1, 0), 8);
        assert_eq!(mip_size(PixelFormat::Bc1, 8, 8, 1, 0), 32);
        assert_eq!(mip_size(PixelFormat::Bc1, 5, 5, 1, 0), 32);
        assert_eq!(mip_size(PixelFormat::Bc1, 4, 4, 2, 0), 16);
    }

    #[test]
    fn a_declared_pitch_wider_than_the_tight_row_is_kept() {
        // Uncompressed 32-bit pixels at width 5: the tight row is
        // 5 * 32 / 8 = 20 bytes. A declared pitch of 24 means padded rows and
        // is reported; one equal to 20 means tightly packed (None).
        fn layout(pitch: u32, pitch_flag: bool) -> dds::DdsLayout {
            let mut dds = vec![0u8; 128];
            dds[..4].copy_from_slice(b"DDS ");
            dds[4..8].copy_from_slice(&124u32.to_le_bytes());
            dds[8..12].copy_from_slice(&if pitch_flag { 0x8u32 } else { 0 }.to_le_bytes());
            dds[12..16].copy_from_slice(&2u32.to_le_bytes()); // height
            dds[16..20].copy_from_slice(&5u32.to_le_bytes()); // width
            dds[20..24].copy_from_slice(&pitch.to_le_bytes());
            dds[76..80].copy_from_slice(&32u32.to_le_bytes()); // pixel_format_size
            dds[80..84].copy_from_slice(&0x40u32.to_le_bytes()); // DDPF_RGB
            dds[88..92].copy_from_slice(&32u32.to_le_bytes()); // bit count
            dds[92..96].copy_from_slice(&0xff0000u32.to_le_bytes());
            dds[96..100].copy_from_slice(&0xff00u32.to_le_bytes());
            dds[100..104].copy_from_slice(&0xffu32.to_le_bytes());
            dds[108..112].copy_from_slice(&0x1000u32.to_le_bytes()); // caps1
            dds::read_layout(&dds).unwrap()
        }
        assert_eq!(layout(24, true).row_pitch, Some(24));
        assert_eq!(layout(20, true).row_pitch, None);
        // Without DDSD_PITCH the field is not a pitch at all.
        assert_eq!(layout(24, false).row_pitch, None);
        // The legacy-mask path: a pitch equal to the Argb8 tight row is None.
        assert_eq!(dds::read_layout(RGBA32_DDS).unwrap().row_pitch, None);
    }

    #[test]
    fn chunked_frames_record_offsets_and_pad_to_8() {
        // A 128x128 Argb8 mip is 65536 bytes -> ceil(65536 / 16384) = 4 chunks.
        let mut dds = dds::header_bytes(PixelFormat::Argb8, 128, 128, 1);
        let pixels: Vec<u8> = (0..128 * 128 * 4u32)
            .map(|i| ((i * 40503) >> 8) as u8)
            .collect();
        dds.extend_from_slice(&pixels);
        let ftex = dds_to_ftex(&dds, ColorSpace::Linear).unwrap();
        // The single 16-byte mip record sits at 64: offset u32, uncompressed
        // u32, compressed u32, index u8, ftexs u8, chunk_count u16.
        let record = &ftex[64..80];
        let frame_offset = u32::from_le_bytes(record[0..4].try_into().unwrap()) as usize;
        let uncompressed = u32::from_le_bytes(record[4..8].try_into().unwrap());
        let compressed = u32::from_le_bytes(record[8..12].try_into().unwrap()) as usize;
        let chunk_count = u16::from_le_bytes(record[14..16].try_into().unwrap()) as usize;
        assert_eq!(uncompressed, 65536);
        assert_eq!(chunk_count, 4);
        let frame = &ftex[frame_offset..frame_offset + compressed];
        // Chunk record i sits at 8*i; the chunk bytes start at 8*chunk_count,
        // so each record's offset is that base plus the stored bytes before it.
        let mut expected = 8 * chunk_count;
        for i in 0..chunk_count {
            let chunk = &frame[8 * i..8 * i + 8];
            let stored = u16::from_le_bytes(chunk[0..2].try_into().unwrap()) as usize;
            let piece = u16::from_le_bytes(chunk[2..4].try_into().unwrap());
            let offset = u32::from_le_bytes(chunk[4..8].try_into().unwrap()) as usize;
            assert_eq!(piece, 16384);
            // The pattern compresses, so every chunk went through zlib: a
            // stored size of 16384 would mean it was stored raw.
            assert!(stored < 16384, "chunk {i} stored raw");
            assert_eq!(offset, expected, "chunk {i}");
            expected += stored;
        }
        // The frame is the records plus the chunk bytes, padded to a
        // multiple of 8.
        assert_eq!(compressed, expected + (8 - expected % 8) % 8);
        assert_eq!(compressed % 8, 0);
    }

    #[test]
    fn a_dds_dimension_past_the_ftex_field_is_an_error() {
        // DDS header layout: u32 width at offset 16. 70,000 does not fit u16.
        let mut dds = BC1_DDS.to_vec();
        dds[16..20].copy_from_slice(&70_000u32.to_le_bytes());
        assert!(matches!(
            dds_to_ftex(&dds, ColorSpace::Srgb),
            Err(FtexError::HeaderFieldOverflow {
                what: "width",
                value: 70_000
            })
        ));
    }

    #[test]
    fn a_dds_without_the_mipmap_caps_bit_has_one_level() {
        // BC1_DDS declares three mips; with DDSCAPS_MIPMAP cleared the count
        // field is meaningless and the texture has one level.
        let mut dds = BC1_DDS.to_vec();
        let caps = u32::from_le_bytes(dds[108..112].try_into().unwrap()) & !0x400000;
        dds[108..112].copy_from_slice(&caps.to_le_bytes());
        assert_eq!(dds::read_layout(&dds).unwrap().mipmaps, 1);
        let ftex = dds_to_ftex(&dds, ColorSpace::Linear).unwrap();
        assert_eq!(info(&ftex).unwrap().mipmaps, 1);
    }

    #[test]
    fn read_layout_rejects_dimensions_d3d_would_not_create() {
        // A 16x16 texture supports five levels; a sixth mip has no pixels
        // left to halve.
        let mut dds = BC1_DDS.to_vec();
        dds[28..32].copy_from_slice(&5u32.to_le_bytes());
        assert_eq!(dds::read_layout(&dds).unwrap().mipmaps, 5);
        dds[28..32].copy_from_slice(&6u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&dds),
            Err(FtexError::UnsupportedDds("mip count past the dimensions"))
        ));
        dds[28..32].copy_from_slice(&3u32.to_le_bytes());
        dds[12..16].copy_from_slice(&0u32.to_le_bytes()); // height
        assert!(matches!(
            dds::read_layout(&dds),
            Err(FtexError::UnsupportedDds("zero dimension"))
        ));
    }

    #[test]
    fn mip_size_past_level_31_reads_as_one_block() {
        // The dimension shifted down past level 31 is 0, clamped to 1: the
        // same value the reference's floor division computes.
        assert_eq!(mip_size(PixelFormat::Bc1, 16, 16, 1, 40), 8);
    }

    #[test]
    fn a_mip_chain_past_level_31_converts_without_panic() {
        // dds_to_ftex keeps the source's acceptance (the dimension bounds are
        // read_layout's only): a 16x16 DDS declaring 40 mips, levels 3..39
        // each a single 1x1 block.
        let mut dds = BC1_DDS.to_vec();
        dds[28..32].copy_from_slice(&40u32.to_le_bytes());
        for _ in 3..40 {
            dds.extend_from_slice(&[0u8; 8]);
        }
        let ftex = dds_to_ftex(&dds, ColorSpace::Linear).unwrap();
        assert_eq!(info(&ftex).unwrap().mipmaps, 40);
        assert_eq!(ftex_to_dds(&ftex).unwrap(), dds);
    }

    #[test]
    fn mip_size_saturates_past_usize() {
        // A u32::MAX square cannot fit usize even on 64-bit; the size
        // saturates so the slice lookup it bounds fails as `Truncated`.
        assert_eq!(
            mip_size(PixelFormat::Argb8, u32::MAX, u32::MAX, 1, 0),
            usize::MAX
        );
    }

    #[test]
    fn dx10_volume_declarations_are_rejected() {
        let dx10 = ftex_to_dds(BC7).unwrap();
        // A 3D resource dimension, depth field 1.
        let mut dim4 = dx10.clone();
        dim4[132..136].copy_from_slice(&4u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&dim4),
            Err(FtexError::UnsupportedDds("volume texture"))
        ));
        // A 2D resource dimension but a depth of 2.
        let mut deep = dx10.clone();
        deep[24..28].copy_from_slice(&2u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&deep),
            Err(FtexError::UnsupportedDds("volume texture"))
        ));
    }

    #[test]
    fn dx10_premultiplied_alpha_is_refused() {
        // misc_flags2's low three bits are the alpha mode: 2 is
        // premultiplied, which the straight-alpha decode cannot honour.
        let dx10 = ftex_to_dds(BC7).unwrap();
        let mut premultiplied = dx10.clone();
        premultiplied[144..148].copy_from_slice(&2u32.to_le_bytes());
        assert!(matches!(
            dds::read_layout(&premultiplied),
            Err(FtexError::UnsupportedDds("premultiplied alpha"))
        ));
        // Straight (1) and opaque (3) still read.
        for mode in [1u32, 3] {
            let mut dds = dx10.clone();
            dds[144..148].copy_from_slice(&mode.to_le_bytes());
            assert_eq!(
                dds::read_layout(&dds).unwrap().pixel,
                dds::DdsPixel::Format(PixelFormat::Bc7),
                "mode {mode}"
            );
        }
    }

    /// A minimal 128-byte legacy-header DDS: `format_flags` carries the
    /// DDPF bits, `fourcc` the format code, `masks` the channel masks.
    fn legacy_dds(format_flags: u32, fourcc: [u8; 4], bit_count: u32, masks: [u32; 4]) -> Vec<u8> {
        let mut dds = vec![0u8; 128];
        dds[..4].copy_from_slice(b"DDS ");
        dds[4..8].copy_from_slice(&124u32.to_le_bytes());
        dds[12..16].copy_from_slice(&1u32.to_le_bytes()); // height
        dds[16..20].copy_from_slice(&1u32.to_le_bytes()); // width
        dds[76..80].copy_from_slice(&32u32.to_le_bytes()); // pixel format size
        dds[80..84].copy_from_slice(&format_flags.to_le_bytes());
        dds[84..88].copy_from_slice(&fourcc);
        dds[88..92].copy_from_slice(&bit_count.to_le_bytes());
        for (i, mask) in masks.iter().enumerate() {
            dds[92 + 4 * i..96 + 4 * i].copy_from_slice(&mask.to_le_bytes());
        }
        dds[108..112].copy_from_slice(&0x1000u32.to_le_bytes()); // caps1
        dds
    }

    #[test]
    fn uncompressed_mask_layouts_classify() {
        const ARGB8: [u32; 4] = [0xff0000, 0xff00, 0xff, 0xff000000];
        // The canonical A8R8G8B8 header.
        assert_eq!(
            dds::read_layout(&legacy_dds(0x41, [0; 4], 32, ARGB8))
                .unwrap()
                .pixel,
            dds::DdsPixel::Format(PixelFormat::Argb8)
        );
        // Each condition false alone falls through to the mask-described
        // layout: rgb flag, alpha flag, then each mask.
        for (flags, masks) in [
            (0x1, ARGB8),
            (0x40, ARGB8),
            (0x41, [0xfe0000, 0xff00, 0xff, 0xff000000]),
            (0x41, [0xff0000, 0xfe00, 0xff, 0xff000000]),
            (0x41, [0xff0000, 0xff00, 0xfe, 0xff000000]),
            (0x41, [0xff0000, 0xff00, 0xff, 0xfe000000]),
        ] {
            assert_eq!(
                dds::read_layout(&legacy_dds(flags, [0; 4], 32, masks))
                    .unwrap()
                    .pixel,
                dds::DdsPixel::Uncompressed {
                    bit_count: 32,
                    r_mask: masks[0],
                    g_mask: masks[1],
                    b_mask: masks[2],
                    a_mask: masks[3],
                }
            );
        }
        // The FTEX direction accepts the canonical masks only with both
        // flags set.
        assert!(matches!(
            dds_to_ftex(&legacy_dds(0x40, [0; 4], 32, ARGB8), ColorSpace::Linear),
            Err(FtexError::UnsupportedDds("unrecognized uncompressed masks"))
        ));
        // The canonical luminance header (a legacy-header R8).
        const L8: [u32; 4] = [0xff, 0, 0, 0];
        assert_eq!(
            dds::read_layout(&legacy_dds(0x20000, [0; 4], 8, L8))
                .unwrap()
                .pixel,
            dds::DdsPixel::Format(PixelFormat::R8)
        );
        // The luminance flag cleared, and each mask condition false alone.
        for (flags, masks) in [
            (0, L8),
            (0x20000, [0, 0, 0, 0]),
            (0x20000, [0xff, 1, 0, 0]),
            (0x20000, [0xff, 0, 1, 0]),
            (0x20000, [0xff, 0, 0, 1]),
        ] {
            assert_eq!(
                dds::read_layout(&legacy_dds(flags, [0; 4], 8, masks))
                    .unwrap()
                    .pixel,
                dds::DdsPixel::Uncompressed {
                    bit_count: 8,
                    r_mask: masks[0],
                    g_mask: masks[1],
                    b_mask: masks[2],
                    a_mask: masks[3],
                }
            );
        }
    }

    #[test]
    fn fourcc_table_matches_the_reference() {
        for (fourcc, format) in [
            (*b"8888", PixelFormat::Argb8),
            (*b"DXT1", PixelFormat::Bc1),
            (*b"DXT3", PixelFormat::Bc2),
            (*b"DXT5", PixelFormat::Bc3),
            (*b"ATI1", PixelFormat::Bc4),
            (*b"ATI2", PixelFormat::Bc5),
            (*b"BC5U", PixelFormat::Bc5),
        ] {
            assert_eq!(
                dds::read_layout(&legacy_dds(0x4, fourcc, 0, [0; 4]))
                    .unwrap()
                    .pixel,
                dds::DdsPixel::Format(format),
                "{fourcc:?}"
            );
        }
        assert!(matches!(
            dds::read_layout(&legacy_dds(0x4, *b"NOPE", 0, [0; 4])),
            Err(FtexError::UnsupportedDds("unrecognized fourcc"))
        ));
    }

    #[test]
    fn dxgi_table_matches_dxgiformat_h() {
        // A DX10 header built on the BC7 fixture's conversion; only the dxgi
        // field (offset 128) changes.
        let dx10 = ftex_to_dds(BC7).unwrap();
        let with_dxgi = |dxgi: u32| {
            let mut dds = dx10.clone();
            dds[128..132].copy_from_slice(&dxgi.to_le_bytes());
            dds
        };
        for (dxgi, format) in [
            (87, PixelFormat::Argb8),
            (91, PixelFormat::Argb8),
            (61, PixelFormat::R8),
            (71, PixelFormat::Bc1),
            (72, PixelFormat::Bc1),
            (74, PixelFormat::Bc2),
            (75, PixelFormat::Bc2),
            (77, PixelFormat::Bc3),
            (78, PixelFormat::Bc3),
            (80, PixelFormat::Bc4),
            (83, PixelFormat::Bc5),
            (95, PixelFormat::Bc6h),
            (98, PixelFormat::Bc7),
            (99, PixelFormat::Bc7),
            (10, PixelFormat::Rgba16F),
            (2, PixelFormat::Rgba32F),
            (24, PixelFormat::Rgb10A2),
            (26, PixelFormat::Rg11B10F),
        ] {
            assert_eq!(
                dds::read_layout(&with_dxgi(dxgi)).unwrap().pixel,
                dds::DdsPixel::Format(format),
                "dxgi {dxgi}"
            );
        }
        // 28/29 (R8G8B8A8) have no FTEX twin and stay masks.
        for dxgi in [28, 29] {
            assert_eq!(
                dds::read_layout(&with_dxgi(dxgi)).unwrap().pixel,
                dds::DdsPixel::Uncompressed {
                    bit_count: 32,
                    r_mask: 0xff,
                    g_mask: 0xff00,
                    b_mask: 0xff0000,
                    a_mask: 0xff000000,
                },
                "dxgi {dxgi}"
            );
        }
        // The signed block formats are refused; 96 is BC6H_SF16, not an
        // sRGB twin of 95.
        for dxgi in [81, 84, 96] {
            assert!(
                matches!(
                    dds::read_layout(&with_dxgi(dxgi)),
                    Err(FtexError::UnsupportedDds("signed block format"))
                ),
                "dxgi {dxgi}"
            );
        }
    }

    #[test]
    fn pixel_format_ids_and_fourccs_match_the_reference() {
        // The id table, as the format comment at the top of the reference
        // lists it.
        for (id, format) in [
            (0, PixelFormat::Argb8),
            (1, PixelFormat::R8),
            (2, PixelFormat::Bc1),
            (3, PixelFormat::Bc2),
            (4, PixelFormat::Bc3),
            (8, PixelFormat::Bc4),
            (9, PixelFormat::Bc5),
            (10, PixelFormat::Bc6h),
            (11, PixelFormat::Bc7),
            (12, PixelFormat::Rgba16F),
            (13, PixelFormat::Rgba32F),
            (14, PixelFormat::Rgb10A2),
            (15, PixelFormat::Rg11B10F),
        ] {
            assert_eq!(PixelFormat::from_id(id), Some(format), "id {id}");
            assert_eq!(format.id(), id, "format {format:?}");
        }
        for id in [5u16, 6, 7, 16] {
            assert_eq!(PixelFormat::from_id(id), None, "id {id}");
        }
        // The legacy formats carry their FourCC; the DXGI-mapped ones write
        // DX10; Argb8 writes masks instead.
        for (format, fourcc) in [
            (PixelFormat::Bc1, *b"DXT1"),
            (PixelFormat::Bc2, *b"DXT3"),
            (PixelFormat::Bc3, *b"DXT5"),
        ] {
            assert_eq!(format.fourcc(), Some(fourcc), "{format:?}");
        }
        assert_eq!(PixelFormat::Argb8.fourcc(), None);
        for format in [
            PixelFormat::R8,
            PixelFormat::Bc4,
            PixelFormat::Bc5,
            PixelFormat::Bc6h,
            PixelFormat::Bc7,
            PixelFormat::Rgba16F,
            PixelFormat::Rgba32F,
            PixelFormat::Rgb10A2,
            PixelFormat::Rg11B10F,
        ] {
            assert_eq!(format.fourcc(), Some(*b"DX10"), "{format:?}");
        }
    }

    #[test]
    fn bc2_converts_with_the_dxt3_fourcc() {
        // The BC3 fixture with its pixel-format id patched 4 -> 3: the same
        // block size, so only the FourCC differs in the DDS.
        let mut ftex = BC3_NRM.to_vec();
        ftex[8..10].copy_from_slice(&3u16.to_le_bytes());
        let mut expected = BC3_NRM_DDS.to_vec();
        expected[84..88].copy_from_slice(b"DXT3");
        assert_eq!(ftex_to_dds(&ftex).unwrap(), expected);
    }

    #[test]
    fn volume_texture_round_trips_through_the_reference_layout() {
        // A 4x4x4 R16G16B16A16_FLOAT volume texture, converted to FTEX and
        // back by the reference implementation.
        assert_eq!(ftex_to_dds(VOLUME).unwrap(), VOLUME_DDS);

        let ours = dds_to_ftex(VOLUME_DDS, ColorSpace::Linear).unwrap();
        assert_eq!(&ours[..64], &VOLUME[..64]);
        // Same mip count -> same number of 16-byte records; uncompressed
        // size, index, ftexs and chunk count match the reference's records.
        let mip_count = usize::from(info(VOLUME).unwrap().mipmaps);
        for i in 0..mip_count {
            let ours_rec = &ours[64 + 16 * i..64 + 16 * i + 16];
            let pft_rec = &VOLUME[64 + 16 * i..64 + 16 * i + 16];
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
        assert_eq!(ftex_to_dds(&ours).unwrap(), VOLUME_DDS);
    }

    #[test]
    fn dds_to_ftex_rejects_partial_cube_and_cube_volume() {
        let mut dds = BC1_DDS.to_vec();
        // The cube-map bit plus only one face bit: an incomplete cube.
        dds[112..116].copy_from_slice(&(0x200u32 | 0x400).to_le_bytes());
        assert!(matches!(
            dds_to_ftex(&dds, ColorSpace::Linear),
            Err(FtexError::UnsupportedDds("incomplete cube map"))
        ));
        // All six faces plus the volume bit and a depth above 1.
        dds[112..116].copy_from_slice(&(0xfe00u32 | 0x200000).to_le_bytes());
        dds[24..28].copy_from_slice(&2u32.to_le_bytes());
        assert!(matches!(
            dds_to_ftex(&dds, ColorSpace::Linear),
            Err(FtexError::UnsupportedDds("cube map with volume depth"))
        ));
        // Without the volume bit the depth field is ignored: a DDS declaring
        // depth 0 still produces FTEX depth 1.
        let mut flat = BC1_DDS.to_vec();
        flat[24..28].copy_from_slice(&0u32.to_le_bytes());
        let ftex = dds_to_ftex(&flat, ColorSpace::Linear).unwrap();
        assert_eq!(info(&ftex).unwrap().depth, 1);
    }

    #[test]
    fn ftex_version_follows_the_pixel_format_id() {
        // Format ids above 4 write version 2.04, the legacy formats 2.03.
        let bc7_dds = ftex_to_dds(BC7).unwrap();
        let ftex = dds_to_ftex(&bc7_dds, ColorSpace::Linear).unwrap();
        assert!((info(&ftex).unwrap().version - 2.04).abs() < 0.001);
        let ftex = dds_to_ftex(BC1_DDS, ColorSpace::Linear).unwrap();
        assert!((info(&ftex).unwrap().version - 2.03).abs() < 0.001);
    }

    /// A 16-byte piece whose level-3 zlib stream is also exactly 16 bytes:
    /// the case `encode_image` must store raw, because readers take equal
    /// sizes to mean "stored". Found by scanning generated pieces with a
    /// varying number of zero bytes, which sweeps the compressed size.
    fn piece_with_equal_sized_zlib() -> [u8; 16] {
        use std::io::Write;
        (0u64..)
            .flat_map(|seed| {
                (0..16).map(move |zeros| {
                    std::array::from_fn::<u8, 16, _>(|i| {
                        if i < zeros {
                            0
                        } else {
                            (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (i * 4 % 24)) as u8
                        }
                    })
                })
            })
            .find(|piece| {
                let mut encoder =
                    flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(3));
                encoder.write_all(piece).unwrap();
                encoder.finish().unwrap().len() == piece.len()
            })
            .expect("some 16-byte piece compresses to exactly 16 bytes")
    }

    #[test]
    fn a_chunk_whose_zlib_stream_matches_its_size_stores_raw() {
        let piece = piece_with_equal_sized_zlib();
        // A 4x4 BC3 mip is exactly 16 bytes: one chunk.
        let mut dds = dds::header_bytes(PixelFormat::Bc3, 4, 4, 1);
        dds.extend_from_slice(&piece);
        let ftex = dds_to_ftex(&dds, ColorSpace::Normal).unwrap();
        // The single mip record points at the frame, which opens with its
        // one chunk record: stored size equal to the piece size means raw.
        let record = &ftex[64..80];
        let offset = u32::from_le_bytes(record[0..4].try_into().unwrap()) as usize;
        let chunk = &ftex[offset..offset + 8];
        assert_eq!(u16::from_le_bytes(chunk[0..2].try_into().unwrap()), 16);
        assert_eq!(u16::from_le_bytes(chunk[2..4].try_into().unwrap()), 16);
        assert_eq!(&ftex[offset + 8..offset + 24], &piece[..]);
        assert_eq!(ftex_to_dds(&ftex).unwrap(), dds);
    }

    #[test]
    fn ftex_header_variants_error() {
        // A cube map cannot carry depth.
        let mut cube = CUBE.to_vec();
        cube[14..16].copy_from_slice(&2u16.to_le_bytes());
        assert!(matches!(
            ftex_to_dds(&cube),
            Err(FtexError::UnsupportedVariant("cube map with depth > 1"))
        ));
        // ftexs-linked textures are not read.
        let mut linked = BC1.to_vec();
        linked[32] = 1; // ftexs_count
        assert!(matches!(
            ftex_to_dds(&linked),
            Err(FtexError::UnsupportedVariant("ftexs count > 0"))
        ));
    }

    #[test]
    fn chunkless_frames_convert() {
        // Frames stored raw (chunk count 0, compressed size 0) and as a
        // single zlib stream (chunk count 0, compressed size > 0).
        assert_eq!(ftex_to_dds(RAW_FRAMES).unwrap(), BC1_DDS);
        assert_eq!(ftex_to_dds(SINGLE_ZLIB).unwrap(), BC1_DDS);
    }

    #[test]
    fn a_chunk_past_the_filled_frame_is_not_read() {
        // BC1's first frame gets one extra chunk record, dangling past the
        // end of the file; the frame's own chunks already fill it, so the
        // dangling record is never consulted.
        let record = |index: usize| &BC1[64 + 16 * index..80 + 16 * index];
        let frame_offset =
            |index: usize| u32::from_le_bytes(record(index)[..4].try_into().unwrap()) as usize;
        // Konami stores the frames smallest-first: the end of each frame is
        // the next offset in sorted order (or the end of the file).
        let mut offsets: Vec<usize> = (0..3).map(&frame_offset).collect();
        offsets.sort_unstable();
        let end = |start: usize| {
            offsets
                .iter()
                .copied()
                .find(|offset| *offset > start)
                .unwrap_or(BC1.len())
        };
        let frames: Vec<&[u8]> = (0..3)
            .map(|index| &BC1[frame_offset(index)..end(frame_offset(index))])
            .collect();
        let chunk_count = u16::from_le_bytes(record(0)[14..16].try_into().unwrap()) as usize;

        let mut ftex = BC1[..64].to_vec();
        let mut position = (64 + 3 * 16) as u32;
        for index in 0..3 {
            let mut patched = record(index).to_vec();
            patched[..4].copy_from_slice(&position.to_le_bytes());
            if index == 0 {
                patched[14..16].copy_from_slice(&((chunk_count as u16 + 1).to_le_bytes()));
                // The frame blob grew by the record's eight bytes.
                let size = u32::from_le_bytes(patched[8..12].try_into().unwrap()) + 8;
                patched[8..12].copy_from_slice(&size.to_le_bytes());
                position += frames[0].len() as u32 + 8;
            } else {
                position += frames[index].len() as u32;
            }
            ftex.extend_from_slice(&patched);
        }
        // The widened chunk table shifts every stored offset by a record.
        let mut table = frames[0][..8 * chunk_count].to_vec();
        for index in 0..chunk_count {
            let offset =
                u32::from_le_bytes(table[8 * index + 4..8 * index + 8].try_into().unwrap()) + 8;
            table[8 * index + 4..8 * index + 8].copy_from_slice(&offset.to_le_bytes());
        }
        ftex.extend_from_slice(&table);
        ftex.extend_from_slice(&8u16.to_le_bytes()); // dangling compressed size
        ftex.extend_from_slice(&8u16.to_le_bytes()); // dangling uncompressed size
        ftex.extend_from_slice(&0x1000000u32.to_le_bytes()); // dangling offset
        ftex.extend_from_slice(&frames[0][8 * chunk_count..]);
        ftex.extend_from_slice(frames[1]);
        ftex.extend_from_slice(frames[2]);

        assert_eq!(ftex_to_dds(&ftex).unwrap(), BC1_DDS);
    }

    #[test]
    fn a_single_zlib_frame_inflates_only_to_its_expected_size() {
        use std::io::Write as _;
        // One mip stored as a single zlib stream that inflates to a
        // megabyte, far past the 128-byte BC1 mip it describes. The stream
        // is cut after the bytes the frame needs, so only a bounded
        // inflate can finish.
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(3));
        encoder.write_all(&[0xABu8; 1 << 20]).unwrap();
        let mut stream = encoder.finish().unwrap();
        stream.truncate(stream.len() / 2);

        let mut ftex = BC1[..64].to_vec();
        ftex[16] = 1; // one mip level
        ftex.extend_from_slice(&80u32.to_le_bytes()); // frame offset
        ftex.extend_from_slice(&128u32.to_le_bytes()); // uncompressed size
        ftex.extend_from_slice(&(stream.len() as u32).to_le_bytes()); // compressed size
        ftex.extend_from_slice(&[0u8, 0]); // mip index, ftexs number
        ftex.extend_from_slice(&0u16.to_le_bytes()); // chunk count: one stream
        ftex.extend_from_slice(&stream);

        let dds = ftex_to_dds(&ftex).unwrap();
        assert_eq!(dds.len(), 128 + 128);
        assert_eq!(dds[128..].to_vec(), vec![0xABu8; 128]);
    }

    #[test]
    fn a_chunk_inflates_only_the_bytes_the_frame_needs() {
        use std::io::Write as _;
        // The frame's first raw chunk leaves four bytes; the second chunk's
        // stream is cut after the first bytes it inflates, so only an
        // inflate bounded to the bytes still needed can finish.
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(3));
        let content: Vec<u8> = (0..512u16).map(|i| (i % 256) as u8).collect();
        encoder.write_all(&content).unwrap();
        let mut stream = encoder.finish().unwrap();
        stream.truncate(stream.len() / 2);

        let mut ftex = BC1[..64].to_vec();
        ftex[16] = 1; // one mip level
        ftex.extend_from_slice(&80u32.to_le_bytes()); // frame offset
        ftex.extend_from_slice(&128u32.to_le_bytes()); // uncompressed size
        ftex.extend_from_slice(&0u32.to_le_bytes()); // compressed size
        ftex.extend_from_slice(&[0u8, 0]); // mip index, ftexs number
        ftex.extend_from_slice(&2u16.to_le_bytes()); // chunk count
        // Chunk records: a raw 124-byte piece, then the broken stream.
        ftex.extend_from_slice(&124u16.to_le_bytes()); // stored
        ftex.extend_from_slice(&124u16.to_le_bytes()); // uncompressed
        ftex.extend_from_slice(&16u32.to_le_bytes()); // after the two records
        ftex.extend_from_slice(&(stream.len() as u16).to_le_bytes());
        ftex.extend_from_slice(&512u16.to_le_bytes());
        ftex.extend_from_slice(&140u32.to_le_bytes()); // after the raw piece
        ftex.extend_from_slice(&[0xEEu8; 124]);
        ftex.extend_from_slice(&stream);

        let dds = ftex_to_dds(&ftex).unwrap();
        assert_eq!(
            dds[128..].to_vec(),
            [&[0xEEu8; 124][..], &content[..4][..]].concat()
        );
    }

    #[test]
    fn dds_to_ftex_rejects_a_row_pitch_past_the_tight_row() {
        // A 2x2 DX10 R8 source declaring pitch 4 when its tight row is 2:
        // the padding would land in the pixels, and FTEX has no pitch
        // field to carry it.
        let mut dds = Vec::new();
        dds.extend_from_slice(b"DDS ");
        dds.extend_from_slice(&124u32.to_le_bytes());
        dds.extend_from_slice(&(0x1u32 | 0x2 | 0x4 | 0x8 | 0x1000).to_le_bytes());
        dds.extend_from_slice(&2u32.to_le_bytes()); // height
        dds.extend_from_slice(&2u32.to_le_bytes()); // width
        dds.extend_from_slice(&4u32.to_le_bytes()); // declared pitch
        dds.extend_from_slice(&0u32.to_le_bytes()); // depth
        dds.extend_from_slice(&1u32.to_le_bytes()); // mipmaps
        dds.extend_from_slice(&[0u8; 44]);
        dds.extend_from_slice(&32u32.to_le_bytes()); // pixel format size
        dds.extend_from_slice(&0x4u32.to_le_bytes()); // DDPF_FOURCC
        dds.extend_from_slice(b"DX10");
        dds.extend_from_slice(&[0u8; 20]); // bit count and masks
        dds.extend_from_slice(&0x1000u32.to_le_bytes()); // caps1: texture
        dds.extend_from_slice(&[0u8; 16]);
        dds.extend_from_slice(&61u32.to_le_bytes()); // DXGI R8_UNORM
        dds.extend_from_slice(&3u32.to_le_bytes()); // 2D
        dds.extend_from_slice(&0u32.to_le_bytes()); // misc flags
        dds.extend_from_slice(&1u32.to_le_bytes()); // array size
        dds.extend_from_slice(&0u32.to_le_bytes()); // misc flags 2
        // Two rows padded to four bytes.
        dds.extend_from_slice(&[10, 20, 0, 0, 30, 40, 0, 0]);
        assert!(matches!(
            dds_to_ftex(&dds, ColorSpace::Linear),
            Err(FtexError::UnsupportedDds("padded rows"))
        ));
        // Without DDSD_PITCH the field is a linear size, not a row pitch:
        // the same header converts.
        dds[8..12].copy_from_slice(&(0x1u32 | 0x2 | 0x4 | 0x1000).to_le_bytes());
        assert!(dds_to_ftex(&dds, ColorSpace::Linear).is_ok());
    }
}
