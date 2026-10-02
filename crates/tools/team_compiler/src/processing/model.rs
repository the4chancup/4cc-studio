//! A player folder's Fox face: its models packed into one `face.fpk` per roster slot, its
//! textures converted once into the player's common folder (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", steps 2, 6 and 7).

use std::collections::BTreeMap;

use aesthetics_export::PlayerFolder;
use fmdl::FmdlFile;
use fmdl::ops::paths::rewrite_texture_paths;
use fpk::{FpkFile, FpkKind};

use super::{Entry, TaskFiles, take, texture};
use crate::paths;
use crate::plan::subset::{FaceFile, face_file, file_stem, holds_hair_model};

/// The face content of `folder`, compiled from its files' bytes in `files` for the player ids
/// `player_ids` of team `team_id`: the textures, then each slot's `face.fpk` and `face.fpkd`.
pub(super) fn face(
    folder: &PlayerFolder,
    player_ids: &[u32],
    team_id: u16,
    files: &mut TaskFiles,
) -> anyhow::Result<Vec<Entry>> {
    let holds_hair_model = holds_hair_model(folder);
    let mut models = Vec::new();
    let mut textures = BTreeMap::new();
    let mut package = FpkFile::new(FpkKind::Fpk);
    for file in &folder.files {
        let role = face_file(&folder.path, file, holds_hair_model)
            .expect("planning skips every export holding a face file with no Phase 3 role");
        let bytes = take(files, file);
        match role {
            FaceFile::Model(stem) => models.push((stem, FmdlFile::read(&bytes)?)),
            FaceFile::Texture(stem, format) => {
                textures.insert(stem, texture::to_ftex(format, file.path.name(), bytes)?);
            }
            FaceFile::Packed(name) => {
                package.insert(name.to_owned(), bytes);
            }
        }
    }

    // A folder's textures go to one common subfolder keyed by the source folder's name, once
    // however many slots map the folder: every slot's model points at that one copy.
    let folder_name = folder.path.name();
    let common_directory = paths::player_common_directory(team_id, folder_name);
    // A texture the folder does not hold is one of the game's own; its directory names the
    // team as `000`, which becomes the team's id.
    let team_segment = format!("/{team_id}/");
    for (stem, mut model) in models {
        rewrite_texture_paths(&mut model, |path| {
            if textures.contains_key(file_stem(&path.file_name)) {
                path.directory.clone_from(&common_directory);
            } else {
                path.directory = path.directory.replace("/000/", &team_segment);
            }
        })?;
        package.insert(format!("{stem}.fmdl"), model.write());
    }

    let mut entries: Vec<Entry> = textures
        .into_iter()
        .map(|(stem, bytes)| {
            (
                paths::player_common_texture(team_id, folder_name, &stem),
                bytes,
            )
        })
        .collect();
    let package = package.write();
    // The game opens a face's `.fpkd` beside its `.fpk`; with the textures in the common
    // folder there is nothing to put in it, so it is an empty package.
    let empty = FpkFile::new(FpkKind::Fpkd).write();
    // The game finds a face by player id, so each slot gets its own copy of the package; the
    // last slot takes the buffer itself rather than one more copy.
    if let Some((last_id, other_ids)) = player_ids.split_last() {
        for player_id in other_ids {
            let folder = paths::face_folder(*player_id);
            entries.push((format!("{folder}/face.fpk"), package.clone()));
            entries.push((format!("{folder}/face.fpkd"), empty.clone()));
        }
        let folder = paths::face_folder(*last_id);
        entries.push((format!("{folder}/face.fpk"), package));
        entries.push((format!("{folder}/face.fpkd"), empty));
    }
    Ok(entries)
}
