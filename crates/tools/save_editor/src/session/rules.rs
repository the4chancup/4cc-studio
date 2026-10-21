//! The AATF ruleset `check-aatf` and the window's badges share
//! (`plans/save_editor.md` "Editing UI", the Cards left bullet): the
//! `aatf_rules` setting's file, resolved and loaded one way for both, else
//! the embedded official ruleset.

use std::path::{Path, PathBuf};

use aatf::Rules;
use anyhow::anyhow;

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
