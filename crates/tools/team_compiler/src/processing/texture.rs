//! Texture conversion for the Fox engine, which reads FTEX (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 5): every accepted image format through
//! `dds_convert`, which picks the codec the target version reads; a model folder's textures as
//! one unit (step 6) and the export's Common textures as another ("Resolved decisions",
//! "Common textures are one task of their export"); a portrait as the DDS every engine reads
//! (`player_folders.md` "Portraits"). A texture's signature and size, a portrait's included,
//! are checked by the deep pass before planning; what conversion itself finds
//! (`texture_codec_unsupported`, `messages.md` "Textures") is a finding on the file, and what
//! it drops is the task's business.

use std::collections::BTreeMap;

use aesthetics_export::FileDescriptor;
use dds_convert::{
    BlockCodec, ConvertError, SourceFormat, Target, TextureRole, decode, encode_dds, source_hash,
};
use studio_core::Disposition;

use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::kit_variants::{KitToken, kit_token, variant_stem};
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

/// The failure of converting the file `name`: `texture_codec_unsupported` for what
/// `dds_convert` refuses to handle, the ordinary failure naming the file for anything else.
pub(super) fn conversion_failure(name: &str, error: ConvertError) -> TextureError {
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
/// returned for the writer to skip their tasks. The stems kept then have their kit variant sets
/// completed against `kits` (`complete_kit_variants`).
pub(super) fn folder_textures(
    folder: &ModelFolder,
    kits: &[u8],
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
    let mut textures: Vec<(String, Vec<u8>)> = copies
        .into_values()
        .filter_map(|stem_copies| {
            // The stem's one copy: the highest package's that is kept, the first of its
            // sources'; the copies kept agree, so which of them is written changes no byte.
            let kept = ModelPackage::ALL
                .into_iter()
                .filter(|package| !dropped.contains(package))
                .find(|package| stem_copies.iter().any(|copy| copy.package == *package))?;
            let copy = stem_copies.into_iter().find(|copy| copy.package == kept)?;
            Some((copy.stem, copy.bytes))
        })
        .collect();
    complete_kit_variants(&mut textures, kits, findings);
    let entries = textures
        .into_iter()
        .map(|(stem, bytes)| (folder.textures.texture(team_id, &stem), bytes))
        .collect();
    Ok((entries, dropped))
}

/// Completes the kit variant sets among `textures`, each a stem as spelled with its converted
/// bytes, against `kits`, the export's kit numbers (`pipeline.md` "4. Per-export non-model
/// steps", Kit-dependent assets): the variants of one reference, compared case-folded, are a
/// set, and for each of `kits` a set has no variant of, in ascending order, the lowest
/// variant's bytes are added under that number's name and `kit_variant_missing` is noted in
/// `findings`, so the game never shows a missing texture for a kit the team has. A variant of a
/// number `kits` does not hold stays as it is.
fn complete_kit_variants(
    textures: &mut Vec<(String, Vec<u8>)>,
    kits: &[u8],
    findings: &mut Vec<Finding>,
) {
    // Every variant as (folded reference, kit number, position in `textures`, reference),
    // sorted so each set is a run with its lowest variant first.
    let mut variants: Vec<(String, u8, usize, String)> = textures
        .iter()
        .enumerate()
        .filter_map(|(index, (stem, _))| match kit_token(stem)? {
            (KitToken::Variant(kit), reference) => {
                Some((vtree::fold_name(&reference), kit, index, reference))
            }
            (KitToken::Reference, _) => None,
        })
        .collect();
    variants.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    for set in variants.chunk_by(|a, b| a.0 == b.0) {
        let (_, _, lowest, reference) = &set[0];
        let copied = textures[*lowest].0.clone();
        for &kit in kits {
            if set.iter().any(|(_, held, ..)| *held == kit) {
                continue;
            }
            let stem = variant_stem(&copied, kit).expect("a variant's stem holds its kit token");
            // Each entry owns its bytes: the gap gets a copy of the lowest variant's.
            let bytes = textures[*lowest].1.clone();
            textures.push((stem, bytes));
            findings.push((
                Code::KitVariantMissing,
                Disposition::Keep,
                vec![
                    ("texture", reference.clone()),
                    ("kit", kit.to_string()),
                    ("copied", copied.clone()),
                ],
            ));
        }
    }
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
/// not a `.common` link uses it. The deep pass has already dropped a texture its checks find
/// wrong. A texture conversion reports a finding on (`texture_codec_unsupported`) is left out
/// alone, the finding noted in `findings` with `DropFile`, and the rest emitted; any other
/// failure fails the task, and with it every Common texture. The players linking a Common
/// model still commit on their own. The textures emitted have their kit variant sets completed
/// against `kits` (`complete_kit_variants`).
pub(super) fn common_textures(
    textures: &[FileDescriptor],
    kits: &[u8],
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Entry>, TaskFailure> {
    let mut converted = Vec::with_capacity(textures.len());
    for file in textures {
        let name = file.path.name();
        let format = texture_format(name)
            .expect("planning lists the `Common/` textures by an extension `dds_convert` accepts");
        match convert(ctx, format, name, &take(files, file)) {
            Ok(bytes) => converted.push((file_stem(name).to_owned(), bytes)),
            Err(TextureError::Finding(code, file)) => {
                findings.push((code, Disposition::DropFile, vec![("file", file)]));
            }
            Err(TextureError::Other(error)) => return Err(TaskFailure::from(error)),
        }
    }
    complete_kit_variants(&mut converted, kits, findings);
    Ok(converted
        .into_iter()
        .map(|(stem, bytes)| (paths::common_texture(team_id, &stem), bytes))
        .collect())
}

/// The texture file `name`, in `format`, converted for the run's version through its
/// converter: an FTEX on every Fox target, in the codec the version reads, with the mip chain
/// a raster source lacks generated, in the role the file's stem gives it. Its signature and
/// size are the deep pass's checks, done before planning; a texture that reaches this point
/// either passed them or is kept by `pass_through`, so it is converted as it is.
pub(super) fn convert(
    ctx: &CompileContext,
    format: SourceFormat,
    name: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, TextureError> {
    let target = Target {
        version: ctx.version,
        role: texture_role(file_stem(name)),
    };
    let converted = ctx
        .converter
        .convert(source_hash(bytes), bytes, format, target, ctx.cache)
        .map_err(|error| conversion_failure(name, error))?;
    // The converter hands out the cache's own buffer, which it may hand out again for the same
    // source; the CPK entry owns its bytes, so the one copy of the texture is here.
    Ok(converted.to_vec())
}

/// The portrait file `name`, in `format`, holding `bytes`, as the DDS every engine reads
/// (`player_folders.md` "Portraits"): a DDS source as it is, any other accepted format
/// encoded to BC3 at its own size with the full mip chain. Its signature and size are the deep
/// pass's checks; a portrait that reaches this point passed them or is kept by
/// `pass_through`, so it is packed whatever its size.
pub(super) fn portrait(
    format: SourceFormat,
    name: &str,
    bytes: Vec<u8>,
) -> Result<Vec<u8>, TextureError> {
    // Decoded even when the DDS goes out as it is: the deep pass reads only the header, and
    // this decode is what fails the task of a portrait whose header or data is broken.
    let decoded = decode(&bytes, format).map_err(|error| conversion_failure(name, error))?;
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
            templates: crate::templates::Templates::embedded(),
            installed: crate::bins::installed::InstalledPaths::Unknown,
            target: crate::processing::EntryTarget::GamePaths,
        }
    }

    fn tracer_kit() -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Midcup Tracer/Kits/g1/kit.dds"),
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
    fn a_texture_of_an_odd_size_converts_as_it_is() {
        // The deep pass reports its size before planning; one `pass_through` keeps is
        // converted: 300x300 with its generated chain down to 1x1.
        let converted = convert(
            &context(PesVersion::Pes21),
            SourceFormat::Png,
            "odd.png",
            &texture_fixture("odd.png"),
        )
        .unwrap();
        let info = ftex::info(&converted).unwrap();
        assert_eq!((info.width, info.height, info.mipmaps), (300, 300, 9));
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
                .join("tests/fixtures/tracer/studio/egg Midcup Tracer/Players/05 - The Chad Stormworks Player/portrait.dds"),
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
        // A portrait's signature and size are the deep pass's checks: one `pass_through`
        // keeps is packed whatever its size, a 300x300 PNG as a 300x300 BC3 DDS.
        let odd = texture_fixture("odd.png");
        let dds = portrait(SourceFormat::Png, "player_05.png", odd.clone()).unwrap();
        let decoded = decode(&dds, SourceFormat::Dds).unwrap();
        assert_eq!((decoded.width, decoded.height), (300, 300));
        // Every source is still decoded, a DDS included: a BC6H DDS is the codec finding, and
        // PNG bytes under a `.dds` name fail the task.
        assert_eq!(
            finding(portrait(
                SourceFormat::Dds,
                "portrait.dds",
                texture_fixture("bc6h.dds")
            )),
            (Code::TextureCodecUnsupported, "portrait.dds".to_owned())
        );
        let Err(TextureError::Other(error)) = portrait(SourceFormat::Dds, "portrait.dds", odd)
        else {
            panic!("PNG bytes under a `.dds` name are not a DDS");
        };
        assert_eq!(format!("{error}"), "portrait.dds: cannot convert");
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
