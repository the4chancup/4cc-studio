//! The 4cc Studio binary: registers the tools, then launches the GUI or dispatches the CLI.
//! With no arguments it opens the shell on the registered tools; `--gui` autorun arrives with
//! the GUI phase.

mod console;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use clap::ArgMatches;
use crossbeam_channel::unbounded;
use log::LevelFilter;
use studio_core::egui_phosphor::regular;
use studio_core::{
    AppPaths, LaunchMode, PlannedTool, SETTINGS_FILE_NAME, Settings, ShellCondition, StudioApp,
    StudioTool, ToolContext, parse_launch, resolve_data_dir, run_cli, run_gui,
    settings_file_writable, user_config_dir,
};

use crate::console::{spawn_notice_printer, spawn_printer};

/// The suite's whole registry as data, in sidebar order (core plan, "Tool plugin
/// interface"): the shell lists after the registered tools every entry whose id no
/// registered tool has, grayed as the roadmap. A registered tool's entry repeats what its
/// `icon()` returns.
const PLANNED: [PlannedTool; 14] = [
    PlannedTool {
        id: "team-compiler",
        label: "Team compiler",
        icon: regular::USERS_THREE,
    },
    PlannedTool {
        id: "save-editor",
        label: "Save editor",
        icon: regular::NOTE_PENCIL,
    },
    PlannedTool {
        id: "stadium-compiler",
        label: "Stadium compiler",
        icon: regular::BUILDINGS,
    },
    PlannedTool {
        id: "export-upgrader",
        label: "Export upgrader",
        icon: regular::ARROW_FAT_UP,
    },
    PlannedTool {
        id: "music-player",
        label: "Music player",
        icon: regular::MUSIC_NOTE,
    },
    PlannedTool {
        id: "music-export-editor",
        label: "Music export editor",
        icon: regular::MUSIC_NOTES,
    },
    PlannedTool {
        id: "match-tracker",
        label: "Match tracker",
        icon: regular::CROSSHAIR,
    },
    PlannedTool {
        id: "kit-config-editor",
        label: "Kit config editor",
        icon: regular::T_SHIRT,
    },
    PlannedTool {
        id: "refs-arranger",
        label: "Refs arranger",
        icon: regular::FLAG,
    },
    PlannedTool {
        id: "balls-compiler",
        label: "Balls compiler",
        icon: regular::SOCCER_BALL,
    },
    PlannedTool {
        id: "player-aesthetics-editor",
        label: "Player aesthetics editor",
        icon: regular::SMILEY,
    },
    PlannedTool {
        id: "team-creator",
        label: "Team creator",
        icon: regular::PLUS_CIRCLE,
    },
    PlannedTool {
        id: "ruleset-editor",
        label: "Ruleset editor",
        icon: regular::SCROLL,
    },
    PlannedTool {
        id: "db-generator",
        label: "DB generator",
        icon: regular::DATABASE,
    },
];

/// The registered tools, in sidebar order.
fn tools() -> Vec<Box<dyn StudioTool>> {
    vec![
        Box::new(team_compiler::Tool::new()),
        Box::new(save_editor::Tool::new()),
        Box::new(export_upgrader::Tool::new()),
    ]
}

/// The diagnostic sink of both modes (core plan, "Diagnostic logging"): `-v` sets the level,
/// `RUST_LOG` overrides it. The GUI mode gets its own file sink in the GUI phase; until then it
/// logs to the console window that opens beside it.
fn install_cli_logger(verbosity: u8) {
    env_logger::Builder::new()
        .filter_level(log_level(verbosity))
        .parse_default_env()
        .init();
}

/// The level `-v` repeated `verbosity` times asks for: warnings only, then info, debug, trace.
fn log_level(verbosity: u8) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

/// The folder holding the executable, which the data location and path settings resolve against.
fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe().context("cannot locate the executable")?;
    Ok(exe
        .parent()
        .context("the executable's path has no folder")?
        .to_path_buf())
}

/// Resolves the data location (presence-based, never asked and never written from the CLI) and
/// loads the settings from it, with every tool's defaults merged in memory. With no settings file
/// yet, the defaults alone and no data directory.
fn load_settings(
    tools: &[Box<dyn StudioTool>],
    exe_dir: PathBuf,
) -> anyhow::Result<(Settings, AppPaths)> {
    let data_dir = resolve_data_dir(&exe_dir, user_config_dir().as_deref());
    let mut settings = match &data_dir {
        Some(dir) => {
            let path = dir.join(SETTINGS_FILE_NAME);
            Settings::load(&path).with_context(|| path.display().to_string())?
        }
        None => Settings::default(),
    };
    for tool in tools {
        settings.merge_defaults(tool.id(), &tool.default_settings());
    }
    Ok((settings, AppPaths { exe_dir, data_dir }))
}

/// What both modes start from: the settings and the base folders. A failure is printed and
/// becomes the exit code: 3 when the executable's folder cannot be found (an environment
/// failure), 2 when the settings file cannot be loaded (a configuration error: nothing ran;
/// `core/architecture.md`, the binary's own codes).
#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn startup(tools: &[Box<dyn StudioTool>]) -> Result<(Settings, AppPaths), ExitCode> {
    let exe_dir = match exe_dir() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("error: {error:#}");
            return Err(ExitCode::from(3));
        }
    };
    load_settings(tools, exe_dir).map_err(|error| {
        eprintln!("error: {error:#}");
        ExitCode::from(2)
    })
}

/// Runs one tool subcommand headless and returns the process exit code: the tool's own, the
/// startup failure's, or 3 when the console printer failed.
#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn run_cli_mode(tools: &[Box<dyn StudioTool>], tool: &str, matches: &ArgMatches) -> ExitCode {
    let (settings, paths) = match startup(tools) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    // The CLI never writes settings, so the read-only data directory is probed rather than
    // learned from a failed save; the condition prints once, as stderr's first line
    // (gui.md "Status bar" "CLI equivalence", ST-CLI-01).
    if let Some(dir) = &paths.data_dir
        && !settings_file_writable(dir)
    {
        eprintln!("{}", ShellCondition::DataDirReadOnly.text());
    }
    let (events_tx, events_rx) = unbounded();
    let (requests_tx, requests_rx) = unbounded();
    let printer = spawn_printer(events_rx);
    let notices = spawn_notice_printer(requests_rx);
    let ctx = ToolContext::new(
        Arc::new(Mutex::new(settings)),
        paths,
        events_tx,
        requests_tx,
    );
    let result = run_cli(tools, tool, matches, &ctx);
    // The printers end when every sender is gone, and the context holds them.
    drop(ctx);
    // The notice thread's `Err` is a failed stderr write: it loses lines like a panic does.
    if printer.join().is_err() || !matches!(notices.join(), Ok(Ok(()))) {
        // Findings were lost, so the tool's verdict cannot be reported as it stands: 3 is the
        // code of a run that did not finish as reported.
        eprintln!("error: the console printer stopped unexpectedly");
        return ExitCode::from(3);
    }
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code)
        }
    }
}

/// Opens the shell on the registered tools and returns when the window closes: 0, the startup
/// failure's code, or 3 when the window could not be opened.
#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn run_gui_mode(tools: Vec<Box<dyn StudioTool>>) -> ExitCode {
    let (settings, paths) = match startup(&tools) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    // The receiver is dropped at once (the `_` pattern): a tool's run has its own sink
    // (`ToolContext::with_events`), and the notices the tools publish reach the status bar
    // through the requests channel — the shell consumes no run events of its own.
    let (events_tx, _) = unbounded();
    let (requests_tx, requests_rx) = unbounded();
    let ctx = ToolContext::new(
        Arc::new(Mutex::new(settings)),
        paths,
        events_tx,
        requests_tx,
    );
    match run_gui(StudioApp::new(
        tools,
        &PLANNED,
        ctx,
        requests_rx,
        user_config_dir(),
    )) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(3)
        }
    }
}

#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn main() -> ExitCode {
    let tools = tools();
    let launch = match parse_launch(&tools, std::env::args_os()) {
        Ok(launch) => launch,
        Err(error) => error.exit(),
    };
    install_cli_logger(launch.verbosity);
    match launch.mode {
        LaunchMode::Gui => run_gui_mode(tools),
        LaunchMode::GuiAutorun { .. } => {
            eprintln!(
                "`--gui` is not available yet; it arrives with the GUI phase. Run the command without it, or `4cc-studio` alone for the window."
            );
            ExitCode::from(2)
        }
        LaunchMode::Cli { tool, matches } => run_cli_mode(&tools, &tool, &matches),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The registry repeats the tools' own labels and icons: a registered tool's rail glyph is
    // its `icon()`, never a second spelling that could drift.
    #[test]
    fn every_registered_tool_has_a_matching_planned_entry() {
        for tool in tools() {
            let entry = PLANNED
                .iter()
                .find(|entry| entry.id == tool.id())
                .unwrap_or_else(|| panic!("{} is missing from the registry", tool.id()));
            assert_eq!(entry.label, tool.label());
            assert_eq!(entry.icon, tool.icon());
        }
    }

    #[test]
    fn each_verbosity_maps_to_its_level() {
        assert_eq!(log_level(0), LevelFilter::Warn);
        assert_eq!(log_level(1), LevelFilter::Info);
        assert_eq!(log_level(2), LevelFilter::Debug);
        assert_eq!(log_level(3), LevelFilter::Trace);
    }

    // A process can install one logger only, so this is the one test that installs it.
    #[test]
    fn installing_the_logger_sets_the_verbosity_level() {
        install_cli_logger(1);
        // `RUST_LOG` in the environment overrides `-v` by design, so with it set the test can
        // only check that a logger is installed.
        if std::env::var_os("RUST_LOG").is_some() {
            assert_ne!(log::max_level(), LevelFilter::Off);
        } else {
            assert_eq!(log::max_level(), LevelFilter::Info);
        }
    }
}
