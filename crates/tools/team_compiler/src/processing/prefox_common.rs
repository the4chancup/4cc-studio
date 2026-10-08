//! The pre-Fox Common output's models and `.mtl` files (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 4): every `.model` and `.mtl` directly in the
//! export's `Common/` folder written once into the team's Common output, and every FMDL there
//! converted once into a `.model` and its material set (step 1, "Format conversion"), where the
//! game loads the ones a player's `face.xml` names through a `.common` link. The Common
//! textures beside them, the template environment map a converted metal material names
//! included, are the Common textures task's.

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::{FileDescriptor, FileKind, ModelFormat};
use pes_version::Engine;
use vtree::ScopePath;

use super::conversion::{PreFoxMaterials, fmdl_for_pre_fox, source_name};
use super::prefox_face::{
    add_environment_map, converted_material_name, point_materials, point_reserved_kit_stems,
    rewritten_materials,
};
use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::face_xml::packed_model_name;
use crate::paths;
use crate::plan::subset::{common_skeleton, file_stem};

/// The Common output's entries for `common`, the files of the export's `Common/` folder at
/// `folder` that planning lists (`TaskKind::CommonModels`), compiled from their bytes in
/// `files` for team `team_id`: each `.model` as it is under its packed name
/// (`packed_model_name`), as the `face.xml` path naming it expects; each `.mtl` under its own
/// name, every texture path naming one of `texture_stems` (the Common textures' stems, folded,
/// each as spelled) pointed at that texture in the team's Common output as its DDS. Each FMDL
/// is converted for `ctx.version` (`fmdl_for_pre_fox`, the `.skl` of its stem as its bind
/// pose, its findings noted in `findings` naming it by its file name), its `.model` packed
/// under its packed name and its material set as `<stem>.mtl`, pointed as the face points a
/// converted set: each metal material first given the environment map in the team's Common
/// output (`add_environment_map`), then the paths naming a Common texture, then the reserved
/// kit stems. A conversion that fails fails the task. Two files packing under one name
/// (case-folded), a member's `.mtl` of a converted FMDL's `<stem>.mtl` name among them, fail
/// the task: neither can be dropped silently.
pub(super) fn common_models(
    folder: &ScopePath,
    common: &[FileDescriptor],
    texture_stems: &BTreeMap<String, String>,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Entry>, TaskFailure> {
    let directory = paths::common_texture_directory(Engine::PreFox, team_id);
    let places = [(texture_stems, directory.as_str())];
    let mut written: Vec<(String, Vec<u8>)> = Vec::new();
    for file in common {
        let name = file.path.name();
        let stem = file_stem(name);
        match file.kind {
            FileKind::Model(ModelFormat::PesModel) => {
                written.push((packed_model_name(stem), take(files, file)));
            }
            FileKind::Model(ModelFormat::Fmdl) => {
                let skeleton = common_skeleton(common, name).map(|skeleton| take(files, skeleton));
                let conversion = fmdl_for_pre_fox(
                    &source_name(&file.path, folder),
                    &take(files, file),
                    skeleton.as_deref(),
                    ctx,
                    findings,
                    PreFoxMaterials::Converted,
                )?;
                let mut materials = conversion.materials;
                // Before the pointing, which respells the environment map's path as `Common/`
                // spells its own `env` texture when it holds one.
                add_environment_map(&mut materials, &directory);
                point_materials(&mut materials, &places);
                point_reserved_kit_stems(&mut materials, &directory);
                written.push((packed_model_name(stem), conversion.model));
                written.push((converted_material_name(stem), materials.write()));
            }
            // The deep pass has dropped a `Common/` `.mtl` that does not read, so a failure of
            // `rewritten_materials` here is not a member's mistake.
            FileKind::Mtl => {
                let bytes = rewritten_materials(file, &take(files, file), &places)?;
                written.push((name.to_owned(), bytes));
            }
            // A `.skl` is read with the FMDL of its stem, its bind pose, above; planning lists
            // no other kind (`plan::common_model_files`).
            FileKind::Skl
            | FileKind::Model(ModelFormat::Gltf)
            | FileKind::Texture
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::SharedLink(_)
            | FileKind::CommonLink
            | FileKind::Marker(_)
            | FileKind::Metadata(_)
            | FileKind::Other => {}
        }
    }
    // Folded, as the game's file system folds them: `Legs.mtl` and `legs.mtl` are one file.
    let mut names = BTreeSet::new();
    for (name, _) in &written {
        if !names.insert(vtree::fold_name(name)) {
            return Err(anyhow::anyhow!("two files of Common/ are packed as {name}").into());
        }
    }
    let mut entries: Vec<Entry> = written
        .into_iter()
        .map(|(name, bytes)| (paths::pre_fox_common_file(team_id, &name), bytes))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}
