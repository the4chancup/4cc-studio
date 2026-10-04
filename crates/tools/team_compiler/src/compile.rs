//! The `compile` command (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! validation `check` runs (the structure pass and the deep pass), run planning, each task's
//! files read in manifest order and the task processed on the worker pool, the writer thread
//! committing the batches in manifest order, and the CPK promoted from staging to the output
//! folder, or the staging discarded when writing or promoting it fails, or when an export's
//! file changes while it is read (`pipeline.md` "Resolved decisions", "Source snapshot").

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender, unbounded};
use pes_version::PesVersion;
use pipeline::{Cancelled, CpkStem, MemoryBudget, Permit};
use studio_core::{Disposition, ExportId, Scope, Severity, ToolContext};

use crate::bins::WorkingBins;
use crate::cli::RunInputs;
use crate::events::RunEvents;
use crate::messages::{Code, tool_message};
use crate::output::writer::CpkOutput;
use crate::output::{deploy, teamnotes};
use crate::plan::{BuildManifest, BuildTask, overrides, plan_run};
use crate::processing::{CompileContext, TaskBatch, TaskFiles, process_task};
use crate::reader::{ContentSource, ExportSource, SourceFailure, SourceKind, SourceRevision};
use crate::validation::{run_budget, run_pool, validation_pass};

/// Compiles every export validation keeps into `<output_folder>/<cpk_stem>.cpk`, after the
/// files of the data directory's `overrides/` folder, reported as events, then collects the
/// compiled exports' notes into `<output_folder>/teamnotes.txt`. Returns the worst severity
/// reported: a CPK that cannot be written or put in place, or an export file that changes
/// while the run reads it, is a Fatal finding, after which the previous CPK is all that is
/// left. An exports folder or an `overrides/` folder that cannot be read is an error.
pub(crate) fn run(
    inputs: &RunInputs,
    cpk_stem: &CpkStem,
    output_folder: &Path,
    no_deploy: bool,
    ctx: &ToolContext,
) -> anyhow::Result<Option<Severity>> {
    let planned = plan(inputs, ctx)?;
    build(planned, cpk_stem, output_folder, no_deploy)
}

/// A run validated and planned, which `build` compiles: its own function so a test can change
/// an export's files between planning and building.
struct PlannedRun {
    version: PesVersion,
    /// The events so far: validation's and planning's findings.
    events: RunEvents,
    budget: Arc<MemoryBudget>,
    pool: rayon::ThreadPool,
    /// The `overrides/` folder's files, by CPK path.
    overrides: BTreeMap<String, PathBuf>,
    /// Every source in export order, with its revision when it was validated.
    sources: Vec<(ExportSource, Option<SourceRevision>)>,
    manifest: BuildManifest,
}

/// `compile`'s first half: the `overrides/` folder listed, the validation pass, each export's
/// findings reported, and the run planned.
fn plan(inputs: &RunInputs, ctx: &ToolContext) -> anyhow::Result<PlannedRun> {
    let version = inputs.common.pes_version;
    // Listed before any export is read, so a tree that cannot be listed stops the run first.
    let (overrides, overrides_active) = overrides::list(ctx.paths().data_dir.as_deref())?;
    let budget = run_budget(inputs);
    let pool = run_pool(inputs)?;
    let pass = validation_pass(inputs, &budget, &pool)?;
    let mut events = RunEvents::new(ctx);
    for message in pass.run_messages {
        events.message(message);
    }
    let mut sources = Vec::new();
    let mut exports = Vec::new();
    for checked in pass.sources {
        events.started(&checked.source);
        for message in checked.messages {
            events.message(message);
        }
        if let Some(resolved) = checked.resolved {
            exports.push((
                checked.source.export_id,
                resolved,
                checked.team_colors,
                checked.notes,
            ));
        }
        sources.push((checked.source, checked.revision));
    }

    let report = plan_run(exports, version);
    for message in report.messages.into_iter().chain(overrides_active) {
        events.message(message);
    }
    Ok(PlannedRun {
        version,
        events,
        budget,
        pool,
        overrides,
        sources,
        manifest: report.manifest,
    })
}

/// `compile`'s second half: the planned tasks read, processed and written into the staged CPK,
/// which is then promoted, and `teamnotes.txt` written. A source that changed while its tasks
/// were read aborts the run with `source_changed_during_run`, the staging discarded.
fn build(
    planned: PlannedRun,
    cpk_stem: &CpkStem,
    output_folder: &Path,
    no_deploy: bool,
) -> anyhow::Result<Option<Severity>> {
    let PlannedRun {
        version,
        events,
        budget,
        pool,
        overrides,
        sources,
        manifest,
    } = planned;
    let tasks = manifest.tasks;
    let team_colors = manifest.team_colors;
    let notes = manifest.notes;
    let mut last_task_of: BTreeMap<ExportId, usize> = BTreeMap::new();
    for (index, task) in tasks.iter().enumerate() {
        last_task_of.insert(task.export_id, index);
    }
    for (source, _) in &sources {
        if !last_task_of.contains_key(&source.export_id) {
            events.processed(source.export_id);
        }
    }
    let last_tasks: BTreeMap<usize, ExportId> = last_task_of
        .into_iter()
        .map(|(export_id, index)| (index, export_id))
        .collect();

    let run_folder = deploy::staging_folder(output_folder);
    let cpk_name = deploy::cpk_file_name(cpk_stem);
    let output = CpkOutput::new(run_folder.join(&cpk_name), overrides);
    let context = CompileContext::new(version, last_tasks.len());
    let (coordinated, (mut events, written)) = std::thread::scope(|scope| {
        let (batches_tx, batches_rx) = unbounded();
        let last_tasks = &last_tasks;
        let team_colors = &team_colors;
        let budget = &budget;
        let writer = scope.spawn(move || {
            let mut output = output;
            let mut events = events;
            // The writer finishes the CPK too, so its file is closed when the thread ends,
            // before a failure removes the staging folder. The bins are built on the bundled
            // bases until the installed ones are read (`pipeline.md` "Bins accumulation").
            let written = write_batches(batches_rx, &mut output, &mut events, last_tasks, budget)
                .and_then(|()| output.finish(version, WorkingBins::bundled(), team_colors));
            (events, written)
        });
        let coordinated = pool.in_place_scope(|pool_scope| {
            coordinate(&sources, tasks, budget, &context, &batches_tx, pool_scope)
        });
        drop(batches_tx);
        let written = writer.join().expect("the writer thread does not panic");
        (coordinated, written)
    });
    if let Some(change) = coordinated {
        // The staged CPK holds only the batches spawned before the change, and its writer may
        // have failed on the ones that never came: the change is reported alone, never also
        // as a write failure, and the staging goes either way.
        if let Err(error) = written {
            log::debug!("the aborted run's staged CPK: {error:#}");
        }
        deploy::discard(&run_folder, output_folder);
        events.message(tool_message(
            Code::SourceChangedDuringRun,
            Scope::Export {
                export_id: change.export_id,
            },
            Disposition::AbortRun,
            vec![("path", change.path)],
        ));
        return Ok(events.worst());
    }

    let cpk_path = output_folder.join(&cpk_name);
    // A bins failure is a CPK write failure too: `uniparam_compile_failed` arrives with Phase
    // 4's installed-bin lookup; in Phase 3 the only bin is built on the bundled base.
    let written = match written {
        Ok((written, messages)) => {
            for message in messages {
                events.message(message);
            }
            written
        }
        Err(error) => {
            abort_output(
                &mut events,
                &run_folder,
                output_folder,
                &cpk_path,
                Code::CpkWriteFailed,
                error,
            );
            return Ok(events.worst());
        }
    };
    if !written {
        return Ok(events.worst());
    }
    match deploy::promote(&run_folder, output_folder, cpk_stem) {
        Ok(promoted) => {
            if no_deploy {
                events.message(tool_message(
                    Code::DeploySkippedByFlag,
                    Scope::Run,
                    Disposition::Keep,
                    vec![("path", promoted.display().to_string())],
                ));
            }
            write_teamnotes(&mut events, output_folder, &notes);
        }
        Err(error) => abort_output(
            &mut events,
            &run_folder,
            output_folder,
            &cpk_path,
            Code::OutputCommitFailed,
            error,
        ),
    }
    Ok(events.worst())
}

/// Writes `notes` (team name, note text) as `<output_folder>/teamnotes.txt`, or removes a
/// previous one when there is no note: a file left over would show the notes of exports this
/// run did not compile. Called once the CPK is in place, so a run that writes none leaves the
/// previous file. A failure is `teamnotes_write_failed`, an Error on the run naming the file
/// and the whole error chain; the CPK stays.
fn write_teamnotes(events: &mut RunEvents, output_folder: &Path, notes: &[(String, String)]) {
    let path = output_folder.join(teamnotes::FILE_NAME);
    if let Err(error) = teamnotes::write(&path, teamnotes::render(notes).as_deref()) {
        events.message(tool_message(
            Code::TeamnotesWriteFailed,
            Scope::Run,
            Disposition::Keep,
            vec![
                ("path", path.display().to_string()),
                ("error", format!("{error:#}")),
            ],
        ));
    }
}

/// A failure writing the CPK or putting it in place: the run's staging is discarded, so the
/// previous CPK at `cpk_path` is all that is left, and the failure is reported as `code`,
/// Fatal on the run, naming that path and the whole error chain.
fn abort_output(
    events: &mut RunEvents,
    run_folder: &Path,
    output_folder: &Path,
    cpk_path: &Path,
    code: Code,
    error: anyhow::Error,
) {
    deploy::discard(run_folder, output_folder);
    events.message(tool_message(
        code,
        Scope::Run,
        Disposition::AbortRun,
        vec![
            ("path", cpk_path.display().to_string()),
            ("error", format!("{error:#}")),
        ],
    ));
}

/// A file of an export that changed while the run read it: `source_changed_during_run`.
#[derive(Debug, PartialEq, Eq)]
struct SourceChange {
    /// The export whose file changed.
    export_id: ExportId,
    /// The file, or the archive, as the system displays its path.
    path: String,
}

/// The coordinator: every task's files read from its export's source, in manifest order, and
/// the task handed to `pool` with its permit, each finished batch sent to `batches`. Each
/// export's source is opened once for all its tasks, which the manifest keeps together. After
/// each read the files read are checked against the source's revision; on a change no further
/// task is read or spawned, the spawned ones finish, and the change is returned. A cancelled
/// budget stops the coordinator before the next source and at a waiting acquire, with no
/// change to report: the cancellation's only trigger is the writer's `cpk_write_failed`,
/// which the writer reports itself.
fn coordinate<'scope>(
    sources: &[(ExportSource, Option<SourceRevision>)],
    tasks: Vec<BuildTask>,
    budget: &Arc<MemoryBudget>,
    context: &'scope CompileContext,
    batches: &'scope Sender<TaskBatch>,
    pool: &rayon::Scope<'scope>,
) -> Option<SourceChange> {
    let spawn = move |index: usize,
                      task: BuildTask,
                      files: Result<TaskFiles, SourceFailure>,
                      permit: Option<Arc<Permit>>| {
        pool.spawn(move |_| {
            let mut batch = task_batch(index, task, files, context);
            batch.permit = permit;
            batches
                .send(batch)
                .expect("the writer receives until every sender is gone");
        });
    };
    // The coordinator reads, not the task: an archive is one sequential stream, so tasks
    // sharing it would only wait on each other, and processing needs no source handle.
    let mut tasks = tasks.into_iter().enumerate().peekable();
    for (source, revision) in sources {
        // A `.7z`'s acquire happens inside its first `read`, which would turn `Cancelled`
        // into a failed read rather than stopping the run; check before opening anything.
        if budget.is_cancelled() {
            return None;
        }
        let content = ContentSource::new(source, budget);
        if source.kind == SourceKind::SevenZ {
            // A `.7z` holds one permit for its whole decompressed buffer, and a task asking
            // for its own while that is held waits forever when the archive is over the cap.
            // So every task is read first, the buffer freed, and the tasks share its permit.
            let mut read = Vec::new();
            while let Some((index, task)) =
                tasks.next_if(|(_, task)| task.export_id == source.export_id)
            {
                let files = read_files(&task, &content);
                read.push((index, task, files));
            }
            // Once, after the one read that decompressed the archive for every task; the
            // archive's stamp is checked whatever the files.
            if !read.is_empty()
                && let Some(change) = source_change(source, revision.as_ref(), [])
            {
                return Some(change);
            }
            let permit = content.into_permit().map(Arc::new);
            for (index, task, files) in read {
                spawn(index, task, files, permit.clone());
            }
        } else {
            // A player folder's group is charged once, at its first task, and its tasks share
            // the permit: the writer holds the group's packages until its textures batch, so
            // a textures task waiting for a permit of its own would wait for memory the held
            // packages never release.
            let mut group_permit: Option<Arc<Permit>> = None;
            while let Some((index, task)) =
                tasks.next_if(|(_, task)| task.export_id == source.export_id)
            {
                let permit = match &task.group {
                    Some(group) if index == group.tasks.start => {
                        // The previous group's tasks are all spawned (groups
                        // are contiguous), so the coordinator's share goes
                        // before the wait: only its batches still hold it.
                        drop(group_permit.take());
                        match budget.acquire(group.charge) {
                            Ok(held) => {
                                let permit = Arc::new(held);
                                group_permit = Some(permit.clone());
                                permit
                            }
                            Err(Cancelled) => return None,
                        }
                    }
                    Some(_) => group_permit
                        .clone()
                        .expect("a group's first task acquired the shared permit"),
                    None => {
                        // The same drop for an ungrouped task's acquire: the
                        // group before it, when there was one, is done.
                        drop(group_permit.take());
                        match budget.acquire(task.charge) {
                            Ok(held) => Arc::new(held),
                            Err(Cancelled) => return None,
                        }
                    }
                };
                let files = read_files(&task, &content);
                // After the read, not before it: a file saved over while it was being read is
                // caught too. A file gone before the read is a change, not a failed read.
                let read = task.kind.files();
                let read = read.iter().map(|file| file.source.as_str());
                if let Some(change) = source_change(source, revision.as_ref(), read) {
                    return Some(change);
                }
                spawn(index, task, files, Some(permit));
            }
        }
    }
    None
}

/// The change `source`'s `revision` sees among `files`, the paths in the source a task read;
/// an archive's own stamp is checked whatever they are.
fn source_change<'a>(
    source: &ExportSource,
    revision: Option<&SourceRevision>,
    files: impl IntoIterator<Item = &'a str>,
) -> Option<SourceChange> {
    let revision = revision.expect("a source with a task was validated, so it was listed");
    let path = revision.changed(source, files)?;
    Some(SourceChange {
        export_id: source.export_id,
        path,
    })
}

/// The writer thread's work: each batch received is committed in manifest order, then every
/// committed batch's messages reported, and its export's `ExportProcessed` after the export's
/// last task (`last_tasks`, by manifest position). Returns the first error writing the CPK,
/// once every batch has been received. A failed `submit` cancels the budget, so the
/// coordinator stops reading and the pool stops processing the rest of the run: the failure
/// this function returns is the finding the run reports.
fn write_batches(
    batches: Receiver<TaskBatch>,
    output: &mut CpkOutput,
    events: &mut RunEvents,
    last_tasks: &BTreeMap<usize, ExportId>,
    budget: &Arc<MemoryBudget>,
) -> anyhow::Result<()> {
    for batch in &batches {
        let committed = match output.submit(batch) {
            Ok(committed) => committed,
            Err(error) => {
                budget.cancel();
                // Keep receiving until the channel closes: a pool task's send must never
                // find it closed, and the coordinator may be waiting for a permit that only
                // dropping a later batch frees.
                for unwritten in &batches {
                    drop(unwritten);
                }
                return Err(error);
            }
        };
        for (index, messages) in committed {
            for message in messages {
                events.message(message);
            }
            if let Some(export_id) = last_tasks.get(&index) {
                events.processed(*export_id);
            }
        }
    }
    Ok(())
}

/// The batch of `task`, the manifest's task number `index`, processed over `files`, the
/// coordinator's read of it. A file that could not be read fails the task with
/// `source_read_failed` naming it, on the task's folder, and the task is not processed.
fn task_batch(
    index: usize,
    task: BuildTask,
    files: Result<TaskFiles, SourceFailure>,
    ctx: &CompileContext,
) -> TaskBatch {
    match files {
        Ok(files) => process_task(index, task, files, ctx),
        Err(failure) => TaskBatch {
            index,
            entries: Vec::new(),
            group: task.group.as_ref().map(|group| group.tasks.clone()),
            skipped: Vec::new(),
            uniparam: None,
            uni_color: None,
            messages: vec![tool_message(
                Code::SourceReadFailed,
                Scope::Folder {
                    export_id: task.export_id,
                    path: task.kind.folder_path(),
                },
                Disposition::DropFolder,
                vec![("path", failure.path), ("error", failure.error)],
            )],
            permit: None,
        },
    }
}

/// The bytes of every file `task` reads, from `content`; the first file that cannot be read
/// stops the reading.
fn read_files(task: &BuildTask, content: &ContentSource) -> Result<TaskFiles, SourceFailure> {
    let mut files = TaskFiles::new();
    for file in task.kind.files() {
        let bytes = content.read(file.source.as_str())?;
        files.insert(file.path.clone(), bytes);
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs::{self, File};
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    use aesthetics_export::FileDescriptor;
    use pes_version::PesVersion;
    use pipeline::MemoryBudget;
    use studio_core::{ExportId, Message, PipelineEvent};
    use teams_list::TeamsList;
    use vtree::ScopePath;

    use super::*;
    use crate::paths::TextureHome;
    use crate::plan::subset::ModelPackage;
    use crate::plan::{ModelFolder, TaskGroup, TaskKind};
    use crate::reader::{ExportSource, Route};
    use crate::settings::TeamCompilerSettings;
    use crate::testing::{sandbox, scratch, tool_context};

    const PLAYER: &str = "Players/05 - The Chad Stormworks Player";

    /// The tracer bullet's export folder, as export 4.
    fn tracer_source() -> ExportSource {
        ExportSource {
            export_id: ExportId(4),
            path: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Tracer"),
            kind: SourceKind::Folder,
            file_name: "egg Tracer".to_owned(),
            display_name: "egg Tracer".to_owned(),
            team_name: None,
        }
    }

    /// The tracer's player folder holding the files `names`, sizes unknown.
    fn player(names: &[&str]) -> ModelFolder {
        ModelFolder {
            path: ScopePath::new(PLAYER).unwrap(),
            files: names
                .iter()
                .map(|name| {
                    let path = ScopePath::new(&format!("{PLAYER}/{name}")).unwrap();
                    FileDescriptor {
                        size: 0,
                        kind: aesthetics_export::classify(path.name()),
                        source: path.clone(),
                        path,
                    }
                })
                .collect(),
            ingame_face: false,
            combined: Vec::new(),
            common_models: Vec::new(),
            common_texture_stems: BTreeSet::new(),
            textures: TextureHome::PlayerCommon {
                folder_name: "05 - The Chad Stormworks Player".to_owned(),
            },
        }
    }

    /// `source` with the revision its listing gives now, as validation hands it on.
    fn listed(source: ExportSource) -> (ExportSource, Option<SourceRevision>) {
        let route = crate::reader::route(std::slice::from_ref(&source)).remove(0);
        let Route::Validate { revision, .. } = route else {
            panic!("{route:?}");
        };
        (source, Some(revision))
    }

    /// Three tasks of the tracer's player 05, as export 4: its face package and its textures
    /// (`shirt.dds`) as one group of charge 30, then a portrait (`shirt.dds` again) of charge 7.
    fn tracer_tasks() -> Vec<BuildTask> {
        let folder = player(&[
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair.skl",
            "fcl_hair_sim.fclo",
            "shirt.dds",
        ]);
        let group = TaskGroup {
            tasks: 0..2,
            packages: vec![ModelPackage::Face],
            charge: 30,
        };
        let task = |kind, charge, group| BuildTask {
            export_id: ExportId(4),
            team_id: 792,
            kind,
            charge,
            group,
        };
        vec![
            task(
                TaskKind::Models {
                    folder: folder.clone(),
                    package: ModelPackage::Face,
                    ids: vec![79205],
                },
                10,
                Some(group.clone()),
            ),
            task(
                TaskKind::Textures {
                    folder: folder.clone(),
                    kits: Vec::new(),
                },
                20,
                Some(group),
            ),
            // The tracer's `shirt.dds` stands in for a portrait: a DDS that passes through.
            task(
                TaskKind::Portrait {
                    player_id: 79205,
                    file: folder
                        .files
                        .iter()
                        .find(|file| file.path.name() == "shirt.dds")
                        .unwrap()
                        .clone(),
                },
                7,
                None,
            ),
        ]
    }

    /// `coordinate` over the tracer's source and `tasks` on a spawned thread with a budget of
    /// `cap` bytes, the batches dropped as they arrive (the writer's role, so permits free
    /// mid-run): the change it returned, or `None` when it did not come back within the guard.
    fn coordinated_with_cap(tasks: Vec<BuildTask>, cap: usize) -> Option<Option<SourceChange>> {
        let (done_tx, done) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let budget = MemoryBudget::new(cap);
            let sources = [listed(tracer_source())];
            let context = CompileContext::new(PesVersion::Pes21, 1);
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .build()
                .unwrap();
            let (batches_tx, batches_rx) = unbounded();
            thread::spawn(move || {
                for batch in &batches_rx {
                    drop(batch);
                }
            });
            let change = pool.in_place_scope(|scope| {
                coordinate(&sources, tasks, &budget, &context, &batches_tx, scope)
            });
            drop(batches_tx);
            done_tx.send(change).unwrap();
        });
        done.recv_timeout(Duration::from_secs(30)).ok()
    }

    /// The coordinator over `sources` and `tasks` on a pool of two threads: what it returned,
    /// and every batch it sent, in manifest order.
    fn coordinated(
        sources: &[(ExportSource, Option<SourceRevision>)],
        tasks: Vec<BuildTask>,
    ) -> (Option<SourceChange>, Vec<TaskBatch>) {
        let budget = MemoryBudget::new(1 << 30);
        let context = CompileContext::new(PesVersion::Pes21, 1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let (batches_tx, batches_rx) = unbounded();

        let change = pool.in_place_scope(|scope| {
            coordinate(sources, tasks, &budget, &context, &batches_tx, scope)
        });
        drop(batches_tx);

        let mut batches: Vec<TaskBatch> = batches_rx.iter().collect();
        batches.sort_by_key(|batch| batch.index);
        (change, batches)
    }

    #[test]
    fn a_group_s_tasks_share_one_permit_of_the_group_s_charge_and_other_tasks_have_their_own() {
        let (change, batches) = coordinated(&[listed(tracer_source())], tracer_tasks());

        assert_eq!(change, None);
        let permits: Vec<&Arc<Permit>> = batches
            .iter()
            .map(|batch| batch.permit.as_ref().expect("every task is charged"))
            .collect();
        assert_eq!(permits.len(), 3);
        assert!(
            Arc::ptr_eq(permits[0], permits[1]),
            "the group's two tasks hold the one permit"
        );
        assert_eq!(permits[0].size(), 30, "the group's whole charge, once");
        assert!(!Arc::ptr_eq(permits[0], permits[2]));
        assert_eq!(permits[2].size(), 7, "the ungrouped task's own charge");
    }

    /// The tracer's tasks reworked: `groups` as (tasks range, charge) pairs, `ungrouped`
    /// charges as single tasks after them, every task's files the tracer's five.
    fn grouped_tasks(groups: &[(usize, usize, usize)], ungrouped: &[usize]) -> Vec<BuildTask> {
        let folder = player(&[
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair.skl",
            "fcl_hair_sim.fclo",
            "shirt.dds",
        ]);
        let kind = || TaskKind::Models {
            folder: folder.clone(),
            package: ModelPackage::Face,
            ids: vec![79205],
        };
        let task = |group| BuildTask {
            export_id: ExportId(4),
            team_id: 792,
            kind: kind(),
            charge: 0,
            group,
        };
        let mut tasks = Vec::new();
        for (start, end, charge) in groups {
            let group = TaskGroup {
                tasks: *start..*end,
                packages: vec![ModelPackage::Face],
                charge: *charge,
            };
            for _ in *start..*end {
                tasks.push(task(Some(group.clone())));
            }
        }
        for charge in ungrouped {
            tasks.push(BuildTask {
                charge: *charge,
                ..task(None)
            });
        }
        tasks
    }

    #[test]
    fn an_oversized_group_lets_the_next_acquire_once_its_batches_are_free() {
        // A group over the cap admits alone; the ungrouped task after it
        // acquires once the group's batches are gone — the coordinator's own
        // share must not hold the group charged.
        let tasks = grouped_tasks(&[(0, 2, 50)], &[5]);
        assert!(
            coordinated_with_cap(tasks, 30).is_some(),
            "the coordinator never returned"
        );
    }

    #[test]
    fn two_groups_over_the_cap_together_do_not_hang_the_coordinator() {
        // Each group fits alone; together they exceed the cap, so the second
        // acquires only after the first's batches are dropped — again only if
        // the coordinator released its own share first.
        let tasks = grouped_tasks(&[(0, 2, 30), (2, 4, 30)], &[]);
        assert!(
            coordinated_with_cap(tasks, 40).is_some(),
            "the coordinator never returned"
        );
    }

    #[test]
    fn a_cancelled_budget_stops_the_coordinator_before_any_task_is_spawned() {
        let budget = MemoryBudget::new(1 << 30);
        budget.cancel();
        let context = CompileContext::new(PesVersion::Pes21, 1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let (batches_tx, batches_rx) = unbounded::<TaskBatch>();
        let sources = [listed(tracer_source())];

        let change = pool.in_place_scope(|scope| {
            coordinate(
                &sources,
                grouped_tasks(&[], &[10, 20, 30]),
                &budget,
                &context,
                &batches_tx,
                scope,
            )
        });
        drop(batches_tx);

        assert_eq!(change, None);
        assert!(
            batches_rx.try_iter().next().is_none(),
            "a task was spawned on a cancelled budget"
        );
    }

    #[test]
    fn a_cancelled_budget_stops_the_coordinator_before_a_7z_source() {
        // A `.7z` source never calls `acquire` itself: its first `read` does, inside
        // `ContentSource`, which reports a cancelled one as a failed read rather than
        // stopping the run. The cancelled check must come before the source is opened.
        let budget = MemoryBudget::new(1 << 30);
        budget.cancel();
        let context = CompileContext::new(PesVersion::Pes21, 1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let (batches_tx, batches_rx) = unbounded::<TaskBatch>();
        let source = ExportSource {
            path: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/sources/egg Tracer.7z"),
            kind: SourceKind::SevenZ,
            file_name: "egg Tracer.7z".to_owned(),
            ..tracer_source()
        };
        let sources = [listed(source)];

        let change = pool.in_place_scope(|scope| {
            coordinate(
                &sources,
                grouped_tasks(&[], &[10, 20, 30]),
                &budget,
                &context,
                &batches_tx,
                scope,
            )
        });
        drop(batches_tx);

        assert_eq!(change, None);
        assert!(
            batches_rx.try_iter().next().is_none(),
            "a task was spawned on a cancelled budget"
        );
    }

    #[test]
    fn a_failed_cpk_write_cancels_the_budget_so_no_task_waits_on_it() {
        let temp = scratch("write_batches_cancel");
        let root = temp.path();
        // A file where the CPK's folder goes: it cannot be created in it.
        fs::write(root.join("blocker"), "").unwrap();
        let mut output = CpkOutput::new(root.join("blocker").join("cup.cpk"), BTreeMap::new());
        let budget = MemoryBudget::new(1 << 30);
        let (batches_tx, batches_rx) = unbounded();
        batches_tx
            .send(TaskBatch {
                index: 0,
                entries: vec![("common/etc/TeamColor.bin".to_owned(), b"bin".to_vec())],
                group: None,
                skipped: Vec::new(),
                uniparam: None,
                uni_color: None,
                messages: Vec::new(),
                permit: None,
            })
            .unwrap();
        drop(batches_tx);
        let mut events = RunEvents::new(&tool_context(root, ""));

        assert!(
            write_batches(
                batches_rx,
                &mut output,
                &mut events,
                &BTreeMap::new(),
                &budget
            )
            .is_err(),
            "the CPK could not be created"
        );
        assert!(
            matches!(budget.acquire(1), Err(Cancelled)),
            "the budget is cancelled"
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_fails_its_task_naming_it() {
        let source = tracer_source();
        let tracer = source.path.clone();
        // The tracer's player folder holds no `face_high.fmdl`.
        let folder = player(&["face_high.fmdl"]);
        let task = BuildTask {
            export_id: ExportId(4),
            team_id: 792,
            kind: TaskKind::Models {
                folder,
                package: ModelPackage::Face,
                ids: vec![79205],
            },
            charge: 0,
            group: None,
        };

        let files = read_files(
            &task,
            &ContentSource::new(&source, &MemoryBudget::new(1 << 30)),
        );
        let batch = task_batch(3, task, files, &CompileContext::new(PesVersion::Pes21, 1));

        assert_eq!(batch.index, 3);
        assert!(batch.entries.is_empty() && batch.uniparam.is_none());
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(message.code.code, "source_read_failed");
        assert_eq!(
            (message.severity, message.disposition),
            (Severity::Error, Disposition::DropFolder)
        );
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(4),
                path: ScopePath::new(PLAYER).unwrap(),
            }
        );
        let path = tracer.join(format!("{PLAYER}/face_high.fmdl"));
        assert_eq!(
            message.context[0],
            ("path".to_owned(), path.display().to_string())
        );
        assert_eq!(message.context[1].0, "error");
    }

    /// How far a test moves a modified time: far past any file system's time granularity.
    const MOVED: Duration = Duration::from_secs(10);

    fn move_modified_time(path: &Path) {
        let listed = fs::metadata(path).unwrap().modified().unwrap();
        File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(listed + MOVED)
            .unwrap();
    }

    #[test]
    fn a_file_rewritten_after_the_listing_stops_the_coordinator_at_the_task_that_reads_it() {
        let temp = sandbox("coordinate_folder_changed");
        let source = ExportSource {
            path: temp.path().join("exports").join("egg Tracer"),
            ..tracer_source()
        };
        let shirt = source.path.join(format!("{PLAYER}/shirt.dds"));
        let sources = [listed(source)];
        // Read by the second task, the group's textures, and by the third, the portrait.
        fs::write(&shirt, b"saved over").unwrap();

        let (change, batches) = coordinated(&sources, tracer_tasks());

        let indices: Vec<usize> = batches.iter().map(|batch| batch.index).collect();
        assert_eq!(indices, [0], "only the first task's batch is sent");
        assert_eq!(
            change,
            Some(SourceChange {
                export_id: ExportId(4),
                path: shirt.display().to_string(),
            })
        );
    }

    #[test]
    fn an_archive_whose_modified_time_moved_after_the_listing_sends_no_batch() {
        for name in ["egg Tracer.zip", "egg Tracer.7z"] {
            let temp = scratch("coordinate_archive_changed");
            let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/sources")
                .join(name);
            let kind = if name.ends_with(".7z") {
                SourceKind::SevenZ
            } else {
                SourceKind::Zip
            };
            let source = ExportSource {
                path: temp.path().join(name),
                kind,
                file_name: name.to_owned(),
                ..tracer_source()
            };
            fs::copy(&fixture, &source.path).unwrap();
            let archive = source.path.display().to_string();
            let sources = [listed(source)];
            move_modified_time(&sources[0].0.path);

            let (change, batches) = coordinated(&sources, tracer_tasks());

            assert!(batches.is_empty(), "{name}: {} batches", batches.len());
            assert_eq!(
                change,
                Some(SourceChange {
                    export_id: ExportId(4),
                    path: archive,
                }),
                "{name}"
            );
        }
    }

    /// The run inputs `compile` resolves for a `sandbox` with no arguments, over `ctx`'s
    /// settings.
    fn sandbox_inputs(root: &Path, ctx: &ToolContext) -> RunInputs {
        RunInputs {
            settings: TeamCompilerSettings::default(),
            common: ctx.common(),
            teams_list: TeamsList::parse("ID\tName\n792\t/egg/\n").unwrap(),
            exports_root: root.join("exports"),
            exports: Vec::new(),
        }
    }

    // TC-PLN-06
    #[test]
    fn a_file_replaced_after_planning_aborts_the_run_and_keeps_the_previous_cpk() {
        // The boots are read before any batch is committed, so nothing is staged yet; the kit
        // is read after the player's group and the portrait are, so a staged CPK exists and
        // must be discarded.
        for (scratch_name, replaced) in [
            ("compile_boots_changed", format!("{PLAYER}/boots.fmdl")),
            ("compile_kit_changed", "Kits/g1/kit.dds".to_owned()),
        ] {
            let temp = sandbox(scratch_name);
            let root = temp.path();
            let output = root.join("output");
            deploy::prepare_output_folder(&output).unwrap();
            let stem = CpkStem::new("4cc_99_test").unwrap();
            let cpk = output.join("4cc_99_test.cpk");
            let ctx = tool_context(root, "");
            let inputs = sandbox_inputs(root, &ctx);
            run(&inputs, &stem, &output, false, &ctx).unwrap();
            let previous = fs::read(&cpk).unwrap();

            let (events_tx, events) = unbounded();
            let ctx = ctx.with_events(events_tx);
            let planned = plan(&inputs, &ctx).unwrap();
            let file = root.join("exports").join("egg Tracer").join(&replaced);
            fs::write(&file, b"saved over from Blender").unwrap();
            let worst = build(planned, &stem, &output, false).unwrap();

            // Fatal is exit code 3: `cli.rs`'s `the_worst_severity_decides_the_exit_code`.
            assert_eq!(worst, Some(Severity::Fatal), "{replaced}");
            let fatal: Vec<Message> = events
                .try_iter()
                .filter_map(|envelope| {
                    let PipelineEvent::Message(message) = envelope.event else {
                        return None;
                    };
                    (message.severity == Severity::Fatal).then_some(message)
                })
                .collect();
            let [message] = fatal.as_slice() else {
                panic!("{replaced}: {fatal:?}");
            };
            assert_eq!(message.code.code, "source_changed_during_run");
            assert_eq!(message.disposition, Disposition::AbortRun);
            assert_eq!(
                message.scope,
                Scope::Export {
                    export_id: ExportId(0)
                }
            );
            assert_eq!(
                message.context,
                [("path".to_owned(), file.display().to_string())]
            );
            assert!(
                fs::read(&cpk).unwrap() == previous,
                "{replaced}: the previous CPK is kept"
            );
            assert!(!output.join(".staging").exists(), "{replaced}");
        }
    }
}
