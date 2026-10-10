//! The deep pass's pairing of each model of a model folder with the `.mtl` it binds its
//! materials from (`team_compiler/messages.md` "XML/MTL content checks"): the `.mtl` the
//! folder's own `face.xml` entry names, else the one its search finds (`mtl_search`), and the
//! `model_material_undefined` finding that comparing the two gives. `mod.rs`'s
//! `folder_findings` drives it, once per model folder, its texture lookup reading which
//! materials a pairing makes mesh-used; `content_findings` asks it, once per export, which
//! `Common/` `.mtl` files a player folder's search may read.

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, ModelFormat,
    ValidatedAestheticsExport, classify, common_link_target,
};
use pes_version::Engine;
use vtree::ScopePath;

use super::model::MaterialRead;
use super::{KeptCommon, MODEL_MATERIAL_UNDEFINED, named_on_folder};
use crate::mtl_search::mtl_for;
use crate::plan::roles::{
    FolderModels, PlayerFile, is_common_file, player_file, selected_common_model,
};

/// A `.model` of a model folder paired with the `.mtl` it binds its materials from: what
/// `model_material_undefined` compares, what makes a `.mtl` material mesh-used for the
/// texture lookup, and on Fox what makes a `.mtl` read at all.
pub(super) struct Pairing<'a> {
    /// The folder's file the pairing is about, which a finding names: a `.model`, or a typed
    /// `.common` link to a `Common/` one.
    pub(super) file: &'a FileDescriptor,
    /// The model whose materials count: the file itself, or a link's kept `Common/` model;
    /// `None` when the pass dropped that model.
    model: Option<&'a ScopePath>,
    /// The `.mtl`: the one the folder's `face.xml` entry names, or the search's (`mtl_for`);
    /// `None` when the search finds none.
    pub(super) mtl: Option<&'a FileDescriptor>,
}

/// The pairings of the models among `files`, those of the model folder at `folder` whose
/// models are `models`, read for a target of `engine`: with the folder's own `face.xml`, the
/// models it lists with the `.mtl` each entry names (`listed`, empty when an xml drops the
/// folder); without, on pre-Fox, each `.model` the face or the boots and gloves read
/// (`PlayerFile::PreFoxModel`, `PlayerFile::PreFoxPart`) and each `.common` link the face types
/// or, under `ingame_face`, his boots or gloves take, loading a Common `.model`
/// (`selected_common_model`; one loading a Common FMDL pairs none, the FMDL's
/// conversion writing its material set) with the `.mtl` its search finds among `files` and
/// `common`'s; on Fox each `.model` with a role (`PlayerFile::Model`: no FMDL of its stem
/// beats it) and each `.common` link with a role loading a Common `.model` (one loading a
/// Common FMDL pairs none, the FMDL carrying its materials) the same way. A shared folder's
/// search sees no `Common/` file: its `.common` links resolve nothing (`FolderModels::shared`),
/// as its tasks' search finds none, so a `.model` whose only `.mtl` is such a link has none.
pub(super) fn pairings<'a>(
    folder: &'a ScopePath,
    files: &'a [FileDescriptor],
    models: &FolderModels,
    engine: Engine,
    listed: Option<Vec<(&'a FileDescriptor, &'a FileDescriptor)>>,
    common: &'a KeptCommon,
) -> Vec<Pairing<'a>> {
    if let Some(listed) = listed {
        return listed
            .into_iter()
            .map(|(model, mtl)| Pairing {
                file: model,
                model: Some(&model.path),
                mtl: Some(mtl),
            })
            .collect();
    }
    let link_targets: &[FileDescriptor] = if models.is_shared() {
        &[]
    } else {
        &common.files
    };
    files
        .iter()
        .filter(|file| pairs_with_mtl(folder, file, models, &common.files))
        .map(|file| {
            let model = match common_link_target(&file.path, folder) {
                Some(linked) => {
                    selected_common_model(&common.files, &linked, engine).map(|model| &model.path)
                }
                None => Some(&file.path),
            };
            Pairing {
                file,
                model,
                mtl: mtl_for(&file.path, folder, files, link_targets),
            }
        })
        .collect()
}

/// Whether `file`, of the model folder at `folder` whose models are `models`, is a model a
/// task converts or writes with the `.mtl` its search finds (`pairings`), its `.common` link
/// resolved among `common`, `Common/`'s files (`links_common_fmdl`); for the target engine
/// `models` were read for.
fn pairs_with_mtl(
    folder: &ScopePath,
    file: &FileDescriptor,
    models: &FolderModels,
    common: &[FileDescriptor],
) -> bool {
    let engine = models.engine();
    let role = player_file(folder, file, models);
    match engine {
        // On Fox a selected `.model` is converted with the `.mtl` its search finds, and so is
        // a linked Common `.model`. One an FMDL of its stem beats has no role and is read by
        // nothing: dropping the folder for it would lose a working FMDL.
        Engine::Fox => {
            (file.kind == FileKind::Model(ModelFormat::PesModel)
                && matches!(role, Some(PlayerFile::Model { .. })))
                || (matches!(role, Some(PlayerFile::CommonModel { .. }))
                    && !links_common_fmdl(folder, file, common, engine))
        }
        // A model link is a part of the face's `face.xml` (`PreFoxCommonModel`), or in a
        // folder holding `ingame_face` a part of his boots or gloves (`PreFoxPart`), whose
        // Common `.model` is written with the `.mtl` its search finds. A link loading a
        // Common FMDL pairs none: its material set is its conversion's. A `.model` with no
        // role, or a per-kit variant left out, is read by nothing.
        Engine::PreFox => {
            let model_link = matches!(role, Some(PlayerFile::PreFoxCommonModel { .. }))
                || (file.kind == FileKind::CommonLink
                    && matches!(role, Some(PlayerFile::PreFoxPart { .. })));
            (file.kind == FileKind::Model(ModelFormat::PesModel)
                && matches!(
                    role,
                    Some(PlayerFile::PreFoxModel { .. } | PlayerFile::PreFoxPart { .. })
                ))
                || (model_link && !links_common_fmdl(folder, file, common, engine))
        }
    }
}

/// Whether the `.common` model link `file` of the model folder at `folder` loads a Common FMDL
/// on a target of `engine`, which takes no `.mtl`: on Fox it carries its materials, on pre-Fox
/// the Common models task converts it with the material set its conversion writes. The Common
/// model it loads, at the link's path (`common_link_target`), is among `common`
/// (`selected_common_model`), or, when that list lacks the file (the kept files, after the pass
/// dropped it), the model its name links.
fn links_common_fmdl(
    folder: &ScopePath,
    file: &FileDescriptor,
    common: &[FileDescriptor],
    engine: Engine,
) -> bool {
    let Some(linked) = common_link_target(&file.path, folder) else {
        return false;
    };
    let kind = selected_common_model(common, &linked, engine)
        .map_or_else(|| classify(&linked), |model| model.kind);
    kind == FileKind::Model(ModelFormat::Fmdl)
}

/// The export paths of the `.mtl` files of `export`'s `Common/` that a search of a player
/// folder may read on PES 2018 to 2021, `player_models` being each player folder's models
/// (`part_source_models`): every one of them when the search of a selected `.model` or of a
/// Common `.model` link (`pairs_with_mtl`, `mtl_for`) finds a `Common/` file, none otherwise.
/// A subfolder's `.mtl` is in the set too, which the caller reads only for a file it reads
/// (`is_read_common_file`: on Fox a subfolder's a link reaches).
/// Fox has no Common models task, so no other reads one. A shared folder's search sees no
/// `Common/` file (`pairings`), so it adds none.
pub(super) fn searched_common_mtls<'a>(
    export: &'a ValidatedAestheticsExport,
    player_models: &[&FolderModels],
) -> BTreeSet<&'a ScopePath> {
    let lands_in_common = export
        .players
        .iter()
        .zip(player_models)
        .any(|(player, models)| {
            let folder = &player.path;
            player.files.iter().any(|file| {
                pairs_with_mtl(folder, file, models, &export.common)
                    && mtl_for(&file.path, folder, &player.files, &export.common)
                        .is_some_and(|mtl| is_common_file(&mtl.path))
            })
        });
    if !lands_in_common {
        return BTreeSet::new();
    }
    // Every one, not only the file the search finds here: the search runs again in the
    // folders' pass over the kept files, and its chain in `Common/` ends with any `.mtl`, so
    // when this pass drops the one found, any other may be the one that search lands on.
    export
        .common
        .iter()
        .filter(|file| file.kind == FileKind::Mtl)
        .map(|file| &file.path)
        .collect()
}

/// The names of the materials the meshes of the models `pairings` pair with the `.mtl` at
/// `mtl` bind: its mesh-used materials. A model that did not parse binds none.
/// `folder_materials` and `common`'s are the parsed pre-Fox files' (`ContentPass::materials`).
pub(super) fn used_names<'a>(
    pairings: &[Pairing],
    mtl: &ScopePath,
    folder_materials: &'a BTreeMap<ScopePath, Vec<MaterialRead>>,
    common: &'a KeptCommon,
) -> BTreeSet<&'a str> {
    pairings
        .iter()
        .filter(|pairing| pairing.mtl.is_some_and(|paired| &paired.path == mtl))
        .filter_map(|pairing| common.materials_of(pairing.model?, folder_materials))
        .flatten()
        .filter(|material| material.mesh_used)
        .map(|material| material.name.as_str())
        .collect()
}

/// `model_material_undefined` on `scope` for `pairing`, a pre-Fox model of the model folder at
/// `folder`; `folder_materials` are the materials of the folder's parsed pre-Fox files,
/// `common`'s of the kept `Common/` ones (`ContentPass::materials`).
///
/// Each file is named below the folder, or by its export path in `Common/`
/// (`named_on_folder`): an entry of the folder's `face.xml` may pair a `Common/` model with
/// the folder's `.mtl`. When its search (`mtl_search::mtl_for`) finds no `.mtl`, the finding
/// names the model and is not pass-through-eligible: the face's `face.xml` must name a
/// material set for the model, and there is none to name. When it has one, the material
/// names the model binds (a link's: the linked `Common/` model's) that the `.mtl` does not
/// define are the finding's, in the model's order, naming the model, the `.mtl` and those
/// names; it is pass-through-eligible: the file packs as it is, and the game renders those
/// meshes with its fallback material. No finding when every name is defined, or when either
/// file did not parse (its own `model_broken` or `mtl_broken` drops it).
pub(super) fn material_finding(
    pairing: &Pairing,
    folder: &ScopePath,
    common: &KeptCommon,
    folder_materials: &BTreeMap<ScopePath, Vec<MaterialRead>>,
    scope: &IssueScope,
) -> Option<ContentFinding> {
    let name = named_on_folder(&pairing.file.path, folder);
    let Some(mtl) = pairing.mtl else {
        return Some(ContentFinding {
            code: MODEL_MATERIAL_UNDEFINED,
            scope: scope.clone(),
            context: vec![("file", name)],
            disposition: Disposition::DropFolder,
            pass_through_eligible: false,
        });
    };
    undefined_materials(
        name,
        pairing.model?,
        mtl,
        folder,
        common,
        folder_materials,
        scope,
    )
}

/// `model_material_undefined` on `scope` for the model at `model`, which the finding names
/// `name`, paired with `mtl`: the material names the model binds that the `.mtl` does not
/// define, in the model's order, naming the `.mtl` (`named_on_folder`); pass-through-eligible,
/// the file packing as it is. `None` when every name is defined, or when either file did not
/// parse. `folder_materials` and `common`'s are the materials of the parsed pre-Fox files
/// (`ContentPass::materials`).
fn undefined_materials(
    name: String,
    model: &ScopePath,
    mtl: &FileDescriptor,
    folder: &ScopePath,
    common: &KeptCommon,
    folder_materials: &BTreeMap<ScopePath, Vec<MaterialRead>>,
    scope: &IssueScope,
) -> Option<ContentFinding> {
    let defined = common.materials_of(&mtl.path, folder_materials)?;
    let undefined: Vec<&str> = common
        .materials_of(model, folder_materials)?
        .iter()
        .filter(|used| !defined.iter().any(|material| material.name == used.name))
        .map(|used| used.name.as_str())
        .collect();
    if undefined.is_empty() {
        return None;
    }
    Some(ContentFinding {
        code: MODEL_MATERIAL_UNDEFINED,
        scope: scope.clone(),
        context: vec![
            ("file", name),
            ("mtl", named_on_folder(&mtl.path, folder)),
            ("materials", undefined.join(", ")),
        ],
        disposition: Disposition::DropFolder,
        pass_through_eligible: true,
    })
}
