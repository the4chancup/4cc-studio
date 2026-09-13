//! The platform: what runs with zero tools installed.
//!
//! `studio_core` holds the tool plugin trait, the settings framework, the pipeline event types
//! every tool emits, the status bar's typed items and the help chapter types. The module tree is
//! closed (core plan, "Crate structure"): format knowledge belongs in a lib crate, a widget used by
//! one tool in that tool's `view/`. This file only re-exports.

pub mod events;
pub mod help;
pub mod settings;
pub mod shell;
pub mod status;
pub mod tool;

pub use events::{
    Disposition, ExportId, ExportRevision, FolderStatus, Message, MessageCode, PipelineEvent,
    PipelineEventEnvelope, RunId, Scope, Severity,
};
pub use help::{HelpSection, HelpTarget, HelpTopic};
pub use settings::{COMMON_KEY, CommonSettings, Settings, SettingsError, Theme};
pub use shell::launch::{Launch, LaunchMode, parse_launch, run_cli};
pub use status::{ActionEffect, Notice, NoticeAction, ShellCondition, ToolActivity};
pub use tool::{ShellRequest, StudioTool, ToolContext};
