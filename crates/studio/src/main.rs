//! The 4cc Studio binary: registers the tools, then launches the GUI or dispatches the CLI.
//! Until the GUI phase, only the CLI path exists.

use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use log::LevelFilter;
use studio_core::{LaunchMode, Settings, StudioTool, ToolContext, parse_launch, run_cli};

/// The registered tools, in sidebar order. Empty until the first tool crate lands.
fn tools() -> Vec<Box<dyn StudioTool>> {
    Vec::new()
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

#[expect(clippy::print_stderr, reason = "CLI result output is the binary's job")]
fn main() -> ExitCode {
    let tools = tools();
    let launch = match parse_launch(&tools, std::env::args_os()) {
        Ok(launch) => launch,
        Err(error) => error.exit(),
    };
    match launch.mode {
        LaunchMode::Gui | LaunchMode::GuiAutorun { .. } => {
            eprintln!("The GUI is not built yet; see `studio --help` for the CLI.");
            ExitCode::from(2)
        }
        LaunchMode::Cli { tool, matches } => {
            install_cli_logger(launch.verbosity);
            // The receivers are dropped: a CLI event printer arrives with the first tool crate.
            let (events_tx, _events_rx) = unbounded();
            let (requests_tx, _requests_rx) = unbounded();
            let ctx = ToolContext::new(
                Arc::new(Mutex::new(Settings::default())),
                events_tx,
                requests_tx,
            );
            match run_cli(&tools, &tool, &matches, &ctx) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {error:#}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
