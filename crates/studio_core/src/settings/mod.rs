//! The settings framework: one TOML file holding the common settings under `[common]` and one
//! table per tool under its id (`[team-compiler]`). Loading fills missing common keys with their
//! defaults; each tool's `default_settings()` is merged into its table the same way, which is what
//! replaces Red's settings-transfer step between versions (core plan, "Settings menu").
//!
//! Saving is the caller's policy: this module writes the file and reports failure; the shell
//! keeps the in-memory settings and raises the `data_dir_read_only` condition.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

pub use common::{CommonSettings, Theme};
use toml::{Table, Value};

/// The key of the common section; no tool may use it as its id.
pub const COMMON_KEY: &str = "common";

/// Everything the settings file holds.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Settings {
    /// The `[common]` section.
    pub common: CommonSettings,
    /// One table per tool, keyed by tool id, exactly as read (unknown keys included).
    tools: BTreeMap<String, Table>,
}

/// Why the settings file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// The file exists but could not be read.
    #[error("cannot read the settings file")]
    Io(#[from] io::Error),
    /// The file is not valid TOML, or a section has the wrong shape.
    #[error("cannot parse the settings file")]
    Parse(#[from] toml::de::Error),
    /// A top-level key other than `common` is not a table (`foo = 1` at the root).
    #[error("settings key `{0}` is not a section")]
    NotASection(String),
}

impl Settings {
    /// Reads the settings file. A missing file is not an error: it yields the defaults, and the
    /// first save creates it.
    pub fn load(path: &Path) -> Result<Settings, SettingsError> {
        match fs::read_to_string(path) {
            Ok(text) => Settings::parse(&text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(error) => Err(error.into()),
        }
    }

    /// Parses settings-file text.
    pub fn parse(text: &str) -> Result<Settings, SettingsError> {
        let mut root: Table = toml::from_str(text)?;
        let common = match root.remove(COMMON_KEY) {
            Some(value) => value.try_into()?,
            None => CommonSettings::default(),
        };
        let mut tools = BTreeMap::new();
        for (key, value) in root {
            match value {
                Value::Table(table) => {
                    tools.insert(key, table);
                }
                _ => return Err(SettingsError::NotASection(key)),
            }
        }
        Ok(Settings { common, tools })
    }

    /// Writes the settings file atomically: to `<path>.tmp`, then renamed over `path`, so a
    /// failure mid-write never leaves a truncated file behind.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let text = self.to_toml_string().map_err(io::Error::other)?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, text)?;
        fs::rename(&tmp, path)
    }

    /// The file text: `[common]` first, then one table per tool in id order.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        let mut root = Table::new();
        root.insert(COMMON_KEY.to_owned(), Value::try_from(&self.common)?);
        for (id, table) in &self.tools {
            root.insert(id.clone(), Value::Table(table.clone()));
        }
        toml::to_string_pretty(&root)
    }

    /// A tool's section, if the file had one.
    pub fn tool(&self, tool_id: &str) -> Option<&Table> {
        self.tools.get(tool_id)
    }

    /// Replaces a tool's section.
    pub fn set_tool(&mut self, tool_id: &str, table: Table) {
        self.tools.insert(tool_id.to_owned(), table);
    }

    /// Fills a tool's section with every default it lacks, recursing into nested tables. Values
    /// already present are kept, including ones no default names.
    pub fn merge_defaults(&mut self, tool_id: &str, defaults: &Table) {
        let table = self.tools.entry(tool_id.to_owned()).or_default();
        merge_missing(table, defaults);
    }
}

fn merge_missing(into: &mut Table, defaults: &Table) {
    for (key, default) in defaults {
        match (into.get_mut(key), default) {
            (None, _) => {
                into.insert(key.clone(), default.clone());
            }
            (Some(Value::Table(existing)), Value::Table(nested)) => merge_missing(existing, nested),
            (Some(_), _) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pes_version::PesVersion;

    fn table(text: &str) -> Table {
        toml::from_str(text).unwrap()
    }

    #[test]
    fn missing_file_loads_as_defaults() {
        let dir = std::env::temp_dir().join("studio_core_settings_missing");
        let loaded = Settings::load(&dir.join("does_not_exist.toml")).unwrap();
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let mut settings = Settings::default();
        settings.common.pes_version = PesVersion::Pes21;
        settings.common.thread_count = 4;
        settings.common.last_tool = Some("save-editor".to_owned());
        settings.set_tool(
            "team-compiler",
            table("dt00_overwrite_allow = true\nlabel = 'x'"),
        );

        let dir = std::env::temp_dir().join("studio_core_settings_roundtrip");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded, settings);
        assert!(!path.with_extension("tmp").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_common_keys_take_defaults_and_unknown_tool_keys_survive() {
        let loaded =
            Settings::parse("[common]\npes_version = 16\n[refs-arranger]\nfuture = 1\n").unwrap();
        assert_eq!(loaded.common.pes_version, PesVersion::Pes16);
        assert_eq!(
            loaded.common.thread_count,
            CommonSettings::default().thread_count
        );
        assert_eq!(
            loaded.tool("refs-arranger").unwrap()["future"],
            Value::Integer(1)
        );
        assert!(loaded.tool("team-compiler").is_none());
    }

    #[test]
    fn merge_defaults_fills_missing_keys_only() {
        let mut settings = Settings::default();
        settings.set_tool("tool", table("kept = 1\n[nested]\nkept = 'a'"));
        settings.merge_defaults(
            "tool",
            &table("kept = 99\nadded = 2\n[nested]\nkept = 'z'\nadded = 'b'\n[deep]\nx = 1"),
        );
        let tool = settings.tool("tool").unwrap();
        assert_eq!(tool["kept"], Value::Integer(1));
        assert_eq!(tool["added"], Value::Integer(2));
        assert_eq!(tool["nested"]["kept"], Value::String("a".to_owned()));
        assert_eq!(tool["nested"]["added"], Value::String("b".to_owned()));
        assert_eq!(tool["deep"]["x"], Value::Integer(1));
    }

    #[test]
    fn merge_defaults_creates_the_section() {
        let mut settings = Settings::default();
        settings.merge_defaults("new-tool", &table("a = 1"));
        assert_eq!(settings.tool("new-tool").unwrap()["a"], Value::Integer(1));
    }

    #[test]
    fn root_scalar_is_an_error() {
        assert!(matches!(
            Settings::parse("stray = 1"),
            Err(SettingsError::NotASection(key)) if key == "stray"
        ));
    }
}
