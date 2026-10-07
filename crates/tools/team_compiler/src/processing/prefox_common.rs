//! The pre-Fox Common output's models and `.mtl` files (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 4): every `.model` and `.mtl` directly in the
//! export's `Common/` folder written once into the team's Common output, where the game loads
//! the ones a player's `face.xml` names through a `.common` link. The Common textures beside
//! them are the Common textures task's.

use std::collections::BTreeMap;

use aesthetics_export::{FileDescriptor, FileKind, ModelFormat};
use pes_version::Engine;

use super::prefox_face::rewritten_materials;
use super::{Entry, TaskFailure, TaskFiles, take};
use crate::face_xml::packed_model_name;
use crate::paths;
use crate::plan::subset::file_stem;

/// The Common output's entries for `common`, the export's `Common/` `.model` and `.mtl`
/// files, compiled from their bytes in `files` for team `team_id`: each model as it is under
/// its packed name (`packed_model_name`), as the `face.xml` path naming it expects; each `.mtl`
/// under its own name, every texture path naming one of `texture_stems` (the Common textures'
/// stems, folded, each as spelled) pointed at that texture in the team's Common output as its
/// DDS. Two files packing under one name fail the task: neither can be dropped silently.
pub(super) fn common_models(
    common: &[FileDescriptor],
    texture_stems: &BTreeMap<String, String>,
    team_id: u16,
    files: &mut TaskFiles,
) -> Result<Vec<Entry>, TaskFailure> {
    let directory = paths::common_texture_directory(Engine::PreFox, team_id);
    let places = [(texture_stems, directory.as_str())];
    let mut packed: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for file in common {
        let name = file.path.name();
        // Planning lists the `.model` and `.mtl` files only. The deep pass has dropped a
        // `Common/` `.mtl` that does not read, so a failure of `rewritten_materials` here is
        // not a member's mistake.
        let (packed_name, bytes) = if file.kind == FileKind::Model(ModelFormat::PesModel) {
            (packed_model_name(file_stem(name)), take(files, file))
        } else {
            let bytes = rewritten_materials(file, &take(files, file), &places)?;
            (name.to_owned(), bytes)
        };
        if packed.contains_key(&packed_name) {
            return Err(anyhow::anyhow!("two files of Common/ are packed as {packed_name}").into());
        }
        packed.insert(packed_name, bytes);
    }
    Ok(packed
        .into_iter()
        .map(|(name, bytes)| (paths::pre_fox_common_file(team_id, &name), bytes))
        .collect())
}
