//! The Save editor: the PES EDIT savefile's teams and players exchanged (Team TOML, the
//! Team compiler's aesthetics patch, the teams list), transplanted and compared.
//!
//! `Tool` registers the tool with the shell: its settings section, its help chapter and its
//! `save-editor` command line (`plans/save_editor.md` "CLI"). The window is a later phase's;
//! until then `view` shows a placeholder line. `pes_savefile` owns every byte and every
//! operation on the model; this crate owns the session (the file that is open) and the
//! command line over it.

mod cli;
mod events;
mod messages;
mod session;
mod settings;

use studio_core::{CliError, HelpSection, HelpTopic, StudioTool, ToolContext};

use crate::messages::TOOL_ID;

/// The Save editor, as registered with the `studio` binary. No window state yet: the view
/// arrives with the GUI phase.
#[derive(Default)]
pub struct Tool;

impl Tool {
    /// The tool, with no save open.
    pub fn new() -> Tool {
        Tool
    }
}

impl StudioTool for Tool {
    fn id(&self) -> &'static str {
        TOOL_ID
    }

    fn label(&self) -> &'static str {
        "Save editor"
    }

    fn view(&mut self, ui: &mut egui::Ui, _ctx: &ToolContext) {
        ui.label("The Save editor's window is not built yet; its commands run from a terminal.");
    }

    fn settings_view(&mut self, ui: &mut egui::Ui) {
        ui.label("The Save editor has no settings yet.");
    }

    fn default_settings(&self) -> toml::Table {
        settings::default_table()
    }

    fn help(&self) -> HelpSection {
        HelpSection {
            title: "Save editor",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_is_registered_as_the_save_editor() {
        let tool = Tool::new();
        assert_eq!(tool.id(), "save-editor");
        assert_eq!(tool.label(), "Save editor");
        assert!(tool.default_settings().is_empty());
        assert_eq!(tool.help().topics.len(), 1);
    }
}
