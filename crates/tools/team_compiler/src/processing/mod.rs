//! Stages 3 and 4, one task's work (`team_compiler/pipeline.md` "3. Per-model-folder parallel
//! steps", "4. Per-export non-model steps"): its source files, already read, converted and
//! packed into the CPK entries the task commits, whole or not at all.

mod kit;
mod model;
mod texture;

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use aesthetics_export::FileDescriptor;
use pes_version::PesVersion;
use pipeline::Permit;
use studio_core::{Disposition, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};
use crate::paths;
use crate::plan::{BuildTask, TaskKind};

/// The bytes of every file a task reads (`TaskKind::files`), keyed by the file's export path:
/// read from the export's source by the coordinator before the task is processed.
pub(crate) type TaskFiles = BTreeMap<ScopePath, Vec<u8>>;

/// What every task of a run reads besides its own content.
pub(crate) struct CompileContext {
    /// The target version, which decides the kit config's encoding and the portraits' file
    /// names.
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
    /// The manifest positions of the player folder's group the task belongs to
    /// (`TaskGroup::tasks`), its textures batch last: the writer holds the group's packages
    /// until that batch arrives and decides them all. `None` for a task the writer commits on
    /// its own. When `shared_texture_conflict` lands (step 4.5), the textures batch is where
    /// a field naming the losing package's position goes, for the decision to skip it.
    pub(crate) group: Option<Range<usize>>,
    /// A kit's config as a `UniformParameter.bin` entry, applied only when the batch is
    /// committed.
    pub(crate) uniparam: Option<Entry>,
    /// The task's findings.
    pub(crate) messages: Vec<Message>,
    /// The memory the task was charged, released once the writer has its entries. A `.7z`
    /// export's tasks share one, its whole decompressed size, released with the last of them.
    pub(crate) permit: Option<Arc<Permit>>,
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
        TaskKind::Models {
            folder,
            package,
            ids,
        } => model::package(folder, *package, ids, task.team_id, &mut files)
            .map(|entries| (entries, None)),
        TaskKind::Textures { folder, .. } => {
            texture::folder_textures(folder, task.team_id, &mut files)
                .map(|entries| (entries, None))
        }
        // A DDS portrait is what the game reads: its bytes go out as they are.
        TaskKind::Portrait { player_id, file } => Ok((
            vec![(
                paths::portrait(ctx.version, *player_id),
                take(&mut files, file),
            )],
            None,
        )),
        TaskKind::Kit { slot, kit } => kit::kit(*slot, kit, task.team_id, ctx.version, &mut files)
            .map(|(entries, config)| (entries, Some(config))),
    };
    let mut batch = TaskBatch {
        index,
        entries: Vec::new(),
        group: task.group.as_ref().map(|group| group.tasks.clone()),
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

    use aesthetics_export::{KitFolder, KitLayout, KitTexture, KitTextureSource};
    use fmdl::FmdlFile;
    use fmdl::ops::paths::texture_paths;
    use fpk::{FpkFile, FpkKind};
    use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};
    use studio_core::{ExportId, Severity};

    use super::*;
    use crate::paths::TextureHome;
    use crate::plan::subset::ModelPackage;
    use crate::plan::{ModelFolder, TaskGroup};
    use crate::templates;

    const PLAYER: &str = "Players/05 - The Chad Stormworks Player";
    const COMMON: &str = "Asset/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/#windx11/shirt.ftex";
    const COMMON_DIRECTORY: &str =
        "/Assets/pes16/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/";

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

    /// The tracer's player folder holding the files `names`, its textures going to its common
    /// subfolder.
    fn player(names: &[&str]) -> ModelFolder {
        ModelFolder {
            path: ScopePath::new(PLAYER).unwrap(),
            files: names
                .iter()
                .map(|name| file(&format!("{PLAYER}/{name}")))
                .collect(),
            textures: TextureHome::PlayerCommon {
                folder_name: "05 - The Chad Stormworks Player".to_owned(),
            },
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

    /// `kind` processed for PES 21 as task 3 of export 4, over its files read from the tracer
    /// folder.
    fn run(kind: TaskKind) -> TaskBatch {
        run_for(kind, PesVersion::Pes21)
    }

    /// `run` for the target `version`, the task in no group.
    fn run_for(kind: TaskKind, version: PesVersion) -> TaskBatch {
        run_task(kind, version, None)
    }

    /// `kind` processed for `version` as task 3 of export 4, in `group`, over its files read
    /// from the tracer folder.
    fn run_task(kind: TaskKind, version: PesVersion, group: Option<TaskGroup>) -> TaskBatch {
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
            group,
        };
        process_task(3, task, files, &CompileContext { version })
    }

    fn paths(batch: &TaskBatch) -> Vec<&str> {
        batch
            .entries
            .iter()
            .map(|(path, _)| path.as_str())
            .collect()
    }

    /// The tracer's player folder, every file the lead's fixture holds.
    fn whole_player() -> ModelFolder {
        player(&[
            "boots.fmdl",
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair.skl",
            "fcl_hair_sim.fclo",
            "glove_l.fmdl",
            "glove_r.fmdl",
            "shirt.dds",
        ])
    }

    /// The (file name, directory) of every texture path of the package's model `name`.
    fn texture_directories(package: &FpkFile, name: &str) -> Vec<(String, String)> {
        let model = FmdlFile::read(package.get(name).unwrap()).unwrap();
        texture_paths(&model)
            .unwrap()
            .into_iter()
            .map(|path| (path.file_name, path.directory))
            .collect()
    }

    /// Asserts the model's own texture points at the player's common folder and the game's
    /// own at the team's, with no `000` placeholder left.
    fn assert_rewritten(directories: &[(String, String)]) {
        assert!(
            directories.contains(&("shirt.dds".to_owned(), COMMON_DIRECTORY.to_owned())),
            "{directories:?}"
        );
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
    fn a_face_package_is_packed_once_per_slot_without_the_textures() {
        let batch = run(TaskKind::Models {
            folder: whole_player(),
            package: ModelPackage::Face,
            ids: vec![79205, 79207],
        });

        assert_eq!(batch.index, 3);
        assert!(batch.messages.is_empty() && batch.uniparam.is_none());
        assert_eq!(batch.group, None);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/face/real/79205/#Win/face.fpk",
                "Asset/model/character/face/real/79205/#Win/face.fpkd",
                "Asset/model/character/face/real/79207/#Win/face.fpk",
                "Asset/model/character/face/real/79207/#Win/face.fpkd",
            ]
        );
        assert_eq!(batch.entries[0].1, batch.entries[2].1, "one package, twice");
        let fpkd = FpkFile::read(&batch.entries[1].1).unwrap();
        assert_eq!((fpkd.kind(), fpkd.len()), (FpkKind::Fpkd, 0));

        let package = FpkFile::read(&batch.entries[0].1).unwrap();
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
        assert_rewritten(&texture_directories(&package, "fcl_hair.fmdl"));
    }

    #[test]
    fn a_boots_package_holds_the_model_and_the_standard_skeleton_when_it_brings_none() {
        let batch = run(TaskKind::Models {
            folder: whole_player(),
            package: ModelPackage::Boots,
            ids: vec![3745],
        });

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/boots/k3745/#Win/boots.fpk",
                "Asset/model/character/boots/k3745/#Win/boots.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"]);
        assert_eq!(package.get("boots.skl").unwrap(), templates::BOOTS_SKELETON);
        assert_rewritten(&texture_directories(&package, "boots.fmdl"));
    }

    #[test]
    fn a_skeleton_named_after_the_boots_model_is_packed_in_the_standard_one_s_place() {
        // The tracer's hair skeleton stands in for a custom boots skeleton: its bytes are a
        // real `.skl`, read as they are.
        let mut folder = player(&["fcl_hair.skl"]);
        let kit_boots = ScopePath::new(&format!("{PLAYER}/kit_boots.fmdl")).unwrap();
        let kit_boots_skl = ScopePath::new(&format!("{PLAYER}/kit_boots.skl")).unwrap();
        folder.files = vec![
            FileDescriptor {
                kind: aesthetics_export::classify(kit_boots.name()),
                path: kit_boots,
                ..file(&format!("{PLAYER}/boots.fmdl"))
            },
            FileDescriptor {
                kind: aesthetics_export::classify(kit_boots_skl.name()),
                path: kit_boots_skl,
                ..file(&format!("{PLAYER}/fcl_hair.skl"))
            },
        ];
        let custom = std::fs::read(tracer().join(format!("{PLAYER}/fcl_hair.skl"))).unwrap();

        let batch = run(TaskKind::Models {
            folder,
            package: ModelPackage::Boots,
            ids: vec![3745, 3747],
        });

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/boots/k3745/#Win/boots.fpk",
                "Asset/model/character/boots/k3745/#Win/boots.fpkd",
                "Asset/model/character/boots/k3747/#Win/boots.fpk",
                "Asset/model/character/boots/k3747/#Win/boots.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"]);
        assert_eq!(package.get("boots.skl").unwrap(), custom);
    }

    #[test]
    fn a_gloves_package_holds_both_hands_under_their_allowed_names() {
        let batch = run(TaskKind::Models {
            folder: whole_player(),
            package: ModelPackage::Gloves,
            ids: vec![3745],
        });

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/glove/g3745/#Win/glove.fpk",
                "Asset/model/character/glove/g3745/#Win/glove.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["glove_l.fmdl", "glove_r.fmdl"]);
        let fpkd = FpkFile::read(&batch.entries[1].1).unwrap();
        assert_eq!((fpkd.kind(), fpkd.len()), (FpkKind::Fpkd, 0));
    }

    #[test]
    fn a_shared_gloves_folder_s_package_and_textures_go_under_its_shared_id() {
        // The tracer's boots model stands in for a left glove: a real FMDL naming `shirt.dds`.
        let folder_path = ScopePath::new("Gloves/Grip").unwrap();
        let glove = ScopePath::new("Gloves/Grip/glove_l.fmdl").unwrap();
        let shirt = ScopePath::new("Gloves/Grip/shirt.dds").unwrap();
        let folder = ModelFolder {
            path: folder_path,
            files: vec![
                FileDescriptor {
                    kind: aesthetics_export::classify(glove.name()),
                    path: glove,
                    ..file(&format!("{PLAYER}/boots.fmdl"))
                },
                FileDescriptor {
                    kind: aesthetics_export::classify(shirt.name()),
                    path: shirt,
                    ..file(&format!("{PLAYER}/shirt.dds"))
                },
            ],
            textures: TextureHome::SharedOutput {
                package: ModelPackage::Gloves,
                id: 644,
            },
        };

        let package = run(TaskKind::Models {
            folder: folder.clone(),
            package: ModelPackage::Gloves,
            ids: vec![644],
        });
        let textures = run(TaskKind::Textures { folder });

        assert!(package.messages.is_empty(), "{:?}", package.messages);
        assert_eq!(
            paths(&package),
            [
                "Asset/model/character/glove/g0644/#Win/glove.fpk",
                "Asset/model/character/glove/g0644/#Win/glove.fpkd",
            ]
        );
        let fpk = FpkFile::read(&package.entries[0].1).unwrap();
        let names: Vec<&str> = fpk.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["glove_l.fmdl"]);
        let directories = texture_directories(&fpk, "glove_l.fmdl");
        assert!(
            directories.contains(&(
                "shirt.dds".to_owned(),
                "/Assets/pes16/model/character/glove/g0644/".to_owned()
            )),
            "{directories:?}"
        );
        assert!(textures.messages.is_empty(), "{:?}", textures.messages);
        assert_eq!(
            paths(&textures),
            ["Asset/model/character/glove/g0644/#windx11/shirt.ftex"]
        );
    }

    #[test]
    fn a_folder_s_textures_are_converted_once_into_its_common_folder_for_its_packages() {
        let batch = run_task(
            TaskKind::Textures {
                folder: whole_player(),
            },
            PesVersion::Pes21,
            Some(TaskGroup {
                tasks: 0..4,
                charge: 0,
            }),
        );

        assert!(batch.messages.is_empty() && batch.uniparam.is_none());
        assert_eq!(
            batch.group,
            Some(0..4),
            "the batch carries its task's group"
        );
        assert_eq!(paths(&batch), [COMMON]);
        let dds = std::fs::read(tracer().join(format!("{PLAYER}/shirt.dds"))).unwrap();
        assert_eq!(
            batch.entries[0].1,
            ftex::dds_to_ftex(&dds, ftex::ColorSpace::Normal).unwrap()
        );
    }

    #[test]
    fn a_portrait_is_emitted_as_it_is_under_the_version_s_name() {
        let file = file(&format!("{PLAYER}/portrait.dds"));
        let source = std::fs::read(tracer().join(file.source.as_str())).unwrap();
        assert!(source.starts_with(b"DDS "), "the fixture is a DDS");
        for (version, path) in [
            (PesVersion::Pes21, "common/render/symbol/player/79205.dds"),
            (
                PesVersion::Pes18,
                "common/render/symbol/player/player_79205.dds",
            ),
        ] {
            let batch = run_for(
                TaskKind::Portrait {
                    player_id: 79205,
                    file: file.clone(),
                },
                version,
            );

            assert!(batch.messages.is_empty() && batch.uniparam.is_none());
            assert_eq!(paths(&batch), [path], "{version}");
            assert_eq!(batch.entries[0].1, source, "{version}");
        }
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

        let batch = run(TaskKind::Models {
            folder,
            package: ModelPackage::Face,
            ids: vec![79205],
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
