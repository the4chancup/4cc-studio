//! Texture conversion for the Fox engine, which reads FTEX, a model folder's textures as one
//! unit (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps", step 6) and the
//! export's Common textures as another ("Resolved decisions", "Common textures are one task
//! of their export").

use std::collections::BTreeMap;

use aesthetics_export::FileDescriptor;
use anyhow::Context;
use studio_core::Disposition;

use super::{Entry, Finding, TaskFailure, TaskFiles, take};
use crate::messages::Code;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, TextureFormat, file_stem, texture_format};

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
            let bytes = to_ftex(format, file.path.name(), take(files, file))?;
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
    files: &mut TaskFiles,
) -> Result<Vec<Entry>, TaskFailure> {
    textures
        .iter()
        .map(|file| {
            let name = file.path.name();
            let format = texture_format(name)
                .expect("planning lists only the `.dds` and `.ftex` files of `Common/`");
            let bytes = to_ftex(format, name, take(files, file))?;
            Ok((paths::common_texture(team_id, file_stem(name)), bytes))
        })
        .collect()
}

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
