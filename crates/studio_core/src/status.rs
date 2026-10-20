//! The status bar's typed items (core plan, "Status bar"). Three slots, one item kind each; tools
//! cannot draw into the bar, they return a `ToolActivity` and publish `Notice`s. Conditions are
//! the shell's alone. There is no free-text status API, on purpose.

use std::path::PathBuf;
use std::time::Duration;

use pes_version::PesVersion;

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
}

impl ShellCondition {
    /// The `egui-phosphor` glyph shown beside the text (the draft's `CONDITION_DEFS`).
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            ShellCondition::DataDirReadOnly => egui_phosphor::regular::WARNING,
            ShellCondition::UpdateAvailable { .. } => egui_phosphor::regular::DOWNLOAD_SIMPLE,
        }
    }

    /// The bar's one-line text, and the line the CLI prints for the condition.
    pub fn text(&self) -> String {
        match self {
            ShellCondition::DataDirReadOnly => "Data folder is read-only".to_owned(),
            ShellCondition::UpdateAvailable { version } => {
                format!("Update available: v{version}")
            }
        }
    }

    /// The bar's tooltip while the condition holds.
    pub(crate) fn tooltip(&self) -> String {
        match self {
            ShellCondition::DataDirReadOnly => {
                "Settings and the teams list cannot be saved; changes last for this session"
                    .to_owned()
            }
            ShellCondition::UpdateAvailable { .. } => "A newer release is out".to_owned(),
        }
    }
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

impl ToolActivity {
    /// `Team compiler · 12/48 · 01:12`, or `Team compiler · 01:12` with no progress; the
    /// elapsed minutes are not capped at 59 (the draft's `activityText`). The status bar's
    /// middle slot and the window title share this line.
    pub(crate) fn text(&self) -> String {
        let elapsed = self.elapsed.as_secs();
        let clock = format!("{:02}:{:02}", elapsed / 60, elapsed % 60);
        match self.progress {
            Some((done, total)) => format!("{} · {done}/{total} · {clock}", self.tool_label),
            None => format!("{} · {clock}", self.tool_label),
        }
    }
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
    /// Rewrite `pes_folder_path` from the folder a running PES exe lives in (the shell's own
    /// action — no tool has a reason to publish it): the sidebar's auto-switch notice offers
    /// it when the running game sits outside the configured folder.
    FixPesPath {
        /// The running exe's folder.
        folder: PathBuf,
        /// The version the notice was raised for — the running game's. A pick can change the
        /// selection while the notice is up, so the click must not read it back.
        version: PesVersion,
    },
}
