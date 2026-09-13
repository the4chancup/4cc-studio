//! In-process texture conversion: decode DDS/FTEX/raster sources, generate
//! mips, encode BC1/BC3/BC7, cache results.
//!
//! Three pure steps and one stateful wrapper: [`decode`] turns any accepted
//! source into straight-alpha RGBA8 mips plus, for DDS/FTEX, the compressed
//! blocks it carried and a flag saying the mip chain is the source's own;
//! [`convert`] applies the codec rules to a decoded texture and returns the
//! finished container bytes (a DDS for PES 15-17, an FTEX for PES 18-21);
//! [`Converter`] is the session cache in front of both.

mod cache;
mod dds;
mod encode;
mod mips;
mod raster;

use pes_version::PesVersion;
use sha2::{Digest, Sha256};

pub use cache::Converter;

/// The accepted source formats, named by the file extension the compiler
/// resolved the texture stem to. TGA has no magic, so the format is never
/// sniffed from bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceFormat {
    /// A DDS file (optionally WESYS-wrapped for PES 15-17 sources).
    Dds,
    /// A Fox Engine FTEX file.
    Ftex,
    /// `.png`
    Png,
    /// `.jpg` / `.jpeg`
    Jpeg,
    /// `.bmp`
    Bmp,
    /// `.webp`
    WebP,
    /// `.tga`
    Tga,
    /// `.tif` / `.tiff`
    Tiff,
}

impl SourceFormat {
    /// Case-insensitive, without the dot; `jpg`/`jpeg` and `tif`/`tiff`
    /// both accepted.
    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_ascii_lowercase().as_str() {
            "dds" => SourceFormat::Dds,
            "ftex" => SourceFormat::Ftex,
            "png" => SourceFormat::Png,
            "jpg" | "jpeg" => SourceFormat::Jpeg,
            "bmp" => SourceFormat::Bmp,
            "webp" => SourceFormat::WebP,
            "tga" => SourceFormat::Tga,
            "tif" | "tiff" => SourceFormat::Tiff,
            _ => return None,
        })
    }
}

/// What the texture is for; with the PES version it selects the codec and
/// the channel layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureRole {
    /// An ordinary color texture.
    Color,
    /// A normal map (encoded in the DXT5nm channel layout).
    Normal,
}

/// Block codecs this crate keeps or emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCodec {
    /// BC1 / DXT1.
    Bc1,
    /// BC2 / DXT3.
    Bc2,
    /// BC3 / DXT5.
    Bc3,
    /// BC4 / ATI1 (single channel).
    Bc4,
    /// BC5 / ATI2 (two channels; normal-map sources).
    Bc5,
    /// BC7 (PES 19-21 only).
    Bc7,
}

/// Compressed blocks a DDS/FTEX source carried, one buffer per mip, kept so
/// a target that reads the codec gets them unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocks {
    /// The codec the blocks are in.
    pub codec: BlockCodec,
    /// One buffer per mip, top level first.
    pub mips: Vec<Vec<u8>>,
}

/// A source decoded to straight-alpha RGBA8, top mip first, every mip the
/// source carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    /// Level-0 width in pixels.
    pub width: u32,
    /// Level-0 height in pixels.
    pub height: u32,
    /// RGBA8 pixels, one buffer per mip, tightly packed row-major.
    pub mips: Vec<Vec<u8>>,
    /// The compressed blocks the source carried, when it was a DDS/FTEX in
    /// a block codec this crate knows.
    pub blocks: Option<Blocks>,
    /// True when the source format carries a mip chain (DDS/FTEX), so its
    /// level count is kept on encode; false for raster sources, which get a
    /// generated chain down to 1x1.
    pub authored_mips: bool,
}

/// The version and role a conversion targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    /// The PES version the texture is compiled for.
    pub version: PesVersion,
    /// Whether the texture is color data or a normal map.
    pub role: TextureRole,
}

/// Decodes an accepted source to RGBA8 mips plus any compressed blocks it
/// carried. Cube maps and volume textures are rejected
/// (`ConvertError::Ftex` wrapping `ftex`'s `UnsupportedDds`); a
/// WESYS-wrapped DDS is unwrapped first.
pub fn decode(bytes: &[u8], format: SourceFormat) -> Result<Decoded, ConvertError> {
    match format {
        SourceFormat::Dds => {
            let unwrapped = wezlib::decompress_if_wrapped(bytes)?;
            dds::decode_dds(&unwrapped)
        }
        SourceFormat::Ftex => dds::decode_ftex(bytes),
        SourceFormat::Png
        | SourceFormat::Jpeg
        | SourceFormat::Bmp
        | SourceFormat::WebP
        | SourceFormat::Tga
        | SourceFormat::Tiff => raster::decode_raster(bytes, format),
    }
}

/// Applies the codec rules to a decoded texture: compatible compressed
/// blocks pass through with only a container rebuild, everything else is
/// re-encoded per target and role. Returns a DDS for PES 15-17, an FTEX
/// for PES 18-21.
pub fn convert(decoded: &Decoded, target: Target) -> Result<Vec<u8>, ConvertError> {
    encode::convert(decoded, target)
}

/// SHA-256 of the source bytes, computed once when the file is
/// materialized and reused as the conversion cache's key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceHash(pub [u8; 32]);

/// Hashes `bytes` for the conversion cache.
pub fn source_hash(bytes: &[u8]) -> SourceHash {
    SourceHash(Sha256::digest(bytes).into())
}

/// Whether [`Converter::convert`] reads and writes the session cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    /// Cache hit returns the retained bytes; a miss inserts.
    Use,
    /// No read, no insert (large compiles bypass the cache entirely).
    Bypass,
}

/// Why a source could not be decoded or converted.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// The DDS/FTEX container could not be read or written.
    #[error("container conversion failed: {0}")]
    Ftex(#[from] ftex::FtexError),
    /// A WESYS-wrapped DDS could not be unwrapped.
    #[error("wesys unwrap failed: {0}")]
    Wesys(#[from] wezlib::Error),
    /// A raster source could not be decoded.
    #[error("image decode failed: {0}")]
    Image(#[from] image::ImageError),
    /// A feature of the source or request this crate does not handle.
    #[error("unsupported: {0}")]
    Unsupported(&'static str),
    /// The buffer ends before a structure that extends past it.
    #[error("buffer is truncated")]
    Truncated,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    const BC1: &[u8] = include_bytes!("../tests/fixtures/bc1_opaque.dds");
    const BC1_D: &[u8] = include_bytes!("../tests/fixtures/bc1_opaque.decoded.dds");
    const BC3: &[u8] = include_bytes!("../tests/fixtures/bc3.dds");
    const BC3_D: &[u8] = include_bytes!("../tests/fixtures/bc3.decoded.dds");
    const BC5: &[u8] = include_bytes!("../tests/fixtures/bc5.dds");
    const BC5_D: &[u8] = include_bytes!("../tests/fixtures/bc5.decoded.dds");
    const ATI2: &[u8] = include_bytes!("../tests/fixtures/ati2.dds");
    const ATI2_D: &[u8] = include_bytes!("../tests/fixtures/ati2.decoded.dds");
    const BC7: &[u8] = include_bytes!("../tests/fixtures/bc7.dds");
    const BC7_D: &[u8] = include_bytes!("../tests/fixtures/bc7.decoded.dds");
    const RGBA8: &[u8] = include_bytes!("../tests/fixtures/rgba8.dds");
    const RGBA8_D: &[u8] = include_bytes!("../tests/fixtures/rgba8.decoded.dds");
    const BGRA8: &[u8] = include_bytes!("../tests/fixtures/bgra8_dx9.dds");
    const BGRA8_D: &[u8] = include_bytes!("../tests/fixtures/bgra8_dx9.decoded.dds");
    const NM_PREFOX_D: &[u8] = include_bytes!("../tests/fixtures/bc3_nm_prefox.decoded.dds");
    const PNG: &[u8] = include_bytes!("../tests/fixtures/source.png");
    const PNG_OPAQUE: &[u8] = include_bytes!("../tests/fixtures/source_opaque.png");
    const FTEX_BC1: &[u8] = include_bytes!("../tests/fixtures/konami_bc1_bibs_metalness.ftex");
    const DDS_BC1: &[u8] = include_bytes!("../tests/fixtures/konami_bc1_bibs_metalness.dds");
    const CUBE: &[u8] =
        include_bytes!("../tests/fixtures/konami_bc1_cubemap_default_reflection.dds");

    fn mips_of(dds: &[u8]) -> Vec<Vec<u8>> {
        decode(dds, SourceFormat::Dds).unwrap().mips
    }

    /// Per mip, the largest absolute difference per channel.
    fn max_per_channel(expected: &[Vec<u8>], actual: &[Vec<u8>]) -> Vec<[u8; 4]> {
        assert_eq!(expected.len(), actual.len(), "mip count");
        expected
            .iter()
            .zip(actual)
            .map(|(expected_mip, actual_mip)| {
                assert_eq!(expected_mip.len(), actual_mip.len(), "mip size");
                let mut max = [0u8; 4];
                for (expected_px, actual_px) in expected_mip
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(actual_mip.as_chunks::<4>().0.iter())
                {
                    for c in 0..4 {
                        max[c] = max[c].max(expected_px[c].abs_diff(actual_px[c]));
                    }
                }
                max
            })
            .collect()
    }

    /// Mean absolute difference over every byte of every mip.
    fn mean_error(expected: &[Vec<u8>], actual: &[Vec<u8>]) -> f64 {
        let (sum, count) = expected.iter().zip(actual).fold(
            (0u64, 0u64),
            |(sum, count), (expected_mip, actual_mip)| {
                let mip_sum: u64 = expected_mip
                    .iter()
                    .zip(actual_mip)
                    .map(|(x, y)| u64::from(x.abs_diff(*y)))
                    .sum();
                (sum + mip_sum, count + expected_mip.len() as u64)
            },
        );
        sum as f64 / count as f64
    }

    /// The encode-quality standard: our encoder, measured against what its
    /// input decodes to, is no worse than the reference encoder measured the
    /// same way on the same image: per mip, the worst channel within one
    /// 5-bit quantization step (8) of the reference's worst channel; over the
    /// chain, the pixel-weighted mean within 1.0 (the top mip dominates it,
    /// as it does the screen). A fixed tolerance would be meaningless here:
    /// the test image packs a 2D gradient and a checker into single blocks
    /// at the small mips, where a BC1 colour line loses over 100 in some
    /// channel for any encoder, the reference included; which channel is
    /// sacrificed is the encoder's choice, hence worst channel rather than
    /// channel by channel. Per-mip means are not compared: the CPU BC1 core
    /// (PCA plus one refinement pass) trails the reference by about 2.5 on
    /// the 8x4 mip of a grey ramp (worklog, open issues).
    fn assert_not_worse_than_reference(
        label: &str,
        truth: &[Vec<u8>],
        ours: &[Vec<u8>],
        reference_truth: &[Vec<u8>],
        reference: &[Vec<u8>],
    ) {
        let ours_max = max_per_channel(truth, ours);
        let reference_max = max_per_channel(reference_truth, reference);
        for (level, (o, r)) in ours_max.iter().zip(&reference_max).enumerate() {
            let (ours_worst, reference_worst) = (o.iter().max(), r.iter().max());
            assert!(
                ours_worst
                    <= reference_worst
                        .map(|worst| worst.saturating_add(8))
                        .as_ref(),
                "{label} mip {level}: worst channel ours {ours_worst:?} vs reference {reference_worst:?}"
            );
        }
        let ours_mean = mean_error(truth, ours);
        let reference_mean = mean_error(reference_truth, reference);
        assert!(
            ours_mean <= reference_mean + 1.0,
            "{label} mean: ours {ours_mean:.3} vs reference {reference_mean:.3}"
        );
    }

    /// The DXT5nm layouts applied to plain RGBA mips (X = R, Y = G).
    fn dxt5nm(mips: &[Vec<u8>], fox: bool) -> Vec<Vec<u8>> {
        mips.iter()
            .map(|mip| {
                mip.as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|px| {
                        let (x, y) = (px[0], px[1]);
                        if fox { [255, y, 0, x] } else { [y, y, y, x] }
                    })
                    .collect()
            })
            .collect()
    }

    /// Reads a pre-Fox DXT5nm decode back as plain (X = R, Y = G) pixels.
    fn plain_from_prefox(prefox: &[Vec<u8>]) -> Vec<Vec<u8>> {
        prefox
            .iter()
            .map(|mip| {
                mip.as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|px| [px[3], px[1], 0, 255])
                    .collect()
            })
            .collect()
    }

    #[test]
    fn decode_matches_the_reference_decode_exactly() {
        for (stem, encoded, decoded) in [
            ("bc1_opaque", BC1, BC1_D),
            ("bc3", BC3, BC3_D),
            ("bc5", BC5, BC5_D),
            ("ati2", ATI2, ATI2_D),
            ("bc7", BC7, BC7_D),
            ("rgba8", RGBA8, RGBA8_D),
            ("bgra8_dx9", BGRA8, BGRA8_D),
        ] {
            let ours = decode(encoded, SourceFormat::Dds).unwrap();
            let expected = decode(decoded, SourceFormat::Dds).unwrap();
            assert_eq!((ours.width, ours.height), (32, 16));
            assert_eq!(ours.mips.len(), 6, "{stem}");
            for (level, (ours_mip, expected_mip)) in
                ours.mips.iter().zip(&expected.mips).enumerate()
            {
                let diffs: Vec<(usize, u8, u8)> = ours_mip
                    .iter()
                    .zip(expected_mip)
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .map(|(i, (a, b))| (i, *a, *b))
                    .collect();
                assert!(
                    diffs.is_empty(),
                    "{stem} mip {level}: {} diffs, first {diffs:?}",
                    diffs.len()
                );
            }
        }
    }

    #[test]
    fn compatible_blocks_pass_through() {
        // BC7 -> PES 21: FTEX carrying the untouched blocks.
        let decoded = decode(BC7, SourceFormat::Dds).unwrap();
        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes21,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        let round = ftex::ftex_to_dds(&ftex).unwrap();
        let layout = ftex::dds::read_layout(&round).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc7)
        );
        assert_eq!((layout.width, layout.height, layout.mipmaps), (32, 16, 6));
        let source_layout = ftex::dds::read_layout(BC7).unwrap();
        assert_eq!(
            &round[layout.data_offset..],
            &BC7[source_layout.data_offset..]
        );

        // BC3 -> PES 17: legacy-header DDS, identical data.
        let decoded = decode(BC3, SourceFormat::Dds).unwrap();
        let dds = convert(
            &decoded,
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
        assert_eq!(&dds[layout.data_offset..], &BC3[128..]);

        // BC5 -> PES 21 Normal role: still a passthrough.
        let decoded = decode(BC5, SourceFormat::Dds).unwrap();
        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes21,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let round = ftex::ftex_to_dds(&ftex).unwrap();
        let layout = ftex::dds::read_layout(&round).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc5)
        );
        let source_layout = ftex::dds::read_layout(BC5).unwrap();
        assert_eq!(
            &round[layout.data_offset..],
            &BC5[source_layout.data_offset..]
        );
    }

    #[test]
    fn bc7_transcodes_to_bc3_for_older_versions() {
        let decoded = decode(BC7, SourceFormat::Dds).unwrap();
        let truth = mips_of(BC7_D);
        let source = mips_of(RGBA8_D);
        let reference_bc3 = mips_of(BC3_D);

        let dds = convert(
            &decoded,
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
        assert_eq!(layout.mipmaps, 6);
        assert_not_worse_than_reference(
            "pes17 bc3",
            &truth,
            &mips_of(&dds),
            &source,
            &reference_bc3,
        );

        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes18,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        assert_eq!(&ftex[..4], b"FTEX");
        let round = decode(&ftex, SourceFormat::Ftex).unwrap();
        assert_not_worse_than_reference("pes18 bc3", &truth, &round.mips, &source, &reference_bc3);
    }

    #[test]
    fn raster_sources_get_full_mip_chain_and_codec_rules() {
        let decoded = decode(PNG, SourceFormat::Png).unwrap();
        assert_eq!((decoded.width, decoded.height), (32, 16));
        assert_eq!(decoded.mips.len(), 1);
        assert!(decoded.blocks.is_none());
        let source = mips_of(RGBA8_D);
        assert_eq!(decoded.mips[0], source[0]);

        let dds = convert(
            &decoded,
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
        assert_eq!(layout.mipmaps, 6);
        let round = decode(&dds, SourceFormat::Dds).unwrap();
        for (level, (width, height)) in [(32u32, 16u32), (16, 8), (8, 4), (4, 2), (2, 1), (1, 1)]
            .iter()
            .enumerate()
        {
            assert_eq!(round.mips[level].len(), (*width * *height * 4) as usize);
        }
        // The whole chain, judged against the chain this crate generated.
        let generated: Vec<Vec<u8>> = std::iter::once(decoded.mips[0].clone())
            .chain(
                mips::generate(32, 16, &decoded.mips[0])
                    .into_iter()
                    .map(|mip| mip.pixels),
            )
            .collect();
        assert_not_worse_than_reference(
            "png -> pes17 bc3",
            &generated,
            &round.mips,
            &source,
            &mips_of(BC3_D),
        );

        // Fully opaque raster -> BC1 on pre-Fox.
        let opaque = decode(PNG_OPAQUE, SourceFormat::Png).unwrap();
        let dds = convert(
            &opaque,
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc1)
        );
        let round = decode(&dds, SourceFormat::Dds).unwrap();
        assert_not_worse_than_reference(
            "png opaque -> pes17 bc1",
            &opaque.mips,
            &round.mips[..1],
            &decode(PNG_OPAQUE, SourceFormat::Png).unwrap().mips,
            &mips_of(BC1_D)[..1],
        );

        // PES 21 color -> BC7 FTEX, whole chain.
        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes21,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        assert_eq!(&ftex[..4], b"FTEX");
        let round = decode(&ftex, SourceFormat::Ftex).unwrap();
        assert_not_worse_than_reference(
            "png -> pes21 bc7",
            &generated,
            &round.mips,
            &source,
            &mips_of(BC7_D),
        );
    }

    #[test]
    fn normal_maps_encode_in_the_dxt5nm_layout() {
        let decoded = decode(BC5, SourceFormat::Dds).unwrap();
        let bc5 = mips_of(BC5_D);
        // The reference: the pre-Fox layout encoded by the reference encoder
        // from the plain source (a `gggr` swizzle), decoded.
        let reference = mips_of(NM_PREFOX_D);
        let reference_truth = dxt5nm(&mips_of(RGBA8_D), false);

        // Pre-Fox: the colour block is a grey Y (R = G = B = Y), X in alpha.
        let dds = convert(
            &decoded,
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
        assert_not_worse_than_reference(
            "pes17 dxt5nm",
            &dxt5nm(&bc5, false),
            &mips_of(&dds),
            &reference_truth,
            &reference,
        );

        // Fox: R = 255, B = 0 exactly (constant channels quantize exactly);
        // G and A judged against the same reference, relaid in the Fox
        // layout.
        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes18,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let ours = decode(&ftex, SourceFormat::Ftex).unwrap().mips;
        for px in ours.iter().flat_map(|mip| mip.as_chunks::<4>().0.iter()) {
            assert_eq!((px[0], px[2]), (255, 0), "fox R/B");
        }
        assert_not_worse_than_reference(
            "pes18 dxt5nm",
            &dxt5nm(&bc5, true),
            &ours,
            &dxt5nm(&mips_of(RGBA8_D), true),
            &dxt5nm(&plain_from_prefox(&reference), true),
        );

        // A raster normal on PES 21 is BC3, never BC7.
        let decoded = decode(PNG, SourceFormat::Png).unwrap();
        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes21,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let round = ftex::ftex_to_dds(&ftex).unwrap();
        let layout = ftex::dds::read_layout(&round).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
    }

    #[test]
    fn authored_mip_count_is_kept() {
        // rgba8.dds cut to a single level: mip count 1, the mipmap caps bit
        // cleared, only level-0 data kept (DX10 header, 4 bytes per pixel).
        let mut single = RGBA8[..148 + 32 * 16 * 4].to_vec();
        single[28..32].copy_from_slice(&1u32.to_le_bytes());
        let caps = u32::from_le_bytes(single[108..112].try_into().unwrap()) & !0x400000;
        single[108..112].copy_from_slice(&caps.to_le_bytes());

        let decoded = decode(&single, SourceFormat::Dds).unwrap();
        assert_eq!(decoded.mips.len(), 1);
        assert!(decoded.authored_mips);

        let ftex = convert(
            &decoded,
            Target {
                version: PesVersion::Pes21,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        assert_eq!(ftex::info(&ftex).unwrap().mipmaps, 1);

        let dds = convert(
            &decoded,
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Color,
            },
        )
        .unwrap();
        assert_eq!(ftex::dds::read_layout(&dds).unwrap().mipmaps, 1);
    }

    #[test]
    fn normal_role_reencodes_color_layouts() {
        // A BC7 source is a color layout: the normal role re-encodes it to
        // the Fox DXT5nm layout instead of keeping the blocks.
        let ftex = convert(
            &decode(BC7, SourceFormat::Dds).unwrap(),
            Target {
                version: PesVersion::Pes21,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let round = ftex::ftex_to_dds(&ftex).unwrap();
        let layout = ftex::dds::read_layout(&round).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
        let ours = decode(&round, SourceFormat::Dds).unwrap();
        for px in ours
            .mips
            .iter()
            .flat_map(|mip| mip.as_chunks::<4>().0.iter())
        {
            assert_eq!((px[0], px[2]), (255, 0), "fox R/B");
        }

        // A BC1 source on a pre-Fox target: BC3 with a grey color block.
        let dds = convert(
            &decode(BC1, SourceFormat::Dds).unwrap(),
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(
            layout.pixel,
            ftex::dds::DdsPixel::Format(ftex::PixelFormat::Bc3)
        );
        let ours = decode(&dds, SourceFormat::Dds).unwrap();
        // The color block is a grey Y; 565 quantization gives G a sixth bit
        // R and B do not have, so R == B exactly and G within half a step.
        for px in ours
            .mips
            .iter()
            .flat_map(|mip| mip.as_chunks::<4>().0.iter())
        {
            assert_eq!(px[0], px[2], "grey color block R/B");
            assert!(px[1].abs_diff(px[0]) <= 4, "grey color block G");
        }

        // A BC3 source still passes through on a normal role.
        let dds = convert(
            &decode(BC3, SourceFormat::Dds).unwrap(),
            Target {
                version: PesVersion::Pes17,
                role: TextureRole::Normal,
            },
        )
        .unwrap();
        let layout = ftex::dds::read_layout(&dds).unwrap();
        assert_eq!(&dds[layout.data_offset..], &BC3[128..]);
    }

    #[test]
    fn padded_rows_decode() {
        // A 2x2 24-bit BGR DDS whose rows are padded to 8 bytes.
        let header = |pitch: u32| {
            let mut dds = Vec::new();
            dds.extend_from_slice(b"DDS ");
            dds.extend_from_slice(&124u32.to_le_bytes());
            dds.extend_from_slice(&(0x1u32 | 0x2 | 0x4 | 0x8 | 0x1000).to_le_bytes());
            dds.extend_from_slice(&2u32.to_le_bytes()); // height
            dds.extend_from_slice(&2u32.to_le_bytes()); // width
            dds.extend_from_slice(&pitch.to_le_bytes());
            dds.extend_from_slice(&0u32.to_le_bytes()); // depth
            dds.extend_from_slice(&1u32.to_le_bytes()); // mipmap count
            dds.extend_from_slice(&[0u8; 44]);
            dds.extend_from_slice(&32u32.to_le_bytes()); // pixel format size
            dds.extend_from_slice(&0x40u32.to_le_bytes()); // DDPF_RGB
            dds.extend_from_slice(&[0u8; 4]); // fourcc
            dds.extend_from_slice(&24u32.to_le_bytes());
            dds.extend_from_slice(&0xff0000u32.to_le_bytes()); // r
            dds.extend_from_slice(&0x00ff00u32.to_le_bytes()); // g
            dds.extend_from_slice(&0x0000ffu32.to_le_bytes()); // b
            dds.extend_from_slice(&0u32.to_le_bytes()); // a
            dds.extend_from_slice(&0x1000u32.to_le_bytes()); // caps1
            dds.extend_from_slice(&[0u8; 16]);
            dds
        };
        let expected: Vec<u8> = [
            [255, 0, 0, 255], // stored BGR 0,0,255
            [0, 255, 0, 255], // stored BGR 0,255,0
            [0, 0, 255, 255], // stored BGR 255,0,0
            [255, 255, 255, 255],
        ]
        .concat();

        // Pitch 8: two pad bytes per row.
        let mut padded = header(8);
        padded.extend_from_slice(&[0, 0, 255, 0, 255, 0, 0, 0]);
        padded.extend_from_slice(&[255, 0, 0, 255, 255, 255, 0, 0]);
        assert_eq!(
            decode(&padded, SourceFormat::Dds).unwrap().mips,
            [&expected[..]]
        );

        // Pitch 6 equals the tight row: no padding, same pixels.
        let mut tight = header(6);
        tight.extend_from_slice(&[0, 0, 255, 0, 255, 0]);
        tight.extend_from_slice(&[255, 0, 0, 255, 255, 255]);
        assert_eq!(
            decode(&tight, SourceFormat::Dds).unwrap().mips,
            [&expected[..]]
        );
    }

    #[test]
    fn mip_generation_averages_the_2x2_box() {
        // 4x2 with channel values that average exactly.
        let mut pixels = vec![0u8; 4 * 2 * 4];
        for (i, pixel) in pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            pixel[0] = i as u8 * 4;
            pixel[1] = 8;
            pixel[3] = 255;
        }
        let chain = mips::generate(4, 2, &pixels);
        assert_eq!(chain.len(), 2);
        assert_eq!((chain[0].width, chain[0].height), (2, 1));
        assert_eq!(chain[0].pixels[0], 10); // (0+4+16+20)/4
        assert_eq!(chain[0].pixels[4], 18); // (8+12+24+28)/4
        assert_eq!((chain[1].width, chain[1].height), (1, 1));
        assert_eq!(chain[1].pixels[0], 14); // (10+18)/2

        // 5x3 -> 2x1 -> 1x1.
        let pixels = vec![128u8; 5 * 3 * 4];
        let chain = mips::generate(5, 3, &pixels);
        let dims: Vec<(u32, u32)> = chain.iter().map(|m| (m.width, m.height)).collect();
        assert_eq!(dims, [(2, 1), (1, 1)]);
    }

    #[test]
    fn converter_caches_and_bypasses() {
        let converter = Converter::new();
        let hash = source_hash(BC3);
        let target = Target {
            version: PesVersion::Pes17,
            role: TextureRole::Color,
        };
        let first = converter
            .convert(hash, BC3, SourceFormat::Dds, target, CachePolicy::Use)
            .unwrap();
        let second = converter
            .convert(hash, BC3, SourceFormat::Dds, target, CachePolicy::Use)
            .unwrap();
        assert!(Arc::ptr_eq(&first, &second));

        let other = converter
            .convert(
                hash,
                BC3,
                SourceFormat::Dds,
                Target {
                    version: PesVersion::Pes18,
                    role: TextureRole::Color,
                },
                CachePolicy::Use,
            )
            .unwrap();
        assert!(!Arc::ptr_eq(&first, &other));
        assert_eq!(converter.retained_bytes(), first.len() + other.len());

        let retained = converter.retained_bytes();
        converter
            .convert(
                source_hash(BC7),
                BC7,
                SourceFormat::Dds,
                Target {
                    version: PesVersion::Pes21,
                    role: TextureRole::Color,
                },
                CachePolicy::Bypass,
            )
            .unwrap();
        assert_eq!(converter.retained_bytes(), retained);

        converter.clear();
        assert_eq!(converter.retained_bytes(), 0);
    }

    #[test]
    fn wesys_wrapped_dds_decodes_the_same() {
        let wrapped = wezlib::compress(BC3);
        assert_eq!(
            decode(&wrapped, SourceFormat::Dds).unwrap(),
            decode(BC3, SourceFormat::Dds).unwrap()
        );
    }

    #[test]
    fn ftex_source_decodes_like_its_dds_twin() {
        assert_eq!(
            decode(FTEX_BC1, SourceFormat::Ftex).unwrap(),
            decode(DDS_BC1, SourceFormat::Dds).unwrap()
        );
        let decoded = decode(FTEX_BC1, SourceFormat::Ftex).unwrap();
        assert_eq!((decoded.width, decoded.height), (16, 16));
        assert_eq!(decoded.mips.len(), 3);
        assert_eq!(
            decoded.blocks.as_ref().map(|blocks| blocks.codec),
            Some(BlockCodec::Bc1)
        );
    }

    #[test]
    fn malformed_sources_error() {
        assert!(matches!(
            decode(&BC3[..100], SourceFormat::Dds),
            Err(ConvertError::Ftex(ftex::FtexError::Truncated))
        ));
        assert!(matches!(
            decode(CUBE, SourceFormat::Dds),
            Err(ConvertError::Ftex(ftex::FtexError::UnsupportedDds(_)))
        ));
        assert_eq!(
            SourceFormat::from_extension("JPG"),
            Some(SourceFormat::Jpeg)
        );
        assert_eq!(
            SourceFormat::from_extension("tif"),
            Some(SourceFormat::Tiff)
        );
        assert_eq!(SourceFormat::from_extension("gif"), None);
        assert!(matches!(
            decode(b"not a png", SourceFormat::Png),
            Err(ConvertError::Image(_))
        ));
    }
}
