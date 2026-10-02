//! Stages 3 and 4, one task's work (`team_compiler/pipeline.md` "3. Per-model-folder parallel
//! steps", "4. Per-export non-model steps"): its source files, already read, converted and
//! packed into the CPK entries the task commits, whole or not at all.

mod kit;
mod model;
mod texture;

use std::collections::BTreeMap;

use aesthetics_export::FileDescriptor;
use pes_version::PesVersion;
use pipeline::Permit;
use studio_core::{Disposition, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};
use crate::plan::{BuildTask, TaskKind};

/// The bytes of every file a task reads (`TaskKind::files`), keyed by the file's export path:
/// read from the export's source by the coordinator before the task is processed.
pub(crate) type TaskFiles = BTreeMap<ScopePath, Vec<u8>>;

/// What every task of a run reads besides its own content.
pub(crate) struct CompileContext {
    /// The target version, which decides the kit config's encoding.
    pub(crate) version: PesVersion,
}

/// One file in a container: its path in a CPK, or its name in a bin, and its bytes.
pub(crate) type Entry = (String, Vec<u8>);

/// One task's result, handed to the writer.
pub(crate) struct TaskBatch {
    /// The task's position in the manifest: the writer commits batches in this order.
    pub(crate) index: usize,
    /// The CPK entries the task commits; empty when the task failed.
    pub(crate) entries: Vec<Entry>,
    /// A kit's config as a `UniformParameter.bin` entry, applied only when the batch is
    /// committed.
    pub(crate) uniparam: Option<Entry>,
    /// The task's findings.
    pub(crate) messages: Vec<Message>,
    /// The memory the task was charged, released once the writer has its entries.
    pub(crate) permit: Option<Permit>,
}

/// Runs `task`, the manifest's task number `index`, over `files`, the bytes of every file it
/// reads. A task that fails commits nothing: its batch has no entries and reports
/// `folder_pack_failed` on its folder, which is left out of the CPK.
pub(crate) fn process_task(
    index: usize,
    task: BuildTask,
    mut files: TaskFiles,
    ctx: &CompileContext,
) -> TaskBatch {
    let result = match &task.kind {
        TaskKind::Face { folder, player_ids } => {
            model::face(folder, player_ids, task.team_id, &mut files).map(|entries| (entries, None))
        }
        TaskKind::Kit { slot, kit } => kit::kit(*slot, kit, task.team_id, ctx.version, &mut files)
            .map(|(entries, config)| (entries, Some(config))),
    };
    let mut batch = TaskBatch {
        index,
        entries: Vec::new(),
        uniparam: None,
        messages: Vec::new(),
        permit: None,
    };
    match result {
        Ok((entries, uniparam)) => {
            batch.entries = entries;
            batch.uniparam = uniparam;
        }
        Err(error) => batch.messages.push(tool_message(
            Code::FolderPackFailed,
            Scope::Folder {
                export_id: task.export_id,
                path: task.kind.folder_path(),
            },
            Disposition::DropFolder,
            vec![("error", format!("{error:#}"))],
        )),
    }
    batch
}

/// The bytes of `file`, taken out of the task's `files`.
fn take(files: &mut TaskFiles, file: &FileDescriptor) -> Vec<u8> {
    files
        .remove(&file.path)
        .expect("the coordinator reads every file `TaskKind::files` lists, each once")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use aesthetics_export::{KitFolder, KitLayout, KitTexture, KitTextureSource, PlayerFolder};
    use fmdl::FmdlFile;
    use fmdl::ops::paths::texture_paths;
    use fpk::{FpkFile, FpkKind};
    use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};
    use studio_core::{ExportId, Severity};

    use super::*;

    const PLAYER: &str = "Players/05 - The Chad Stormworks Player";
    const COMMON: &str = "Asset/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/#windx11/shirt.ftex";

    /// The tracer fixture's export folder.
    fn tracer() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tracer/studio/egg Tracer")
    }

    /// The export file at `relative`, described as the structure pass would.
    fn file(relative: &str) -> FileDescriptor {
        let path = ScopePath::new(relative).unwrap();
        FileDescriptor {
            size: std::fs::metadata(tracer().join(relative)).map_or(0, |data| data.len()),
            kind: aesthetics_export::classify(path.name()),
            source: path.clone(),
            path,
        }
    }

    fn player(names: &[&str]) -> PlayerFolder {
        PlayerFolder {
            path: ScopePath::new(PLAYER).unwrap(),
            player_name: "The Chad Stormworks Player".to_owned(),
            files: names
                .iter()
                .map(|name| file(&format!("{PLAYER}/{name}")))
                .collect(),
            links: Vec::new(),
            ingame_face: false,
            fpc: None,
            portrait: None,
            settings: None,
        }
    }

    fn kit(config: bool, layout: Option<KitLayout>) -> KitFolder {
        KitFolder {
            path: ScopePath::new("Kits/g1").unwrap(),
            label: None,
            config: config.then(|| file("Kits/g1/config.toml")),
            colors: None,
            icon: Some(11),
            layout,
            textures: vec![KitTexture {
                stem: "kit".to_owned(),
                file: file("Kits/g1/kit.dds"),
                source: KitTextureSource::Own,
            }],
        }
    }

    /// `kind` processed as task 3 of export 4, over its files read from the tracer folder.
    fn run(kind: TaskKind) -> TaskBatch {
        let files = kind
            .files()
            .into_iter()
            .map(|file| {
                let bytes = std::fs::read(tracer().join(file.source.as_str())).unwrap();
                (file.path.clone(), bytes)
            })
            .collect();
        let task = BuildTask {
            export_id: ExportId(4),
            team_id: 792,
            kind,
            charge: 0,
        };
        process_task(
            3,
            task,
            files,
            &CompileContext {
                version: PesVersion::Pes21,
            },
        )
    }

    fn paths(batch: &TaskBatch) -> Vec<&str> {
        batch
            .entries
            .iter()
            .map(|(path, _)| path.as_str())
            .collect()
    }

    #[test]
    fn a_folder_mapped_to_two_slots_converts_its_textures_once_and_packs_a_face_per_slot() {
        let folder = player(&[
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair.skl",
            "fcl_hair_sim.fclo",
            "shirt.dds",
        ]);

        let batch = run(TaskKind::Face {
            folder,
            player_ids: vec![79205, 79207],
        });

        assert_eq!(batch.index, 3);
        assert!(batch.messages.is_empty() && batch.uniparam.is_none());
        assert_eq!(
            paths(&batch),
            [
                COMMON,
                "Asset/model/character/face/real/79205/#Win/face.fpk",
                "Asset/model/character/face/real/79205/#Win/face.fpkd",
                "Asset/model/character/face/real/79207/#Win/face.fpk",
                "Asset/model/character/face/real/79207/#Win/face.fpkd",
            ]
        );
        assert_eq!(batch.entries[1].1, batch.entries[3].1, "one package, twice");
        let fpkd = FpkFile::read(&batch.entries[2].1).unwrap();
        assert_eq!((fpkd.kind(), fpkd.len()), (FpkKind::Fpkd, 0));

        let package = FpkFile::read(&batch.entries[1].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(
            names,
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        let model = FmdlFile::read(package.get("fcl_hair.fmdl").unwrap()).unwrap();
        let directories: Vec<(String, String)> = texture_paths(&model)
            .unwrap()
            .into_iter()
            .map(|path| (path.file_name, path.directory))
            .collect();
        assert!(directories.contains(&(
            "shirt.dds".to_owned(),
            "/Assets/pes16/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/"
                .to_owned()
        )));
        assert!(
            directories.iter().any(|(_, directory)| directory
                == "/Assets/pes16/model/character/common/792/sourceimages/"),
            "{directories:?}"
        );
        assert!(
            directories
                .iter()
                .all(|(_, directory)| !directory.contains("/000/")),
            "{directories:?}"
        );
    }

    #[test]
    fn a_kit_commits_its_texture_and_its_config_as_entry_and_bin_entry() {
        let batch = run(TaskKind::Kit {
            slot: KitSlot::G1,
            kit: kit(true, Some(KitLayout::Fox)),
        });

        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/uniform/texture/#windx11/u0792g1.ftex",
                "common/character0/model/character/uniform/team/792/792_DEF_GK1st_realUni.bin",
            ]
        );
        let (name, config) = batch.uniparam.as_ref().unwrap();
        assert_eq!(name, "792_DEF_GK1st_realUni.bin");
        assert_eq!(config, &batch.entries[1].1);
    }

    #[test]
    fn a_kit_without_config_toml_is_encoded_from_the_template() {
        let batch = run(TaskKind::Kit {
            slot: KitSlot::G1,
            kit: kit(false, None),
        });

        let presence = TexturePresence {
            kit: true,
            ..TexturePresence::default()
        };
        let names = texture_names(792, KitSlot::G1, presence);
        let template = KitConfig::template().encode_with_names(PesVersion::Pes21, &names);
        assert_eq!(batch.uniparam.unwrap().1, template);
    }

    #[test]
    fn a_model_that_cannot_be_read_fails_its_task() {
        // `face_diff.bin`'s bytes under a face model's name: a file with its Phase 3 role
        // whose content cannot be read as an FMDL.
        let mut folder = player(&["face_diff.bin"]);
        let model = ScopePath::new(&format!("{PLAYER}/face_high.fmdl")).unwrap();
        folder.files.push(FileDescriptor {
            kind: aesthetics_export::classify(model.name()),
            path: model,
            ..file(&format!("{PLAYER}/face_diff.bin"))
        });

        let batch = run(TaskKind::Face {
            folder,
            player_ids: vec![79205],
        });

        assert!(batch.entries.is_empty() && batch.uniparam.is_none());
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(message.code.code, "folder_pack_failed");
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
    }
}
