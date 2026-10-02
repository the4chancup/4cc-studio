//! The Team compiler's command line run in-process (`team_compiler/settings.md` "CLI"): the
//! preflight's refusals, `check`'s findings and what `compile` writes, observed as the exit
//! code, the `Message` events the tool emits (which the binary prints one line each) and the
//! CPK's entries.

#[path = "../common/mod.rs"]
mod common;

mod check;
mod compile;
mod compile_exports;
mod preflight;
mod sources;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use common::{Run, Sandbox};
use studio_core::{ExportId, PipelineEvent, Scope, StudioTool};
use team_compiler::Tool;

impl Sandbox {
    fn write(&self, relative: &str, contents: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    /// Copies `tests/fixtures/sources/<name>` to `<relative folder>/<name>`, leaving the fixture
    /// untouched.
    fn copy_fixture(&self, name: &str, folder: &str) {
        self.write(&format!("{folder}/{name}"), &source_fixture(name));
    }

    /// The platform's spelling of a path inside the sandbox, as a source failure names it.
    fn display(&self, relative: &str) -> String {
        relative
            .split('/')
            .fold(self.root.clone(), |path, segment| path.join(segment))
            .display()
            .to_string()
    }

    /// An absolute path inside the sandbox, as a command-line argument.
    fn arg(&self, relative: &str) -> String {
        self.root.join(relative).to_str().unwrap().to_owned()
    }
}

impl Run {
    /// The refusal, asserting its exit code and that its message names every one of `names`.
    fn assert_refused(&self, exit_code: u8, names: &[&str]) {
        let error = match &self.result {
            Ok(code) => panic!("not refused: exit code {code}"),
            Err(error) => error,
        };
        let text = error.to_string();
        assert_eq!(error.exit_code, exit_code, "{text}");
        for name in names {
            assert!(
                text.contains(name),
                "the refusal does not name {name}: {text}"
            );
        }
        assert!(self.events.is_empty(), "a refused command emits nothing");
    }

    /// Every `Message` event as one line: the export's file name (from its `ExportStarted`),
    /// severity, code, disposition, scope and context, so a test asserts all of them at once.
    fn messages(&self) -> Vec<String> {
        let mut names: BTreeMap<ExportId, String> = BTreeMap::new();
        let mut lines = Vec::new();
        for envelope in &self.events {
            match &envelope.event {
                PipelineEvent::ExportStarted {
                    export_id,
                    display_name,
                } => {
                    names.insert(*export_id, display_name.clone());
                }
                PipelineEvent::Message(message) => {
                    assert_eq!(envelope.export_id, message.scope.export_id());
                    let source = match message.scope.export_id() {
                        Some(export_id) => format!("{}: ", names[&export_id]),
                        None => String::new(),
                    };
                    let location = match &message.scope {
                        Scope::Run | Scope::Export { .. } => String::new(),
                        Scope::Folder { path, .. } | Scope::File { path, .. } => {
                            format!(" at {}", path.as_str())
                        }
                        Scope::RosterEntry {
                            file, line, slot, ..
                        } => format!(" at {} line {line} slot {slot:?}", file.as_str()),
                    };
                    let context: Vec<String> = message
                        .context
                        .iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect();
                    lines.push(format!(
                        "{source}{:?} {} [{:?}]{location} ({})",
                        message.severity,
                        message.code.code,
                        message.disposition,
                        context.join(", ")
                    ));
                }
                PipelineEvent::ExportProcessed { .. }
                | PipelineEvent::FolderStatus { .. }
                | PipelineEvent::Progress { .. }
                | PipelineEvent::Complete { .. }
                | PipelineEvent::UnmigratedContentFound => {}
            }
        }
        lines
    }
}

/// The bytes of `tests/fixtures/sources/<name>`, to be written under another name.
fn source_fixture(name: &str) -> Vec<u8> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sources")
        .join(name);
    fs::read(fixture).unwrap()
}

/// Every file under `root`, by relative path, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                files.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
            }
        }
    }
    files
}

/// The lines of `lines` about `source`, without their `<source>: ` prefix, so the findings of
/// sources holding the same export compare equal.
fn findings_of<'a>(lines: &'a [String], source: &str) -> Vec<&'a str> {
    let prefix = format!("{source}: ");
    lines
        .iter()
        .filter_map(|line| line.strip_prefix(prefix.as_str()))
        .collect()
}

/// One player folder with a face model: an export that validates with no finding.
const CLEAN_PLAYER: &str = "Players/03 - A/face_high.fmdl";

#[test]
fn the_tool_is_registered_as_the_team_compiler() {
    let tool = Tool::new();
    assert_eq!(tool.id(), "team-compiler");
    assert_eq!(tool.label(), "Team compiler");
    assert_eq!(
        tool.default_settings()["cpk_name"].as_str(),
        Some("4cc_90_test")
    );
}
