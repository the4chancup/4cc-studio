//! The referees' marker (`team_compiler/blue_port.md` "Referee export processing"): the refs
//! export's `ref_marker.dds` converted into the referees' Common output, and a flat square
//! under each referee written as the referees' reserved collar, its texture pointed at that
//! texture. On Fox the square is the bundled marker FMDL, written as the collar's own model; on
//! pre-Fox it is the referee template tree's prop model, written as the collar's referee model
//! with a reduced `.mtl` beside it, and the bundled empty collar as the collar's own model,
//! which the referee needs to exist.

use aesthetics_export::FileDescriptor;
use anyhow::Context;
use fmdl::ops::paths::rewrite_texture_paths;
use fmdl::{FmdlError, FmdlFile};
use pes_model::format::mtl::MaterialSet;
use pes_version::Engine;

use super::{CompileContext, Entry, TaskFailure, TaskFiles, texture};
use crate::paths::{self, REFEREE_MARKER_COLLAR, REFEREE_TEAM_ID};
use crate::templates::{REFEREE_PROP_MODEL, REFEREE_PROP_MTL, Templates};

/// The base texture's file name in the bundled marker model, the one the compile repoints.
const BUNDLED_TEXTURE: &str = "cup_logo.dds";

/// The stem the marker texture goes out under in the referees' Common output.
const MARKER_STEM: &str = "ref_marker";

/// The one material the pre-Fox prop model binds.
const PROP_MATERIAL: &str = "judge_incom";

/// The sampler of `PROP_MATERIAL` naming the texture drawn on the square.
const PROP_SAMPLER: &str = "DiffuseMap";

/// The marker `marker`, from its bytes in `files`, as the entries of `ctx`'s engine: first its
/// texture converted as a Common texture is, under `ref_marker` in the referees' Common
/// output; then, on Fox, the bundled marker model with its base texture pointed at it, as the
/// referees' collar; on pre-Fox, the template tree's prop model as the collar's referee model,
/// its `.mtl` beside it (`marker_materials`), and the bundled empty collar as the collar's own
/// model. A failed conversion fails the task with its texture code, so nothing goes out and
/// the referees wear no collar naming a texture the CPK lacks; so does a prop `.mtl` that does
/// not read or lacks its material.
pub(super) fn referee_marker(
    marker: &FileDescriptor,
    ctx: &CompileContext,
    files: &mut TaskFiles,
) -> Result<Vec<Entry>, TaskFailure> {
    let texture = texture::common_texture(marker, ctx, files)?;
    let engine = ctx.version.engine();
    let mut entries = vec![(
        paths::common_texture(engine, REFEREE_TEAM_ID, MARKER_STEM),
        texture,
    )];
    match engine {
        Engine::Fox => entries.push((
            paths::collar(Engine::Fox, REFEREE_MARKER_COLLAR),
            marker_model(ctx.templates.referee_marker())?,
        )),
        Engine::PreFox => entries.extend(pre_fox_marker(&ctx.templates)?),
    }
    Ok(entries)
}

/// The pre-Fox marker's model entries from `templates`: the pre-Fox tree's prop model as it is
/// at the marker collar's referee model path, its `.mtl` reduced (`marker_materials`) beside
/// it, and the empty collar at the collar's own path.
fn pre_fox_marker(templates: &Templates) -> anyhow::Result<[Entry; 3]> {
    let model = templates.referee_tree_file(Engine::PreFox, REFEREE_PROP_MODEL);
    let materials = templates.referee_tree_file(Engine::PreFox, REFEREE_PROP_MTL);
    Ok([
        (paths::referee_collar(REFEREE_MARKER_COLLAR), model.to_vec()),
        (
            paths::referee_collar_mtl(REFEREE_MARKER_COLLAR),
            marker_materials(materials)?,
        ),
        (
            paths::collar(Engine::PreFox, REFEREE_MARKER_COLLAR),
            templates.collar_empty().to_vec(),
        ),
    ])
}

/// The prop `.mtl` `bytes` holding `judge_incom` alone, the one material the prop model binds,
/// its diffuse map pointed at the marker texture in the referees' Common output. The other
/// materials name textures the refs CPK does not carry, so they are left out. A set that does
/// not read, or that has no `judge_incom`, is the error naming the file.
fn marker_materials(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut set = MaterialSet::read(bytes)
        .with_context(|| format!("{REFEREE_PROP_MTL}: not a material set"))?;
    set.materials
        .retain(|material| material.name == PROP_MATERIAL);
    anyhow::ensure!(
        !set.materials.is_empty(),
        "{REFEREE_PROP_MTL}: no material {PROP_MATERIAL}"
    );
    pes_model::ops::paths::rewrite_texture_paths(&mut set, |path| {
        if path.sampler == PROP_SAMPLER {
            path.directory = paths::common_texture_directory(Engine::PreFox, REFEREE_TEAM_ID);
            path.file_name = format!("{MARKER_STEM}.dds");
        }
    });
    Ok(set.write())
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
    use pes_model::format::mtl::MaterialEntry;

    use super::*;

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

    #[test]
    fn the_pre_fox_prop_model_is_one_square_weighted_to_no_bone() {
        let templates = Templates::embedded();
        let bytes = templates.referee_tree_file(Engine::PreFox, REFEREE_PROP_MODEL);
        let file = pes_model::format::PreFoxModel::read(bytes).unwrap();
        let model = pes_model::model::Model::from_file(&file).unwrap();

        assert_eq!(model.materials, ["judge_incom"]);
        let [mesh] = model.meshes.as_slice() else {
            panic!("{} meshes", model.meshes.len());
        };
        assert_eq!(mesh.vertices.positions.len(), 4);
        // The pre-Fox form of static painting: the model has no bone at all and its vertices
        // no weights, so no bone of the body skeleton moves the square.
        assert!(model.bones.is_empty(), "{:?}", model.bones);
        assert_eq!(mesh.bone_group, Vec::<usize>::new());
        assert_eq!(mesh.vertices.bone_indices, None);
        assert_eq!(mesh.vertices.bone_weights, None);
    }

    #[test]
    fn the_pre_fox_marker_materials_are_judge_incom_alone_naming_the_marker_texture() {
        let templates = Templates::embedded();
        let template = templates.referee_tree_file(Engine::PreFox, REFEREE_PROP_MTL);
        let mut expected = MaterialSet::read(template).unwrap();
        assert_eq!(
            expected
                .materials
                .iter()
                .map(|material| material.name.as_str())
                .collect::<Vec<_>>(),
            [
                "judge_incom",
                "judge_watch",
                "judge_pen",
                "judge_whistle",
                "referee_flag_mat"
            ]
        );
        expected.materials.truncate(1);
        let [MaterialEntry::Sampler(sampler), ..] = expected.materials[0].entries.as_mut_slice()
        else {
            panic!("judge_incom's first entry is its sampler");
        };
        assert_eq!(
            (sampler.name.as_str(), sampler.path.as_str()),
            ("DiffuseMap", "./incom_bsm.dds")
        );
        "model/character/uniform/common/999/ref_marker.dds".clone_into(&mut sampler.path);

        let written = marker_materials(template).unwrap();

        assert_eq!(MaterialSet::read(&written).unwrap(), expected);
    }

    #[test]
    fn a_prop_mtl_that_does_not_read_or_lacks_judge_incom_is_the_error_naming_it() {
        let error = marker_materials(b"not a material set").unwrap_err();
        assert!(
            format!("{error:#}").starts_with(
                "common/character1/model/character/parts/referee/referee_prop.mtl: not a \
                 material set: "
            ),
            "{error:#}"
        );

        let other = b"<materialset>\n  <material name=\"judge_watch\" shader=\"Basic_CN\">\n  \
                      </material>\n</materialset>\n";
        let error = marker_materials(other).unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            "common/character1/model/character/parts/referee/referee_prop.mtl: no material \
             judge_incom"
        );
    }
}
