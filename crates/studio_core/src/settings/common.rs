//! The common settings: what every tool shares (core plan, "Settings menu"; Team compiler plan,
//! "Common settings"). Defaults are Studio's, not Red's, where the plan says so.

use std::path::PathBuf;
use std::thread::available_parallelism;

use pes_version::PesVersion;
use serde::{Deserialize, Serialize};

/// Red's default PES folder, with `**` standing for the two-digit version.
#[cfg(windows)]
const DEFAULT_PES_FOLDER: &str = r"C:\Program Files (x86)\Pro Evolution Soccer 20**";
/// No sensible default exists off Windows; the user sets it.
#[cfg(not(windows))]
const DEFAULT_PES_FOLDER: &str = "";

/// Settings shared by the shell and every tool, stored under `[common]` in the settings file.
/// Every field has a default, so a missing key loads as its default (this is how a new version's
/// settings appear in an old file).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommonSettings {
    /// The PES version the suite targets; the sidebar selector.
    pub pes_version: PesVersion,
    /// The PES installation folder, with `**` standing for the two-digit version (`20**`).
    pub pes_folder_path: String,
    /// Where exports are picked up. A relative path resolves beside the executable in both
    /// data-location modes, so it sits next to `quick_compile.bat`.
    pub exports_folder_path: PathBuf,
    /// Worker threads for pipelines; `0` means automatic (logical cores minus one).
    pub thread_count: usize,
    /// Memory budget cap for pipelines, as a percentage of physical memory.
    pub memory_cap_percent: f32,
    /// GUI color theme.
    pub theme: Theme,
    /// Whether the shell looks for new releases.
    pub check_for_updates: bool,
    /// Hours between release checks.
    pub check_interval_hours: u32,
    /// When the last release check ran, in seconds since the Unix epoch.
    pub last_update_check: Option<u64>,
    /// A release the user chose to skip; the updater stays quiet about it.
    pub skipped_version: Option<String>,
    /// The tool active when the GUI last closed; `studio` with no arguments reopens it.
    pub last_tool: Option<String>,
}

impl Default for CommonSettings {
    fn default() -> Self {
        CommonSettings {
            pes_version: PesVersion::Pes19,
            pes_folder_path: DEFAULT_PES_FOLDER.to_owned(),
            exports_folder_path: PathBuf::from("exports"),
            thread_count: 0,
            memory_cap_percent: 80.0,
            theme: Theme::Dark,
            check_for_updates: true,
            check_interval_hours: 24,
            last_update_check: None,
            skipped_version: None,
            last_tool: None,
        }
    }
}

impl CommonSettings {
    /// The worker thread count a pipeline should use: `thread_count` when set, else the logical
    /// core count minus one (a core is reserved for the reader and writer), never below one.
    pub fn worker_threads(&self) -> usize {
        if self.thread_count > 0 {
            return self.thread_count;
        }
        let logical = available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1);
        logical.saturating_sub(1).max(1)
    }
}

/// GUI color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Dark background (egui's default).
    Dark,
    /// Light background.
    Light,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_thread_count_wins_and_auto_is_at_least_one() {
        let mut settings = CommonSettings::default();
        assert!(settings.worker_threads() >= 1);
        settings.thread_count = 3;
        assert_eq!(settings.worker_threads(), 3);
    }
}
