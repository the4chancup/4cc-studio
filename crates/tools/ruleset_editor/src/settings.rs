//! The Ruleset editor's section of the settings file (`plans/ruleset_editor.md`
//! "Settings"): where Open and Save as start, and the start page's recent
//! files.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// The recent-files list's length.
const RECENT_MAX: usize = 10;

/// The Ruleset editor's settings, the keys this version reads.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct RulesetEditorSettings {
    /// The folder Open and Save as start in; empty: none stored yet.
    pub(crate) last_folder: String,
    /// The start page's recent files, newest first, at most 10; the stored
    /// list keeps files that no longer exist (the start page skips them).
    pub(crate) recent_files: Vec<String>,
}

impl RulesetEditorSettings {
    /// A file opened or saved moves to the front of the recent list
    /// (an earlier entry of it goes), the list cut to 10, and `last_folder`
    /// set to the file's folder.
    pub(crate) fn remember(&mut self, path: &Path) {
        let file = path.to_string_lossy().into_owned();
        self.recent_files.retain(|entry| entry != &file);
        self.recent_files.insert(0, file);
        self.recent_files.truncate(RECENT_MAX);
        if let Some(folder) = path.parent() {
            self.last_folder = folder.to_string_lossy().into_owned();
        }
    }

    /// The settings as a settings table, written on every open and save.
    pub(crate) fn to_table(&self) -> toml::Table {
        toml::Table::try_from(self).expect("the settings are a string and a string list")
    }
}

/// The defaults as a settings table, merged into the file's section for every key it lacks.
pub(crate) fn default_table() -> toml::Table {
    toml::Table::try_from(RulesetEditorSettings::default())
        .expect("the default is a string and a string list")
}

/// Reads the tool's settings table. A missing key loads as its default; a key this version does
/// not know is ignored, so a newer version's settings file does not break an older binary; a key
/// of the wrong type is an error naming it.
pub(crate) fn from_table(table: &toml::Table) -> Result<RulesetEditorSettings, toml::de::Error> {
    table.clone().try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_puts_the_path_first_and_sets_the_last_folder() {
        let mut settings = RulesetEditorSettings::default();
        settings.remember(Path::new("cups/invitational/rules.4ccrules"));
        assert_eq!(settings.recent_files, ["cups/invitational/rules.4ccrules"]);
        assert_eq!(settings.last_folder, "cups/invitational");
        // A second file heads the list, the first following.
        settings.remember(Path::new("other/season.4ccrules"));
        assert_eq!(
            settings.recent_files,
            ["other/season.4ccrules", "cups/invitational/rules.4ccrules"]
        );
        // A file already listed moves to the front without duplicating.
        settings.remember(Path::new("cups/invitational/rules.4ccrules"));
        assert_eq!(
            settings.recent_files,
            ["cups/invitational/rules.4ccrules", "other/season.4ccrules"]
        );
    }

    #[test]
    fn a_full_list_is_cut_to_ten() {
        let mut settings = RulesetEditorSettings::default();
        for index in 0..15 {
            settings.remember(Path::new(&format!("f/{index}.4ccrules")));
        }
        assert_eq!(settings.recent_files.len(), 10);
        assert_eq!(settings.recent_files[0], "f/14.4ccrules");
        assert_eq!(settings.recent_files[9], "f/5.4ccrules");
    }

    #[test]
    fn the_table_round_trip_carries_both_keys() {
        let mut settings = RulesetEditorSettings::default();
        settings.remember(Path::new("f/rules.4ccrules"));
        let table = settings.to_table();
        assert_eq!(from_table(&table).unwrap(), settings);
        // The defaults merge for keys the file lacks; an unknown key reads back as the defaults.
        let table: toml::Table =
            toml::from_str("last_folder = \"f\"\nsetting_from_a_newer_version = 3").unwrap();
        assert_eq!(
            from_table(&table).unwrap(),
            RulesetEditorSettings {
                last_folder: "f".to_string(),
                recent_files: Vec::new()
            }
        );
        assert_eq!(default_table()["last_folder"].as_str(), Some(""));
        assert_eq!(
            default_table()["recent_files"].as_array(),
            Some(&Vec::new())
        );
    }
}
