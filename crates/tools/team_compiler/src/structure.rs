//! The structure pass over every export source of a run, which `check` reports and `compile`
//! starts from: each source's route, its issues and its identity (`team_compiler/README.md`
//! "Acceptance", Phase 3 scope). The lib validates; this schedules it, export by export.

use std::sync::Arc;

use aesthetics_export::{
    ExportIdentity, ResolvedAestheticsExport, SharedKind, SourceError, ValidationContext,
    parse_listing,
};
use pes_version::PesVersion;
use pipeline::MemoryBudget;
use studio_core::{Disposition, Message, Scope};

use crate::cli::RunInputs;
use crate::messages::{Code, issue_message, tool_message};
use crate::plan::ids::{SHARED_COUNT, shared_folders_taking_ids};
use crate::reader::{self, ExportSource, Route};

/// The structure pass's outcome for the whole run.
pub(crate) struct StructurePass {
    /// Findings about the run rather than one export (the duplicate-refs summary), reported
    /// before any export's.
    pub(crate) run_messages: Vec<Message>,
    /// Every source, in discovery order.
    pub(crate) sources: Vec<CheckedSource>,
}

/// One source after the structure pass.
pub(crate) struct CheckedSource {
    /// The source.
    pub(crate) source: ExportSource,
    /// Its findings: its route's when it was set aside, else its issues and its identity.
    pub(crate) messages: Vec<Message>,
    /// The export with its identity resolved; `None` when a finding dropped it.
    pub(crate) resolved: Option<ResolvedAestheticsExport>,
}

/// The run's memory budget, at the share of the available memory the settings give.
pub(crate) fn run_budget(inputs: &RunInputs) -> Arc<MemoryBudget> {
    MemoryBudget::new(pipeline::memory_cap(f64::from(
        inputs.common.memory_cap_percent,
    )))
}

/// Discovers the run's sources, routes them and runs the structure pass and identity on each
/// one headed for validation; the structure pass's `.7z` reads are charged to `budget`. An
/// exports folder holding no export is `no_exports_found`, on the run. Only an exports folder
/// that cannot be read is an error.
pub(crate) fn structure_pass(
    inputs: &RunInputs,
    budget: &Arc<MemoryBudget>,
) -> anyhow::Result<StructurePass> {
    let sources = reader::discover(&inputs.exports_root, &inputs.exports)?;
    let routes = reader::route(&sources, budget);

    let mut run_messages = Vec::new();
    // Every `--export` path yields a source, so no source at all means the root's scan found
    // none.
    if sources.is_empty() {
        run_messages.push(tool_message(
            Code::NoExportsFound,
            Scope::Run,
            Disposition::Keep,
            vec![("folder", inputs.exports_root.display().to_string())],
        ));
    }
    let conflicting: Vec<&str> = sources
        .iter()
        .zip(&routes)
        .filter(|(_, route)| matches!(route, Route::ConflictingRefs))
        .map(|(source, _)| source.file_name.as_str())
        .collect();
    if !conflicting.is_empty() {
        run_messages.push(tool_message(
            Code::MultipleRefExports,
            Scope::Run,
            Disposition::DropExport,
            vec![("exports", conflicting.join(", "))],
        ));
    }

    let sources = sources
        .into_iter()
        .zip(routes)
        .map(|(source, route)| check_source(inputs, source, route))
        .collect();
    Ok(StructurePass {
        run_messages,
        sources,
    })
}

/// One source through its route, the structure pass and identity.
fn check_source(inputs: &RunInputs, source: ExportSource, route: Route) -> CheckedSource {
    let export = Scope::Export {
        export_id: source.export_id,
    };
    // Each route's catalog consequence is "export skipped".
    let skipped = |source, code, context| CheckedSource {
        source,
        messages: vec![tool_message(
            code,
            export.clone(),
            Disposition::DropExport,
            context,
        )],
        resolved: None,
    };
    let (listing, metadata) = match route {
        Route::Unreadable(failure) => {
            return skipped(
                source,
                Code::ExportExtractFailed,
                vec![("path", failure.path), ("error", failure.error)],
            );
        }
        Route::Disabled => return skipped(source, Code::ExportDisabled, vec![]),
        Route::Balls => return skipped(source, Code::ExportBallsSkipped, vec![]),
        Route::ConflictingRefs => return skipped(source, Code::MultipleRefExports, vec![]),
        Route::Validate { listing, metadata } => (listing, metadata),
    };

    let parsed = match parse_listing(listing, metadata) {
        Ok(parsed) => parsed,
        Err(error) => {
            let (path, error) = match error {
                SourceError::Path { path, error } => (path, error.to_string()),
                SourceError::Collision { path, error } => (path, error.to_string()),
            };
            return skipped(
                source,
                Code::ExportExtractFailed,
                vec![("path", path), ("error", error)],
            );
        }
    };
    let strict = inputs.settings.strict_file_type_check;
    let report = parsed.validate(&ValidationContext {
        version: inputs.common.pes_version,
        strict_file_type_check: strict,
        pass_through: inputs.settings.pass_through,
    });
    let mut messages: Vec<Message> = report
        .issues
        .iter()
        .map(|issue| issue_message(issue, source.export_id, strict))
        .collect();
    // `None` means an issue dropped the export, and that issue is already among the messages.
    let resolved = report.validated.and_then(|validated| {
        match validated.resolve_identity(&inputs.teams_list) {
            Ok(resolved) => {
                messages.push(identified_message(export.clone(), &resolved.identity));
                let exhausted = pool_messages(&resolved, inputs.common.pes_version, &export);
                if exhausted.is_empty() {
                    Some(resolved)
                } else {
                    messages.extend(exhausted);
                    None
                }
            }
            Err(error) => {
                messages.push(tool_message(
                    Code::TeamNameUnknown,
                    export.clone(),
                    Disposition::DropExport,
                    vec![("team_name", error.team_name.as_str().to_owned())],
                ));
                None
            }
        }
    });
    CheckedSource {
        source,
        messages,
        resolved,
    }
}

/// `boots_id_pool_exhausted` and `gloves_id_pool_exhausted`, each when more shared folders of
/// its kind take an id for `version` than the team's block holds, naming the count; the export
/// is dropped (`team_compiler/README.md` TC-MOD-06). A referee export has no block: its ids
/// are its slots' (`k99NN`).
fn pool_messages(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
    export: &Scope,
) -> Vec<Message> {
    match resolved.identity {
        ExportIdentity::Team { .. } => {}
        ExportIdentity::Referees => return Vec::new(),
    }
    [
        (SharedKind::Boots, Code::BootsIdPoolExhausted),
        (SharedKind::Gloves, Code::GlovesIdPoolExhausted),
    ]
    .into_iter()
    .filter_map(|(kind, code)| {
        let count = shared_folders_taking_ids(&resolved.export, version.engine(), kind).len();
        (count > usize::from(SHARED_COUNT)).then(|| {
            tool_message(
                code,
                export.clone(),
                Disposition::DropExport,
                vec![("count", count.to_string())],
            )
        })
    })
    .collect()
}

/// `export_identified` naming the team and its id, or the referees.
fn identified_message(export: Scope, identity: &ExportIdentity) -> Message {
    let context = match identity {
        ExportIdentity::Team { id, name } => {
            vec![("team", name.as_str().to_owned()), ("id", id.to_string())]
        }
        ExportIdentity::Referees => vec![("team", "referees".to_owned())],
    };
    tool_message(Code::ExportIdentified, export, Disposition::Keep, context)
}

#[cfg(test)]
mod tests {
    use studio_core::{ExportId, Severity};

    use super::*;
    use crate::testing::resolved;

    /// The export `co - Pool` whose slots 01 to `count` each link their own shared folder of
    /// `kind` (`Boots/S01/` for slot 01), each player folder also holding `local`.
    fn linking_export(count: u8, kind: SharedKind, local: &[&str]) -> ResolvedAestheticsExport {
        let (extension, content_folder, model) = match kind {
            SharedKind::Face => ("face", "Faces", "face_high.fmdl"),
            SharedKind::Boots => ("boots", "Boots", "boots.fmdl"),
            SharedKind::Gloves => ("gloves", "Gloves", "glove_l.fmdl"),
        };
        let mut files = Vec::new();
        for slot in 1..=count {
            let player = format!("Players/{slot:02} - P{slot:02}");
            files.push(format!("{player}/S{slot:02}.{extension}"));
            for name in local {
                files.push(format!("{player}/{name}"));
            }
            files.push(format!("{content_folder}/S{slot:02}/{model}"));
        }
        let files: Vec<(&str, u64)> = files.iter().map(|path| (path.as_str(), 1)).collect();
        resolved("co - Pool", &files, &[], None)
    }

    /// `pool_messages` over `export` for `version`, each as (code, disposition, count).
    fn pool(
        export: &ResolvedAestheticsExport,
        version: PesVersion,
    ) -> Vec<(String, Disposition, String)> {
        let scope = Scope::Export {
            export_id: ExportId(0),
        };
        pool_messages(export, version, &scope)
            .into_iter()
            .map(|message| {
                assert_eq!(message.scope, scope);
                assert_eq!(message.severity, Severity::Error);
                let [(key, count)] = message.context.as_slice() else {
                    panic!("{:?}", message.context);
                };
                assert_eq!(key, "count");
                (
                    message.code.code.to_string(),
                    message.disposition,
                    count.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn eighteen_shared_gloves_folders_exhaust_the_pool_and_seventeen_fit() {
        assert_eq!(
            pool(
                &linking_export(18, SharedKind::Gloves, &[]),
                PesVersion::Pes21
            ),
            [(
                "gloves_id_pool_exhausted".to_owned(),
                Disposition::DropExport,
                "18".to_owned()
            )]
        );
        assert_eq!(
            pool(
                &linking_export(17, SharedKind::Gloves, &[]),
                PesVersion::Pes21
            ),
            []
        );
    }

    #[test]
    fn links_beside_local_boots_count_only_pre_fox() {
        // On Fox each player combines its link into its own package; pre-Fox has no package
        // of the player's own, so every shared folder takes an id.
        let export = linking_export(18, SharedKind::Boots, &["kit_boots.fmdl"]);
        assert_eq!(pool(&export, PesVersion::Pes21), []);
        assert_eq!(
            pool(&export, PesVersion::Pes17),
            [(
                "boots_id_pool_exhausted".to_owned(),
                Disposition::DropExport,
                "18".to_owned()
            )]
        );
    }
}
