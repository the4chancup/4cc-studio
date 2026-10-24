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

use studio_core::widgets::{PathKind, path_row};
use studio_core::{CliError, HelpSection, HelpTopic, StudioTool, ToolContext};

use crate::messages::TOOL_ID;

/// The Save editor, as registered with the `studio` binary. `view` is the window's
/// state (`plans/save_editor.md` "Editing UI").
#[derive(Default)]
pub struct Tool {
    view: view::SaveView,
    /// The AATF rules file row's edit buffer in the settings section (`path_row`
    /// commits it to the setting on Enter or focus loss, so a half-typed path
    /// never reaches it).
    aatf_rules: String,
}

impl Tool {
    /// The tool, with no save open.
    pub fn new() -> Tool {
        Tool {
            view: view::SaveView::new(),
            aatf_rules: String::new(),
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
        studio_core::icons::NOTE_PENCIL
    }

    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext) {
        // A settings error counts as an empty setting (the embedded
        // official ruleset, Live AATF off).
        let table = ctx.tool_settings(TOOL_ID);
        let tool_settings = settings::from_table(&table).unwrap_or_default();
        let file =
            session::rules::rules_file(&tool_settings.aatf_rules, ctx.paths().data_dir.as_deref());
        self.view.live_aatf = tool_settings.live_aatf;
        let common = ctx.common();
        self.view.hide_ids = common.fpc_hide_ids();
        self.view
            .select_portraits(ui.ctx(), &common.pes_folder(), common.pes_version);
        view::show(
            ui,
            &mut self.view,
            &studio_core::running_pes,
            file.as_deref(),
        );
        // The toolbar's toggle is a setting: the table as read with only
        // `live_aatf` replaced, so any other key survives (as
        // `settings_view` writes `aatf_rules`).
        if self.view.live_aatf != tool_settings.live_aatf {
            let mut table = table;
            table.insert(
                "live_aatf".to_owned(),
                toml::Value::Boolean(self.view.live_aatf),
            );
            ctx.set_tool_settings(TOOL_ID, table);
        }
    }

    fn settings_view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext) {
        let mut table = ctx.tool_settings(TOOL_ID);
        let mut current = table
            .get("aatf_rules")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let mut committed = false;
        egui::Grid::new("save_editor_settings")
            .num_columns(2)
            .show(ui, |ui| {
                committed = path_row(
                    ui,
                    "AATF rules file",
                    "aatf_rules",
                    "The AATF rules file check-aatf and the AATF panel run; empty: the \
                     official ruleset, built in.",
                    &mut self.aatf_rules,
                    &mut current,
                    PathKind::File {
                        filter: "AATF rules",
                        extensions: &["rhai"],
                    },
                );
            });
        // The table as read with only `aatf_rules` replaced, so any other key
        // survives.
        if committed {
            table.insert("aatf_rules".to_owned(), toml::Value::String(current));
            ctx.set_tool_settings(TOOL_ID, table);
        }
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
                    title: "The team bar, Team settings and the Team menu",
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
                HelpTopic {
                    slug: "tactics",
                    title: "The Tactics tab",
                    body: include_str!("../help/08_tactics.md"),
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
        assert_eq!(tool.help().topics.len(), 8);
    }
}
