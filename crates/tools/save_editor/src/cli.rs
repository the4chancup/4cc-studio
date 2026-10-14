//! The `save-editor` command line (`plans/save_editor.md` "CLI"): the batch surface for the
//! operations the cups script. Every command reads an explicit save; a refusal (a bad
//! argument, an unreadable file, a team the save lacks) comes before anything is written.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use aatf::{Rules, TeamView};
use anyhow::anyhow;
use clap::{ArgGroup, Args, Command, FromArgMatches, Subcommand};
use pes_savefile::interchange::aesthetics_patch::AestheticsPatch;
use pes_savefile::interchange::team_toml::TeamToml;
use pes_savefile::ops::compare::{CompareError, DiffScope};
use pes_savefile::ops::transplant::{TransplantError, parse_selection};
use pes_version::PesVersion;
use studio_core::{CliError, RunningPes, Severity, ToolContext, running_pes};

use crate::events::Report;
use crate::messages::{
    Code, TOOL_ID, aatf_check_failed, aatf_violation, import_note, message, patch_skip,
    stripped_mark,
};
use crate::session::Session;
use crate::{diff, settings, teams_list};

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
    /// Copy selected players' looks from a donor save of the same version onto a save.
    Transplant(TransplantArgs),
    /// List every field that differs between two saves of one version, player by player.
    Diff(DiffArgs),
    /// Check the save's cup teams against an AATF ruleset, one line per violation.
    CheckAatf(CheckAatfArgs),
    /// Merge the save's team names and ids into the data directory's teams_list.txt.
    ExportTeamsList(ExportTeamsListArgs),
}

/// `check-aatf <save> [--teams <list>] [--rules <file>]`.
#[derive(Debug, Args)]
struct CheckAatfArgs {
    /// The save to read (EDIT00000000).
    save: PathBuf,
    /// Only these teams, comma-separated ids and inclusive ranges (701,705-710); every team
    /// of the save in the cup range (701 to 920) otherwise.
    #[arg(long, value_name = "LIST")]
    teams: Option<String>,
    /// The AATF rules file to check against; the aatf_rules setting's file otherwise, or the
    /// official ruleset built in when that is empty.
    #[arg(long, value_name = "FILE")]
    rules: Option<PathBuf>,
}

/// `export-teams-list <save> [--yes]`.
#[derive(Debug, Args)]
struct ExportTeamsListArgs {
    /// The save whose teams are merged in (EDIT00000000).
    save: PathBuf,
    /// Write the merged list instead of only printing what would change.
    #[arg(long)]
    yes: bool,
}

/// `transplant <donor> --base <save> --out <save> --select <list>`.
#[derive(Debug, Args)]
struct TransplantArgs {
    /// The save the looks come from (EDIT00000000).
    donor: PathBuf,
    /// The save that takes them; never written unless it is also --out.
    #[arg(long, value_name = "SAVE")]
    base: PathBuf,
    /// Where the result goes; an existing file there is kept as <out>.bak.
    #[arg(long, value_name = "SAVE")]
    out: PathBuf,
    /// Who takes whose look, comma-separated: a player id (70105, his own look from the
    /// donor), target:donor (70107:70207), a team (703, its 23 slots) or a team range
    /// (704-705).
    #[arg(long, value_name = "LIST")]
    select: String,
}

/// `diff <first> <second> [--aesthetics | --gameplay] [--json]`.
#[derive(Debug, Args)]
#[command(group = ArgGroup::new("scope").args(["aesthetics", "gameplay"]))]
struct DiffArgs {
    /// The save whose values are the old ones.
    first: PathBuf,
    /// The save whose values are the new ones.
    second: PathBuf,
    /// Only the look: boots, gloves, base copy, physique, strip, face.
    #[arg(long)]
    aesthetics: bool,
    /// Only the rest: names, basics, abilities, positions, skills, motion.
    #[arg(long)]
    gameplay: bool,
    /// The report as one JSON document on stdout, for tooling.
    #[arg(long)]
    json: bool,
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
        SaveEditorCommand::Transplant(args) => transplant(&args, &mut report)?,
        SaveEditorCommand::Diff(args) => diff_saves(&args, &mut report)?,
        SaveEditorCommand::CheckAatf(args) => check_aatf(&args, ctx, &mut report)?,
        SaveEditorCommand::ExportTeamsList(args) => {
            export_teams_list(&args, ctx.paths().data_dir.as_deref(), &mut report)?;
        }
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

/// The refusal a writing command makes while a PES runs: the game writes its own save
/// back when it closes, over the one the command wrote.
fn refuse_while_running(running: &[RunningPes]) -> Result<(), CliError> {
    match running.first() {
        Some(game) => Err(invalid(anyhow!(
            "{} is running: it writes its own save back when it closes, over this one; \
             close it and run the command again",
            game.version
        ))),
        None => Ok(()),
    }
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
    refuse_while_running(&running_pes())?;
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
    refuse_while_running(&running_pes())?;
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

/// `transplant`: the selection is parsed, then the lib checks the version pair and every id
/// before the first copy, so a refusal leaves `--out` unwritten; a player at -1 on either
/// side of a pair is a warning line.
fn transplant(args: &TransplantArgs, report: &mut Report) -> Result<(), CliError> {
    refuse_while_running(&running_pes())?;
    let mut pairs = Vec::new();
    for item in args.select.split(',') {
        pairs.extend(parse_selection(item.trim()).map_err(|error| invalid(error.into()))?);
    }
    let donor = open(&args.donor)?;
    let mut session = open(&args.base)?;
    let marks = session
        .transplant(&donor, &pairs)
        .map_err(|error| match error {
            TransplantError::VersionMismatch { target, donor } => {
                version_mismatch("the base save", target, "the donor", donor)
            }
            TransplantError::TargetMissing(_) | TransplantError::DonorMissing(_) => {
                invalid(error.into())
            }
        })?;
    for mark in marks {
        report.message(stripped_mark(mark));
    }
    write_save(&session, &args.out)
}

/// The refusal of two saves of different versions, each named as a member reads it
/// (`PES 2021`): the lib's own text spells the versions as the enum does.
fn version_mismatch(
    first_role: &str,
    first: PesVersion,
    second_role: &str,
    second: PesVersion,
) -> CliError {
    invalid(anyhow!(
        "{first_role} is a {first} save and {second_role} a {second} one: both must be of one version"
    ))
}

/// `diff`: the comparator's report as findings; differences are its output, so the exit
/// code is 0, and two versions are refused.
fn diff_saves(args: &DiffArgs, report: &mut Report) -> Result<(), CliError> {
    let first = open(&args.first)?;
    let second = open(&args.second)?;
    let scope = match (args.aesthetics, args.gameplay) {
        (true, false) => Some(DiffScope::Aesthetics),
        (false, true) => Some(DiffScope::Gameplay),
        // clap makes the two flags exclusive.
        (false, false) | (true, true) => None,
    };
    let result = if args.json {
        diff::report_json(first.file(), second.file(), scope, report)
    } else {
        diff::report(first.file(), second.file(), scope, report)
    };
    result.map_err(|error| match error {
        CompareError::VersionMismatch { a, b } => {
            version_mismatch("the first save", a, "the second", b)
        }
        CompareError::Codec(_) => invalid(error.into()),
    })
}

/// `check-aatf`: the ruleset is `--rules`, else the `aatf_rules` setting's file, else the
/// embedded official ruleset; a file that does not read or fails load-time validation is
/// refused naming it, so the command doubles as a rules file's check. Each team is checked
/// in turn, a team's failed check is an error line and the next team is still checked; a
/// violation is one line at its severity, suggestions aside.
fn check_aatf(
    args: &CheckAatfArgs,
    ctx: &ToolContext,
    report: &mut Report,
) -> Result<(), CliError> {
    let rules = match rules_path(args, ctx)? {
        Some(path) => Rules::load(&read_text(&path)?)
            .map_err(|error| invalid(anyhow!("{}: {error}", path.display())))?,
        None => Rules::official(),
    };
    let session = open(&args.save)?;
    let ids = match &args.teams {
        Some(text) => parse_teams(text).map_err(|error| invalid(anyhow!("--teams: {error}")))?,
        None => {
            let mut ids: Vec<u32> = session
                .team_ids()
                .filter(|id| (701..=920).contains(id))
                .collect();
            ids.sort_unstable();
            ids
        }
    };
    for id in &ids {
        if session.file().team(*id).is_none() {
            return Err(invalid(anyhow!("the save has no team {id}")));
        }
    }
    for id in ids {
        let team = session.file().team(id).expect("checked above");
        let violations = match TeamView::new(session.file(), id)
            .map_err(|error| error.to_string())
            .and_then(|view| rules.check_team(&view).map_err(|error| error.to_string()))
        {
            Ok(violations) => violations,
            Err(error) => {
                report.message(aatf_check_failed(id, error));
                continue;
            }
        };
        for violation in violations {
            let severity = match violation.severity {
                aatf::Severity::Error => Severity::Error,
                aatf::Severity::Warning => Severity::Warning,
                // Suggestions are not printed: there is nothing to fix in an unused allowance.
                aatf::Severity::Suggestion => continue,
            };
            let player = violation
                .slot
                .and_then(|slot| team.roster.get(usize::from(slot)))
                .map(|slot| slot.player_id);
            report.message(aatf_violation(id, player, &violation, severity));
        }
    }
    Ok(())
}

/// The rules file `check-aatf` reads: `--rules`, else the `aatf_rules` setting when it names
/// one, else `None` (the embedded official ruleset). A relative path is used as given.
fn rules_path(args: &CheckAatfArgs, ctx: &ToolContext) -> Result<Option<PathBuf>, CliError> {
    if let Some(path) = &args.rules {
        return Ok(Some(path.clone()));
    }
    // A toml error's text ends with a newline, which would leave a blank line under `error: `.
    let settings = settings::from_table(&ctx.tool_settings(TOOL_ID)).map_err(|error| {
        invalid(anyhow!(
            "settings [{TOOL_ID}]: {}",
            error.to_string().trim_end()
        ))
    })?;
    Ok((!settings.aatf_rules.is_empty()).then(|| PathBuf::from(&settings.aatf_rules)))
}

/// `--teams`' list: comma-separated ids and inclusive ranges (`701,705-710`), sorted with
/// duplicates dropped; a malformed item or a range whose start is past its end is an `Err`
/// naming it.
fn parse_teams(text: &str) -> Result<Vec<u32>, String> {
    let mut ids = Vec::new();
    for item in text.split(',') {
        let item = item.trim();
        let (start, end) = match item.split_once('-') {
            Some((first, last)) => (
                first
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| format!("{item:?}: not a team id or a range"))?,
                last.trim()
                    .parse::<u32>()
                    .map_err(|_| format!("{item:?}: not a team id or a range"))?,
            ),
            None => {
                let id = item
                    .parse::<u32>()
                    .map_err(|_| format!("{item:?}: not a team id or a range"))?;
                (id, id)
            }
        };
        if start > end {
            return Err(format!("{item:?}: the range's start is past its end"));
        }
        ids.extend(start..=end);
    }
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

/// `export-teams-list`: the save's teams merged into the data directory's list, the summary
/// one line per change, the file written only with `--yes`. A data directory that cannot
/// take the file (none chosen yet, or unwritable, found by attempting the write) is the
/// `teams_list_read_only` error with nothing written. A dropped change is an error too; the
/// other changes are written.
fn export_teams_list(
    args: &ExportTeamsListArgs,
    data_dir: Option<&Path>,
    report: &mut Report,
) -> Result<(), CliError> {
    let session = open(&args.save)?;
    let (incoming, duplicates) = teams_list::from_save(session.file());
    for duplicate in &duplicates {
        report.message(message(
            Code::TeamsListNameDuplicate,
            vec![
                ("team", duplicate.team.to_string()),
                ("name", duplicate.name.as_str().to_owned()),
                ("kept", duplicate.kept.to_string()),
            ],
        ));
    }
    let path = data_dir.map(|dir| dir.join(teams_list::FILE_NAME));
    let working = teams_list::load_working(path.as_deref()).map_err(invalid)?;
    let (merged, summary) = teams_list::reconcile(&working, &incoming);
    for (id, name) in &summary.added {
        report.message(message(
            Code::TeamsListAdded,
            vec![
                ("team", id.get().to_string()),
                ("name", name.as_str().to_owned()),
            ],
        ));
    }
    for (name, old, new) in &summary.overridden {
        report.message(message(
            Code::TeamsListOverridden,
            vec![
                ("name", name.as_str().to_owned()),
                ("old", old.get().to_string()),
                ("new", new.get().to_string()),
            ],
        ));
    }
    for (id, name) in &summary.unresolved {
        report.message(message(
            Code::TeamsListUnresolved,
            vec![
                ("team", id.get().to_string()),
                ("name", name.as_str().to_owned()),
            ],
        ));
    }
    // `placeholders_added` stays empty: the save's list carries team rows alone.
    report.message(message(
        Code::TeamsListKept,
        vec![("count", summary.kept.to_string())],
    ));
    let shown = path.as_deref().map_or_else(
        || teams_list::FILE_NAME.to_owned(),
        |path| path.display().to_string(),
    );
    if !args.yes {
        report.message(message(Code::TeamsListNotWritten, vec![("path", shown)]));
        return Ok(());
    }
    let written = match &path {
        Some(path) => teams_list::write(path, &merged),
        None => Err(io::Error::other("no data directory is chosen yet")),
    };
    match written {
        Ok(()) => report.message(message(Code::TeamsListWritten, vec![("path", shown)])),
        Err(error) => report.message(message(
            Code::TeamsListReadOnly,
            vec![("path", shown), ("error", error.to_string())],
        )),
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
    fn refuse_while_running_accepts_no_game_and_refuses_a_running_one() {
        assert!(refuse_while_running(&[]).is_ok());
        let running = [RunningPes {
            version: PesVersion::Pes21,
            exe: None,
        }];
        let error = refuse_while_running(&running).unwrap_err();
        assert_eq!(error.exit_code, 2);
        assert!(
            error.error.to_string().contains("PES 2021"),
            "{}",
            error.error
        );
    }

    #[test]
    fn parse_teams_reads_ids_and_inclusive_ranges_sorted_without_duplicates() {
        assert_eq!(parse_teams("718").unwrap(), vec![718]);
        assert_eq!(
            parse_teams("705-707,701,706").unwrap(),
            vec![701, 705, 706, 707]
        );
        for text in ["710-705", "7x", ""] {
            assert!(parse_teams(text).is_err(), "{text}");
        }
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
