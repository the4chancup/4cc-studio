//! The Team compiler: Studio-format aesthetics exports compiled to CPK archives.
//!
//! `Tool` registers the tool with the shell: its window (`view`, over the run log in
//! `gui_run`), its settings section, its help chapter and its `team-compiler` command line,
//! whose preflight refuses an invalid invocation or configuration. `check` runs validation (the
//! structure pass, then the deep pass over file contents) on every export; `compile` runs the
//! same validation, plans the kept exports' face and kit tasks, processes them and writes the
//! CPK.

mod bins;
mod check;
mod cli;
mod compile;
mod deep;
mod events;
mod face_diff;
mod gui_run;
mod kit_variants;
mod messages;
mod output;
mod paths;
mod plan;
mod processing;
mod reader;
mod settings;
mod templates;
#[cfg(test)]
mod testing;
mod validation;
mod view;

use studio_core::{CliError, HelpSection, HelpTopic, StudioTool, ToolContext};

use crate::gui_run::RunLog;
use crate::messages::TOOL_ID;

/// The Team compiler, as registered with the `studio` binary. Its one piece of window state is
/// the run log: the `compile` the Compile button started, and its lines.
#[derive(Default)]
pub struct Tool {
    run_log: RunLog,
}

impl Tool {
    /// The tool with no run started.
    pub fn new() -> Tool {
        Tool::default()
    }
}

impl StudioTool for Tool {
    fn id(&self) -> &'static str {
        TOOL_ID
    }

    fn label(&self) -> &'static str {
        "Team compiler"
    }

    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext) {
        if view::show(ui, ctx, &self.run_log) {
            self.run_log.start_compile(ctx);
        }
    }

    fn tick(&mut self, _ctx: &ToolContext) {
        self.run_log.poll();
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
