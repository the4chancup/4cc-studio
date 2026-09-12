//! Argument parsing for the three launch modes (core plan, "Launch modes"):
//!
//! | Invocation | Mode |
//! |---|---|
//! | `studio` | GUI, last-used tool active |
//! | `studio <tool-id> <command> [args]` | headless CLI |
//! | `studio --gui <tool-id> <command> [args]` | GUI autorun: the GUI opens on the tool and runs the command |
//!
//! Every tool contributes one clap subcommand named after its id; `--gui` is the shell's own flag,
//! parsed here so no tool's command knows about it.

use std::ffi::OsString;

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::tool::{StudioTool, ToolContext};

/// How the binary was started.
#[derive(Debug, Clone)]
pub enum LaunchMode {
    /// No arguments: open the GUI.
    Gui,
    /// A tool subcommand: run it headless.
    Cli {
        /// The tool's id.
        tool: String,
        /// The tool subcommand's parsed arguments.
        matches: ArgMatches,
    },
    /// `--gui` plus a tool subcommand: open the GUI on the tool and run the command inside it.
    GuiAutorun {
        /// The tool's id.
        tool: String,
        /// The tool subcommand's parsed arguments.
        matches: ArgMatches,
    },
}

/// The root `studio` command with one subcommand per registered tool.
pub fn root_command(tools: &[Box<dyn StudioTool>]) -> Command {
    let mut command = Command::new("studio")
        .about("4cc Studio: the 4cc community's PES tools")
        .arg(
            Arg::new("gui")
                .long("gui")
                .action(ArgAction::SetTrue)
                .help("Open the GUI on the tool and run the command inside it"),
        );
    for tool in tools {
        command = command.subcommand(tool.cli_command().name(tool.id()).about(tool.label()));
    }
    command
}

/// Parses the process arguments (`args[0]` included) into a launch mode. A clap error carries
/// its own help or usage text; the binary prints it with `Error::exit`.
pub fn parse_launch(
    tools: &[Box<dyn StudioTool>],
    args: impl IntoIterator<Item = OsString>,
) -> Result<LaunchMode, clap::Error> {
    let matches = root_command(tools).try_get_matches_from(args)?;
    let gui = matches.get_flag("gui");
    Ok(match matches.subcommand() {
        None => LaunchMode::Gui,
        Some((tool, sub)) if gui => LaunchMode::GuiAutorun {
            tool: tool.to_owned(),
            matches: sub.clone(),
        },
        Some((tool, sub)) => LaunchMode::Cli {
            tool: tool.to_owned(),
            matches: sub.clone(),
        },
    })
}

/// Runs a parsed CLI subcommand on the tool that owns it.
pub fn run_cli(
    tools: &[Box<dyn StudioTool>],
    tool_id: &str,
    matches: &ArgMatches,
    ctx: &ToolContext,
) -> anyhow::Result<()> {
    let tool = tools
        .iter()
        .find(|tool| tool.id() == tool_id)
        .ok_or_else(|| anyhow::anyhow!("no tool with id `{tool_id}`"))?;
    tool.cli_run(matches, ctx)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crossbeam_channel::unbounded;

    use super::*;
    use crate::help::HelpSection;
    use crate::settings::Settings;
    use crate::status::Notice;
    use crate::tool::ShellRequest;

    struct StubTool;

    impl StudioTool for StubTool {
        fn id(&self) -> &'static str {
            "stub"
        }
        fn label(&self) -> &'static str {
            "Stub"
        }
        fn view(&mut self, _ui: &mut egui::Ui, _ctx: &ToolContext) {}
        fn settings_view(&mut self, _ui: &mut egui::Ui) {}
        fn default_settings(&self) -> toml::Table {
            toml::Table::new()
        }
        fn help(&self) -> HelpSection {
            HelpSection {
                title: "Stub",
                topics: Vec::new(),
            }
        }
        fn cli_command(&self) -> Command {
            Command::new("ignored-name")
                .subcommand(Command::new("ping").arg(Arg::new("what").required(true)))
        }
        fn cli_run(&self, matches: &ArgMatches, ctx: &ToolContext) -> anyhow::Result<()> {
            let Some(("ping", ping)) = matches.subcommand() else {
                anyhow::bail!("unknown command");
            };
            let what: &String = ping.get_one("what").unwrap();
            ctx.notify(Notice {
                text: format!("pong {what}"),
                action: None,
            });
            Ok(())
        }
    }

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    fn tools() -> Vec<Box<dyn StudioTool>> {
        vec![Box::new(StubTool)]
    }

    #[test]
    fn no_arguments_means_gui() {
        assert!(matches!(
            parse_launch(&tools(), args(&["studio"])),
            Ok(LaunchMode::Gui)
        ));
    }

    #[test]
    fn tool_subcommand_is_named_after_the_tool_id() {
        let launch = parse_launch(&tools(), args(&["studio", "stub", "ping", "x"])).unwrap();
        assert!(matches!(launch, LaunchMode::Cli { tool, .. } if tool == "stub"));
        assert!(parse_launch(&tools(), args(&["studio", "ignored-name", "ping", "x"])).is_err());
    }

    #[test]
    fn gui_flag_makes_it_an_autorun() {
        let launch =
            parse_launch(&tools(), args(&["studio", "--gui", "stub", "ping", "x"])).unwrap();
        assert!(matches!(launch, LaunchMode::GuiAutorun { tool, .. } if tool == "stub"));
    }

    #[test]
    fn unknown_tool_and_missing_argument_are_clap_errors() {
        assert!(parse_launch(&tools(), args(&["studio", "nope"])).is_err());
        assert!(parse_launch(&tools(), args(&["studio", "stub", "ping"])).is_err());
    }

    #[test]
    fn cli_dispatch_reaches_the_tool_with_its_context() {
        let tools = tools();
        let LaunchMode::Cli { tool, matches } =
            parse_launch(&tools, args(&["studio", "stub", "ping", "there"])).unwrap()
        else {
            panic!("expected CLI mode");
        };
        let (events_tx, _events_rx) = unbounded();
        let (requests_tx, requests_rx) = unbounded();
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(Settings::default())),
            events_tx,
            requests_tx,
        );
        run_cli(&tools, &tool, &matches, &ctx).unwrap();
        assert_eq!(
            requests_rx.try_recv().unwrap(),
            ShellRequest::Notify(Notice {
                text: "pong there".to_owned(),
                action: None
            })
        );
        assert!(run_cli(&tools, "missing", &matches, &ctx).is_err());
    }
}
