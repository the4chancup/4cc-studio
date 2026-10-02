//! The `compile` command (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! structure pass `check` runs, run planning, each task's files read and the task processed and
//! handed to the writer in manifest order, and the CPK promoted from staging to the output
//! folder.

use std::path::Path;

use pipeline::CpkStem;
use studio_core::{Disposition, Scope, Severity, ToolContext};

use crate::cli::RunInputs;
use crate::events::RunEvents;
use crate::messages::{Code, tool_message};
use crate::output::deploy;
use crate::output::writer::CpkOutput;
use crate::plan::{BuildTask, plan_run};
use crate::processing::{CompileContext, TaskBatch, TaskFiles, process_task};
use crate::reader::{ContentSource, SourceFailure, SourceKind};
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

    let run_folder = deploy::staging_folder(output_folder);
    let mut writer = CpkOutput::new(run_folder.join(deploy::cpk_file_name(cpk_stem)));
    let context = CompileContext { version };
    let mut tasks = report.manifest.tasks.into_iter().enumerate().peekable();
    for source in &sources {
        // Opened once for all of the export's tasks, which the manifest keeps together.
        let mut content = ContentSource::new(source, &budget);
        while let Some((index, task)) =
            tasks.next_if(|(_, task)| task.export_id == source.export_id)
        {
            // A `.7z`'s tasks share the permit its content source holds for the whole
            // decompressed archive; a task of another source is charged its own reads.
            let permit = if source.kind == SourceKind::SevenZ {
                None
            } else {
                Some(budget.acquire(task.charge)?)
            };
            let mut batch = task_batch(index, task, &mut content, &context);
            batch.permit = permit;
            for message in std::mem::take(&mut batch.messages) {
                events.message(message);
            }
            writer.submit(batch)?;
        }
        events.processed(source.export_id);
    }

    if writer.finish(version)? {
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

/// The batch of `task`, the manifest's task number `index`: its files read from `content`, then
/// processed. A file that cannot be read fails the task with `source_read_failed` naming it,
/// on the task's folder, and the task is not processed.
fn task_batch(
    index: usize,
    task: BuildTask,
    content: &mut ContentSource,
    ctx: &CompileContext,
) -> TaskBatch {
    // The coordinator reads, not the task: an archive is one sequential stream, so tasks
    // sharing it would only wait on each other, and processing needs no source handle.
    match read_files(&task, content) {
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

        let batch = task_batch(
            3,
            task,
            &mut ContentSource::new(&source, &MemoryBudget::new(1 << 30)),
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
