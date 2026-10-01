//! The 4cc Studio binary: registers the tools, then launches the GUI or dispatches the CLI.
//! Until the GUI phase, only the CLI path exists.

mod console;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use clap::ArgMatches;
use crossbeam_channel::unbounded;
use log::LevelFilter;
use studio_core::{
    AppPaths, LaunchMode, SETTINGS_FILE_NAME, Settings, StudioTool, ToolContext, parse_launch,
    resolve_data_dir, run_cli, user_config_dir,
};

use crate::console::spawn_printer;

/// The registered tools, in sidebar order.
fn tools() -> Vec<Box<dyn StudioTool>> {
    vec![Box::new(team_compiler::Tool)]
}

/// The CLI diagnostic sink (core plan, "Diagnostic logging"): `-v` sets the level, `RUST_LOG`
/// overrides it. The GUI mode gets its own file sink in the GUI phase.
fn install_cli_logger(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter_level(level)
        .parse_default_env()
        .init();
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

/// Runs one tool subcommand headless and returns the process exit code: the tool's own, 2 when
/// the settings file cannot be loaded (a configuration error: nothing ran), or 3 when the
/// executable's folder cannot be found or the console printer failed (environment failures;
/// `core/architecture.md`, the binary's own codes).
#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn run_cli_mode(tools: &[Box<dyn StudioTool>], tool: &str, matches: &ArgMatches) -> ExitCode {
    let exe_dir = match exe_dir() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("error: {error:#}");
            return ExitCode::from(3);
        }
    };
    let (settings, paths) = match load_settings(tools, exe_dir) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("error: {error:#}");
            return ExitCode::from(2);
        }
    };
    let (events_tx, events_rx) = unbounded();
    let (requests_tx, _requests_rx) = unbounded();
    let printer = spawn_printer(events_rx);
    let ctx = ToolContext::new(
        Arc::new(Mutex::new(settings)),
        paths,
        events_tx,
        requests_tx,
    );
    let result = run_cli(tools, tool, matches, &ctx);
    // The printer ends when every sender is gone, and the context holds them.
    drop(ctx);
    if printer.join().is_err() {
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

#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn main() -> ExitCode {
    let tools = tools();
    let launch = match parse_launch(&tools, std::env::args_os()) {
        Ok(launch) => launch,
        Err(error) => error.exit(),
    };
    match launch.mode {
        LaunchMode::Gui | LaunchMode::GuiAutorun { .. } => {
            eprintln!("The GUI is not built yet; see `4cc-studio --help` for the CLI.");
            ExitCode::from(2)
        }
        LaunchMode::Cli { tool, matches } => {
            install_cli_logger(launch.verbosity);
            run_cli_mode(&tools, &tool, &matches)
        }
    }
}
