//! A run's event stream: what `check` and `compile` report to the shell or the console, export
//! by export, and the worst severity among it, which decides the command's exit code.

use studio_core::{
    ExportId, Message, PipelineEvent, PipelineEventEnvelope, RunId, Severity, ToolContext,
};

use crate::reader::ExportSource;

/// A command is one run per process, so every event carries the same run id.
const RUN_ID: RunId = RunId(1);

/// A run's event stream, and the worst severity among the messages it carried, which decides
/// the command's exit code.
pub(crate) struct RunEvents {
    ctx: ToolContext,
    worst: Option<Severity>,
}

impl RunEvents {
    /// Events sent through `ctx`, none yet.
    pub(crate) fn new(ctx: &ToolContext) -> RunEvents {
        // A clone shares the context's channels, so the events reach the same receivers.
        RunEvents {
            ctx: ctx.clone(),
            worst: None,
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
        self.emit(message.scope.export_id(), PipelineEvent::Message(message));
    }

    /// The export left the pipeline, whatever its outcome.
    pub(crate) fn processed(&self, export_id: ExportId) {
        self.emit(
            Some(export_id),
            PipelineEvent::ExportProcessed { export_id },
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
