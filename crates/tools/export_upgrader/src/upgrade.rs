//! `upgrade`: one old export read, resolved, written under its derived name
//! and validated, in the order `plans/export_upgrader.md` "CLI" gives: the
//! invocation checks first (nothing written by a refusal), then the source,
//! the identity, the output folder, the migration report and the shared
//! validation's findings on what was written.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use aesthetics_export::{
    CanonicalListing, ListedEntry, ListedKind, SmallMetadata, SourceError, ValidationContext,
    is_small_metadata, parse_listing, read_colors_txt,
};
use anyhow::anyhow;
use pes_savefile::EditFile;
use pes_version::PesVersion;
use studio_core::{
    CliError, Disposition, ExportId, Message, PipelineEvent, PipelineEventEnvelope, RunId, Scope,
    Severity, ToolContext,
};
use teams_list::{TeamId, TeamName, TeamsList};
use vtree::ScopePath;

use crate::cli::{ABORTED, CLEAN, ERRORS, INVALID, UpgradeArgs};
use crate::messages::{Code, extract_failed, issue_message, team_name_unknown, tool_message};
use crate::naming::output_name;
use crate::note;
use crate::source::{self, OldExport, SourceFailure};

/// The input export's id in this run's events.
const INPUT: ExportId = ExportId(1);
/// The output export's.
const OUTPUT: ExportId = ExportId(2);
/// A command is one run per process, so every event carries the same run id.
const RUN_ID: RunId = RunId(1);

/// The input path made absolute, so its name and parent are the export's own:
/// `upgrade .` run inside an export resolves to the export folder itself, and
/// `upgrade ..` to its parent — `absolute` keeps a trailing `..` on POSIX, so
/// a path with no file name after it is canonicalized (the folder it points
/// at).
fn resolve_input(path: &Path) -> Result<PathBuf, CliError> {
    let resolved = std::path::absolute(path).map_err(|error| {
        invalid(anyhow!(
            "{}: cannot resolve the path: {error}",
            path.display()
        ))
    })?;
    if resolved.file_name().is_some() {
        return Ok(resolved);
    }
    fs::canonicalize(&resolved).map_err(|error| {
        invalid(anyhow!(
            "{}: cannot resolve the path: {error}",
            path.display()
        ))
    })
}

/// The run's verdict (the plan's "Message catalog"): `migration_clean` and
/// exit 0 when no finding is an Error or Fatal and no finding drops anything;
/// `migration_not_clean` and exit 1 otherwise, a Warning that drops a folder
/// included, since the compile would lose it.
fn verdict(worst: Option<Severity>, dropped: bool) -> (Code, u8) {
    if worst.is_none_or(|worst| worst < Severity::Error) && !dropped {
        (Code::MigrationClean, CLEAN)
    } else {
        (Code::MigrationNotClean, ERRORS)
    }
}

/// What the export resolved to: a listed team, or the referee package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Identity {
    /// A listed team; `id` is strictly 701-920.
    Team {
        /// The teams-list id.
        id: TeamId,
        /// The canonical `/xx/` name.
        name: TeamName,
    },
    /// A referee export; its name's first word is `refs`.
    Referees,
}

/// A run's event stream: the findings it reports to the console (one line
/// each) or the shell, and the worst severity among them, which decides the
/// exit code.
struct RunEvents {
    ctx: ToolContext,
    worst: Option<Severity>,
}

impl RunEvents {
    /// Events sent through `ctx`, none yet.
    fn new(ctx: &ToolContext) -> RunEvents {
        // A clone shares the context's channels, so the events reach the same
        // receivers.
        RunEvents {
            ctx: ctx.clone(),
            worst: None,
        }
    }

    /// An export entered the run: the console names its findings by this name.
    fn started(&self, export_id: ExportId, display_name: &str) {
        self.emit(
            Some(export_id),
            PipelineEvent::ExportStarted {
                export_id,
                display_name: display_name.to_owned(),
            },
        );
    }

    /// One finding, addressed to the export its scope names.
    fn message(&mut self, message: Message) {
        self.worst = self.worst.max(Some(message.severity));
        self.emit(message.scope.export_id(), PipelineEvent::Message(message));
    }

    /// The one event for unmigrated content existing at all.
    fn unmigrated(&self) {
        self.emit(None, PipelineEvent::UnmigratedContentFound);
    }

    /// The worst severity reported so far; `None` when no finding was.
    fn worst(&self) -> Option<Severity> {
        self.worst
    }

    fn emit(&self, export_id: Option<ExportId>, event: PipelineEvent) {
        self.ctx.emit(PipelineEventEnvelope {
            run_id: RUN_ID,
            export_id,
            export_revision: None,
            event,
        });
    }
}

/// A refusal before anything was written.
fn invalid(error: anyhow::Error) -> CliError {
    CliError::new(INVALID, error)
}

/// A run aborted before anything was written.
fn aborted(error: anyhow::Error) -> CliError {
    CliError::new(ABORTED, error)
}

/// Writes `contents` to `output`'s root file `name`; a failure is the
/// aborted code naming the file.
fn write_root_file(output: &Path, name: &str, contents: String) -> Result<(), CliError> {
    let path = output.join(name);
    fs::write(&path, contents).map_err(|error| {
        aborted(anyhow!(
            "{}: cannot write the file: {error}",
            path.display()
        ))
    })
}

/// What `check_invocation` approved (steps 1-4): the input resolved and
/// named, the flags checked, the save loaded, the teams list parsed.
struct Invocation {
    /// The input path, made absolute.
    path: PathBuf,
    /// The export's name: the folder's name, or the archive file's stem.
    name: String,
    /// Whether the name's first word is `refs`: a referee export.
    referees: bool,
    /// The save `--savefile` loaded, when given.
    save: Option<EditFile>,
    /// The working teams list.
    teams_list: TeamsList,
}

/// Steps 1-4: the invocation checks, before anything is written (`run`'s
/// order, `plans/export_upgrader.md` "CLI").
fn check_invocation(args: &UpgradeArgs, ctx: &ToolContext) -> Result<Invocation, CliError> {
    // Step 1: the export must exist and be a folder or a `.zip`/`.7z`.
    let path = &args.export;
    if !path.exists() {
        return Err(invalid(anyhow!(
            "{}: no such folder or file",
            path.display()
        )));
    }
    if !path.is_dir() && !source::is_archive(path) {
        return Err(invalid(anyhow!(
            "{}: not a folder, .zip or .7z",
            path.display()
        )));
    }
    let path = resolve_input(path)?;
    let name = source::source_name(&path);

    // Step 2: a referee export (the name's first word is `refs`) takes neither
    // flag; a team export takes exactly one.
    let referees = aesthetics_export::team_name(&name).is_some_and(|name| name.is_referees());
    if referees {
        let flag = if args.savefile.is_some() {
            Some("--savefile")
        } else {
            args.loose.then_some("--loose")
        };
        if let Some(flag) = flag {
            return Err(invalid(anyhow!(
                "{flag}: a referee export takes neither --savefile nor --loose"
            )));
        }
    } else if args.savefile.is_some() == args.loose {
        return Err(invalid(anyhow!(
            "a team export takes --savefile or --loose, exactly one"
        )));
    }

    // Step 3: the savefile, when one is given.
    let save = match &args.savefile {
        Some(savefile) => Some(EditFile::load(savefile).map_err(|error| {
            invalid(anyhow!(
                "{}: cannot read the savefile: {error}",
                savefile.display()
            ))
        })?),
        None => None,
    };

    // Step 4: the teams list, the data directory's file or the embedded one.
    let teams_list = load_teams_list(ctx.paths().data_dir.as_deref())?;
    Ok(Invocation {
        path,
        name,
        referees,
        save,
        teams_list,
    })
}

/// Runs `upgrade`: the invocation check, then the export opened and resolved,
/// written and reported, validated and judged.
pub(crate) fn run(args: &UpgradeArgs, ctx: &ToolContext) -> Result<u8, CliError> {
    let invocation = check_invocation(args, ctx)?;
    let mut events = RunEvents::new(ctx);

    // Step 5: the export itself.
    events.started(INPUT, &invocation.name);
    let mut export = match OldExport::open(&invocation.path) {
        Ok(export) => export,
        Err(failure) => {
            events.message(extract_failed(INPUT, failure));
            return Ok(ERRORS);
        }
    };

    // Step 6: nothing an old export holds at the root is no old export.
    if !source::holds_old_export(&export.files, &export.folders) {
        events.message(tool_message(
            Code::NoOldExport,
            Scope::Export { export_id: INPUT },
            Disposition::DropExport,
            vec![],
        ));
        return Ok(ERRORS);
    }

    // Step 7: the identity — a referee export, else the Note's `Team:` line,
    // else the name's first word, each against the teams list. The Note is
    // found for every export (it is consumed, never unmigrated content); its
    // `Team:` line is read only for a team export.
    let note_path = note::find(&export.files).map(str::to_owned);
    let note_text = match (&note_path, invocation.referees) {
        (Some(note_path), false) => match export.read(note_path) {
            Ok(bytes) => Some(note::text(&bytes)),
            Err(failure) => {
                events.message(extract_failed(INPUT, failure));
                return Ok(ERRORS);
            }
        },
        _ => None,
    };
    let team_value = note_text.as_deref().and_then(note::team_line);
    let identity = match resolve(
        &export.name,
        invocation.referees,
        team_value,
        note_path.as_deref(),
        &invocation.teams_list,
    ) {
        Ok(identity) => identity,
        Err((scope, name)) => {
            events.message(team_name_unknown(scope, name));
            return Ok(ERRORS);
        }
    };

    // Step 8: the savefile must hold the resolved team.
    if let (Some(save), Identity::Team { id, name }) = (&invocation.save, &identity)
        && save.team(u32::from(id.get())).is_none()
    {
        return Err(invalid(anyhow!(
            "--savefile {}: it does not hold the resolved team {name} ({id})",
            args.savefile
                .as_ref()
                .expect("the save came from --savefile")
                .display()
        )));
    }

    // Step 9: the output folder, refused when it exists or cannot be made.
    let output_name = output_name(&export.name, &identity);
    let base = args.output.clone().unwrap_or_else(|| {
        invocation
            .path
            .parent()
            .unwrap_or(Path::new(""))
            .to_path_buf()
    });
    let output = base.join(&output_name);
    if output.exists() {
        return Err(aborted(anyhow!(
            "{}: the output folder already exists",
            output.display()
        )));
    }
    // `-o`'s directory is created when missing; the input's parent exists by
    // definition (it holds the input).
    if let Some(dir) = &args.output {
        fs::create_dir_all(dir).map_err(|error| {
            aborted(anyhow!(
                "{}: cannot create the directory: {error}",
                dir.display()
            ))
        })?;
    }

    // Step 10: the output (this slice writes the folder alone); every input
    // file no step consumed — every file but the Note — is unmigrated content.
    fs::create_dir(&output).map_err(|error| {
        aborted(anyhow!(
            "{}: cannot create the output folder: {error}",
            output.display()
        ))
    })?;
    events.started(OUTPUT, &output_name);

    // The Note's team colors become the root `colors.txt`, its "Other Notes"
    // section the root `notes.txt` (step 6; a referee export's Note gives
    // neither). An entry whose colors do not read is unmigrated content
    // naming the Note with its line; the kept entries are written as the Note
    // spells them, one per line. The Note itself is never written.
    let mut any_unmigrated = false;
    if let Some(note_text) = &note_text {
        let mut colors = String::new();
        for entry in note::team_entries(note_text) {
            let read = read_colors_txt(entry.colors.as_bytes(), 1);
            if read.colors.len() == 1 && read.refused.is_empty() {
                colors.push_str(&entry.colors);
                colors.push('\n');
            } else {
                any_unmigrated = true;
                events.message(tool_message(
                    Code::UnmigratedContent,
                    Scope::File {
                        export_id: INPUT,
                        path: ScopePath::new(
                            note_path
                                .as_deref()
                                .expect("a Note text exists only when a Note was found"),
                        )
                        .expect("every listed path was checked when the source opened"),
                    },
                    Disposition::DropFile,
                    vec![("line", entry.line)],
                ));
            }
        }
        if !colors.is_empty() {
            write_root_file(&output, "colors.txt", colors)?;
        }
        if let Some(notes) = note::other_notes(note_text) {
            write_root_file(&output, "notes.txt", format!("{notes}\n"))?;
        }
    }

    for file in &export.files {
        if note_path.as_deref() == Some(file.as_str()) {
            continue;
        }
        any_unmigrated = true;
        events.message(tool_message(
            Code::UnmigratedContent,
            Scope::File {
                export_id: INPUT,
                path: ScopePath::new(file)
                    .expect("every listed path was checked when the source opened"),
            },
            Disposition::DropFile,
            vec![],
        ));
    }
    if any_unmigrated {
        events.unmigrated();
    }

    // Step 11: the shared validation on what was written ("Verification").
    let version = match &invocation.save {
        Some(save) => save.version(),
        None => {
            if export
                .files
                .iter()
                .any(|file| file.to_ascii_lowercase().ends_with(".model"))
            {
                PesVersion::Pes16
            } else {
                PesVersion::Pes21
            }
        }
    };
    let report = validate_output(&output, &output_name, version, &mut events);

    // Step 12: the verdict line and the exit code.
    let dropped = report.is_some_and(|report| drops_anything(&report.issues));
    let (code, exit_code) = verdict(events.worst(), dropped);
    events.message(tool_message(code, Scope::Run, Disposition::Keep, vec![]));
    Ok(exit_code)
}

/// Whether any issue drops something (a file, a slot, a folder, the export):
/// the verdict's "no finding drops anything" half.
fn drops_anything(issues: &[aesthetics_export::ValidationIssue]) -> bool {
    issues
        .iter()
        .any(|issue| issue.disposition != aesthetics_export::Disposition::Keep)
}

/// The export's identity (step 7): `Referees` for a referee export, else the
/// Note's `Team:` value folded and looked up in the teams list, else the
/// name's first word the same way. `Err((scope, name))` is `team_name_unknown`
/// reporting the `Team:` line's file or the export, with the value as written.
fn resolve(
    export_name: &str,
    referees: bool,
    team_value: Option<String>,
    note_path: Option<&str>,
    teams_list: &TeamsList,
) -> Result<Identity, (Scope, String)> {
    if referees {
        return Ok(Identity::Referees);
    }
    if let Some(value) = team_value {
        let scope = Scope::File {
            export_id: INPUT,
            path: ScopePath::new(note_path.expect("a `Team:` value came from a Note"))
                .expect("every listed path was checked when the source opened"),
        };
        return match TeamName::new(&value)
            .ok()
            .and_then(|name| lookup(name, teams_list))
        {
            Some(identity) => Ok(identity),
            None => Err((scope, value)),
        };
    }
    match aesthetics_export::team_name(export_name).and_then(|name| lookup(name, teams_list)) {
        Some(identity) => Ok(identity),
        None => Err((Scope::Export { export_id: INPUT }, export_name.to_owned())),
    }
}

/// The team `name`'s row in the teams list, as an identity.
fn lookup(name: TeamName, teams_list: &TeamsList) -> Option<Identity> {
    teams_list
        .id_of(&name)
        .map(|id| Identity::Team { id, name })
}

/// The teams list: `<data_dir>/teams_list.txt` when present, else the embedded
/// list (the upgrader has no setting of its own for it, step 6); unreadable or
/// unparsable aborts, as the compiler's loader's.
fn load_teams_list(data_dir: Option<&Path>) -> Result<TeamsList, CliError> {
    let path = match data_dir {
        Some(dir) => dir.join("teams_list.txt"),
        None => return Ok(embedded_teams_list()),
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(embedded_teams_list());
        }
        Err(error) => {
            return Err(aborted(anyhow!(
                "{}: cannot read the teams list: {error}",
                path.display()
            )));
        }
    };
    TeamsList::parse(&text).map_err(|error| {
        aborted(anyhow!(
            "{}: not a valid teams list: {error}",
            path.display()
        ))
    })
}

fn embedded_teams_list() -> TeamsList {
    TeamsList::parse(TeamsList::UPSTREAM)
        .expect("the embedded teams list parses; the teams_list crate's tests check it")
}

/// Lists `output`, parses it back and validates it ("Verification"), reporting
/// every issue on the output export; the report on success, `None` when the
/// listing cannot be read or parsed (`export_extract_failed`, already
/// reported).
fn validate_output(
    output: &Path,
    output_name: &str,
    version: PesVersion,
    events: &mut RunEvents,
) -> Option<aesthetics_export::ValidationReport> {
    let (files, folders) = match source::list_folder(output) {
        Ok(listed) => listed,
        Err(failure) => {
            events.message(extract_failed(OUTPUT, failure));
            return None;
        }
    };
    let mut metadata = BTreeSet::new();
    let mut entries = Vec::new();
    for (path, size) in &files {
        if is_small_metadata(path) {
            metadata.insert(path.clone());
        }
        entries.push(ListedEntry {
            path: path.clone(),
            kind: ListedKind::File { size: *size },
        });
    }
    for path in &folders {
        entries.push(ListedEntry {
            path: path.clone(),
            kind: ListedKind::Folder,
        });
    }
    let metadata = SmallMetadata {
        files: metadata
            .into_iter()
            .map(|path| {
                let bytes = fs::read(output.join(&path)).map_err(|error| error.to_string());
                (path, bytes)
            })
            .collect(),
    };
    let listing = CanonicalListing {
        display_name: output_name.to_owned(),
        entries,
    };
    let parsed = match parse_listing(listing, metadata) {
        Ok(parsed) => parsed,
        Err(error) => {
            let (path, error) = match error {
                SourceError::Path { path, error } => (path, error.to_string()),
                SourceError::Collision { path, error } => (path, error.to_string()),
            };
            events.message(extract_failed(OUTPUT, SourceFailure { path, error }));
            return None;
        }
    };
    let context = ValidationContext {
        version,
        strict_file_type_check: false,
        pass_through: false,
        installed_common_textures: BTreeSet::new(),
    };
    let report = parsed.validate(&context);
    for issue in &report.issues {
        events.message(issue_message(issue, OUTPUT));
    }
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch folder under the temp dir (`CARGO_TARGET_TMPDIR` is set for
    /// integration tests only).
    fn scratch(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("export_upgrader_{name}_{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn an_input_ending_in_a_dot_or_dot_dot_resolves_to_its_folder() {
        let root = scratch("resolve_input");
        fs::create_dir_all(root.join("co Spring/Other")).unwrap();
        for input in [
            root.join("co Spring").join("."),
            root.join("co Spring").join("Other").join(".."),
        ] {
            let resolved = resolve_input(&input).unwrap();
            assert_eq!(source::source_name(&resolved), "co Spring", "{input:?}");
            assert_eq!(resolved.parent(), Some(root.as_path()), "{input:?}");
        }
    }

    #[test]
    fn load_teams_list_falls_back_to_the_embedded_list() {
        let root = scratch("teams_list_missing");
        let embedded = TeamsList::parse(TeamsList::UPSTREAM).unwrap();
        // No `teams_list.txt` in the data directory, and no data directory at
        // all: the embedded list.
        assert_eq!(load_teams_list(Some(&root)).unwrap(), embedded);
        assert_eq!(load_teams_list(None).unwrap(), embedded);
    }

    #[test]
    fn load_teams_list_aborts_on_an_unreadable_or_unparsable_file() {
        let root = scratch("teams_list_bad");
        // A `teams_list.txt` that is a folder cannot be read.
        fs::create_dir(root.join("teams_list.txt")).unwrap();
        let error = load_teams_list(Some(&root)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED, "{error}");
        // One that does not parse is aborted too.
        fs::remove_dir_all(&root).unwrap();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("teams_list.txt"), "not a list").unwrap();
        let error = load_teams_list(Some(&root)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED, "{error}");
    }

    #[test]
    fn worst_tracks_the_worst_severity_reported() {
        use std::sync::{Arc, Mutex};

        use studio_core::{AppPaths, Settings};

        let (events_tx, _events_rx) = crossbeam_channel::unbounded();
        let (requests_tx, _requests_rx) = crossbeam_channel::unbounded();
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(Settings::default())),
            AppPaths {
                exe_dir: PathBuf::new(),
                data_dir: None,
            },
            events_tx,
            requests_tx,
        );
        let mut events = RunEvents::new(&ctx);
        assert_eq!(events.worst(), None);
        events.message(tool_message(
            Code::MigrationClean,
            Scope::Run,
            Disposition::Keep,
            vec![],
        ));
        assert_eq!(events.worst(), Some(Severity::Info));
        events.message(tool_message(
            Code::UnmigratedContent,
            Scope::Run,
            Disposition::DropFile,
            vec![],
        ));
        assert_eq!(events.worst(), Some(Severity::Error));
        events.message(tool_message(
            Code::MigrationNotClean,
            Scope::Run,
            Disposition::Keep,
            vec![],
        ));
        assert_eq!(events.worst(), Some(Severity::Error));
    }

    #[test]
    fn drops_anything_is_true_exactly_when_an_issue_drops() {
        use aesthetics_export::{Disposition, IssueScope, ValidationIssue};

        let issue = |disposition| ValidationIssue {
            code: "export_empty",
            scope: IssueScope::Export,
            context: Vec::new(),
            disposition,
            passed_through: false,
        };
        assert!(!drops_anything(&[]));
        assert!(!drops_anything(&[issue(Disposition::Keep)]));
        for disposition in [
            Disposition::DropFile,
            Disposition::DropSlot,
            Disposition::DropFolder,
            Disposition::DropExport,
        ] {
            assert!(drops_anything(&[issue(disposition)]), "{disposition:?}");
        }
    }

    #[test]
    fn the_verdict_is_clean_only_with_no_error_and_no_drop() {
        assert_eq!(verdict(None, false), (Code::MigrationClean, CLEAN));
        assert_eq!(
            verdict(Some(Severity::Warning), false),
            (Code::MigrationClean, CLEAN)
        );
        assert_eq!(
            verdict(Some(Severity::Warning), true),
            (Code::MigrationNotClean, ERRORS)
        );
        assert_eq!(
            verdict(Some(Severity::Error), false),
            (Code::MigrationNotClean, ERRORS)
        );
    }
}
