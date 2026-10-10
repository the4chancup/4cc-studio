//! Validation of every export source of a run, which `check` reports and `compile` starts
//! from: each source's route, the structure pass's issues, the deep pass's content findings
//! over the export it leaves (`team_compiler/pipeline.md` "2. Per-export serial steps"), the
//! identity, and the refusal of two exports of one team. The lib validates and `deep` reads
//! the contents; this schedules them on the run's worker pool, each export's outcome kept in
//! discovery order.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use aesthetics_export::{
    ExportIdentity, FileDescriptor, FileKind, ModelFormat, ModelSuffix, ParsedAestheticsExport,
    ResolvedAestheticsExport, SharedKind, SourceError, ValidatedAestheticsExport,
    ValidationContext, common_link_name, model_suffix, parse_listing, read_colors_txt,
};
use anyhow::Context;
use pes_version::{Engine, PesVersion};
use pipeline::{MIN_FREE_BUDGET, MemoryBudget};
use rayon::prelude::*;
use studio_core::{Disposition, ExportId, Message, Scope};
use teams_list::TeamId;
use vtree::ScopePath;

use crate::bins::installed::InstalledPaths;
use crate::bins::{Rgb, TEAM_COLORS};
use crate::cli::RunInputs;
use crate::deep;
use crate::messages::{Code, issue_message, tool_message};
use crate::plan::ids::{SHARED_COUNT, shared_folders_taking_ids, shared_folders_with_no_model};
use crate::plan::mapped_players;
use crate::plan::roles::{
    FolderModels, ModelPackage, PlayerFile, common_skeleton, file_stem, is_read_common_file,
    is_user_face_xml, plans_a_package, player_file, role_position, shared_folders,
};
use crate::reader::{self, ContentSource, ExportSource, Route, SourceKind, SourceRevision};

/// Validation's outcome for the whole run.
pub(crate) struct ValidationPass {
    /// Findings about the run rather than one export (the duplicate-refs summary), reported
    /// before any export's.
    pub(crate) run_messages: Vec<Message>,
    /// Every source, in discovery order.
    pub(crate) sources: Vec<CheckedSource>,
}

/// One source after validation.
pub(crate) struct CheckedSource {
    /// The source.
    pub(crate) source: ExportSource,
    /// Its findings: its route's when it was set aside, else its issues and its identity, then
    /// `duplicate_aesthetics_export` when another export resolved to its team.
    pub(crate) messages: Vec<Message>,
    /// The export with its identity resolved; `None` when a finding dropped it.
    pub(crate) resolved: Option<ResolvedAestheticsExport>,
    /// The valid colors of the export's root `colors.txt`, at most four (empty when no line
    /// gives one); `None` when the export has no such file, is a referee export, or a
    /// finding dropped it before its identity was resolved. Read only beside `resolved`.
    pub(crate) team_colors: Option<Vec<Rgb>>,
    /// The text of the root `notes.txt` validation kept, its BOM removed; `None` when the
    /// export has no such note or a finding dropped it before its identity was resolved.
    pub(crate) notes: Option<String>,
    /// The export paths of the models the deep pass found carrying hand weights
    /// (`deep::ContentPass::hand_weighted`), which planning reads for the hand auto-split;
    /// empty when the deep pass did not run. Read only beside `resolved`.
    pub(crate) hand_weighted: BTreeSet<ScopePath>,
    /// The export paths of the FMDLs the deep pass found holding a metal material
    /// (`deep::ContentPass::metal_models`), which planning reads for the template environment
    /// map on PES 15-17; empty when the deep pass did not run. Read only beside `resolved`.
    pub(crate) metal_models: BTreeSet<ScopePath>,
    /// What the source looked like when it was listed, which `compile` checks its tasks'
    /// reads against; `None` when routing or the listing's parse set the source aside.
    /// `check` carries it and does nothing with it.
    pub(crate) revision: Option<SourceRevision>,
    /// The source kept open from its check for its tasks: a `.7z` `keeps_archive` admits,
    /// decompressed and holding its permit, so the coordinator reads the tasks' files out of
    /// the same buffer; `None` for a folder or a `.zip` (opened cheaply by the coordinator),
    /// for a `.7z` not kept, and under `check`, which has no tasks.
    pub(crate) content: Option<ContentSource>,
}

/// The run's memory budget, at the share of the available memory the settings give.
pub(crate) fn run_budget(inputs: &RunInputs) -> Arc<MemoryBudget> {
    MemoryBudget::new(pipeline::memory_cap(f64::from(
        inputs.common.memory_cap_percent,
    )))
}

/// The run's worker pool, of the `thread_count` the settings give (every logical core but one
/// when it is 0): validation's parallel work runs on it, then `compile`'s tasks.
pub(crate) fn run_pool(inputs: &RunInputs) -> anyhow::Result<rayon::ThreadPool> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(pipeline::thread_count_detect(inputs.common.thread_count))
        .build()
        .context("cannot start the worker threads")
}

/// Routes the run's `sources` (`reader::discover`'s) and runs the structure pass, the deep
/// pass and identity on each one headed for validation, on `pool`; a `.7z` export is read once
/// for both passes, charged to `budget`, and with `keep_archives` (`compile`'s, not `check`'s)
/// kept decompressed for its tasks when `keeps_archive` admits it. A texture `.common` link
/// may name a texture of the team's Common output that one of the `installed` CPKs holds.
/// Then the exports of a team several resolve to are refused (`refuse_duplicate_teams`). No
/// source at all is `no_exports_found`, on the run.
pub(crate) fn validation_pass(
    inputs: &RunInputs,
    sources: Vec<ExportSource>,
    installed: &InstalledPaths,
    budget: &Arc<MemoryBudget>,
    pool: &rayon::ThreadPool,
    keep_archives: bool,
) -> anyhow::Result<ValidationPass> {
    let routes = pool.install(|| reader::route(&sources));

    let mut run_messages = Vec::new();
    // Every `--export` path yields a source, so no source at all means the root's scan found
    // none.
    if sources.is_empty() {
        run_messages.push(tool_message(
            Code::NoExportsFound,
            Scope::Run,
            Disposition::Keep,
            vec![("folder", inputs.exports_root.display().to_string())],
        ));
    }
    let conflicting: Vec<&str> = sources
        .iter()
        .zip(&routes)
        .filter(|(_, route)| matches!(route, Route::ConflictingRefs))
        .map(|(source, _)| source.file_name.as_str())
        .collect();
    if !conflicting.is_empty() {
        run_messages.push(tool_message(
            Code::MultipleRefExports,
            Scope::Run,
            Disposition::DropExport,
            vec![("exports", conflicting.join(", "))],
        ));
    }

    let mut sources =
        pool.install(|| check_sources(inputs, installed, sources, routes, budget, keep_archives));
    refuse_duplicate_teams(&mut sources);
    Ok(ValidationPass {
        run_messages,
        sources,
    })
}

/// `duplicate_aesthetics_export` on each export of a team that two or more of `sources`
/// resolve to, naming the team's ID and every one of those exports in export order, and none
/// of them compiled (`pipeline.md` "3. Per-model-folder parallel steps"). Only an export whose
/// identity resolved to a team counts: one a finding dropped before has no team to conflict
/// over, and a referee export has `multiple_ref_exports`.
fn refuse_duplicate_teams(sources: &mut [CheckedSource]) {
    let mut by_team: BTreeMap<TeamId, Vec<usize>> = BTreeMap::new();
    for (index, checked) in sources.iter().enumerate() {
        if let Some(ResolvedAestheticsExport {
            identity: ExportIdentity::Team { id, .. },
            ..
        }) = &checked.resolved
        {
            by_team.entry(*id).or_default().push(index);
        }
    }
    for (id, indices) in by_team {
        if indices.len() < 2 {
            continue;
        }
        let exports: Vec<&str> = indices
            .iter()
            .map(|&index| sources[index].source.file_name.as_str())
            .collect();
        let exports = exports.join(", ");
        for index in indices {
            let checked = &mut sources[index];
            checked.messages.push(tool_message(
                Code::DuplicateAestheticsExport,
                Scope::Export {
                    export_id: checked.source.export_id,
                },
                Disposition::DropExport,
                vec![("id", id.to_string()), ("exports", exports.clone())],
            ));
            checked.resolved = None;
            // A refused export has no task, so a `.7z` kept for its tasks goes now.
            checked.content = None;
        }
    }
}

/// Each source through `check_source` with its route, returned in discovery order: the
/// folder and `.zip` sources in parallel, then the `.7z` sources one after another, each
/// kept decompressed for its tasks when `keep_archives` and `keeps_archive` admit it.
fn check_sources(
    inputs: &RunInputs,
    installed: &InstalledPaths,
    sources: Vec<ExportSource>,
    routes: Vec<Route>,
    budget: &Arc<MemoryBudget>,
    keep_archives: bool,
) -> Vec<CheckedSource> {
    // Not one parallel iterator over every source: a `.7z`'s check waits for its whole-archive
    // permit, and a worker waiting on its own export's parallel checks takes other work. A
    // `.7z` check started that way can wait for the permit of a `.7z` export suspended below
    // it on the same thread, which never resumes. A folder's or a zip's check never waits for
    // memory, so those run in parallel; the `.7z` ones run in turn, once the others are done.
    let (in_turn, in_parallel): (Vec<_>, Vec<_>) = sources
        .into_iter()
        .zip(routes)
        .enumerate()
        .partition(|(_, (source, _))| match source.kind {
            SourceKind::SevenZ => true,
            SourceKind::Folder | SourceKind::Zip => false,
        });
    let mut checked: Vec<(usize, CheckedSource)> = in_parallel
        .into_par_iter()
        .map(|(index, (source, route))| {
            (
                index,
                check_source(inputs, installed, source, route, budget),
            )
        })
        .collect();
    // Every `.7z`'s size, known from its listing before any is checked: the keep rule leaves
    // room for the largest of the others.
    let sizes: Vec<usize> = in_turn
        .iter()
        .map(|(_, (_, route))| match route {
            Route::Validate { decompressed, .. } => *decompressed,
            Route::Unreadable(_) | Route::Disabled | Route::Balls | Route::ConflictingRefs => 0,
        })
        .collect();
    // Decided here, in the order the `.7z` exports are checked, and before the next one's
    // check: an archive not kept is let go at once, so the next check's whole-archive acquire
    // never waits on a buffer no task will read.
    let mut kept = 0;
    for (position, (index, (source, route))) in in_turn.into_iter().enumerate() {
        let mut outcome = check_source(inputs, installed, source, route, budget);
        let held = outcome.content.as_ref().map_or(0, ContentSource::held);
        let others = largest_other(&sizes, position);
        // An export a finding dropped has no task to read the archive for.
        if keep_archives
            && outcome.resolved.is_some()
            && keeps_archive(held, kept, budget.cap(), others)
        {
            kept += held;
        } else {
            outcome.content = None;
        }
        checked.push((index, outcome));
    }
    checked.sort_by_key(|(index, _)| *index);
    checked.into_iter().map(|(_, checked)| checked).collect()
}

/// Whether `compile` keeps a solid `.7z` export of `size` decompressed bytes from its check to
/// its tasks, with `kept` bytes of archives already kept, on a budget of `cap`, when the
/// largest of the run's other `.7z` exports decompresses to `largest_other` bytes
/// (`libs/pipeline.md` "What a solid `.7z` is charged"): when it is at most an eighth of the
/// cap and, with it kept, the cap leaves free of kept archives at least `MIN_FREE_BUDGET` and
/// at least `largest_other`.
///
/// The eighth keeps the small exports a midcup compiles from archives and leaves a large one
/// to a second decompression. The floor leaves the tasks room to run: a task's charges do not
/// wait for room, and kept archives filling the budget would stall the tasks that release
/// them. The `largest_other` clause exists because `acquire` waits where `charge` does not,
/// and a kept permit is released only when the coordinator reaches its source: a later
/// check's whole-archive acquire, or an earlier source's in the coordinator, that needs more
/// than the kept archives leave would wait forever. An archive over the cap acquires only on
/// an empty budget, so with one in the run nothing is kept.
pub(crate) fn keeps_archive(size: usize, kept: usize, cap: usize, largest_other: usize) -> bool {
    // A sum past `usize` (a 32-bit host only) is over the cap, never a wrap into a keep.
    size <= cap / 8
        && kept
            .checked_add(size)
            .is_some_and(|total| total <= cap.saturating_sub(MIN_FREE_BUDGET.max(largest_other)))
}

/// The largest of `sizes` but the one at `index`; 0 when there is no other. Counting the
/// archives already kept too is conservative (each holds its bytes already) and needs no
/// record of which ones are.
fn largest_other(sizes: &[usize], index: usize) -> usize {
    sizes
        .iter()
        .enumerate()
        .filter(|(other, _)| *other != index)
        .map(|(_, size)| *size)
        .max()
        .unwrap_or(0)
}

/// One source through its route, the structure pass, the deep pass and identity. The small
/// metadata and the deep pass's files are read through one `ContentSource`, so a `.7z` is
/// decompressed once for both passes, under one permit; a `.7z`'s source comes back in
/// `content` for the caller to keep for its tasks or let go.
fn check_source(
    inputs: &RunInputs,
    installed: &InstalledPaths,
    source: ExportSource,
    route: Route,
    budget: &Arc<MemoryBudget>,
) -> CheckedSource {
    let export = Scope::Export {
        export_id: source.export_id,
    };
    // Each route's catalog consequence is "export skipped".
    let skipped = |source, code, context| CheckedSource {
        source,
        messages: vec![tool_message(
            code,
            export.clone(),
            Disposition::DropExport,
            context,
        )],
        resolved: None,
        team_colors: None,
        notes: None,
        hand_weighted: BTreeSet::new(),
        metal_models: BTreeSet::new(),
        revision: None,
        content: None,
    };
    let (listing, revision) = match route {
        Route::Unreadable(failure) => {
            return skipped(
                source,
                Code::ExportExtractFailed,
                vec![("path", failure.path), ("error", failure.error)],
            );
        }
        Route::Disabled => return skipped(source, Code::ExportDisabled, vec![]),
        Route::Balls => return skipped(source, Code::ExportBallsSkipped, vec![]),
        Route::ConflictingRefs => return skipped(source, Code::MultipleRefExports, vec![]),
        Route::Validate {
            listing, revision, ..
        } => (listing, revision),
    };

    let content = ContentSource::new(&source, budget);
    let metadata = content.read_metadata(&listing);
    let parsed = match parse_listing(listing, metadata) {
        Ok(parsed) => parsed,
        Err(error) => {
            let (path, error) = match error {
                SourceError::Path { path, error } => (path, error.to_string()),
                SourceError::Collision { path, error } => (path, error.to_string()),
            };
            return skipped(
                source,
                Code::ExportExtractFailed,
                vec![("path", path), ("error", error)],
            );
        }
    };
    let strict = inputs.settings.strict_file_type_check;
    let team_id = team_id(inputs, &parsed);
    let context = ValidationContext {
        version: inputs.common.pes_version,
        strict_file_type_check: strict,
        pass_through: inputs.settings.pass_through,
        installed_common_textures: installed_common_textures(inputs, installed, team_id),
    };
    let mut report = parsed.validate(&context);
    // The deep pass reads only what the structure pass kept; its findings derive the report
    // again, so they drop, cascade and pass through as the structure pass's own do.
    let mut hand_weighted = BTreeSet::new();
    let mut metal_models = BTreeSet::new();
    if let Some(validated) = &report.validated {
        let pass = deep::content_findings(
            validated,
            &content,
            inputs.common.pes_version,
            installed_common_stems(inputs, installed, team_id),
            context.pass_through,
        );
        hand_weighted = pass.hand_weighted;
        metal_models = pass.metal_models;
        if !pass.findings.is_empty() {
            report = report.with_content_findings(pass.findings, &context);
        }
    }
    let team_colors = report
        .validated
        .as_ref()
        .and_then(|validated| team_colors(validated, &content));
    let notes = report
        .validated
        .as_ref()
        .and_then(|validated| notes(validated, &content));
    // Nothing past the deep pass reads the source. A folder or a zip is let go now: its tasks
    // open it again cheaply. A `.7z` is returned for the caller to keep for its tasks or let
    // go, which holds its buffer and permit through identity too, a few microseconds.
    let content = match source.kind {
        SourceKind::SevenZ => Some(content),
        SourceKind::Folder | SourceKind::Zip => None,
    };
    let mut messages: Vec<Message> = report
        .issues
        .iter()
        .map(|issue| issue_message(issue, source.export_id, strict))
        .collect();
    // `None` means an issue dropped the export, and that issue is already among the messages.
    let resolved = report.validated.and_then(|validated| {
        match validated.resolve_identity(&inputs.teams_list) {
            Ok(resolved) => {
                messages.push(identified_message(export.clone(), &resolved.identity));
                messages.extend(model_name_messages(
                    &resolved,
                    inputs.common.pes_version,
                    source.export_id,
                    &hand_weighted,
                ));
                messages.extend(no_model_messages(
                    &resolved,
                    inputs.common.pes_version,
                    source.export_id,
                    &hand_weighted,
                ));
                let exhausted = pool_messages(
                    &resolved,
                    inputs.common.pes_version,
                    &export,
                    &hand_weighted,
                );
                if exhausted.is_empty() {
                    Some(resolved)
                } else {
                    messages.extend(exhausted);
                    None
                }
            }
            Err(error) => {
                messages.push(tool_message(
                    Code::TeamNameUnknown,
                    export.clone(),
                    Disposition::DropExport,
                    vec![("team_name", error.team_name.as_str().to_owned())],
                ));
                None
            }
        }
    });
    CheckedSource {
        source,
        messages,
        resolved,
        team_colors,
        notes,
        hand_weighted,
        metal_models,
        revision: Some(revision),
        content,
    }
}

/// The ID of `parsed`'s team, read from the teams list before validation resolves the
/// identity; `None` for an export with none (a referee export, a team the list does not hold,
/// a name with no team token).
fn team_id(inputs: &RunInputs, parsed: &ParsedAestheticsExport) -> Option<TeamId> {
    parsed
        .draft
        .team_name
        .as_ref()
        .and_then(|name| inputs.teams_list.id_of(name))
}

/// The stems of the textures the `installed` CPKs hold in the Fox Common output of team
/// `team_id`, which the export's texture `.common` links may name (`pipeline.md` "Resolved
/// decisions", "A texture a model names must exist"). An export with no team ID has an empty
/// set, as has a pre-Fox target, where an installed Common texture does not satisfy a link
/// yet.
fn installed_common_textures(
    inputs: &RunInputs,
    installed: &InstalledPaths,
    team_id: Option<TeamId>,
) -> BTreeSet<String> {
    match inputs.common.pes_version.engine() {
        Engine::Fox => {}
        Engine::PreFox => return BTreeSet::new(),
    }
    match team_id {
        Some(team_id) => installed.common_texture_stems(Engine::Fox, team_id.get()),
        None => BTreeSet::new(),
    }
}

/// The stems, folded, of the textures the `installed` CPKs hold in the Common output of team
/// `team_id` for the run's target, which the deep pass looks a pre-Fox `.mtl`'s Common path up
/// in (`deep::content_findings`); `None` when the lookup cannot be made or the export has no
/// team ID, so a texture not in the export may still be installed.
fn installed_common_stems(
    inputs: &RunInputs,
    installed: &InstalledPaths,
    team_id: Option<TeamId>,
) -> Option<BTreeSet<String>> {
    let team_id = team_id?;
    match installed {
        InstalledPaths::Unknown => None,
        InstalledPaths::Known(_) => {
            Some(installed.common_texture_stems(inputs.common.pes_version.engine(), team_id.get()))
        }
    }
}

/// The valid colors of `export`'s root `colors.txt`, read from `content`, or `None` when the
/// export has none. A referee export has no `TeamColor.bin` record, so its file is not read.
/// The deep pass has already reported the file's refused lines, and a read that failed there
/// removed the file from `export`, so nothing is reported here; a read that fails only now is
/// logged and gives no color: the file exists, so it must not be reported as missing.
fn team_colors(export: &ValidatedAestheticsExport, content: &ContentSource) -> Option<Vec<Rgb>> {
    if export.team_name.is_referees() {
        return None;
    }
    let file = export.root.team_colors.as_ref()?;
    match content.read(file.source.as_str()) {
        Ok(bytes) => Some(read_colors_txt(&bytes, TEAM_COLORS).colors),
        Err(failure) => {
            log::debug!(
                "{}: the team colors cannot be read again: {}",
                failure.path,
                failure.error
            );
            Some(Vec::new())
        }
    }
}

/// The text of `export`'s root `notes.txt`, read from `content`, its BOM removed, or `None`
/// when validation kept no note. Validation has reported the file and dropped one that is not
/// UTF-8 or holds only whitespace; a read that fails only now is logged and gives no note.
fn notes(export: &ValidatedAestheticsExport, content: &ContentSource) -> Option<String> {
    let file = export.root.notes.as_ref()?;
    let bytes = match content.read(file.source.as_str()) {
        Ok(bytes) => bytes,
        Err(failure) => {
            log::debug!(
                "{}: the notes cannot be read again: {}",
                failure.path,
                failure.error
            );
            return None;
        }
    };
    match String::from_utf8(bytes) {
        Ok(text) => match text.strip_prefix('\u{feff}') {
            Some(without_bom) => Some(without_bom.to_owned()),
            None => Some(text),
        },
        Err(error) => {
            log::debug!(
                "{}: the notes are no longer UTF-8: {error}",
                file.path.as_str()
            );
            None
        }
    }
}

/// `boots_id_pool_exhausted` and `gloves_id_pool_exhausted`, each when more shared folders of
/// its kind take an id for `version` than the team's block holds, naming the count; the export
/// is dropped (`team_compiler/README.md` TC-MOD-06). A referee export has no block: its ids
/// are its slots' (`k99NN`). `hand_weighted` is the deep pass's, as planning counts the
/// folders with it (`shared_folders_taking_ids`).
fn pool_messages(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
    export: &Scope,
    hand_weighted: &BTreeSet<ScopePath>,
) -> Vec<Message> {
    match resolved.identity {
        ExportIdentity::Team { .. } => {}
        ExportIdentity::Referees => return Vec::new(),
    }
    [
        (SharedKind::Boots, Code::BootsIdPoolExhausted),
        (SharedKind::Gloves, Code::GlovesIdPoolExhausted),
    ]
    .into_iter()
    .filter_map(|(kind, code)| {
        let count =
            shared_folders_taking_ids(&resolved.export, version.engine(), kind, hand_weighted)
                .len();
        (count > usize::from(SHARED_COUNT)).then(|| {
            tool_message(
                code,
                export.clone(),
                Disposition::DropExport,
                vec![("count", count.to_string())],
            )
        })
    })
    .collect()
}

/// `shared_folder_no_model` on each shared boots or gloves folder a mapped player links
/// plainly that holds no model of its kind for `version`, boots first, naming the first
/// player folder linking it plainly, in roster order (`player_folders.md` "Assigns IDs
/// automatically"): planning gives it no ID and no task, and that player wears the game's own.
/// A shared face, or a folder linked beside the player's own model, is a texture source and
/// gets nothing. `hand_weighted` as for `pool_messages`.
fn no_model_messages(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
    export_id: ExportId,
    hand_weighted: &BTreeSet<ScopePath>,
) -> Vec<Message> {
    [SharedKind::Boots, SharedKind::Gloves]
        .into_iter()
        .flat_map(|kind| {
            shared_folders_with_no_model(&resolved.export, version.engine(), kind, hand_weighted)
        })
        .map(|(folder, player)| {
            tool_message(
                Code::SharedFolderNoModel,
                Scope::Folder {
                    export_id,
                    path: folder.path.clone(),
                },
                Disposition::Keep,
                vec![("player", player.path.as_str().to_owned())],
            )
        })
        .collect()
}

/// `fmdl_fcl_hair_fallback` for each Fox model the `fcl_hair` merge takes without being named
/// for it, its name saying nothing about what it is (a model takes its role from its name,
/// at any depth of the folder), and `skl_no_slot` for each `.skl` paired with a `face_high`,
/// `hair_high` or `oral` model, which has no slot to land in and is ignored
/// (`player_folders.md` "Model names", "Subfolders", "SKL pairing"; `team_compiler/README.md`
/// TC-MOD-13), and `face_file_not_used` for each face file of a folder with no face model,
/// which is not read (`pipeline.md` "2. Per-export serial steps", item 4; TC-MOD-32), and
/// `file_not_used` for each file a model folder admits that no package reads, planning giving
/// it no role (the same paragraph, item 4), or a texture of a mapped player folder planning no
/// package (`plans_a_package`, with `hand_weighted` the models the deep pass found carrying
/// hand weights), which gets no textures task: over every mapped player folder and every
/// shared folder, each finding on the folder holding the file. A `.common` model link is
/// reported like the model it brings in, naming the link: the fallback by the linked name's
/// suffix, `skl_no_slot` when `Common/` holds the `.skl` of a slotless model's stem. The roles
/// are `roles::player_file`'s, so a finding never disagrees with the routing: under
/// `ingame_face` a model the hair would take is the boots', and no fallback. A pre-Fox target
/// types a model by its name, so it reports neither the fallback nor `skl_no_slot`. A Fox
/// target reports a folder's own `face.xml` (directly in it) as `xml_ignored_fox`: Fox has no
/// `face.xml`, and the folder's models compile as without it. A pre-Fox target reports a
/// shared boots or gloves folder's own as `xml_ignored_shared`: its output is one model or a
/// `glove.xml`, which no face xml drives. Then `file_not_used` on the export for each
/// `Common/` file of a kind no task reads, on PES 18-21 also every file below a subfolder, and
/// for a refs export's kits, logo, portraits and collars (`referee_messages`).
fn model_name_messages(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
    export_id: ExportId,
    hand_weighted: &BTreeSet<ScopePath>,
) -> Vec<Message> {
    let engine = version.engine();
    let export = &resolved.export;
    let mut messages = Vec::new();
    for folder in mapped_players(export) {
        let models = FolderModels::of_player(folder, export, engine);
        file_role_messages(
            &folder.path,
            &folder.files,
            &models,
            plans_a_package(export, folder, engine, hand_weighted),
            &export.common,
            export_id,
            &mut messages,
        );
    }
    // Validation drops a shared folder no mapped player links, so every shared folder here is
    // one some player's package is assembled from or one compiled on its own, which holds a
    // model of its kind or is `shared_folder_no_model`.
    for (kind, folder) in shared_folders(export) {
        let models = FolderModels::of_shared(&folder.path, &folder.files, kind, engine);
        file_role_messages(
            &folder.path,
            &folder.files,
            &models,
            true,
            &export.common,
            export_id,
            &mut messages,
        );
    }
    // `Common/` is a library: a model, `.mtl` or `.skl` no link or conversion takes is no
    // mistake, a texture is the Common textures task's and a glTF planning's
    // (`model_gltf_unsupported`). A file of a directory the target does not read
    // (`is_read_common_file`: on Fox a subfolder's) is read by nothing whatever its kind.
    for file in &export.common {
        let read_by_no_task = !is_read_common_file(&file.path, engine)
            || matches!(
                file.kind,
                FileKind::Fclo | FileKind::Xml | FileKind::Bin | FileKind::MaterialsToml
            );
        if read_by_no_task {
            messages.push(export_file_not_used(export_id, file.path.as_str()));
        }
    }
    match resolved.identity {
        ExportIdentity::Team { .. } => {}
        ExportIdentity::Referees => referee_messages(export, export_id, &mut messages),
    }
    messages
}

/// `file_not_used` on the export for each of the refs `export`'s kit folders, its logo files,
/// its portraits (a mapped folder's, then `Portraits/`) and its `Collars/` files, each named by
/// its export path: a referee has no kit slot, team logo or player id, so none of them has a
/// place to go, the referees' kits being the template tree's (`blue_port.md` "Referee export
/// processing"), and planning keeps them out.
fn referee_messages(
    export: &ValidatedAestheticsExport,
    export_id: ExportId,
    messages: &mut Vec<Message>,
) {
    let kits = export.kits.kits.values().map(|kit| &kit.path);
    let logo = export
        .logo
        .iter()
        .flat_map(|logo| std::iter::once(&logo.main).chain(&logo.small))
        .map(|file| &file.file.path);
    let portraits = mapped_players(export)
        .into_iter()
        .filter_map(|folder| folder.portrait.as_ref())
        .chain(export.portraits.values())
        .map(|file| &file.path);
    let collars = export.collars.iter().map(|file| &file.path);
    for path in kits.chain(logo).chain(portraits).chain(collars) {
        messages.push(export_file_not_used(export_id, path.as_str()));
    }
}

/// `file_not_used` on the export `export_id`, naming the file (or a kit folder) at the export
/// path `path`.
fn export_file_not_used(export_id: ExportId, path: &str) -> Message {
    tool_message(
        Code::FileNotUsed,
        Scope::Export { export_id },
        Disposition::Keep,
        vec![("file", path.to_owned())],
    )
}

/// `model_name_messages`'s findings on `files`, the files of the folder at `path` whose models
/// are `models`, for the target `models` were computed for, in file order; `plans_package`
/// says whether planning gives the folder a package task (`plans_a_package`), without which
/// its textures are `file_not_used`; `common` is the export's `Common/` files, where a
/// `.common` link's model and skeleton are.
fn file_role_messages(
    path: &ScopePath,
    files: &[FileDescriptor],
    models: &FolderModels,
    plans_package: bool,
    common: &[FileDescriptor],
    export_id: ExportId,
    messages: &mut Vec<Message>,
) {
    for file in files {
        let name = file.path.name();
        // Fox has no `face.xml`: a member's own has no role there, and is ignored. Pre-Fox
        // ignores a shared boots or gloves folder's: its output is one model or a `glove.xml`,
        // which no face xml drives.
        let ignored_xml = match models.engine() {
            Engine::Fox if is_user_face_xml(path, file, models.is_shared()) => {
                Some(Code::XmlIgnoredFox)
            }
            Engine::PreFox
                if models.is_shared_boots_or_gloves()
                    && is_user_face_xml(path, file, models.is_shared()) =>
            {
                Some(Code::XmlIgnoredShared)
            }
            Engine::Fox | Engine::PreFox => None,
        };
        if let Some(code) = ignored_xml {
            messages.push(tool_message(
                code,
                Scope::Folder {
                    export_id,
                    path: path.clone(),
                },
                Disposition::Keep,
                vec![("file", deep::relative(&file.path, path))],
            ));
            continue;
        }
        let code = match player_file(path, file, models) {
            Some(PlayerFile::Model {
                package: ModelPackage::Face,
                name: "fcl_hair",
            }) if model_suffix(file_stem(name)) != Some(ModelSuffix::FclHair) => {
                Code::FmdlFclHairFallback
            }
            Some(PlayerFile::SlotlessSkeleton) => Code::SklNoSlot,
            Some(PlayerFile::UnusedFaceFile) => Code::FaceFileNotUsed,
            // Planning gives a folder planning no package no textures task.
            Some(PlayerFile::Texture { .. }) if !plans_package => Code::FileNotUsed,
            Some(PlayerFile::CommonModel {
                package: ModelPackage::Face,
                name: allowed,
            }) => {
                let linked = common_link_name(name)
                    .expect("a CommonModel role implies a `.common` link name");
                if allowed == "fcl_hair" {
                    if model_suffix(file_stem(&linked)) == Some(ModelSuffix::FclHair) {
                        continue;
                    }
                    Code::FmdlFclHairFallback
                } else if common_skeleton(common, &linked).is_some() {
                    Code::SklNoSlot
                } else {
                    continue;
                }
            }
            Some(
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel {
                    package: ModelPackage::Boots | ModelPackage::Gloves,
                    ..
                }
                | PlayerFile::Packed { .. }
                | PlayerFile::FaceDiffXml
                | PlayerFile::Skeleton { .. }
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::Texture { .. }
                | PlayerFile::CommonTexture(_)
                | PlayerFile::PreFoxModel { .. }
                | PlayerFile::PreFoxPart { .. }
                | PlayerFile::PreFoxCommonModel { .. }
                | PlayerFile::Material
                | PlayerFile::CommonMaterial
                | PlayerFile::FaceXml
                | PlayerFile::ConversionSkeleton
                | PlayerFile::UnsupportedGltf,
            ) => continue,
            // Named below the folder, as `xml_ignored_fox` names its file: `gloves/keeper.fmdl`
            // says where the member put it.
            None if !unread_for_a_known_reason(path, file, models) => {
                messages.push(tool_message(
                    Code::FileNotUsed,
                    Scope::Folder {
                        export_id,
                        path: path.clone(),
                    },
                    Disposition::Keep,
                    vec![("file", deep::relative(&file.path, path))],
                ));
                continue;
            }
            None => continue,
        };
        // Named below the folder, as `file_not_used` names its file: `gloves/keeper.fmdl`
        // takes its role from its name, and the path says where the member put it.
        messages.push(tool_message(
            code,
            Scope::Folder {
                export_id,
                path: path.clone(),
            },
            Disposition::Keep,
            vec![("file", deep::relative(&file.path, path))],
        ));
    }
}

/// Whether `file` of the model folder at `path`, which has no role among `models`, is left
/// unread for a reason that needs no `file_not_used` (`messages.md` `file_not_used`, "Not
/// reported"): a model another representation of its stem beats (TC-MOD-26); on PES 15-17 a
/// Fox file the target has no counterpart for, an FMDL with no role, a `.skl` no converted
/// FMDL pairs or a `.fclo` (`pipeline.md` step 3 "Format conversion"); a file the structure
/// pass names when it is out of place (`file_type_disallowed`): a marker, metadata, a shared
/// folder link or a file of no known kind, one outside the places a model folder admits a file
/// with a role (`roles::role_position`), and a `.common` link in a shared folder.
fn unread_for_a_known_reason(
    path: &ScopePath,
    file: &FileDescriptor,
    models: &FolderModels,
) -> bool {
    let other_engine_companion = match models.engine() {
        Engine::Fox => false,
        Engine::PreFox => matches!(
            file.kind,
            FileKind::Model(ModelFormat::Fmdl) | FileKind::Skl | FileKind::Fclo
        ),
    };
    let named_by_the_structure_pass = matches!(
        file.kind,
        FileKind::Marker(_) | FileKind::Metadata(_) | FileKind::SharedLink(_) | FileKind::Other
    ) || !role_position(path, file, models.is_shared())
        || (models.is_shared() && file.kind == FileKind::CommonLink);
    models.beaten(file) || other_engine_companion || named_by_the_structure_pass
}

/// `export_identified` naming the team and its id, or the referees.
fn identified_message(export: Scope, identity: &ExportIdentity) -> Message {
    let context = match identity {
        ExportIdentity::Team { id, name } => {
            vec![("team", name.as_str().to_owned()), ("id", id.to_string())]
        }
        ExportIdentity::Referees => vec![("team", "referees".to_owned())],
    };
    tool_message(Code::ExportIdentified, export, Disposition::Keep, context)
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::{resolved, resolved_with_issues, scratch};

    /// The export `co Midcup Pool` whose slots 01 to `count` each link their own shared folder of
    /// `kind` (`Boots/S01/` for slot 01), each player folder also holding `local`.
    fn linking_export(count: u8, kind: SharedKind, local: &[&str]) -> ResolvedAestheticsExport {
        let (extension, content_folder, model) = match kind {
            SharedKind::Face => ("face", "Faces", "face_high.fmdl"),
            SharedKind::Boots => ("boots", "Boots", "boots.fmdl"),
            SharedKind::Gloves => ("gloves", "Gloves", "glove_l.fmdl"),
        };
        let mut files = Vec::new();
        for slot in 1..=count {
            let player = format!("Players/{slot:02} - P{slot:02}");
            files.push(format!("{player}/S{slot:02}.{extension}"));
            for name in local {
                files.push(format!("{player}/{name}"));
            }
            files.push(format!("{content_folder}/S{slot:02}/{model}"));
        }
        let files: Vec<(&str, u64)> = files.iter().map(|path| (path.as_str(), 1)).collect();
        resolved("co Midcup Pool", &files, &[], None)
    }

    /// `pool_messages` over `export` for `version`, each as (code, disposition, count).
    fn pool(
        export: &ResolvedAestheticsExport,
        version: PesVersion,
    ) -> Vec<(String, Disposition, String)> {
        let scope = Scope::Export {
            export_id: ExportId(0),
        };
        pool_messages(export, version, &scope, &BTreeSet::new())
            .into_iter()
            .map(|message| {
                assert_eq!(message.scope, scope);
                assert_eq!(message.severity, Severity::Error);
                let [(key, count)] = message.context.as_slice() else {
                    panic!("{:?}", message.context);
                };
                assert_eq!(key, "count");
                (
                    message.code.code.to_string(),
                    message.disposition,
                    count.clone(),
                )
            })
            .collect()
    }

    /// `model_name_messages` over `export` for `version`, each as one line: severity, code,
    /// disposition, folder (none for a finding on the export) and context.
    fn names(export: &ResolvedAestheticsExport, version: PesVersion) -> Vec<String> {
        model_name_messages(export, version, ExportId(2), &BTreeSet::new())
            .into_iter()
            .map(line)
            .collect()
    }

    /// `message`, a finding on export 2, as one line: severity, code, disposition, folder
    /// (none for a finding on the export) and context.
    fn line(message: Message) -> String {
        let location = match &message.scope {
            Scope::Folder { export_id, path } => {
                assert_eq!(*export_id, ExportId(2));
                format!(" at {}", path.as_str())
            }
            Scope::Export { export_id } => {
                assert_eq!(*export_id, ExportId(2));
                String::new()
            }
            Scope::Run | Scope::File { .. } | Scope::RosterEntry { .. } => {
                panic!("{:?}", message.scope)
            }
        };
        let context: Vec<String> = message
            .context
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        format!(
            "{:?} {} [{:?}]{location} ({})",
            message.severity,
            message.code.code,
            message.disposition,
            context.join(", ")
        )
    }

    #[test]
    fn an_unsuffixed_model_and_a_slotless_skeleton_are_reported_on_their_folder_on_fox_only() {
        let files = [
            ("Players/03 - A/torso.fmdl", 1),
            ("Players/03 - A/torso.skl", 1),
            ("Players/03 - A/face_high.fmdl", 1),
            ("Players/03 - A/face_high.skl", 1),
            ("Players/03 - A/x_fcl_hair.fmdl", 1),
            ("Players/03 - A/x_fcl_hair.skl", 1),
            ("Players/07 - B/Round.face", 0),
            ("Players/07 - B/kit_boots.fmdl", 1),
            ("Players/07 - B/kit_boots.skl", 1),
            ("Faces/Round/legs.fmdl", 1),
            ("Faces/Round/oral.fmdl", 1),
            ("Faces/Round/oral.skl", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        // The hair's and the boots' skeletons have their slots; a shared face folder's files
        // are reported on that folder, once, however many players link it. Within a folder
        // the findings follow its files, in the folded name order validation keeps them in.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning skl_no_slot [Keep] at Players/03 - A (file=face_high.skl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=torso.fmdl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Faces/Round (file=legs.fmdl)",
                "Warning skl_no_slot [Keep] at Faces/Round (file=oral.skl)",
            ]
        );
        assert_eq!(names(&export, PesVersion::Pes17), Vec::<String>::new());
    }

    #[test]
    fn a_subfolder_s_part_is_reported_by_its_own_name_naming_it_below_the_folder() {
        let files = [
            ("Players/03 - A/boots/hair_high.fmdl", 1),
            ("Players/03 - A/boots/hair_high.skl", 1),
            ("Players/03 - A/face/boots.fmdl", 1),
            ("Players/03 - A/face/torso.fmdl", 1),
            ("Players/03 - A/gloves/keeper.fmdl", 1),
            ("Players/03 - A/fcl_hair.fmdl", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        // No subfolder name forces a category: `hair_high` in `boots/` is face content, whose
        // skeleton has no slot, `boots.fmdl` in `face/` the boots, and a model named for no part
        // in `face/` or `gloves/` hair content, each named below the folder.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning skl_no_slot [Keep] at Players/03 - A (file=boots/hair_high.skl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=face/torso.fmdl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=gloves/keeper.fmdl)",
            ]
        );
    }

    #[test]
    fn a_common_link_is_reported_like_the_model_it_brings_in_naming_the_link() {
        let files = [
            ("Players/03 - A/legs.fmdl.common", 0),
            ("Players/03 - A/x_fcl_hair.fmdl.common", 0),
            ("Players/03 - A/face_high.fmdl.common", 0),
            ("Players/03 - A/oral.fmdl.common.txt", 0),
            ("Common/legs.fmdl", 1),
            ("Common/legs.skl", 1),
            ("Common/x_fcl_hair.fmdl", 1),
            ("Common/face_high.fmdl", 1),
            ("Common/face_high.skl", 1),
            ("Common/oral.fmdl", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        // `legs` is hair content by its name; `face_high` has no slot for Common's skeleton,
        // and `oral` has none to report.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning skl_no_slot [Keep] at Players/03 - A (file=face_high.fmdl.common)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=legs.fmdl.common)",
            ]
        );
        assert_eq!(names(&export, PesVersion::Pes17), Vec::<String>::new());
    }

    #[test]
    fn under_ingame_face_a_model_the_hair_would_take_is_no_fallback() {
        let files = [
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/torso.fmdl", 1),
            ("Players/03 - A/face/hat.fmdl", 1),
            ("Players/03 - A/legs.fmdl.common", 0),
            ("Common/legs.fmdl", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        assert_eq!(names(&export, PesVersion::Pes21), Vec::<String>::new());
    }

    #[test]
    fn each_face_file_of_a_folder_with_no_face_model_is_not_used_in_file_order() {
        let files = [
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/boots.fmdl", 1),
            ("Players/03 - A/fcl_hair_sim.fclo", 1),
            ("Players/03 - A/face_diff.bin", 1),
            ("Players/05 - B/kit_boots.fmdl", 1),
            ("Players/05 - B/face_diff.xml", 1),
            ("Players/05 - B/fcl_hair_sim.fclo", 1),
            ("Players/07 - C/face_high.fmdl", 1),
            ("Players/07 - C/face_diff.bin", 1),
            ("Players/07 - C/fcl_hair_sim.fclo", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        // A marked folder and an unmarked one with no face model; a face model uses them.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Info face_file_not_used [Keep] at Players/03 - A (file=face_diff.bin)",
                "Info face_file_not_used [Keep] at Players/03 - A (file=fcl_hair_sim.fclo)",
                "Info face_file_not_used [Keep] at Players/05 - B (file=face_diff.xml)",
                "Info face_file_not_used [Keep] at Players/05 - B (file=fcl_hair_sim.fclo)",
            ]
        );
        // Pre-Fox converts an unmarked folder's `.fmdl` for the face, which uses its face
        // diff, and gives a `.fclo` no role, so the simulation file is not reported.
        assert_eq!(
            names(&export, PesVersion::Pes17),
            ["Info face_file_not_used [Keep] at Players/03 - A (file=face_diff.bin)"]
        );
    }

    #[test]
    fn a_face_s_own_face_xml_is_ignored_on_fox_only() {
        let files = [
            ("Players/03 - A/face_high.fmdl", 1),
            ("Players/03 - A/face.xml", 1),
            ("Players/05 - B/kit_boots.fmdl", 1),
            ("Players/05 - B/face.xml", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        // In a folder with no face model too.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Info xml_ignored_fox [Keep] at Players/03 - A (file=face.xml)",
                "Info xml_ignored_fox [Keep] at Players/05 - B (file=face.xml)",
            ]
        );
        assert_eq!(names(&export, PesVersion::Pes17), Vec::<String>::new());
    }

    #[test]
    fn a_shared_folder_s_own_face_xml_is_ignored_on_fox_and_in_boots_or_gloves_on_pre_fox() {
        let files = [
            ("Players/05 - A/Round.face", 0),
            ("Players/05 - A/Crocs.boots", 0),
            ("Faces/Round/face_high.model", 1),
            ("Faces/Round/face_high.mtl", 1),
            ("Faces/Round/face.xml", 1),
            ("Boots/Crocs/boots.model", 1),
            ("Boots/Crocs/boots.mtl", 1),
            ("Boots/Crocs/face.xml", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        // On pre-Fox a shared face's xml is the xml of each face linking it, read with no
        // finding; a boots folder's output is one model, which no face xml drives.
        assert_eq!(
            names(&export, PesVersion::Pes17),
            ["Info xml_ignored_shared [Keep] at Boots/Crocs (file=face.xml)"]
        );
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Info xml_ignored_fox [Keep] at Faces/Round (file=face.xml)",
                "Info xml_ignored_fox [Keep] at Boots/Crocs (file=face.xml)",
            ]
        );
    }

    #[test]
    fn on_pre_fox_a_link_to_a_per_kit_model_is_file_not_used() {
        let files = [
            ("Players/05 - A/pants_kit1.model.common", 0),
            ("Common/pants_kit1.model", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        assert_eq!(
            names(&export, PesVersion::Pes17),
            ["Warning file_not_used [Keep] at Players/05 - A (file=pants_kit1.model.common)"]
        );
    }

    #[test]
    fn a_player_folder_no_slot_maps_is_not_walked() {
        let (export, issues) = resolved_with_issues(
            "co Midcup Names",
            &[
                ("Players/A/face_high.fmdl", 1),
                ("Players/Unlisted/torso.fmdl", 1),
            ],
            &[],
            Some(b"03 A\n"),
        );
        assert_eq!(issues, ["player_unlisted"]);
        assert_eq!(names(&export, PesVersion::Pes21), Vec::<String>::new());
    }

    #[test]
    fn a_file_no_package_reads_is_file_not_used_on_its_folder_named_below_it() {
        let files = [
            // A `.bin` other than the face diff, below a subfolder; a glove's skeleton (no glove
            // has a slot); a `.model` the FMDL of its stem beats.
            ("Players/03 - A/boots.fmdl", 1),
            ("Players/03 - A/boots.model", 1),
            ("Players/03 - A/boots/extra.bin", 1),
            ("Players/03 - A/face_high.fmdl", 1),
            ("Players/03 - A/glove_l.fmdl", 1),
            ("Players/03 - A/glove_l.skl", 1),
            // A pre-Fox face, a `.bin` deeper down, and a skeleton pairing no model.
            ("Players/05 - B/face_high.model", 1),
            ("Players/05 - B/face_high.mtl", 1),
            ("Players/05 - B/parts/old/extra.bin", 1),
            ("Players/05 - B/torso.skl", 1),
        ];
        let export = resolved("co Midcup Names", &files, &[], None);
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning file_not_used [Keep] at Players/03 - A (file=boots/extra.bin)",
                "Warning file_not_used [Keep] at Players/03 - A (file=glove_l.skl)",
                "Warning file_not_used [Keep] at Players/05 - B (file=parts/old/extra.bin)",
                "Warning file_not_used [Keep] at Players/05 - B (file=torso.skl)",
            ]
        );
        // Pre-Fox converts the FMDLs (`glove_l.skl` the bind pose of `glove_l.fmdl`), and an
        // FMDL or a `.skl` with no role there is the other engine's companion: not reported.
        assert_eq!(
            names(&export, PesVersion::Pes17),
            [
                "Warning file_not_used [Keep] at Players/03 - A (file=boots/extra.bin)",
                "Warning file_not_used [Keep] at Players/05 - B (file=parts/old/extra.bin)",
            ]
        );
    }

    #[test]
    fn a_file_the_structure_pass_named_is_not_file_not_used_again() {
        let files = [
            ("Players/05 - A/face_high.fmdl", 1),
            ("Players/05 - A/torso.skl", 1),
        ];
        let mut export = resolved("co Midcup Names", &files, &[], None);
        // Kept only with the strict file-type check off, which `resolved` has on: added as the
        // structure pass would keep them, with their `file_type_disallowed` (a link below a
        // subfolder is no link).
        for name in ["extra/hair.dds.common", "common/legs.fmdl.common"] {
            let path = ScopePath::new(&format!("Players/05 - A/{name}")).unwrap();
            export.export.players[0].files.push(FileDescriptor {
                size: 0,
                kind: aesthetics_export::classify(path.name()),
                source: path.clone(),
                path,
            });
        }
        assert_eq!(
            names(&export, PesVersion::Pes21),
            ["Warning file_not_used [Keep] at Players/05 - A (file=torso.skl)"]
        );
    }

    #[test]
    fn a_shared_folder_s_and_common_s_files_no_task_reads_are_file_not_used() {
        let files = [
            ("Players/03 - A/face_high.fmdl", 1),
            ("Players/03 - A/Round.face", 0),
            ("Players/03 - A/Grip.gloves", 0),
            ("Faces/Round/hair_high.fmdl", 1),
            ("Gloves/Grip/glove_l.fmdl", 1),
            ("Gloves/Grip/glove_l.skl", 1),
            ("Common/notes.xml", 1),
            ("Common/x.mtl", 1),
            ("Common/x.model", 1),
            ("Common/x.skl", 1),
            ("Common/x.dds", 1),
        ];
        let mut export = resolved("co Midcup Names", &files, &[], None);
        // A `.common` link in a shared folder is kept only with the strict file-type check
        // off, which `resolved` has on: added as the structure pass would keep it, with its
        // `file_type_disallowed`.
        let link = ScopePath::new("Faces/Round/legs.fmdl.common").unwrap();
        export.export.faces[0].files.push(FileDescriptor {
            size: 0,
            kind: aesthetics_export::classify(link.name()),
            source: link.clone(),
            path: link,
        });
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning file_not_used [Keep] at Gloves/Grip (file=glove_l.skl)",
                "Warning file_not_used [Keep] (file=Common/notes.xml)",
            ]
        );
        assert_eq!(
            names(&export, PesVersion::Pes17),
            ["Warning file_not_used [Keep] (file=Common/notes.xml)"]
        );
    }

    #[test]
    fn a_refs_export_s_kits_logo_portraits_and_collars_are_file_not_used_and_a_team_s_are_not() {
        let content = [
            ("Kits/p1/kit.dds", 1),
            ("logo.png", 1),
            ("Collars/collar_12.fmdl", 1),
        ];
        let referees: Vec<(&str, u64)> = [
            ("Players/Ref A/face_high.fmdl", 1),
            ("Players/Ref A/portrait.dds", 1),
            ("Portraits/player_02.dds", 1),
        ]
        .into_iter()
        .chain(content)
        .collect();
        let export = resolved("refs Cup", &referees, &[], Some(b"01 Ref A\n"));
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning file_not_used [Keep] (file=Kits/p1)",
                "Warning file_not_used [Keep] (file=logo.png)",
                "Warning file_not_used [Keep] (file=Players/Ref A/portrait.dds)",
                "Warning file_not_used [Keep] (file=Portraits/player_02.dds)",
                "Warning file_not_used [Keep] (file=Collars/collar_12.fmdl)",
            ]
        );
        let team: Vec<(&str, u64)> = [("Players/03 - A/face_high.fmdl", 1)]
            .into_iter()
            .chain(content)
            .collect();
        let export = resolved("co Midcup Names", &team, &[], None);
        assert_eq!(names(&export, PesVersion::Pes21), Vec::<String>::new());
    }

    /// `team_colors` over the folder export `name`, in the scratch folder `scratch_name`,
    /// holding `files` (path, size) and, when given, a root `colors.txt` of `colors`.
    fn colors_of(
        scratch_name: &str,
        name: &str,
        files: &[(&str, u64)],
        players_txt: Option<&[u8]>,
        colors: Option<&[u8]>,
    ) -> Option<Vec<Rgb>> {
        let temp = scratch(scratch_name);
        let mut files = files.to_vec();
        if let Some(bytes) = colors {
            std::fs::write(temp.path().join("colors.txt"), bytes).unwrap();
            files.push(("colors.txt", bytes.len() as u64));
        }
        let export = resolved(name, &files, &[], players_txt);
        let source = ExportSource {
            export_id: ExportId(0),
            path: temp.path().to_path_buf(),
            kind: SourceKind::Folder,
            file_name: name.to_owned(),
            display_name: name.to_owned(),
            team_name: None,
        };
        team_colors(
            &export.export,
            &ContentSource::new(&source, &MemoryBudget::new(1 << 30)),
        )
    }

    #[test]
    fn a_team_s_root_colors_txt_gives_its_valid_colors() {
        let kit = [("Kits/p1/kit.dds", 1)];
        assert_eq!(
            colors_of(
                "team_colors_two",
                "co Midcup Colors",
                &kit,
                None,
                Some(b"#c11200\n#414141\n")
            ),
            Some(vec![[0xc1, 0x12, 0x00], [0x41, 0x41, 0x41]])
        );
        // A line that gives no color, which the deep pass reported, and a fifth color past
        // the record's four.
        assert_eq!(
            colors_of(
                "team_colors_capped",
                "co Midcup Colors",
                &kit,
                None,
                Some(b"bad\n1 2 3\n4 5 6\n7 8 9\n10 11 12\n13 14 15\n")
            ),
            Some(vec![[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]])
        );
        assert_eq!(
            colors_of(
                "team_colors_invalid",
                "co Midcup Colors",
                &kit,
                None,
                Some(b"bad\n")
            ),
            Some(Vec::new())
        );
        assert_eq!(
            colors_of("team_colors_none", "co Midcup Colors", &kit, None, None),
            None
        );
    }

    #[test]
    fn a_referee_export_s_colors_txt_is_not_read() {
        assert_eq!(
            colors_of(
                "team_colors_referees",
                "refs Cup",
                &[("Players/Keeper/face_high.fmdl", 1)],
                Some(b"01 Keeper\n"),
                Some(b"#c11200\n")
            ),
            None
        );
    }

    /// `notes` over a folder export `co Midcup Notes`, in the scratch folder `scratch_name`, whose
    /// root `notes.txt` validation kept holding `bytes`, or without one.
    fn notes_of(scratch_name: &str, bytes: Option<&[u8]>) -> Option<String> {
        let temp = scratch(scratch_name);
        let mut export = resolved("co Midcup Notes", &[("Kits/p1/kit.dds", 1)], &[], None);
        if let Some(bytes) = bytes {
            std::fs::write(temp.path().join("notes.txt"), bytes).unwrap();
            let path = ScopePath::new("notes.txt").unwrap();
            export.export.root.notes = Some(FileDescriptor {
                size: bytes.len() as u64,
                kind: aesthetics_export::classify(path.name()),
                source: path.clone(),
                path,
            });
        }
        let source = ExportSource {
            export_id: ExportId(0),
            path: temp.path().to_path_buf(),
            kind: SourceKind::Folder,
            file_name: "co Midcup Notes".to_owned(),
            display_name: "co Midcup Notes".to_owned(),
            team_name: None,
        };
        notes(
            &export.export,
            &ContentSource::new(&source, &MemoryBudget::new(1 << 30)),
        )
    }

    #[test]
    fn a_kept_notes_txt_gives_its_text_without_its_bom_and_no_note_gives_none() {
        assert_eq!(
            notes_of(
                "notes_bom",
                Some(b"\xEF\xBB\xBFOur GK kit\r\nis invisible.\r\n")
            )
            .as_deref(),
            Some("Our GK kit\r\nis invisible.\r\n")
        );
        assert_eq!(
            notes_of("notes_plain", Some(b"Hello")).as_deref(),
            Some("Hello")
        );
        assert_eq!(notes_of("notes_none", None), None);
    }

    /// The folder source `file_name`, numbered `export_id`: the duplicate rule reads only the
    /// id and the file name.
    fn source(export_id: u64, file_name: &str) -> ExportSource {
        ExportSource {
            export_id: ExportId(export_id),
            path: file_name.into(),
            kind: SourceKind::Folder,
            file_name: file_name.to_owned(),
            display_name: file_name.to_owned(),
            team_name: aesthetics_export::team_name(file_name),
        }
    }

    /// `file_name`, numbered `export_id`, as validation leaves a clean export: identified as
    /// its name's team, holding one kit.
    fn identified(export_id: u64, file_name: &str) -> CheckedSource {
        let resolved = resolved(file_name, &[("Kits/p1/kit.dds", 1)], &[], None);
        let export = Scope::Export {
            export_id: ExportId(export_id),
        };
        CheckedSource {
            source: source(export_id, file_name),
            messages: vec![identified_message(export, &resolved.identity)],
            resolved: Some(resolved),
            team_colors: None,
            notes: None,
            hand_weighted: BTreeSet::new(),
            metal_models: BTreeSet::new(),
            revision: None,
            content: None,
        }
    }

    /// `file_name`, numbered `export_id`, as validation leaves an export its `NO_USE` marker
    /// disables: no identity.
    fn disabled(export_id: u64, file_name: &str) -> CheckedSource {
        let export = Scope::Export {
            export_id: ExportId(export_id),
        };
        CheckedSource {
            source: source(export_id, file_name),
            messages: vec![tool_message(
                Code::ExportDisabled,
                export,
                Disposition::DropExport,
                vec![],
            )],
            resolved: None,
            team_colors: None,
            notes: None,
            hand_weighted: BTreeSet::new(),
            metal_models: BTreeSet::new(),
            revision: None,
            content: None,
        }
    }

    /// Each of `sources` after the duplicate rule, as its file name, whether it is still
    /// resolved, and its messages, each as one line: severity, code, disposition and context.
    fn after_duplicate_rule(mut sources: Vec<CheckedSource>) -> Vec<(String, bool, Vec<String>)> {
        refuse_duplicate_teams(&mut sources);
        sources
            .into_iter()
            .map(|checked| {
                let export = Scope::Export {
                    export_id: checked.source.export_id,
                };
                let lines = checked
                    .messages
                    .iter()
                    .map(|message| {
                        assert_eq!(message.scope, export);
                        let context: Vec<String> = message
                            .context
                            .iter()
                            .map(|(key, value)| format!("{key}={value}"))
                            .collect();
                        format!(
                            "{:?} {} [{:?}] ({})",
                            message.severity,
                            message.code.code,
                            message.disposition,
                            context.join(", ")
                        )
                    })
                    .collect();
                (checked.source.file_name, checked.resolved.is_some(), lines)
            })
            .collect()
    }

    /// One `after_duplicate_rule` entry.
    fn outcome(file_name: &str, resolved: bool, lines: &[&str]) -> (String, bool, Vec<String>) {
        (
            file_name.to_owned(),
            resolved,
            lines.iter().map(|line| (*line).to_owned()).collect(),
        )
    }

    const IDENTIFIED_702: &str = "Info export_identified [Keep] (team=/a/, id=702)";
    const IDENTIFIED_714: &str = "Info export_identified [Keep] (team=/co/, id=714)";

    #[test]
    fn two_exports_of_one_team_are_both_refused_and_the_other_team_s_kept() {
        let duplicate = "Error duplicate_aesthetics_export [DropExport] (id=714, exports=co Midcup A, co Midcup B.zip)";
        assert_eq!(
            after_duplicate_rule(vec![
                identified(0, "a Midcup Home"),
                identified(1, "co Midcup A"),
                identified(2, "co Midcup B.zip"),
            ]),
            [
                outcome("a Midcup Home", true, &[IDENTIFIED_702]),
                outcome("co Midcup A", false, &[IDENTIFIED_714, duplicate]),
                outcome("co Midcup B.zip", false, &[IDENTIFIED_714, duplicate]),
            ]
        );
    }

    #[test]
    fn a_refused_duplicate_lets_go_of_the_source_kept_for_its_tasks() {
        let budget = MemoryBudget::new(1 << 30);
        let mut sources = vec![
            identified(0, "a Midcup Home"),
            identified(1, "co Midcup A"),
            identified(2, "co Midcup B"),
        ];
        for checked in &mut sources {
            checked.content = Some(ContentSource::new(&checked.source, &budget));
        }

        refuse_duplicate_teams(&mut sources);

        let kept: Vec<bool> = sources
            .iter()
            .map(|checked| checked.content.is_some())
            .collect();
        assert_eq!(kept, [true, false, false]);
    }

    #[test]
    fn an_archive_is_kept_when_at_most_an_eighth_of_the_cap_and_the_floor_stays_free() {
        const GIB: usize = 1 << 30;
        let cap = 8 * GIB;
        assert!(keeps_archive(GIB, 0, cap, 0));
        assert!(
            !keeps_archive(GIB + 1, 0, cap, 0),
            "over an eighth of the cap"
        );
        assert!(
            keeps_archive(GIB, 6 * GIB, cap, 0),
            "7 GiB kept leaves the floor"
        );
        assert!(
            !keeps_archive(GIB, 6 * GIB + 1, cap, 0),
            "the floor is not left"
        );
        // A cap of the floor alone leaves nothing to keep.
        assert!(!keeps_archive(1, 0, GIB, 0));
        assert!(!keeps_archive(GIB / 8, 0, GIB, 0));
        // A total past `usize` is over the cap, not a wrap into a keep.
        assert!(!keeps_archive(1, usize::MAX, usize::MAX, 0));
    }

    #[test]
    fn an_archive_is_kept_only_when_the_largest_other_archive_still_fits_beside_the_kept() {
        const GIB: usize = 1 << 30;
        let cap = 8 * GIB;
        // `cap - 7 GiB` leaves 1 GiB, exactly this archive.
        assert!(keeps_archive(GIB, 0, cap, 7 * GIB));
        assert!(!keeps_archive(GIB, 0, cap, 7 * GIB + 1));
        // Another archive over the cap acquires only on an empty budget: nothing is kept.
        assert!(!keeps_archive(GIB, 0, cap, 9 * GIB));
        assert!(!keeps_archive(1, 0, cap, 9 * GIB));
    }

    /// The validation pass over the `.7z` fixtures `co Midcup Spring.7z` and `egg Midcup
    /// Tracer.7z` on a budget of `cap` bytes, with `keep_archives`: each source's file name and
    /// the bytes its kept content holds, `None` when none is kept.
    fn seven_z_pass(
        scratch_name: &str,
        cap: usize,
        keep_archives: bool,
    ) -> Vec<(String, Option<usize>)> {
        let temp = scratch(scratch_name);
        let root = temp.path();
        let exports = root.join("exports");
        std::fs::create_dir_all(&exports).unwrap();
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sources");
        for name in ["co Midcup Spring.7z", "egg Midcup Tracer.7z"] {
            std::fs::copy(fixtures.join(name), exports.join(name)).unwrap();
        }
        let inputs = RunInputs {
            settings: crate::settings::TeamCompilerSettings::default(),
            common: crate::testing::tool_context(root, "").common(),
            teams_list: teams_list::TeamsList::parse("ID\tName\n714\t/co/\n792\t/egg/\n").unwrap(),
            exports_root: exports.clone(),
            exports: Vec::new(),
        };
        let sources = reader::discover(&exports, &[]).unwrap();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let budget = MemoryBudget::new(cap);

        let pass = validation_pass(
            &inputs,
            sources,
            &InstalledPaths::Unknown,
            &budget,
            &pool,
            keep_archives,
        )
        .unwrap();

        pass.sources
            .into_iter()
            .map(|checked| {
                let held = checked.content.as_ref().map(ContentSource::held);
                (checked.source.file_name, held)
            })
            .collect()
    }

    #[test]
    fn compile_keeps_both_small_7z_exports_for_their_tasks_and_check_keeps_none() {
        // The archives decompress to 34 and 153,538 bytes, 153,572 kept in all: far under an
        // eighth of the cap, and the cap less the floor leaves room for both.
        assert_eq!(
            seven_z_pass("keep_7z_compile", 2 << 30, true),
            [
                ("co Midcup Spring.7z".to_owned(), Some(34)),
                ("egg Midcup Tracer.7z".to_owned(), Some(153_538)),
            ]
        );
        assert_eq!(
            seven_z_pass("keep_7z_check", 2 << 30, false),
            [
                ("co Midcup Spring.7z".to_owned(), None),
                ("egg Midcup Tracer.7z".to_owned(), None),
            ]
        );
    }

    #[test]
    fn an_archive_the_kept_ones_leave_no_room_for_is_not_kept() {
        // The cap leaves 153,560 bytes past the floor: room for the first archive checked,
        // `co Midcup Spring.7z` (34 bytes), or for the tracer (153,538) alone, but 12 bytes
        // short of both (153,572), so the tracer, checked second, is let go.
        assert_eq!(
            seven_z_pass("keep_7z_running_total", (1 << 30) + 153_560, true),
            [
                ("co Midcup Spring.7z".to_owned(), Some(34)),
                ("egg Midcup Tracer.7z".to_owned(), None),
            ]
        );
    }

    #[test]
    fn the_largest_other_size_leaves_out_the_one_at_its_index() {
        let sizes = [5, 9, 7];
        assert_eq!(largest_other(&sizes, 0), 9);
        assert_eq!(largest_other(&sizes, 1), 7);
        assert_eq!(largest_other(&sizes, 2), 9);
        assert_eq!(largest_other(&[5], 0), 0);
    }

    #[test]
    fn a_disabled_export_beside_one_of_its_team_is_no_duplicate() {
        assert_eq!(
            after_duplicate_rule(vec![
                identified(0, "co Midcup A"),
                disabled(1, "co Midcup B")
            ]),
            [
                outcome("co Midcup A", true, &[IDENTIFIED_714]),
                outcome(
                    "co Midcup B",
                    false,
                    &["Info export_disabled [DropExport] ()"]
                ),
            ]
        );
    }

    #[test]
    fn each_of_three_exports_of_one_team_names_all_three() {
        let duplicate = "Error duplicate_aesthetics_export [DropExport] (id=714, exports=co Midcup A, co Midcup B.zip, co Midcup C.7z)";
        assert_eq!(
            after_duplicate_rule(vec![
                identified(0, "co Midcup A"),
                identified(1, "co Midcup B.zip"),
                identified(2, "co Midcup C.7z"),
            ]),
            [
                outcome("co Midcup A", false, &[IDENTIFIED_714, duplicate]),
                outcome("co Midcup B.zip", false, &[IDENTIFIED_714, duplicate]),
                outcome("co Midcup C.7z", false, &[IDENTIFIED_714, duplicate]),
            ]
        );
    }

    #[test]
    fn eighteen_shared_gloves_folders_exhaust_the_pool_and_seventeen_fit() {
        assert_eq!(
            pool(
                &linking_export(18, SharedKind::Gloves, &[]),
                PesVersion::Pes21
            ),
            [(
                "gloves_id_pool_exhausted".to_owned(),
                Disposition::DropExport,
                "18".to_owned()
            )]
        );
        assert_eq!(
            pool(
                &linking_export(17, SharedKind::Gloves, &[]),
                PesVersion::Pes21
            ),
            []
        );
    }

    #[test]
    fn links_beside_local_boots_count_only_pre_fox() {
        // On Fox each player combines its link into its own package; pre-Fox has no package
        // of the player's own, so every shared folder takes an id.
        let export = linking_export(18, SharedKind::Boots, &["kit_boots.fmdl"]);
        assert_eq!(pool(&export, PesVersion::Pes21), []);
        assert_eq!(
            pool(&export, PesVersion::Pes17),
            [(
                "boots_id_pool_exhausted".to_owned(),
                Disposition::DropExport,
                "18".to_owned()
            )]
        );
    }

    #[test]
    fn a_plainly_linked_boots_folder_with_no_model_is_reported_naming_its_first_linking_player() {
        // Studs (textures only) is linked plainly by slots 05 and 09; Zebra holds a model and
        // Glb a glTF, which planning drops it for. Round is a face, a texture source; Combi is
        // linked beside slot 08's own boots on Fox, a texture source of his boots too.
        let files = [
            ("Players/05 - A/Studs.boots", 0),
            ("Players/06 - B/Zebra.boots", 0),
            ("Players/07 - C/Round.face", 0),
            ("Players/07 - C/Glb.gloves", 0),
            ("Players/08 - D/Combi.boots", 0),
            ("Players/08 - D/boots.fmdl", 1),
            ("Players/09 - E/Studs.boots", 0),
            ("Boots/Studs/studs.dds", 1),
            ("Boots/Zebra/boots.fmdl", 1),
            ("Boots/Zebra/boots.model", 1),
            ("Boots/Combi/combi.dds", 1),
            ("Gloves/Glb/glove_l.glb", 1),
            ("Faces/Round/round.dds", 1),
        ];
        let export = resolved("co Midcup Studs", &files, &[], None);
        let lines = |version| -> Vec<String> {
            no_model_messages(&export, version, ExportId(2), &BTreeSet::new())
                .into_iter()
                .map(line)
                .collect()
        };
        assert_eq!(
            lines(PesVersion::Pes21),
            ["Warning shared_folder_no_model [Keep] at Boots/Studs (player=Players/05 - A)"]
        );
        // Pre-Fox, slot 08's link is plain (his boots model is a part of his face), so Combi
        // is reported too, in the export's folder order.
        assert_eq!(
            lines(PesVersion::Pes17),
            [
                "Warning shared_folder_no_model [Keep] at Boots/Combi (player=Players/08 - D)",
                "Warning shared_folder_no_model [Keep] at Boots/Studs (player=Players/05 - A)",
            ]
        );
    }
}
