//! The AATF ruleset `check-aatf` and the window share, so the quick
//! actions, the Cards left badge and Live AATF read a player as the
//! command line does: the `aatf_ruleset` setting resolved one way for
//! both (`resolve`) against the built-in rulesets and the ones imported
//! into the data folder's `rules/` (`imported`). `team_views`/`check_teams`
//! are the check both the command line and Live AATF run.

use std::path::Path;
use std::sync::Arc;

use aatf::{Rules, TeamView, Violation};
use anyhow::anyhow;
use pes_savefile::EditFile;
use rayon::prelude::*;

/// One team's AATF check: its violations, or the error text (a team view
/// or the script).
pub(crate) type TeamCheck = Result<Vec<Violation>, String>;

/// The first built-in ruleset's stem: the setting's default, and the
/// ruleset a choice that is gone falls back to.
pub(crate) const FIRST_BUILTIN: &str = "autumn26";

/// The data folder's subfolder the imported rulesets' copies live in.
const RULES_FOLDER: &str = "rules";

/// An imported ruleset: a rules file in the data folder's `rules/` that
/// loads.
pub(crate) struct Imported {
    /// The copy's file name (`cup.4ccrules`): the `aatf_ruleset` value
    /// that picks it.
    pub(crate) file_name: String,
    /// The copy, loaded.
    pub(crate) rules: Arc<Rules>,
}

/// What the `aatf_ruleset` setting resolved to.
pub(crate) struct Choice {
    /// The ruleset, loaded.
    pub(crate) rules: Arc<Rules>,
    /// The built-in stem or the imported file name the rules came from:
    /// the setting itself, or the first built-in stem when what it named
    /// is gone.
    pub(crate) key: String,
    /// The warning to show when the setting named a ruleset that is gone
    /// and the first built-in one stands in for it.
    pub(crate) notice: Option<String>,
}

/// The rules file at `path` read and loaded; a file that does not read
/// or load is the error naming it, the text `check-aatf` prints.
pub(crate) fn load_file(path: &Path) -> anyhow::Result<Rules> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| anyhow!("{}: cannot read the file: {error}", path.display()))?;
    Rules::load(&text).map_err(|error| anyhow!("{}: {error}", path.display()))
}

/// The imported rulesets: the `.4ccrules` files in `data_dir`'s `rules/`
/// that load, sorted by description. A file that does not read or load is
/// left out with no word to the member, since nobody picked it; a setting
/// naming it falls back in `resolve`, which says so. No data folder, or no
/// `rules/` in it: none.
pub(crate) fn imported(data_dir: Option<&Path>) -> Vec<Imported> {
    let Some(folder) = data_dir.map(|dir| dir.join(RULES_FOLDER)) else {
        return Vec::new();
    };
    let entries = match std::fs::read_dir(&folder) {
        Ok(entries) => entries,
        Err(error) => {
            log::debug!("no imported rulesets in {}: {error}", folder.display());
            return Vec::new();
        }
    };
    let mut imported = Vec::new();
    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(error) => {
                log::debug!("{}: an entry cannot be read: {error}", folder.display());
                continue;
            }
        };
        if path.extension().and_then(|extension| extension.to_str())
            != Some(aatf::RULES_FILE_EXTENSION)
        {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        match load_file(&path) {
            Ok(rules) => imported.push(Imported {
                file_name: file_name.to_owned(),
                rules: Arc::new(rules),
            }),
            Err(error) => log::debug!("left out of the imported rulesets: {error:#}"),
        }
    }
    imported.sort_by(|a, b| a.rules.ruleset().name.cmp(&b.rules.ruleset().name));
    imported
}

/// The ruleset `setting` picks: a built-in stem's, else the imported file
/// of that name's, else (a copy gone or no longer loading, a typo) the
/// first built-in one with the notice saying so.
pub(crate) fn resolve(setting: &str, imported: &[Imported]) -> Choice {
    if let Some(rules) = aatf::builtin(setting) {
        return Choice {
            rules: Arc::new(rules),
            key: setting.to_owned(),
            notice: None,
        };
    }
    if let Some(import) = imported.iter().find(|import| import.file_name == setting) {
        return Choice {
            rules: Arc::clone(&import.rules),
            key: setting.to_owned(),
            notice: None,
        };
    }
    let rules = aatf::builtin(FIRST_BUILTIN).expect("the first built-in ruleset is built in");
    let notice = format!(
        "The AATF ruleset picked last is gone from the data folder: {} is used instead.",
        rules.ruleset().name
    );
    Choice {
        rules: Arc::new(rules),
        key: FIRST_BUILTIN.to_owned(),
        notice: Some(notice),
    }
}

/// The views of `ids`' teams, built where the `EditFile` lives (the UI
/// thread).
pub(crate) fn team_views(file: &EditFile, ids: &[u32]) -> Vec<Result<TeamView, String>> {
    ids.iter()
        .map(|&id| TeamView::new(file, id).map_err(|error| error.to_string()))
        .collect()
}

/// `views` checked on `rules` in parallel, in order.
pub(crate) fn check_teams(rules: &Rules, views: &[Result<TeamView, String>]) -> Vec<TeamCheck> {
    views
        .par_iter()
        .map(|view| {
            view.as_ref()
                .map_err(Clone::clone)
                .and_then(|view| rules.check_team(view).map_err(|error| error.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `dir`'s `rules/` holding `name`, the official file renamed
    /// `description`.
    fn import(dir: &Path, name: &str, description: &str) {
        let folder = dir.join(RULES_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        let text = aatf::official_file().replacen(
            "name: \"4cc Autumn 26\"",
            &format!("name: \"{description}\""),
            1,
        );
        assert!(text.contains(description), "premise: the name was replaced");
        std::fs::write(folder.join(name), text).unwrap();
    }

    /// A built-in stem resolves to its ruleset, no notice; with no data
    /// folder there are no imports, the built-ins alone.
    #[test]
    fn a_builtin_stem_resolves_to_its_ruleset() {
        assert!(imported(None).is_empty(), "no data folder: no imports");
        let choice = resolve("vtl11", &[]);
        assert_eq!(choice.rules.ruleset().name, "/vt/ League 11");
        assert_eq!(choice.key, "vtl11");
        assert_eq!(choice.notice, None);
    }

    /// The `.4ccrules` files of `rules/` that load are the imports, by
    /// description; a file that does not load and a file of another ending
    /// are left out; an imported file name resolves to its copy.
    #[test]
    fn the_imported_files_that_load_are_listed_and_resolve() {
        let dir = crate::test_support::scratch("rules_imported");
        // The descriptions sort the other way round from the file names.
        import(&dir, "alpha.4ccrules", "Zeta Cup");
        import(&dir, "zeta.4ccrules", "Beta Cup");
        import(&dir, "notes.txt", "Text Cup");
        std::fs::write(dir.join(RULES_FOLDER).join("broken.4ccrules"), "not rhai {").unwrap();

        let list = imported(Some(&dir));
        let rows: Vec<(&str, &str)> = list
            .iter()
            .map(|import| {
                (
                    import.file_name.as_str(),
                    import.rules.ruleset().name.as_str(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                ("zeta.4ccrules", "Beta Cup"),
                ("alpha.4ccrules", "Zeta Cup")
            ]
        );

        let choice = resolve("alpha.4ccrules", &list);
        assert_eq!(choice.rules.ruleset().name, "Zeta Cup");
        assert_eq!(choice.key, "alpha.4ccrules");
        assert_eq!(choice.notice, None);
    }

    /// A setting naming a copy no longer in `rules/` (or one that no
    /// longer loads) falls back to the first built-in ruleset, with the
    /// notice naming it.
    #[test]
    fn a_ruleset_that_is_gone_falls_back_to_the_first_builtin_with_a_notice() {
        let dir = crate::test_support::scratch("rules_gone");
        import(&dir, "cup.4ccrules", "Cup");
        std::fs::write(dir.join(RULES_FOLDER).join("broken.4ccrules"), "not rhai {").unwrap();
        let list = imported(Some(&dir));
        for setting in ["gone.4ccrules", "broken.4ccrules"] {
            let choice = resolve(setting, &list);
            assert_eq!(choice.rules.ruleset().name, "4cc Autumn 26", "{setting}");
            assert_eq!(choice.key, "autumn26", "{setting}");
            assert_eq!(
                choice.notice.as_deref(),
                Some(
                    "The AATF ruleset picked last is gone from the data folder: 4cc Autumn 26 \
                     is used instead."
                ),
                "{setting}"
            );
        }
    }

    /// The loader reads the content, never the ending (`spec/aatf_rules.md`
    /// "The rules file"): a valid file loads whatever its name ends in.
    #[test]
    fn the_rules_file_loads_whatever_it_ends_in() {
        let dir = crate::test_support::scratch("rules_extension_ignored");
        for name in ["rules.4ccrules", "rules.txt"] {
            let path = dir.join(name);
            std::fs::write(&path, aatf::official_file()).unwrap();
            let rules = load_file(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
            assert_eq!(rules.ruleset().name, "4cc Autumn 26");
        }
    }
}
