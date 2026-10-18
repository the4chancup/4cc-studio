//! The `export-upgrader` command line (`plans/export_upgrader.md` "CLI"): a
//! parsed command turned into a run, and the run's outcome turned into the
//! exit code scripts read.

use std::path::PathBuf;

use clap::{Args, Command, FromArgMatches, Subcommand};
use studio_core::{CliError, ToolContext};

use crate::messages::TOOL_ID;
use crate::upgrade;

// The exit codes are the command line's contract with scripts (the plan's
// "CLI"; the Team compiler's mapping).
/// Exit code of a run that finished with no Error finding and nothing
/// dropped.
pub(crate) const CLEAN: u8 = 0;
/// Exit code of a run that finished, but reported an Error finding or one
/// that drops something (unmigrated content, the unknown team, an export that
/// does not extract).
pub(crate) const ERRORS: u8 = 1;
/// Exit code of an invalid invocation: nothing was written.
pub(crate) const INVALID: u8 = 2;
/// Exit code of a run aborted: before anything is written (the output folder
/// exists or cannot be created, the teams list cannot be read), or when a
/// file of the output cannot be written, the incomplete output left.
pub(crate) const ABORTED: u8 = 3;

#[derive(Debug, Subcommand)]
enum ExportUpgraderCommand {
    /// Migrate an old (AET) export to the Studio format.
    Upgrade(UpgradeArgs),
}

/// `upgrade <export> (--savefile <EDIT> | --loose) [-o <dir>]`.
#[derive(Debug, Args)]
pub(crate) struct UpgradeArgs {
    /// The old export: a folder, a .zip or a .7z.
    pub(crate) export: PathBuf,
    /// The save (EDIT00000000) whose records say which player wears which
    /// boots and gloves.
    #[arg(long, value_name = "EDIT")]
    pub(crate) savefile: Option<PathBuf>,
    /// No savefile: the wearers are inferred from the IDs' positions in the
    /// team's old block.
    #[arg(long)]
    pub(crate) loose: bool,
    /// The folder the output goes in, created when missing; the input's own
    /// folder by default.
    #[arg(short, long, value_name = "DIR")]
    pub(crate) output: Option<PathBuf>,
}

/// The `export-upgrader` subcommand and its own subcommands.
pub(crate) fn command() -> Command {
    ExportUpgraderCommand::augment_subcommands(Command::new(TOOL_ID).subcommand_required(true))
}

/// Runs a parsed `export-upgrader` command.
pub(crate) fn run(matches: &clap::ArgMatches, ctx: &ToolContext) -> Result<u8, CliError> {
    let command = ExportUpgraderCommand::from_arg_matches(matches)
        .map_err(|error| CliError::new(INVALID, error))?;
    match command {
        ExportUpgraderCommand::Upgrade(args) => upgrade::run(&args, ctx),
    }
}
