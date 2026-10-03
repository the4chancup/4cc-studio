//! Stage 2, run planning (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! identity-resolved exports become one manifest of tasks, each an atomic unit that commits
//! whole or not at all.

pub(crate) mod ids;
pub(crate) mod subset;

use std::collections::BTreeSet;
use std::ops::Range;

use aesthetics_export::{
    ExportIdentity, FileDescriptor, KitFolder, KitsFolder, PlayerFolder, PlayerIndex, PlayerSlot,
    ResolvedAestheticsExport, SharedKind, SharedModelFolder, ValidatedAestheticsExport,
    ValidatedRoster, common_link_name,
};
use kit_config::KitSlot;
use pes_version::{Engine, PesVersion};
use studio_core::{Disposition, ExportId, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};
use crate::paths::TextureHome;
use ids::{PlannedModelIds, shared_folders_taking_ids};
use subset::{
    FolderModels, ModelPackage, PlayerFile, common_file, common_skeleton, file_stem,
    first_not_compiled, is_part_of, link_combines, link_name, linked_folder, package_of,
    player_file, skeleton_slot, texture_format,
};

/// What planning produced: the manifest and the findings planning itself made.
pub(crate) struct PlanReport {
    /// Every task of the run, in canonical order.
    pub(crate) manifest: BuildManifest,
    /// Planning's findings (`content_not_yet_compiled`, `kit_config_generated`,
    /// `kit_placeholder`).
    pub(crate) messages: Vec<Message>,
}

/// The run's tasks in canonical order: by export, then each mapped player folder's tasks (its
/// face, boots and gloves packages, then its textures) by first roster slot, the shared boots
/// folders taking an id (each its package, then its textures) in id order, then the shared
/// gloves folders the same way, the export's Common textures as one task, the portraits by
/// player id, then the kits by slot. The writer lays the CPK out in this order whatever order
/// the tasks finish in, so the same exports always give the same bytes.
pub(crate) struct BuildManifest {
    /// The tasks, in canonical order.
    pub(crate) tasks: Vec<BuildTask>,
}

/// One unit of work: one package of a player folder's models, the folder's textures, one
/// player's portrait, or one kit.
pub(crate) struct BuildTask {
    /// The export the task's content comes from.
    pub(crate) export_id: ExportId,
    /// The export's team id, which game paths and file names carry.
    pub(crate) team_id: u16,
    /// What the task compiles.
    pub(crate) kind: TaskKind,
    /// The source bytes the task reads, what its memory permit is charged (`libs/pipeline.md`
    /// "Memory budget").
    pub(crate) charge: usize,
    /// The player folder's group the task belongs to, when its folder has textures; `None`
    /// for every other task, which the writer commits on its own.
    pub(crate) group: Option<TaskGroup>,
}

/// A model folder's tasks as one unit for the writer: its packages (face, boots, gloves in
/// canonical order; a shared folder has one), then its textures task last, contiguous in the
/// manifest. The writer holds the packages until the textures batch arrives and decides the
/// group, so a folder whose textures failed leaves no package pointing at textures that are
/// not in the CPK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskGroup {
    /// The group's manifest positions, the textures task at `tasks.end - 1`.
    pub(crate) tasks: Range<usize>,
    /// The package each `Models` task of the group compiles, in manifest order: `packages[i]`
    /// is the package of task `tasks.start + i`, so the textures batch can name the task of a
    /// package it drops (`shared_texture_conflict`) for the writer to skip.
    pub(crate) packages: Vec<ModelPackage>,
    /// The sum of the members' charges: the coordinator acquires it once, as one permit the
    /// members share, since a textures task waiting for a permit of its own while the writer
    /// holds its packages' would wait forever.
    pub(crate) charge: usize,
}

/// The folder a `Models` or `Textures` task compiles: a mapped player folder, or a shared
/// boots or gloves folder a mapped player links plainly. Its files take the roles of a player
/// folder's (`subset::player_file`); where its textures go differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelFolder {
    /// The folder's export path (`Players/05 - A`, `Boots/Crocs`): the scope its findings name.
    pub(crate) path: ScopePath,
    /// Its own files.
    pub(crate) files: Vec<FileDescriptor>,
    /// The shared folders a player folder combines, in link order. Empty for a shared folder,
    /// and for a player linking plainly or not at all.
    pub(crate) combined: Vec<CombinedFolder>,
    /// The player folder's `.common` model links, each resolved to the Common model it brings
    /// in as a part. Empty for a shared folder, which holds no link.
    pub(crate) common_models: Vec<CommonModel>,
    /// The stems, as spelled, of the textures directly in the export's `Common/` folder, which
    /// the export's Common textures task emits into the team's Common output: a Common part's
    /// texture paths of these stems name that output, not the folder's texture home
    /// (`pipeline.md` "3. Per-model-folder parallel steps", step 6).
    pub(crate) common_texture_stems: BTreeSet<String>,
    /// Where its textures go, which its models' texture paths are rewritten to name.
    pub(crate) textures: TextureHome,
}

/// A player folder's `.common` link to an FMDL, resolved against the export's `Common/` folder
/// as validation resolved it (`player_folders.md` "Common model links and model merging"): the
/// Common model is a part of the package the link's role names, and its skeleton travels with
/// it ("SKL pairing").
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommonModel {
    /// The link file's export path (`Players/05 - A/legs.fmdl.common`), which the folder's
    /// files list and whose role (`PlayerFile::CommonModel`) the Common model takes.
    pub(crate) link: ScopePath,
    /// The Common model the link names (`Common/legs.fmdl`): what the Models task reads.
    pub(crate) model: FileDescriptor,
    /// The `.skl` of the model's stem directly in `Common/` (`Common/legs.skl`), when there is
    /// one and the role has a skeleton slot; `None` otherwise, a slotless role's skeleton being
    /// the structure pass's `skl_no_slot`.
    pub(crate) skeleton: Option<FileDescriptor>,
}

/// A shared folder a player folder combines (`player_folders.md` "A link plus local models
/// combines"), with the package it feeds: a `Faces/` folder the face, a `Boots/` folder the
/// boots, a `Gloves/` folder the gloves. Its models are parts of that package like the player's
/// own, and its textures join the player's textures task, counting for that package when a
/// stem conflicts (`pipeline.md` "3. Per-model-folder parallel steps", step 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombinedFolder {
    /// The package the folder's models and textures feed.
    pub(crate) package: ModelPackage,
    /// The shared folder.
    pub(crate) folder: SharedModelFolder,
}

/// One source of a model folder's files with their roles (`ModelFolder::roles`): the package
/// the source feeds, the source folder's path, and each of its files `compile` builds with
/// its role.
pub(crate) type SourceRoles<'a> = (
    ModelPackage,
    &'a ScopePath,
    Vec<(&'a FileDescriptor, PlayerFile)>,
);

impl ModelFolder {
    /// The folder's files by source with each file's role, the folder's own files first and
    /// then each combined folder's. A role resolves against the file's own source
    /// (`Boots/Crocs`'s `boots.skl` pairs with `Boots/Crocs`'s `boots.fmdl`, not the
    /// player's). The face's `face_diff.bin` and `fcl_hair_sim.fclo` come once: a combined
    /// face folder's copy is left out when the player folder holds one, and never read
    /// (`player_folders.md` "A link plus local models combines"). A `.common` model link
    /// stands for the Common files it resolved to (`common_models`): the Common model under
    /// the link's role, in the link's place, and its Common skeleton under the role's slot,
    /// paired with it by their shared `Common/<stem>`; the empty link itself is never read.
    pub(crate) fn roles(&self) -> Vec<SourceRoles<'_>> {
        let mut own = FolderModels::of(&self.path, &self.files);
        if self
            .combined
            .iter()
            .any(|shared| shared.package == ModelPackage::Face)
        {
            own = own.with_linked_face();
        }
        let mut sources = vec![(self.own_package(), &self.path, &self.files, own)];
        for shared in &self.combined {
            let path = &shared.folder.path;
            let files = &shared.folder.files;
            sources.push((shared.package, path, files, FolderModels::of(path, files)));
        }
        // The names of the face files an earlier source holds.
        let mut packed: Vec<&'static str> = Vec::new();
        let mut roles = Vec::new();
        for (package, path, files, models) in sources {
            let mut source_roles = Vec::new();
            for file in files {
                let Some(role) = player_file(path, file, &models) else {
                    continue;
                };
                match role {
                    PlayerFile::Packed { name, .. } => {
                        if packed.contains(&name) {
                            continue;
                        }
                        packed.push(name);
                    }
                    PlayerFile::CommonModel { package, name } => {
                        let resolved = self
                            .common_models
                            .iter()
                            .find(|common| common.link == file.path)
                            .expect("planning resolves every `.common` model link of a folder");
                        source_roles.push((&resolved.model, role));
                        if let Some(skeleton) = &resolved.skeleton {
                            let name = skeleton_slot(package, name).expect(
                                "planning pairs a skeleton only with a part that has a slot",
                            );
                            source_roles.push((skeleton, PlayerFile::Skeleton { package, name }));
                        }
                        continue;
                    }
                    PlayerFile::Model { .. }
                    | PlayerFile::Skeleton { .. }
                    | PlayerFile::SlotlessSkeleton
                    | PlayerFile::Texture(..) => {}
                }
                source_roles.push((file, role));
            }
            roles.push((package, path, source_roles));
        }
        roles
    }

    /// The package the folder's own files feed, which its own textures count for when a stem
    /// conflicts: a player folder's stand for its face, a shared folder's for its one package.
    fn own_package(&self) -> ModelPackage {
        match &self.textures {
            TextureHome::PlayerCommon { .. } => ModelPackage::Face,
            TextureHome::SharedOutput { package, .. } => *package,
        }
    }
}

/// What a task compiles.
pub(crate) enum TaskKind {
    /// One package of a model folder's models: a player folder's face, boots or gloves, or a
    /// shared folder's one package, with the files packed beside them. One task per package,
    /// whatever the number of roster slots mapping a player folder: the package is built once
    /// and emitted under each slot's id.
    Models {
        /// The model folder.
        folder: ModelFolder,
        /// Which of its packages.
        package: ModelPackage,
        /// The ids the package is emitted under: for a player folder one per roster slot
        /// mapping it, in slot order (the slot's player id for the face, its planned
        /// boots/gloves id for the other two); for a shared folder its one shared id.
        ids: Vec<u32>,
    },
    /// A model folder's own textures, converted once into the folder's texture home, which
    /// every package of the folder points at. Always the last task of the folder's `TaskGroup`.
    Textures {
        /// The model folder.
        folder: ModelFolder,
    },
    /// The textures directly in the export's `Common/` folder, converted once into the team's
    /// Common output, whether or not a `.common` link uses them (`pipeline.md` "Resolved
    /// decisions", "Common textures are one task of their export"). One task per export, in no
    /// group: its textures serve every linking player, so it commits on its own, and when it
    /// fails the linking players still commit.
    CommonTextures {
        /// The `Common/` folder's export path, the scope the task's findings name.
        folder: ScopePath,
        /// Its `.dds` and `.ftex` files.
        textures: Vec<FileDescriptor>,
    },
    /// One player's portrait, a DDS emitted as it is under the target version's file name.
    /// One task per player id: a folder two roster slots map gives two tasks over its one
    /// `portrait.dds`.
    Portrait {
        /// The player id the portrait is for.
        player_id: u32,
        /// The portrait file: the player folder's `portrait.dds`, or `Portraits/player_NN.dds`.
        file: FileDescriptor,
    },
    /// One kit, its `all/` inheritance already applied to its textures.
    Kit {
        /// The game's kit slot.
        slot: KitSlot,
        /// The kit folder.
        kit: KitFolder,
    },
}

impl TaskKind {
    /// The folder the task compiles, as the export spells it (a portrait's is its file): the
    /// scope its findings name.
    pub(crate) fn folder_path(&self) -> ScopePath {
        match self {
            TaskKind::Models { folder, .. } | TaskKind::Textures { folder, .. } => {
                folder.path.clone()
            }
            TaskKind::CommonTextures { folder, .. } => folder.clone(),
            TaskKind::Portrait { file, .. } => file.path.clone(),
            TaskKind::Kit { kit, .. } => kit.path.clone(),
        }
    }

    /// Every file the task reads from its export: a package's models (a `.common` link's
    /// Common model and skeleton, never the link) and the files packed beside them; a folder's
    /// textures; the Common textures; a portrait's one file; a kit's config, when it has one,
    /// and its effective textures.
    pub(crate) fn files(&self) -> Vec<&FileDescriptor> {
        match self {
            TaskKind::Models {
                folder, package, ..
            } => folder_files(folder, |role| role.package() == Some(*package)),
            TaskKind::Textures { folder, .. } => {
                folder_files(folder, |role| matches!(role, PlayerFile::Texture(..)))
            }
            TaskKind::CommonTextures { textures, .. } => textures.iter().collect(),
            TaskKind::Portrait { file, .. } => vec![file],
            TaskKind::Kit { kit, .. } => kit
                .config
                .iter()
                .chain(kit.textures.iter().map(|texture| &texture.file))
                .collect(),
        }
    }
}

/// The files of `folder` whose role `wanted` accepts: its own in their order, then each
/// combined folder's.
fn folder_files(
    folder: &ModelFolder,
    wanted: impl Fn(&PlayerFile) -> bool,
) -> Vec<&FileDescriptor> {
    folder
        .roles()
        .into_iter()
        .flat_map(|(_, _, files)| files)
        .filter(|(_, role)| wanted(role))
        .map(|(file, _)| file)
        .collect()
}

/// Every player folder a roster slot maps, in the export's folder order. A folder no slot maps
/// is not compiled.
pub(crate) fn mapped_players(export: &ValidatedAestheticsExport) -> Vec<&PlayerFolder> {
    let mapped: Vec<PlayerIndex> = match &export.roster {
        ValidatedRoster::Team(slots) => slots.values().copied().collect(),
        ValidatedRoster::Referees(slots) => slots.values().copied().collect(),
    };
    export
        .players
        .iter()
        .enumerate()
        .filter(|(index, _)| mapped.contains(&PlayerIndex(*index)))
        .map(|(_, folder)| folder)
        .collect()
}

/// Plans the run over the identity-resolved exports, given in `ExportId` order, for the target
/// `version`. An export holding anything Phase 3 cannot compile yet plans no task and reports
/// `content_not_yet_compiled` naming the first such item.
pub(crate) fn plan_run(
    exports: Vec<(ExportId, ResolvedAestheticsExport)>,
    version: PesVersion,
) -> PlanReport {
    let mut tasks = Vec::new();
    let mut messages = Vec::new();
    for (export_id, mut resolved) in exports {
        match version.engine() {
            Engine::Fox => drop_kit_masks(&mut resolved.export.kits),
            Engine::PreFox => {}
        }
        if let Some(item) = first_not_compiled(&resolved, version) {
            messages.push(tool_message(
                Code::ContentNotYetCompiled,
                Scope::Export { export_id },
                Disposition::DropExport,
                vec![item],
            ));
            continue;
        }
        let ExportIdentity::Team { id, .. } = resolved.identity else {
            unreachable!("the subset gate skips every referee export");
        };
        let team_id = id.get();
        let model_ids = PlannedModelIds::for_team(id);
        let mut export = resolved.export;
        // The shared folders taking an id, each with its package and that id, in the id order
        // of the kind: the boots folders, then the gloves folders.
        let mut shared: Vec<(ModelFolder, ModelPackage, u32)> = Vec::new();
        for kind in [SharedKind::Boots, SharedKind::Gloves] {
            let package = package_of(kind);
            let folders = shared_folders_taking_ids(&export, version.engine(), kind);
            for (index, folder) in folders.into_iter().enumerate() {
                let shared_id =
                    u32::from(model_ids.shared(index).expect(
                        "the structure pass drops an export whose shared pool is exhausted",
                    ));
                let folder = ModelFolder {
                    path: folder.path.clone(),
                    files: folder.files.clone(),
                    combined: Vec::new(),
                    common_models: Vec::new(),
                    common_texture_stems: BTreeSet::new(),
                    textures: TextureHome::SharedOutput {
                        package,
                        id: shared_id,
                    },
                };
                shared.push((folder, package, shared_id));
            }
        }
        // The textures directly in `Common/`: one task of the export's, and the stems a Common
        // part's paths name that task's output for.
        let common_textures: Vec<FileDescriptor> = export
            .common
            .iter()
            .filter(|file| texture_format(file.path.name()).is_some())
            .cloned()
            .collect();
        let common_texture_stems: BTreeSet<String> = common_textures
            .iter()
            .map(|file| file_stem(file.path.name()).to_owned())
            .collect();
        // A folder's portrait goes out once per slot mapping the folder; the gate has refused
        // any slot with a portrait from both sources, so no player id comes up twice.
        let mut portraits: Vec<(u32, FileDescriptor)> = Vec::new();
        // The player folders are taken out so the rest of the export (its shared folders)
        // stays readable while each folder's links are resolved against it.
        let players = std::mem::take(&mut export.players);
        for (folder, slots) in player_folders(players, &export.roster) {
            if let Some(portrait) = &folder.portrait {
                portraits.extend(
                    slots
                        .iter()
                        .map(|slot| (slot.player_id(id), portrait.clone())),
                );
            }
            // A link beside a local model of its package combines: the shared folder's files
            // become a second source of the player's folder, and the folder is reported once,
            // however many slots map it.
            let mut combined = Vec::new();
            for link in folder
                .links
                .iter()
                .filter(|link| link_combines(&folder, link))
            {
                let shared = linked_folder(&export, link)
                    .expect("validation drops a player folder whose link names no shared folder");
                messages.push(tool_message(
                    Code::LinkCombined,
                    Scope::Folder {
                        export_id,
                        path: folder.path.clone(),
                    },
                    Disposition::Keep,
                    vec![("link", link_name(link.kind, &link.name))],
                ));
                combined.push(CombinedFolder {
                    package: package_of(link.kind),
                    folder: shared.clone(),
                });
            }
            let packages = ModelPackage::ALL.map(|package| {
                let ids = slots
                    .iter()
                    .map(|slot| match package {
                        ModelPackage::Face => slot.player_id(id),
                        ModelPackage::Boots | ModelPackage::Gloves => {
                            u32::from(model_ids.exclusive(*slot))
                        }
                    })
                    .collect();
                (package, ids)
            });
            let model_folder = ModelFolder {
                textures: TextureHome::PlayerCommon {
                    folder_name: folder.path.name().to_owned(),
                },
                common_models: common_models(&folder, &export.common),
                common_texture_stems: common_texture_stems.clone(),
                path: folder.path,
                files: folder.files,
                combined,
            };
            folder_tasks(export_id, team_id, model_folder, &packages, &mut tasks);
        }
        for (folder, package, shared_id) in shared {
            folder_tasks(
                export_id,
                team_id,
                folder,
                &[(package, vec![shared_id])],
                &mut tasks,
            );
        }
        if let Some(first) = common_textures.first() {
            let folder = first
                .path
                .parent()
                .expect("a Common texture sits in the export's Common/ folder");
            tasks.push(task(
                export_id,
                team_id,
                TaskKind::CommonTextures {
                    folder,
                    textures: common_textures,
                },
            ));
        }
        portraits.extend(
            export
                .portraits
                .into_iter()
                .map(|(slot, file)| (slot.player_id(id), file)),
        );
        portraits.sort_by_key(|(player_id, _)| *player_id);
        for (player_id, file) in portraits {
            tasks.push(task(
                export_id,
                team_id,
                TaskKind::Portrait { player_id, file },
            ));
        }
        for (slot, kit) in export.kits.kits {
            let folder = || Scope::Folder {
                export_id,
                path: kit.path.clone(),
            };
            if kit.config.is_none() {
                messages.push(tool_message(
                    Code::KitConfigGenerated,
                    folder(),
                    Disposition::Keep,
                    vec![],
                ));
            }
            if !kit.textures.iter().any(|texture| texture.stem == "kit") {
                messages.push(tool_message(
                    Code::KitPlaceholder,
                    folder(),
                    Disposition::Keep,
                    vec![],
                ));
            }
            tasks.push(task(export_id, team_id, TaskKind::Kit { slot, kit }));
        }
    }
    PlanReport {
        manifest: BuildManifest { tasks },
        messages,
    }
}

/// Pushes `folder`'s tasks onto `tasks`: one `Models` task for each of `packages` any of the
/// folder's sources holds a model of (a face link alone makes the shared face the player's),
/// emitted under that package's ids, then, when the folder has textures, its `Textures` task,
/// the lot as one `TaskGroup`.
fn folder_tasks(
    export_id: ExportId,
    team_id: u16,
    folder: ModelFolder,
    packages: &[(ModelPackage, Vec<u32>)],
    tasks: &mut Vec<BuildTask>,
) {
    let first = tasks.len();
    let mut held = Vec::new();
    for (package, ids) in packages {
        let models = folder_files(&folder, |role| is_part_of(role, *package));
        if models.is_empty() {
            continue;
        }
        held.push(*package);
        tasks.push(task(
            export_id,
            team_id,
            TaskKind::Models {
                folder: folder.clone(),
                package: *package,
                ids: ids.clone(),
            },
        ));
    }
    if folder_files(&folder, |role| matches!(role, PlayerFile::Texture(..))).is_empty() {
        return;
    }
    tasks.push(task(export_id, team_id, TaskKind::Textures { folder }));
    let members = &mut tasks[first..];
    let charge = members
        .iter()
        .fold(0usize, |sum, task| sum.saturating_add(task.charge));
    let group = TaskGroup {
        tasks: first..first + members.len(),
        packages: held,
        charge,
    };
    for task in members {
        task.group = Some(group.clone());
    }
}

/// Removes every kit's `kit_mask`. A Fox kit has no mask slot, so a Fox target never emits
/// one; it goes before the subset gate, which would otherwise skip the export for it, and
/// before the kit's task, which would otherwise read it.
fn drop_kit_masks(kits: &mut KitsFolder) {
    for kit in kits.kits.values_mut() {
        kit.textures.retain(|texture| texture.stem != "kit_mask");
    }
}

/// Every roster-mapped player folder with the slots mapping it, in slot order, the folders
/// ordered by their first slot. A folder no slot maps is not compiled.
fn player_folders(
    players: Vec<PlayerFolder>,
    roster: &ValidatedRoster,
) -> Vec<(PlayerFolder, Vec<PlayerSlot>)> {
    // A referee roster never reaches here: its export plans no task.
    let ValidatedRoster::Team(slots) = roster else {
        return Vec::new();
    };
    let mut mapped: Vec<(PlayerIndex, Vec<PlayerSlot>)> = Vec::new();
    for (slot, index) in slots {
        match mapped.iter_mut().find(|(known, _)| known == index) {
            Some((_, folder_slots)) => folder_slots.push(*slot),
            None => mapped.push((*index, vec![*slot])),
        }
    }
    let mut players: Vec<Option<PlayerFolder>> = players.into_iter().map(Some).collect();
    mapped
        .into_iter()
        .filter_map(|(index, folder_slots)| {
            let folder = players.get_mut(index.0)?.take()?;
            Some((folder, folder_slots))
        })
        .collect()
}

/// The player folder's `.common` model links resolved against `common`, the export's `Common/`
/// files, exactly as validation resolved them (a file directly in `Common/`, matched by
/// case-folded name): each with its Common model and, when the link's role has a skeleton
/// slot, the Common `.skl` of the model's stem. Validation drops a folder whose link names no
/// Common file, so every link here resolves.
fn common_models(folder: &PlayerFolder, common: &[FileDescriptor]) -> Vec<CommonModel> {
    let models = FolderModels::of_player(folder);
    folder
        .files
        .iter()
        .filter_map(|file| {
            let Some(PlayerFile::CommonModel { package, name }) =
                player_file(&folder.path, file, &models)
            else {
                return None;
            };
            let linked = common_link_name(file.path.name())
                .expect("a CommonModel role implies a `.common` link name");
            let model = common_file(common, &linked)
                .expect("validation drops a player folder whose link names no Common file");
            let skeleton = skeleton_slot(package, name)
                .and_then(|_| common_skeleton(common, &linked))
                .cloned();
            Some(CommonModel {
                link: file.path.clone(),
                model: model.clone(),
                skeleton,
            })
        })
        .collect()
}

/// The task compiling `kind`, charged the bytes of the files it reads. A sum past `usize` (a
/// 32-bit host only) is over any memory cap, and so is the saturated value.
fn task(export_id: ExportId, team_id: u16, kind: TaskKind) -> BuildTask {
    let size: u64 = kind.files().iter().map(|file| file.size).sum();
    BuildTask {
        export_id,
        team_id,
        charge: usize::try_from(size).unwrap_or(usize::MAX),
        kind,
        group: None,
    }
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::{resolved, resolved_with_issues};

    /// Each task as one line: export, team, what it compiles, charge.
    fn summary(report: &PlanReport) -> Vec<String> {
        report
            .manifest
            .tasks
            .iter()
            .map(|task| {
                let what = match &task.kind {
                    TaskKind::Models {
                        folder,
                        package,
                        ids,
                    } => format!("{package:?} {} {ids:?}", folder.path.as_str()),
                    TaskKind::Textures { folder } => {
                        format!("textures {}", folder.path.as_str())
                    }
                    TaskKind::CommonTextures { folder, textures } => {
                        format!("common textures {} ({})", folder.as_str(), textures.len())
                    }
                    TaskKind::Portrait { player_id, file } => {
                        format!("portrait {player_id} {}", file.path.as_str())
                    }
                    TaskKind::Kit { slot, kit } => {
                        format!("kit {} {}", slot.as_str(), kit.path.as_str())
                    }
                };
                format!(
                    "{} {} {what} charge {}",
                    task.export_id.0, task.team_id, task.charge
                )
            })
            .collect()
    }

    #[test]
    fn tasks_go_by_export_then_faces_by_first_slot_then_kits_by_slot() {
        let first = resolved(
            "dbg - Two",
            &[
                ("Players/Zed/face_high.fmdl", 10),
                ("Players/Zed/face_diff.bin", 0),
                ("Players/Zed/hair.dds", 5),
                ("Players/Amy/face_high.fmdl", 20),
                ("Players/Amy/face_diff.bin", 0),
                ("Kits/g1/kit.dds", 7),
                ("Kits/g1/config.toml", 3),
                ("Kits/p2 - Away/kit.dds", 8),
            ],
            &["Kits/p1"],
            // Amy is listed first in the file but holds the later slot.
            Some(b"07 Amy\n03 Zed\n09 Amy\n"),
        );
        let second = resolved(
            "co - One",
            &[
                ("Players/04 - B/face_high.fmdl", 1),
                ("Players/04 - B/face_diff.bin", 0),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![(ExportId(0), first), (ExportId(1), second)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 790 Face Players/Zed [79003] charge 10",
                "0 790 textures Players/Zed charge 5",
                "0 790 Face Players/Amy [79007, 79009] charge 20",
                "0 790 kit p1 Kits/p1 charge 0",
                "0 790 kit p2 Kits/p2 - Away charge 8",
                "0 790 kit g1 Kits/g1 charge 10",
                "1 714 Face Players/04 - B [71404] charge 1",
            ]
        );
        // Zed's two tasks are one group charged as one; Amy, with no textures, is ungrouped.
        let groups: Vec<Option<TaskGroup>> = report
            .manifest
            .tasks
            .iter()
            .map(|task| task.group.clone())
            .collect();
        let zed = Some(TaskGroup {
            tasks: 0..2,
            packages: vec![ModelPackage::Face],
            charge: 15,
        });
        assert_eq!(groups, [zed.clone(), zed, None, None, None, None, None]);
    }

    #[test]
    fn a_folder_s_packages_go_face_boots_gloves_then_its_textures_under_the_planned_ids() {
        let export = resolved(
            "co - Models",
            &[
                ("Players/A/glove_l.fmdl", 4),
                ("Players/A/kit_boots.skl", 2),
                ("Players/A/kit_boots.fmdl", 8),
                ("Players/A/shirt.dds", 16),
                ("Players/A/face_high.fmdl", 32),
                ("Players/A/face_high.skl", 128),
                ("Players/A/face_diff.bin", 1),
                ("Players/23 - B/boots.fmdl", 64),
            ],
            &[],
            Some(b"05 A\n07 A\n23 23 - B\n"),
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // Slots 05 and 07 of team 714 own the exclusive ids 625 and 627; slot 23 owns 643.
        // `face_high.skl` has no slot: no task reads it, so no charge counts it.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/A [71405, 71407] charge 33",
                "0 714 Boots Players/A [625, 627] charge 10",
                "0 714 Gloves Players/A [625, 627] charge 4",
                "0 714 textures Players/A charge 16",
                "0 714 Boots Players/23 - B [643] charge 64",
            ]
        );
        let group = Some(TaskGroup {
            tasks: 0..4,
            packages: ModelPackage::ALL.to_vec(),
            charge: 63,
        });
        for index in 0..4 {
            assert_eq!(report.manifest.tasks[index].group, group, "task {index}");
        }
        assert_eq!(report.manifest.tasks[4].group, None);
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.name())
                .collect()
        };
        assert_eq!(files(0), ["face_diff.bin", "face_high.fmdl"]);
        assert_eq!(files(1), ["kit_boots.fmdl", "kit_boots.skl"]);
        assert_eq!(files(2), ["glove_l.fmdl"]);
        assert_eq!(files(3), ["shirt.dds"]);
        for index in 0..4 {
            assert_eq!(
                report.manifest.tasks[index].kind.folder_path(),
                scope_path("Players/A"),
                "task {index}"
            );
        }
    }

    #[test]
    fn shared_folders_follow_the_players_boots_then_gloves_under_the_shared_ids_in_name_order() {
        let export = resolved(
            "co - Shared",
            &[
                ("Players/03 - A/Zebra.boots", 0),
                ("Players/03 - A/Grip.gloves", 0),
                ("Players/07 - B/face_high.fmdl", 32),
                ("Players/07 - B/face_diff.bin", 1),
                ("Players/07 - B/apple.boots", 0),
                ("Players/11 - C/Zebra.boots", 0),
                ("Boots/Zebra/boots.fmdl", 8),
                ("Boots/Zebra/boots.skl", 2),
                ("Boots/Zebra/shirt.dds", 16),
                ("Boots/apple/kit_boots.fmdl", 4),
                ("Gloves/Grip/glove_l.fmdl", 64),
                ("Gloves/Grip/grip.dds", 128),
            ],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // Slots 03 and 11 wear Zebra and 07 apple, so no player has a boots package; team
        // 714's shared ids start at 644, in case-folded name order.
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/07 - B [71407] charge 33",
                "0 714 Boots Boots/apple [644] charge 4",
                "0 714 Boots Boots/Zebra [645] charge 10",
                "0 714 textures Boots/Zebra charge 16",
                "0 714 Gloves Gloves/Grip [644] charge 64",
                "0 714 textures Gloves/Grip charge 128",
            ]
        );
        let groups: Vec<Option<TaskGroup>> = report
            .manifest
            .tasks
            .iter()
            .map(|task| task.group.clone())
            .collect();
        let zebra = Some(TaskGroup {
            tasks: 2..4,
            packages: vec![ModelPackage::Boots],
            charge: 26,
        });
        let grip = Some(TaskGroup {
            tasks: 4..6,
            packages: vec![ModelPackage::Gloves],
            charge: 192,
        });
        assert_eq!(
            groups,
            [None, None, zebra.clone(), zebra, grip.clone(), grip]
        );
        let homes: Vec<&TextureHome> = report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Models { folder, .. } | TaskKind::Textures { folder } => {
                    Some(&folder.textures)
                }
                TaskKind::CommonTextures { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. } => None,
            })
            .collect();
        assert_eq!(
            homes,
            [
                &TextureHome::PlayerCommon {
                    folder_name: "07 - B".to_owned()
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Boots,
                    id: 644
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Boots,
                    id: 645
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Boots,
                    id: 645
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Gloves,
                    id: 644
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Gloves,
                    id: 644
                },
            ]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(
            files(2),
            ["Boots/Zebra/boots.fmdl", "Boots/Zebra/boots.skl"]
        );
        assert_eq!(files(3), ["Boots/Zebra/shirt.dds"]);
        assert_eq!(
            report.manifest.tasks[2].kind.folder_path(),
            scope_path("Boots/Zebra")
        );
    }

    #[test]
    fn a_link_beside_a_local_model_combines_the_shared_folder_into_the_player_s_package() {
        let files = [
            ("Players/05 - A/Crocs.boots", 0),
            ("Players/05 - A/kit_boots.fmdl", 8),
            ("Players/05 - A/kit_boots.skl", 2),
            ("Players/05 - A/Grip.gloves", 0),
            ("Players/05 - A/glove_l.fmdl", 4),
            ("Boots/Crocs/boots.fmdl", 16),
            ("Boots/Crocs/boots.skl", 1),
            ("Boots/Crocs/sole.dds", 32),
            ("Gloves/Grip/glove_r.fmdl", 64),
        ];
        let export = resolved("co - Combined", &files, &[], None);

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // The shared folders are only sources of parts: no shared id, no output of their own.
        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 27",
                "0 714 Gloves Players/05 - A [625] charge 68",
                "0 714 textures Players/05 - A charge 32",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [
                ("link_combined", "Players/05 - A", Disposition::Keep),
                ("link_combined", "Players/05 - A", Disposition::Keep),
            ]
        );
        let links: Vec<&str> = report
            .messages
            .iter()
            .map(|message| message.context[0].1.as_str())
            .collect();
        assert_eq!(links, ["Crocs.boots", "Grip.gloves"]);
        assert!(
            report
                .messages
                .iter()
                .all(|message| message.context[0].0 == "link"),
            "{:?}",
            report.messages
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        // The player's own files first, then each combined folder's, each skeleton pairing
        // with the model of its stem in its own folder.
        assert_eq!(
            files(0),
            [
                "Players/05 - A/kit_boots.fmdl",
                "Players/05 - A/kit_boots.skl",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/boots.skl",
            ]
        );
        assert_eq!(
            files(1),
            ["Players/05 - A/glove_l.fmdl", "Gloves/Grip/glove_r.fmdl"]
        );
        assert_eq!(files(2), ["Boots/Crocs/sole.dds"]);
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[0].kind else {
            panic!("a package task");
        };
        let combined: Vec<(ModelPackage, &str)> = folder
            .combined
            .iter()
            .map(|shared| (shared.package, shared.folder.path.as_str()))
            .collect();
        assert_eq!(
            combined,
            [
                (ModelPackage::Boots, "Boots/Crocs"),
                (ModelPackage::Gloves, "Gloves/Grip")
            ]
        );
        assert_eq!(
            folder.textures,
            TextureHome::PlayerCommon {
                folder_name: "05 - A".to_owned()
            }
        );
        let group = Some(TaskGroup {
            tasks: 0..3,
            packages: vec![ModelPackage::Boots, ModelPackage::Gloves],
            charge: 127,
        });
        for index in 0..3 {
            assert_eq!(report.manifest.tasks[index].group, group, "task {index}");
        }
    }

    #[test]
    fn a_reserved_subfolder_s_parts_join_their_category_s_task_and_a_link_of_it_combines() {
        let export = resolved(
            "co - Subfolders",
            &[
                ("Players/05 - A/boots/boots.fmdl", 8),
                ("Players/05 - A/boots/boots.skl", 2),
                ("Players/05 - A/common/skin.dds", 16),
                ("Players/05 - A/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 4),
            ],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // The boots alone: no face task for a folder whose only model is in `boots/`.
        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 14",
                "0 714 textures Players/05 - A charge 16",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(
            files(0),
            [
                "Players/05 - A/boots/boots.fmdl",
                "Players/05 - A/boots/boots.skl",
                "Boots/Crocs/boots.fmdl",
            ]
        );
        assert_eq!(files(1), ["Players/05 - A/common/skin.dds"]);
    }

    #[test]
    fn a_common_link_s_task_reads_the_common_model_and_skeleton_and_the_common_textures_are_one_task()
     {
        let export = resolved(
            "co - Common",
            &[
                ("Players/05 - A/torso.fmdl", 8),
                ("Players/05 - A/legs.fmdl.common", 0),
                ("Players/05 - A/boots/Kit_Boots.fmdl.common", 0),
                ("Players/05 - A/face_high.fmdl.common", 0),
                ("Players/05 - A/skin.dds", 4),
                ("Players/07 - B/legs.fmdl.common", 0),
                ("Players/07 - B/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 2),
                ("Common/Legs.fmdl", 16),
                ("Common/legs.skl", 1),
                ("Common/kit_boots.fmdl", 32),
                ("Common/kit_boots.skl", 64),
                ("Common/face_high.fmdl", 128),
                ("Common/face_high.skl", 256),
                ("Common/spare.fmdl", 512),
                ("Common/Cloth.dds", 1024),
                ("Common/hair.ftex", 2048),
                ("Portraits/player_05.dds", 3),
            ],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // Slot 05's face reads the local part, the two Common models and the hair's skeleton,
        // never the links nor the slotless `face_high.skl`; its boots read Common's model and
        // skeleton. The Common textures follow the shared folders and precede the portraits,
        // and the unlinked `spare.fmdl` is read by nothing.
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 153",
                "0 714 Boots Players/05 - A [625] charge 96",
                "0 714 textures Players/05 - A charge 4",
                "0 714 Face Players/07 - B [71407] charge 17",
                "0 714 Boots Boots/Crocs [644] charge 2",
                "0 714 common textures Common (2) charge 3072",
                "0 714 portrait 71405 Portraits/player_05.dds charge 3",
            ]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(
            files(0),
            [
                "Common/face_high.fmdl",
                "Common/Legs.fmdl",
                "Common/legs.skl",
                "Players/05 - A/torso.fmdl",
            ]
        );
        assert_eq!(files(1), ["Common/kit_boots.fmdl", "Common/kit_boots.skl"]);
        assert_eq!(files(3), ["Common/Legs.fmdl", "Common/legs.skl"]);
        assert_eq!(files(5), ["Common/Cloth.dds", "Common/hair.ftex"]);
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[0].kind else {
            panic!("a package task");
        };
        let links: Vec<(&str, &str, Option<&str>)> = folder
            .common_models
            .iter()
            .map(|common| {
                (
                    common.link.as_str(),
                    common.model.path.as_str(),
                    common.skeleton.as_ref().map(|file| file.path.as_str()),
                )
            })
            .collect();
        assert_eq!(
            links,
            [
                (
                    "Players/05 - A/boots/Kit_Boots.fmdl.common",
                    "Common/kit_boots.fmdl",
                    Some("Common/kit_boots.skl")
                ),
                (
                    "Players/05 - A/face_high.fmdl.common",
                    "Common/face_high.fmdl",
                    None
                ),
                (
                    "Players/05 - A/legs.fmdl.common",
                    "Common/Legs.fmdl",
                    Some("Common/legs.skl")
                ),
            ]
        );
        assert_eq!(
            folder.common_texture_stems,
            BTreeSet::from(["Cloth".to_owned(), "hair".to_owned()])
        );
        // The Common task is in no group: a player's packages never wait for it.
        let common = &report.manifest.tasks[5];
        assert_eq!(common.group, None);
        assert_eq!(common.kind.folder_path(), scope_path("Common"));
        assert_eq!(
            report.manifest.tasks[2].group,
            Some(TaskGroup {
                tasks: 0..3,
                packages: vec![ModelPackage::Face, ModelPackage::Boots],
                charge: 253,
            })
        );
    }

    #[test]
    fn a_face_link_alone_makes_the_shared_face_the_player_s_and_the_player_s_face_files_win() {
        let export = resolved(
            "co - Faces",
            &[
                ("Players/05 - A/Longhair.face", 0),
                ("Players/05 - A/face_diff.bin", 3),
                ("Players/05 - A/skin.dds", 5),
                ("Faces/Longhair/hair_high.fmdl", 16),
                ("Faces/Longhair/face_diff.bin", 7),
                ("Faces/Longhair/hair.dds", 32),
            ],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // The face task exists for the shared model alone, and is charged the player's
        // `face_diff.bin`, not the shared folder's, which is never read.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 19",
                "0 714 textures Players/05 - A charge 37",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        assert_eq!(report.messages[0].context[0].1, "Longhair.face");
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(
            files(0),
            [
                "Players/05 - A/face_diff.bin",
                "Faces/Longhair/hair_high.fmdl"
            ]
        );
        assert_eq!(
            files(1),
            ["Players/05 - A/skin.dds", "Faces/Longhair/hair.dds"]
        );
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[0].kind else {
            panic!("a package task");
        };
        assert_eq!(folder.combined[0].package, ModelPackage::Face);
        let group = Some(TaskGroup {
            tasks: 0..2,
            packages: vec![ModelPackage::Face],
            charge: 56,
        });
        assert_eq!(report.manifest.tasks[0].group, group);
    }

    #[test]
    fn a_combined_shared_folder_still_compiles_on_its_own_for_a_player_linking_it_plainly() {
        let export = resolved(
            "co - Combined",
            &[
                ("Players/05 - A/Crocs.boots", 0),
                ("Players/05 - A/kit_boots.fmdl", 8),
                ("Players/07 - B/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 16),
            ],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 24",
                "0 714 Boots Boots/Crocs [644] charge 16",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[1].kind else {
            panic!("a package task");
        };
        assert!(folder.combined.is_empty(), "the shared folder's own task");
    }

    #[test]
    fn portraits_follow_the_faces_by_player_id_one_per_slot_of_their_folder() {
        let export = resolved(
            "dbg - Portraits",
            &[
                ("Players/Zed/face_high.fmdl", 10),
                ("Players/Zed/face_diff.bin", 0),
                ("Players/Zed/portrait.dds", 4),
                ("Portraits/player_05.dds", 6),
                ("Portraits/player_01.dds", 5),
                ("Kits/g1/kit.dds", 7),
            ],
            &[],
            Some(b"07 Zed\n03 Zed\n"),
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        assert_eq!(
            summary(&report),
            [
                "0 790 Face Players/Zed [79003, 79007] charge 10",
                "0 790 portrait 79001 Portraits/player_01.dds charge 5",
                "0 790 portrait 79003 Players/Zed/portrait.dds charge 4",
                "0 790 portrait 79005 Portraits/player_05.dds charge 6",
                "0 790 portrait 79007 Players/Zed/portrait.dds charge 4",
                "0 790 kit g1 Kits/g1 charge 7",
            ]
        );
        let portrait = &report.manifest.tasks[2].kind;
        assert_eq!(
            portrait.folder_path(),
            scope_path("Players/Zed/portrait.dds")
        );
        let files: Vec<&str> = portrait
            .files()
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        assert_eq!(files, ["Players/Zed/portrait.dds"]);
    }

    #[test]
    fn a_kit_without_config_toml_reports_its_generated_config() {
        let export = resolved(
            "co - Kits",
            &[("Kits/p1/config.toml", 3), ("Kits/p2 - Away/kit.dds", 8)],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(3), export)], PesVersion::Pes21);

        let messages: Vec<(&str, &Scope, Disposition)> = report
            .messages
            .iter()
            .map(|message| {
                (
                    message.code.code.as_ref(),
                    &message.scope,
                    message.disposition,
                )
            })
            .collect();
        let scope = Scope::Folder {
            export_id: ExportId(3),
            path: ScopePath::new("Kits/p2 - Away").unwrap(),
        };
        // p1 holds only its config, so it is a placeholder kit.
        let p1 = Scope::Folder {
            export_id: ExportId(3),
            path: ScopePath::new("Kits/p1").unwrap(),
        };
        assert_eq!(
            messages,
            [
                ("kit_placeholder", &p1, Disposition::Keep),
                ("kit_config_generated", &scope, Disposition::Keep),
            ]
        );
        assert_eq!(
            report.manifest.tasks[1].kind.folder_path(),
            scope_path("Kits/p2 - Away")
        );
        assert_eq!(
            report.manifest.tasks[0].kind.folder_path(),
            scope_path("Kits/p1")
        );
    }

    /// Each planning message as (code, scope path, disposition).
    fn message_summary(report: &PlanReport) -> Vec<(&str, &str, Disposition)> {
        report
            .messages
            .iter()
            .map(|message| {
                let Scope::Folder { path, .. } = &message.scope else {
                    panic!("{:?}", message.scope);
                };
                (
                    message.code.code.as_ref(),
                    path.as_str(),
                    message.disposition,
                )
            })
            .collect()
    }

    #[test]
    fn a_kit_without_a_main_texture_reports_the_placeholder_after_its_config() {
        let export = resolved(
            "co - Kits",
            &[
                ("Kits/p1/kit.dds", 8),
                ("Kits/p3/kit_back.dds", 8),
                ("Kits/p4/config.toml", 3),
            ],
            &["Kits/p2"],
            None,
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        assert_eq!(
            message_summary(&report),
            [
                ("kit_config_generated", "Kits/p1", Disposition::Keep),
                ("kit_config_generated", "Kits/p2", Disposition::Keep),
                ("kit_placeholder", "Kits/p2", Disposition::Keep),
                ("kit_config_generated", "Kits/p3", Disposition::Keep),
                ("kit_placeholder", "Kits/p3", Disposition::Keep),
                ("kit_placeholder", "Kits/p4", Disposition::Keep),
            ]
        );
        assert!(
            report
                .messages
                .iter()
                .all(|message| message.context.is_empty()),
            "{:?}",
            report.messages
        );
    }

    #[test]
    fn a_kit_inheriting_the_main_texture_from_all_is_no_placeholder() {
        // The inheritance finding is validation's, not planning's.
        let (export, issues) = resolved_with_issues(
            "co - Kits",
            &[("Kits/all/kit.dds", 8), ("Kits/p1/config.toml", 3)],
            &[],
            None,
        );
        assert_eq!(issues, ["kit_textures_inherited"]);

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        let [task] = report.manifest.tasks.as_slice() else {
            panic!("{}", report.manifest.tasks.len());
        };
        let TaskKind::Kit { kit, .. } = &task.kind else {
            panic!("a kit task");
        };
        let stems: Vec<&str> = kit
            .textures
            .iter()
            .map(|texture| texture.stem.as_str())
            .collect();
        assert_eq!(stems, ["kit"]);
    }

    #[test]
    fn a_face_task_names_its_player_folder() {
        let export = resolved(
            "co - One",
            &[
                ("Players/04 - B/face_high.fmdl", 1),
                ("Players/04 - B/face_diff.bin", 0),
            ],
            &[],
            None,
        );
        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);
        assert_eq!(
            report.manifest.tasks[0].kind.folder_path(),
            scope_path("Players/04 - B")
        );
    }

    #[test]
    fn an_export_the_subset_gate_refuses_plans_no_task_and_reports_why() {
        let referees = resolved(
            "refs Cup",
            &[
                ("Players/Keeper/face_high.fmdl", 1),
                ("Players/Keeper/face_diff.bin", 0),
            ],
            &[],
            Some(b"01 Keeper\n"),
        );
        let kit = resolved("co - Kit", &[("Kits/g1/kit.dds", 1)], &[], None);

        let report = plan_run(
            vec![(ExportId(2), referees), (ExportId(3), kit)],
            PesVersion::Pes21,
        );

        assert_eq!(summary(&report), ["3 714 kit g1 Kits/g1 charge 1"]);
        assert_eq!(report.manifest.tasks[0].kind.files().len(), 1);
        let [skipped, generated] = report.messages.as_slice() else {
            panic!("{:?}", report.messages);
        };
        assert_eq!(skipped.code.code, "content_not_yet_compiled");
        assert_eq!(
            (skipped.severity, skipped.disposition),
            (Severity::Error, Disposition::DropExport)
        );
        assert_eq!(
            skipped.scope,
            Scope::Export {
                export_id: ExportId(2)
            }
        );
        assert_eq!(skipped.context, [("what".to_owned(), "refs".to_owned())]);
        assert_eq!(generated.code.code, "kit_config_generated");
    }

    fn scope_path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }
}
