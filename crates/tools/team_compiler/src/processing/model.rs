//! One package of a model folder's Fox models (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", steps 2, 3 and 7): its models renamed to their allowed
//! names, their texture paths pointed at the folder's texture home, packed with the files
//! that go beside them into one `.fpk` emitted under each of the package's ids.

use std::collections::BTreeSet;

use fmdl::FmdlFile;
use fmdl::ops::paths::rewrite_texture_paths;
use fpk::{FpkFile, FpkKind};

use super::{Entry, TaskFiles, take};
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{FolderModels, ModelPackage, PlayerFile, file_stem, player_file};
use crate::templates;

/// The `package` of `folder`, compiled from its files' bytes in `files` for team `team_id`
/// and emitted under each of `ids`: the package's `.fpk` and an empty `.fpkd` per id.
pub(super) fn package(
    folder: &ModelFolder,
    package: ModelPackage,
    ids: &[u32],
    team_id: u16,
    files: &mut TaskFiles,
) -> anyhow::Result<Vec<Entry>> {
    let folder_models = FolderModels::of(&folder.path, &folder.files);
    let mut models = Vec::new();
    let mut texture_stems = BTreeSet::new();
    let mut fpk = FpkFile::new(FpkKind::Fpk);
    for file in &folder.files {
        let role = player_file(&folder.path, file, &folder_models)
            .expect("planning skips every export holding a player file with no role yet");
        match role {
            PlayerFile::Model {
                package: owner,
                name,
            } if owner == package => {
                models.push((name, FmdlFile::read(&take(files, file))?));
            }
            PlayerFile::Packed {
                package: owner,
                name,
            } if owner == package => {
                fpk.insert(name.to_owned(), take(files, file));
            }
            // The textures are the textures task's; this task only points its models at them.
            PlayerFile::Texture(stem, _) => {
                texture_stems.insert(stem);
            }
            PlayerFile::Model { .. } | PlayerFile::Packed { .. } => {}
        }
    }
    // The game loads boots with a `boots.skl` beside the model; a boots model with no
    // skeleton of its own gets the standard full-body one.
    if package == ModelPackage::Boots && fpk.get("boots.skl").is_none() {
        fpk.insert("boots.skl".to_owned(), templates::BOOTS_SKELETON.to_vec());
    }

    // A folder's textures sit in its one texture home, once however many ids the package is
    // emitted under: every copy of the model points at that one location.
    let texture_directory = folder.textures.directory(team_id);
    // A texture the folder does not hold is one of the game's own; its directory names the
    // team as `000`, which becomes the team's id.
    let team_segment = format!("/{team_id}/");
    for (name, mut model) in models {
        rewrite_texture_paths(&mut model, |path| {
            if texture_stems.contains(file_stem(&path.file_name)) {
                path.directory.clone_from(&texture_directory);
            } else {
                path.directory = path.directory.replace("/000/", &team_segment);
            }
        })?;
        fpk.insert(format!("{name}.fmdl"), model.write());
    }

    let fpk = fpk.write();
    // The game opens a package's `.fpkd` beside its `.fpk`; with the textures in the common
    // folder there is nothing to put in it, so it is an empty package.
    let empty = FpkFile::new(FpkKind::Fpkd).write();
    let stem = package.file_stem();
    let mut entries = Vec::new();
    // The game finds a package by its id, so each slot gets its own copy; the last slot takes
    // the buffer itself rather than one more copy.
    if let Some((last_id, other_ids)) = ids.split_last() {
        for id in other_ids {
            let folder = paths::package_folder(package, *id);
            entries.push((format!("{folder}/{stem}.fpk"), fpk.clone()));
            entries.push((format!("{folder}/{stem}.fpkd"), empty.clone()));
        }
        let folder = paths::package_folder(package, *last_id);
        entries.push((format!("{folder}/{stem}.fpk"), fpk));
        entries.push((format!("{folder}/{stem}.fpkd"), empty));
    }
    Ok(entries)
}
