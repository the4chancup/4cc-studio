//! The Save editor's section of the settings file: its five keys. `check-aatf` and the window's
//! AATF checks read `aatf_rules`; the window reads `live_aatf` (the toolbar's Live AATF toggle),
//! `live_suggestions` (its Show suggestions checkbox), `aesthetics_from_exports` ("Aesthetics
//! from exports", the aesthetics lock's setting) and `scroll_names` (the Tactics formation bar's
//! checkbox). A key enters with the code that reads it, so the file never carries a setting
//! nothing honors.

use serde::{Deserialize, Serialize};

/// The Save editor's settings, the keys this version reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SaveEditorSettings {
    /// The AATF rules file `check-aatf` and the AATF panel run; empty: the official ruleset,
    /// built in.
    pub(crate) aatf_rules: String,
    /// The toolbar's Live AATF toggle; on, a docked panel lists the open team's violations
    /// after every edit.
    pub(crate) live_aatf: bool,
    /// Show suggestions, the checkbox beside the Live AATF toggle; on, Live AATF also lists
    /// each player's unused allowances.
    pub(crate) live_suggestions: bool,
    /// "Aesthetics from exports", off by default because the cup the suite serves first
    /// keeps its looks in the save. On, the save's aesthetics write only through the
    /// Appearance section's "Edit aesthetics anyway" switch; off, they edit as every other
    /// field does.
    pub(crate) aesthetics_from_exports: bool,
    /// "Scroll names" (the Tactics formation bar's checkbox), on by default: a view setting,
    /// not an edit — `Tool::view` reads it at start and writes it back when the checkbox
    /// changes it, as `live_aatf` is written.
    #[serde(default = "default_scroll_names")]
    pub(crate) scroll_names: bool,
}

impl Default for SaveEditorSettings {
    fn default() -> SaveEditorSettings {
        SaveEditorSettings {
            aatf_rules: String::new(),
            live_aatf: false,
            live_suggestions: false,
            aesthetics_from_exports: false,
            scroll_names: default_scroll_names(),
        }
    }
}

/// `scroll_names`'s default: names scroll, as the game's do.
fn default_scroll_names() -> bool {
    true
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
        let table: toml::Table = toml::from_str("aatf_rules = \"vgl27.4ccrules\"").unwrap();
        assert_eq!(from_table(&table).unwrap().aatf_rules, "vgl27.4ccrules");
    }

    // SE-GUI-35's GIVEN (the setting off is the default).
    #[test]
    fn aesthetics_from_exports_defaults_off_and_round_trips() {
        assert_eq!(
            default_table()
                .get("aesthetics_from_exports")
                .and_then(toml::Value::as_bool),
            Some(false),
            "off by default"
        );
        let table: toml::Table = toml::from_str("aesthetics_from_exports = true").unwrap();
        let written = toml::Table::try_from(from_table(&table).unwrap()).unwrap();
        assert_eq!(
            written
                .get("aesthetics_from_exports")
                .and_then(toml::Value::as_bool),
            Some(true),
            "read in and written back"
        );
    }
}
