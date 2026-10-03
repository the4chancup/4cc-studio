//! Texture conversion for the Fox engine, which reads FTEX (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 5): every accepted image format through
//! `dds_convert`, which picks the codec the target version reads; a model folder's textures as
//! one unit (step 6) and the export's Common textures as another ("Resolved decisions",
//! "Common textures are one task of their export").

use std::collections::BTreeMap;

use aesthetics_export::FileDescriptor;
use anyhow::Context;
use dds_convert::{SourceFormat, Target, TextureRole, source_hash};
use studio_core::Disposition;

use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::messages::Code;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem, texture_format};

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
/// not a `.common` link uses it. A texture that cannot convert fails the task, and with it
/// every Common texture; the players linking a Common model still commit on their own.
pub(super) fn common_textures(
    textures: &[FileDescriptor],
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
) -> Result<Vec<Entry>, TaskFailure> {
    textures
        .iter()
        .map(|file| {
            let name = file.path.name();
            let format = texture_format(name).expect(
                "planning lists the `Common/` textures by an extension `dds_convert` accepts",
            );
            let bytes = convert(ctx, format, name, &take(files, file))?;
            Ok((paths::common_texture(team_id, file_stem(name)), bytes))
        })
        .collect()
}

/// The texture file `name`, in `format`, converted for the run's version through its
/// converter: an FTEX on every Fox target, in the codec the version reads, with the mip chain
/// a raster source lacks generated, in the role the file's stem gives it. A source that cannot
/// be converted is an error naming the file.
pub(super) fn convert(
    ctx: &CompileContext,
    format: SourceFormat,
    name: &str,
    bytes: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let target = Target {
        version: ctx.version,
        role: texture_role(file_stem(name)),
    };
    let converted = ctx
        .converter
        .convert(source_hash(bytes), bytes, format, target, ctx.cache)
        .with_context(|| format!("{name}: cannot convert to FTEX"))?;
    // The converter hands out the cache's own buffer, which it may hand out again for the same
    // source; the CPK entry owns its bytes, so the one copy of the texture is here.
    Ok(converted.to_vec())
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
        assert_eq!(format!("{error}"), "kit.dds: cannot convert to FTEX");
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
