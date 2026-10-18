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

use crate::carried::{CarriedFiles, carried_files, under};
use crate::cli::{ABORTED, CLEAN, ERRORS, INVALID, UpgradeArgs};
use crate::face_xml::{face_xml_rewrites, kept_common_files};
use crate::kit_magic::{drop_collisions, respell_outputs};
use crate::kits::{KitFolders, kit_folders};
use crate::materials::{MaterialRewrites, material_references};
use crate::messages::{
    Code, INPUT, OUTPUT, extract_failed, issue_message, read_failed, team_name_unknown,
    tool_message,
};
use crate::model_names::rename_models;
use crate::naming::output_name;
use crate::note;
use crate::players::{PlayerFolders, player_folders};
use crate::roster::Roster;
use crate::settings::{SettingsFiles, settings_files};
use crate::source::{self, OldExport, SourceFailure};

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
    let resolved = if resolved.file_name().is_some() {
        resolved
    } else {
        fs::canonicalize(&resolved).map_err(|error| {
            invalid(anyhow!(
                "{}: cannot resolve the path: {error}",
                path.display()
            ))
        })?
    };
    // A filesystem root still has no file name: it is no export (and
    // `source_name`'s `expect` holds).
    if resolved.file_name().is_none() {
        return Err(invalid(anyhow!(
            "{}: a filesystem root is not an export",
            path.display()
        )));
    }
    Ok(resolved)
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
#[derive(Debug)]
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
/// each) or the shell, and the worst severity among them and whether any
/// drops something, which decide the exit code.
struct RunEvents {
    ctx: ToolContext,
    worst: Option<Severity>,
    dropped: bool,
}

impl RunEvents {
    /// Events sent through `ctx`, none yet.
    fn new(ctx: &ToolContext) -> RunEvents {
        // A clone shares the context's channels, so the events reach the same
        // receivers.
        RunEvents {
            ctx: ctx.clone(),
            worst: None,
            dropped: false,
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
        self.dropped |= message.disposition != Disposition::Keep;
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

    /// Whether a finding reported so far drops something (a file, a folder,
    /// the export): the verdict's "nothing dropped" half for the tool's own
    /// findings, beside the output validation's issues.
    fn dropped(&self) -> bool {
        self.dropped
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

/// A run aborted before anything was written, or while writing the output
/// (`cli.rs`'s `ABORTED`: the incomplete output is left).
fn aborted(error: anyhow::Error) -> CliError {
    CliError::new(ABORTED, error)
}

/// Writes `bytes` to `output`'s file at `relative`, parent folders created; a
/// failure is the aborted code naming the file and saying the output is
/// incomplete: it is left for the user to remove, since a second run refuses
/// an existing output folder.
fn write_output_file(output: &Path, relative: &str, bytes: &[u8]) -> Result<(), CliError> {
    let path = output.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            aborted(anyhow!(
                "{}: cannot create the directory: {error}; the output {} is incomplete",
                parent.display(),
                output.display()
            ))
        })?;
    }
    fs::write(&path, bytes).map_err(|error| {
        aborted(anyhow!(
            "{}: cannot write the file: {error}; the output {} is incomplete",
            path.display(),
            output.display()
        ))
    })
}

/// What `check_invocation` approved (the "CLI" paragraph's checks): the
/// input resolved and named, the flags checked, the save loaded, the teams
/// list parsed.
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

/// The "CLI" paragraph's invocation checks, before anything is written
/// (`run`'s order, `plans/export_upgrader.md` "CLI").
fn check_invocation(args: &UpgradeArgs, ctx: &ToolContext) -> Result<Invocation, CliError> {
    // "CLI": the export must exist and be a folder or a `.zip`/`.7z`.
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

    // "CLI": a referee export (the name's first word is `refs`) takes neither
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

    // "CLI": the savefile, when one is given, must read as a save.
    let save = match &args.savefile {
        Some(savefile) => Some(EditFile::load(savefile).map_err(|error| {
            invalid(anyhow!(
                "{}: cannot read the savefile: {error}",
                savefile.display()
            ))
        })?),
        None => None,
    };

    // "CLI": the teams list, the data directory's file or the embedded one.
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

    // Step 1: the export itself.
    events.started(INPUT, &invocation.name);
    let mut export = match OldExport::open(&invocation.path) {
        Ok(export) => export,
        Err(failure) => {
            events.message(extract_failed(INPUT, failure));
            return Ok(ERRORS);
        }
    };

    // Step 1: nothing an old export holds at the root is no old export.
    if !source::holds_old_export(&export.files, &export.folders) {
        events.message(tool_message(
            Code::NoOldExport,
            Scope::Export { export_id: INPUT },
            Disposition::DropExport,
            vec![],
        ));
        return Ok(ERRORS);
    }

    // Step 6: the identity — a referee export, else the Note's `Team:` line,
    // else the name's first word, each against the teams list. A team export's
    // Note is consumed here and below; a referee export's is unmigrated
    // content (step 9).
    let note_path = note::find(&export.files).map(str::to_owned);
    let note_text = match (&note_path, invocation.referees) {
        (Some(note_path), false) => match export.read(note_path) {
            Ok(bytes) => Some(note::text(&bytes)),
            Err(failure) => {
                // The Note decides the team: its failure stops the run with
                // nothing written (DECISIONS 2026-10-18 "the first slice's
                // review and the face-folder rules of 6.2a" (2)).
                events.message(read_failed(INPUT, failure, Disposition::DropExport));
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

    // "CLI": the savefile must hold the resolved team.
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

    // Step 2: the team's roster, when a save is given — its players by slot,
    // read by ID (loose mode and referee exports have none).
    let roster = match (&invocation.save, &identity) {
        (Some(save), Identity::Team { id, .. }) => Some(Roster::read(save, *id)),
        _ => None,
    };

    // "CLI": the output folder, refused when it exists or cannot be made.
    let output_name = output_name(&export.name, &identity);
    let base = args.output.clone().unwrap_or_else(|| {
        invocation
            .path
            .parent()
            .expect("the input is absolute and not a filesystem root (resolve_input)")
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

    // "CLI": the output folder itself, refused when it cannot be created
    // (exit 3).
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
        for entry in note::color_entries(note_text).team {
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
            write_output_file(&output, "colors.txt", colors.as_bytes())?;
        }
        if let Some(notes) = note::other_notes(note_text) {
            write_output_file(&output, "notes.txt", format!("{notes}\n").as_bytes())?;
        }
    }

    // The migration's target version decides the engine now (steps 3-4 merge
    // only on Fox); "Verification" validates with it too.
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

    // Steps 2-4's plan for the face and item folders, step 5's for the kits,
    // step 7's for the carried folders, the `.mtl` rewrites, the `face.xml`
    // respells and the kit magic, and step 8's `settings.toml` files:
    // `run`'s findings, then every file copied and written. A referee
    // export's folders stay unmigrated content (6.6's); it holds no
    // `Kit Configs`/`Kit Textures` either, which the same fallback reports.
    let files = export.files.clone();
    let (plan, kits, carried, materials, face_xml, settings) = match &identity {
        Identity::Referees => (
            PlayerFolders::default(),
            KitFolders::default(),
            CarriedFiles::default(),
            MaterialRewrites::default(),
            Vec::new(),
            SettingsFiles::default(),
        ),
        Identity::Team { id, .. } => {
            // The coverage tag decides step 5's `all/` kits and step 8's
            // worn-0 and missing-custom rules alike.
            let full = matches!(
                aesthetics_export::coverage(&output_name),
                Some(aesthetics_export::ExportCoverage::Full)
            );
            let mut plan =
                player_folders(&files, roster.as_ref(), version.engine(), |a, b| {
                    match (export.read(a), export.read(b)) {
                        (Ok(a), Ok(b)) => a == b,
                        _ => false,
                    }
                });
            // Step 8: a `settings.toml` per player folder, from the save's
            // records (loose mode has none).
            let settings = match (&invocation.save, &roster) {
                (Some(save), Some(roster)) => settings_files(save, roster, &plan.folders, full),
                _ => SettingsFiles::default(),
            };
            let mut carried = carried_files(&files, &plan.portraits, |path| export.read(path));
            // The kit magic respells the plans' output names, then step
            // 7's model renames apply (`kept` the `Common/` names a member
            // `face.xml` or an xml folder's link spells, computed from the
            // respelled plan copies; both plans renamed in one call, so a
            // link sees its `Common/` target's rename); a path another
            // already takes is the later input's, reported, its finding
            // following the plan's own.
            let plan_copies = respell_outputs(std::mem::take(&mut plan.copies), *id);
            let kept = kept_common_files(&plan_copies, *id, |path| export.read(path));
            let plan_len = plan_copies.len();
            let mut renamed = rename_models(
                plan_copies
                    .into_iter()
                    .chain(respell_outputs(std::mem::take(&mut carried.copies), *id))
                    .collect(),
                &kept,
            );
            let carried_copies = renamed.split_off(plan_len);
            let (copies, dropped) = drop_collisions(renamed);
            plan.copies = copies;
            plan.findings.extend(dropped);
            let (copies, dropped) = drop_collisions(carried_copies);
            carried.copies = copies;
            carried.findings.extend(dropped);
            // `Common/` for the materials: the carried files' respelled
            // output paths, which the links are spelled after.
            let common: Vec<String> = carried
                .copies
                .iter()
                .filter(|(_, output)| under(output, "common").is_some())
                .map(|(_, output)| output.clone())
                .collect();
            let all_copies: Vec<(String, String)> =
                plan.copies.iter().chain(&carried.copies).cloned().collect();
            let materials =
                material_references(&all_copies, &common, *id, |path| export.read(path));
            let face_xml = face_xml_rewrites(&plan.copies, *id, |path| export.read(path));
            (
                plan,
                kit_folders(
                    &files,
                    version,
                    note_path.as_deref().zip(note_text.as_deref()),
                    full,
                    |path| export.read(path),
                ),
                carried,
                materials,
                face_xml,
                settings,
            )
        }
    };
    for message in plan
        .findings
        .into_iter()
        .chain(kits.findings)
        .chain(carried.findings)
        .chain(materials.findings)
        .chain(settings.findings)
    {
        if message.code.code.as_ref() == Code::UnmigratedContent.as_str() {
            any_unmigrated = true;
        }
        events.message(message);
    }
    for (from, to) in plan
        .copies
        .iter()
        .chain(&kits.copies)
        .chain(&carried.copies)
    {
        if materials.written.iter().any(|(output, _)| output == to)
            || face_xml.iter().any(|(output, _)| output == to)
        {
            // A rewritten `.mtl` or `face.xml` replaces its input's copy.
            continue;
        }
        match export.read(from) {
            Ok(bytes) => write_output_file(&output, to, &bytes)?,
            Err(failure) => {
                // One file's failure does not cost the rest (DECISIONS
                // 2026-10-18 "the first slice's review and the face-folder
                // rules of 6.2a" (2)).
                events.message(read_failed(INPUT, failure, Disposition::DropFile));
            }
        }
    }
    for (path, bytes) in &kits.written {
        write_output_file(&output, path, bytes)?;
    }
    for (path, bytes) in &materials.written {
        write_output_file(&output, path, bytes)?;
    }
    for (path, bytes) in &face_xml {
        write_output_file(&output, path, bytes)?;
    }
    for link in &materials.links {
        write_output_file(&output, link, b"")?;
    }
    for marker in &plan.markers {
        write_output_file(&output, marker, b"")?;
    }
    for (path, bytes) in &settings.written {
        write_output_file(&output, path, bytes)?;
    }

    // Step 10: every input file no step consumed is unmigrated content.
    for file in &export.files {
        // A team export's Note is consumed (the root files above); a referee
        // export's is unmigrated content like any file (step 9).
        if !invocation.referees && note_path.as_deref() == Some(file.as_str()) {
            continue;
        }
        if plan.consumed.contains(file)
            || kits.consumed.contains(file)
            || carried.consumed.contains(file)
        {
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

    // "Verification": the shared validation on what was written. Every issue
    // goes through `events.message`, whose disposition sets `dropped`.
    validate_output(&output, &output_name, version, &mut events);

    // "CLI": the verdict line and the exit code: not clean when a finding
    // was an Error or dropped anything, the tool's own and the output
    // validation's alike.
    let (code, exit_code) = verdict(events.worst(), events.dropped());
    events.message(tool_message(code, Scope::Run, Disposition::Keep, vec![]));
    Ok(exit_code)
}

/// The export's identity (step 6): `Referees` for a referee export, else the
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
/// every issue on the output export; nothing when the listing cannot be read
/// or parsed (`export_extract_failed`, already reported).
fn validate_output(output: &Path, output_name: &str, version: PesVersion, events: &mut RunEvents) {
    let (files, folders) = match source::list_folder(output) {
        Ok(listed) => listed,
        Err(failure) => {
            events.message(extract_failed(OUTPUT, failure));
            return;
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
            return;
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
    fn a_warning_finding_that_drops_something_is_not_clean() {
        use std::sync::{Arc, Mutex};

        use studio_core::{AppPaths, Settings};

        // The verdict's "nothing dropped" half sees the tool's own findings
        // too: a finding below Error that drops a folder still refuses
        // `migration_clean` (DECISIONS 2026-10-18 "its own input read, and
        // the gaps the first slice meets" (7)).
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
        assert!(!events.dropped());
        events.message(tool_message(
            Code::RosterSlotEmpty,
            Scope::Folder {
                export_id: INPUT,
                path: ScopePath::new("Faces/XXX15 - A").unwrap(),
            },
            Disposition::DropFolder,
            vec![],
        ));
        assert_eq!(events.worst(), Some(Severity::Warning));
        assert!(events.dropped());
        assert_eq!(
            verdict(events.worst(), events.dropped()),
            (Code::MigrationNotClean, ERRORS)
        );
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
