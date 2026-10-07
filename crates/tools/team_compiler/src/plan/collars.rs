//! Which team export keeps which collar (`team_compiler/pipeline.md` "Collars", "Resolved
//! decisions", "Collar contract"): a stock collar is replaced by one export of the run, and an
//! export holds one collar, since its kit configs take one ID.

use std::collections::BTreeMap;

use aesthetics_export::FileDescriptor;
use studio_core::{Disposition, ExportId, Message, Scope};

use super::subset::{CollarFile, collar_file, file_stem};
use crate::deep::collar::named_id;
use crate::messages::{Code, tool_message};

/// The collar the export `export_id`, named `export_name`, keeps of its `Collars/` files
/// `collars`, with the stock collar it replaces, claimed in `claimed` (each claimed collar's ID
/// and its claimant's name, run-wide). Its FMDLs are taken in path order: the first whose ID
/// no earlier export claimed claims it; every other, its ID claimed already or the export
/// holding a collar already (its own same ID included), reports `collar_id_conflict` on its
/// file, naming the claimant, and is left out. A file of another kind claims nothing
/// (`collar_file`). `claimed` starts empty: the suite's own collars, 105 and 77, never get
/// here, the deep pass having dropped a file named for either as a conflict.
pub(crate) fn export_collar(
    export_id: ExportId,
    export_name: &str,
    collars: &[FileDescriptor],
    claimed: &mut BTreeMap<u8, String>,
    messages: &mut Vec<Message>,
) -> Option<(FileDescriptor, u8)> {
    let mut kept: Option<(FileDescriptor, u8)> = None;
    // The structure pass lists a folder's files in path order.
    for file in collars {
        match collar_file(file) {
            CollarFile::Compiled => {}
            CollarFile::NotCompiled | CollarFile::PassedOver => continue,
        }
        let name = file.path.name();
        let id = named_id(file_stem(name))
            .expect("the deep pass drops a collar file whose name gives no collar ID");
        let claimant = match claimed.get(&id) {
            Some(claimant) => claimant.clone(),
            None if kept.is_some() => export_name.to_owned(),
            None => {
                claimed.insert(id, export_name.to_owned());
                kept = Some((file.clone(), id));
                continue;
            }
        };
        messages.push(tool_message(
            Code::CollarIdConflict,
            Scope::File {
                export_id,
                path: file.path.clone(),
            },
            Disposition::DropFile,
            vec![("file", name.to_owned()), ("claimant", claimant)],
        ));
    }
    kept
}

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;
    use studio_core::Severity;
    use vtree::ScopePath;

    use super::*;
    use crate::plan::{PlanReport, TaskKind, plan_run};
    use crate::testing::{resolved, two_team_colors};

    /// The run planned for PES 21 over the exports `exports` (name, files), in that order, each
    /// with two team colors, every file one byte.
    fn planned(exports: &[(&str, &[&str])]) -> PlanReport {
        let exports = exports
            .iter()
            .zip(0..)
            .map(|((name, files), index)| {
                let files: Vec<(&str, u64)> = files.iter().map(|path| (*path, 1)).collect();
                (
                    ExportId(index),
                    resolved(name, &files, &[], None),
                    two_team_colors(),
                    None,
                )
            })
            .collect();
        plan_run(exports, PesVersion::Pes21)
    }

    /// Each collar task of `report` as (export, file path, ID).
    fn collar_tasks(report: &PlanReport) -> Vec<(u64, &str, u8)> {
        report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Collar { file, id } => Some((task.export_id.0, file.path.as_str(), *id)),
                TaskKind::Models { .. }
                | TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. } => None,
            })
            .collect()
    }

    /// The collar each kit task of `report` gives its config, as (export, slot, collar).
    fn kit_collars(report: &PlanReport) -> Vec<(u64, &str, Option<u8>)> {
        report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Kit { slot, edits, .. } => {
                    Some((task.export_id.0, slot.as_str(), edits.collar))
                }
                TaskKind::Models { .. }
                | TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
            .collect()
    }

    /// `report`'s `collar_id_conflict` findings, each an Error dropping its file, as one line:
    /// the export, the file's path and the context (`0 Collars/x.fmdl file=x.fmdl claimant=y`).
    fn conflicts(report: &PlanReport) -> Vec<String> {
        report
            .messages
            .iter()
            .filter(|message| message.code.code == "collar_id_conflict")
            .map(|message| {
                assert_eq!(
                    (message.severity, message.disposition),
                    (Severity::Error, Disposition::DropFile)
                );
                let Scope::File { export_id, path } = &message.scope else {
                    panic!("{:?}", message.scope);
                };
                let context: Vec<String> = message
                    .context
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect();
                format!("{} {} {}", export_id.0, path.as_str(), context.join(" "))
            })
            .collect()
    }

    #[test]
    fn of_two_exports_claiming_one_collar_the_first_keeps_it_and_the_second_s_kits_lose_it() {
        let report = planned(&[
            (
                "a Midcup Collars",
                &["Collars/collar_12.fmdl", "Kits/p1/kit.dds"],
            ),
            (
                "co Midcup Collars",
                &["Collars/collar_12.fmdl", "Kits/p1/kit.dds"],
            ),
        ]);

        assert_eq!(collar_tasks(&report), [(0, "Collars/collar_12.fmdl", 12)]);
        assert_eq!(
            conflicts(&report),
            ["1 Collars/collar_12.fmdl file=collar_12.fmdl claimant=a Midcup Collars"]
        );
        assert_eq!(kit_collars(&report), [(0, "p1", Some(12)), (1, "p1", None)]);
        // The absent slots of each team get what its kits get.
        let collars: Vec<Option<u8>> = report
            .manifest
            .team_kits
            .iter()
            .map(|team| team.edits.collar)
            .collect();
        assert_eq!(collars, [Some(12), None]);
    }

    #[test]
    fn an_export_holds_one_collar_and_its_second_is_a_conflict_naming_itself() {
        // Path order puts `collar_012` first: it claims 12, and `collar_12` names the same
        // collar. `collar_13`, unclaimed, is still the export's second.
        let report = planned(&[(
            "co Midcup Collars",
            &[
                "Collars/collar_12.fmdl",
                "Collars/collar_012.fmdl",
                "Collars/collar_13.fmdl",
                "Kits/p1/kit.dds",
            ],
        )]);

        assert_eq!(collar_tasks(&report), [(0, "Collars/collar_012.fmdl", 12)]);
        assert_eq!(
            conflicts(&report),
            [
                "0 Collars/collar_12.fmdl file=collar_12.fmdl claimant=co Midcup Collars",
                "0 Collars/collar_13.fmdl file=collar_13.fmdl claimant=co Midcup Collars",
            ]
        );
        assert_eq!(kit_collars(&report), [(0, "p1", Some(12))]);
    }

    #[test]
    fn a_collar_task_comes_last_in_its_export_s_range_after_the_logo() {
        let report = planned(&[(
            "co Midcup Collars",
            &["Collars/collar_12.fmdl", "Kits/p1/kit.dds", "logo.png"],
        )]);

        let kinds: Vec<&str> = report
            .manifest
            .tasks
            .iter()
            .map(|task| match &task.kind {
                TaskKind::Kit { .. } => "kit",
                TaskKind::Logo { .. } => "logo",
                TaskKind::Collar { .. } => "collar",
                TaskKind::Models { .. }
                | TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::RefereeMarker { .. } => "other",
            })
            .collect();
        assert_eq!(kinds, ["kit", "logo", "collar"]);
    }

    #[test]
    fn a_file_of_another_kind_in_collars_claims_nothing_and_reports_nothing() {
        // Kept by the structure pass only with the strict file-type check off, which the
        // planning tests' validation has on: added as that pass would keep it.
        let mut export = resolved("co Midcup Collars", &[("Kits/p1/kit.dds", 1)], &[], None);
        let path = ScopePath::new("Collars/collar_12.dds").unwrap();
        export.export.collars.push(FileDescriptor {
            size: 1,
            kind: aesthetics_export::classify(path.name()),
            source: path.clone(),
            path,
        });

        let report = plan_run(
            vec![(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(collar_tasks(&report), []);
        assert_eq!(conflicts(&report), Vec::<String>::new());
        assert_eq!(kit_collars(&report), [(0, "p1", None)]);
    }
}
