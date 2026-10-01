//! A player folder's Fox face: its models packed into one `face.fpk` per roster slot, its
//! textures converted once into the player's common folder (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", steps 2, 6 and 7).

use std::collections::BTreeMap;

use aesthetics_export::{FileDescriptor, FileKind, ModelFormat, PlayerFolder};
use anyhow::bail;
use fmdl::FmdlFile;
use fmdl::ops::paths::rewrite_texture_paths;
use fpk::{FpkFile, FpkKind};
use vtree::ScopePath;

use super::{Entry, TaskFiles, file_stem, take, texture};
use crate::paths;

/// The face model names the Fox engine loads.
const FACE_STEMS: [&str; 4] = ["face_high", "hair_high", "oral", "fcl_hair"];

/// What one file of a player folder becomes in the face output.
#[derive(Debug, PartialEq, Eq)]
enum FaceFile {
    /// A face model with this stem, packed as `<stem>.fmdl` with its texture paths rewritten.
    Model(String),
    /// A texture with this stem, converted into the player's common folder.
    Texture(String),
    /// A file packed into the face package as it is, under this name.
    Packed(&'static str),
}

/// The face content of `folder`, compiled from its files' bytes in `files` for the player ids
/// `player_ids` of team `team_id`: the textures, then each slot's `face.fpk` and `face.fpkd`.
pub(super) fn face(
    folder: &PlayerFolder,
    player_ids: &[u32],
    team_id: u16,
    files: &mut TaskFiles,
) -> anyhow::Result<Vec<Entry>> {
    let holds_hair_model = folder
        .files
        .iter()
        .any(|file| file.path.name() == "fcl_hair.fmdl");
    let mut models = Vec::new();
    let mut textures = BTreeMap::new();
    let mut package = FpkFile::new(FpkKind::Fpk);
    for file in &folder.files {
        let role = face_file(&folder.path, file, holds_hair_model)?;
        let bytes = take(files, file);
        match role {
            FaceFile::Model(stem) => models.push((stem, FmdlFile::read(&bytes)?)),
            FaceFile::Texture(stem) => {
                textures.insert(stem, texture::to_ftex(file.path.name(), bytes)?);
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
    // The game finds a face by player id, so each slot gets its own copy of the package.
    for player_id in player_ids {
        let folder = paths::face_folder(*player_id);
        entries.push((format!("{folder}/face.fpk"), package.clone()));
        entries.push((format!("{folder}/face.fpkd"), empty.clone()));
    }
    Ok(entries)
}

/// What `file` of the player folder at `folder` becomes, or why it cannot be compiled yet.
/// `holds_hair_model`: the folder holds `fcl_hair.fmdl`, the model a `fcl_hair.skl` pairs with.
fn face_file(
    folder: &ScopePath,
    file: &FileDescriptor,
    holds_hair_model: bool,
) -> anyhow::Result<FaceFile> {
    let name = file.path.name();
    if file.path.parent().as_ref() != Some(folder) {
        bail!(
            "{}: a file in a subfolder cannot be compiled yet",
            file.path.as_str()
        );
    }
    let stem = file_stem(name);
    match file.kind {
        FileKind::Model(ModelFormat::Fmdl) if FACE_STEMS.contains(&stem) => {
            Ok(FaceFile::Model(stem.to_owned()))
        }
        FileKind::Texture => Ok(FaceFile::Texture(stem.to_owned())),
        // The game loads the hair simulation's skeleton as `fcl_hair_sim.skl`; the export
        // names a skeleton after the model it pairs with, and without that model the skeleton
        // has nothing to drive.
        FileKind::Skl if stem == "fcl_hair" && holds_hair_model => {
            Ok(FaceFile::Packed("fcl_hair_sim.skl"))
        }
        FileKind::Skl if stem == "fcl_hair" => bail!(
            "{}: fcl_hair.skl needs fcl_hair.fmdl beside it",
            file.path.as_str()
        ),
        FileKind::Bin if name == "face_diff.bin" => Ok(FaceFile::Packed("face_diff.bin")),
        FileKind::Fclo if name == "fcl_hair_sim.fclo" => Ok(FaceFile::Packed("fcl_hair_sim.fclo")),
        FileKind::Model(_)
        | FileKind::Skl
        | FileKind::Bin
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::SharedLink(_)
        | FileKind::CommonLink
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => bail!("{}: cannot be compiled yet", file.path.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The role of the file at `relative` in `Players/03 - A`, a folder that holds
    /// `fcl_hair.fmdl` when `holds_hair_model`.
    fn role_in(relative: &str, holds_hair_model: bool) -> Result<FaceFile, String> {
        let path = ScopePath::new(relative).unwrap();
        let file = FileDescriptor {
            size: 0,
            kind: aesthetics_export::classify(path.name()),
            source: path.clone(),
            path,
        };
        face_file(
            &ScopePath::new("Players/03 - A").unwrap(),
            &file,
            holds_hair_model,
        )
        .map_err(|error| error.to_string())
    }

    /// The role of the file at `relative` in a folder that holds `fcl_hair.fmdl`.
    fn role(relative: &str) -> Result<FaceFile, String> {
        role_in(relative, true)
    }

    #[test]
    fn each_face_file_has_its_role_or_cannot_be_compiled_yet() {
        assert_eq!(
            role("Players/03 - A/hair_high.fmdl"),
            Ok(FaceFile::Model("hair_high".to_owned()))
        );
        assert_eq!(
            role("Players/03 - A/face_high.fmdl"),
            Ok(FaceFile::Model("face_high".to_owned()))
        );
        assert_eq!(
            role("Players/03 - A/oral.fmdl"),
            Ok(FaceFile::Model("oral".to_owned()))
        );
        assert_eq!(
            role("Players/03 - A/fcl_hair.fmdl"),
            Ok(FaceFile::Model("fcl_hair".to_owned()))
        );
        assert_eq!(
            role("Players/03 - A/skin.png"),
            Ok(FaceFile::Texture("skin".to_owned()))
        );
        assert_eq!(
            role("Players/03 - A/fcl_hair.skl"),
            Ok(FaceFile::Packed("fcl_hair_sim.skl"))
        );
        assert_eq!(
            role_in("Players/03 - A/fcl_hair.skl", false),
            Err(
                "Players/03 - A/fcl_hair.skl: fcl_hair.skl needs fcl_hair.fmdl beside it"
                    .to_owned()
            )
        );
        assert_eq!(
            role("Players/03 - A/face_diff.bin"),
            Ok(FaceFile::Packed("face_diff.bin"))
        );
        assert_eq!(
            role("Players/03 - A/fcl_hair_sim.fclo"),
            Ok(FaceFile::Packed("fcl_hair_sim.fclo"))
        );
        for refused in [
            "Players/03 - A/torso.fmdl",
            "Players/03 - A/boots.fmdl",
            "Players/03 - A/face_high.model",
            "Players/03 - A/boots.skl",
            "Players/03 - A/face.xml",
            "Players/03 - A/hair.png.common",
            "Players/03 - A/face_diff2.bin",
            "Players/03 - A/Face_Diff.bin",
            "Players/03 - A/fcl_hair.fclo",
        ] {
            assert_eq!(
                role(refused),
                Err(format!("{refused}: cannot be compiled yet")),
                "{refused}"
            );
        }
        assert_eq!(
            role("Players/03 - A/face/face_high.fmdl"),
            Err(
                "Players/03 - A/face/face_high.fmdl: a file in a subfolder cannot be compiled yet"
                    .to_owned()
            )
        );
    }
}
