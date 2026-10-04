//! The `compile` command (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! validation `check` runs (the structure pass and the deep pass), run planning, each task's
//! files read in manifest order and the task processed on the worker pool, the writer thread
//! committing the batches in manifest order, and the CPK promoted from staging to the output
//! folder, or the staging discarded when writing or promoting it fails.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender, unbounded};
use pipeline::{CpkStem, MemoryBudget, Permit};
use studio_core::{Disposition, ExportId, Scope, Severity, ToolContext};

use crate::bins::WorkingBins;
use crate::cli::RunInputs;
use crate::events::RunEvents;
use crate::messages::{Code, tool_message};
use crate::output::deploy;
use crate::output::writer::CpkOutput;
use crate::plan::{BuildTask, plan_run};
use crate::processing::{CompileContext, TaskBatch, TaskFiles, process_task};
use crate::reader::{ContentSource, ExportSource, SourceFailure, SourceKind};
use crate::validation::{run_budget, run_pool, validation_pass};

/// Compiles every export validation keeps into `<output_folder>/<cpk_stem>.cpk`,
/// reported as events. Returns the worst severity reported: a CPK that cannot be written or
/// put in place is a Fatal finding, after which the previous CPK is all that is left. An
/// exports folder that cannot be read is an error.
pub(crate) fn run(
    inputs: &RunInputs,
    cpk_stem: &CpkStem,
    output_folder: &Path,
    no_deploy: bool,
    ctx: &ToolContext,
) -> anyhow::Result<Option<Severity>> {
    let version = inputs.common.pes_version;
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
            exports.push((checked.source.export_id, resolved, checked.team_colors));
        }
        sources.push(checked.source);
    }

    let report = plan_run(exports, version);
    for message in report.messages {
        events.message(message);
    }

    let tasks = report.manifest.tasks;
    let team_colors = report.manifest.team_colors;
    let mut last_task_of: BTreeMap<ExportId, usize> = BTreeMap::new();
    for (index, task) in tasks.iter().enumerate() {
        last_task_of.insert(task.export_id, index);
    }
    for source in &sources {
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
    let output = CpkOutput::new(run_folder.join(&cpk_name));
    let context = CompileContext::new(version, last_tasks.len());
    let (coordinated, (mut events, written)) = std::thread::scope(|scope| {
        let (batches_tx, batches_rx) = unbounded();
        let last_tasks = &last_tasks;
        let team_colors = &team_colors;
        let writer = scope.spawn(move || {
            let mut output = output;
            let mut events = events;
            // The writer finishes the CPK too, so its file is closed when the thread ends,
            // before a failure removes the staging folder. The bins are built on the bundled
            // bases until the installed ones are read (`pipeline.md` "Bins accumulation").
            let written = write_batches(batches_rx, &mut output, &mut events, last_tasks)
                .and_then(|()| output.finish(version, WorkingBins::bundled(), team_colors));
            (events, written)
        });
        let coordinated = pool.in_place_scope(|pool_scope| {
            coordinate(&sources, tasks, &budget, &context, &batches_tx, pool_scope)
        });
        drop(batches_tx);
        let written = writer.join().expect("the writer thread does not panic");
        (coordinated, written)
    });
    coordinated?;

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

/// The coordinator: every task's files read from its export's source, in manifest order, and
/// the task handed to `pool` with its permit, each finished batch sent to `batches`. Each
/// export's source is opened once for all its tasks, which the manifest keeps together. Fails
/// only when the run is cancelled while a task waits for its permit.
fn coordinate<'scope>(
    sources: &[ExportSource],
    tasks: Vec<BuildTask>,
    budget: &Arc<MemoryBudget>,
    context: &'scope CompileContext,
    batches: &'scope Sender<TaskBatch>,
    pool: &rayon::Scope<'scope>,
) -> anyhow::Result<()> {
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
    for source in sources {
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
                        let permit = Arc::new(budget.acquire(group.charge)?);
                        group_permit = Some(permit.clone());
                        permit
                    }
                    Some(_) => group_permit
                        .clone()
                        .expect("a group's first task acquired the shared permit"),
                    None => Arc::new(budget.acquire(task.charge)?),
                };
                let files = read_files(&task, &content);
                spawn(index, task, files, Some(permit));
            }
        }
    }
    Ok(())
}

/// The writer thread's work: each batch received is committed in manifest order, then every
/// committed batch's messages reported, and its export's `ExportProcessed` after the export's
/// last task (`last_tasks`, by manifest position). Returns the first error writing the CPK,
/// once every batch has been received.
fn write_batches(
    batches: Receiver<TaskBatch>,
    output: &mut CpkOutput,
    events: &mut RunEvents,
    last_tasks: &BTreeMap<usize, ExportId>,
) -> anyhow::Result<()> {
    for batch in &batches {
        let committed = match output.submit(batch) {
            Ok(committed) => committed,
            Err(error) => {
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
    use std::path::Path;

    use aesthetics_export::FileDescriptor;
    use pes_version::PesVersion;
    use pipeline::MemoryBudget;
    use studio_core::ExportId;
    use vtree::ScopePath;

    use super::*;
    use crate::paths::TextureHome;
    use crate::plan::subset::ModelPackage;
    use crate::plan::{ModelFolder, TaskGroup, TaskKind};
    use crate::reader::ExportSource;

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
            combined: Vec::new(),
            common_models: Vec::new(),
            common_texture_stems: BTreeSet::new(),
            textures: TextureHome::PlayerCommon {
                folder_name: "05 - The Chad Stormworks Player".to_owned(),
            },
        }
    }

    #[test]
    fn a_group_s_tasks_share_one_permit_of_the_group_s_charge_and_other_tasks_have_their_own() {
        let source = tracer_source();
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
        let tasks = vec![
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
        ];
        let budget = MemoryBudget::new(1 << 30);
        let context = CompileContext::new(PesVersion::Pes21, 1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let (batches_tx, batches_rx) = unbounded();

        pool.in_place_scope(|scope| {
            coordinate(&[source], tasks, &budget, &context, &batches_tx, scope)
        })
        .unwrap();
        drop(batches_tx);

        let mut batches: Vec<TaskBatch> = batches_rx.iter().collect();
        batches.sort_by_key(|batch| batch.index);
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
}
