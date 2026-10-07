//! A pre-Fox boots or gloves output (`team_compiler/pipeline.md` "3. Per-model-folder parallel
//! steps", step 7): loose files in the shape of the game's own boots and glove folders, written
//! under an id. A `Boots/` or `Gloves/` folder a player links plainly is written once under its
//! shared id; an `ingame_face` player's own boots and gloves, his parts of each and the models
//! of a folder of their kind his link combines, under his exclusive id (`player_folders.md`
//! "`ingame_face` marker"). Boots are one model as `boots.model` and the `.mtl` it uses as
//! `boots.mtl`, the names the game loads, several models merged into one
//! (`pes_model::ops::merge`); gloves are each model and the `.mtl` files they use under their
//! own names lowercased, listed in a generated `glove.xml`, unmerged. A shared folder's
//! textures sit beside them (`TextureHome::SharedOutput`), the `.mtl` paths naming them
//! `./<stem>.dds`; a player's are in his common folder (`TextureHome::PlayerCommon`), named as
//! his face's `.mtl` files name them. `materialize` writes the files into the output's folder.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::Arc;

use aesthetics_export::FileDescriptor;
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;
use pes_model::model::Model;
use pes_model::ops::merge::{MergeError, merge};
use pes_version::Engine;
use pipeline::MemoryBudget;
use studio_core::Disposition;
use vtree::ScopePath;

use super::materialize::PackageFiles;
use super::prefox_face::{insert, linked_texture_stem, read_materials, rewritten_materials};
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::face_xml::{XmlEntry, glove_xml, ratio};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem};

/// What the deep pass guarantees of every `.model` a task reads on pre-Fox.
const MTL_FOUND: &str = "the deep pass drops a folder holding a `.model` no `.mtl` is found for \
                         (`model_material_undefined`)";

/// One `.model` of the output, with the source folder it was found in: its `.mtl` is searched
/// for there (`mtl_for`), a combined folder's never in the player's folder.
struct SourceModel<'a> {
    /// The model file.
    file: &'a FileDescriptor,
    /// The type a `glove.xml` lists it under, its role's (`PlayerFile::PreFoxModel`,
    /// `PlayerFile::PreFoxPart`); the boots list no model.
    xml_type: String,
    /// Its source folder's export path.
    source_path: &'a ScopePath,
    /// Its source folder's files.
    source_files: &'a [FileDescriptor],
}

/// One part of a boots output: its `.model`'s export path, which names it in an error, the
/// model's bytes, and the material set it uses, its texture paths already pointed where the
/// output's textures are.
struct BootsPart<'a> {
    /// The model's export path.
    path: &'a ScopePath,
    /// The model's bytes.
    model: Vec<u8>,
    /// The material set the model uses.
    materials: &'a MaterialSet,
}

/// The files of `folder`'s pre-Fox `package`, boots or gloves, compiled from its files' bytes in
/// `files` for team `team_id`, by their names in the output's folder. The models of the
/// package are a shared folder's, or an `ingame_face` player's parts with a combined folder's
/// of the package's kind. Boots: the models as `boots.model` and the `.mtl` each uses
/// (`mtl_for`) as `boots.mtl` (`boots_files`), the merge of several noted in `findings` as
/// `model_merged`. Gloves: each model under its file name lowercased, the `.mtl` each uses
/// likewise, once however many use it, and a `glove.xml` listing the models in the order of
/// their export paths, case-folded, each typed by its role and naming its `.mtl`; a player's
/// model or `.mtl` replaces a combined folder's packing under its name (`without_replaced`).
/// A `.mtl` no model uses is not packed. Every `.mtl` packed has each texture path naming one
/// of the folder's textures by its stem pointed at the folder's texture home, and one naming a
/// stem a texture link of the folder stands for at that texture in the team's Common output,
/// as the face's are (`rewritten_materials`). Two files packing under one name otherwise fail
/// the task. A merge is charged to `ctx`'s memory budget.
pub(super) fn package(
    folder: &ModelFolder,
    package: ModelPackage,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<PackageFiles, TaskFailure> {
    let mut models = Vec::new();
    // The folder's texture stems, folded, each with its stem as the folder spells it: the
    // name its converted DDS has in the texture home (`folder_textures`).
    let mut textures: BTreeMap<String, String> = BTreeMap::new();
    // The stems the folder's texture links stand for, folded, each with the stem of the
    // `Common/` texture the link names. A shared folder holds no link.
    let mut linked: BTreeMap<String, String> = BTreeMap::new();
    // `roles()` yields the folder's own files, then each combined folder's in `combined`'s
    // order, so each source's roles pair with its own file list here.
    let source_files = iter::once(&folder.files).chain(
        folder
            .combined
            .iter()
            .map(|combined| &combined.folder.files),
    );
    for ((source_package, source_path, source_roles), source_files) in
        folder.roles().into_iter().zip(source_files)
    {
        for (file, role) in source_roles {
            let source_model = |xml_type| SourceModel {
                file,
                xml_type,
                source_path,
                source_files,
            };
            match role {
                // A shared folder's model is a part of the package the folder feeds, a
                // combined one's of the player's.
                PlayerFile::PreFoxModel { xml_type } if source_package == package => {
                    models.push(source_model(xml_type));
                }
                PlayerFile::PreFoxPart {
                    package: owner,
                    xml_type,
                } if owner == package => models.push(source_model(xml_type)),
                PlayerFile::Texture(stem, _) => {
                    textures.insert(vtree::fold_name(&stem), stem);
                }
                PlayerFile::CommonTexture(stem) => {
                    linked.insert(vtree::fold_name(&stem), linked_texture_stem(folder, file));
                }
                // A `.mtl` is packed when a model uses it (`mtl_for`), below.
                PlayerFile::Material => {}
                // A model of another package, and the roles planning gives none of the
                // package's files: the subset gate names a shared boots or gloves folder's
                // other files (`pre_fox_shared_not_compiled`), and a link to a Common model or
                // `.mtl` beside `ingame_face` (`compiled_under_ingame_face`).
                PlayerFile::PreFoxModel { .. }
                | PlayerFile::PreFoxPart { .. }
                | PlayerFile::Model { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::Packed { .. }
                | PlayerFile::FaceDiffXml
                | PlayerFile::UnusedFaceFile
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::PreFoxCommonModel { .. }
                | PlayerFile::CommonMaterial => {}
            }
        }
    }
    let home = folder.textures.directory(Engine::PreFox, team_id);
    let common_directory = paths::common_texture_directory(Engine::PreFox, team_id);
    // A stem the folder holds is its own, before one a texture link stands for, as in the
    // face (`prefox_face::face`).
    let places = [
        (&textures, home.as_str()),
        (&linked, common_directory.as_str()),
    ];
    let mut contents = PackageFiles::new();
    match package {
        ModelPackage::Boots => {
            // The order the Fox merge takes the same parts in (`model::package`): by file
            // name, folded as the file system folds it, then by export path.
            models.sort_by_cached_key(|model| {
                (
                    vtree::fold_name(model.file.path.name()),
                    model.file.path.as_str().to_owned(),
                )
            });
            let used: Vec<&FileDescriptor> = models
                .iter()
                .map(|model| {
                    mtl_for(
                        &model.file.path,
                        model.source_path,
                        model.source_files,
                        &folder.common_files,
                    )
                    .expect(MTL_FOUND)
                })
                .collect();
            // Each `.mtl` is read once, however many parts use it.
            let mut sets: BTreeMap<&ScopePath, MaterialSet> = BTreeMap::new();
            for material in &used {
                if !sets.contains_key(&material.path) {
                    let set = read_materials(material, &take(files, material), &places)?;
                    sets.insert(&material.path, set);
                }
            }
            let parts = models
                .iter()
                .zip(&used)
                .map(|(model, material)| BootsPart {
                    path: &model.file.path,
                    model: take(files, model.file),
                    materials: &sets[&material.path],
                })
                .collect();
            let merged = models.len() > 1;
            let (model, materials) = boots_files(&ctx.budget, parts)?;
            if merged {
                findings.push((
                    Code::ModelMerged,
                    Disposition::Keep,
                    vec![("model", "boots.model".to_owned())],
                ));
            }
            insert(&mut contents, package, "boots.model".to_owned(), model)?;
            insert(&mut contents, package, "boots.mtl".to_owned(), materials)?;
        }
        ModelPackage::Gloves => {
            let own = |source_path: &ScopePath| source_path == &folder.path;
            let mut models = without_replaced(models, |model| {
                (
                    model.file.path.name().to_ascii_lowercase(),
                    own(model.source_path),
                )
            });
            // By export path, case-folded, then as spelled, so a recompile lists them alike.
            models.sort_by_cached_key(|model| {
                (
                    model.file.path.fold_key(),
                    model.file.path.as_str().to_owned(),
                )
            });
            let mut entries = Vec::new();
            // The `.mtl` files the models use, each once, with whether the player's own folder
            // holds it: a model's search finds one in its own source folder.
            let mut used: Vec<(&FileDescriptor, bool)> = Vec::new();
            for model in models {
                let name = model.file.path.name();
                let material = mtl_for(
                    &model.file.path,
                    model.source_path,
                    model.source_files,
                    &folder.common_files,
                )
                .expect(MTL_FOUND);
                if !used.iter().any(|(file, _)| file.path == material.path) {
                    used.push((material, own(model.source_path)));
                }
                let packed = name.to_ascii_lowercase();
                entries.push(XmlEntry {
                    xml_type: model.xml_type,
                    path: format!("./{packed}"),
                    material: format!("./{}", material.path.name().to_ascii_lowercase()),
                    ratio: ratio(file_stem(name)).map(str::to_owned),
                });
                insert(&mut contents, package, packed, take(files, model.file))?;
            }
            let used = without_replaced(used, |(file, own)| {
                (file.path.name().to_ascii_lowercase(), *own)
            });
            for (file, _) in used {
                let bytes = rewritten_materials(file, &take(files, file), &places)?;
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

/// `items`, gloves files of a player or a shared folder, less each of a folder the player
/// combines that packs under a name one of his own packs under: his file replaces it, as a
/// copy of his over the linked folder's would (`player_folders.md` "`ingame_face` with shared
/// links"). `packed` gives an item's packed name and whether the player's own folder holds it;
/// a shared folder's items are all its own, so none is left out.
fn without_replaced<T>(items: Vec<T>, packed: impl Fn(&T) -> (String, bool)) -> Vec<T> {
    let own_names: BTreeSet<String> = items
        .iter()
        .map(&packed)
        .filter_map(|(name, own)| own.then_some(name))
        .collect();
    items
        .into_iter()
        .filter(|item| {
            let (name, own) = packed(item);
            own || !own_names.contains(&name)
        })
        .collect()
}

/// The `boots.model` and `boots.mtl` bytes of `parts`, given in merge order: one part's model
/// as it is beside its material set; several merged into one model and one material set
/// (`pes_model::ops::merge`), a disagreement between them failing the task with its own code
/// (`From<MergeError>`). A model that does not read fails the task naming it.
fn boots_files(
    budget: &Arc<MemoryBudget>,
    mut parts: Vec<BootsPart>,
) -> Result<(Vec<u8>, Vec<u8>), TaskFailure> {
    if parts.len() == 1 {
        let part = parts.swap_remove(0);
        return Ok((part.model, part.materials.write()));
    }
    // The parsed parts, their merge and its written model, charged at the parts' source size,
    // an estimate of each form, as the Fox merge charges them (`model::package`).
    let _parts_charge = budget.charge(parts.iter().map(|part| part.model.len()).sum());
    let models = parts
        .iter()
        .map(|part| {
            PreFoxModel::read(&part.model)
                .and_then(|file| Model::from_file(&file))
                .map_err(|error| anyhow::anyhow!("{}: {error}", part.path.as_str()).into())
        })
        .collect::<Result<Vec<Model>, TaskFailure>>()?;
    let pairs: Vec<(&Model, &MaterialSet)> = models
        .iter()
        .zip(parts.iter().map(|part| part.materials))
        .collect();
    let (model, materials) = merge(&pairs)?;
    let bytes = model
        .to_file()
        .and_then(|file| file.write())
        .map_err(|error| anyhow::anyhow!("boots.model: {error}"))?;
    Ok((bytes, materials.write()))
}

impl From<MergeError> for TaskFailure {
    fn from(error: MergeError) -> TaskFailure {
        match error {
            MergeError::MaterialConflict { name } => TaskFailure {
                code: Code::MergeMaterialConflict,
                context: vec![("material", name)],
            },
            MergeError::SkeletonConflict { name } => TaskFailure {
                code: Code::SklMergeConflict,
                context: vec![("bone", name)],
            },
            MergeError::FlagsConflict => TaskFailure {
                code: Code::ModelMergeFlagsConflict,
                context: Vec::new(),
            },
            // A part that fails validation: not a disagreement between the parts a member
            // resolves, and the deep pass has dropped a folder whose `.model` does not read.
            MergeError::Other(_) => TaskFailure::from(anyhow::Error::from(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use pes_model::format::ModelError;

    use super::*;

    /// The bytes of `pes_model`'s fixture `name`.
    fn fixture(name: &str) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../libs/pes_model/tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    /// The card-head template's face model: one bone, one mesh, one material, `card`.
    fn card() -> Model {
        Model::from_file(&PreFoxModel::read(&fixture("cardhead_face_high.model")).unwrap()).unwrap()
    }

    /// The card-head template's material set, defining `card`.
    fn card_set() -> MaterialSet {
        MaterialSet::read(&fixture("cardhead_materials.mtl")).unwrap()
    }

    fn written(model: &Model) -> Vec<u8> {
        model.to_file().unwrap().write().unwrap()
    }

    /// `boots_files` over several `parts`, each a model and the material set it uses, in this
    /// order, asserting the merge was charged at the parts' written size.
    fn boots_of(parts: &[(&Model, &MaterialSet)]) -> Result<(Vec<u8>, Vec<u8>), TaskFailure> {
        let paths: Vec<ScopePath> = (0..parts.len())
            .map(|index| ScopePath::new(&format!("Boots/Twin/part{index}_boots.model")).unwrap())
            .collect();
        let parts: Vec<BootsPart> = parts
            .iter()
            .zip(&paths)
            .map(|((model, materials), path)| BootsPart {
                path,
                model: written(model),
                materials,
            })
            .collect();
        let size = parts.iter().map(|part| part.model.len()).sum();
        let budget = MemoryBudget::new(usize::MAX);
        let result = boots_files(&budget, parts);
        assert_eq!(budget.peak(), size);
        result
    }

    /// The card-head model and its material set with the material renamed `name`.
    fn card_named(name: &str) -> (Model, MaterialSet) {
        let mut model = card();
        model.materials[0] = name.to_owned();
        let mut set = card_set();
        set.materials[0].name = name.to_owned();
        (model, set)
    }

    /// The files of `result`, which must have succeeded.
    fn written_boots(result: Result<(Vec<u8>, Vec<u8>), TaskFailure>) -> (Vec<u8>, Vec<u8>) {
        match result {
            Ok(files) => files,
            Err(failure) => panic!("{:?} {:?}", failure.code, failure.context),
        }
    }

    #[test]
    fn one_boots_part_is_written_as_it_is_and_several_are_merged_in_order() {
        let source = fixture("cardhead_face_high.model");
        let (studs, studs_set) = card_named("studs");
        let set = card_set();
        let path = ScopePath::new("Boots/Twin/boots.model").unwrap();
        let one = vec![BootsPart {
            path: &path,
            model: source.clone(),
            materials: &set,
        }];
        // One part is not parsed, so nothing is charged.
        let budget = MemoryBudget::new(usize::MAX);
        let (model, materials) = written_boots(boots_files(&budget, one));
        assert_eq!(budget.peak(), 0);
        assert_eq!(model, source);
        assert_eq!(materials, set.write());

        let (model, materials) = written_boots(boots_of(&[(&card(), &set), (&studs, &studs_set)]));
        let merged = Model::from_file(&PreFoxModel::read(&model).unwrap()).unwrap();
        assert_eq!(merged.meshes.len(), 2);
        assert_eq!(merged.materials, ["card", "studs"]);
        let names: Vec<String> = MaterialSet::read(&materials)
            .unwrap()
            .materials
            .into_iter()
            .map(|material| material.name)
            .collect();
        assert_eq!(names, ["card", "studs"]);
    }

    #[test]
    fn a_player_s_own_gloves_file_replaces_a_combined_folder_s_of_its_packed_name() {
        // Each a packed name and whether the player's own folder holds it.
        let items = vec![
            ("glove_l.mtl", true),
            ("x_glover.mtl", true),
            ("glove_l.mtl", false),
            ("glove_r.mtl", false),
        ];
        let kept = without_replaced(items, |(name, own)| ((*name).to_owned(), *own));
        assert_eq!(
            kept,
            [
                ("glove_l.mtl", true),
                ("x_glover.mtl", true),
                ("glove_r.mtl", false)
            ]
        );
        // A shared folder's are all its own: two of one name are both kept, for `insert` to
        // refuse.
        let shared = vec![("glove_l.mtl", true), ("glove_l.mtl", true)];
        assert_eq!(
            without_replaced(shared.clone(), |(name, own)| ((*name).to_owned(), *own)),
            shared
        );
    }

    /// The code and context of `failure`.
    fn failure_of(failure: TaskFailure) -> (Code, Vec<(&'static str, String)>) {
        (failure.code, failure.context)
    }

    #[test]
    fn each_merge_conflict_fails_with_its_own_code_and_context() {
        let material = MergeError::MaterialConflict {
            name: "skin".to_owned(),
        };
        assert_eq!(
            failure_of(material.into()),
            (
                Code::MergeMaterialConflict,
                vec![("material", "skin".to_owned())]
            )
        );
        let bone = MergeError::SkeletonConflict {
            name: "sk_head".to_owned(),
        };
        assert_eq!(
            failure_of(bone.into()),
            (Code::SklMergeConflict, vec![("bone", "sk_head".to_owned())])
        );
        assert_eq!(
            failure_of(MergeError::FlagsConflict.into()),
            (Code::ModelMergeFlagsConflict, Vec::new())
        );
        assert_eq!(
            failure_of(MergeError::Other(ModelError::Truncated).into()),
            (
                Code::FolderPackFailed,
                vec![("error", "model is truncated".to_owned())]
            )
        );
        // Parts whose headers carry different flags.
        let mut flagged = card();
        flagged.flags = 4;
        let set = card_set();
        let Err(failure) = boots_of(&[(&card(), &set), (&flagged, &set)]) else {
            panic!("parts with different flags merged");
        };
        assert_eq!(
            failure_of(failure),
            (Code::ModelMergeFlagsConflict, Vec::new())
        );
    }
}
