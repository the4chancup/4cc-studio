//! The Ruleset editor: an AATF ruleset for your own invitational built through
//! forms and saved as a rules file the Save editor and the Team creator use
//! like the official one (`plans/ruleset_editor.md`).
//!
//! `Tool` registers the tool with the shell: its help chapter and its
//! `ruleset-editor` command line (`validate`, `update-logic`). The window is a
//! later build's; until then `view` shows a placeholder line. `document` owns a
//! rules file open in memory (its ruleset, its logic status and its custom
//! checks as read) and saves it back through the same validation `Rules::load`
//! applies; `summary` renders the ruleset as readable rules text; `cli` drives
//! them headless; `messages` is the finding catalog.

mod cli;
mod document;
mod events;
mod messages;
mod summary;
#[cfg(test)]
mod test_support;

use studio_core::{CliError, HelpSection, HelpTopic, StudioTool, ToolContext};

use crate::messages::TOOL_ID;

/// The Ruleset editor, as registered with the `studio` binary. No window
/// state yet: the view arrives with a later build.
#[derive(Default)]
pub struct Tool;

impl Tool {
    /// The tool.
    pub fn new() -> Tool {
        Tool
    }
}

impl StudioTool for Tool {
    fn id(&self) -> &'static str {
        TOOL_ID
    }

    fn label(&self) -> &'static str {
        "Ruleset editor"
    }

    fn icon(&self) -> &'static str {
        studio_core::egui_phosphor::regular::SCROLL
    }

    fn view(&mut self, ui: &mut egui::Ui, _ctx: &ToolContext) {
        ui.label("The Ruleset editor's window comes in a later build; its command line works now.");
    }

    /// The tool has no settings yet: `settings.rs` arrives with the view.
    fn default_settings(&self) -> toml::Table {
        toml::Table::new()
    }

    fn help(&self) -> HelpSection {
        HelpSection {
            title: "Ruleset editor",
            topics: vec![HelpTopic {
                slug: "command-line",
                title: "Running from a terminal",
                body: include_str!("../help/01_command_line.md"),
            }],
            messages: messages::catalog(),
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
    fn the_tool_is_registered_as_the_ruleset_editor() {
        let tool = Tool::new();
        assert_eq!(tool.id(), "ruleset-editor");
        assert_eq!(tool.label(), "Ruleset editor");
        assert!(tool.default_settings().is_empty());
        assert_eq!(tool.help().topics.len(), 1);
    }
}
