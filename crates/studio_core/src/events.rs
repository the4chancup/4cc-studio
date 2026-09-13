//! Pipeline events: what a tool's pipeline reports while it runs, and the structured findings
//! (`Message`) every tool emits instead of printing text.
//!
//! The CLI prints events as console lines; the GUI turns them into grid cells and log lines. Every
//! event travels in a `PipelineEventEnvelope`, whose run and revision ids let the GUI drop results
//! from a run or a filesystem snapshot that is no longer current (core plan, "Event system").

use std::borrow::Cow;

use vtree::ScopePath;

/// Identifies one pipeline run. Events from an older run are stale and are dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunId(pub u64);

/// Stable identity of one export (a folder or an archive) within a source set, independent of its
/// display name: two exports may share a display stem, and a rename must not confuse the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExportId(pub u64);

/// Revision of an export's on-disk contents; the folder watcher bumps it on every change.
/// Events carrying an older revision describe a snapshot that no longer exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExportRevision(pub u64);

/// A pipeline event with the identity needed to detect a stale result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineEventEnvelope {
    /// The run this event belongs to.
    pub run_id: RunId,
    /// The export this event is about, if any (run-wide events carry `None`).
    pub export_id: Option<ExportId>,
    /// The export revision the event was computed from, if any.
    pub export_revision: Option<ExportRevision>,
    /// The event itself.
    pub event: PipelineEvent,
}

/// What a pipeline reports while it runs. Tool-neutral: team names, team ids and other
/// tool-specific identity belong in tool state or in a `Message`'s context, not here.
///
/// `Progress` and `Complete` are placeholder shapes: the Team compiler plan extends them (run strip
/// metrics, structured compile/deployment outcomes) before the views that need them are built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineEvent {
    /// An export entered the pipeline; `display_name` is what the grid row shows.
    ExportStarted {
        /// The export.
        export_id: ExportId,
        /// Human-readable name for the row label.
        display_name: String,
    },
    /// An export left the pipeline, whatever its outcome.
    ExportProcessed {
        /// The export.
        export_id: ExportId,
    },
    /// A folder (player, kit, shared) inside an export changed state.
    FolderStatus {
        /// The export the folder belongs to.
        export_id: ExportId,
        /// The folder's canonical path within the export.
        path: ScopePath,
        /// Its new state.
        status: FolderStatus,
    },
    /// A structured finding for the user (a warning, an error, an info line).
    Message(Message),
    /// Coarse run progress: `processed` of `total` units done.
    Progress {
        /// Units finished so far.
        processed: usize,
        /// Units in the run.
        total: usize,
    },
    /// The run ended.
    Complete {
        /// Whether the run as a whole succeeded.
        success: bool,
        /// Files written to the output.
        files_written: usize,
        /// Folders skipped because of their findings.
        skipped_folders: usize,
        /// Duplicate entries dropped.
        duplicates: usize,
        /// Folders that ended with error-level findings.
        folders_with_errors: usize,
    },
    /// Content the tool could not place anywhere (the Export upgrader's leftovers).
    UnmigratedContentFound,
}

/// The state of one grid cell: a folder or slot during the check phase, then during compilation.
///
/// There is no "pending" compile state: every checked cell is implicitly pending once compilation
/// starts, and `Processing` keeps the check-phase color while bolding the label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FolderStatus {
    /// Not validated yet.
    Unchecked,
    /// Validation in progress.
    Checking,
    /// Shallow-checked, no issues found (an archive not yet extracted).
    PartialOk,
    /// Deep-checked, no issues.
    FullOk,
    /// Checked with warnings; does not block compilation.
    Warning,
    /// Shallow check found errors; the list may be incomplete.
    PartialError,
    /// Deep check found errors; the list is authoritative.
    Error,
    /// Being processed by the pipeline.
    Processing,
    /// Processed successfully.
    Done,
    /// Processed with warnings.
    DoneWithWarning,
    /// Written despite error-level findings (pass-through, or a dropped file whose folder was
    /// otherwise processed).
    DoneWithErrors,
}

/// Stable, cross-tool identity of a finding: the owning tool plus its code within that tool's
/// message catalog. The catalog in the tool's `messages.rs` maps the code to text, severity and
/// disposition; the help window's generated "Messages" topic lists every code, and a log line
/// links to it through the tool id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageCode {
    /// The tool whose catalog defines the code (`team-compiler`).
    pub tool_id: &'static str,
    /// The code within that catalog (`player_number_duplicate`); borrowed for catalog constants,
    /// owned when a code is built at run time.
    pub code: Cow<'static, str>,
}

/// A structured finding: stable code, how serious it is, what the pipeline did about it, what
/// it is about, and the context fields the display template fills in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Cross-tool stable identity.
    pub code: MessageCode,
    /// Presentation and filtering.
    pub severity: Severity,
    /// Processing consequence, independent of severity.
    pub disposition: Disposition,
    /// What the message is about; drives grid cell mapping.
    pub scope: Scope,
    /// Structured fields (file name, expected and found values, ...), in template order.
    pub context: Vec<(String, String)>,
}

/// How serious a finding is. Ordered: `Info < Warning < Error < Fatal`; the "errors" filter
/// includes `Fatal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    /// Informational.
    Info,
    /// Something worth knowing that did not stop processing.
    Warning,
    /// Something was dropped or written wrong.
    Error,
    /// The run cannot continue.
    Fatal,
}

/// What the pipeline did in response to a finding. A consequence, not a severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Disposition {
    /// Content kept as is.
    Keep,
    /// One file discarded.
    DropFile,
    /// One normalized roster assignment (a `players.txt` line) discarded.
    DropSlot,
    /// One folder discarded.
    DropFolder,
    /// The whole export discarded.
    DropExport,
    /// The run aborted.
    AbortRun,
}

/// What a finding is about. Identity travels as ids and canonical paths; human-readable names
/// stay in the message context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Not tied to an export (output stage, settings).
    Run,
    /// A whole export.
    Export {
        /// The export.
        export_id: ExportId,
    },
    /// A player, shared or kit folder inside an export.
    Folder {
        /// The export.
        export_id: ExportId,
        /// The folder's canonical path within the export.
        path: ScopePath,
    },
    /// One file inside an export.
    File {
        /// The export.
        export_id: ExportId,
        /// The file's canonical path within the export.
        path: ScopePath,
    },
    /// One line of a roster file (`players.txt`).
    RosterEntry {
        /// The export.
        export_id: ExportId,
        /// The roster file.
        file: ScopePath,
        /// 1-based line number.
        line: usize,
        /// The slot the line assigns, when it could be read.
        slot: Option<u16>,
    },
}

impl Scope {
    /// The export this scope belongs to, if it is not run-wide.
    pub fn export_id(&self) -> Option<ExportId> {
        match self {
            Scope::Run => None,
            Scope::Export { export_id }
            | Scope::Folder { export_id, .. }
            | Scope::File { export_id, .. }
            | Scope::RosterEntry { export_id, .. } => Some(*export_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_info_below_fatal() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn same_code_in_two_tools_is_two_identities() {
        let compiler = MessageCode {
            tool_id: "team-compiler",
            code: Cow::Borrowed("bad_file"),
        };
        let upgrader = MessageCode {
            tool_id: "export-upgrader",
            code: Cow::Borrowed("bad_file"),
        };
        assert_ne!(compiler, upgrader);
        assert_eq!(
            compiler,
            MessageCode {
                tool_id: "team-compiler",
                code: Cow::Owned("bad_file".to_owned())
            }
        );
    }

    #[test]
    fn scope_reports_its_export() {
        let path = ScopePath::new("Kits/p1").unwrap();
        assert_eq!(Scope::Run.export_id(), None);
        assert_eq!(
            Scope::Folder {
                export_id: ExportId(7),
                path
            }
            .export_id(),
            Some(ExportId(7))
        );
    }
}
