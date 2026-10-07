//! The compiled players' rows of `BootsList.bin` and `GloveList.bin` (`team_compiler/pipeline.md`
//! "Bins accumulation"), decided at planning from the `Models` tasks: a player whose boots or
//! gloves a task builds is pointed at the ID the task emits them under once it commits, and a
//! `Full` export's player with none loses his installed row.

use aesthetics_export::ExportCoverage;
use vtree::ScopePath;

use super::subset::ModelPackage;
use super::{BuildTask, TaskKind};
use crate::bins::player_tables::ItemTable;

/// One compiled player's row of `BootsList.bin` or `GloveList.bin`, as planning decides it
/// ("Bins accumulation").
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ItemRow {
    /// The table the row is in.
    pub(crate) table: ItemTable,
    /// The player's id: his team ID times 100 plus his roster slot (71405).
    pub(crate) player_id: u32,
    /// What happens to his row.
    pub(crate) change: RowChange,
}

/// What a compile does to a player's row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowChange {
    /// The row becomes (player id, `id`) when manifest task `task` commits, the task that puts
    /// the item under `id` into the CPK; else the installed row stays.
    Set { id: u32, task: usize },
    /// A `Full` export's compiled player with no item of the table: the installed row goes.
    Remove,
}

/// A compiled player folder as its rows need it.
pub(super) struct RowPlayer {
    /// The folder's export path, which its own `Models` tasks compile.
    pub(super) path: ScopePath,
    /// The player id of each roster slot mapping the folder, in slot order: the order of the
    /// ids its own `Models` tasks emit their package under.
    pub(super) player_ids: Vec<u32>,
    /// The export paths of the shared folders the player folder links.
    pub(super) linked: Vec<ScopePath>,
}

/// The rows of `players`, the compiled player folders of one export whose `coverage` is given,
/// against `tasks`, the manifest so far, whose export's tasks start at `first`. A player's
/// boots come from his folder's own boots package, emitted under each of his slots' exclusive
/// ids, or else from the package of a shared boots folder he links, emitted under its shared
/// id: `Set` with that task. A link that combines makes the shared folder a part of his own
/// package, so his own task is the one. With neither, his row is removed for a `Full` export
/// and left alone for a `Midcup` one. Gloves alike.
pub(super) fn export_rows(
    tasks: &[BuildTask],
    first: usize,
    players: &[RowPlayer],
    coverage: ExportCoverage,
) -> Vec<ItemRow> {
    let mut rows = Vec::new();
    for player in players {
        for table in ItemTable::ALL {
            let package = package(table);
            let own = models_task(tasks, first, &player.path, package)
                .map(|(task, ids)| (task, ids.to_vec()));
            let linked = || {
                player.linked.iter().find_map(|path| {
                    let (task, ids) = models_task(tasks, first, path, package)?;
                    let id = *ids
                        .first()
                        .expect("a shared folder's package is emitted under its one shared id");
                    Some((task, vec![id; player.player_ids.len()]))
                })
            };
            match own.or_else(linked) {
                Some((task, ids)) => rows.extend(player.player_ids.iter().zip(ids).map(
                    |(player_id, id)| ItemRow {
                        table,
                        player_id: *player_id,
                        change: RowChange::Set { id, task },
                    },
                )),
                None => match coverage {
                    ExportCoverage::Full => {
                        rows.extend(player.player_ids.iter().map(|player_id| ItemRow {
                            table,
                            player_id: *player_id,
                            change: RowChange::Remove,
                        }))
                    }
                    ExportCoverage::Midcup => {}
                },
            }
        }
    }
    rows
}

/// The package whose IDs `table` lists.
fn package(table: ItemTable) -> ModelPackage {
    match table {
        ItemTable::Boots => ModelPackage::Boots,
        ItemTable::Gloves => ModelPackage::Gloves,
    }
}

/// The manifest position and ids of the `Models` task of `tasks`, from position `first` on,
/// that builds `package` of the folder at `path`; `None` when no task does (the folder holds
/// no model of it).
fn models_task<'a>(
    tasks: &'a [BuildTask],
    first: usize,
    path: &ScopePath,
    package: ModelPackage,
) -> Option<(usize, &'a [u32])> {
    tasks
        .iter()
        .enumerate()
        .skip(first)
        .find_map(|(index, task)| {
            if let TaskKind::Models {
                folder,
                package: built,
                ids,
            } = &task.kind
                && folder.path == *path
                && *built == package
            {
                Some((index, ids.as_slice()))
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;
    use studio_core::ExportId;

    use super::*;
    use crate::plan::plan_run;
    use crate::testing::{resolved, two_team_colors};

    /// A `Models` task as the tests compare it: its manifest position, package and ids.
    type PlannedModels = (usize, ModelPackage, Vec<u32>);

    /// The `/co/` export `<coverage> Rows`, planned for PES 21: slot 05 holds boots of its own
    /// and links the shared gloves `Keeper`, slot 06 holds only a face, slot 07 holds boots of
    /// its own and links the shared boots `Crocs`, which combine with them. Its rows, and its
    /// `Models` tasks.
    fn planned(coverage: &str) -> (Vec<ItemRow>, Vec<PlannedModels>) {
        let export = resolved(
            &format!("co {coverage} Rows"),
            &[
                ("Players/05 - A/boots.fmdl", 1),
                ("Players/05 - A/Keeper.gloves", 0),
                ("Players/06 - B/face_high.fmdl", 1),
                ("Players/07 - C/kit_boots.fmdl", 1),
                ("Players/07 - C/Crocs.boots", 0),
                ("Gloves/Keeper/glove_l.fmdl", 1),
                ("Boots/Crocs/boots.fmdl", 1),
            ],
            &[],
            None,
        );
        let report = plan_run(
            vec![(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        let models = report
            .manifest
            .tasks
            .iter()
            .enumerate()
            .filter_map(|(index, task)| match &task.kind {
                TaskKind::Models { package, ids, .. } => Some((index, *package, ids.clone())),
                TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. } => None,
            })
            .collect();
        (report.manifest.item_rows, models)
    }

    /// The row setting `player_id`'s item of `table` to `id` once task `task` commits.
    fn set(table: ItemTable, player_id: u32, id: u32, task: usize) -> ItemRow {
        ItemRow {
            table,
            player_id,
            change: RowChange::Set { id, task },
        }
    }

    /// The row removing `player_id`'s item of `table`.
    fn remove(table: ItemTable, player_id: u32) -> ItemRow {
        ItemRow {
            table,
            player_id,
            change: RowChange::Remove,
        }
    }

    /// The tasks `planned` gives both exports: the three faces, slot 05's and slot 07's boots
    /// under their exclusive ids (07's with `Crocs` combined in, so `Crocs` takes no id), then
    /// `Keeper`'s gloves under the first shared id.
    fn expected_models() -> Vec<PlannedModels> {
        vec![
            (0, ModelPackage::Face, vec![71405]),
            (1, ModelPackage::Boots, vec![625]),
            (2, ModelPackage::Face, vec![71406]),
            (3, ModelPackage::Face, vec![71407]),
            (4, ModelPackage::Boots, vec![627]),
            (5, ModelPackage::Gloves, vec![644]),
        ]
    }

    #[test]
    fn a_full_export_sets_its_players_rows_from_their_tasks_and_removes_the_rest() {
        let (rows, models) = planned("Full");
        assert_eq!(models, expected_models());
        assert_eq!(
            rows,
            [
                set(ItemTable::Boots, 71405, 625, 1),
                set(ItemTable::Gloves, 71405, 644, 5),
                remove(ItemTable::Boots, 71406),
                remove(ItemTable::Gloves, 71406),
                set(ItemTable::Boots, 71407, 627, 4),
                remove(ItemTable::Gloves, 71407),
            ]
        );
    }

    #[test]
    fn a_midcup_export_sets_its_players_rows_and_leaves_the_rest_alone() {
        let (rows, models) = planned("Midcup");
        assert_eq!(models, expected_models());
        assert_eq!(
            rows,
            [
                set(ItemTable::Boots, 71405, 625, 1),
                set(ItemTable::Gloves, 71405, 644, 5),
                set(ItemTable::Boots, 71407, 627, 4),
            ]
        );
    }

    #[test]
    fn a_folder_two_slots_map_gives_each_slot_its_own_row() {
        let export = resolved(
            "co Midcup Twice",
            &[
                ("Players/A/boots.fmdl", 1),
                ("Players/B/Keeper.gloves", 0),
                ("Gloves/Keeper/glove_l.fmdl", 1),
            ],
            &[],
            Some(b"03 A\n08 A\n09 B\n11 B\n"),
        );
        let report = plan_run(
            vec![(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        // A's face (0) and boots (1) under slots 03 and 08; B's face (2); Keeper's gloves (3).
        assert_eq!(
            report.manifest.item_rows,
            [
                set(ItemTable::Boots, 71403, 623, 1),
                set(ItemTable::Boots, 71408, 628, 1),
                set(ItemTable::Gloves, 71409, 644, 3),
                set(ItemTable::Gloves, 71411, 644, 3),
            ]
        );
    }
}
