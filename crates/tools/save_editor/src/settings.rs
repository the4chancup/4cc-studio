//! The Save editor's section of the settings file. A key enters with the step whose code reads
//! it, so the file never carries a setting nothing honors yet: the commands of step 5.4a read
//! none (the AATF rules path arrives with `check-aatf`, the comparator defaults and the last
//! folder with the window), so the section is empty today.

use serde::{Deserialize, Serialize};

/// The Save editor's settings, the keys this version reads: none yet.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SaveEditorSettings {}

/// The defaults as a settings table, merged into the file's section for every key it lacks.
pub(crate) fn default_table() -> toml::Table {
    toml::Table::try_from(SaveEditorSettings::default())
        .expect("a struct with no field is an empty table")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_an_empty_table_an_unknown_key_reads_back_as_the_defaults() {
        assert!(default_table().is_empty());
        let table: toml::Table = toml::from_str("setting_from_a_newer_version = 3").unwrap();
        let settings: SaveEditorSettings = table.try_into().unwrap();
        assert_eq!(settings, SaveEditorSettings::default());
    }
}
