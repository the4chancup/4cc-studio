//! Stage 2, run planning (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! identity-resolved exports become one manifest of tasks, each an atomic unit that commits
//! whole or not at all.

pub(crate) mod subset;

use aesthetics_export::{
    ExportIdentity, FileDescriptor, KitFolder, KitsFolder, PlayerFolder, PlayerIndex,
    ResolvedAestheticsExport, ValidatedRoster,
};
use kit_config::KitSlot;
use pes_version::{Engine, PesVersion};
use studio_core::{Disposition, ExportId, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};
use subset::first_not_compiled;

/// What planning produced: the manifest and the findings planning itself made.
pub(crate) struct PlanReport {
    /// Every task of the run, in canonical order.
    pub(crate) manifest: BuildManifest,
    /// Planning's findings (`content_not_yet_compiled`, `kit_config_generated`).
    pub(crate) messages: Vec<Message>,
}

/// The run's tasks in canonical order: by export, then the faces by first roster slot, then the
/// kits by slot. The writer lays the CPK out in this order whatever order the tasks finish in,
/// so the same exports always give the same bytes.
pub(crate) struct BuildManifest {
    /// The tasks, in canonical order.
    pub(crate) tasks: Vec<BuildTask>,
}

/// One unit of work: one player folder's face, or one kit.
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
}

/// What a task compiles.
pub(crate) enum TaskKind {
    /// A mapped player folder's face content. One task per folder, whatever the number of
    /// roster slots mapping it: its textures are converted once and every slot's face package
    /// points at them, one `face.fpk` per slot.
    Face {
        /// The player folder.
        folder: PlayerFolder,
        /// The player id of every roster slot mapping the folder, in slot order.
        player_ids: Vec<u32>,
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
    /// The folder the task compiles, as the export spells it: the scope its findings name.
    pub(crate) fn folder_path(&self) -> ScopePath {
        match self {
            TaskKind::Face { folder, .. } => folder.path.clone(),
            TaskKind::Kit { kit, .. } => kit.path.clone(),
        }
    }

    /// Every file the task reads from its export: a face's folder files; a kit's config, when
    /// it has one, and its effective textures.
    pub(crate) fn files(&self) -> Vec<&FileDescriptor> {
        match self {
            TaskKind::Face { folder, .. } => folder.files.iter().collect(),
            TaskKind::Kit { kit, .. } => kit
                .config
                .iter()
                .chain(kit.textures.iter().map(|texture| &texture.file))
                .collect(),
        }
    }
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
        if version.engine() == Engine::Fox {
            drop_kit_masks(&mut resolved.export.kits);
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
        let export = resolved.export;
        for (folder, player_ids) in face_folders(export.players, &export.roster, id) {
            tasks.push(task(
                export_id,
                team_id,
                TaskKind::Face { folder, player_ids },
            ));
        }
        for (slot, kit) in export.kits.kits {
            if kit.config.is_none() {
                messages.push(tool_message(
                    Code::KitConfigGenerated,
                    Scope::Folder {
                        export_id,
                        path: kit.path.clone(),
                    },
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

/// Every roster-mapped player folder with the player ids of its slots, ordered by each
/// folder's first slot. A folder no slot maps is not compiled.
fn face_folders(
    players: Vec<PlayerFolder>,
    roster: &ValidatedRoster,
    team: teams_list::TeamId,
) -> Vec<(PlayerFolder, Vec<u32>)> {
    // A referee roster never reaches here: its export plans no task.
    let ValidatedRoster::Team(slots) = roster else {
        return Vec::new();
    };
    let mut mapped: Vec<(PlayerIndex, Vec<u32>)> = Vec::new();
    for (slot, index) in slots {
        let player_id = slot.player_id(team);
        match mapped.iter_mut().find(|(known, _)| known == index) {
            Some((_, player_ids)) => player_ids.push(player_id),
            None => mapped.push((*index, vec![player_id])),
        }
    }
    let mut players: Vec<Option<PlayerFolder>> = players.into_iter().map(Some).collect();
    mapped
        .into_iter()
        .filter_map(|(index, player_ids)| {
            let folder = players.get_mut(index.0)?.take()?;
            Some((folder, player_ids))
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
    }
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::resolved;

    /// Each task as one line: export, team, what it compiles, charge.
    fn summary(report: &PlanReport) -> Vec<String> {
        report
            .manifest
            .tasks
            .iter()
            .map(|task| {
                let what = match &task.kind {
                    TaskKind::Face { folder, player_ids } => {
                        format!("face {} {player_ids:?}", folder.path.as_str())
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
            "da - Two",
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
                "0 702 face Players/Zed [70203] charge 15",
                "0 702 face Players/Amy [70207, 70209] charge 20",
                "0 702 kit p1 Kits/p1 charge 0",
                "0 702 kit p2 Kits/p2 - Away charge 8",
                "0 702 kit g1 Kits/g1 charge 10",
                "1 701 face Players/04 - B [70104] charge 1",
            ]
        );
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
        assert_eq!(
            messages,
            [("kit_config_generated", &scope, Disposition::Keep)]
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

        assert_eq!(summary(&report), ["3 701 kit g1 Kits/g1 charge 1"]);
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
