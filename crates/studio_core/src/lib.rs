//! The platform: what runs with zero tools installed.
//!
//! `studio_core` holds the tool plugin trait, the settings framework, the pipeline event types
//! every tool emits, the status bar's typed items and the help chapter types. The module tree is
//! closed (core plan, "Crate structure"): format knowledge belongs in a lib crate, a widget used by
//! one tool in that tool's `view/`. This file only re-exports.

pub mod events;
pub mod help;
pub mod pes_process;
pub mod settings;
pub mod shell;
pub mod status;
mod theme;
pub mod tool;

// The icon font is the sidebar's rail alphabet: `StudioTool::icon` and `PlannedTool::icon`
// name its glyphs, so the tool crates and `studio`'s registry reach the constants through this
// re-export instead of each depending on the font crate.
pub use egui_phosphor;
pub use events::{
    Disposition, EventLines, ExportId, ExportRevision, FolderStatus, Message, MessageCode,
    PipelineEvent, PipelineEventEnvelope, RunId, Scope, Severity,
};
pub use help::{
    CatalogRow, HelpSection, HelpTarget, HelpTopic, chapter_markdown, chapters, run_help,
    topic_markdown,
};
pub use pes_process::{RunningPes, running_pes};
pub use settings::{COMMON_KEY, CommonSettings, Settings, SettingsError, Theme};
#[cfg(not(target_arch = "wasm32"))]
pub use settings::{SETTINGS_FILE_NAME, resolve_data_dir, settings_file_writable, user_config_dir};
#[cfg(not(target_arch = "wasm32"))]
pub use shell::run_gui;
pub use shell::{Launch, LaunchMode, StudioApp, parse_launch, run_cli};
pub use status::{ActionEffect, Notice, NoticeAction, ShellCondition, ToolActivity};
pub use tool::{AppPaths, CliError, PlannedTool, ShellRequest, StudioTool, ToolContext};
