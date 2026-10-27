//! The Save editor: the PES EDIT savefile's teams and players exchanged (Team TOML, the
//! Team compiler's aesthetics patch, the teams list) and compared.
//!
//! `Tool` registers the tool with the shell: its settings section, its help chapter and its
//! `save-editor` command line (`spec/save_editor.md` "CLI"), and the window (`view/`: the
//! player list beside the selected row's player or team view). `pes_savefile` owns every byte and
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
use studio_core::{CliError, HelpSection, HelpTopic, RunningPes, StudioTool, ToolContext};

use crate::messages::TOOL_ID;

/// The "Aesthetics from exports" row's hover text in the settings menu.
const AESTHETICS_FROM_EXPORTS_TIP: &str = "On when the cup's looks are compiled from exports: \
     the save's aesthetics are then read-only, behind Edit aesthetics anyway on the Appearance \
     section.";

/// The AATF rules file row's picker (the settings row's `PathKind`):
/// the one ending a member's rules file carries.
const AATF_RULES_PICKER: PathKind = PathKind::File {
    filter: "AATF rules",
    extensions: &[aatf::RULES_FILE_EXTENSION],
};

/// The Save editor, as registered with the `studio` binary. `view` is the window's
/// state.
pub struct Tool {
    view: view::SaveView,
    /// The AATF rules file row's edit buffer in the settings section (`path_row`
    /// commits it to the setting on Enter or focus loss, so a half-typed path
    /// never reaches it).
    aatf_rules: String,
    /// The running-PES probe `Tool::view` hands `view::show`: the real
    /// process list in production, a stub reporting no game under `cargo
    /// test` — the file pickers' `#[cfg]` split
    /// (`comparator::default_open_dialog`). A unit test sets it to drive
    /// the running-game question.
    running: fn() -> Vec<RunningPes>,
}

impl Tool {
    /// The tool, with no save open.
    pub fn new() -> Tool {
        Tool {
            view: view::SaveView::new(),
            aatf_rules: String::new(),
            running: running_probe(),
        }
    }
}

impl Default for Tool {
    fn default() -> Tool {
        Tool::new()
    }
}

/// The running-PES probe: the real process list (`studio_core::running_pes`).
#[cfg(not(test))]
fn running_probe() -> fn() -> Vec<RunningPes> {
    studio_core::running_pes
}

/// `running_probe` under `cargo test`: no game at all — a test never reads
/// the real process list, which would open the Edit-mode question on a
/// machine with the fixture's PES running.
#[cfg(test)]
fn running_probe() -> fn() -> Vec<RunningPes> {
    || Vec::new()
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

    /// The unsaved-changes prompt guards the window's close (SE-GUI-05):
    /// a dirty session holds it and opens the prompt for `Close` (its
    /// Discard re-asks through `request_close`, answered once the session
    /// is dropped). A prompt already up retargets.
    fn hold_close(&mut self) -> bool {
        if self.view.is_dirty() {
            view::prompt::open(&mut self.view, view::prompt::Pending::Close);
            true
        } else {
            false
        }
    }

    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext) {
        // A settings error counts as an empty setting (the embedded
        // official ruleset, Live AATF off, the aesthetics lock off).
        let table = ctx.tool_settings(TOOL_ID);
        let tool_settings = settings::from_table(&table).unwrap_or_default();
        let file =
            session::rules::rules_file(&tool_settings.aatf_rules, ctx.paths().data_dir.as_deref());
        self.view.live_aatf = tool_settings.live_aatf;
        self.view.aesthetics_from_exports = tool_settings.aesthetics_from_exports;
        self.view.tactics.scroll_names = tool_settings.scroll_names;
        let common = ctx.common();
        self.view.hide_ids = common.fpc_hide_ids();
        self.view
            .select_portraits(ui.ctx(), &common.pes_folder(), common.pes_version);
        view::show(ui, &mut self.view, &self.running, file.as_deref());
        if std::mem::take(&mut self.view.request_close) {
            ctx.request_close();
        }
        if let Some(notice) = self.view.notice.take() {
            ctx.notify(notice);
        }
        // The toolbar's toggle and the formation bar's Scroll names are
        // settings: the table as read with only those keys replaced, so
        // any other key survives (as `settings_view` writes `aatf_rules`).
        if self.view.live_aatf != tool_settings.live_aatf
            || self.view.tactics.scroll_names != tool_settings.scroll_names
        {
            let mut table = table;
            table.insert(
                "live_aatf".to_owned(),
                toml::Value::Boolean(self.view.live_aatf),
            );
            table.insert(
                "scroll_names".to_owned(),
                toml::Value::Boolean(self.view.tactics.scroll_names),
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
        let mut aesthetics = table
            .get("aesthetics_from_exports")
            .and_then(toml::Value::as_bool)
            .unwrap_or_default();
        let mut committed = false;
        let mut committed_aesthetics = false;
        egui::Grid::new("save_editor_settings")
            .num_columns(2)
            .show(ui, |ui| {
                committed = path_row(
                    ui,
                    "AATF rules file",
                    "aatf_rules",
                    "The cup's rules file the Save editor checks teams against. Leave it \
                     empty to check against the 4cc's official rules.",
                    &mut self.aatf_rules,
                    &mut current,
                    AATF_RULES_PICKER,
                );
                // The common grid's row shape: the label cell's tooltip
                // leads with the key, the widget's is the sentence alone.
                let label = ui.label("Aesthetics from exports").on_hover_text(format!(
                    "aesthetics_from_exports: {AESTHETICS_FROM_EXPORTS_TIP}"
                ));
                committed_aesthetics = ui
                    .add(egui::Checkbox::without_text(&mut aesthetics))
                    .labelled_by(label.id)
                    .on_hover_text(AESTHETICS_FROM_EXPORTS_TIP)
                    .changed();
                ui.end_row();
            });
        // The table as read with only the changed keys replaced, so any
        // other key survives.
        if committed {
            table.insert("aatf_rules".to_owned(), toml::Value::String(current));
        }
        if committed_aesthetics {
            table.insert(
                "aesthetics_from_exports".to_owned(),
                toml::Value::Boolean(aesthetics),
            );
        }
        if committed || committed_aesthetics {
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
                    title: "The team view and the Team menu",
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
                    title: "The Tactics editor",
                    body: include_str!("../help/08_tactics.md"),
                },
            ],
            messages: messages::catalog(),
        }
    }

    fn cli_command(&self) -> clap::Command {
        cli::command()
    }

    /// The open save's path, bracketed in the idle window title. No view draws it, so the
    /// toolbar keeps its one row.
    fn title_detail(&self) -> Option<String> {
        self.view.save_path().map(|path| path.display().to_string())
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

    /// The AATF rules file row's picker offers `.4ccrules` alone: the
    /// ending every rules file a member handles carries
    /// (`spec/aatf_rules.md` "The rules file").
    #[test]
    fn the_rules_file_row_picks_4ccrules_alone() {
        let PathKind::File { extensions, .. } = AATF_RULES_PICKER else {
            panic!("the rules file row is a file picker")
        };
        assert_eq!(extensions, &["4ccrules"]);
    }
}
