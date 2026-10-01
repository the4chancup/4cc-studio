//! The Team compiler: Studio-format aesthetics exports compiled to CPK archives.
//!
//! `Tool` registers the tool with the shell: its settings section, its help chapter and its
//! `team-compiler` command line, whose preflight refuses an invalid invocation or configuration.
//! `check` runs the structure pass over every export; `compile` runs the same pass, plans the
//! kept exports' face and kit tasks, processes them and writes the CPK.

mod check;
mod cli;
mod compile;
mod events;
mod messages;
mod output;
mod paths;
mod plan;
mod processing;
mod reader;
mod settings;
mod structure;
mod templates;
#[cfg(test)]
mod testing;

use studio_core::{CliError, HelpSection, HelpTopic, StudioTool, ToolContext};

use crate::messages::TOOL_ID;

/// The Team compiler, as registered with the `studio` binary.
pub struct Tool;

impl StudioTool for Tool {
    fn id(&self) -> &'static str {
        TOOL_ID
    }

    fn label(&self) -> &'static str {
        "Team compiler"
    }

    fn view(&mut self, ui: &mut egui::Ui, _ctx: &ToolContext) {
        ui.label("The Team compiler's window is not built yet.");
    }

    fn settings_view(&mut self, ui: &mut egui::Ui) {
        ui.label("The Team compiler's settings are not built yet.");
    }

    fn default_settings(&self) -> toml::Table {
        settings::default_table()
    }

    fn help(&self) -> HelpSection {
        HelpSection {
            title: "Team compiler",
            topics: vec![HelpTopic {
                slug: "command-line",
                title: "Running from a terminal",
                body: include_str!("../help/01_command_line.md"),
            }],
        }
    }

    fn cli_command(&self) -> clap::Command {
        cli::command()
    }

    fn cli_run(&self, matches: &clap::ArgMatches, ctx: &ToolContext) -> Result<u8, CliError> {
        cli::run(matches, ctx)
    }
}
