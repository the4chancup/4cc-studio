//! Texture conversion for the Fox engine, which reads FTEX (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 5): every accepted image format through
//! `dds_convert`, which picks the codec the target version reads; a model folder's textures as
//! one unit (step 6) and the export's Common textures as another ("Resolved decisions",
//! "Common textures are one task of their export"); a portrait as the DDS every engine reads
//! (`player_folders.md` "Portraits"). The four ways a texture fails are findings on the file
//! (`messages.md` "Textures"); what each drops is the task's business.

use std::collections::BTreeMap;

use aesthetics_export::FileDescriptor;
use anyhow::Context;
use dds_convert::{
    BlockCodec, ConvertError, SourceFormat, Target, TextureRole, decode, encode_dds, source_hash,
};
use studio_core::Disposition;

use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::messages::Code;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem, texture_format};

/// Why a texture could not be converted.
#[derive(Debug)]
pub(super) enum TextureError {
    /// One of the texture findings (`messages.md` "Textures"), on the file named: what it
    /// drops depends on where the texture is, so the task decides.
    Finding(Code, String),
    /// Any other failure, which is the ordinary `folder_pack_failed`.
    Other(anyhow::Error),
}

impl From<TextureError> for TaskFailure {
    fn from(error: TextureError) -> TaskFailure {
        match error {
            TextureError::Finding(code, file) => TaskFailure {
                code,
                context: vec![("file", file)],
            },
            TextureError::Other(error) => TaskFailure::from(error),
        }
    }
}

/// The accepted formats that open with a fixed signature, and that signature; WebP's `RIFF`
/// is checked with its `WEBP` tag below, and TGA has none.
const SIGNATURES: [(&[u8], SourceFormat); 7] = [
    (b"DDS ", SourceFormat::Dds),
    (b"FTEX", SourceFormat::Ftex),
    (b"\x89PNG", SourceFormat::Png),
    (&[0xff, 0xd8, 0xff], SourceFormat::Jpeg),
    (b"BM", SourceFormat::Bmp),
    (b"II*\0", SourceFormat::Tiff),
    (b"MM\0*", SourceFormat::Tiff),
];

/// The accepted format whose signature `bytes` open with, if any.
fn signature_format(bytes: &[u8]) -> Option<SourceFormat> {
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(SourceFormat::WebP);
    }
    SIGNATURES
        .iter()
        .find(|(signature, _)| bytes.starts_with(signature))
        .map(|(_, format)| *format)
}

/// `texture_type_mismatch` when the file `name`'s `bytes` open with the signature of another
/// accepted format than `format`, the one its extension names: a file renamed, not resaved.
fn check_signature(format: SourceFormat, name: &str, bytes: &[u8]) -> Result<(), TextureError> {
    match signature_format(bytes) {
        Some(sniffed) if sniffed != format => Err(TextureError::Finding(
            Code::TextureTypeMismatch,
            name.to_owned(),
        )),
        Some(_) | None => Ok(()),
    }
}

/// `texture_too_small` when a side of the file `name`'s texture is under 4 pixels, one block;
/// `texture_not_pow2` when `needs_pow2` and a side is not a power of two (a portrait always; a
/// Fox texture when it is mipmapped, so a single-level texture of any size passes).
fn check_dimensions(
    name: &str,
    width: u32,
    height: u32,
    needs_pow2: bool,
) -> Result<(), TextureError> {
    if width < 4 || height < 4 {
        return Err(TextureError::Finding(
            Code::TextureTooSmall,
            name.to_owned(),
        ));
    }
    if needs_pow2 && !(width.is_power_of_two() && height.is_power_of_two()) {
        return Err(TextureError::Finding(Code::TextureNotPow2, name.to_owned()));
    }
    Ok(())
}

/// The failure of converting the file `name`: `texture_codec_unsupported` for what
/// `dds_convert` refuses to handle, the ordinary failure naming the file for anything else.
fn conversion_failure(name: &str, error: ConvertError) -> TextureError {
    match error {
        ConvertError::Unsupported(_) => {
            TextureError::Finding(Code::TextureCodecUnsupported, name.to_owned())
        }
        ConvertError::Ftex(_)
        | ConvertError::Wesys(_)
        | ConvertError::Image(_)
        | ConvertError::InvalidDecoded(_)
        | ConvertError::Truncated => TextureError::Other(
            anyhow::Error::from(error).context(format!("{name}: cannot convert")),
        ),
    }
}

/// One source's copy of a texture: the package the source feeds, the stem as the source
/// spells it, and the FTEX bytes.
struct TextureCopy {
    package: ModelPackage,
    stem: String,
    bytes: Vec<u8>,
}

/// The textures of `folder`, its own and its combined folders', converted from their bytes in
/// `files` into the folder's texture home for team `team_id`, by stem compared case-folded:
/// one entry per stem, however many of the folder's models use it and however many ids its
/// packages are emitted under. A stem several sources hold is one entry when their bytes agree.
/// When they differ within one package the task fails with `merged_texture_conflict`: the one
/// model those sources build has no winner. When they differ across packages the higher
/// package in canonical order (face > boots > gloves) wins, `shared_texture_conflict` is noted
/// in `findings` per stem and lower package, and the lower package is dropped: its textures
/// task's entries leave out every texture only its sources hold, and the dropped packages are
/// returned for the writer to skip their tasks.
pub(super) fn folder_textures(
    folder: &ModelFolder,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<(Vec<Entry>, Vec<ModelPackage>), TaskFailure> {
    // Each stem's copies in source order: the player's own folder's, then each combined
    // folder's.
    let mut copies: BTreeMap<String, Vec<TextureCopy>> = BTreeMap::new();
    for (package, _, source_files) in folder.roles() {
        for (file, role) in source_files {
            let PlayerFile::Texture(stem, format) = role else {
                continue;
            };
            let bytes = convert(ctx, format, file.path.name(), &take(files, file))?;
            copies
                .entry(vtree::fold_name(&stem))
                .or_default()
                .push(TextureCopy {
                    package,
                    stem,
                    bytes,
                });
        }
    }
    let mut dropped: Vec<ModelPackage> = Vec::new();
    for stem_copies in copies.values() {
        resolve_stem(stem_copies, &mut dropped, findings)?;
    }
    let entries = copies
        .into_values()
        .filter_map(|stem_copies| {
            // The stem's one copy: the highest package's that is kept, the first of its
            // sources'; the copies kept agree, so which of them is written changes no byte.
            let kept = ModelPackage::ALL
                .into_iter()
                .filter(|package| !dropped.contains(package))
                .find(|package| stem_copies.iter().any(|copy| copy.package == *package))?;
            let copy = stem_copies.into_iter().find(|copy| copy.package == kept)?;
            Some((folder.textures.texture(team_id, &copy.stem), copy.bytes))
        })
        .collect();
    Ok((entries, dropped))
}

/// Decides one stem held by `copies`, several sources' in source order: a disagreement within
/// one package fails the task; across packages, each package lower than the highest one holding
/// the stem whose bytes differ from its is noted and added to `dropped`.
fn resolve_stem(
    copies: &[TextureCopy],
    dropped: &mut Vec<ModelPackage>,
    findings: &mut Vec<Finding>,
) -> Result<(), TaskFailure> {
    for package in ModelPackage::ALL {
        let mut of_package = copies.iter().filter(|copy| copy.package == package);
        if let Some(first) = of_package.next()
            && of_package.any(|copy| copy.bytes != first.bytes)
        {
            return Err(TaskFailure {
                code: Code::MergedTextureConflict,
                context: vec![("texture", first.stem.clone())],
            });
        }
    }
    let Some(winner) = ModelPackage::ALL
        .into_iter()
        .find_map(|package| copies.iter().find(|copy| copy.package == package))
    else {
        return Ok(());
    };
    for copy in copies {
        if copy.package == winner.package || copy.bytes == winner.bytes {
            continue;
        }
        findings.push((
            Code::SharedTextureConflict,
            Disposition::DropFolder,
            vec![
                ("texture", winner.stem.clone()),
                ("dropped", copy.package.name().to_owned()),
            ],
        ));
        if !dropped.contains(&copy.package) {
            dropped.push(copy.package);
        }
    }
    Ok(())
}

/// The export's Common `textures`, converted from their bytes in `files` into the team's Common
/// output for team `team_id`, each under its stem as spelled: one entry per texture, whether or
/// not a `.common` link uses it. A texture with a finding is left out alone, the finding noted
/// in `findings` with `DropFile`, and the rest emitted; any other failure fails the task, and
/// with it every Common texture. The players linking a Common model still commit on their own.
pub(super) fn common_textures(
    textures: &[FileDescriptor],
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Entry>, TaskFailure> {
    let mut entries = Vec::with_capacity(textures.len());
    for file in textures {
        let name = file.path.name();
        let format = texture_format(name)
            .expect("planning lists the `Common/` textures by an extension `dds_convert` accepts");
        match convert(ctx, format, name, &take(files, file)) {
            Ok(bytes) => entries.push((paths::common_texture(team_id, file_stem(name)), bytes)),
            Err(TextureError::Finding(code, file)) => {
                findings.push((code, Disposition::DropFile, vec![("file", file)]));
            }
            Err(TextureError::Other(error)) => return Err(TaskFailure::from(error)),
        }
    }
    Ok(entries)
}

/// The texture file `name`, in `format`, converted for the run's version through its
/// converter: an FTEX on every Fox target, in the codec the version reads, with the mip chain
/// a raster source lacks generated, in the role the file's stem gives it. The findings are
/// checked around the conversion: the signature before it, the size on the FTEX written, which
/// is the decoded size (its header is 64 bytes, so this costs no second decode).
pub(super) fn convert(
    ctx: &CompileContext,
    format: SourceFormat,
    name: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, TextureError> {
    check_signature(format, name, bytes)?;
    let target = Target {
        version: ctx.version,
        role: texture_role(file_stem(name)),
    };
    let converted = ctx
        .converter
        .convert(source_hash(bytes), bytes, format, target, ctx.cache)
        .map_err(|error| conversion_failure(name, error))?;
    let info = ftex::info(&converted)
        .with_context(|| format!("{name}: the converted FTEX header"))
        .map_err(TextureError::Other)?;
    check_dimensions(
        name,
        u32::from(info.width),
        u32::from(info.height),
        info.mipmaps > 1,
    )?;
    // The converter hands out the cache's own buffer, which it may hand out again for the same
    // source; the CPK entry owns its bytes, so the one copy of the texture is here.
    Ok(converted.to_vec())
}

/// The portrait file `name`, in `format`, holding `bytes`, as the DDS every engine reads
/// (`player_folders.md` "Portraits"): a DDS source as it is, after the findings are checked on
/// its decode; any other accepted format decoded and encoded to BC3 at its own size with the
/// full mip chain. A portrait's sides must be powers of two whatever its mip count.
pub(super) fn portrait(
    format: SourceFormat,
    name: &str,
    bytes: Vec<u8>,
) -> Result<Vec<u8>, TextureError> {
    check_signature(format, name, &bytes)?;
    let decoded = decode(&bytes, format).map_err(|error| conversion_failure(name, error))?;
    check_dimensions(name, decoded.width, decoded.height, true)?;
    match format {
        SourceFormat::Dds => Ok(bytes),
        SourceFormat::Ftex
        | SourceFormat::Png
        | SourceFormat::Jpeg
        | SourceFormat::Bmp
        | SourceFormat::WebP
        | SourceFormat::Tga
        | SourceFormat::Tiff => {
            encode_dds(&decoded, BlockCodec::Bc3).map_err(|error| conversion_failure(name, error))
        }
    }
}

/// The role of a texture by its `stem`: a normal map when the stem ends in `_nrm` in any
/// case, the suffix of the game's own normal maps (`skin_nrm`, `oral_nrm`); color otherwise.
pub(super) fn texture_role(stem: &str) -> TextureRole {
    let bytes = stem.as_bytes();
    let tail = bytes.len().checked_sub(4).map(|at| &bytes[at..]);
    if tail.is_some_and(|tail| tail.eq_ignore_ascii_case(b"_nrm")) {
        TextureRole::Normal
    } else {
        TextureRole::Color
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use dds_convert::{CachePolicy, Converter, decode};
    use pes_version::PesVersion;

    use super::*;

    fn context(version: PesVersion) -> CompileContext {
        CompileContext {
            version,
            converter: Converter::new(),
            cache: CachePolicy::Bypass,
        }
    }

    fn tracer_kit() -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Tracer/Kits/g1/kit.dds"),
        )
        .unwrap()
    }

    #[test]
    fn a_texture_is_converted_for_the_run_s_version_or_fails_naming_the_file() {
        let dds = tracer_kit();
        let decoded = decode(&dds, SourceFormat::Dds).unwrap();
        for version in [PesVersion::Pes18, PesVersion::Pes21] {
            let converted = convert(&context(version), SourceFormat::Dds, "kit.dds", &dds).unwrap();
            let target = Target {
                version,
                role: TextureRole::Color,
            };
            assert_eq!(
                converted,
                dds_convert::convert(&decoded, target).unwrap(),
                "{version}"
            );
        }
        let error = convert(
            &context(PesVersion::Pes21),
            SourceFormat::Dds,
            "kit.dds",
            b"not a DDS",
        )
        .unwrap_err();
        let TextureError::Other(error) = error else {
            panic!("a DDS without its header is no finding");
        };
        assert_eq!(format!("{error}"), "kit.dds: cannot convert");
    }

    /// The bytes of `tests/fixtures/textures/<name>`.
    fn texture_fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/textures")
                .join(name),
        )
        .unwrap()
    }

    /// The (code, file) of a texture finding; panics on any other outcome.
    fn finding(result: Result<Vec<u8>, TextureError>) -> (Code, String) {
        match result {
            Ok(_) => panic!("converted"),
            Err(TextureError::Finding(code, file)) => (code, file),
            Err(TextureError::Other(error)) => panic!("{error:#}"),
        }
    }

    #[test]
    fn each_accepted_format_s_signature_is_recognized_and_tga_has_none() {
        // A real file of each format opens with its signature (the kit and portrait fixtures
        // are real encoder output); a signature pasted onto nothing is still that format's.
        assert_eq!(
            signature_format(&tracer_kit()),
            Some(SourceFormat::Dds),
            "dds"
        );
        assert_eq!(
            signature_format(&texture_fixture("kit.png")),
            Some(SourceFormat::Png),
            "png"
        );
        assert_eq!(
            signature_format(&texture_fixture("portrait.webp")),
            Some(SourceFormat::WebP),
            "webp"
        );
        assert_eq!(
            signature_format(&texture_fixture("kit_back.tga")),
            None,
            "tga has no signature"
        );
        for (bytes, format) in [
            (&b"FTEX\x00\x00\x00\x00"[..], SourceFormat::Ftex),
            (&[0xff, 0xd8, 0xff, 0xe0, 0, 0x10], SourceFormat::Jpeg),
            (&b"BM\x36\x00\x00\x00"[..], SourceFormat::Bmp),
            (&b"II*\0\x08\x00\x00\x00"[..], SourceFormat::Tiff),
            (&b"MM\0*\x00\x00\x00\x08"[..], SourceFormat::Tiff),
            (&b"RIFF\x00\x00\x00\x00WEBPVP8 "[..], SourceFormat::WebP),
        ] {
            assert_eq!(signature_format(bytes), Some(format), "{format:?}");
        }
        // A RIFF that is not WebP, and bytes opening with none of them, are no format.
        assert_eq!(signature_format(b"RIFF\x00\x00\x00\x00WAVEfmt "), None);
        assert_eq!(signature_format(b"RIFF"), None, "too short for the tag");
        assert_eq!(signature_format(b"not a texture"), None);
        assert_eq!(signature_format(b""), None);
    }

    #[test]
    fn a_file_opening_with_another_format_s_signature_is_a_type_mismatch() {
        // PNG bytes under a `.dds` name: renamed, not resaved.
        let png = texture_fixture("portrait.png");
        assert_eq!(
            finding(convert(
                &context(PesVersion::Pes21),
                SourceFormat::Dds,
                "skin.dds",
                &png
            )),
            (Code::TextureTypeMismatch, "skin.dds".to_owned())
        );
        // The same bytes under a `.tga` name: TGA has no signature, but PNG's is another's.
        assert_eq!(
            finding(convert(
                &context(PesVersion::Pes21),
                SourceFormat::Tga,
                "skin.tga",
                &png
            )),
            (Code::TextureTypeMismatch, "skin.tga".to_owned())
        );
        // A DDS with its header cut off opens with no signature: no mismatch, and the decode
        // fails as before.
        let cut = tracer_kit()[128..].to_vec();
        assert!(matches!(
            convert(
                &context(PesVersion::Pes21),
                SourceFormat::Dds,
                "kit.dds",
                &cut
            ),
            Err(TextureError::Other(_))
        ));
    }

    #[test]
    fn a_small_or_odd_texture_is_a_finding_and_a_single_level_odd_one_is_not() {
        let ctx = context(PesVersion::Pes21);
        assert_eq!(
            finding(convert(
                &ctx,
                SourceFormat::Png,
                "tiny.png",
                &texture_fixture("tiny.png")
            )),
            (Code::TextureTooSmall, "tiny.png".to_owned())
        );
        assert_eq!(
            finding(convert(
                &ctx,
                SourceFormat::Png,
                "odd.png",
                &texture_fixture("odd.png")
            )),
            (Code::TextureNotPow2, "odd.png".to_owned())
        );
        // The checks themselves: 4 is the smallest side, a side under it on either axis is
        // too small whatever the other, and the power-of-two rule applies only when asked.
        assert!(check_dimensions("t", 4, 4, true).is_ok());
        assert!(check_dimensions("t", 300, 300, false).is_ok());
        for (width, height) in [(3, 300), (300, 3), (3, 3)] {
            assert!(matches!(
                check_dimensions("t", width, height, false),
                Err(TextureError::Finding(Code::TextureTooSmall, _))
            ));
        }
        for (width, height) in [(300, 256), (256, 300)] {
            assert!(matches!(
                check_dimensions("t", width, height, true),
                Err(TextureError::Finding(Code::TextureNotPow2, _))
            ));
        }
    }

    #[test]
    fn a_codec_dds_convert_does_not_decode_is_codec_unsupported() {
        assert_eq!(
            finding(convert(
                &context(PesVersion::Pes21),
                SourceFormat::Dds,
                "skin.dds",
                &texture_fixture("bc6h.dds")
            )),
            (Code::TextureCodecUnsupported, "skin.dds".to_owned())
        );
    }

    #[test]
    fn a_single_level_texture_with_an_odd_side_converts_on_fox() {
        // 12x12 BC3 with one mip level: the power-of-two rule is for mipmapped textures, so
        // this one goes through at its size with its one level.
        let converted = convert(
            &context(PesVersion::Pes21),
            SourceFormat::Dds,
            "skin.dds",
            &texture_fixture("single_level.dds"),
        )
        .unwrap();
        let info = ftex::info(&converted).unwrap();
        assert_eq!((info.width, info.height, info.mipmaps), (12, 12, 1));
    }

    #[test]
    fn a_dds_portrait_passes_through_and_any_other_format_is_a_bc3_dds() {
        let tracer = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Tracer/Players/05 - The Chad Stormworks Player/portrait.dds"),
        )
        .unwrap();
        assert_eq!(
            portrait(SourceFormat::Dds, "portrait.dds", tracer.clone()).unwrap(),
            tracer
        );
        for (name, format) in [
            ("portrait.png", SourceFormat::Png),
            ("portrait.webp", SourceFormat::WebP),
        ] {
            let dds = portrait(format, name, texture_fixture(name)).unwrap();
            let decoded = decode(&dds, SourceFormat::Dds).unwrap();
            assert_eq!(
                decoded.blocks.as_ref().map(|blocks| blocks.codec),
                Some(BlockCodec::Bc3),
                "{name}"
            );
            assert_eq!(
                (decoded.width, decoded.height, decoded.mips.len()),
                (128, 128, 8),
                "{name}"
            );
        }
        // The findings apply to a portrait as to any texture, the power-of-two rule whatever
        // its mip count: a 64x128 DDS passes, a 300x300 PNG does not, nor a 3x3 one, nor a
        // BC6H DDS, nor PNG bytes under a `.dds` name.
        let odd = texture_fixture("odd.png");
        assert_eq!(
            finding(portrait(SourceFormat::Png, "player_05.png", odd.clone())),
            (Code::TextureNotPow2, "player_05.png".to_owned())
        );
        assert_eq!(
            finding(portrait(
                SourceFormat::Png,
                "portrait.png",
                texture_fixture("tiny.png")
            )),
            (Code::TextureTooSmall, "portrait.png".to_owned())
        );
        assert_eq!(
            finding(portrait(
                SourceFormat::Dds,
                "portrait.dds",
                texture_fixture("bc6h.dds")
            )),
            (Code::TextureCodecUnsupported, "portrait.dds".to_owned())
        );
        assert_eq!(
            finding(portrait(SourceFormat::Dds, "portrait.dds", odd)),
            (Code::TextureTypeMismatch, "portrait.dds".to_owned())
        );
    }

    #[test]
    fn a_stem_ending_in_nrm_in_any_case_is_a_normal_map() {
        for normal in ["skin_nrm", "SKIN_NRM", "oral_Nrm", "_nrm"] {
            assert_eq!(texture_role(normal), TextureRole::Normal, "{normal}");
        }
        for color in ["skin", "nrm", "skin_nrm2", "skin-nrm", "kit", ""] {
            assert_eq!(texture_role(color), TextureRole::Color, "{color}");
        }
    }
}
