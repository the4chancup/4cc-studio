//! Argument parsing for the three launch modes (core plan, "Launch modes"):
//!
//! | Invocation | Mode |
//! |---|---|
//! | `4cc-studio` | GUI, last-used tool active |
//! | `4cc-studio <tool-id> <command> [args]` | headless CLI |
//! | `4cc-studio --gui <tool-id> <command> [args]` | GUI autorun: the GUI opens on the tool and runs the command |
//!
//! Every tool contributes one clap subcommand named after its id; `--gui` and `-v` are the
//! shell's own flags, parsed here so no tool's command knows about them.

use std::ffi::OsString;

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::settings::COMMON_KEY;
use crate::tool::{CliError, StudioTool, ToolContext};

/// The parsed command line: the mode plus the shell's own flags.
#[derive(Debug, Clone)]
pub struct Launch {
    /// How the binary was started.
    pub mode: LaunchMode,
    /// Diagnostic verbosity: `0` warnings only, `1` (`-v`) info, `2` (`-vv`) debug, more is trace.
    /// `RUST_LOG`, when set, overrides it (core plan, "Diagnostic logging").
    pub verbosity: u8,
}

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

/// The root `4cc-studio` command with one subcommand per registered tool.
///
/// # Panics
/// If two tools share an id, or a tool uses the reserved settings key `common` as its id: the
/// registry is a compile-time list, so this is a programming error caught at startup.
pub fn root_command(tools: &[Box<dyn StudioTool>]) -> Command {
    let mut command = Command::new("4cc-studio")
        .about("4cc Studio: the 4cc community's PES tools")
        .arg(
            Arg::new("gui")
                .long("gui")
                .action(ArgAction::SetTrue)
                .help("Open the GUI on the tool and run the command inside it"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .action(ArgAction::Count)
                .global(true)
                .help("Diagnostic output: -v info, -vv debug, -vvv trace"),
        );
    for (index, tool) in tools.iter().enumerate() {
        let id = tool.id();
        assert_ne!(
            id, COMMON_KEY,
            "`{COMMON_KEY}` is the common settings key, not a tool id"
        );
        assert!(
            tools[..index].iter().all(|other| other.id() != id),
            "two tools are registered with the id `{id}`"
        );
        command = command.subcommand(tool.cli_command().name(id).about(tool.label()));
    }
    command
}

/// Parses the process arguments (`args[0]` included). A clap error carries its own help or usage
/// text; the binary prints it with `Error::exit`.
pub fn parse_launch(
    tools: &[Box<dyn StudioTool>],
    args: impl IntoIterator<Item = OsString>,
) -> Result<Launch, clap::Error> {
    let matches = root_command(tools).try_get_matches_from(args)?;
    let gui = matches.get_flag("gui");
    let verbosity = matches.get_count("verbose");
    let mode = match matches.subcommand() {
        None => LaunchMode::Gui,
        Some((tool, sub)) if gui => LaunchMode::GuiAutorun {
            tool: tool.to_owned(),
            matches: sub.clone(),
        },
        Some((tool, sub)) => LaunchMode::Cli {
            tool: tool.to_owned(),
            matches: sub.clone(),
        },
    };
    Ok(Launch { mode, verbosity })
}

/// Runs a parsed CLI subcommand on the tool that owns it and returns what the tool returned
/// (its exit code, or its refusal) unchanged. An id no tool has is refused with exit code 2.
pub fn run_cli(
    tools: &[Box<dyn StudioTool>],
    tool_id: &str,
    matches: &ArgMatches,
    ctx: &ToolContext,
) -> Result<u8, CliError> {
    let tool = tools
        .iter()
        .find(|tool| tool.id() == tool_id)
        .ok_or_else(|| CliError::new(2, anyhow::anyhow!("no tool with id `{tool_id}`")))?;
    tool.cli_run(matches, ctx)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use crossbeam_channel::unbounded;

    use super::*;
    use crate::help::HelpSection;
    use crate::settings::Settings;
    use crate::status::Notice;
    use crate::tool::{AppPaths, ShellRequest};

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
                .subcommand(Command::new("verdict"))
                .subcommand(Command::new("refuse"))
        }
        fn cli_run(&self, matches: &ArgMatches, ctx: &ToolContext) -> Result<u8, CliError> {
            match matches.subcommand() {
                Some(("ping", ping)) => {
                    let what: &String = ping.get_one("what").unwrap();
                    ctx.notify(Notice {
                        text: format!("pong {what}"),
                        action: None,
                    });
                    Ok(0)
                }
                Some(("verdict", _)) => Ok(1),
                Some(("refuse", _)) => Err(CliError::new(3, anyhow::anyhow!("refused"))),
                _ => Err(CliError::new(2, anyhow::anyhow!("unknown command"))),
            }
        }
    }

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    fn tools() -> Vec<Box<dyn StudioTool>> {
        vec![Box::new(StubTool)]
    }

    fn test_context() -> (ToolContext, crossbeam_channel::Receiver<ShellRequest>) {
        let (events_tx, _events_rx) = unbounded();
        let (requests_tx, requests_rx) = unbounded();
        let paths = AppPaths {
            exe_dir: PathBuf::from("exe"),
            data_dir: None,
        };
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(Settings::default())),
            paths,
            events_tx,
            requests_tx,
        );
        (ctx, requests_rx)
    }

    fn cli_matches(tools: &[Box<dyn StudioTool>], list: &[&str]) -> (String, ArgMatches) {
        let LaunchMode::Cli { tool, matches } = parse_launch(tools, args(list)).unwrap().mode
        else {
            panic!("expected CLI mode");
        };
        (tool, matches)
    }

    fn mode(list: &[&str]) -> LaunchMode {
        parse_launch(&tools(), args(list)).unwrap().mode
    }

    #[test]
    fn no_arguments_means_gui() {
        assert!(matches!(mode(&["studio"]), LaunchMode::Gui));
    }

    #[test]
    fn tool_subcommand_is_named_after_the_tool_id() {
        assert!(
            matches!(mode(&["studio", "stub", "ping", "x"]), LaunchMode::Cli { tool, .. } if tool == "stub")
        );
        assert!(parse_launch(&tools(), args(&["studio", "ignored-name", "ping", "x"])).is_err());
    }

    #[test]
    fn gui_flag_makes_it_an_autorun() {
        let launch = mode(&["studio", "--gui", "stub", "ping", "x"]);
        assert!(matches!(launch, LaunchMode::GuiAutorun { tool, .. } if tool == "stub"));
    }

    #[test]
    fn verbosity_counts_and_is_accepted_after_the_subcommand() {
        assert_eq!(
            parse_launch(&tools(), args(&["studio"])).unwrap().verbosity,
            0
        );
        assert_eq!(
            parse_launch(&tools(), args(&["studio", "-vv"]))
                .unwrap()
                .verbosity,
            2
        );
        let launch = parse_launch(&tools(), args(&["studio", "stub", "-v", "ping", "x"])).unwrap();
        assert_eq!(launch.verbosity, 1);
        assert!(matches!(launch.mode, LaunchMode::Cli { .. }));
    }

    #[test]
    #[should_panic(expected = "two tools are registered")]
    fn duplicate_tool_ids_are_a_programming_error() {
        let twice: Vec<Box<dyn StudioTool>> = vec![Box::new(StubTool), Box::new(StubTool)];
        root_command(&twice);
    }

    #[test]
    fn unknown_tool_and_missing_argument_are_clap_errors() {
        assert!(parse_launch(&tools(), args(&["studio", "nope"])).is_err());
        assert!(parse_launch(&tools(), args(&["studio", "stub", "ping"])).is_err());
    }

    #[test]
    fn cli_dispatch_reaches_the_tool_with_its_context() {
        let tools = tools();
        let (tool, matches) = cli_matches(&tools, &["studio", "stub", "ping", "there"]);
        let (ctx, requests_rx) = test_context();
        assert_eq!(run_cli(&tools, &tool, &matches, &ctx).unwrap(), 0);
        assert_eq!(
            requests_rx.try_recv().unwrap(),
            ShellRequest::Notify(Notice {
                text: "pong there".to_owned(),
                action: None
            })
        );
    }

    #[test]
    fn unknown_tool_id_is_refused_with_exit_code_2() {
        let tools = tools();
        let (_, matches) = cli_matches(&tools, &["studio", "stub", "verdict"]);
        let (ctx, _requests_rx) = test_context();
        let error = run_cli(&tools, "missing", &matches, &ctx).unwrap_err();
        assert_eq!(error.exit_code, 2);
        assert_eq!(error.to_string(), "no tool with id `missing`");
    }

    #[test]
    fn the_tools_exit_code_and_refusal_pass_through_unchanged() {
        let tools = tools();
        let (ctx, _requests_rx) = test_context();
        let (tool, verdict) = cli_matches(&tools, &["studio", "stub", "verdict"]);
        assert_eq!(run_cli(&tools, &tool, &verdict, &ctx).unwrap(), 1);
        let (tool, refuse) = cli_matches(&tools, &["studio", "stub", "refuse"]);
        let error = run_cli(&tools, &tool, &refuse, &ctx).unwrap_err();
        assert_eq!(error.exit_code, 3);
        assert_eq!(error.to_string(), "refused");
    }
}
