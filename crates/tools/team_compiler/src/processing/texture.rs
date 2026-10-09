//! Texture conversion for the target's engine, FTEX on Fox, DDS on pre-Fox
//! (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps", step 5): every accepted
//! image format through
//! `dds_convert`, which picks the codec the target version reads; a model folder's textures as
//! one unit (step 6) and the export's Common textures as another ("Resolved decisions",
//! "Common textures are one task of their export"); a portrait as the DDS every engine reads
//! (`player_folders.md` "Portraits"). A texture's signature and size, a portrait's included,
//! are checked by the deep pass before planning; what conversion itself finds
//! (`texture_codec_unsupported`, `messages.md` "Textures") is a finding on the file, and what
//! it drops is the task's business.

use std::collections::BTreeMap;
use std::sync::Arc;

use aesthetics_export::{FileDescriptor, KitToken, kit_token, variant_stem};
use dds_convert::{
    BlockCodec, ConvertError, SourceFormat, Target, TextureRole, decode, encode_dds, probe,
    source_hash,
};
use ftex::FtexError;
use ftex::dds::{DdsPixel, read_layout};
use pes_version::Engine;
use pipeline::{MemoryBudget, Permit};
use studio_core::Disposition;

use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::messages::Code;
use crate::paths;
use crate::plan::roles::{ModelPackage, PlayerFile, file_stem, texture_format};
use crate::plan::{ENVIRONMENT_MAP_STEM, ModelFolder};

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

/// The failure of converting the file `name`. What `dds_convert` refuses as a codec and what
/// `ftex::dds::read_layout` refuses as a kind of DDS (a signed block format, a volume texture,
/// an array, a paletted DDS, an incomplete cube map) are one class, a file of a kind the
/// target cannot be given: the catalog's file-level `texture_codec_unsupported`. A header that
/// cannot be read, or pixel data cut short, is the ordinary failure naming the file, which
/// fails the task. The cube-map route's `ftex::dds_to_ftex` error is mapped here too, so its
/// refusals take the same finding.
pub(super) fn conversion_failure(name: &str, error: ConvertError) -> TextureError {
    match error {
        ConvertError::Unsupported(_) | ConvertError::Ftex(FtexError::UnsupportedDds(_)) => {
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
/// returned for the writer to skip their tasks. A folder taking the template environment map
/// (`ModelFolder::takes_template_environment_map`) gets it as `env.dds`, emitted as it is. The
/// stems kept then have their kit variant sets completed against `kits`
/// (`complete_kit_variants`).
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
    let environment_map = folder.takes_template_environment_map().then(|| {
        (
            ENVIRONMENT_MAP_STEM.to_owned(),
            ctx.templates.environment_map().to_vec(),
        )
    });
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
        .chain(environment_map)
        .collect();
    complete_kit_variants(&mut textures, kits, findings);
    let entries = textures
        .into_iter()
        .map(|(stem, bytes)| {
            let path = folder
                .textures
                .texture(ctx.version.engine(), team_id, &stem);
            (path, bytes)
        })
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
/// output for team `team_id` on the run's engine (`paths::common_texture`: FTEX on Fox, DDS on
/// pre-Fox), each under its stem as spelled: one entry per texture, whether or
/// not a `.common` link uses it. The deep pass has already dropped a texture its checks find
/// wrong. A texture conversion reports a finding on (`texture_codec_unsupported`) is left out
/// alone, the finding noted in `findings` with `DropFile`, and the rest emitted; any other
/// failure fails the task, and with it every Common texture. The players linking a Common
/// model still commit on their own. With `environment_map` (`TaskKind::CommonTextures`) the
/// template environment map is emitted as `env.dds` beside them, as it is. The textures emitted
/// have their kit variant sets completed against `kits` (`complete_kit_variants`).
pub(super) fn common_textures(
    textures: &[FileDescriptor],
    kits: &[u8],
    environment_map: bool,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Entry>, TaskFailure> {
    let mut converted = Vec::with_capacity(textures.len());
    for file in textures {
        let name = file.path.name();
        match common_texture(file, ctx, files) {
            Ok(bytes) => converted.push((file_stem(name).to_owned(), bytes)),
            Err(TextureError::Finding(code, file)) => {
                findings.push((code, Disposition::DropFile, vec![("file", file)]));
            }
            Err(TextureError::Other(error)) => return Err(TaskFailure::from(error)),
        }
    }
    if environment_map {
        converted.push((
            ENVIRONMENT_MAP_STEM.to_owned(),
            ctx.templates.environment_map().to_vec(),
        ));
    }
    complete_kit_variants(&mut converted, kits, findings);
    let engine = ctx.version.engine();
    Ok(converted
        .into_iter()
        .map(|(stem, bytes)| (paths::common_texture(engine, team_id, &stem), bytes))
        .collect())
}

/// One texture of a team's Common output, `file`, converted from its bytes in `files` as any
/// texture is (`convert`), in the format its extension names: a `Common/` texture, or the
/// referees' marker.
pub(super) fn common_texture(
    file: &FileDescriptor,
    ctx: &CompileContext,
    files: &mut TaskFiles,
) -> Result<Vec<u8>, TextureError> {
    let name = file.path.name();
    let format = texture_format(name).expect(
        "planning lists a Common texture, the marker included, by an extension `dds_convert` \
         accepts",
    );
    convert(ctx, format, name, &take(files, file))
}

/// The texture file `name`, in `format`, converted for the run's version through its
/// converter: an FTEX on every Fox target, a DDS on pre-Fox, in the codec the version reads,
/// with the mip chain a raster source lacks generated, in the role the file's stem gives it.
/// On pre-Fox a WESYS-wrapped DDS whose blocks the conversion keeps as they are is returned as
/// it is, still wrapped, which the game reads as it reads a plain DDS (`settings.md`, "DDS
/// compression cost"). A plain cube-map DDS is never decoded: it goes out as it is on pre-Fox
/// and as the FTEX cube map `ftex::dds_to_ftex` writes on Fox. Its signature and
/// size are the deep pass's checks, done before planning; a texture that reaches this point
/// either passed them or is kept by `pass_through`, so it is converted as it is.
pub(super) fn convert(
    ctx: &CompileContext,
    format: SourceFormat,
    name: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, TextureError> {
    // The converter decodes into one 2D image and refuses a cube map, which the game takes on
    // both engines: on pre-Fox the DDS as it is, on Fox the FTEX cube map the container
    // conversion writes from the DDS's own blocks. Nothing is decoded, so no decode is charged:
    // the output is about the source's size, and the writer charges a task's entries. The
    // texture type is the normal-map one, the type every Fox texture the converter writes
    // carries, here with the cube bit. A WESYS-wrapped DDS's header is compressed, so a
    // wrapped cube map is not recognized and fails in the converter.
    if format == SourceFormat::Dds && ftex::dds::is_cube_map(bytes) {
        return match ctx.version.engine() {
            Engine::PreFox => Ok(bytes.to_vec()),
            Engine::Fox => ftex::dds_to_ftex(bytes, ftex::ColorSpace::Normal)
                .map_err(|error| conversion_failure(name, ConvertError::Ftex(error))),
        };
    }
    let target = Target {
        version: ctx.version,
        role: texture_role(file_stem(name)),
    };
    // The decode happens inside the converter, so it is charged before the call. It is
    // released only after the pass-through test, not right after the call: that test unwraps
    // a wrapped source again, the copy the charge's source-size share covers. A cache hit
    // decodes nothing, and is charged all the same.
    let charge = decode_charge(&ctx.budget, bytes, format)
        .map_err(|error| conversion_failure(name, error))?;
    let converted = ctx
        .converter
        .convert(source_hash(bytes), bytes, format, target, ctx.cache)
        .map_err(|error| conversion_failure(name, error))?;
    let wrapped_dds = format == SourceFormat::Dds && wezlib::is_wrapped(bytes);
    let pass_through = match ctx.version.engine() {
        Engine::PreFox => wrapped_dds && keeps_blocks(bytes, &converted),
        Engine::Fox => false,
    };
    drop(charge);
    if pass_through {
        return Ok(bytes.to_vec());
    }
    // The converter hands out the cache's own buffer, which it may hand out again for the same
    // source; the CPK entry owns its bytes, so the one copy of the texture is here.
    Ok(converted.to_vec())
}

/// Whether `converted`, the DDS the converter made of the WESYS-wrapped DDS `wrapped`, holds
/// the source's blocks as they are: the same mip data after the header, only the header
/// rebuilt. The data alone decides it: the blocks encode the codec, the size and the chain, so
/// a converter that re-encoded, resized or re-chained the texture wrote other bytes. Read from
/// the converter's output rather than decided again here, so the rule of which codecs a
/// version keeps stays the converter's alone.
fn keeps_blocks(wrapped: &[u8], converted: &[u8]) -> bool {
    // The converter has just read both: it unwrapped and decoded `wrapped`, or a cache hit
    // stands for the same bytes decoded before, and wrote `converted` itself.
    let source = wezlib::decompress(wrapped).expect("the converter has unwrapped this DDS");
    let from = read_layout(&source).expect("the converter has read this DDS's header");
    let to = read_layout(converted).expect("the converter writes a DDS it can read");
    source.get(from.data_offset..) == converted.get(to.data_offset..)
}

/// The charge to `budget` of decoding `bytes`, a texture in `format`: the RGBA size of every
/// level the converted texture carries (`dds_convert::probe`, a raster source's generated
/// chain included), plus the source's own size, for the copy of its blocks or the unwrapped
/// DDS the decode makes. Read from the header alone, so it is taken before the decode; the
/// caller holds the permit as long as the decoded image, bound to a name (`_decode_charge`
/// lives to the end of its scope; `let _ =` would release it at once). A header the probe
/// cannot read is the error the decode would have met.
pub(super) fn decode_charge(
    budget: &Arc<MemoryBudget>,
    bytes: &[u8],
    format: SourceFormat,
) -> Result<Permit, ConvertError> {
    let size = probe(bytes, format)?;
    let width = u64::from(size.width);
    let height = u64::from(size.height);
    let pixels: u64 = (0..size.mipmaps)
        .map(|level| (width >> level).max(1) * (height >> level).max(1))
        .sum();
    let bytes_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let charge = pixels.saturating_mul(4).saturating_add(bytes_len);
    // A charge past `usize` (a 32-bit host only) is over any memory cap, and so is the
    // saturated value.
    Ok(budget.charge(usize::try_from(charge).unwrap_or(usize::MAX)))
}

/// The portrait file `name`, in `format`, holding `bytes`, as the DDS every engine reads
/// (`player_folders.md` "Portraits"): a DDS source under the legacy 128-byte header as it is;
/// one under a DX10 extension header with the header `ftex::dds::header_bytes` writes for its
/// format and its pixel data unchanged, noted in `findings` as `portrait_header_rewritten`, or,
/// for an uncompressed layout that has no `PixelFormat` or whose rows are padded, encoded like
/// a raster source; any other accepted format encoded to BC3 at its own size with the full mip
/// chain, an FTEX's own levels dropped. Its signature and size are the deep pass's checks; a
/// portrait that reaches this point passed them or is kept by `pass_through`, so it is packed
/// whatever its size. The decode is charged to `budget` while it lives.
pub(super) fn portrait(
    budget: &Arc<MemoryBudget>,
    format: SourceFormat,
    name: &str,
    bytes: Vec<u8>,
    findings: &mut Vec<Finding>,
) -> Result<Vec<u8>, TextureError> {
    // Decoded even when the DDS goes out as it is: the deep pass reads only the header, and
    // this decode is what fails the task of a portrait whose header or data is broken.
    let _decode_charge =
        decode_charge(budget, &bytes, format).map_err(|error| conversion_failure(name, error))?;
    let mut decoded = decode(&bytes, format).map_err(|error| conversion_failure(name, error))?;
    match format {
        // The game's own DDS reader crashed PES 19 on a BC3 portrait under a DX10 header with
        // the sRGB id, and took the same blocks under the legacy header: the blocks are fine, so
        // a DX10 header is rebuilt and the data kept (a re-encode would be lossy). An
        // uncompressed DX10 layout has no header `header_bytes` can write, so it takes the
        // raster route below, and so does one whose rows are padded past the tight row, since
        // `header_bytes` declares tightly packed rows the kept data would not match.
        SourceFormat::Dds => {
            let plain = wezlib::decompress_if_wrapped(&bytes)
                .map_err(|error| conversion_failure(name, ConvertError::Wesys(error)))?;
            let layout = read_layout(&plain)
                .map_err(|error| conversion_failure(name, ConvertError::Ftex(error)))?;
            if layout.data_offset == 128 {
                return Ok(bytes);
            }
            match layout.pixel {
                DdsPixel::Format(pixel_format) if layout.row_pitch.is_none() => {
                    // `read_layout` maps the DXGI id to a format and keeps neither; it is the
                    // DX10 header's first field, right after the 128-byte legacy header.
                    let dxgi = u32::from_le_bytes([plain[128], plain[129], plain[130], plain[131]]);
                    let mut rewritten = ftex::dds::header_bytes(
                        pixel_format,
                        layout.width,
                        layout.height,
                        layout.mipmaps,
                    );
                    rewritten.extend_from_slice(&plain[layout.data_offset..]);
                    findings.push((
                        Code::PortraitHeaderRewritten,
                        Disposition::Keep,
                        vec![("file", name.to_owned()), ("dxgi", dxgi.to_string())],
                    ));
                    return Ok(rewritten);
                }
                DdsPixel::Format(_) | DdsPixel::Uncompressed { .. } => {
                    decoded.mips.truncate(1);
                    decoded.authored_mips = false;
                    decoded.blocks = None;
                }
            }
        }
        // The plan gives every portrait that is not a DDS the full chain; an FTEX's authored
        // levels, which the encoder would otherwise keep, are the one thing it would add over
        // a raster source, and the plan does not ask for them. Its blocks go too: the encoder
        // emits a source's blocks as they are when they are already BC3, whatever their level
        // count, so a BC3 FTEX would keep its own chain.
        SourceFormat::Ftex => {
            decoded.mips.truncate(1);
            decoded.authored_mips = false;
            decoded.blocks = None;
        }
        SourceFormat::Png
        | SourceFormat::Jpeg
        | SourceFormat::Bmp
        | SourceFormat::WebP
        | SourceFormat::Tga
        | SourceFormat::Tiff => {}
    }
    encode_dds(&decoded, BlockCodec::Bc3).map_err(|error| conversion_failure(name, error))
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
            target: crate::processing::EntryTarget::GamePaths {
                engine: version.engine(),
            },
            budget: MemoryBudget::new(usize::MAX),
            compress_dds: false,
        }
    }

    /// A budget no test fills, for the tests that check bytes, not charges.
    fn unlimited() -> Arc<MemoryBudget> {
        MemoryBudget::new(usize::MAX)
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

    #[test]
    fn a_wrapped_dds_whose_blocks_the_version_keeps_is_emitted_as_it_is_on_pre_fox_only() {
        // The tracer's kit, BC1, which PES 17 reads as it is: the wrapped source verbatim.
        let dds = tracer_kit();
        let wrapped = wezlib::compress(&dds);
        let on_pes_17 = convert(
            &context(PesVersion::Pes17),
            SourceFormat::Dds,
            "kit.dds",
            &wrapped,
        )
        .unwrap();
        assert!(on_pes_17 == wrapped, "the wrapped source as it is");
        // On Fox it is unwrapped and converted, to the FTEX its plain bytes give.
        let decoded = decode(&dds, SourceFormat::Dds).unwrap();
        let on_pes_21 = convert(
            &context(PesVersion::Pes21),
            SourceFormat::Dds,
            "kit.dds",
            &wrapped,
        )
        .unwrap();
        let fox = Target {
            version: PesVersion::Pes21,
            role: TextureRole::Color,
        };
        assert!(on_pes_21 == dds_convert::convert(&decoded, fox).unwrap());
        // A normal map keeps no BC1 blocks: the wrapped source is converted, to the BC3 its
        // plain bytes give, and left to the entries' wrapping.
        let normal = convert(
            &context(PesVersion::Pes17),
            SourceFormat::Dds,
            "kit_nrm.dds",
            &wrapped,
        )
        .unwrap();
        let pre_fox_normal = Target {
            version: PesVersion::Pes17,
            role: TextureRole::Normal,
        };
        assert!(normal == dds_convert::convert(&decoded, pre_fox_normal).unwrap());
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
        let mut findings = Vec::new();
        assert_eq!(
            portrait(
                &unlimited(),
                SourceFormat::Dds,
                "portrait.dds",
                tracer.clone(),
                &mut findings
            )
            .unwrap(),
            tracer
        );
        assert!(findings.is_empty(), "a legacy-header DDS: {findings:?}");
        for (name, format) in [
            ("portrait.png", SourceFormat::Png),
            ("portrait.webp", SourceFormat::WebP),
        ] {
            let dds = portrait(
                &unlimited(),
                format,
                name,
                texture_fixture(name),
                &mut findings,
            )
            .unwrap();
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
        let dds = portrait(
            &unlimited(),
            SourceFormat::Png,
            "player_05.png",
            odd.clone(),
            &mut findings,
        )
        .unwrap();
        let decoded = decode(&dds, SourceFormat::Dds).unwrap();
        assert_eq!((decoded.width, decoded.height), (300, 300));
        assert!(findings.is_empty(), "raster sources: {findings:?}");
        // Every source is still decoded, a DDS included: a BC6H DDS is the codec finding, and
        // PNG bytes under a `.dds` name fail the task.
        assert_eq!(
            finding(portrait(
                &unlimited(),
                SourceFormat::Dds,
                "portrait.dds",
                texture_fixture("bc6h.dds"),
                &mut findings
            )),
            (Code::TextureCodecUnsupported, "portrait.dds".to_owned())
        );
        let Err(TextureError::Other(error)) = portrait(
            &unlimited(),
            SourceFormat::Dds,
            "portrait.dds",
            odd,
            &mut findings,
        ) else {
            panic!("PNG bytes under a `.dds` name are not a DDS");
        };
        assert_eq!(format!("{error}"), "portrait.dds: cannot convert");
    }

    #[test]
    fn an_uncompressed_dx10_portrait_is_encoded_like_a_raster_one() {
        // A 4x4 R8G8B8A8 DDS under a DX10 header (DXGI 28), one level: no legacy header can
        // carry it, so it is a BC3 DDS with the full chain, and no header was rewritten.
        let mut dds = ftex::dds::header_bytes(ftex::PixelFormat::Bc7, 4, 4, 1);
        dds[128..132].copy_from_slice(&28u32.to_le_bytes());
        dds.extend_from_slice(&[0x80; 4 * 4 * 4]);
        let mut findings = Vec::new();
        let encoded = portrait(
            &unlimited(),
            SourceFormat::Dds,
            "portrait.dds",
            dds,
            &mut findings,
        )
        .unwrap();
        let decoded = decode(&encoded, SourceFormat::Dds).unwrap();
        assert_eq!(
            decoded.blocks.as_ref().map(|blocks| blocks.codec),
            Some(BlockCodec::Bc3)
        );
        assert_eq!(
            (decoded.width, decoded.height, decoded.mips.len()),
            (4, 4, 3)
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn a_dx10_bgra8_portrait_with_padded_rows_is_encoded_like_a_raster_one() {
        // A 4x4 B8G8R8A8 DDS under a DX10 header (DXGI 87) declaring a 20-byte row pitch over
        // 16-byte rows: the legacy header declares tight rows, so it is not re-headered.
        let mut dds = ftex::dds::header_bytes(ftex::PixelFormat::Bc7, 4, 4, 1);
        dds[128..132].copy_from_slice(&87u32.to_le_bytes());
        // The header's flags (offset 8) gain DDSD_PITCH (0x8); the pitch is at offset 20.
        let flags = u32::from_le_bytes(dds[8..12].try_into().unwrap()) | 0x8;
        dds[8..12].copy_from_slice(&flags.to_le_bytes());
        dds[20..24].copy_from_slice(&20u32.to_le_bytes());
        for _ in 0..4 {
            dds.extend_from_slice(&[0x80; 16]);
            dds.extend_from_slice(&[0; 4]);
        }
        let layout = read_layout(&dds).unwrap();
        assert_eq!(
            (layout.pixel, layout.row_pitch),
            (DdsPixel::Format(ftex::PixelFormat::Argb8), Some(20))
        );
        let mut findings = Vec::new();
        let encoded = portrait(
            &unlimited(),
            SourceFormat::Dds,
            "portrait.dds",
            dds,
            &mut findings,
        )
        .unwrap();
        let decoded = decode(&encoded, SourceFormat::Dds).unwrap();
        assert_eq!(
            decoded.blocks.as_ref().map(|blocks| blocks.codec),
            Some(BlockCodec::Bc3)
        );
        assert_eq!(
            (decoded.width, decoded.height, decoded.mips.len()),
            (4, 4, 3)
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn a_decode_is_charged_every_level_s_rgba_size_and_its_source_s() {
        let budget = MemoryBudget::new(usize::MAX);
        // A 300x300 PNG holds one level; its conversion generates the chain down to 1x1:
        // 300, 150, 75, 37, 18, 9, 4, 2 and 1 pixels across.
        let png = texture_fixture("odd.png");
        let levels =
            300 * 300 + 150 * 150 + 75 * 75 + 37 * 37 + 18 * 18 + 9 * 9 + 4 * 4 + 2 * 2 + 1;
        let charge = decode_charge(&budget, &png, SourceFormat::Png).unwrap();
        assert_eq!(charge.size(), 4 * levels + png.len());
        drop(charge);
        // The tracer's kit: a 16x16 BC1 DDS with its five levels.
        let kit = tracer_kit();
        let charge = decode_charge(&budget, &kit, SourceFormat::Dds).unwrap();
        assert_eq!(
            charge.size(),
            4 * (16 * 16 + 8 * 8 + 4 * 4 + 2 * 2 + 1) + kit.len()
        );
        drop(charge);
        // The tracer's portrait: a 64x128 BC3 DDS with eight levels, the narrow side held at
        // one pixel once it gets there.
        let portrait = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Midcup Tracer/Players/05 - The Chad Stormworks Player/portrait.dds"),
        )
        .unwrap();
        let charge = decode_charge(&budget, &portrait, SourceFormat::Dds).unwrap();
        // The last two levels are 1x2 and 1x1.
        let levels = 64 * 128 + 32 * 64 + 16 * 32 + 8 * 16 + 4 * 8 + 2 * 4 + 2 + 1;
        assert_eq!(charge.size(), 4 * levels + portrait.len());
        drop(charge);
        // A header the probe cannot read is the decode's error.
        assert!(decode_charge(&budget, b"not a DDS", SourceFormat::Dds).is_err());
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
