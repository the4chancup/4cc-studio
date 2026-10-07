//! The referees' marker on Fox (`team_compiler/blue_port.md` "Referee export processing"): the
//! refs export's `ref_marker.dds` converted into the referees' Common output, and the bundled
//! marker model, a flat square under each referee, written as the referees' reserved collar
//! with its base texture pointed at that texture.

use aesthetics_export::FileDescriptor;
use fmdl::ops::paths::rewrite_texture_paths;
use fmdl::{FmdlError, FmdlFile};
use pes_version::Engine;

use super::{CompileContext, Entry, TaskFailure, TaskFiles, texture};
use crate::paths::{self, REFEREE_MARKER_COLLAR, REFEREE_TEAM_ID};

/// The base texture's file name in the bundled marker model, the one the compile repoints.
const BUNDLED_TEXTURE: &str = "cup_logo.dds";

/// The stem the marker texture goes out under in the referees' Common output.
const MARKER_STEM: &str = "ref_marker";

/// The marker `marker`, from its bytes in `files`, as two entries: its texture converted as a
/// Common texture is, under `ref_marker` in the referees' Common output, then the bundled
/// marker model of `ctx`'s templates, its base texture pointed at it, as the referees'
/// collar. A failed conversion fails the task with its texture code, so neither goes out and
/// the referees wear no collar naming a texture the CPK lacks.
pub(super) fn referee_marker(
    marker: &FileDescriptor,
    ctx: &CompileContext,
    files: &mut TaskFiles,
) -> Result<Vec<Entry>, TaskFailure> {
    let texture = texture::common_texture(marker, ctx, files)?;
    let model = marker_model(ctx.templates.referee_marker())?;
    Ok(vec![
        (
            paths::common_texture(Engine::Fox, REFEREE_TEAM_ID, MARKER_STEM),
            texture,
        ),
        (paths::collar(REFEREE_MARKER_COLLAR), model),
    ])
}

/// The marker model `bytes` with its base texture, `cup_logo.dds`, pointed at the marker
/// texture in the referees' Common output; its two other textures, the game's dummy normal and
/// specular maps, as they are.
fn marker_model(bytes: &[u8]) -> Result<Vec<u8>, FmdlError> {
    let mut file = FmdlFile::read(bytes)?;
    rewrite_texture_paths(&mut file, |path| {
        if path.file_name == BUNDLED_TEXTURE {
            path.directory = paths::common_texture_directory(Engine::Fox, REFEREE_TEAM_ID);
            path.file_name = format!("{MARKER_STEM}.dds");
        }
    })?;
    Ok(file.write())
}

#[cfg(test)]
mod tests {
    use fmdl::ops::paths::{TexturePath, texture_paths};
    use fmdl::{Model, Texture};

    use super::*;
    use crate::templates::Templates;

    /// The texture path `directory` + `file_name`.
    fn texture_path(directory: &str, file_name: &str) -> TexturePath {
        TexturePath {
            file_name: file_name.to_owned(),
            directory: directory.to_owned(),
        }
    }

    /// `model` with every material's texture paths blanked, its samplers kept.
    fn without_textures(mut model: Model) -> Model {
        for material in &mut model.materials {
            for (_, texture) in &mut material.textures {
                *texture = Texture {
                    file_name: String::new(),
                    directory: String::new(),
                };
            }
        }
        model
    }

    #[test]
    fn the_marker_model_s_base_texture_names_the_referees_marker_and_nothing_else_changes() {
        let bundled = Templates::embedded().referee_marker().to_vec();
        let bundled_file = FmdlFile::read(&bundled).unwrap();
        let dummies = "/Assets/pes16/model/character/common/sourceimages/";
        assert_eq!(
            texture_paths(&bundled_file).unwrap(),
            [
                texture_path(
                    "/Assets/pes16/model/character/common/000/sourceimages/",
                    "cup_logo.dds"
                ),
                texture_path(dummies, "dummy_nrm.tga"),
                texture_path(dummies, "dummy_srm.tga"),
            ],
            "the bundled model"
        );

        let marker = FmdlFile::read(&marker_model(&bundled).unwrap()).unwrap();

        assert_eq!(
            texture_paths(&marker).unwrap(),
            [
                texture_path(
                    "/Assets/pes16/model/character/common/999/sourceimages/",
                    "ref_marker.dds"
                ),
                texture_path(dummies, "dummy_nrm.tga"),
                texture_path(dummies, "dummy_srm.tga"),
            ]
        );
        // Bones, meshes, groups and materials but their textures: the semantic model is the
        // narrowest equality the crate gives that leaves the texture table aside.
        assert_eq!(
            without_textures(Model::from_file(&marker).unwrap()),
            without_textures(Model::from_file(&bundled_file).unwrap())
        );
    }
}
