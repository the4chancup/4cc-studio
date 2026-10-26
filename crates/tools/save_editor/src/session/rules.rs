//! The AATF ruleset `check-aatf` and the window's Cards left badge share
//! (`plans/save_editor.md` "Editing UI", the Cards left bullet): the
//! `aatf_rules` setting's file, resolved and loaded one way for both, else
//! the embedded official ruleset. `team_views`/`check_teams` are the
//! check both the command line and the AATF panel run; `validate` is the
//! panel's Validate rules button.

use std::path::{Path, PathBuf};

use aatf::{LoadError, Rules, TeamView, Violation};
use anyhow::anyhow;
use pes_savefile::EditFile;
use rayon::prelude::*;

/// One team's AATF check: its violations, or the error text (a team view
/// or the script).
pub(crate) type TeamCheck = Result<Vec<Violation>, String>;

/// The rules file the `aatf_rules` setting names, resolved as `check-aatf`
/// resolves it (a relative path in the data directory when one is chosen,
/// else as given); `None` when the setting is empty (the embedded official
/// ruleset).
pub(crate) fn rules_file(setting: &str, data_dir: Option<&Path>) -> Option<PathBuf> {
    if setting.is_empty() {
        return None;
    }
    let path = PathBuf::from(setting);
    Some(match (path.is_relative(), data_dir) {
        (true, Some(data_dir)) => data_dir.join(path),
        _ => path,
    })
}

/// The ruleset `file` loads: `None` the embedded official ruleset; a file
/// that does not read or load is the error naming it, the text `check-aatf`
/// prints.
pub(crate) fn load_rules(file: Option<&Path>) -> anyhow::Result<Rules> {
    let Some(path) = file else {
        return Ok(Rules::official());
    };
    let text = std::fs::read_to_string(path)
        .map_err(|error| anyhow!("{}: cannot read the file: {error}", path.display()))?;
    Rules::load(&text).map_err(|error| anyhow!("{}: {error}", path.display()))
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

/// Load-time validation of `file` (`None`: the embedded file): one line
/// per finding, empty when it loads. `LoadError::Rules` is one line per
/// `RuleError`, every other error its own `Display` (which carries the
/// line and column or the key path); a file that does not read is one
/// line, `cannot read the file: <error>`.
pub(crate) fn validate(file: Option<&Path>) -> Vec<String> {
    let text = match file {
        None => aatf::official_file(),
        Some(path) => match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => return vec![format!("cannot read the file: {error}")],
        },
    };
    match Rules::load(&text) {
        Ok(_) => Vec::new(),
        Err(LoadError::Rules(errors)) => errors.iter().map(|error| error.to_string()).collect(),
        Err(error) => vec![error.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // plans/save_editor.md "Configurable AATF rules", the Validate rules
    // bullet: a file that does not read is one line, `cannot read the
    // file: <error>`.
    #[test]
    fn an_unreadable_file_validates_to_one_line() {
        let missing = crate::test_support::scratch("validate_unreadable").join("none.4ccrules");
        let lines = validate(Some(&missing));
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].starts_with("cannot read the file: "), "{lines:?}");
    }

    // plans/save_editor.md "Configurable AATF rules", the Validate rules
    // bullet: `LoadError::Rules` fans out to one line per `RuleError`,
    // each starting with its key path; a file that loads is no lines.
    #[test]
    fn rule_errors_fan_out_one_line_each_and_a_clean_file_is_none() {
        assert!(validate(None).is_empty(), "the embedded file loads");

        // The official file with two values broken: the tier counts over
        // the squad size, and an unknown suggestion word.
        let text = aatf::official_file()
            .replacen("squad_size: 23", "squad_size: 4", 1)
            .replacen("suggestions: [\"cards\"", "suggestions: [\"bogus\"", 1);
        let path = crate::test_support::scratch("validate_two_errors").join("two.4ccrules");
        std::fs::write(&path, text).unwrap();
        assert_eq!(
            validate(Some(&path)),
            [
                "tiers: counts total 5 over squad_size 4",
                "suggestions[0]: unknown suggestion \"bogus\"",
            ]
        );
    }

    /// The loader reads the content, never the ending (`spec/aatf_rules.md`
    /// "The rules file"): a valid file loads whatever its name ends in.
    #[test]
    fn the_rules_file_loads_whatever_it_ends_in() {
        let dir = crate::test_support::scratch("rules_extension_ignored");
        for name in ["rules.4ccrules", "rules.txt"] {
            let path = dir.join(name);
            std::fs::write(&path, aatf::official_file()).unwrap();
            let rules = load_rules(Some(&path)).unwrap_or_else(|error| panic!("{name}: {error}"));
            assert_eq!(rules.ruleset().name, "Autumn 26");
        }
    }
}
