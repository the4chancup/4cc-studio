//! A run's event stream: what `check` and `compile` report to the shell or the console, export
//! by export, and the worst severity among it, which decides the command's exit code.

use std::collections::BTreeSet;

use studio_core::{
    Disposition, ExportId, Message, PipelineEvent, PipelineEventEnvelope, RunId, Scope, Severity,
    ToolContext,
};
use vtree::ScopePath;

use crate::messages::Code;
use crate::reader::ExportSource;

/// A command is one run per process, so every event carries the same run id.
const RUN_ID: RunId = RunId(1);

/// A run's event stream, and the worst severity among the messages it carried, which decides
/// the command's exit code.
pub(crate) struct RunEvents {
    ctx: ToolContext,
    worst: Option<Severity>,
    /// The folders a finding's verdict dropped (`DropFolder`), for `Complete`'s summary.
    skipped_folders: usize,
    /// The entries an `overrides/` file replaced (`duplicate_path`), for `Complete`'s summary.
    duplicates: usize,
    /// The folders that reported an error-level finding, by their canonical path inside their
    /// export: one folder counts once however many findings it has.
    folders_with_errors: BTreeSet<(ExportId, ScopePath)>,
}

impl RunEvents {
    /// Events sent through `ctx`, none yet.
    pub(crate) fn new(ctx: &ToolContext) -> RunEvents {
        // A clone shares the context's channels, so the events reach the same receivers.
        RunEvents {
            ctx: ctx.clone(),
            worst: None,
            skipped_folders: 0,
            duplicates: 0,
            folders_with_errors: BTreeSet::new(),
        }
    }

    /// `source` entered the pipeline.
    pub(crate) fn started(&self, source: &ExportSource) {
        // The console names the export by its file name, extension included, so a folder and
        // an archive with one stem stay apart in every line (TC-SRC-02).
        self.emit(
            Some(source.export_id),
            PipelineEvent::ExportStarted {
                export_id: source.export_id,
                display_name: source.file_name.clone(),
            },
        );
    }

    /// One finding, addressed to the export its scope names.
    pub(crate) fn message(&mut self, message: Message) {
        self.worst = self.worst.max(Some(message.severity));
        if message.disposition == Disposition::DropFolder {
            self.skipped_folders += 1;
        }
        if message.code.code.as_ref() == Code::DuplicatePath.as_str() {
            self.duplicates += 1;
        }
        // A file's error is its folder's too; a roster line's or an export's counts no folder.
        if message.severity >= Severity::Error {
            let folder = match &message.scope {
                Scope::Folder { export_id, path } => Some((*export_id, path.clone())),
                Scope::File { export_id, path } => path.parent().map(|parent| (*export_id, parent)),
                Scope::Run | Scope::Export { .. } | Scope::RosterEntry { .. } => None,
            };
            if let Some(folder) = folder {
                self.folders_with_errors.insert(folder);
            }
        }
        self.emit(message.scope.export_id(), PipelineEvent::Message(message));
    }

    /// The export left the pipeline, whatever its outcome.
    pub(crate) fn processed(&self, export_id: ExportId) {
        self.emit(
            Some(export_id),
            PipelineEvent::ExportProcessed { export_id },
        );
    }

    /// `processed` of the manifest's `total` tasks committed (`Progress`, the console's parts
    /// counter). Run-wide: a task is the run's work, no export's.
    pub(crate) fn progress(&self, processed: usize, total: usize) {
        self.emit(None, PipelineEvent::Progress { processed, total });
    }

    /// The run ended after compiling (`Complete`): `files_written` entries in the output, the
    /// counts the findings kept, and `success` when nothing error-level was reported.
    pub(crate) fn complete(&self, files_written: usize) {
        self.emit(
            None,
            PipelineEvent::Complete {
                success: self.worst.is_none_or(|worst| worst < Severity::Error),
                files_written,
                skipped_folders: self.skipped_folders,
                duplicates: self.duplicates,
                folders_with_errors: self.folders_with_errors.len(),
            },
        );
    }

    /// The worst severity reported so far; `None` when no message was.
    pub(crate) fn worst(&self) -> Option<Severity> {
        self.worst
    }

    fn emit(&self, export_id: Option<ExportId>, event: PipelineEvent) {
        self.ctx.emit(PipelineEventEnvelope {
            run_id: RUN_ID,
            export_id,
            export_revision: None,
            event,
        });
    }
}
