//! The `compile` command (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! structure pass `check` runs, run planning, each task's files read in manifest order and the
//! task processed on the worker pool, the writer thread committing the batches in manifest
//! order, and the CPK promoted from staging to the output folder.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use crossbeam_channel::{Receiver, Sender, unbounded};
use pipeline::{CpkStem, MemoryBudget, Permit};
use studio_core::{Disposition, ExportId, Scope, Severity, ToolContext};

use crate::cli::RunInputs;
use crate::events::RunEvents;
use crate::messages::{Code, tool_message};
use crate::output::deploy;
use crate::output::writer::CpkOutput;
use crate::plan::{BuildTask, plan_run};
use crate::processing::{CompileContext, TaskBatch, TaskFiles, process_task};
use crate::reader::{ContentSource, ExportSource, SourceFailure, SourceKind};
use crate::structure::{run_budget, structure_pass};

/// Compiles every export the structure pass keeps into `<output_folder>/<cpk_stem>.cpk`,
/// reported as events. Returns the worst severity reported. An exports folder that cannot be
/// read, and a CPK that cannot be written, are errors.
pub(crate) fn run(
    inputs: &RunInputs,
    cpk_stem: &CpkStem,
    output_folder: &Path,
    no_deploy: bool,
    ctx: &ToolContext,
) -> anyhow::Result<Option<Severity>> {
    let version = inputs.common.pes_version;
    let budget = run_budget(inputs);
    let pass = structure_pass(inputs, &budget)?;
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
            exports.push((checked.source.export_id, resolved));
        }
        sources.push(checked.source);
    }

    let report = plan_run(exports, version);
    for message in report.messages {
        events.message(message);
    }

    let tasks = report.manifest.tasks;
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

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(pipeline::thread_count_detect(inputs.common.thread_count))
        .build()
        .context("cannot start the worker threads")?;
    let run_folder = deploy::staging_folder(output_folder);
    let output = CpkOutput::new(run_folder.join(deploy::cpk_file_name(cpk_stem)));
    let context = CompileContext { version };
    let (coordinated, (output, mut events, written)) = std::thread::scope(|scope| {
        let (batches_tx, batches_rx) = unbounded();
        let last_tasks = &last_tasks;
        let writer = scope.spawn(move || {
            let mut output = output;
            let mut events = events;
            let written = write_batches(batches_rx, &mut output, &mut events, last_tasks);
            (output, events, written)
        });
        let coordinated = pool.in_place_scope(|pool_scope| {
            coordinate(&sources, tasks, &budget, &context, &batches_tx, pool_scope)
        });
        drop(batches_tx);
        let written = writer.join().expect("the writer thread does not panic");
        (coordinated, written)
    });
    coordinated?;
    written?;

    if output.finish(version)? {
        let promoted = deploy::promote(&run_folder, output_folder, cpk_stem)?;
        if no_deploy {
            events.message(tool_message(
                Code::DeploySkippedByFlag,
                Scope::Run,
                Disposition::Keep,
                vec![("path", promoted.display().to_string())],
            ));
        }
    }
    Ok(events.worst())
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
        let mut content = ContentSource::new(source, budget);
        if source.kind == SourceKind::SevenZ {
            // A `.7z` holds one permit for its whole decompressed buffer, and a task asking
            // for its own while that is held waits forever when the archive is over the cap.
            // So every task is read first, the buffer freed, and the tasks share its permit.
            let mut read = Vec::new();
            while let Some((index, task)) =
                tasks.next_if(|(_, task)| task.export_id == source.export_id)
            {
                let files = read_files(&task, &mut content);
                read.push((index, task, files));
            }
            let permit = content.into_permit().map(Arc::new);
            for (index, task, files) in read {
                spawn(index, task, files, permit.clone());
            }
        } else {
            while let Some((index, task)) =
                tasks.next_if(|(_, task)| task.export_id == source.export_id)
            {
                let permit = Arc::new(budget.acquire(task.charge)?);
                let files = read_files(&task, &mut content);
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
            uniparam: None,
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
fn read_files(task: &BuildTask, content: &mut ContentSource) -> Result<TaskFiles, SourceFailure> {
    let mut files = TaskFiles::new();
    for file in task.kind.files() {
        let bytes = content.read(file.source.as_str())?;
        files.insert(file.path.clone(), bytes);
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use aesthetics_export::{FileDescriptor, PlayerFolder};
    use pes_version::PesVersion;
    use pipeline::MemoryBudget;
    use studio_core::ExportId;
    use vtree::ScopePath;

    use super::*;
    use crate::plan::TaskKind;
    use crate::reader::ExportSource;

    const PLAYER: &str = "Players/05 - The Chad Stormworks Player";

    #[test]
    fn a_file_that_cannot_be_read_fails_its_task_naming_it() {
        let tracer =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tracer/studio/egg Tracer");
        let source = ExportSource {
            export_id: ExportId(4),
            path: tracer.clone(),
            kind: SourceKind::Folder,
            file_name: "egg Tracer".to_owned(),
            display_name: "egg Tracer".to_owned(),
            team_name: None,
        };
        // The tracer's player folder holds no `face_high.fmdl`.
        let missing = ScopePath::new(&format!("{PLAYER}/face_high.fmdl")).unwrap();
        let folder = PlayerFolder {
            path: ScopePath::new(PLAYER).unwrap(),
            player_name: "The Chad Stormworks Player".to_owned(),
            files: vec![FileDescriptor {
                size: 0,
                kind: aesthetics_export::classify(missing.name()),
                source: missing.clone(),
                path: missing,
            }],
            links: Vec::new(),
            ingame_face: false,
            fpc: None,
            portrait: None,
            settings: None,
        };
        let task = BuildTask {
            export_id: ExportId(4),
            team_id: 792,
            kind: TaskKind::Face {
                folder,
                player_ids: vec![79205],
            },
            charge: 0,
        };

        let files = read_files(
            &task,
            &mut ContentSource::new(&source, &MemoryBudget::new(1 << 30)),
        );
        let batch = task_batch(
            3,
            task,
            files,
            &CompileContext {
                version: PesVersion::Pes21,
            },
        );

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
