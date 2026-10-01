//! What the crate's unit tests share.

use std::fs;
use std::path::PathBuf;

/// A fresh, empty `<temp>/team_compiler_<name>_<pid>` folder; `name` is unique per test.
pub(crate) fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("team_compiler_{name}_{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}
