//! The `save-editor` command line (`plans/save_editor.md` "CLI"): the batch surface for the
//! operations the cups script. Every command reads an explicit save; a refusal (a bad
//! argument, an unreadable file, a team the save lacks) comes before anything is written.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use clap::{ArgGroup, Args, Command, FromArgMatches, Subcommand};
use pes_savefile::interchange::aesthetics_patch::AestheticsPatch;
use pes_savefile::interchange::team_toml::TeamToml;
use studio_core::{CliError, Severity, ToolContext};

use crate::events::Report;
use crate::messages::{Code, TOOL_ID, import_note, message, patch_skip};
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
    /// Patch a save's team from a Team TOML file: the team its [team] id names takes the
    /// file's keys, the rest of the save is untouched.
    ImportToml(ApplyArgs),
    /// Apply the Team compiler's aesthetics patch (its players' motions and names) to a save.
    ApplyPatch(ApplyArgs),
}

/// `<file> --base <save> --out <save>`: the file to apply, the save it is applied to, and
/// where the result goes (`--out` may be `--base`; an existing file there is first copied to
/// `<out>.bak`).
#[derive(Debug, Args)]
struct ApplyArgs {
    /// The Team TOML or aesthetics patch to apply.
    file: PathBuf,
    /// The save to read (EDIT00000000); never written unless it is also --out.
    #[arg(long, value_name = "SAVE")]
    base: PathBuf,
    /// Where the patched save goes; an existing file there is kept as <out>.bak.
    #[arg(long, value_name = "SAVE")]
    out: PathBuf,
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
        SaveEditorCommand::ImportToml(args) => import_toml(&args, &mut report)?,
        SaveEditorCommand::ApplyPatch(args) => apply_patch(&args, &mut report)?,
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

/// Reads the text file a command applies; an unreadable one is a refusal naming it.
fn read_text(path: &Path) -> Result<String, CliError> {
    fs::read_to_string(path)
        .map_err(|error| invalid(anyhow!("{}: cannot read the file: {error}", path.display())))
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

/// Writes the session's save to `out`; a failure is the write-failed code.
fn write_save(session: &Session, out: &Path) -> Result<(), CliError> {
    session
        .write(out)
        .map_err(|error| CliError::new(WRITE_FAILED, error))
}

/// `import-toml`: the document is parsed and applied in memory first, so a document the lib
/// refuses (a text too long, a key it does not know) refuses the command with `--out`
/// unwritten; each note is one line.
fn import_toml(args: &ApplyArgs, report: &mut Report) -> Result<(), CliError> {
    let document = TeamToml::parse(&read_text(&args.file)?)
        .map_err(|error| invalid(anyhow!("{}: {error}", args.file.display())))?;
    let mut session = open(&args.base)?;
    let notes = session.import_team(&document).map_err(invalid)?;
    for note in &notes {
        report.message(import_note(note));
    }
    write_save(&session, &args.out)
}

/// `apply-patch`: a version or allocation-scheme mismatch (and a pre-Fox patch) is refused
/// before anything is touched; a team or player the save lacks is an Error finding and the
/// rest is written.
fn apply_patch(args: &ApplyArgs, report: &mut Report) -> Result<(), CliError> {
    let patch = AestheticsPatch::parse(&read_text(&args.file)?)
        .map_err(|error| invalid(anyhow!("{}: {error}", args.file.display())))?;
    let mut session = open(&args.base)?;
    let summary = session
        .apply_patch(&patch)
        .map_err(|error| invalid(anyhow!("{}: {error}", args.file.display())))?;
    for team in &summary.applied_teams {
        report.message(message(
            Code::PatchTeamApplied,
            vec![("team", team.to_string())],
        ));
    }
    for skip in summary.skipped {
        report.message(patch_skip(skip));
    }
    write_save(&session, &args.out)
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
