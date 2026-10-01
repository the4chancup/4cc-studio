//! Where the settings file lives: next to the executable or in the user config directory, chosen
//! by which one already holds the file (core plan, `distribution.md` "Data location"). Resolution
//! only looks; asking the user and writing the file belong to the GUI's first start.

use std::path::{Path, PathBuf};

use directories::BaseDirs;

/// The settings file's name, in either data location.
pub const SETTINGS_FILE_NAME: &str = "settings.toml";

/// The data directory, by presence (core plan, `distribution.md` "Data location"):
/// `<exe_dir>/data` when it holds the settings file, else `config_dir` when it does,
/// else `None` (no settings file yet: the first GUI start asks). Never creates anything.
pub fn resolve_data_dir(exe_dir: &Path, config_dir: Option<&Path>) -> Option<PathBuf> {
    let portable = exe_dir.join("data");
    if portable.join(SETTINGS_FILE_NAME).is_file() {
        return Some(portable);
    }
    let config_dir = config_dir?;
    config_dir
        .join(SETTINGS_FILE_NAME)
        .is_file()
        .then(|| config_dir.to_path_buf())
}

/// The user config location: `%APPDATA%\4cc-studio` on Windows, `~/.config/4cc-studio`
/// on Linux. `None` when the OS reports no home.
pub fn user_config_dir() -> Option<PathBuf> {
    // `BaseDirs`, not `ProjectDirs`: the latter appends `\config` on Windows, which is not the
    // folder the plan names.
    Some(BaseDirs::new()?.config_dir().join("4cc-studio"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// A fresh `<temp>/studio_core_location_<name>_<pid>` folder with `exe/` and `config/` inside.
    fn sandbox(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "studio_core_location_{name}_{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("exe")).unwrap();
        fs::create_dir_all(root.join("config")).unwrap();
        root
    }

    fn listing(dir: &Path) -> Vec<PathBuf> {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        entries.sort();
        entries
    }

    fn put_settings(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(SETTINGS_FILE_NAME), "").unwrap();
    }

    #[test]
    fn portable_settings_win_when_both_exist() {
        let root = sandbox("both");
        put_settings(&root.join("exe/data"));
        put_settings(&root.join("config"));
        assert_eq!(
            resolve_data_dir(&root.join("exe"), Some(&root.join("config"))),
            Some(root.join("exe/data"))
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn config_dir_alone_resolves_to_it() {
        let root = sandbox("config_only");
        put_settings(&root.join("config"));
        assert_eq!(
            resolve_data_dir(&root.join("exe"), Some(&root.join("config"))),
            Some(root.join("config"))
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_data_folder_without_the_settings_file_does_not_count() {
        let root = sandbox("empty_data");
        fs::create_dir_all(root.join("exe/data")).unwrap();
        assert_eq!(
            resolve_data_dir(&root.join("exe"), Some(&root.join("config"))),
            None
        );
        put_settings(&root.join("config"));
        assert_eq!(
            resolve_data_dir(&root.join("exe"), Some(&root.join("config"))),
            Some(root.join("config"))
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn neither_location_resolves_to_none_and_creates_nothing() {
        let root = sandbox("neither");
        let before = (listing(&root.join("exe")), listing(&root.join("config")));
        assert_eq!(
            resolve_data_dir(&root.join("exe"), Some(&root.join("config"))),
            None
        );
        assert_eq!(
            before,
            (listing(&root.join("exe")), listing(&root.join("config")))
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn no_config_dir_and_no_portable_file_resolves_to_none() {
        let root = sandbox("no_config");
        assert_eq!(resolve_data_dir(&root.join("exe"), None), None);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn user_config_dir_ends_in_the_app_folder() {
        assert!(user_config_dir().unwrap().ends_with("4cc-studio"));
    }
}
