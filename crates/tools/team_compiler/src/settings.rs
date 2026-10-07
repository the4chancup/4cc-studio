//! The Team compiler's section of the settings file (`team_compiler/settings.md`, "Team compiler
//! settings"). A key enters with the phase whose code reads it, so the file never carries a
//! setting nothing honors yet.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The Team compiler's settings, the keys this version reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct TeamCompilerSettings {
    /// The single output CPK's name, without `.cpk`.
    pub(crate) cpk_name: String,
    /// Where compiled CPKs go; a relative path resolves beside the executable.
    pub(crate) output_folder_path: PathBuf,
    /// Cup DLC mode: team content split into size-capped parts plus a bins CPK.
    pub(crate) multicpk_mode: bool,
    /// The stem of the teams part slots in multi-CPK mode: every official list entry
    /// `{prefix}_{NN}_{stem}.cpk` is a slot (`4cc_41_teams`), filled in the order of `NN`.
    pub(crate) teams_cpk_name: String,
    /// The most bytes a teams part may hold, its table of contents included; a single CPK
    /// past it is only warned about. A byte count, not text like `3 GB`: it needs no unit
    /// parser and leaves no doubt between GB and GiB.
    pub(crate) cpk_part_max_size: u64,
    /// The CPK, without `.cpk`, that holds the bins and the overrides in multi-CPK mode.
    pub(crate) bins_cpk_name: String,
    /// Disallowed file types are errors when on, info notes when off.
    pub(crate) strict_file_type_check: bool,
    /// Keep folders with errors instead of discarding them.
    pub(crate) pass_through: bool,
    /// The working teams list; a relative path resolves in the data directory.
    pub(crate) teams_list_path: PathBuf,
}

impl Default for TeamCompilerSettings {
    fn default() -> Self {
        TeamCompilerSettings {
            cpk_name: "4cc_99_test".to_owned(),
            output_folder_path: PathBuf::from("output"),
            multicpk_mode: false,
            teams_cpk_name: "teams".to_owned(),
            // 3 GiB: comfortably under the 4 GiB a Git for Windows object can hold, the limit
            // the cup DLC's repository has.
            cpk_part_max_size: 3_221_225_472,
            bins_cpk_name: "4cc_08_bins".to_owned(),
            strict_file_type_check: true,
            pass_through: false,
            teams_list_path: PathBuf::from("teams_list.txt"),
        }
    }
}

/// The defaults as a settings table, merged into the file's section for every key it lacks.
pub(crate) fn default_table() -> toml::Table {
    toml::Table::try_from(TeamCompilerSettings::default()).expect(
        "the defaults are strings, ASCII paths, booleans and a count far below i64::MAX, \
             all of which TOML holds",
    )
}

impl TeamCompilerSettings {
    /// The CPK below which the installed CPKs are walked for the working bins and looked in for
    /// textures: the run's first CPK in the list's order, which is the bins CPK in multi-CPK
    /// mode (`pipeline.md` "Multi-CPK mode: teams parts", "The run's first CPK is its
    /// boundary"), else `cpk_name`.
    pub(crate) fn boundary_cpk_name(&self) -> &str {
        if self.multicpk_mode {
            &self.bins_cpk_name
        } else {
            &self.cpk_name
        }
    }
}

/// Reads the tool's settings table. A missing key loads as its default; a key this version does
/// not know is ignored, so a newer version's settings file does not break an older binary; a key
/// of the wrong type is an error naming it.
pub(crate) fn from_table(table: &toml::Table) -> Result<TeamCompilerSettings, toml::de::Error> {
    table.clone().try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_through_the_table() {
        let table = default_table();
        assert_eq!(table["cpk_name"].as_str(), Some("4cc_99_test"));
        assert_eq!(table["strict_file_type_check"].as_bool(), Some(true));
        assert_eq!(table["teams_cpk_name"].as_str(), Some("teams"));
        assert_eq!(table["cpk_part_max_size"].as_integer(), Some(3_221_225_472));
        assert_eq!(table["bins_cpk_name"].as_str(), Some("4cc_08_bins"));
        assert_eq!(from_table(&table).unwrap(), TeamCompilerSettings::default());
    }

    #[test]
    fn the_boundary_is_the_bins_cpk_in_multi_cpk_mode_and_cpk_name_otherwise() {
        let mut settings = TeamCompilerSettings {
            cpk_name: "4cc_61_midcup".to_owned(),
            ..TeamCompilerSettings::default()
        };
        assert_eq!(settings.boundary_cpk_name(), "4cc_61_midcup");
        settings.multicpk_mode = true;
        assert_eq!(settings.boundary_cpk_name(), "4cc_08_bins");
    }

    #[test]
    fn a_missing_key_takes_its_default_and_an_unknown_key_is_ignored() {
        let table: toml::Table =
            toml::from_str("pass_through = true\nsetting_from_a_newer_version = 3").unwrap();
        let settings = from_table(&table).unwrap();
        assert!(settings.pass_through);
        assert_eq!(settings.cpk_name, "4cc_99_test");
    }

    #[test]
    fn a_key_of_the_wrong_type_is_an_error_naming_it() {
        let table: toml::Table = toml::from_str("strict_file_type_check = \"yes\"").unwrap();
        let error = from_table(&table).unwrap_err();
        assert!(
            error.to_string().contains("strict_file_type_check"),
            "{error}"
        );
    }
}
