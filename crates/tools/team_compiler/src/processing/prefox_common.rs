//! The pre-Fox Common output's models and `.mtl` files (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 4): every `.model` and `.mtl` of a directory of the
//! export's `Common/` folder (`Common/` itself or a subfolder, packed at its own path:
//! `pipeline.md` "Common") written once into the team's Common output (a `.model` the
//! conversion pre-check finds posed off the version's skeleton moved onto it), and every FMDL
//! there converted once into a `.model` and its material set (step 1, "Format conversion"),
//! where the game loads the ones a player's `face.xml` names through a `.common` link. The Common
//! textures beside them, the template environment map a converted metal material names
//! included, are the Common textures task's.

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::{FileDescriptor, FileKind, ModelFormat};
use pes_version::Engine;
use vtree::ScopePath;

use super::conversion::{ConvertedMaterials, fmdl_for_pre_fox, model_for_pre_fox, source_name};
use super::prefox_face::{
    MaterialPlaces, add_environment_map, converted_material_name, point_materials,
    point_reserved_kit_stems, rewritten_materials,
};
use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::face_xml::packed_model_name;
use crate::mtl_search::mtl_for;
use crate::paths::{self, TextureDirectory};
use crate::plan::roles::file_stem;

/// The Common output's entries for `common`, the files of the `Common/` directory at `folder`
/// that planning lists (`TaskKind::CommonModels`), compiled from their bytes in `files` for
/// team `team_id`, each below the directory's own path in the team's Common output
/// (`paths::common_subpath`): each `.model` under its packed name (`packed_model_name`), as
/// the `face.xml` path naming it expects, as it is or, when the conversion pre-check flags it
/// (`model_for_pre_fox`, with the `.mtl` its search finds in the directory, its findings naming
/// it by its file name), moved onto `ctx.version`'s skeleton; a `.model` with no `.mtl` in the
/// directory is packed as it is, the import having no set to read it with; each `.mtl` under
/// its own name, every texture path naming one of `texture_stems` (the directory's textures'
/// stems, folded, each as spelled) pointed at that texture in the team's Common output as its
/// DDS. Each FMDL is converted for `ctx.version` (`fmdl_for_pre_fox`, the `.skl` of its stem
/// as its bind pose, its findings noted in `findings` naming it by its file name), its
/// `.model` packed under its packed name and its material set as `<stem>.mtl`, pointed as the
/// face points a converted set: each metal material first given the environment map in the
/// team's Common output (`add_environment_map`), then the paths naming a Common texture, then
/// the reserved kit stems. A conversion that fails fails the task. Two files packing under one
/// name (case-folded), a member's `.mtl` of a converted FMDL's `<stem>.mtl` name among them,
/// fail the task: neither can be dropped silently.
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
    // The directory's own place in the team's Common output, where its textures are packed
    // (`texture::common_textures`): a path naming one of them is pointed there. The
    // environment map and the reserved kit stems stay at the Common output's own directory,
    // where the map is emitted and the modded exes substitute the kit.
    let subpath = paths::common_subpath(folder);
    let textures_directory = TextureDirectory::plain(format!("{directory}{subpath}"));
    let places = MaterialPlaces {
        by_name: vec![(texture_stems, &textures_directory)],
        below: None,
    };
    let mut written: Vec<(String, Vec<u8>)> = Vec::new();
    for file in common {
        let name = file.path.name();
        let stem = file_stem(name);
        match file.kind {
            FileKind::Model(ModelFormat::PesModel) => {
                let source = take(files, file);
                // `Common/` is both the model's folder and the Common folder its search
                // looks in. With no `.mtl` there, the deep pass drops nothing: a linking
                // player's own `.mtl` may define the model's materials (his link's search
                // finds it), or no player links the model. The conversion's import needs a
                // set, so such a model is packed as it is.
                let bytes = match mtl_for(&file.path, folder, common, common) {
                    Some(material) => {
                        let mtl = files
                            .get(&material.path)
                            .expect("planning lists every `.mtl` of `Common/` for the task");
                        model_for_pre_fox(
                            &source_name(&file.path, folder),
                            source,
                            mtl,
                            ctx,
                            findings,
                            ConvertedMaterials::Converted,
                        )?
                    }
                    None => source,
                };
                written.push((packed_model_name(stem), bytes));
            }
            FileKind::Model(ModelFormat::Fmdl) => {
                let skeleton = skeleton_beside(common, name).map(|skeleton| take(files, skeleton));
                let conversion = fmdl_for_pre_fox(
                    &source_name(&file.path, folder),
                    &take(files, file),
                    skeleton.as_deref(),
                    ctx,
                    findings,
                    ConvertedMaterials::Converted,
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
                // Read in place, not taken: a `.model` sorting after it (`zz.model` using
                // `materials.mtl`) reads it for its pre-check.
                let source = files
                    .get(&file.path)
                    .expect("the coordinator reads every file `TaskKind::files` lists");
                let bytes = rewritten_materials(file, source, &places)?;
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
        .map(|(name, bytes)| {
            let path = paths::pre_fox_common_file(team_id, &format!("{subpath}{name}"));
            (path, bytes)
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

/// The `.skl` among `common`, the files of one `Common/` directory, paired with its model named
/// `model_name` (`legs.skl` for `legs.fmdl`), matched by case-folded name, when there is one:
/// the bind pose of the FMDL the task converts. Not `roles::common_skeleton`, which looks
/// directly in `Common/` alone, as a link does: a subfolder's FMDL pairs its own directory's.
fn skeleton_beside<'a>(
    common: &'a [FileDescriptor],
    model_name: &str,
) -> Option<&'a FileDescriptor> {
    let key = vtree::fold_name(&format!("{}.skl", file_stem(model_name)));
    common
        .iter()
        .find(|file| vtree::fold_name(file.path.name()) == key)
}
