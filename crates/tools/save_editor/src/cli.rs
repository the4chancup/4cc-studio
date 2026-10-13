//! The `save-editor` command line (`plans/save_editor.md` "CLI"): the batch surface for the
//! operations the cups script. Every command reads an explicit save; a refusal (a bad
//! argument, an unreadable file, a team the save lacks) comes before anything is written.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use clap::{ArgGroup, Args, Command, FromArgMatches, Subcommand};
use studio_core::{CliError, Severity, ToolContext};

use crate::events::Report;
use crate::messages::{Code, TOOL_ID, message};
use crate::session::Session;

// The exit codes are the command line's contract with scripts, the Team compiler's meanings.
/// Exit code of a command that did its work with no Error finding.
const DONE: u8 = 0;
/// Exit code of a command that did its work with an Error finding (a patch's team the save
/// lacks).
const ERRORS: u8 = 1;
/// Exit code of a command refused before anything was written.
const INVALID: u8 = 2;
/// Exit code of a command whose write failed; what was already written stands.
const WRITE_FAILED: u8 = 3;

#[derive(Debug, Subcommand)]
enum SaveEditorCommand {
    /// Write one team of a save as a Team TOML file, or every team as one file each.
    ExportToml(ExportTomlArgs),
}

/// `export-toml <save> --team <id> --out <file>` or `--all-teams --out-dir <folder>`: the
/// two forms are exclusive, one is required, and each takes its own destination.
#[derive(Debug, Args)]
#[command(group = ArgGroup::new("selection").required(true).args(["team", "all_teams"]))]
struct ExportTomlArgs {
    /// The save to read (EDIT00000000).
    save: PathBuf,
    /// The one team to write, by id.
    #[arg(long, value_name = "ID", requires = "out")]
    team: Option<u32>,
    /// The file the one team goes to; replaced when it exists.
    #[arg(short, long, value_name = "FILE", requires = "team")]
    out: Option<PathBuf>,
    /// Every team of the save, one <id>.toml each.
    #[arg(long, requires = "out_dir")]
    all_teams: bool,
    /// The folder the <id>.toml files go in; created when missing, its files replaced.
    #[arg(long, value_name = "FOLDER", requires = "all_teams")]
    out_dir: Option<PathBuf>,
}

/// The `save-editor` subcommand and its own subcommands.
pub(crate) fn command() -> Command {
    SaveEditorCommand::augment_subcommands(Command::new(TOOL_ID).subcommand_required(true))
}

/// Runs a parsed `save-editor` command.
pub(crate) fn run(matches: &clap::ArgMatches, ctx: &ToolContext) -> Result<u8, CliError> {
    let command = SaveEditorCommand::from_arg_matches(matches)
        .map_err(|error| CliError::new(INVALID, error))?;
    let mut report = Report::new(ctx);
    match command {
        SaveEditorCommand::ExportToml(args) => export_toml(&args, &mut report)?,
    }
    Ok(exit_code(report.worst()))
}

/// `ERRORS` when a finding was an Error, else `DONE`, warnings and notes included.
fn exit_code(worst: Option<Severity>) -> u8 {
    match worst {
        Some(Severity::Error | Severity::Fatal) => ERRORS,
        Some(Severity::Warning | Severity::Info) | None => DONE,
    }
}

/// A refusal before anything was written.
fn invalid(error: anyhow::Error) -> CliError {
    CliError::new(INVALID, error)
}

/// Opens the save a command reads; an unreadable one is a refusal.
fn open(path: &Path) -> Result<Session, CliError> {
    Session::open(path).map_err(invalid)
}

/// Writes `text` to `path`, replacing an existing file; a failure names the file.
fn write_text(path: &Path, text: &str) -> Result<(), CliError> {
    fs::write(path, text).map_err(|error| {
        CliError::new(
            WRITE_FAILED,
            anyhow!("{}: cannot write the file: {error}", path.display()),
        )
    })
}

/// `export-toml`: every team's text is built before the first file is written, so a team
/// that cannot be exported refuses the command with nothing written.
fn export_toml(args: &ExportTomlArgs, report: &mut Report) -> Result<(), CliError> {
    let session = open(&args.save)?;
    let files: Vec<(u32, PathBuf, String)> = match (args.team, &args.out, &args.out_dir) {
        (Some(id), Some(out), _) => {
            vec![(id, out.clone(), session.export_team(id).map_err(invalid)?)]
        }
        (None, _, Some(out_dir)) => {
            let mut files = Vec::new();
            for id in session.team_ids() {
                let path = out_dir.join(format!("{id}.toml"));
                files.push((id, path, session.export_team(id).map_err(invalid)?));
            }
            fs::create_dir_all(out_dir).map_err(|error| {
                CliError::new(
                    WRITE_FAILED,
                    anyhow!("{}: cannot create the folder: {error}", out_dir.display()),
                )
            })?;
            files
        }
        // clap requires one form and each form's destination.
        (Some(_), None, _) | (None, _, None) => {
            return Err(invalid(anyhow!(
                "export-toml takes --team <id> with --out <file>, or --all-teams with --out-dir <folder>"
            )));
        }
    };
    for (id, path, text) in files {
        write_text(&path, &text)?;
        report.message(message(
            Code::TeamExported,
            vec![
                ("team", id.to_string()),
                ("path", path.display().to_string()),
            ],
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exit_code_is_one_for_an_error_and_zero_otherwise() {
        assert_eq!(exit_code(None), 0);
        assert_eq!(exit_code(Some(Severity::Info)), 0);
        assert_eq!(exit_code(Some(Severity::Warning)), 0);
        assert_eq!(exit_code(Some(Severity::Error)), 1);
        assert_eq!(exit_code(Some(Severity::Fatal)), 1);
    }

    #[test]
    fn export_toml_requires_one_form_with_its_own_destination() {
        let parse = |args: &[&str]| {
            command()
                .try_get_matches_from(std::iter::once("save-editor").chain(args.iter().copied()))
        };
        assert!(parse(&["export-toml", "EDIT", "--team", "714", "--out", "a.toml"]).is_ok());
        assert!(parse(&["export-toml", "EDIT", "--all-teams", "--out-dir", "teams"]).is_ok());
        for bad in [
            vec!["export-toml", "EDIT"],
            vec!["export-toml", "EDIT", "--team", "714"],
            vec!["export-toml", "EDIT", "--all-teams"],
            vec!["export-toml", "EDIT", "--team", "714", "--out-dir", "teams"],
            vec![
                "export-toml",
                "EDIT",
                "--team",
                "714",
                "--out",
                "a.toml",
                "--all-teams",
                "--out-dir",
                "t",
            ],
        ] {
            assert!(parse(&bad).is_err(), "{bad:?}");
        }
    }
}
