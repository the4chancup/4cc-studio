//! The Save editor: the PES EDIT savefile's teams and players exchanged (Team TOML, the
//! Team compiler's aesthetics patch, the teams list), transplanted and compared.
//!
//! `Tool` registers the tool with the shell: its settings section, its help chapter and its
//! `save-editor` command line (`spec/save_editor.md` "CLI"), and the window (`view/`, the
//! master-detail of `plans/save_editor.md` "Editing UI"). `pes_savefile` owns every byte and
//! every operation on the model; this crate owns the session (the file that is open) and the
//! command line over it.

mod cli;
mod diff;
mod events;
mod messages;
mod session;
mod settings;
mod teams_list;
#[cfg(test)]
mod test_support;
mod view;

use studio_core::{CliError, HelpSection, HelpTopic, StudioTool, ToolContext};

use crate::messages::TOOL_ID;

/// The Save editor, as registered with the `studio` binary. `view` is the window's
/// state (`plans/save_editor.md` "Editing UI").
#[derive(Default)]
pub struct Tool {
    view: view::SaveView,
}

impl Tool {
    /// The tool, with no save open.
    pub fn new() -> Tool {
        Tool {
            view: view::SaveView::new(),
        }
    }
}

impl StudioTool for Tool {
    fn id(&self) -> &'static str {
        TOOL_ID
    }

    fn label(&self) -> &'static str {
        "Save editor"
    }

    fn icon(&self) -> &'static str {
        studio_core::egui_phosphor::regular::NOTE_PENCIL
    }

    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext) {
        // A settings error counts as an empty setting (the embedded
        // official ruleset).
        let aatf_rules = settings::from_table(&ctx.tool_settings(TOOL_ID))
            .map(|settings| settings.aatf_rules)
            .unwrap_or_default();
        let file = session::rules::rules_file(&aatf_rules, ctx.paths().data_dir.as_deref());
        self.view.hide_ids = ctx.common().fpc_hide_ids();
        view::show(
            ui,
            &mut self.view,
            &studio_core::running_pes,
            file.as_deref(),
        );
    }

    fn settings_view(&mut self, ui: &mut egui::Ui, _ctx: &ToolContext) {
        ui.label(
            "The Save editor has one setting, aatf_rules: the AATF rules file check-aatf \
             runs. Its editor comes with the Save editor's window.",
        );
    }

    fn default_settings(&self) -> toml::Table {
        settings::default_table()
    }

    fn help(&self) -> HelpSection {
        HelpSection {
            title: "Save editor",
            topics: vec![
                HelpTopic {
                    slug: "command-line",
                    title: "Running from a terminal",
                    body: include_str!("../help/01_command_line.md"),
                },
                HelpTopic {
                    slug: "window",
                    title: "The window",
                    body: include_str!("../help/02_window.md"),
                },
                HelpTopic {
                    slug: "abilities",
                    title: "Abilities & Skills",
                    body: include_str!("../help/03_abilities.md"),
                },
                HelpTopic {
                    slug: "appearance",
                    title: "Appearance",
                    body: include_str!("../help/04_appearance.md"),
                },
                HelpTopic {
                    slug: "team",
                    title: "The Team tab and the Team menu",
                    body: include_str!("../help/05_team.md"),
                },
                HelpTopic {
                    slug: "player",
                    title: "The Player menu",
                    body: include_str!("../help/06_player.md"),
                },
                HelpTopic {
                    slug: "savefile",
                    title: "The Savefile menu",
                    body: include_str!("../help/07_savefile.md"),
                },
            ],
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
    fn the_tool_is_registered_as_the_save_editor() {
        let tool = Tool::new();
        assert_eq!(tool.id(), "save-editor");
        assert_eq!(tool.label(), "Save editor");
        assert_eq!(tool.default_settings()["aatf_rules"].as_str(), Some(""));
        assert_eq!(tool.help().topics.len(), 7);
    }
}
