//! The status bar's typed items (core plan, "Status bar"). Three slots, one item kind each; tools
//! cannot draw into the bar, they return a `ToolActivity` and publish `Notice`s. Conditions are
//! the shell's alone. There is no free-text status API, on purpose.

use std::path::PathBuf;
use std::time::Duration;

use crate::help::HelpTarget;

/// A persistent environment state the shell computes itself (left slot). Shown while it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCondition {
    /// Settings and the teams list cannot be written; the session keeps working in memory.
    DataDirReadOnly,
    /// A newer release exists.
    UpdateAvailable {
        /// The version available.
        version: String,
    },
    /// The configured PES folder does not contain the selected version.
    PesInstallMissing,
}

/// What a tool is doing in the background (middle slot), shown while a different tool's view is
/// on screen, and in the window title for every tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolActivity {
    /// The tool's label (`Team compiler`).
    pub tool_label: &'static str,
    /// `(done, total)` when the work has a countable size.
    pub progress: Option<(usize, usize)>,
    /// Time since the work started.
    pub elapsed: Duration,
}

/// A one-off event (right slot): text plus at most one action. Only the most recent is shown;
/// an event worth keeping is also a finding in a log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// What happened (`Compiled 48 teams`).
    pub text: String,
    /// The one thing the user can do about it.
    pub action: Option<NoticeAction>,
}

/// The clickable part of a notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeAction {
    /// Button label (`Open output`).
    pub label: String,
    /// What clicking does.
    pub effect: ActionEffect,
}

/// The closed set of things a notice action can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionEffect {
    /// Open a folder in the system file browser.
    OpenFolder(PathBuf),
    /// Open the settings menu.
    OpenSettings,
    /// Open the help window at a target.
    OpenHelp(HelpTarget),
    /// Make another tool the active view.
    SwitchTool(String),
}
