//! The sandbox the in-process tests run the Team compiler's command line in: a fresh folder
//! standing in for the executable's, with its own settings text and teams list.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use studio_core::{AppPaths, CliError, PipelineEventEnvelope, Settings, StudioTool, ToolContext};
use team_compiler::Tool;

/// The teams list every sandbox carries, so identities are predictable: `/co/` for the tests'
/// own exports, `/dbg/` and `/esg/` for more teams of their own, `/egg/` for the tracer
/// fixture. Each row is the bundled upstream list's own, so an id a test asserts is the one a
/// member sees.
const TEAMS_LIST: &str = "ID\tName\n714\t/co/\n790\t/dbg/\n792\t/egg/\n793\t/esg/\n";

/// A fresh folder standing in for the executable's folder: `data/` holds the teams list, the
/// exports root defaults to `exports/` beside it and the output folder to `output/`.
pub struct Sandbox {
    pub root: PathBuf,
}

/// What one command returned, and the events it emitted.
pub struct Run {
    pub result: Result<u8, CliError>,
    pub events: Vec<PipelineEventEnvelope>,
}

impl Sandbox {
    pub fn new(name: &str) -> Sandbox {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("team_compiler_in_process")
            .join(name);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("data/teams_list.txt"), TEAMS_LIST).unwrap();
        Sandbox { root }
    }

    /// Runs `team-compiler <args>` with `settings` as the settings file's text.
    pub fn run(&self, settings: &str, args: &[&str]) -> Run {
        let mut parsed = Settings::parse(settings).unwrap();
        parsed.merge_defaults(Tool.id(), &Tool.default_settings());
        let (events_tx, events_rx) = unbounded();
        let (requests_tx, _requests_rx) = unbounded();
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(parsed)),
            AppPaths {
                exe_dir: self.root.clone(),
                data_dir: Some(self.root.join("data")),
            },
            events_tx,
            requests_tx,
        );
        let matches = Tool
            .cli_command()
            .try_get_matches_from(std::iter::once("team-compiler").chain(args.iter().copied()))
            .unwrap();
        let result = Tool.cli_run(&matches, &ctx);
        drop(ctx);
        Run {
            result,
            events: events_rx.iter().collect(),
        }
    }
}

impl Run {
    pub fn exit_code(&self) -> u8 {
        match &self.result {
            Ok(code) => *code,
            Err(error) => panic!("refused with {}: {error}", error.exit_code),
        }
    }
}
