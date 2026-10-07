//! A pre-Fox shared boots or gloves output (`team_compiler/pipeline.md` "3. Per-model-folder
//! parallel steps", step 7): a `Boots/` or `Gloves/` folder a player links, written once under
//! its shared id as loose files in the shape of the game's own boots and glove folders. A boots
//! folder's one model as `boots.model` and the `.mtl` it uses as `boots.mtl`, the names the
//! game loads; a gloves folder's models and `.mtl` files under their own names lowercased,
//! listed in a generated `glove.xml`. The folder's textures sit beside them
//! (`TextureHome::SharedOutput`), the `.mtl` paths naming them `./<stem>.dds`. `materialize`
//! writes the files into the output's folder.

use std::collections::BTreeMap;

use pes_version::Engine;

use super::materialize::PackageFiles;
use super::prefox_face::{insert, rewritten_materials};
use super::{TaskFailure, TaskFiles, take};
use crate::face_xml::{XmlEntry, glove_xml, ratio};
use crate::mtl_search::mtl_for;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem};

/// What the deep pass guarantees of every `.model` a task reads on pre-Fox.
const MTL_FOUND: &str = "the deep pass drops a folder holding a `.model` no `.mtl` is found for \
                         (`model_material_undefined`)";

/// The files of the shared `folder`'s pre-Fox `package`, boots or gloves, compiled from its
/// files' bytes in `files` for team `team_id`, by their names in the output's folder. Boots:
/// the folder's one model as `boots.model`, its bytes as they are, and the `.mtl` it uses
/// (`mtl_for`) as `boots.mtl`; the folder's other `.mtl` files are not packed. Gloves: each
/// model under its file name lowercased, each `.mtl` likewise, and a `glove.xml` listing the
/// models in the order of their export paths, case-folded, each typed by its role and naming
/// the `.mtl` its search finds. Every `.mtl` packed has each texture path naming one of the
/// folder's textures by its stem pointed beside it (`rewritten_materials`). Two files packing
/// under one name fail the task. Only a shared folder reaches here: a pre-Fox player's boots
/// and gloves models are parts of his face.
pub(super) fn package(
    folder: &ModelFolder,
    package: ModelPackage,
    team_id: u16,
    files: &mut TaskFiles,
) -> Result<PackageFiles, TaskFailure> {
    let mut models = Vec::new();
    let mut materials = Vec::new();
    // The folder's texture stems, folded, each with its stem as the folder spells it: the
    // name its converted DDS has beside the models (`folder_textures`).
    let mut textures: BTreeMap<String, String> = BTreeMap::new();
    // A shared folder is its own one source: it combines no other folder.
    for (_, _, source_roles) in folder.roles() {
        for (file, role) in source_roles {
            match role {
                PlayerFile::PreFoxModel { xml_type } => models.push((file, xml_type)),
                PlayerFile::Material => materials.push(file),
                PlayerFile::Texture(stem, _) => {
                    textures.insert(vtree::fold_name(&stem), stem);
                }
                // The subset gate names any other file of a shared boots or gloves folder
                // (`pre_fox_shared_not_compiled`).
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::Packed { .. }
                | PlayerFile::FaceDiffXml
                | PlayerFile::UnusedFaceFile
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::CommonTexture(_) => {}
            }
        }
    }
    // By export path, case-folded, then as spelled, so a recompile lists them alike.
    models.sort_by_cached_key(|(file, _)| (file.path.fold_key(), file.path.as_str().to_owned()));
    let home = folder.textures.directory(Engine::PreFox, team_id);
    let mut contents = PackageFiles::new();
    match package {
        ModelPackage::Boots => {
            let [(model, _)] = models.as_slice() else {
                panic!(
                    "the subset gate names a shared boots folder holding no model or several \
                     (`pre_fox_shared_not_compiled`)"
                );
            };
            let material = mtl_for(&model.path, &folder.path, &folder.files).expect(MTL_FOUND);
            insert(
                &mut contents,
                package,
                "boots.model".to_owned(),
                take(files, model),
            )?;
            let bytes = rewritten_materials(material, &take(files, material), &textures, &home)?;
            insert(&mut contents, package, "boots.mtl".to_owned(), bytes)?;
        }
        ModelPackage::Gloves => {
            let mut entries = Vec::new();
            for (model, xml_type) in models {
                let name = model.path.name();
                let material = mtl_for(&model.path, &folder.path, &folder.files).expect(MTL_FOUND);
                let packed = name.to_ascii_lowercase();
                entries.push(XmlEntry {
                    xml_type,
                    path: format!("./{packed}"),
                    material: format!("./{}", material.path.name().to_ascii_lowercase()),
                    ratio: ratio(file_stem(name)).map(str::to_owned),
                });
                insert(&mut contents, package, packed, take(files, model))?;
            }
            for file in materials {
                let bytes = rewritten_materials(file, &take(files, file), &textures, &home)?;
                let name = file.path.name().to_ascii_lowercase();
                insert(&mut contents, package, name, bytes)?;
            }
            insert(
                &mut contents,
                package,
                "glove.xml".to_owned(),
                glove_xml(&entries),
            )?;
        }
        ModelPackage::Face => unreachable!(
            "a shared face has no output of its own: it is copied into each linking player's \
             face (`prefox_face::face`)"
        ),
    }
    Ok(contents)
}
