//! Stage 2, run planning (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! identity-resolved exports become one manifest of tasks, each an atomic unit that commits
//! whole or not at all.

pub(crate) mod ids;
pub(crate) mod subset;

use std::ops::Range;

use aesthetics_export::{
    ExportIdentity, FileDescriptor, KitFolder, KitsFolder, PlayerFolder, PlayerIndex, PlayerSlot,
    ResolvedAestheticsExport, ValidatedRoster,
};
use kit_config::KitSlot;
use pes_version::{Engine, PesVersion};
use studio_core::{Disposition, ExportId, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};
use ids::PlannedModelIds;
use subset::{FolderModels, ModelPackage, PlayerFile, first_not_compiled, player_file};

/// What planning produced: the manifest and the findings planning itself made.
pub(crate) struct PlanReport {
    /// Every task of the run, in canonical order.
    pub(crate) manifest: BuildManifest,
    /// Planning's findings (`content_not_yet_compiled`, `kit_config_generated`,
    /// `kit_placeholder`).
    pub(crate) messages: Vec<Message>,
}

/// The run's tasks in canonical order: by export, then each mapped player folder's tasks (its
/// face, boots and gloves packages, then its textures) by first roster slot, the portraits by
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

/// A player folder's tasks as one unit for the writer: its packages (face, boots, gloves in
/// canonical order), then its textures task last, contiguous in the manifest. The writer
/// holds the packages until the textures batch arrives and decides the group, so a folder
/// whose textures failed leaves no package pointing at textures that are not in the CPK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskGroup {
    /// The group's manifest positions, the textures task at `tasks.end - 1`.
    pub(crate) tasks: Range<usize>,
    /// The sum of the members' charges: the coordinator acquires it once, as one permit the
    /// members share, since a textures task waiting for a permit of its own while the writer
    /// holds its packages' would wait forever.
    pub(crate) charge: usize,
}

/// What a task compiles.
pub(crate) enum TaskKind {
    /// One package of a mapped player folder's models: its face, its boots or its gloves,
    /// with the files packed beside them. One task per package, whatever the number of
    /// roster slots mapping the folder: the package is built once and emitted under each
    /// slot's id.
    Models {
        /// The player folder.
        folder: PlayerFolder,
        /// Which of its packages.
        package: ModelPackage,
        /// The id the package is emitted under for every roster slot mapping the folder, in
        /// slot order: the slot's player id for the face, its planned boots/gloves id for
        /// the other two.
        ids: Vec<u32>,
    },
    /// A mapped player folder's own textures, converted once into the player's common
    /// folder, which every package of the folder points at. Always the last task of the
    /// folder's `TaskGroup`.
    Textures {
        /// The player folder.
        folder: PlayerFolder,
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
            TaskKind::Portrait { file, .. } => file.path.clone(),
            TaskKind::Kit { kit, .. } => kit.path.clone(),
        }
    }

    /// Every file the task reads from its export: a package's models and the files packed
    /// beside them; a folder's textures; a portrait's one file; a kit's config, when it has
    /// one, and its effective textures.
    pub(crate) fn files(&self) -> Vec<&FileDescriptor> {
        match self {
            TaskKind::Models {
                folder, package, ..
            } => folder_files(folder, |role| role.package() == Some(*package)),
            TaskKind::Textures { folder, .. } => {
                folder_files(folder, |role| role.package().is_none())
            }
            TaskKind::Portrait { file, .. } => vec![file],
            TaskKind::Kit { kit, .. } => kit
                .config
                .iter()
                .chain(kit.textures.iter().map(|texture| &texture.file))
                .collect(),
        }
    }
}

/// The files of `folder` whose role `wanted` accepts, in the folder's order.
fn folder_files(
    folder: &PlayerFolder,
    wanted: impl Fn(&PlayerFile) -> bool,
) -> Vec<&FileDescriptor> {
    let models = FolderModels::of(folder);
    folder
        .files
        .iter()
        .filter(|file| player_file(&folder.path, file, &models).is_some_and(|role| wanted(&role)))
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
        let export = resolved.export;
        // A folder's portrait goes out once per slot mapping the folder; the gate has refused
        // any slot with a portrait from both sources, so no player id comes up twice.
        let mut portraits: Vec<(u32, FileDescriptor)> = Vec::new();
        for (folder, slots) in player_folders(export.players, &export.roster) {
            if let Some(portrait) = &folder.portrait {
                portraits.extend(
                    slots
                        .iter()
                        .map(|slot| (slot.player_id(id), portrait.clone())),
                );
            }
            let first = tasks.len();
            for package in ModelPackage::ALL {
                let holds_model = folder_files(
                    &folder,
                    |role| matches!(role, PlayerFile::Model { package: owner, .. } if *owner == package),
                );
                if holds_model.is_empty() {
                    continue;
                }
                let ids = slots
                    .iter()
                    .map(|slot| match package {
                        ModelPackage::Face => slot.player_id(id),
                        ModelPackage::Boots | ModelPackage::Gloves => {
                            u32::from(model_ids.exclusive(*slot))
                        }
                    })
                    .collect();
                tasks.push(task(
                    export_id,
                    team_id,
                    TaskKind::Models {
                        folder: folder.clone(),
                        package,
                        ids,
                    },
                ));
            }
            if !folder_files(&folder, |role| role.package().is_none()).is_empty() {
                tasks.push(task(export_id, team_id, TaskKind::Textures { folder }));
                let members = &mut tasks[first..];
                let charge = members
                    .iter()
                    .fold(0usize, |sum, task| sum.saturating_add(task.charge));
                let group = TaskGroup {
                    tasks: first..first + members.len(),
                    charge,
                };
                for task in members {
                    task.group = Some(group.clone());
                }
            }
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
                ("Players/A/face_diff.bin", 1),
                ("Players/23 - B/boots.fmdl", 64),
            ],
            &[],
            Some(b"05 A\n07 A\n23 23 - B\n"),
        );

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        // Slots 05 and 07 of team 714 own the exclusive ids 625 and 627; slot 23 owns 643.
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
