//! The Save editor's section of the settings file. A key enters with the step whose code reads
//! it, so the file never carries a setting nothing honors yet: `check-aatf` (step 5.4b) reads
//! `aatf_rules`; the window reads `live_aatf` (the toolbar's Live AATF toggle); the comparator
//! defaults and the last folder arrive with the window.

use serde::{Deserialize, Serialize};

/// The Save editor's settings, the keys this version reads.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SaveEditorSettings {
    /// The AATF rules file `check-aatf` and the AATF panel run; empty: the official ruleset,
    /// built in.
    pub(crate) aatf_rules: String,
    /// The toolbar's Live AATF toggle; on, a docked panel lists the open team's violations
    /// after every edit.
    pub(crate) live_aatf: bool,
}

/// The defaults as a settings table, merged into the file's section for every key it lacks.
pub(crate) fn default_table() -> toml::Table {
    toml::Table::try_from(SaveEditorSettings::default()).expect("the defaults serialize")
}

/// Reads the tool's settings table. A missing key loads as its default; a key this version does
/// not know is ignored, so a newer version's settings file does not break an older binary; a key
/// of the wrong type is an error naming it.
pub(crate) fn from_table(table: &toml::Table) -> Result<SaveEditorSettings, toml::de::Error> {
    table.clone().try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_carry_no_rules_file_and_an_unknown_key_reads_back_as_the_defaults() {
        assert_eq!(default_table()["aatf_rules"].as_str(), Some(""));
        let table: toml::Table = toml::from_str("setting_from_a_newer_version = 3").unwrap();
        assert_eq!(from_table(&table).unwrap(), SaveEditorSettings::default());
    }

    #[test]
    fn aatf_rules_names_a_rules_file() {
        let table: toml::Table = toml::from_str("aatf_rules = \"vgl27.rhai\"").unwrap();
        assert_eq!(from_table(&table).unwrap().aatf_rules, "vgl27.rhai");
    }
}
