//! Stage 2, run planning (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! identity-resolved exports become one manifest of tasks, each an atomic unit that commits
//! whole or not at all.

use aesthetics_export::{
    ExportIdentity, FileDescriptor, KitFolder, PlayerFolder, PlayerIndex, ResolvedAestheticsExport,
    ValidatedRoster,
};
use kit_config::KitSlot;
use studio_core::{Disposition, ExportId, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};

/// What planning produced: the manifest and the findings planning itself made.
pub(crate) struct PlanReport {
    /// Every task of the run, in canonical order.
    pub(crate) manifest: BuildManifest,
    /// Planning's findings (`kit_config_generated`).
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

/// Plans the run over the identity-resolved exports, given in `ExportId` order.
pub(crate) fn plan_run(exports: Vec<(ExportId, ResolvedAestheticsExport)>) -> PlanReport {
    let mut tasks = Vec::new();
    let mut messages = Vec::new();
    for (export_id, resolved) in exports {
        // The referee export's slots and paths are Phase 4's; it plans no task yet.
        let ExportIdentity::Team { id, .. } = resolved.identity else {
            continue;
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
    use std::collections::BTreeMap;

    use aesthetics_export::{
        CanonicalListing, ListedEntry, ListedKind, SmallMetadata, ValidationContext, parse_listing,
    };
    use pes_version::PesVersion;
    use teams_list::TeamsList;

    use super::*;

    /// The export `name` with these files (path, size), folders and `players.txt`, validated
    /// for PES 21 and resolved against a teams list holding `701 /co/` and `702 /da/`.
    fn resolved(
        name: &str,
        files: &[(&str, u64)],
        folders: &[&str],
        players_txt: Option<&[u8]>,
    ) -> ResolvedAestheticsExport {
        let roster = players_txt.map(|bytes| ("players.txt", bytes.len() as u64));
        let files = files
            .iter()
            .copied()
            .chain(roster)
            .map(|(path, size)| (path.to_owned(), ListedKind::File { size }));
        let folders = folders
            .iter()
            .map(|path| ((*path).to_owned(), ListedKind::Folder));
        let listing = CanonicalListing {
            display_name: name.to_owned(),
            entries: files
                .chain(folders)
                .map(|(path, kind)| ListedEntry { path, kind })
                .collect(),
        };
        let metadata = SmallMetadata {
            files: players_txt
                .map(|bytes| ("players.txt".to_owned(), Ok(bytes.to_vec())))
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
        };
        let report = parse_listing(listing, metadata)
            .unwrap()
            .validate(&ValidationContext {
                version: PesVersion::Pes21,
                strict_file_type_check: true,
                pass_through: false,
            });
        assert_eq!(report.issues, [], "a clean export");
        let teams = TeamsList::parse("ID\tName\n701\t/co/\n702\t/da/\n").unwrap();
        report.validated.unwrap().resolve_identity(&teams).unwrap()
    }

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
                ("Players/Zed/hair.dds", 5),
                ("Players/Amy/face_high.fmdl", 20),
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
            &[("Players/04 - B/face_high.fmdl", 1)],
            &[],
            None,
        );

        let report = plan_run(vec![(ExportId(0), first), (ExportId(1), second)]);

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

        let report = plan_run(vec![(ExportId(3), export)]);

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
            &[("Players/04 - B/face_high.fmdl", 1)],
            &[],
            None,
        );
        let report = plan_run(vec![(ExportId(0), export)]);
        assert_eq!(
            report.manifest.tasks[0].kind.folder_path(),
            scope_path("Players/04 - B")
        );
    }

    #[test]
    fn a_referee_export_plans_no_task() {
        let export = resolved(
            "refs Cup",
            &[("Players/Keeper/face_high.fmdl", 1)],
            &[],
            Some(b"01 Keeper\n"),
        );
        assert!(
            plan_run(vec![(ExportId(0), export)])
                .manifest
                .tasks
                .is_empty()
        );
    }

    fn scope_path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }
}
