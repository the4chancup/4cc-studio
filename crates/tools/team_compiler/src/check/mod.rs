//! The `check` command: the structure pass over every export source, reported as events. The
//! lib validates; this schedules it, export by export, and reports each source's route, its
//! issues and its identity (`team_compiler/README.md` "Acceptance", Phase 3 scope).

use aesthetics_export::{
    ExportIdentity, IdentityError, ResolvedAestheticsExport, SourceError, ValidationContext,
    parse_listing,
};
use studio_core::{
    CliError, Disposition, ExportId, Message, PipelineEvent, PipelineEventEnvelope, RunId, Scope,
    Severity, ToolContext,
};

use crate::cli::{ABORTED, CLEAN, ERRORS, RunInputs};
use crate::messages::{issue_message, tool_message};
use crate::reader::{self, ExportSource, Route};

/// `check` is one run per process, so every event carries the same run id.
const RUN_ID: RunId = RunId(1);

/// Checks every export source of the run and returns the exit code, which the worst finding
/// decides. Only an exports folder that cannot be read refuses the command.
pub(crate) fn run(inputs: &RunInputs, ctx: &ToolContext) -> Result<u8, CliError> {
    let sources = reader::discover(&inputs.exports_root, &inputs.exports)
        .map_err(|error| CliError::new(ABORTED, error))?;
    let routes = reader::route(&sources);
    let mut worst = None;

    // The duplicate-refs summary is a discovery finding about the run, so it comes first.
    let conflicting: Vec<&str> = sources
        .iter()
        .zip(&routes)
        .filter(|(_, route)| matches!(route, Route::ConflictingRefs))
        .map(|(source, _)| source.file_name.as_str())
        .collect();
    if !conflicting.is_empty() {
        let summary = tool_message(
            "multiple_ref_exports",
            Scope::Run,
            Disposition::DropExport,
            vec![("exports", conflicting.join(", "))],
        );
        worst = worst.max(Some(summary.severity));
        emit(ctx, None, PipelineEvent::Message(summary));
    }

    for (source, route) in sources.iter().zip(routes) {
        let export_id = Some(source.export_id);
        emit(
            ctx,
            export_id,
            PipelineEvent::ExportStarted {
                export_id: source.export_id,
                display_name: source.file_name.clone(),
            },
        );
        for message in check_source(inputs, source, route) {
            worst = worst.max(Some(message.severity));
            emit(ctx, export_id, PipelineEvent::Message(message));
        }
        emit(
            ctx,
            export_id,
            PipelineEvent::ExportProcessed {
                export_id: source.export_id,
            },
        );
    }
    Ok(exit_code(worst))
}

/// One source's findings: its route's when it is set aside, else its structure pass's and its
/// identity.
fn check_source(inputs: &RunInputs, source: &ExportSource, route: Route) -> Vec<Message> {
    let export = Scope::Export {
        export_id: source.export_id,
    };
    // Each route's catalog consequence is "export skipped".
    let skipped = |code, context| {
        vec![tool_message(
            code,
            export.clone(),
            Disposition::DropExport,
            context,
        )]
    };
    let listing = match route {
        Route::Unreadable(failure) => {
            return skipped(
                "export_extract_failed",
                vec![("path", failure.path), ("error", failure.error)],
            );
        }
        Route::Disabled => return skipped("export_disabled", vec![]),
        Route::Balls => return skipped("export_balls_skipped", vec![]),
        Route::ConflictingRefs => return skipped("multiple_ref_exports", vec![]),
        Route::Validate(listing) => listing,
    };

    let metadata = reader::read_metadata(source, &listing);
    let parsed = match parse_listing(listing, metadata) {
        Ok(parsed) => parsed,
        Err(error) => {
            let (path, error) = match error {
                SourceError::Path { path, error } => (path, error.to_string()),
                SourceError::Collision { path, error } => (path, error.to_string()),
            };
            return skipped(
                "export_extract_failed",
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
    if let Some(validated) = report.validated {
        messages.push(identity_message(
            export,
            validated.resolve_identity(&inputs.teams_list),
        ));
    }
    messages
}

/// `export_identified` naming the team and its id (or the referees), else `team_name_unknown`.
fn identity_message(
    export: Scope,
    identity: Result<ResolvedAestheticsExport, IdentityError>,
) -> Message {
    match identity.map(|resolved| resolved.identity) {
        Ok(ExportIdentity::Team { id, name }) => tool_message(
            "export_identified",
            export,
            Disposition::Keep,
            vec![("team", name.as_str().to_owned()), ("id", id.to_string())],
        ),
        Ok(ExportIdentity::Referees) => tool_message(
            "export_identified",
            export,
            Disposition::Keep,
            vec![("team", "referees".to_owned())],
        ),
        Err(error) => tool_message(
            "team_name_unknown",
            export,
            Disposition::DropExport,
            vec![("team_name", error.team_name.as_str().to_owned())],
        ),
    }
}

fn emit(ctx: &ToolContext, export_id: Option<ExportId>, event: PipelineEvent) {
    ctx.emit(PipelineEventEnvelope {
        run_id: RUN_ID,
        export_id,
        export_revision: None,
        event,
    });
}

/// `settings.md` "CLI": 1 when some export had an Error, 3 when the run hit a Fatal finding,
/// else 0, warnings and notes included.
fn exit_code(worst: Option<Severity>) -> u8 {
    match worst {
        Some(Severity::Fatal) => ABORTED,
        Some(Severity::Error) => ERRORS,
        Some(Severity::Warning | Severity::Info) | None => CLEAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worst_severity_decides_the_exit_code() {
        assert_eq!(exit_code(None), 0);
        assert_eq!(exit_code(Some(Severity::Info)), 0);
        assert_eq!(exit_code(Some(Severity::Warning)), 0);
        assert_eq!(exit_code(Some(Severity::Error)), 1);
        assert_eq!(exit_code(Some(Severity::Fatal)), 3);
    }
}
